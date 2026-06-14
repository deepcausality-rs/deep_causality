# graded-metrics Specification

## Purpose
Variable-resolution (graded) meshes — fine near walls, coarse in the freestream — are a
prerequisite for credible wall-bounded CFD (Re-10⁴ cavity, the Stage-4 cylinder boundary
layer). In the Regge topology/geometry separation a graded mesh is a `PerEdge` *metric
state* on an unchanged lattice, so the combinatorial guarantees (`d∘d = 0`, discrete
Stokes, divergence-free-by-construction) hold **exactly at any grading** — only accuracy
order is at stake. This capability provides the per-axis graded constructors (geometric,
tanh), pins the exact-structure-at-any-grading guarantee, and verifies — via a method-of-
manufactured-solutions study — that **both march operators (convective `i_X ω` and viscous
`Δ₀ = δd`) retain second order under smooth grading**, in both the max- and L2-norms, to a
3:1 spacing ratio. Only the error constant grows mildly with grading; the order does not
degrade. (An earlier revision of the study mis-measured a convective order-loss; the cause
was a DEC cochain-convention bug in the *measurement* — feeding pointwise 1-form values
instead of edge-integrals — not an operator defect. See
`changes/reverted/fix-graded-convective-consistency.md`.)

## Requirements
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

### Requirement: Operator order verified second order on graded metrics
The G1 wedge and interior-product property tests (Leibniz, Cartan) SHALL converge at second
order on graded metrics, not only uniform ones. An MMS truncation study SHALL measure both
march operators' order across a grading-amplitude sweep, in the max- and L2-norms. The heavy
study lives in an example, with a second-order convergence rung in CI.

**Measured outcome.** With DEC cochains handled consistently (1-forms as edge-integrals,
`×ℓ_edge`; the 1-form output normalised `÷ℓ`), **both** the convective interior product and
the viscous Laplacian are **second order in both norms at every grading amplitude**, to a
3:1 spacing ratio; only the error constant grows mildly with grading. So smooth grading
retains second order. A `1.0`-`1.3` near-wall growth ratio (standard practice) is well
inside the second-order regime. Structure (divergence-freeness of the Leray projection) is
metric-free and exact at *every* grading, pinned independently by the topology exactness
gate. **Correctness note:** omitting the `ℓ_edge` cochain factor mis-measures a false
order-loss on graded meshes (invisible on uniform meshes where `ℓ = 1`); the CI test and the
example both enforce the correct convention.

#### Scenario: Convective and viscous operators are second order under smooth grading
- **WHEN** the interior-product Cartan MMS and the Laplacian MMS run on a smoothly graded metric with consistent (edge-integral) cochains
- **THEN** the discrete operators converge to their continuum references at approximately second order (≥ ~3.5× error drop per grid doubling), as on the uniform metric

#### Scenario: Order is reported across the grading sweep
- **WHEN** the graded MMS example sweeps the grading amplitude
- **THEN** it prints the per-amplitude observed order in both norms for both operators, showing the order holds at ≈ 2 (only the error constant grows) and that structure stays exact at every amplitude

