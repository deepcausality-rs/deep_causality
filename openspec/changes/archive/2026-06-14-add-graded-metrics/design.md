## Context

`variable-grid-geometry.md` §2's thesis: in the Regge topology/geometry separation a
variable mesh is a *metric state*, not a data structure. The `deep_causality_topology`
audit confirms the substrate is already in place (see the proposal): the four-level
`EdgeLengths` union, the all-variant dispatch in `cell_volume` / `has_hodge_star` /
`axis_length_at_position`, and the `edge_index(position, axis)` / `edges_along(axis)` /
`iter_cells(1)` ordering. R1 adds constructors and tests, not operators.

## Goals / Non-Goals

**Goals**
- Per-axis geometric and tanh graded constructors producing a `PerEdge` geometry.
- The exact-conservation / exact-divergence-free guarantee on arbitrary grading, as the
  headline test.
- Operator property tests and an MMS truncation study on graded metrics.
- Axis-aligned wall-normal case first-class (cut-cell forward-compat).

**Non-Goals**
- Metric *adaptation* from an indicator (R2) — and specifically the causal indicator, which
  needs Stage 2. R1 is static grading laws only.
- Topological refinement / AMR (R3).
- Non-axis-aligned curved metrics beyond what `from_edge_lengths` already admits.
- Any change to the DEC operators, the star, or the solver — they already consume `PerEdge`.

## Decisions

### D1: A graded mesh is a `PerEdge` geometry built from a per-edge law
Grading varies edge length with position along the graded axis, so the general result is
the `PerEdge` variant (not `PerAxis`, which is uniform within an axis). The constructor
takes the `LatticeComplex`, enumerates edges in `iter_cells(1)` order, and fills
`lengths[edge_index(position, axis)] = law(position, axis)`, producing a vector sized to
`sum over axes of edges_along(axis)` — exactly what `axis_length_at_position`'s `PerEdge`
arm and the malformed-length panic expect.

### D2: Two analytic stretching laws, per-axis
- **Geometric:** spacing grows by a constant ratio `r` per cell away from a wall
  (`Δ_{i+1} = r · Δ_i`), the standard near-wall family; `r` near 1.0–1.3 is the industrial
  range (`variable-grid-geometry.md` §7).
- **Tanh / hyperbolic:** smooth two-sided clustering toward one or both ends of an axis.
Each law is selected per axis; axes not graded keep uniform spacing. The wall-normal axis
is first-class: a single-axis graded constructor that leaves the other axes uniform is the
common case and must be ergonomic.

### D3: Exactness is combinatorial, hence grading-independent (the headline)
`d∘d = 0` and the discrete Stokes theorem never see the metric, so the Leray projection's
divergence-free-by-construction property holds on any grading. The headline test marches /
projects on a strongly graded lattice and asserts the field is divergence-free to the
solve's exactness, and that the discrete conservation invariants hold — at gradings far
outside the accuracy-good range. Only accuracy order degrades on bad grading; structure
never does. This is the single most important assertion of the change.

### D4: Accuracy is example-hosted; CI carries cheap exactness
Per the tests-fast / examples-verify split established in Stage 3: the graded Taylor–Green
MMS truncation study (observed-order regression across a grading sweep, and the growth-rate
limit where second order collapses) lives in an example. CI carries fast regressions: the
exact-divergence-free assertion on a graded lattice, the constructor's `PerEdge`-sizing
correctness, and the G1 operator property tests (Leibniz, Cartan) evaluated on a graded
metric.

### D5: Composes with cut cells; independent of Stage 2
The cut-cell volume override and graded edge lengths both feed the same `cell_volume`
dispatch, so they compose (a graded-and-cut cell is a clipped volume from graded lengths).
R1 consumes only G1 operator tests on graded metrics; it does not touch the analysis
pipeline. R2's causal-adaptation indicator (which consumes Stage 2) is explicitly out of
scope.

## Risks / Trade-offs

- **Abrupt grading degrades accuracy locally.** Stated up front and quantified by the D4
  truncation study (the growth-rate limit), not hidden. Structure is never at risk (D3).
- **Extreme anisotropy stresses the primal–dual volume ratios in the star**
  (`variable-grid-geometry.md` §4). The truncation study reports where accuracy collapses;
  this is a documented limit of the metric-only approach, and the reason R3 exists.
- **`PerEdge` sizing must match the lattice edge count** or `axis_length_at_position`
  panics loudly (by existing design). The constructor builds the vector from the lattice's
  own enumeration, so the sizes match by construction; a round-trip test pins it.

## Resolved decisions

- **Grading laws:** geometric + tanh ship; arbitrary per-edge assignment is already covered
  by the existing `from_edge_lengths`. The analytic laws are convenience constructors over
  that general field.
- **Hosting:** constructors in `deep_causality_topology`; MMS study in an example. No new
  crate.
