# Milestone structure

Two groups, each ending green (tests passing on touched crates both feature configs, clippy/fmt
clean) with a prepared commit message. Per AGENTS.md golden rules: agents never `git commit` and
never delete files — each group gate prepares a commit message and asks the user to commit.

> **Depends on:** `add-boundary-zone-abstraction` (archived) and the `CutCellRegistry` /
> `CutFaceFragment` surface from `add-cut-cells-and-immersed-boundaries`.

## S. Free-slip / far-field boundary zone (slip-boundary)

- [x] S1 `SlipWall<D>` `BoundaryZone` (`dec/boundary/slip_wall.rs`): declares a free-slip face via
      the new `collect_slip_edges` hook (default no-op) returning the face's wall-tangential edges;
      `NoSlipConstraint::remove_edges` un-pins them, so the no-slip set is
      `(auto walls ∪ constrained_edges) \ slip_edges`. No slip face ⇒ set unchanged ⇒ bit-identical.
- [x] S2 Wired the un-pin through `with_zones` → `DecNsRate::apply_slip` (recomputes the rate
      constraint); the rate/seed/step all read the reduced no-slip set. **Confirmed** the
      boundary-clipped star gives a clean zero-shear condition — a uniform plug flow is preserved to
      <1e-8 (2D) / <1e-7 (3D) with no stencil fix (the flagged risk is resolved).
- [x] S3 Gates (`dec/slip_wall_tests`, 2D and 3D): free-slip preserves a uniform plug flow (no
      boundary layer); no-slip pins the wall edges to zero (the contrast); bit-identity holds (the
      1578 prior physics tests, incl. no-slip cavity/Poiseuille, pass unchanged).
- [x] S4 Group gate: format, clippy (0 warnings), full physics tests (1581 pass); commit message
      prepared.

## F. Surface-force diagnostic (surface-force-diagnostic)

- [x] F1 `pressure_surface_force(registry, cell_pressure) -> [R; D]` integrating `−∮ p n dA` over
      the immersed `CutFaceFragment`s (`dec/surface_force.rs`), plus `fragment_area_vector` (the
      `∮ n dA` closure check). Precision- and dimension-generic. **The viscous (friction) traction
      is deferred to the D2/D3 cylinder build** — it needs a `sharp`+finite-difference strain
      reconstruction at the cut cells and is only meaningfully verified against the reference drag,
      not a fast analytic gate (per tests-fast / examples-verify).
- [x] F2 `force_coefficient(force_component, u_ref, reference_area) -> R` (`C = F / (½ρU²A)`).
- [x] F3 Gates (`dec/surface_force_tests`, **2D disk + 3D cylinder**): the fragment normals close
      (`∮ n dA ≈ 0`); a uniform pressure gives zero net force (exact); a linear pressure gradient
      gives `−∇p · V_solid` within the O(h) cell-center tolerance (10%); the coefficient normalizes
      exactly.
- [x] F4 Group gate: format, clippy (0 warnings), full physics tests (1584 pass) + bazel; commit
      message prepared.

## V. Change gate + handoff

- [x] V1 `openspec validate --strict` ✓, format ✓, clippy 0 ✓, full physics (1584) tests + bazel
      ✓; final commit message prepared; change synchronized into the living specs and archived.
      (The viscous traction folds into the D2/D3 cylinder build — see V2 — so it is gated against
      the reference drag there, not as an isolated unit.)
- [ ] V2 Handoff: `add-cut-cells-and-immersed-boundaries` D2/D3 now has every primitive (open
      boundary, free-slip far-field, surface forces, all `const D`-generic) — the isolated-cylinder
      ladder is implemented and gated there, then that change closes: the **2D laminar** rungs
      (Strouhal + `C_d`, Re ≈ 100–200), a **3D DNS** transition rung (Re ≈ 200–300), and Re ≈ 3900
      by DNS as a compute-bound capability check (not a CI rung). A **cheap** high-Re path
      (turbulence closure / wall functions) remains a separate Stage-5 change.
