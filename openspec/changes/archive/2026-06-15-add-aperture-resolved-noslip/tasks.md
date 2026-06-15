# Aperture-resolved immersed no-slip

Four groups, each landing green (touched-crate tests pass in both feature configs, clippy/fmt clean,
prepared commit message). Per AGENTS.md golden rules: no `git commit`, no file deletion — each group
gate prepares a commit message and asks the user to commit. Group A is the topology geometry
(fragment → constraint rows); Group B is the generalized projector; Group C is the physics wiring;
Group D is the cylinder-validation gate.

## 1. A — Cut-face constraint geometry (topology)

- [x] 1.1 `CutCellRegistry::cut_face_constraints`: for each `Cut` cell, emit the sparse no-slip and
      no-penetration rows from its `CutFaceFragment`s (area, outward normal) and apertures — each row
      a list of `(edge_index, weight)` plus a target value (0 here) and a row weight. Geometry only;
      no solver coupling. (`deep_causality_topology`, beside `solid_incident_edges`.)
- [x] 1.2 Fragment-velocity reconstruction: define which incident edges interpolate a fragment's
      velocity and their weights (the `sharp`-style metric averaging), as the row coefficients.
      (Per-axis aperture-weighted average of the cell's parallel edges; `axis_reconstruction`.)
- [x] 1.3 Reduction proof: on axis-aligned `Solid` layers (apertures 0/1, axis-aligned fragments) the
      emitted rows collapse to the binary `solid_incident_edges` pins. Unit test, 2D + 3D, f64 +
      Float106. (Fully-fluid/solid cells emit nothing; the dry-bounded edge drops — `f64`+`Float106`.)
- [x] 1.4 Exactness/consistency tests: rows are produced as measures (area/aperture weighted), pinned
      against a flat half-space cut (analytic) and a disk/cylinder cut; full-`Fluid`/full-`Solid`
      cells emit no partial rows; registry round-trip.
- [x] 1.5 Group A gate: `make format`, clippy clean, full topology tests both feature configs +
      the new cut-face-constraint tests (cargo). Prepare commit message; ask the user to commit.

## 2. B — Generalized constrained projector (topology)

- [x] 2.1 Extend the constrained/open Leray projector with an **additive** weighted-constraint path
      (`leray_project_constrained_weighted_opts`): solve `{δu = 0} ∩ {Cᵀu = b}` as the SPD augmented
      dual system `G M⁻¹ Gᵀ y = G f − c` (`G` stacks `∂₁M₁` divergence rows + `Cᵀ` wall rows,
      `y = [φ; λ]`), Jacobi-preconditioned CG. Existing `*_constrained_opts` / `*_open_opts`
      signatures and behavior unchanged. (Constrained gauge = the per-stage hot path; weighted +
      inflow-reference composition deferred — the seed projection keeps the binary path.)
- [x] 2.2 Binary special case: kept both paths — binary pins stay on `zeroed_edges` (masking), and an
      empty `constraint_rows` delegates to `leray_project_constrained_warm_opts`, so the binary
      staircase result is bit-identical (`empty_rows_are_bit_identical_to_the_constrained_path`).
- [x] 2.3 Warm-start compatibility: the weighted path accepts the same φ guess (λ block seeded at
      zero); `warm_start_matches_the_cold_weighted_solve`.
- [x] 2.4 Tests, incl. the **Phase-1 formulation gate** (design Decision 2): single cut cell, project,
      assert reconstructed **fragment velocity zero to tolerance** (`single_cut_cell_drives_fragment_
      velocity_to_zero`). Plus: divergence-free with a mixed binary pin; binary-equivalence bit-identity;
      warm/cold agreement; no-penetration-row off ablation still enforces the tangential rows.
- [x] 2.5 Group B gate: format, clippy clean, full topology tests both feature configs (1254 pass).
      Prepare commit message; ask the user to commit.

## 3. C — Physics wiring (no new solver plumbing)

- [x] 3.0 Lever C (independent, free accuracy win): switched `viscous_surface_force` to the
      **one-sided wall-normal gradient with the true distance Δh** (`S_ij·N_j`, Kirkpatrick 2003) for
      the friction-`C_d`. Added a `centroid` to `CutFaceFragment` (wall anchor; computed in the
      half-space/cylinder/disk intersection), and a multilinear `sample_velocity` one cell out along
      the normal. Re-pinned the analytic test to a genuine no-slip Couette profile (`u_x = a·(y−y_w)`,
      zero at the wall) — still `F = μ·a·A`. Read-only; the marched solver is untouched.
- [x] 3.1 `NoSlipConstraint<R>` (now generic) assembles the aperture-resolved **tangential** rows
      from `cut_face_constraints` for `Cut` cells plus the solid-interior binary pins
      (`solid_incident` minus every edge a cut-face row governs), replacing the staircase for immersed
      bodies while leaving axis-aligned wall edges unchanged. Auto-enabled when the registry has `Cut`
      cells; no-Cut / empty / periodic fall back to the staircase, unchanged. **Open question 4.3
      resolved:** the no-penetration rows are dropped — a closed body's are linearly dependent (the
      `∮ n·u = 0` identity, which floors the projection CG) and redundant (interior-pin + div-free
      already gives zero net surface flux); the tangential rows set separation.
