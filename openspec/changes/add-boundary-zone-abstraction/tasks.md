# Milestone structure

Six groups, each ending green (tests passing on touched crates in both feature configurations,
clippy/fmt clean) with a prepared commit message. Per AGENTS.md golden rules: agents never
`git commit` and never delete files — each group gate prepares a commit message and asks the user
to commit; `make fix` / `make build` / `make test` are run by the user on review.

Reduction-first: the non-breaking gates (Z, P) land and are pinned bit-identical **before** any
new open-boundary behaviour (I, O) is added.

## Z. Boundary-zone abstraction (boundary-zone-abstraction)

- [x] Z1 `BoundaryZone<D, R>` trait with one collect-into hook per solver stage (rate source,
      constrained edges, time-dependent lift, prescribed/inflow edges, reference/outflow vertices),
      each defaulting to a no-op. **Static dispatch only** — no `dyn`/trait objects.
      (`dec/boundary/boundary_zone.rs`.)
- [x] Z2 Static (HKT-aligned) composition: a typed tuple is itself a `BoundaryZone` folding each
      hook over its members (`(A, B)`, identity `()`); the `with_zones` builder is generic over the
      composed zone type `Z: BoundaryZone<D, R>`. Folds rate source = Σ collect_rate_source and lift
      = ∪ collect_lift at construction.
- [x] Z3 Re-express the explicit **actuators** as zones (numerics preserved): `MovingWall` (lift),
      `BodyForceZone` (rate source). Structural boundaries — wall no-slip (non-periodic axes) and
      immersed cut bodies (`CutCellRegistry` on the metric) — stay auto-derived at the
      topology/metric layer (a refinement of the design: they are structural, not actuators).
      *Caller migration* (threading zones through the examples/tests and consolidating the public
      surface) lands with the open-boundary groups, since `with_zones` must also carry the
      inflow/outflow sets.
- [x] Z4 Numerical-equivalence gate (`dec/boundary_zone_tests`): a `BodyForceZone` solver marches
      **bit-identically** to `new(.., Some(force))` (Poiseuille), a `MovingWall` zone to
      `with_moving_wall` (lid-driven cavity), and `(BodyForceZone, MovingWall)` composes to the
      combined legacy march — all bit-for-bit.
- [x] Z5 Group gate: format, clippy (0 warnings), full physics tests (1574 pass) + the 3 new
      boundary-zone tests (cargo + bazel); commit message prepared.

## P. Net-flux mixed-BC Leray projection (open-boundary-flux-projection)

- [x] P1 `Manifold::leray_project_open_opts(field, zeroed_edges, prescribed_edges,
      reference_vertices, opts)` extends the constrained Leray / pressure-Poisson solve:
      `prescribed_edges` (inflow) are fixed at their field value, excluded from the free solve, but
      their flux **is counted** in the divergence RHS (full masses); `reference_vertices` (outflow)
      pin `φ = 0`; `zeroed_edges` stay zero. Precision-generic over `R: RealField`.
      (`deep_causality_topology/src/types/manifold/differential/leray.rs`.)
- [x] P2 Well-posedness, with a subtlety found in implementation: masking an inflow edge
      **disconnects** its ghostless inlet vertex from the interior in the free-edge graph, and the
      net flux cannot leave under an all-Neumann gauge — so a prescribed inflow **requires** a
      reference (rejected otherwise). The divergence is enforced only on the reference-reachable
      component (a flood fill over free edges); the inlet ring carries the prescribed velocity with
      its divergence unconstrained (the open-boundary condition). Mass is conserved: outflow flux =
      −inflow flux (pinned by the uniform-channel test).
- [x] P3 Closed-domain reduction gate: with all roles empty the operator equals the plain
      projection, and with only `zeroed_edges` it is **bit-identical** to the constrained
      projection (the source/reference branches are skipped) — `leray_open_tests`
      `open_with_no_boundary_equals_the_plain_projection` /
      `open_with_only_zeroed_edges_equals_the_constrained_projection`; the existing 25 leray +
      constrained tests pass unchanged.
