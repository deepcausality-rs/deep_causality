## ADDED Requirements

### Requirement: Leray projection as a first-class half-decomposition entry point
`Manifold<K, R>` (for `K::Metric: HasHodgeStar<R>`) SHALL provide
`leray_project(field)` for 1-forms, computing `P(ω) = ω − d(Δ₀⁻¹ δω)` via a single
gauge-fixed grade-0 CG solve (mean subtraction, as in the existing decomposition),
without invoking the β-step. CG non-convergence SHALL propagate as a typed error.
The grade-0 potential SHALL be retrievable alongside the projected field (the
later pressure-recovery diagnostic consumes it; this change does not emit
pressure).

#### Scenario: Projection annihilates exact gradients
- **WHEN** `leray_project` is applied to `d φ` for a sampled smooth potential `φ` on a torus
- **THEN** the result has norm at or below the CG tolerance

#### Scenario: Projection exactness across precisions
- **WHEN** `leray_project` is applied to an arbitrary smooth 1-form sample
- **THEN** the discrete divergence of the result is at or below the CG tolerance at f32, f64, and Float106 (with the existing per-backend tolerance clamping)

#### Scenario: Idempotency
- **WHEN** `leray_project` is applied twice in succession
- **THEN** the second application changes the field by at most the CG tolerance

#### Scenario: Harmonic mean flow is retained on the torus
- **WHEN** a constant (harmonic) 1-form component is present in the input on a periodic lattice
- **THEN** it appears unchanged in the projected output (the projector removes only the gradient part; the mean flow is divergence-free and conserved)

#### Scenario: Solver path is independent of the β-step
- **WHEN** `leray_project` runs on a periodic lattice where the full decomposition's β-step would be singular
- **THEN** it succeeds, demonstrating the half-decomposition's well-posedness on tori

### Requirement: Harmonic-kernel deflation makes full hodge_decompose well-posed on periodic lattices
`hodge_decompose` SHALL deflate the harmonic kernel from the β-step solve on
periodic lattices with `β_k > 0`, by projecting the right-hand side and the CG
iterates onto the orthogonal complement of the constructively-known torus harmonic
basis (constant cochains per axis combination). The public signature SHALL remain
unchanged; behavior on contractible (open) lattices SHALL be bit-compatible with
the current implementation.

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
