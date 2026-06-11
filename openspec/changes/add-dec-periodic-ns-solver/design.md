# Design: add-dec-periodic-ns-solver

## Context

Stage 0 delivered the complete operator substrate, all tested at
f32/f64/Float106:

- `deep_causality_topology`: `Manifold::{exterior_derivative, codifferential,
  hodge_star, laplacian, wedge, interior_product, de_rham,
  de_rham_from_integrals, sharp, leray_project, leray_project_opts}` on
  `LatticeComplex<D, R>` with `CubicalReggeGeometry` metric;
  `LerayProjection<R>` carrying `(projected, potential)`;
  `HodgeDecomposeOptions<R>` (tolerance, max_iterations). The conventions
  are pinned by tests: `Δ_dR = −∇²` on a flat torus, cup ordering by exact
  Leibniz, de Rham edge orientation by the FTC test.
- `deep_causality_physics`: `quantities::fluid_dynamics::{VelocityOneForm,
  VorticityTwoForm, PressureZeroForm, BodyForceOneForm, SolenoidalField}`.
  `VelocityOneForm` carries exactly `Clone + Add + Mul<R>`; `SolenoidalField`
  is constructible only via `from_leray_projection` / `from_hodge_projection`
  and exposes `as_one_form()`.
- `deep_causality_calculus`: `Rk4::new(dt, rate)` is an
  `Arrow<In = S, Out = S>` for `S: Clone + Add + Mul<R>`;
  `EndoArrow::{iterate_n, iterate_until}` from `deep_causality_haft`.
- The Stage 0 capstone test already cross-validates the assembled DEC RHS
  against the pointwise kernel + tangent-functor oracle at second order
  (`tests/theories/fluid_dynamics/dec_cross_validation_tests.rs`).

What is missing is the march: nothing composes these into a time-stepping
solver. This change is deliberately a composition exercise — the strategic
value (challenge entry, paper seed) lies in the solver core being a short
chain of library calls.

## Goals / Non-Goals

**Goals:**

- Incompressible NS on periodic lattices, 2D and 3D, velocity as an edge
  1-form throughout, precision-generic over `R: RealField`.
- Rotational-form RHS under projection: the marched rate is
  `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)` — the projector inside the rate, exactly
  the governing equation of `cfd-gap.md` §2; CFL guard each step.
- The type-state contract enforced end to end: the public step maps
  `SolenoidalField → SolenoidalField`; an unprojected field cannot be
  marched.
- Diagnostics sufficient for the validation ladder: kinetic energy,
  enstrophy, helicity, max speed, discrete divergence, opt-in pressure
  recovery (both conventions).
- Validation ladder items 4–6 and 8 of `cfd-gap.md` §7 in CI with
  convergence-order assertions; item 7 (Re-1600) as an example program.

**Non-Goals:**

- Wall boundary conditions (G5 — Stage 3), implicit time stepping,
  turbulence closures, CG preconditioning, GPU, performance competition.
- New features in topology, calculus, haft, or num — the stage exists to
  prove their published APIs compose. Upstream *defects* found during
  assembly are not a non-goal: this is a monorepo, so a discovered bug is
  fixed immediately at its source (with its own test), as part of this
  change's work.

## Decisions

### D1 — Home: `deep_causality_physics::theories::fluid_dynamics::dec`

The solver is a discretization of the incompressible-NS governing theory,
so it lives beside the pointwise regime evaluators in
`src/theories/fluid_dynamics/`, as a folder module `dec/` (one type per
file per the crate taxonomy). Topology stays domain-agnostic (decision 1 of
the gap note); the physics crate is where domain semantics (ν, body force,
CFL) belong. Kernels (`src/kernels/fluids/`) remain pointwise; the DEC
solver is not a kernel.

### D2 — `deep_causality_calculus` becomes a runtime dependency of physics

`Rk4` is the integrator of record (gap note §3.3); the march is library
code, so the dependency moves from `[dev-dependencies]` to
`[dependencies]`. Tier check: calculus is Tier 1 (haft, num), physics is
Tier 4 — the edge is acyclic. AGENTS.md's dependency table gains
`deep_causality_calculus` in the physics row. The alternative — hand-rolling
RK4 inside physics — was rejected: it duplicates a published operator and
destroys the composition story the stage exists to prove.

### D3 — Rate surfaces: infallible unprojected assembly, fallible projected rate

