## ADDED Requirements

### Requirement: De Rham map from vertex vector fields to edge cochains
`deep_causality_topology` SHALL provide a de Rham map taking a vertex-sampled
vector field (`D` components per vertex) on a `Manifold<LatticeComplex<D, R>, _>`
to an edge 1-form, by integrating the tangential component along each oriented
edge (`u♭(e) ≈ u·t̂ Δx` at minimum; exact line integrals where the caller supplies
them). Edge orientation SHALL be consistent with `exterior_derivative`.

#### Scenario: Exact on gradient fields
- **WHEN** the de Rham map is applied to the exact gradient of a potential whose line integrals are supplied exactly
- **THEN** the resulting 1-form equals `d` applied to the vertex-sampled potential, exactly (fundamental theorem of calculus, which simultaneously pins the orientation convention)

#### Scenario: Length mismatch is rejected
- **WHEN** the input vector field length does not equal `D × num_vertices`
- **THEN** a typed error is returned

### Requirement: Sharp map from edge cochains to vertex vector fields
`deep_causality_topology` SHALL provide a sharp (♯) map taking an edge 1-form to a
vertex-sampled vector field by metric-weighted averaging of the incident edges per
axis, honoring per-axis periodicity.

#### Scenario: Constant fields round-trip exactly
- **WHEN** a constant vector field is mapped through de Rham then sharp on a torus
- **THEN** the original field is recovered to the backend's machine precision

### Requirement: Transfer pair satisfies iso laws at discretization order
The de Rham/sharp pair SHALL be encoded with the Tier-2 iso witness vocabulary,
with property tests asserting (a) round-trip error of second order under grid
refinement on smooth fields (not exactness — the pair is an isomorphism only up to
discretization), and (b) naturality with respect to linear maps via the
`iso::test_support` helpers, at f32, f64, and Float106.

#### Scenario: Round-trip converges at second order
- **WHEN** `sharp(de_rham(u)) − u` is measured for the sampled Taylor–Green field across a sequence of grid refinements
- **THEN** the error norm decreases at second order

#### Scenario: Naturality under scalar linear maps
- **WHEN** a linear map is applied before the de Rham map and, alternatively, after it (component-wise on the cochain)
- **THEN** the two paths agree per the naturality law helpers
