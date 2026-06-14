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

- [x] A1 Per-axis **geometric** graded constructor `from_graded_geometric` on
      `CubicalReggeGeometry` (`base[a] · ratio[a]^pos` per edge, built in `iter_cells(1)`
      order via `from_edge_lengths`; ungraded axes uniform). Bound `RealField +
      FromPrimitive`, not `Float` (uses `powf`, not the `Float`-only `powi`).
- [x] A2 Per-axis **tanh** graded constructor `from_graded_tanh` (two-sided Vinokur
      clustering on a single wall-normal axis, others uniform) — the wall-normal-first-class
      form. Degenerates to uniform as `β → 0`.
- [x] A3 Sizing + law test: the `PerEdge` vector is sized to the edge count and every edge
      carries the analytic law value (`geometric_grading_produces_correct_per_edge_lengths`),
      plus `graded_cell_volume_equals_edge_length_product` pinning the volume dispatch.
- [x] A4 Headline exactness test
      (`leray_projection_stays_divergence_free_under_strong_grading`): on a strongly graded
      torus (ratio 1.4) the Leray-projected random field has divergence < 1e-9 — structure
      holds because `d` / Stokes are combinatorial and metric-free.
- [x] A5 Group gate: `make format` clean; full `deep_causality_topology` Bazel suite green
      (124 tests). Tanh clustering + wall-normal-uniform asserted in the same test file.

## B. Accuracy verification on graded metrics (graded-metrics) — REMAINING

- [ ] B1 Run the G1 wedge / interior-product property tests (Leibniz, Cartan) on a graded
      metric, not only uniform — the operator laws must hold under grading. (Wedge/`d` are
      metric-free so trivially hold; the meaningful case is the interior-product Cartan
      refinement study on a graded metric.)
- [ ] B2 Graded Taylor–Green MMS example: observed-order regression across a grading sweep;
      confirm smooth grading retains second order; quantify the growth-rate limit where
      order collapses (report it, do not hide it). Hosted as an example per tests-fast /
      examples-verify. (Substantial — a new example crate driving the solver.)
- [ ] B3 Cheap CI accuracy-adjacent regression: a small graded-MMS rung that runs within the
      fast-test budget (the heavy sweep stays in the example).
- [ ] B4 Group gate: format, clippy, full tests both feature configs; update the example
      README with the grading-limit numbers; prepare the Group B commit message and ask the
      user to commit. Change exit: graded metrics available for credible Re 10⁴ cavity and
      the Stage-4 cylinder boundary layer.
