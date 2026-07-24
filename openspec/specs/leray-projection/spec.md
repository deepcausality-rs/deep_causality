# leray-projection

## Purpose

The Leray projection `P = I − ∇Δ⁻¹∇·` as a first-class entry point: the half-decomposition
that returns the solenoidal part without paying for the full Hodge decomposition, the
well-posedness of the full decomposition on periodic lattices, and the constrained variant
that projects onto no-slip ∩ divergence-free on wall-bounded lattices.

## Requirements
### Requirement: Leray projection as a first-class half-decomposition entry point
`Manifold<K, R>` (for `K::Metric: HasHodgeStar<R>`) SHALL provide
`leray_project(field)` for 1-forms, computing `P(ω) = ω − d(Δ₀⁻¹ δω)` via a single
gauge-fixed grade-0 solve (mean subtraction, as in the existing decomposition),
without invoking the β-step. The grade-0 solve dispatches by domain:

* **Fully periodic uniform Euclidean lattices**: the spectral (DFT) solve —
  exact to rounding, no tolerance, iteration budget, or convergence-failure
  mode.
* **Uniform Euclidean lattices whose axes are each periodic or walled**:
  the direct Neumann solve (DFT/DCT per axis) with no-flux wall semantics —
  same exactness properties.
* **All other manifolds** (per-edge metrics, non-uniform geometry,
  simplicial): the gauge-fixed CG solve with unchanged semantics —
  Jacobi-preconditioned where the boundary-corrected diagonal is available
  — and CG non-convergence SHALL propagate as a typed error.

Dispatch SHALL be automatic from the lattice's periodicity and metric; no
new options surface is introduced. The grade-0 potential SHALL be
retrievable alongside the projected field (the pressure-recovery diagnostic
consumes it).

#### Scenario: Projection annihilates exact gradients
- **WHEN** `leray_project` is applied to `d φ` for a sampled smooth potential `φ` on a torus
- **THEN** the result has norm at rounding level for the precision (spectral path)

#### Scenario: Projection exactness across precisions
- **WHEN** `leray_project` is applied to an arbitrary smooth 1-form sample
- **THEN** the discrete divergence of the result is at rounding level on direct-solve domains (periodic and uniform walled), and at or below the CG tolerance elsewhere, at f32 and f64 (with the existing per-backend tolerance clamping where CG runs)

#### Scenario: Idempotency
- **WHEN** `leray_project` is applied twice in succession
- **THEN** the second application changes the field by at most rounding level on direct-solve domains (CG tolerance on the CG path)

#### Scenario: Harmonic mean flow is retained on the torus
- **WHEN** a constant (harmonic) 1-form component is present in the input on a periodic lattice
- **THEN** it appears unchanged in the projected output (the projector removes only the gradient part; the mean flow is divergence-free and conserved)

#### Scenario: Solver path is independent of the β-step
- **WHEN** `leray_project` runs on a periodic lattice where the full decomposition's β-step would be singular
- **THEN** it succeeds, demonstrating the half-decomposition's well-posedness on tori

#### Scenario: Dispatch follows domain class
- **WHEN** `leray_project` runs on a fully periodic lattice, a uniform walled/mixed lattice with the corrected star, and a per-edge-metric lattice
- **THEN** the first takes the DFT path, the second the DFT/DCT Neumann path, and the third the (preconditioned) CG path with unchanged error semantics

#### Scenario: Direct solves agree with CG
- **WHEN** the same 1-form is projected via a direct path and via the retained, internally invokable CG path on the same manifold (periodic and walled cases)
- **THEN** the projected fields and grade-0 potentials agree within the CG tolerance

#### Scenario: No flux through walls
- **WHEN** `leray_project` runs on a walled lattice
- **THEN** the removed gradient component has vanishing wall-normal boundary trace to rounding

### Requirement: Full hodge_decompose is well-posed on periodic lattices
`hodge_decompose` SHALL converge on periodic lattices with `β_k > 0` and produce
pairwise-orthogonal components, with its public signature and its contractible
(open) lattice behavior unchanged.

*Implementation finding (2026-06-11, supersedes the design's deflation mechanism):*
the β-step's right-hand side `dω` is M-orthogonal to the harmonic kernel in exact
arithmetic, so CG's Krylov space stays in `range(Δ)` and the consistent singular
system converges without deflation — verified by tests on 2D/3D tori up to 16×16
at the default (1e-10 relative) tolerance, plus mixed-periodicity lattices. No
deflation machinery ships; the constructive torus harmonic basis (constant
cochains per periodic-axis combination, M-normalized) remains the documented
fallback if larger-scale use ever exhibits kernel-drift stagnation, with the
16×16 stress test as the canary.

#### Scenario: Decomposition converges on the torus
- **WHEN** `hodge_decompose` is applied to a smooth 1-form on `square_torus` and `cubic_torus` (where `β₁ = D > 0`)
- **THEN** it converges within the iteration budget and returns components `dα`, `δβ`, `h`

#### Scenario: Components are pairwise orthogonal
- **WHEN** the three returned components on a torus are pairwise inner-producted under the Hodge star metric
- **THEN** each inner product is at or below the CG tolerance

#### Scenario: Open-lattice behavior is unchanged
- **WHEN** the existing decomposition test suite runs on contractible lattices
- **THEN** all results match the pre-change behavior (regression guard)

#### Scenario: Half- and full-decomposition agree on the gradient part
- **WHEN** both `leray_project` and the deflated `hodge_decompose` are applied to the same 1-form on a torus
- **THEN** `ω − dα` from the full decomposition equals the `leray_project` output within CG tolerance

### Requirement: Constrained Leray projection
`Manifold<K, R>` SHALL provide `leray_project_constrained_opts(field,
constrained_edges, opts)`: the M-orthogonal projection onto the
intersection of the divergence-free subspace with the essential
constraint subspace `S = {u : u|_E = 0}` for a caller-supplied edge set
`E`. The projection SHALL satisfy both invariants simultaneously —
`u|_E = 0` exactly and the full (unmasked) discrete divergence at the
solve's exactness — realized as `u = P_S(field) − P_F dφ` with
`(∂ M₁|_F d) φ = ∂ M₁ P_S(field)` (the grade-0 operator assembled with
the constrained edges' masses removed, the gradient correction masked to
free edges). The masked operator loses per-axis separability, so the
solve SHALL run Jacobi-preconditioned CG; vertices whose every incident
edge is constrained yield structurally null rows whose right-hand-side
entries are zeroed, with the consistency gauge taken over the connected
block. An empty edge set SHALL delegate to the plain projection
bit-identically; invalid edge indices, field-length mismatches, the
fully-constrained degenerate case, and CG non-convergence SHALL return
typed errors.

#### Scenario: Both invariants hold simultaneously
- **WHEN** the constrained projection runs on mixed-periodicity and all-walls lattices (including null corner rows)
- **THEN** every constrained edge is exactly zero and the full divergence is at the solve's exactness

#### Scenario: The projection is M-orthogonal and idempotent
- **WHEN** the removed component is tested against members of the intersection subspace, and the projection is applied twice
- **THEN** the M-inner products vanish at rounding and the second application changes nothing beyond solve tolerance

#### Scenario: Empty constraint set delegates
- **WHEN** the constrained projection runs with no constrained edges
- **THEN** the result is bit-identical to `leray_project_opts`
