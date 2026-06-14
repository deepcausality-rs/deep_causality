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

- [ ] F1 `surface_force(field, manifold, registry) -> Force` integrating `−p n + μ(∇u+∇uᵀ)·n` over
      the immersed `CutFaceFragment`s: pressure from the projection potential (minus the dynamic
      head), viscous traction from the edge-cochain gradient. Returns the force vector and the
      pressure/viscous split. Precision-generic over `R: RealField`.
- [ ] F2 `drag_lift_coefficients(force, u_ref, length) -> (C_d, C_l)`.
- [ ] F3 Gates (**2D and 3D**): zero net force in a uniform-pressure field; the exact analytic
      force in a linear pressure gradient; both to rounding. Pressure/viscous split reported for
      cross-check.
- [ ] F4 Group gate: format, clippy, full physics tests both feature configs; commit message.

## V. Change gate + handoff

- [ ] V1 `openspec validate --strict`, format, clippy, full physics + topology tests both feature
      configs and bazel; prepare the final commit message; archive this change.
- [ ] V2 Handoff: `add-cut-cells-and-immersed-boundaries` D2/D3 now has every primitive (open
      boundary, free-slip far-field, surface forces, all `const D`-generic) — the isolated-cylinder
      ladder is implemented and gated there, then that change closes: the **2D laminar** rungs
      (Strouhal + `C_d`, Re ≈ 100–200), a **3D DNS** transition rung (Re ≈ 200–300), and Re ≈ 3900
      by DNS as a compute-bound capability check (not a CI rung). A **cheap** high-Re path
      (turbulence closure / wall functions) remains a separate Stage-5 change.
