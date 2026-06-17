## Context

`openspec/notes/cfd/cfd-gap.md` is the ground-truth note: it audited the workspace
and found the DEC stack (`d`, `δ`, `⋆`, `Δ`, `hodge_decompose`, per-edge-length
Regge metric, matrix-free CG, `Rk4`/`EndoArrow`, the CoMonad stencil machinery)
already shipped, with four bounded gaps between it and an assemblable DEC-native
incompressible solver. The note's §0 records five resolved decisions this design
inherits verbatim: domain-agnostic operators in topology with unit types layered in
physics; DEC-native (edge 1-form velocity) from the start; carriers as
domain-specific unit types, not arithmetic on raw tensors; pressure recovery
opt-in; 3D as the target. Architectural conventions from `AGENTS.md` apply:
one-type-one-module, no prelude, tests mirror src, 100% coverage, no `unsafe`,
static dispatch, workspace lints.

Literature anchors: Hirani (2003) for the discrete wedge/interior product;
Mohamed–Hirani–Samtaney (JCP 312, 2016) for the solver shape these operators feed
(see `openspec/notes/cfd/references.md`).

## Goals / Non-Goals

**Goals:**

- `wedge` and `interior_product` on cubical lattice cochains with conventions
  pinned by executable law tests (Leibniz, Cartan) — closes G1+G4.
- Lawful pointwise-vector ↔ edge-cochain transfer (de Rham map, sharp) — closes G2.
- Typed-form carriers including the `SolenoidalField<R>` type-state — closes G3
  and unifies the duplicate type with `3DCausalFluidDynamics.md` B4.
- `leray_project` as a one-CG-solve entry point; β-step harmonic deflation so full
  `hodge_decompose` is well-posed on tori — closes G6.
- Every new public API generic over `R: RealField`; f32/f64/Float106 exercised in
  tests.

**Non-Goals:**

- The solver marching loop (note §5.4) — assembled in a later change against these
  APIs.
- Wall boundary conditions (note G5), graded metrics (R1 of
  `variable-grid-geometry.md`), preconditioning, GPU, parallelism.
- Any change to the pointwise regime kernels in
  `deep_causality_physics::theories::fluid_dynamics` — they stay frozen as the
  independent cross-validation oracle (note §6).

## Decisions

**D1 — Wedge as a primal cubical cup product; interior product derived, not
hand-rolled.** The wedge is implemented as the cup product on axis-aligned cells
(the cubical case avoids the simplicial averaging zoo); `interior_product` is then
the composition `(−1)^{k(n−k)} ⋆(⋆ω ∧ X♭)` reusing the shipped `hodge_star`.
Rationale: one new combinatorial operator instead of two; the sign factor and
orientation conventions are pinned in exactly one place. Alternative considered —
direct contraction formulas per grade — rejected: more code paths, more convention
surface, no reuse of the (already metric-correct) star.

**D2 — Conventions are test-pinned, not comment-pinned (G4).** Three conventions
are fixed by property tests that fail loudly on violation: (a) `Δ_dR = −∇²` on a
flat torus (viscous sign), (b) cup-product ordering consistent with `boundary`
orientation, (c) de Rham edge orientation consistent with `exterior_derivative`.
The MMS cross-validation (note §6) is the systemic detector; the law tests are the
unit-level detectors. Rationale: the gap note flags sign errors as the
silent-anti-diffusion failure mode; comments do not catch them, tests do.

**D3 — De Rham map and sharp live in `deep_causality_topology`; the iso laws are
asserted with the Tier-2 witness vocabulary.** Per note open question 4 and
decision 1's rule (domain-agnostic ⇒ topology): the maps are metric-aware lattice
operations with no physics content. The Tier-2 `Iso` witness (cross-crate-safe)
carries the law tests: round-trip asserted *at discretization order* (not
exactness — the pair is an isomorphism only up to O(h²)), naturality via
`iso::test_support`. Alternative — physics-side placement next to the typed forms
— rejected: the analysis pipeline and future non-fluid consumers need the same
transfer without a physics dependency.

**D4 — One `SolenoidalField<R>`, physics crate, two constructors.** The type-state
unification per note §10.2: private fields; `pub(crate)` construction reachable
only through (a) the Leray projection path (solver, per-step) and (b)
`from_hodge_projection` (analysis pipeline, per-snapshot). `Add`/`Mul<R>` are
implemented on the *unprojected* `VelocityOneForm<R>` (the `Rk4` state) and **not**
on `SolenoidalField` — summing two projected fields does not yield a projected
field under floating point, so the algebra is deliberately absent; re-projection is
the only path back. Rationale: the invariant is the point; leaking arithmetic
would quietly re-open the hole the type exists to close.

