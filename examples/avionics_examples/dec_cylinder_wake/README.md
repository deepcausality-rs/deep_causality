# Cut-cell cylinder wake — CFD Stage 4 (Group D harness)

Flow past a circular cylinder built as an **immersed cut-cell body**, exercising the full
Stage-4 cut-cell stack end to end:

- the cylinder is a `CutCellRegistry` from the analytic disk primitive
  (`CutCellRegistry::from_primitive`, A4) — exact clipped volumes + apertures, not a staircase;
- the cut **Hodge star** (B5) makes every operator (compiled stencils, constrained Leray
  projection, codifferential) see the partial cells transparently;
- the immersed **no-slip / no-penetration** condition (B4) pins the body's edges through the
  existing constrained projector.

```text
cargo run --release -p avionics_examples --example dec_cylinder_wake
```

It streams a CSV to stdout (`step, t, kinetic_energy, max_speed, div_residual, v_probe`) and
prints the case setup and a shedding-Strouhal estimate to stderr.

## What this harness is — and is not

The DEC solver's boundary conditions today are no-slip / moving walls, body force, and
periodicity — there is **no inflow / outflow boundary yet** (that arrives with the Stage-4
uncertain-inflow zone, Group C). So the flow is driven by a streamwise body force in a
**periodic channel** (periodic-x, wall-y) containing the cylinder: the confined /
periodic-array cylinder, which sheds a von-Kármán street and is a faithful exercise of the
cut-cell machinery.

The quantitative **isolated-cylinder Reynolds ladder** against Lehmkuhl et al. (2013) and the
Williamson lineage (tasks D2/D3 — Strouhal and drag over Re 100–3900) needs that
inflow/outflow surface **plus** the small-cell stabilizer selection (B1–B3). It is **not**
claimed here; the printed Strouhal is for the confined/periodic case and is a qualitative
shedding check, not a reference comparison.

## Small-cell guard (placeholder for B1–B3)

Arbitrarily small cut cells tighten the CFL bound catastrophically — the canonical cut-cell
hazard the Group-B stabilizer (cell-merging vs flux-redistribution) will resolve. Until that
decision lands, this harness applies a simple **volume-fraction merge**: a cut cell wetted
below `MERGE_FRACTION` is absorbed into the solid body. This is a crude cell-merging stand-in
(a preview of B1), enough to keep the march stable while the real stabilizer is prototyped on
this very case.

## Cheap CI regressions (D4)

The fast, no-march regressions that gate the substrate live in the crate test suites, not in
this example (per the tests-fast / examples-verify split):

- **cut-geometry exactness** — the clipped fluid volumes of a disk sum to the exact
  `domain − π r²` (`deep_causality_topology` `cut_cell::consistency_tests`), alongside the
  per-primitive exactness at f64 and Float106 (`cut_cell::intersection_tests`);
- **axis-aligned consistency** — an empty registry's cut star is byte-equal to the Stage-3
  star, and an empty-registry march is bit-identical to the plain solver
  (`cut_cell::cut_star_tests`; physics `cut_cell_wiring_tests`);
- **immersed no-slip** — an immersed solid block pins its edges to zero and the flow stays
  divergence-free (physics `cut_cell_wiring_tests`).

The **small-cell stability** rung (a deliberately tiny cut marches without CFL blow-up under
the chosen stabilizer) lands with the B1–B3 stabilizer selection.
