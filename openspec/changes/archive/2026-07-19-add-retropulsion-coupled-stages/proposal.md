## Why

The plasma-retropulsion roadmap
(`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md`, milestone M3 /
design-note Stage 3, physics half) makes the burn *physically couple*. M1 landed and measured the
forcing seam (verdict **AMBER**: the A0 Jarvinen–Adams correlation is the drag authority, not a
field-contracted decrement — `derisk-verdict.md`), and M2 landed the contracts every burn stage
composes against: the throttle command channel, the `mass`/`propellant`/`ignited` scalars, the
additive `add_aero_force` idiom, the `BurnEnvelope` gate axes, and the inert `PropulsionStub` seam.
This change fills that seam with production stages, so a burn leg feels thrust in the force channel
and the IMU, carries the plume's drag decrement from the cited correlation, and detects and logs
the new flight regimes — with nothing yet wired into a flying example (M5) and no guidance law or
terminal leg (M4).

The M1 verdict is resolved, not open: `PlumeObstruction` carries the **A0 correlation as the drag
authority**; a marched-layer imprint through the landed `ForcingRegion` seam is optional
state-realism only and never the force-channel drag closure. This is the exact depth the M2
`PropulsionStub` already stubbed — M3 productionizes it behind the same contract, so swapping the
stub for these stages changes no consumer.

## What Changes

- A production **`RetroThrust`** stage (`deep_causality_cfd`) replaces the thrust half of
  `PropulsionStub`: it forms the retro-thrust acceleration `−T/m·v̂` from the commanded throttle
  and the carried mass, composes it onto the aero-force channel with `add_aero_force` (after the
  lift stage, before the force consumers — the M2 order contract), depletes `propellant` and
  reduces `mass` by `ṁ·Δt` (`ṁ` from `propellant_mass_flow_kernel`), and sets `ignited`. It is
  **strictly inert at throttle ≤ 0** (the `PropulsionStub` inertness contract), so a burn-phase
  stack carries it from the start and ignition stays a published-command event. The IMU senses the
  burn with no new code — the ESKF specific-force term already reads the summed force.
- A production **`PlumeObstruction`** stage carries the **A0 drag decrement** as the drag
  authority: it derives `C_T` from the world's `commanded_throttle` (through
  `momentum_flux_ratio_kernel` → `srp_thrust_coefficient_kernel`), reads
  `srp_preserved_drag_fraction_kernel`, and scales the forebody drag on the force channel by the
  preserved fraction. It **may** additionally imprint the plume on the marched compressible layer
  through the landed `ForcingRegion` seam (mask from `cordell_braun_plume_boundary_kernel` at that
  world's `C_T`) — but that imprint is **state realism only** and MUST NOT replace the correlation
  as the force-channel drag closure (the AMBER verdict). The correlation is a cited authority in
  flight, cross-checked against M1's measured band.
- **Flight-regime classifier axes** extend `RegimeClass` / `RegimeClassify`: new Mach-regime
  (supersonic / transonic / subsonic from the published `flight_mach`), thrust-state (coast / burn
  from `ignited`), and touchdown (above / at the altitude floor from `flight_altitude`) axes, folded
  into the regime `key()` so a Mach crossing, a burn↔coast transition, or a touchdown is a logged
  regime change. This emits transitions 4–6 of the design note's §2 inventory into the provenance
  log alongside the existing rarefaction / comms-denial transitions.
- A **burn-leg harness** (test, not an example) proves the coupled physics: a corridor-class world
  marched through a commanded burn shows the ordered regime cascade in provenance, an in-flight
  contracted decrement inside M1's band, and step-integrity across the leg — extending the M2
  inheritance-guard harness pattern.

No breaking changes: `RetroThrust` and `PlumeObstruction` are new stages behind the seam
`PropulsionStub` already occupies; the classifier gains fields and `key()` terms additively, so the
corridor's existing rarefaction/comms classification is unchanged when the new scalars are absent.

## Capabilities

### New Capabilities

- `retro-thrust-stage`: the production retro-thrust `PhysicsStage` — thrust force composition,
  propellant/mass depletion, the `ignited` flag, zero-throttle inertness, and IMU sensing through
  the existing specific-force seam.
- `plume-obstruction-stage`: the production plume `PhysicsStage` — the A0 correlation as the
  in-flight drag authority, the optional state-realism `ForcingRegion` imprint that never overrides
  it, and the in-flight-decrement-matches-M1-band cross-check.
- `flight-regime-classifier`: the Mach / thrust / touchdown classifier axes on
  `RegimeClass` / `RegimeClassify`, their `key()` transitions, and the regime cascade in provenance.

## Impact

- `deep_causality_cfd`: new `types/flow/retropulsion.rs` stages (`RetroThrust`, `PlumeObstruction`)
  beside the M2 `PropulsionStub`; `types/flow/corridor.rs` (`RegimeClass` fields, `RegimeClassify`
  axes + `key()`); root re-exports in `lib.rs`; mirrored tests + `tests/BUILD.bazel` registration;
  a burn-leg inheritance/cascade harness test.
- `deep_causality_physics`: consumed only (existing propulsion kernels: `propellant_mass_flow`,
  `momentum_flux_ratio`, `srp_thrust_coefficient`, `srp_preserved_drag_fraction`,
  `cordell_braun_plume_boundary`); no changes.
- Downstream: M4 (`add-retropulsion-terminal-descent`) composes `ThrottleGuidance` above these
  stages and wires live `CyberneticCorrect` burn-axis enforcement; M5 wires the example. M1's
  `ForcingRegion` seam and M2's throttle/scalar/force-RMW/envelope contracts are consumed unchanged.
