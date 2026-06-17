# CFD Gap Note: Closing the Solver Gap with the Uniform Math Already in Place

Status: foundation note for a follow-up specification. Decisions resolved 2026-06-10.
Related: `openspec/notes/arrow/causal-arrow-generalization.md` (the Arrow algebra this note builds on).

## 0. Resolved decisions

These were the open questions of the first draft; all are now decided and the note is
written against them.

1. **Sampled-field operators live in `deep_causality_topology`**, formulated
   domain-agnostically. `deep_causality_physics` layers unit-bearing types on top, in
   its existing tradition: units encode invariants so that impossible states cannot be
   constructed.
2. **DEC-native throughout, from the start.** Velocity is an edge 1-form for the entire
   solve. No vertex-collocated prototype stage, no MVP reduction: the underlying
   mathematics and mechanics are already in place and need wiring, not rebuilding.
3. **Carriers are domain-specific unit types** in the physics crate (typed forms with
   trait constraints), not arithmetic impls bolted onto `CausalTensor` or `Manifold`.
4. **Pressure recovery is an opt-in diagnostic output.**
5. **The target is 3D fluid dynamics.** 2D cases serve as analytic validation rungs,
   not as the goal.

The community challenge details are not yet known (announced 2026-06-09/10); closing
these gaps is justified independently, since any subsequent CFD solver needs the same
substrate.

## 1. Goal and scope

Build an incompressible Navier–Stokes solver on periodic lattices in 2D and 3D,
DEC-native (velocity as an edge 1-form throughout), validated against analytic
Taylor–Green decay and published 3D reference data, assembled from the uniform math
already in the monorepo. Precision is a type parameter (f32 / f64 / Float106)
end to end.

Out of scope for the first milestone: wall boundary conditions (lid-driven cavity),
implicit time stepping, turbulence closures, GPU acceleration, performance competition
with established solvers.

## 2. The governing formulation

DEC-native incompressible NS in rotational (Lamb) form under Leray projection. With
`u♭` the velocity 1-form, `ω = d u♭` the vorticity 2-form, and `P` the Leray projector
(the divergence-free component of the Hodge decomposition):

```text
∂u♭/∂t = P( − i_u ω  +  ν Δ u♭  +  g♭ )
```

Three structural consequences:

- **Pressure vanishes from the equations.** In Lamb form the convective term splits as
  `(u·∇)u = ∇(|u|²/2) − u × ω`; the gradient part is exact (a `d` of a 0-form) and `P`
  annihilates it together with `∇p`. What survives projection is `P(−i_u ω)`.
- **Pressure recovery (opt-in diagnostic):** the gradient potential of the Hodge
  decomposition of the *unprojected* RHS is the Bernoulli pressure `p + ρ|u|²/2`; the
  static pressure follows by subtracting the kinetic 0-form. One already-computed
  component plus one pointwise subtraction — no extra solve. The spec must state which
  convention (static vs. Bernoulli) the diagnostic emits.
- **Incompressibility is exact by construction.** The projected field's discrete
  divergence is zero to the CG tolerance at every precision backend — not approximately
  zero by discretization luck. "We do not approximate incompressibility; the projector
  is the equation."

**The projector needs only half the decomposition.** `P(ω) = ω − dα` where
`α = d φ_α` and `Δ₀ φ_α = δω` — the grade-0 Poisson solve only, already gauge-fixed by
mean subtraction. The β-step (`Δ_{k+1} ψ_β = dω`) never runs in the solver core. Two
consequences: the projection costs **one** CG solve per evaluation, not two; and the
documented singularity of the β-step CG on periodic lattices with `β_k > 0` (Risk 1 of
the archived `add-hodge-decomposition`; see `3DCausalFluidDynamics.md` §7) is
**sidestepped entirely** for the solver. The spec should expose `leray_project` as its
own entry point implementing the half-decomposition rather than calling full
`hodge_decompose`. On a torus the harmonic component `h` (the mean flow, `β₁ = D`
dimensions of it) is divergence-free and correctly retained by `ω − dα`; its
conservation under the march is a free momentum-conservation test.

Literature anchor for the spec: Hirani, *Discrete Exterior Calculus* (Caltech, 2003)
for the operator definitions; Mohamed, Hirani & Samtaney, *Discrete exterior calculus
discretization of incompressible Navier–Stokes equations over surface simplicial
meshes* (JCP 312, 2016) for exactly this solver shape on simplicial meshes. The
cubical-lattice case here is structurally simpler (axis-aligned cup-product wedge).