**D5 — `leray_project` is a separate entry point, not a flag on
`hodge_decompose`.** It implements `P(ω) = ω − d(Δ₀⁻¹ δω)`: the grade-0 solve only,
gauge-fixed by the existing mean subtraction, one CG evaluation. Rationale (note
§2): the solver core must not depend on the β-step at all — cost (one solve, not
two) and well-posedness on tori (β-step singularity sidestepped) both follow.
Alternative — always calling full `hodge_decompose` and discarding components —
rejected: doubles per-step cost and couples the solver to G6.

**D6 — G6 closed by empirical pinning, not deflation (revised during apply,
2026-06-11).** Tests-first implementation falsified the premise: the β-step's RHS
`dω` is M-orthogonal to the harmonic kernel (⟨dω, h⟩_M = ⟨ω, δh⟩_M = 0), so CG's
Krylov space remains in `range(Δ)` and the consistent singular system converges on
tori — verified at 2D/3D, mixed periodicity, and 16×16 at default tolerance.
Building deflation would have been speculative machinery (AGENTS.md §2). What
ships instead: the behavior pinned by tests (the 16×16 case as the drift canary)
and the constructive-basis deflation documented as the fallback with a named
trigger. The original mechanism (M-normalized indicator cochains per
periodic-axis subset, S = ∅ excluded to keep grade-0 mean subtraction
bit-identical, projector-wrapped `apply`) is preserved here for that eventuality.

**D7 — Capability boundaries follow crate boundaries.** `dec-exterior-algebra`,
`de-rham-transfer`, `leray-projection` specify topology-crate behavior;
`typed-fluid-forms` specifies physics-crate behavior. One change, four specs,
sequential implementation in dependency order (G1+G4 → G2 → G3 → G6/leray) —
matching the note's gap ordering and keeping each review surface single-crate, per
the project's block-gate tradition (`3DCausalFluidDynamics.md` §3).

## Risks / Trade-offs

- **[Wedge primal–dual bookkeeping is the schedule risk]** (note §8) → budget the
  G1 spike explicitly; the Leibniz/Cartan law tests are written *first* and drive
  the convention choices; the cubical (axis-aligned) case avoids the worst
  simplicial interpolation complexity.
- **[Sign/orientation errors masquerade as physics]** → D2's test-pinning plus the
  note-§6 cross-validation harness (tangent-functor analytic derivatives vs. DEC
  operators on the same Taylor–Green field) localize defects to
  conventions/operators/transfer respectively.
- **[Type-state ergonomics]** → forbidding arithmetic on `SolenoidalField` forces
  an explicit re-projection step in every chain; this is intended friction, but the
  design accepts it may need a documented `into_inner()` escape hatch for
  diagnostics (read-only access, never re-wrapping).
- **[Deflation correctness depends on the harmonic basis being exact]** → on the
  torus it is exact by construction (lattice constants); the spec scopes G6 to
  periodic lattices and leaves open-lattice HMF questions to G5 explicitly.
- **[Performance: unpreconditioned CG]** → accepted for this change (note §8: log
  it, don't fix it); preconditioning is a later performance change-set.

## Migration Plan

Additive APIs only; no existing signature changes. `hodge_decompose` behavior on
periodic lattices changes from documented-failure (Risk 1) to correct — strictly a
fix; the archived risk note is superseded and referenced in the changelog. Rollout
is crate-release-ordered: topology first (G1, G2, G6, `leray_project`), then
physics (G3, consuming the released topology APIs) — matching the
release-then-author rule in `openspec/notes/cfd-challange/entry-plan.md` §5.
Rollback: additive surface; reverting is removing the new modules.

## Open Questions

1. Cup-product convention detail (which primal–dual averaging in the wedge) — per
   note §9 Q1, resolved *during* the G1 spike by whichever choice satisfies the
   Leibniz/Cartan tests; the spec fixes the laws, not the formula.
2. Pressure diagnostic convention (static vs. Bernoulli vs. both) — deferred to
   the solver-assembly change; nothing in this change emits pressure.
3. Whether `VorticityTwoForm` warrants its own type-state (closedness `dω = 0` is
   automatic for `ω = du♭`) — default: plain typed wrapper, no state.
