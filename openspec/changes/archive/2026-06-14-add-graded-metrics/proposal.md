## Why

CFD rung R1 of `variable-grid-geometry.md`: variable wall-normal spacing on the cubical
lattice. Every real wall-bounded computation grades its mesh — a turbulent boundary layer
needs wall-normal spacing three-to-four orders finer than the outer flow, and a uniform
mesh fine enough for the wall is computationally absurd everywhere else. Stage 3 shipped
the wall substrate and validated the lid-driven cavity at Re 1000 on a uniform mesh;
**Re 10⁴ and the Stage-4 cylinder wake are not credible on uniform meshes** because the
boundary layer is unresolved. R1 is the credibility prerequisite the roadmap defers Re 10⁴
to, and it lands now alongside the Stage-4 cut-cell preparation.

R1 is unusually cheap here because the substrate already exists. The audit of
`deep_causality_topology/src/types/cubical_regge_geometry/` confirms:

- `CubicalReggeGeometry<D, R, S>` already stores the **full four-level edge-length union** —
  `UnitEdge | Uniform | PerAxis | PerEdge { Vec<R> }` — with constructors for each level
  (`unit`, `uniform`, `per_axis`, `from_edge_lengths`).
- `cell_volume`, `top_cell_volume`, the diagonal Hodge star (`has_hodge_star.rs`, including
  its boundary clip), and `axis_length_at_position` **already dispatch all four variants**.
- `LatticeComplex` exposes the `edge_index(position, axis)` / `edges_along(axis)` /
  `iter_cells(1)` ordering that a `PerEdge` length vector is indexed by.

So a graded mesh is *representable today* — it is a `PerEdge` length assignment on an
unchanged lattice. What is missing is only the ergonomics: there are **no graded
constructors** (a grep confirms none), and the operator stack and accuracy have only been
exercised on uniform metrics. R1 supplies the stretching-law constructors and the test
battery that pins behavior on graded metrics; the operators already do the work.

R1 is **independent of Stage 2** (the causal-analysis tap): per `variable-grid-geometry.md`
§5, R1 consumes only "G1 operator tests on graded metrics"; it is **R2** (metric adaptation)
that consumes Stage 2's causal indicator, not R1. R1 composes with Stage 4 cut cells —
both feed the same `cell_volume` dispatch, so a graded-and-cut cell is a clipped volume
computed from graded edge lengths.

## What Changes

- **Graded edge-length constructors (NEW, `graded-metrics`).** Per-axis analytic stretching
  laws on `CubicalReggeGeometry` that build a `PerEdge` geometry over a given
  `LatticeComplex`:
  - **Geometric** grading (constant growth ratio `r` per axis — the standard near-wall
    family) and **tanh / hyperbolic** grading (smooth two-sided clustering).
  - Per-axis selection with the **axis-aligned wall-normal case first-class**
    (`variable-grid-geometry.md` §4), so wall-normal grading composes with the Stage-3
    walls and the Stage-4 cut cells.
  - Built by evaluating the law per edge in the `iter_cells(1)` / `edge_index` order so the
    `PerEdge` vector is correctly sized (`sum over axes of edges_along(axis)`).
- **Exactness on graded metrics (NEW, `graded-metrics`).** The headline guarantee: because
  `d∘d = 0` and the discrete Stokes theorem are combinatorial (metric-free), discrete
  conservation and the divergence-free-by-construction property of the Leray projection hold
  for **any** edge-length assignment. This is asserted exact (CG/solve tolerance) at
  arbitrary grading — only *accuracy order* is at stake on a graded mesh, never structure.
- **Operator and truncation verification (NEW, `graded-metrics`).** The G1 wedge /
  interior-product property tests (Leibniz, Cartan) run on graded metrics, not only uniform
  ones; an MMS truncation study (graded Taylor–Green) confirms smooth grading retains
  second order and quantifies where abrupt growth-rate jumps degrade it. The heavy MMS study
  lives in an example per the tests-fast / examples-verify split; CI carries the cheap
  exactness regressions.

## Impact

- **Affected specs (new capability):** `graded-metrics`.
- **Affected code:** `deep_causality_topology` only (the graded constructors on
  `CubicalReggeGeometry`; the operator stack and star are unchanged — they already dispatch
  `PerEdge`). A graded-MMS example under `examples/`.
- **No breaking changes.** Purely additive constructors; `unit` / `uniform` / `per_axis` /
  `from_edge_lengths` and all existing uniform results are untouched.
- **Forward links:** unblocks credible Re 10⁴ cavity and the Stage-4 cylinder wake
  boundary layer; the wall-normal-first-class constructors anticipate
  `add-cut-cells-and-immersed-boundaries`. Independent of Stage 2; R2 (metric adaptation)
  will later consume this plus the Stage-2 causal indicator.
