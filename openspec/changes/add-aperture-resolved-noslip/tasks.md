# Aperture-resolved immersed no-slip

Four groups, each landing green (touched-crate tests pass in both feature configs, clippy/fmt clean,
prepared commit message). Per AGENTS.md golden rules: no `git commit`, no file deletion — each group
gate prepares a commit message and asks the user to commit. Group A is the topology geometry
(fragment → constraint rows); Group B is the generalized projector; Group C is the physics wiring;
Group D is the cylinder-validation gate.

## 1. A — Cut-face constraint geometry (topology)

- [ ] 1.1 `CutCellRegistry::cut_face_constraints`: for each `Cut` cell, emit the sparse no-slip and
      no-penetration rows from its `CutFaceFragment`s (area, outward normal) and apertures — each row
      a list of `(edge_index, weight)` plus a target value (0 here) and a row weight. Geometry only;
      no solver coupling. (`deep_causality_topology`, beside `solid_incident_edges`.)
- [ ] 1.2 Fragment-velocity reconstruction: define which incident edges interpolate a fragment's
      velocity and their weights (the `sharp`-style metric averaging), as the row coefficients.
- [ ] 1.3 Reduction proof: on axis-aligned `Solid` layers (apertures 0/1, axis-aligned fragments) the
      emitted rows collapse to the binary `solid_incident_edges` pins. Unit test, 2D + 3D, f64 +
      Float106.
- [ ] 1.4 Exactness/consistency tests: rows are produced as measures (area/aperture weighted), pinned
      against a flat half-space cut (analytic) and a disk/cylinder cut; full-`Fluid`/full-`Solid`
      cells emit no partial rows; registry round-trip.
- [ ] 1.5 Group A gate: `make format`, clippy clean, full topology tests both feature configs +
      the new cut-face-constraint tests (cargo + bazel). Prepare commit message; ask the user to commit.

## 2. B — Generalized constrained projector (topology)

- [ ] 2.1 Extend the constrained/open Leray projector with an **additive** weighted-constraint path
      (`leray_project_constrained_weighted_opts` / open variant): solve `{δu = 0} ∩ {Cᵀu = b}` in the
      same KKT projection, Jacobi-preconditioned CG. Existing `*_constrained_opts` / `*_open_opts`
      signatures and behavior unchanged.
- [ ] 2.2 Binary special case: a weighted set of single-edge, hard (zero-target) rows reproduces the
      existing `zeroed_edges` result bit-for-bit (decide: subsume the binary path or keep both).
- [ ] 2.3 Warm-start compatibility: the weighted path accepts the same φ initial guess as the
      existing warm variants (`leray_project_*_warm_opts`).
- [ ] 2.4 Tests: weighted projection is divergence-free to tolerance; binary-equivalence bit-identity;
      a single analytic cut-face row drives the reconstructed face velocity to zero; CG converges
      within budget with cell-merging on a sliver case.
- [ ] 2.5 Group B gate: format, clippy, full topology tests both feature configs. Prepare commit
      message; ask the user to commit.

## 3. C — Physics wiring (no new solver plumbing)

- [ ] 3.1 `NoSlipConstraint` assembles the aperture-resolved rows from `cut_face_constraints` for
      `Cut` cells and the zero-interior pins for `Solid` cells, replacing the staircase set for
      immersed bodies while leaving axis-aligned wall-tangential edges unchanged.
- [ ] 3.2 Route the per-stage projection, the seed projection, and the re-entry projection through the
      weighted projector when aperture-resolved rows are present; binary path otherwise (so periodic /
      wall-only / empty-registry stay on the exact existing path).
- [ ] 3.3 Equivalence tests: empty registry bit-identical to Stage-3; axis-aligned solid layer
      reproduces the analytic Poiseuille profile to rounding (the existing
      `axis_aligned_solid_layer_reproduces_the_wall_poiseuille` gate, now through the resolved path);
      an immersed body marches divergence-free.
- [ ] 3.4 Group C gate: format, clippy (0 warnings), full physics + topology tests both feature
      configs. Prepare commit message; ask the user to commit.

## 4. D — Cylinder validation gate

- [ ] 4.1 Run `dec_cylinder_validation` at `Re = 100` with the aperture-resolved body at 16 and
      24 cells/D; record whether the wake sheds (sustained, non-decaying `v_probe`) versus the
      staircase baseline (steady) at the same resolution.
- [ ] 4.2 Report Strouhal vs Williamson (`≈ 0.164`) and cycle-mean `C_d` vs Lehmkuhl (`≈ 1.33`),
      versus the staircase result, in the example README / a results note.
- [ ] 4.3 Ablation: no-penetration row on/off, to resolve the open question of whether it is needed
      beyond the cut Hodge star's flux down-weighting.
- [ ] 4.4 Cheap CI rung (no heavy march): the consistency reduction (Group A 1.3) and binary-equivalence
      (Group B 2.2) carry the regression; the shedding run stays an example, per tests-fast /
      examples-verify.
- [ ] 4.5 Group D gate: format, clippy, both feature configs green for the shipped rungs; example
      README updated with the aperture-resolved numbers. Prepare commit message; ask the user to commit.
