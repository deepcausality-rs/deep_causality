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
The G1 wedge and interior-product property tests (Leibniz, Cartan) SHALL pass / converge on
graded metrics, not only uniform ones. An MMS truncation study SHALL measure the
convective-operator (interior-product) order across a grading-amplitude sweep and SHALL
report — without hiding it — the amplitude at which the order degrades. The heavy study
lives in an example, with a cheap convergence rung in CI.

**Measured outcome (recorded honestly, amending the original optimistic claim).** The study
found that the discrete interior product is clean second order on a uniform mesh and remains
*convergent* under mild grading (the error keeps decreasing), but it **loses formal second
order under grading** — the finest-grid order falls below 1.5 by an adjacent-spacing ratio
of ≈ 1.11 and plateaus beyond. This is the convective operator's anisotropy-consistency
limit, the same class as the convective-term form-slot issue, and a candidate for the same
vector-slot M-adjoint fix as a follow-up. Crucially, this is an *accuracy* limit only:
**structure** (divergence-freeness of the Leray projection) is metric-free and exact at
*every* grading, pinned independently by the topology exactness gate.

#### Scenario: Wedge/interior-product laws hold under grading
- **WHEN** the interior-product Cartan MMS runs on a smoothly graded metric (mild amplitude)
- **THEN** the discrete `i_X dω + d i_X ω` converges to the Lie derivative under refinement (error decreasing), identically in form to the uniform-metric test

#### Scenario: The order-degradation boundary is measured and reported
- **WHEN** the graded MMS example sweeps the grading amplitude
- **THEN** it prints the per-amplitude observed order and reports the amplitude at which the convective operator's order drops below 1.5, while affirming that structure stays exact at every amplitude