## 3. What already exists

### 3.1 Topology: the discrete differential geometry (`deep_causality_topology`)

- **`LatticeComplex<const D, R>`** (`src/types/lattice_complex/`): regular cubical
  lattice, per-axis periodic or open boundaries, oriented `boundary` operator,
  per-grade cell iteration, `ChainComplex` impl, `square_torus`/`cubic_torus`/
  `hypercubic_torus` constructors. The 1-cells (links, `D × num_vertices` on a torus)
  make an edge-cochain velocity natively a staggered (MAC-like) representation — the
  structure conventional collocated codes retrofit to suppress checkerboard pressure
  modes falls out of the complex itself.
- **`CubicalReggeGeometry<D, R, S>`** implements `HasHodgeStar` and carries per-edge
  lengths; its star weights primal/dual volumes from those lengths
  (`has_hodge_star.rs`, `volumes.rs`). **Physical grid spacing is therefore already
  handled**: `d` stays metric-free (as it must), and all `dx` scaling enters through
  `⋆` — confirmed in the audit, not a gap.
- **`Manifold<K, T>`** with the DEC operator set: `exterior_derivative(k)`,
  `codifferential(k)`, `hodge_star(k)`, `laplacian(k)` (the Hodge–de Rham Laplacian
  `Δ = dδ + δd`).
- **`hodge_decompose(field, k)`**: discrete Hodge–Helmholtz decomposition
  `ω = dα + δβ + h` via two Poisson solves through the matrix-free
  `deep_causality_sparse::cg_solve`, gauge-fixed, tolerance clamped per precision
  backend, module doc sized for "the downstream fluid pipeline (≤ 256³)". The
  `δβ + h` part **is** the Leray projection of a 1-form.
- **Neighborhoods + CoMonad** (`VonNeumann`/`Moore`/`KRing`, `extend` on
  `ManifoldWitness`): the lawful stencil machinery, demonstrated in the
  `cubical_heat_diffusion` example.
- **`DifferentialForm<T>`** (`src/types/differential_form/`): graded coefficient
  carrier over `CausalTensor` with degree/dim bookkeeping.

### 3.2 Physics: regime evaluators and the unit tradition (`deep_causality_physics`)

`src/theories/fluid_dynamics/` provides pointwise RHS kernels for all four regimes
(incompressible, compressible 3-equation system, Euler, Stokes), precision-generic,
with typed quantities (`Velocity3`, `VelocityGradient`, `Density`,
`KinematicViscosity`, `AccelerationVector`) whose constructors enforce physical
invariants. In the DEC-native solver these pointwise kernels are **not** the inner
loop; they become the independent cross-validation oracle (§6) and the model for the
typed-form layer (§5.3). `src/kernels/fluids/` additionally supplies the diagnostic
vocabulary (dimensionless groups, coherent structures, turbulence quantities).

### 3.3 Calculus: operators as arrows (`deep_causality_calculus`)

- Differentiation as the tangent functor (`DifferentiableField<N>`, nested `Dual`),
  used by the validation harness to manufacture exact derivatives of analytic fields.
- Integration as endomorphism iteration: `Rk4`/`Euler` implement
  `Arrow<In = S, Out = S>` for any `S: Clone + Add + Mul<R>` — `S` can be the whole
  1-form state. `EndoArrow` supplies `iterate_n` / `iterate_to_fixpoint` /
  `iterate_until` (fixed horizon, steady state, integrate-until-event).

### 3.4 HAFT and NUM: the algebra the new code must respect

- **`Arrow`** (value-level strong category: `compose`, `first`/`second`, `split`,
  `fanout`, fluent `arrow()` builder; concrete types, no `dyn`, total composition).
- **`CoMonad`** (`extract`/`extend`) — the home of stencils.
- **Iso tiers** (Tier 1/2 concrete in `deep_causality_num::iso`, Tier 3 `NaturalIso`
  in haft) with property-test helpers — the designated vocabulary for lawful
  representation changes (§5.2).
- **`RealField`/`Scalar`/`Float106`/`Dual`** — the precision-polymorphic scalar tower;
  nothing in the solver names a concrete float.

## 4. The gaps

Going DEC-native collapsed the first draft's stencil gap and shrank its iso gap; it
exposed one new operator gap. Four gaps remain, in dependency order.

### G1 — Wedge product and interior product on lattice cochains

