# Milestone structure

Six groups, each ending green (tests passing on touched crates in both feature configurations,
clippy/fmt clean) with a prepared commit message. Per AGENTS.md golden rules: agents never
`git commit` and never delete files — each group gate prepares a commit message and asks the user
to commit; `make fix` / `make build` / `make test` are run by the user on review.

Reduction-first: the non-breaking gates (Z, P) land and are pinned bit-identical **before** any
new open-boundary behaviour (I, O) is added.

## Z. Boundary-zone abstraction (boundary-zone-abstraction)

- [ ] Z1 `BoundaryZone<D, R>` trait with one collect-into hook per solver stage (L1 metric
      overlay, L2 rate source, L3a constrained edges, L3b time-dependent lift, L4 flux role, L5
      boundary update), each defaulting to a no-op; `FluxRole { Closed, Source, Reference }`.
      **Static dispatch only** — no `dyn`/trait objects.
- [ ] Z2 Static (HKT) composition: a typed tuple/cons is itself a `BoundaryZone` folding each stage
      over its members (`(A, B)`, identity `NoZones`), on the `deep_causality_haft` foundation. The
      solver is generic over the composed zone type `Z: BoundaryZone<D, R>`; `solver.with(zone)`
      yields a solver over `(Zone, Z)`. The step folds each stage's contributions (constraint set =
      ∪ constrained_edges; lift = ∪ lift; rate source = Σ rate_source; metric overlay → geometry).
- [ ] Z3 Re-express the existing BCs as zones (numerics preserved): `NoSlipWall` (L3a),
      `MovingWall` (L3a+L3b), `BodyForce` (L2), `ImmersedBody` (L1 cut registry + L3a solid-incident
      edges). The zone constructors **replace** the ad-hoc entry points (`with_moving_wall`, the
      body-force argument, the cut-registry attachment) — no back-compat shims (nothing is
      published); downstream callers move to the zone API.
- [ ] Z4 Non-breaking gate: a solver carrying the equivalent zones marches **bit-identically** to
      the pre-refactor solver on Poiseuille (no-slip), the lid-driven cavity (moving wall), and an
      immersed solid block (cut body). Wall/periodic paths unchanged.
- [ ] Z5 Group gate: format, clippy, full physics tests both feature configs; commit message.

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

- [ ] I1 `Inflow` zone: prescribed wall-normal Dirichlet (L3a constrained normal edges + L3b lift
      = prescribed `u·n`) reporting `FluxRole::Source` (L4). Uniform and per-edge profiles.
- [ ] I2 Analytic gate: a uniform inflow into a straight channel with an outflow produces uniform
      flow (exact), mass-conservative to the solve tolerance.

## O. Outflow zone (open-boundary-outflow)

- [ ] O1 `Outflow` zone: convective / zero-gradient boundary time-update (L5, upwinded
      extrapolation from the interior) reporting `FluxRole::Reference` (L4) — not pinned in the
      constraint set.
- [ ] O2 Gate: an inflow/outflow channel reaches a steady uniform state without spurious boundary
      reflections corrupting the interior; mass in = mass out each step.
- [ ] O3 Group gate (I+O): format, clippy, full physics tests both feature configs; commit message.

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
