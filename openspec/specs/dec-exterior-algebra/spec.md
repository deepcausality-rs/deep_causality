## ADDED Requirements

### Requirement: Wedge product on cubical lattice cochains
`Manifold<LatticeComplex<D, R>, _>` SHALL provide a wedge product `wedge(alpha, k, beta, l)`
taking a k-form and an l-form to a (k+l)-form, implemented as the cubical
(axis-aligned) cup product, bilinear over `R: RealField`, with orientation
consistent with the lattice `boundary` operator. The operation SHALL return an
error when `k + l > D` or when either field length does not match
`num_cells(k)` / `num_cells(l)`.

#### Scenario: Leibniz rule holds as a property test
- **WHEN** `d(wedge(alpha, beta))` and `wedge(d alpha, beta) + (−1)^k wedge(alpha, d beta)` are computed for sampled smooth fields on a torus
- **THEN** the two results agree to the discretization order observed under grid refinement

#### Scenario: Graded anticommutativity
- **WHEN** `wedge(alpha_k, beta_l)` and `wedge(beta_l, alpha_k)` are computed
- **THEN** they satisfy `alpha ∧ beta = (−1)^{kl} beta ∧ alpha` to machine precision of the backend

#### Scenario: Grade overflow is rejected
- **WHEN** `wedge` is called with `k + l > D`
- **THEN** a typed `TopologyError` is returned and no computation is attempted

### Requirement: Interior product derived from wedge and Hodge star
`Manifold<LatticeComplex<D, R>, _>` SHALL provide `interior_product(x_flat, omega, k)`
computing `i_X ω = (−1)^{k(D−k)} ⋆(⋆ω ∧ X♭)` for a 1-form `X♭` and a k-form `ω`,
producing a (k−1)-form, reusing the existing `hodge_star` (no independent
contraction formulas).

#### Scenario: Cartan's magic formula validates against the tangent functor
- **WHEN** `i_X d ω + d i_X ω` is computed on a sampled Taylor–Green field and compared with the analytic Lie derivative evaluated via `Dual`-number differentiation of the closed-form field
- **THEN** the discrete and analytic results agree at second order under grid refinement

#### Scenario: Convective term cross-validates against the pointwise oracle
- **WHEN** `i_u (d u♭)` is computed on the sampled Taylor–Green velocity and compared with the tangent-functor evaluation of `∇(|u|²/2) − (u·∇)u` at vertices (transferred through the de Rham/sharp maps)
- **THEN** the two independent evaluations agree at second order under grid refinement

#### Scenario: Contraction with the same 1-form twice vanishes
- **WHEN** `i_X (i_X ω)` is computed for any 2-form `ω`
- **THEN** the result is zero to the backend's machine precision

### Requirement: Sign and orientation conventions are pinned by tests
The crate SHALL pin three conventions with dedicated tests that fail on violation:
(a) the Hodge–de Rham Laplacian satisfies `Δ_dR = −∇²` on a flat torus; (b) the
cup-product ordering agrees with the `boundary` orientation (the `(−1)^{k(D−k)}`
factor in the interior product depends on it); (c) all wedge/interior-product APIs
are generic over `R: RealField` and exercised at f32, f64, and Float106.

#### Scenario: Laplacian sign convention
- **WHEN** `laplacian(0)` is applied to a sampled single-mode sine field on a torus
- **THEN** the result equals `+k²` times the field (i.e., `Δ_dR = −∇²`) within discretization tolerance, so the viscous term is `−ν Δ_dR u♭`

#### Scenario: Precision parametricity
- **WHEN** the wedge and interior-product test suite is instantiated at f32, f64, and Float106
- **THEN** all law tests pass at each backend's expected tolerance