*Status: **closed 2026-06-11** by `openspec/changes/add-dec-solver-foundations` — antisymmetrized cubical cup-product wedge (Leibniz exact at machine precision on arbitrary cochains; graded anticommutativity exact) and the star-derived interior product (all 2D/3D sign pins exact on constants; Cartan and convective cross-validation at second order).*

The convective term needs the contraction `i_u ω` of the vorticity 2-form with the
velocity. The topology crate has `d`, `δ`, `⋆`, `Δ`, and the Hodge decomposition — but
**no wedge product (`∧`) and no interior product (`i_X`)**; a code search confirms
neither exists. This is the solver-core gap and the only genuinely new mathematics.

It is bounded and has a published recipe: the discrete interior product is derived
from the wedge and the existing star, `i_X ω = (−1)^{k(n−k)} ⋆(⋆ω ∧ X♭)` (Hirani 2003,
§8), so the construction reduces to a primal cup-product wedge on the cubical lattice
— simpler than the simplicial case because cells are axis-aligned. The hardest part is
the primal–dual interpolation bookkeeping inside the wedge; this is the honest 20%
where DEC implementations earn their effort, and the spec should budget for it
explicitly.

### G2 — Musical isomorphisms (♭/♯), repositioned to the boundary of the computation

*Status: **closed 2026-06-11** — `Manifold::{de_rham, de_rham_from_integrals, sharp}` with the FTC orientation pin, exact constant round-trips, second-order smooth round-trips, and the `DeRhamSharpIso` Tier-2 witness in `extensions::iso_de_rham`.*

DEC-native removes ♭/♯ from the solve loop entirely. They remain necessary at the
edges of the computation:

- **Initial conditions**: the de Rham map — integrate the analytic velocity along each
  edge (`u♭(e) ≈ u·t̂ dx`, or exact line integrals where available) to seed the cochain.
- **Diagnostics and output**: ♯ to recover pointwise vectors for the CFL estimate
  (`max |u|`), energy/enstrophy reporting, and visualization.
- **Cross-validation** (§6): comparing the DEC RHS against the pointwise kernel RHS
  requires moving between representations lawfully.

### G3 — Typed-form carriers in the physics crate

*Status: **closed 2026-06-11** — `units::fluid_dynamics::{VelocityOneForm, VorticityTwoForm, PressureZeroForm, BodyForceOneForm, SolenoidalField}`; the type-state's unconstructibility (no public constructor, no arithmetic) is enforced by `compile_fail` doctests; the velocity carrier rides `Rk4` at f32/f64/Float106.*

Per decision 3, the algebraic operations the `Rk4` arrow requires (`Clone + Add +
Mul<R>`) live on domain-specific unit types, not on raw tensors. The spec should
define, in `deep_causality_physics`, typed wrappers over the topology carriers —
`VelocityOneForm<R>`, `VorticityTwoForm<R>`, `PressureZeroForm<R>`,
`BodyForceOneForm<R>` — whose constructors enforce the physical preconditions
(metric-bearing manifold, grade match, finite values), in the crate's existing
impossible-states-unconstructible tradition.

One invariant deserves type-state treatment: **divergence-freeness**. A
`ProjectedVelocityOneForm<R>` that only the Leray projector can construct makes "you
cannot time-step an unprojected field" a compile-time fact: the march step accepts only
the projected type, the RK4 output is unprojected by type, and the `bind` chain is the
only path back. The invariant the whole method rests on becomes unrepresentable to
violate — this is the strongest single demonstration of the units-as-invariants wisdom
the solver can offer, and it belongs in the challenge entry's narrative.

### G4 — Sign and orientation conventions, pinned and tested

*Status: **closed 2026-06-11** — `Δ_dR = −∇²` pinned by the exact-eigenvector sine test; cup ordering pinned by machine-precision Leibniz; de Rham orientation pinned by the FTC test.*

Three conventions must be fixed once, in the spec, with tests:

- The Hodge–de Rham Laplacian satisfies `Δ_{dR} = −∇²` on a flat torus; the viscous
  term is therefore `−ν Δ_{dR} u♭`. Getting this sign wrong produces anti-diffusion
  that a coarse Taylor–Green run can mask at small `ν`.
- Orientation conventions of `boundary` vs. the wedge's cup-product ordering must
  agree (the `(−1)^{k(n−k)}` factor in G1 depends on them).
- The de Rham map's edge-orientation convention must match `exterior_derivative`'s.

The MMS cross-validation in §6 is the systematic detector for all three.

### G5 — Wall boundary conditions (deferred; the one place new substrate is required)