- [x] 3.2 Routed: per-stage rate → `leray_project_constrained_weighted_opts`; seed + re-entry state →
      `leray_project_open_weighted_opts` (via a new `SolenoidalField::from_open_leray_projection_
      weighted_opts`). Empty rows delegate to the binary path bit-identically, so periodic / wall-only
      / empty-registry / axis-aligned-solid stay on the exact existing path.
- [x] 3.3 Equivalence tests green: empty-registry bit-identical and axis-aligned-solid Poiseuille
      (existing gates, now through the weighted entry point with empty rows); new
      `aperture_resolved_disk_marches_divergence_free_with_body_no_slip` (a primitive disk with genuine
      Cut cells marches divergence-free over a long run, tangential no-slip satisfied on the state).
- [x] 3.4 Group C gate: format, clippy 0 warnings, full physics (1589) + topology (1256) tests both
      feature configs. Prepare commit message; ask the user to commit.

## 4. D — Cylinder validation gate

- [~] 4.1 **Harness ready, run pending (user's pinned terminal).** The example builds the cylinder
      from a `Primitive`, so the aperture-resolved body is auto-on; `STAIRCASE=1` flips to the
      staircase baseline on the same geometry. The go/no-go pair (aperture-resolved vs staircase at
      16 cells/D, `LY_D=16`) and the minutes-target framing are in the README. Smoke-verified: both
      modes march divergence-free (~1e-12) at 12/D. The developed 16/D run + `v_probe` comparison +
      wall-clock are filled in after the run.
- [~] 4.2 **Pending the 4.1 run.** README states the references (Williamson `St ≈ 0.164`;
      Dröge–Verstappen `C_d ≈ 1.24` = 0.93 + 0.31) and the comparison protocol; the measured
      aperture-resolved `St` / cycle-mean `C_d` / wall-clock get recorded once the gate run completes.
- [x] 4.3 Ablation resolved at the Group C level: the no-penetration rows are **off** by default —
      a closed body's are linearly dependent (`∮ n·u = 0`, floors the CG) and redundant given the
      solid-interior pins + divergence-freeness. Tangential no-slip carries the wall. (An empirical
      on/off cylinder sweep can still be run, but the formulation question is answered.)
- [x] 4.4 Cheap CI rung: the Group A reduction/exactness, the Group B single-cut-cell gate +
      binary-equivalence, and the Group C `aperture_resolved_disk_marches_divergence_free` test carry
      the regression; the shedding run stays an example (tests-fast / examples-verify).
- [x] 4.5 Group D gate (code side): format, clippy 0 warnings, physics (1589) + topology (1256) green
      both feature configs; example builds and smoke-runs in both no-slip modes; README updated with
      the go/no-go commands, the `STAIRCASE` knob, and the references. Prepare commit message; ask the
      user to commit. (Numbers table filled after the run.)
