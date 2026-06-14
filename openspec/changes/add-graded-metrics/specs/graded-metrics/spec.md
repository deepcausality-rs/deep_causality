## ADDED Requirements

### Requirement: Graded edge-length constructors
`CubicalReggeGeometry` SHALL provide per-axis analytic graded constructors — a geometric
law (constant growth ratio per cell) and a tanh / hyperbolic clustering law — that build a
`PerEdge` geometry over a given `LatticeComplex`. Ungraded axes SHALL remain uniform, and a
single graded axis (the wall-normal case) SHALL be expressible as a first-class convenience
form. The produced `PerEdge` length vector SHALL be sized to the lattice's edge count
(`sum over axes of edges_along(axis)`) and indexed in the `iter_cells(1)` / `edge_index`
order so the existing volume and Hodge-star dispatch consume it without modification.

#### Scenario: Geometric grading produces a correctly sized PerEdge geometry
- **WHEN** a geometric per-axis graded constructor is applied to a lattice with a given growth ratio
- **THEN** the resulting geometry is `PerEdge`, its length vector matches the lattice edge count, and `axis_length_at_position` returns the law's value at every edge

#### Scenario: Wall-normal-only grading leaves other axes uniform
- **WHEN** grading is requested on a single wall-normal axis
- **THEN** edge lengths vary along that axis per the law and are uniform along the other axes

### Requirement: Exact conservation and divergence-freeness at any grading
The solver SHALL preserve discrete conservation and exact divergence-freeness on any graded
metric — to the solve's tolerance, including gradings far outside the accuracy-good range —
because the exterior derivative and the discrete Stokes theorem are combinatorial and never
see the metric. The Leray projection's divergence-free-by-construction property is therefore
grading-independent; grading SHALL affect only accuracy order, never structure.

#### Scenario: Projected field stays divergence-free under strong grading
- **WHEN** a field is Leray-projected (or marched) on a strongly graded lattice with a ratio well outside the accuracy-good range
- **THEN** the result is divergence-free to the solve's exactness and the discrete conservation invariants hold

### Requirement: Operator laws and truncation order verified on graded metrics
The G1 wedge and interior-product property tests (Leibniz, Cartan) SHALL pass on graded
metrics, not only uniform ones. An MMS truncation study on a graded Taylor–Green case SHALL
confirm that smooth grading retains second-order accuracy and SHALL quantify the growth-rate
limit at which the order degrades; the heavy study lives in an example, with a cheap rung in
CI.

#### Scenario: Wedge/interior-product laws hold under grading
- **WHEN** the Leibniz and Cartan property tests run on a graded metric
- **THEN** they pass within tolerance, identically to the uniform-metric case

#### Scenario: Smooth grading retains second order
- **WHEN** the graded Taylor–Green MMS study runs across a smooth-grading refinement sweep
- **THEN** the observed spatial order is approximately second order, and the growth-rate limit where order collapses is reported