Deferred not for lack of fit but for size: G5 alone is comparable to G1+G2+G3
combined, and it is purely additive — nothing in the periodic milestone needs rework
when it lands. The ecosystem already has the right sockets: per-axis periodicity
(`LatticeComplex::new(shape, periodic)` supports periodic-x/walled-y today),
neighborhoods that trim at open boundaries, a matrix-free CG that accepts any
boundary-aware operator as a closure, and identifiable boundary cells. What is missing
at those sockets is mathematical content in three layers:

1. **Boundary-corrected Hodge star**: dual cells are clipped at walls (halved at
   faces, quartered at edges); the Regge star's volume ratios must account for it.
2. **The decomposition changes theorem**: with boundary, the closed-manifold Hodge
   split becomes the Hodge–Morrey–Friedrichs decomposition with per-grade boundary
   conditions; the practical route is a single Neumann–Poisson projection solve (no
   flux through the wall) rather than full HMF, but it is a new solve path through
   the existing CG, not a parameter flip.
3. **No-slip in the viscous operator**: zeroing tangential wall edges is easy;
   consistent Laplacian boundary rows (ghost handling) is the standard labor.

Validation also changes character at this gap — from closed-form MMS proof to
reference-data comparison. The staging that preserves the analytic-first discipline:
**laminar Poiseuille channel flow first** (periodic x, walls y — constructible today,
flat aligned walls, no corners, exact parabolic steady state to validate against),
then the lid-driven cavity against the Ghia et al. (1982) centerline tables, with its
lid-corner singularity acknowledged as the standard nuisance.

### G6 — Harmonic-kernel handling in full `hodge_decompose` on periodic lattices

