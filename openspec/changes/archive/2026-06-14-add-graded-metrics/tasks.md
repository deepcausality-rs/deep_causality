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

## B. Accuracy verification on graded metrics (graded-metrics)

- [x] B1 Interior-product Cartan law on a graded metric
      (`cartan_formula_converges_under_smooth_grading`): the manufactured Lie-derivative MMS
      on a smoothly graded (sinusoidally modulated, torus-compatible) metric, evaluated at
      physical (cumulative-length) midpoints with the metric-correct `X♭`. The discrete
      `i_X dω + d i_X ω` converges under refinement at mild grading. (Wedge/`d` are
      metric-free so their laws hold trivially under grading.)
- [x] B3 Cheap CI accuracy rung: the B1 test *is* the fast graded-MMS convergence rung (16²
      max grid). It also surfaced the real finding — at *strong* grading (amp 0.3, spacing
      ratio ~1.85) the interior product's order degrades and plateaus (~23% rel-error),
      while *mild* grading (amp 0.1) keeps converging. That boundary is what the example
      quantifies.
- [x] B2 Graded MMS **example** (`examples/avionics_examples/dec_graded_mms`): sweeps grading
      amplitude × resolution and prints per-amplitude observed order. **Finding (honest,
      amends the optimistic spec):** uniform is clean 2nd order; the convective operator
      (interior product) stays *convergent* under mild grading but **loses formal 2nd order**
      — order falls below 1.5 by spacing-ratio ≈ 1.11 and plateaus — the anisotropy-
      consistency limit (same class as the convective-term form-slot issue; candidate for the
      same vector-slot fix). Structure (divergence-freeness) is exact at every amplitude.
      README records the table + conclusions.
- [x] B4 Group gate (for the shipped B1/B3): `make format` clean; full `deep_causality_topology`
      Bazel suite green (124 tests). The example README + numbers land with B2.