`Rk4<S, R, F>` requires an infallible `rate: Fn(&S) -> S`. The rate
therefore has two surfaces. The **unprojected assembly**
(`eval_unprojected`) is infallible: the DEC operators' only failure modes
for a fixed manifold are dimension mismatch and missing metric, both
excluded once at construction, so its internal `Result`s unwrap with
`expect` against construction-time invariants (precedent:
`VelocityOneForm::add`). The **projected rate** (`eval_projected`) adds
one CG solve and returns `Result` — CG convergence is data-dependent and
must not panic. Inside the `Rk4` closure a stage's CG failure is parked in
a `Cell<Option<PhysicsError>>` and the stage yields a zero rate; the step
checks the slot immediately after the arrow run and short-circuits with
the recorded error. This keeps the literal `Rk4` composition and the
fallible plumbing in `Result`.

### D4 — The projector lives inside the rate (revised during implementation)

**Implementation finding.** The first build placed one projection *after*
the RK4 step (Chorin splitting). The validation ladder falsified it: at
`ν = 0` the march lost 5–20% of its energy over `T = 10`, and the loss
halved with `dt` — the textbook first-order splitting dissipation (the
unprojected stages grow an O(1) gradient component whose energy the
post-step projection discards). The fix is the formulation the gap note's
§2 equation states: march `∂u♭/∂t = P(rhs(u♭))`, the exact ODE on the
divergence-free subspace, with **no splitting error at all** — each RK4
stage evaluates the projected rate (one CG per stage, four per step).
After the fix the inviscid energy drift collapsed to the spatial-residue
level and the Taylor–Green envelope error became cleanly second order.

```text
SolenoidalField ──as_one_form──► VelocityOneForm
    ──Rk4::run over P∘rhs──► VelocityOneForm  (projected stages; CG error deferred)
    ──leray_project──► SolenoidalField + φ    (type-state re-entry; near-free)
    ──cfl_check──► SolenoidalField            (fallible bind: violation short-circuits)
```

The RK4 combination of divergence-free increments is divergence-free, so
the re-entry projection — kept because projection is `SolenoidalField`'s
only construction path — receives an already-solenoidal field and its CG
terminates almost immediately. Cost: ~4 full CG solves per step instead
of 1; the structure-preservation is what the stage exists to demonstrate,
so the cost is taken. Initial conditions enter once through
`de_rham`/`de_rham_from_integrals` into `VelocityOneForm`, then through
one `t = 0` projection (the sampled analytic field is divergence-free
analytically, not discretely).

### D5 — Solver type and configuration

`DecNsSolver<'m, const D: usize, R>` borrows the manifold and owns the
physics: `nu: KinematicViscosity`-equivalent scalar, `dt`, optional
`BodyForceOneForm`, `HodgeDecomposeOptions` for the projection CG, and two
CFL safety factors (advective and diffusive, defaults 0.9). Methods:
`step(&SolenoidalField) -> Result<StepOutput>` (the bind chain),
`run_n(state, n)` and `run_until(state, predicate, max_steps)` (the
`EndoArrow` modes lifted over the fallible step — a plain loop carrying
`Result`, since `iterate_until` itself is infallible by signature).
`StepOutput` carries the new state and the per-step diagnostics needed for
free (max speed, divergence residual) so callers do not recompute them.

### D6 — CFL guard definition