*Status: **closed 2026-06-11**, with a finding that supersedes this section's mechanism: the β-step's RHS `dω` is M-orthogonal to the harmonic kernel, so the consistent singular CG converges on tori without deflation — pinned by tests (2D/3D tori, mixed periodicity, 16×16 drift canary). `leray_project` ships as the one-solve half-decomposition entry point. Deflation remains the documented fallback (see the change's design D6).*

Surfaced by cross-reading `3DCausalFluidDynamics.md` §7: the β-step CG inside
`hodge_decompose` is singular on periodic-topology lattices where `β_k > 0` (Risk 1 of
the archived `add-hodge-decomposition`), and the solver's torus domains have
`β₁ = D`, `β₂ > 0`. Per §2, the **solver core is unaffected** — `leray_project` uses
only the gauge-fixed grade-0 solve. G6 blocks only the *full* decomposition on
periodic domains, which the diagnostic/causal-analysis tap (§10) and the
`FluidSignature` pipeline consume. Closure: deflate the harmonic kernel out of the
β-step solve (project the RHS and iterates against a harmonic basis, which the lattice
provides constructively on a torus), or solve in the orthogonal complement. Bounded
work in the existing CG path; needed before the §10 analysis tap runs on torus data,
not before the solver does.

## 5. Closing the gaps inside the existing algebras

The principle stands: each gap closes *inside* an algebra that already exists. The
solver is then a composition proof, which is the point of the exercise.

### 5.1 G1 closes in the DEC layer of the topology crate

Add to `Manifold<LatticeComplex<D, R>, _>` (domain-agnostic, per decision 1):

- `wedge(other, k, l) -> CausalTensor<R>` — cubical cup product of a k-form and an
  l-form, orientation-consistent with `boundary`;
- `interior_product(vector_form) -> CausalTensor<R>` — via `⋆(⋆ω ∧ X♭)` composed from
  the new wedge and the existing `hodge_star`, with the sign factor pinned by G4.

Acceptance: (a) Leibniz rule `d(α ∧ β) = dα ∧ β + (−1)^k α ∧ dβ` as a property test;
(b) Cartan's magic formula `L_X = i_X d + d i_X` checked against the analytic Lie
derivative of the sampled Taylor–Green field; (c) `i_u du♭` vs. the tangent-functor
evaluation of `∇(|u|²/2) − (u·∇)u` on the same analytic field — the two derivative
paths (exact via `Dual`, discrete via DEC) cross-validate with observed second-order
convergence.

### 5.2 G2 closes in the iso vocabulary

The de Rham map (vertex-vector → edge-cochain) and ♯ (edge-cochain → vertex-vector)
are a structure-preserving correspondence between two carriers of one object — encode
them with the Tier 1/2 iso traits and assert the round-trip and naturality laws via
`iso::test_support`. (Strictly the pair is an isomorphism only up to discretization
order; the property test asserts round-trip error of the expected order rather than
exactness, and the spec should say so.)

### 5.3 G3 closes in the physics crate's unit tradition

The typed forms of §4-G3, with `Add`/`Mul<R>` (and nothing more) so the state rides
`Rk4` directly. The type-state `ProjectedVelocityOneForm` is constructed exclusively by
the projection step (private-field newtype, constructor `pub(crate)` to the projection
module). The pointwise quantities (`Velocity3` etc.) remain untouched; the two
families meet only in the cross-validation harness through the §5.2 isos.

### 5.4 The marching loop closes in the Arrow algebra and the causal monad

Unchanged from the first draft, now over typed forms:

```text
step = rk4_advect_diffuse (Arrow over UnprojectedVelocityOneForm)
       →  bind: leray_project   (fallible: CgFailure short-circuits; emits ProjectedVelocityOneForm)
       →  bind: cfl_check       (fallible: violation short-circuits)
run  = EndoArrow::iterate_until(stop_event)
```

Pure numerics in the arrow, fallible plumbing in the monad, the projection placement
(between steps, Chorin-style, first-order splitting at the projection) stated honestly.
Initial conditions are projected once at `t = 0`: the sampled analytic field is
divergence-free analytically but not necessarily discretely.

## 6. Verification architecture: two independent formulations of one RHS

A structural advantage worth making explicit: the repository now holds **two
independent evaluations of the same physics** — the pointwise regime evaluator
(`incompressible_ns_rhs_kernel` fed by tangent-functor derivatives of an analytic
field) and the DEC operator form (`−i_u du♭ + ν Δu♭` on the sampled cochain). They
share no discretization code. Agreement at the expected convergence order verifies
both; disagreement localizes the defect (conventions → G4, operators → G1, transfer →
G2). This MMS cross-check extends the existing `cfd_taylor_green` harness and should
run in CI, per the project's verification culture.

## 7. Validation plan (acceptance criteria for the spec)

1. **Operator laws** (G1): Leibniz, Cartan, and the convective cross-validation of
   §5.1, with convergence-order assertions.
2. **Iso lawfulness** (G2): round-trip at expected order, naturality, via the iso test
   helpers.
3. **Projection exactness**: discrete divergence of `ProjectedVelocityOneForm` ≤ CG
   tolerance at f32, f64, Float106.
4. **2D Taylor–Green** on `square_torus`: kinetic-energy decay vs. analytic
   `exp(−2νt)`; convergence tables over grid size and `dt` in CI.
5. **2D-in-3D Taylor–Green** (`w = 0`) on `cubic_torus`: same analytic envelope,
   exercising every 3D code path (3D wedge/star/projection) while a closed-form answer
   still exists. This rung is what makes "the target is 3D" testable before turbulence.
6. **Inviscid invariants** (3D, `ν = 0`): kinetic energy and helicity conservation
   over the integration horizon — the standard structure-preservation check for
   DEC/rotational-form schemes.
7. **Full 3D Taylor–Green at Re 1600** (flagship): energy-dissipation-rate curve vs.
   the published DNS reference data for the standard case (the higher-order-workshop
   benchmark). Qualitative agreement at prototype resolutions (64³–128³), stated as
   such; this is the case CFD readers recognize on sight.
8. **Double shear layer** (2D, stretch): roll-up plus conservation diagnostics from
   the existing coherent-structure kernels.

## 8. Risks and non-gaps

- **CG cost without preconditioning**: at 128³ the 1-form Poisson solves dominate the
  step. Acceptable for the prototype; a diagonal/multigrid preconditioner is the first
  performance follow-up, not a correctness item. Log it, don't fix it.
- **Wedge primal–dual bookkeeping** (G1) is the schedule risk; everything else in this
  note is wiring. Budget the spike accordingly.
- **Explicit-step stability**: advective CFL plus the diffusive limit `dt ≲ dx²/(2Dν)`
  both enforced by the `cfl_check` bind — mechanism already demonstrated in
  `effect_diffusion_on_manifold`.
- **Metric scaling**: not a gap — per-edge lengths flow through the Regge Hodge star
  (confirmed in `cubical_regge_geometry/has_hodge_star.rs`).
- **Energy behavior of the rotational form**: the Lamb-form/DEC combination is the
  structure-preserving choice in the literature (MHS 2016); test 6 measures it rather
  than assuming it.

## 9. Remaining open questions

1. Cup-product convention for the cubical wedge (which primal–dual averaging), to be
   fixed during the G1 spike against the Leibniz/Cartan property tests.
2. Whether the challenge (documentation pending) prescribes a benchmark case, grid
   sizes, or a reporting format that overrides the §7 defaults.
3. Pressure diagnostic convention: emit static pressure, Bernoulli pressure, or both.
4. Home of the de Rham map: topology (it is metric-free and domain-agnostic) vs. next
   to the ♯ in the iso layer — decide when writing the G2 module, applying decision 1's
   rule (domain-agnostic ⇒ topology).

## 10. Relationship to the companion CFD notes

This note is the mathematical core of a three-altitude program; the other two
documents in this folder are the platform vision above it and the analysis pipeline
beside it.

### 10.1 `causal_cfd.md` (the platform vision)

Closing G1–G4 delivers `causal_cfd.md`'s two largest gap items in a stronger form
than that note sketched: its §4.1 assembly layer and §4.3 pressure–velocity coupling
collapse into the DEC-native Leray formulation of §2 here (no separate Poisson-solve
machinery; one gauge-fixed CG per step). Several of its open items are now resolved
or stale and should be updated when it is next revised:

- §4.4 "linear solver needs an audit" — resolved: matrix-free `cg_solve` ships in
  `deep_causality_sparse` and is already wired into `hodge_decompose`.
- §4.5 "only Euler exists" — stale: `Rk4`/`Euler` ship as Arrow endomorphisms in
  `deep_causality_calculus` with `iterate_n`/`iterate_until`.
- §7 Q1 (staggered vs. co-located) — resolved by decision 2 here: edge-form velocity
  *is* the staggered/mimetic choice, by construction.
- §7 Q4 (granularity of `Intervenable` integration) — resolved by §5.4 here:
  coarse-grained binds (whole-field march step, projection, CFL check), not per-cell.
- §10 item 10.4 (mass-conservation enforcement as a corrective intervention) —
  **obsoleted upward**: the `ProjectedVelocityOneForm` type-state makes divergence
  drift unrepresentable rather than monitored-and-corrected. One entry of the
  corrective library becomes a compile-time guarantee.

The §5.4 bind chain here is the seed of `causal_cfd.md`'s §10 corrective library:
the `cfl_check` bind is 10.1 (CFL-adaptive timestepping) in embryonic form, and the
`CgFailure` short-circuit is where 10.2 (divergence rescue) attaches. The periodic
milestone here, followed by this note's G5 staging (Poiseuille → cavity), lands
exactly at `causal_cfd.md` Phase 1's lid-driven-cavity target — with the
structure-preserving core de-risked before any cut-cell work begins.

### 10.2 `3DCausalFluidDynamics.md` (the causal-analysis pipeline)

The deepest synergy in the program: **the solver computes the analysis pipeline's
most expensive input as a byproduct.** The Leray projection evaluates (half of) the
Hodge decomposition of the velocity field at every time step; the
`FluidSignature<R>` pipeline (B1a–B1c) consumes exactly a `HodgeDecomposition<R>` of
a velocity 1-form on the same `Manifold`, the same `R`. Once both exist, causal
attribution of simulated flow (signature → `RollingHistory` → SURD) is a tap on the
solve chain — one extra β-step solve per sampled snapshot (G6), not a separate
pipeline. `causal_cfd.md` §8 lists "causal discovery on flow data" as out-of-scope
research-flagship territory; with this note's solver it becomes a closed loop:
simulate → decompose (free) → attribute. No other CFD code can do this, because no
other CFD code's projection step *is* a Hodge decomposition.

Reconciliations the follow-up spec must make:

- **Type unification**: `3DCausalFluidDynamics.md` B4's `SolenoidalField<R>`
  (`from_hodge_projection`) and this note's `ProjectedVelocityOneForm<R>` are the
  same invariant. One type, one home (physics crate), serving both the solver's
  type-state and the analysis pipeline's carrier.
- **G6 ordering**: the B-pipeline on periodic (torus) data requires the G6 CG
  upgrade; on open lattices it runs today. The solver core requires neither.
- **Status audit before scheduling**: B5's kernel list (convective acceleration,
  viscous diffusion, pressure gradient, Q-criterion, λ₂, turbulence scales) appears
  to have shipped already in `deep_causality_physics/src/kernels/fluids/`
  (`causal_cfd.md` §2.2 confirms, "1431 tests"). B5 — and parts of B1b's private
  helpers — may be complete or nearly so; audit before allocating its ~7h.
