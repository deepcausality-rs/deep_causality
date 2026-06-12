## Why

Two follow-ons from the `add-fft` change, folded into one change set because
they touch the same operator stack and unblock the same roadmap milestones.

**Performance:** the spectral Poisson solve moved the DEC solver bottleneck
entirely into the rate assembly (`−i_u(du♭) − νΔu♭`): 30 ms serial / 11.2 ms
parallel of the 137 ms / 57 ms step at 32³, ×4 RK4 stages. The operators are
evaluated generically — CSR traversal for `d`/`δ`, per-cell index arithmetic
for interior product and sharp, an intermediate tensor per operator call —
on a lattice where every stencil is fixed and known at manifold construction.
Compiling the stencils once and streaming the rate through fused passes is
the remaining structural win, and it is what brings the Re-1600 dissipation
curve at 64³–128³ (the open Stage 1 exit artifact) into overnight range.

**Walls:** CFD Stage 3 per `cfd-gap.md` §G5 and `cfd-roadmap.md` — the one
place new mathematical substrate is required (sized ≈ G1+G2+G3 combined),
and the gate to the lid-driven-cavity milestone that opens the grant frame.
The sockets exist (per-axis periodicity, boundary-trimming neighborhoods,
closure-accepting CG); the missing content is the boundary-corrected Hodge
star, the Neumann pressure projection, and no-slip viscous rows. The
spectral direct-solve advantage carries over: on uniform boxes the Neumann
Laplacian diagonalizes under the DCT, a small extension to
`deep_causality_fft`'s existing machinery.

## What Changes

**Performance track (periodic solver, pure additive):**

- Compiled per-manifold stencil tables for the cubical-lattice DEC operator
  pipeline (`d`, `δ`, diagonal Hodge factors, interior-product transport,
  sharp): flat index/coefficient arrays built once, replacing CSR traversal
  and per-cell index arithmetic on the hot path.
- Fused, allocation-free rate assembly: the two rate terms stream through a
  reusable workspace; no intermediate `CausalTensor` per operator call
  inside the RK4 stages. Equivalence to the generic operator composition is
  test-gated; the generic path remains the reference and the non-lattice
  fallback.
- Spectral evaluation of the viscous term `νΔ₁u♭` on fully periodic
  lattices (the three edge families are shifted sub-lattices, each
  rFFT-able), opt-in at solver construction. Time integration is unchanged
  (drop-in per-stage evaluation; IMEX/integrating factor is a non-goal).

**Walls track (Stage 3):**

- **BREAKING (spec-level, additive in code):** the boundary-corrected Hodge
  star — dual volumes clipped at walls (halved at faces, quartered at
  edges, eighthed at 3D corners) on open axes of the cubical geometry.
  Existing open-lattice results change where they were interior-only
  approximations.
- DCT support in `deep_causality_fft` (plan-based DCT-I/II/III real
  transforms on the existing kernel machinery).
- The Neumann–Poisson projection path for wall-bounded pressure solves:
  direct spectral solve (DCT on wall axes, DFT on periodic axes) on uniform
  Euclidean boxes; Jacobi-preconditioned CG as the general fallback and
  first preconditioner.
- No-slip in the viscous operator: tangential wall-edge constraints and
  consistent symmetric Laplacian boundary rows.
- Solver wiring: `DecNsSolver` accepts mixed-periodicity manifolds; no-slip
  enforcement as a typed chain stage; wall-aware CFL.
- Validation ladder extension, analytic-first: laminar Poiseuille
  (periodic-x, wall-y; exact parabolic steady state) in CI, then the
  lid-driven cavity at Re 1000 against the Ghia et al. (1982) centerline
  tables (coarse rung in CI, full resolution as an example program).

## Capabilities

### New Capabilities
- `dec-stencil-operators`: compiled stencil tables + fused allocation-free
  rate assembly for cubical lattices; equivalence and performance gates.
- `spectral-diffusion`: opt-in spectral evaluation of the grade-1 viscous
  term on fully periodic lattices.
- `fft-dct`: plan-based DCT-I/II/III real transforms in
  `deep_causality_fft`.
- `wall-hodge-star`: boundary-corrected dual volumes on open axes of the
  cubical Regge geometry.
- `neumann-poisson`: wall-aware grade-0 pressure solve — mixed DCT/DFT
  direct solve on uniform boxes, Jacobi-preconditioned CG fallback.
- `no-slip-viscous`: no-slip boundary conditions in the grade-1 viscous
  operator.
- `wall-bounded-ns`: solver wiring for wall-bounded domains plus the
  Poiseuille and Ghia-cavity validation rungs.

### Modified Capabilities
- `leray-projection`: dispatch extends to wall-bounded uniform lattices —
  the grade-0 solve routes to the Neumann direct solve (or preconditioned
  CG) with no-flux boundary semantics; the periodic spectral path and the
  CG error semantics elsewhere are unchanged.
- `dec-ns-rate`: assembly realizes the same operator composition through
  the compiled stencil pipeline (equivalence-gated); the viscous term may
  be evaluated spectrally on periodic lattices.
- `dec-ns-validation`: the ladder gains the Poiseuille rung (CI) and the
  Ghia lid-driven-cavity rung (coarse CI + example).

## Impact

- `deep_causality_topology`: stencil-table module beside the existing
  differential operators; boundary-corrected cubical Hodge star; Neumann
  dispatch in `solve_laplacian`; no-slip operator rows. The generic
  operator path is untouched and remains the cross-validation reference.
- `deep_causality_fft`: new DCT plan types; no changes to existing
  transforms.
- `deep_causality_sparse`: Jacobi-preconditioned variant of `cg_solve`
  (additive API).
- `deep_causality_physics`: rate workspace over the stencil pipeline;
  solver acceptance of mixed-periodicity manifolds; new validation tests;
  cavity example; `dec_solver_benchmark` extended with stencil/spectral
  variants.
- Existing periodic results: bit-level changes only where evaluation order
  changes (equivalence-gated at tolerance); existing open-lattice Hodge
  star results change *by intent* at boundaries (the old values were
  interior-only approximations) — affected tests are corrected, not
  loosened.
- No new external dependencies; `unsafe_code = "forbid"` everywhere.
