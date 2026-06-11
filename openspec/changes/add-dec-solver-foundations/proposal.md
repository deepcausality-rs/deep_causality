## Why

The DEC-native incompressible Navier–Stokes solver (periodic lattices, 2D/3D, Leray
projection) specified in `openspec/notes/cfd/cfd-gap.md` is blocked by exactly four
gaps in otherwise-complete machinery: the topology crate has `d`, `δ`, `⋆`, `Δ`, and
the Hodge decomposition, but no wedge or interior product (the convective term);
no lawful transfer between pointwise vector fields and edge cochains (initial
conditions, diagnostics, cross-validation); no typed-form carriers for the solver
state; and a β-step CG that is singular on periodic lattices, which blocks the
causal-analysis tap (`3DCausalFluidDynamics.md`) on torus data. Closing these now
gives the solver — and every later CFD stage in `openspec/notes/cfd/cfd-roadmap.md`
— published APIs to assemble against.

## What Changes

- Add a cubical cup-product **wedge** `α ∧ β` and a derived **interior product**
  `i_X ω = (−1)^{k(n−k)} ⋆(⋆ω ∧ X♭)` (Hirani 2003, §8) to
  `Manifold<LatticeComplex<D, R>, _>` in `deep_causality_topology`, with sign and
  orientation conventions pinned once and enforced by Leibniz/Cartan property tests
  (note G1 + G4).
- Add the **de Rham map** (vertex vector field → edge 1-form, via edge line
  integrals) and the **sharp map** (edge 1-form → vertex vector field) as a Tier-2
  iso pair with round-trip-at-order and naturality property tests (note G2).
- Add **typed-form carriers** in `deep_causality_physics`: `VelocityOneForm<R>`,
  `VorticityTwoForm<R>`, `PressureZeroForm<R>`, `BodyForceOneForm<R>`, plus the
  type-state `SolenoidalField<R>` constructible only by the Leray projector —
  unifying the `ProjectedVelocityOneForm` of the gap note with the
  `SolenoidalField` of `3DCausalFluidDynamics.md` B4 into one type (note G3,
  §10.2).
- Add **`leray_project`** to `deep_causality_topology`: the grade-0
  half-decomposition `P(ω) = ω − d(Δ₀⁻¹ δω)` as its own entry point — one
  gauge-fixed CG solve, no β-step, hence well-posed on periodic lattices today
  (note §2).
- Fix **harmonic-kernel deflation** in `hodge_decompose`'s β-step so the *full*
  decomposition is well-posed on periodic lattices with `β_k > 0` (note G6;
  Risk 1 of the archived `add-hodge-decomposition`).

Out of scope: wall boundary conditions (note G5), the solver marching loop itself
(note §5.4 — assembled later against these APIs), GPU, preconditioning, and any
change to the pointwise regime kernels (they remain the independent
cross-validation oracle).

## Capabilities

### New Capabilities

- `dec-exterior-algebra`: wedge product and interior product on cubical lattice
  cochains, with pinned sign/orientation conventions; the operator layer that makes
  the Lamb-form convective term `i_u ω` computable (closes G1, G4).
- `de-rham-transfer`: lawful ♭/♯ transfer between pointwise vector fields and edge
  cochains (de Rham map and sharp), encoded in the iso vocabulary with order-aware
  round-trip laws (closes G2).
- `typed-fluid-forms`: unit-bearing typed wrappers over form carriers in the
  physics crate, including the divergence-free `SolenoidalField<R>` type-state
  whose only constructors are the projection paths (closes G3).
- `leray-projection`: the half-decomposition Leray projector as a first-class
  topology API, plus harmonic-kernel deflation making full `hodge_decompose`
  well-posed on periodic lattices (closes G6, enables note §2).

### Modified Capabilities

<!-- none: openspec/specs/ is empty; all capabilities introduced here are new -->

## Impact

- **Crates**: `deep_causality_topology` (wedge, interior product, de Rham/sharp or
  iso side per design decision, `leray_project`, β-step deflation),
  `deep_causality_physics` (typed forms; new dep edge on `deep_causality_topology`
  if not already present — document in `Cargo.toml` + `BUILD.bazel`),
  `deep_causality_num`/`deep_causality_haft` (consumed, unchanged).
- **APIs**: additive only; `hodge_decompose`'s public signature is unchanged (its
  periodic-lattice behavior goes from documented-singular to correct).
- **Downstream consumers unblocked**: the DEC solver assembly (note §5.4), the
  `FluidSignature` pipeline on torus data (`3DCausalFluidDynamics.md` B1b–B3), and
  the CFD challenge entry's library-parity prerequisite
  (`openspec/notes/cfd-challange/entry-plan.md` §5 release-then-author rule).
- **Precision**: every new API generic over `R: RealField` (f32 / f64 / Float106);
  no `f64` in any new public signature.