- [x] P4 Group gate: format, clippy (0 warnings), full topology tests both feature configs (1241
      pass) + the 4 new open-boundary tests (cargo + bazel `leray_open_tests`); commit message
      prepared. (User commits per the golden rule.)

## I. Inflow zone (open-boundary-inflow)

- [x] I1 `Inflow<D, R>` zone (`dec/boundary/inflow.rs`): a prescribed wall-normal Dirichlet
      velocity — `collect_lift` sets the face's normal edges to `speed·length`, `collect_prescribed_edges`
      marks them prescribed. Wired through the solver: the rate pins the inflow edges to zero rate
      (`rate_constrained = no_slip ∪ inflow` in `project_raw`), and the velocity re-entry + seed use
      `SolenoidalField::from_open_leray_projection_opts` (prescribed inflow, flux counted). The
      uncertain types never enter the core — folded in `with_zones`.
- [x] I2 Analytic gate (`dec/inflow_outflow_tests`): a uniform inflow into a periodic-y channel
      with an outflow marches to **exact uniform, divergence-free flow** (1e-6), mass-conservative.

## O. Outflow zone (open-boundary-outflow)

- [x] O1 `Outflow<D>` zone (`dec/boundary/outflow.rs`): `collect_reference_vertices` yields the
      outflow face's vertices as the projection's pressure reference (`φ = 0`), so the outflow
      velocity is free and adjusts to balance the inflow flux. The v1 outflow is therefore
      reference-only (zero-gradient via incompressibility); the explicit upwinded convective
      boundary time-update (L5) is a documented refinement behind the same zone (not needed for the
      uniform-channel gate, where zero-gradient is exact).
- [x] O2 Gate: the inflow/outflow channel reaches a steady uniform state with no interior
      reflection and mass in = mass out (the I2 gate exercises both zones together).
- [x] O3 Group gate (I+O): format, clippy (0 warnings), full physics tests (1575 pass) + the
      open-boundary solver gate (cargo + bazel); commit message prepared. The now-dead
      `from_constrained_leray_projection_opts` wrapper was removed (its callers moved to the open
      path); the 105 dec tests stay bit-identical (closed domains reduce through the open path).

## U. Uncertain boundary source — cross-domain generalization (uncertain-boundary-source)

- [ ] U1 `UncertainBoundarySource<R>` wrapping the L3b value channel of any Dirichlet zone:
      per-step `lift_to_uncertain` → collapse → last-good in `State`; dropout → `.intervene` +
      `EffectLog` at a configurable `DropoutVerbosity`. Depends only on `MaybeUncertain<R>` + the
      monad (no fluid concepts) — reusable across domains.
- [ ] U2 Re-express the cut-cells change's `UncertainInflowZone` as `UncertainBoundarySource` over
      an `Inflow`/`MovingWall` zone; its Group-C tests pass **bit-for-bit** through the general
      source.
- [ ] U3 Group gate: format, clippy, full physics + uncertain tests both feature configs; commit
      message.

## V. Validation + handoff (open-boundary-validation)

- [ ] V1 Analytic open-boundary rungs (fast, no heavy march): uniform-channel exactness (I2),
      mass conservation (O2), closed-domain bit-identity (P3).
- [ ] V2 Handoff: confirm the Inflow/Outflow zones express the isolated-cylinder external-flow
      domain (west `Inflow`, east `Outflow`, far-field top/bottom, immersed cut cylinder) — the
      `add-cut-cells-and-immersed-boundaries` D2/D3 Re-ladder consumes these zones (that ladder is
      implemented and gated **in that change**, then it is closed).
- [ ] V3 Change gate: `openspec validate --strict`, format, clippy, full physics + uncertain tests
      both feature configs and bazel; prepare the final commit message; **archive this change**
      (it is the prerequisite that must close before the cut-cells change is closed).