After projection, `sharp` recovers pointwise vectors; `max |u|` and the
minimum edge length `dx_min` (from the Regge geometry's per-edge lengths)
give the two limits:

- advective: `dt ≤ C_adv · dx_min / max|u|` (skipped while `max|u| = 0`),
- diffusive: `dt ≤ C_diff · dx_min² / (2·D·ν)` (skipped when `ν = 0`).

Violation returns a dedicated error carrying both the limit and the actual
`dt`. This is the embryonic 10.1 corrective bind of `causal_cfd.md` §10;
adaptive `dt` is a follow-up, not this change.

### D7 — Pressure recovery costs one opt-in solve

`pressure_diagnostic(&SolenoidalField)` evaluates the unprojected RHS at
the current state and Leray-projects it (one CG solve, only when called),
returning both conventions as `PressureZeroForm`s: Bernoulli
(`p + ½|u|²` **is** the grade-0 potential at ρ = 1 — the true dynamics is
`∂u/∂t = rhs_unproj − ∇B`, so `(I − P)rhs = +∇B`) and static (Bernoulli
minus the kinetic 0-form assembled from `sharp` magnitudes). Resolves
gap-note open question 3 by emitting the pair. The per-step stage
projections discard exactly this gradient, but their potentials belong to
intermediate stage states; the diagnostic evaluates at the *current*
state, which is why it remains a separate opt-in solve.

### D8 — Diagnostic definitions (DEC-native, no new operators)

- Kinetic energy `E = ½ Σ_e u_e · (⋆u)_e` — the discrete `½∫ u♭ ∧ ⋆u♭`
  through the diagonal star.
- Enstrophy `Z = ½ Σ_f ω_f · (⋆ω)_f` with `ω = d u♭`.
- Helicity (3D only) `H = Σ_c (u♭ ∧ du♭)_c` — the wedge gives the top-form
  cochain whose coefficients are already cell integrals; requesting it on a
  2D manifold is an error.
- Max speed: `sharp`, then the max Euclidean vertex-vector norm.
- Divergence residual: `‖δ u♭‖_∞` — the projection-exactness witness.

### D9 — Causal-monad wrapper in the existing tradition

A `wrappers.rs` beside the solver exposes the step as
`PropagatingEffect`-returning functions (`Ok → pure`, `Err →
from_error(CausalityError::from(e))`), exactly like the existing kernel
wrappers. The solver core stays `Result`-based; the wrapper is the
integration point with the causal monad, not the engine.

### D10 — Validation tiers: CI tests vs. example program

CI carries the analytic rungs at small grids: 2D Taylor–Green decay with a
convergence table over `[8, 16, 32]` (observed spatial order ≥ 1.9 at f64;
f32 asserted at looser tolerance; Float106 at the f64 gate), the 2D-in-3D
Taylor–Green on `cubic_torus` (same envelope, all 3D operator paths), and
inviscid energy/helicity drift bounds at `ν = 0`.

CI also carries the double shear layer (§7 item 8, the Brown–Minion / thin
shear layer roll-up case) at one modest 2D resolution and f64 only — it is
a physics-behavior rung, not a precision rung. It has no closed form, so
its gates are structural rather than analytic: (a) **roll-up witness** —
the cross-stream kinetic energy, seeded at the small perturbation
amplitude, grows by at least an order of magnitude before the horizon;
(b) **2D conservation character** — at `ν > 0`, kinetic energy and
enstrophy are both monotonically non-increasing within a documented
tolerance (no vortex stretching in 2D), and every sampled state stays
divergence-free at projection tolerance; (c) **coherent-structure tap** —
the existing Q-criterion kernel, fed by a test-side central-difference
gradient of the `sharp`-recovered field, reports vortex-core (positive-Q)
cells in the rolled-up state that are absent at `t = 0`. This rung is what
connects the solver to the diagnostic vocabulary in
`kernels/fluids/` and pre-stages the Stage 2 analysis tap. The Re-1600 3D run at
64³–128³ is minutes-to-hours of CG work — it ships as an example binary
in `examples/avionics_examples/` — the home of the existing
`cfd_taylor_green` harness, which this example extends to the DEC solver —
with the resolution as a parameter, producing the dissipation-rate curve as
CSV on stdout; CI never runs it.

## Risks / Trade-offs

- **CG cost without preconditioning** dominates large-grid steps; accepted
  for the prototype (gap note §8), logged as the first performance
  follow-up. CI grids are chosen small enough that the suite stays fast.
- **Four CG solves per step** (projected stages, D4) instead of the
  Chorin placement's one: the price of marching the exact projected
  dynamics. Accepted — the splitting alternative measurably destroys the
  inviscid invariants the stage exists to demonstrate; preconditioning
  remains the first performance follow-up.
- **`expect` in the unprojected assembly** (D3) trades a theoretical panic
  for composition with the published `Rk4`. The invariants are
  construction-checked and the panic paths are documented coverage
  exemptions, consistent with the crate's existing precedent. The
  data-dependent failure (CG) is `Result`-carried via the deferred slot,
  never a panic.
- **Helicity conservation is measured, not assumed**: the
  rotational-form/DEC combination is the structure-preserving choice in the
  literature (MHS 2016), but the inviscid test asserts bounded drift rather
  than exact conservation, with the bound recorded in the test.
- **f32 at the extremes**: the inviscid-invariant and convergence gates use
  precision-dependent tolerances (as Stage 0 did); if f32 cannot hold a
  gate at the smallest grids, the test documents the floor rather than
  weakening f64.
