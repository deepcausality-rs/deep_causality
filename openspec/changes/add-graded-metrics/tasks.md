# Milestone structure

Two sub-groups, each ending green (full tests on the touched crate in both feature
configurations, clippy/fmt clean) with a prepared commit message. Group A is the graded
constructors plus the headline exactness guarantee; Group B is the accuracy verification
(operator property tests + the MMS truncation study). Small change — the operators already
dispatch `PerEdge`; this adds constructors and tests.

Per AGENTS.md golden rules: agents never `git commit` and never delete files — each group
gate prepares a commit message and asks the user to commit. `make` targets are run by the
user on review.

## A. Graded constructors + exact-structure guarantee (graded-metrics)

- [ ] A1 Per-axis **geometric** graded constructor on `CubicalReggeGeometry`: builds a
      `PerEdge` geometry over a given `LatticeComplex` by evaluating `Δ_{i+1} = r · Δ_i`
      per edge in `iter_cells(1)` / `edge_index` order; ungraded axes stay uniform.
- [ ] A2 Per-axis **tanh / hyperbolic** graded constructor (two-sided clustering); same
      `PerEdge` construction path. A single-axis (wall-normal-first-class) convenience form.
- [ ] A3 `PerEdge`-sizing round-trip test: the produced vector length equals
      `sum over axes of edges_along(axis)` and `axis_length_at_position` returns the
      law's value at every edge without hitting the malformed-length panic.
- [ ] A4 Headline exactness test: on a strongly graded lattice (ratio well outside the
      accuracy-good range), the Leray-projected / marched field is divergence-free to the
      solve's exactness and the discrete conservation invariants hold.
- [ ] A5 Group gate: format, clippy, full `deep_causality_topology` tests both feature
      configs; prepare the Group A commit message and ask the user to commit.

## B. Accuracy verification on graded metrics (graded-metrics)

- [ ] B1 Run the G1 wedge / interior-product property tests (Leibniz, Cartan) on a graded
      metric, not only uniform — the operator laws must hold under grading.
- [ ] B2 Graded Taylor–Green MMS example: observed-order regression across a grading sweep;
      confirm smooth grading retains second order; quantify the growth-rate limit where
      order collapses (report it, do not hide it). Hosted as an example per tests-fast /
      examples-verify.
- [ ] B3 Cheap CI accuracy-adjacent regression: a small graded-MMS rung that runs within the
      fast-test budget (the heavy sweep stays in the example).
- [ ] B4 Group gate: format, clippy, full tests both feature configs; update the example
      README with the grading-limit numbers; prepare the Group B commit message and ask the
      user to commit. Change exit: graded metrics available for credible Re 10⁴ cavity and
      the Stage-4 cylinder boundary layer.
