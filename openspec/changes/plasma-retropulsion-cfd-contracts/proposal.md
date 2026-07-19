## Why

The plasma-retropulsion roadmap (`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md`,
milestone M2 / design-note Stage 0) requires the cfd-side seams — a throttle command channel, carried
propulsion state, an extended safety envelope, a full-height atmosphere, and the onboard weather-table
loader — before any burn stage, guidance law, or example can be built. Landing these contracts now, with
an inert stub behind the propulsion seam and a bit-identical corridor inheritance guard, lets M3–M5
build against fixed seams instead of placeholders, and proves the extension "only appends": the flown
corridor example must not change by one bit.

## What Changes

- `CoupledField` gains a second typed command channel (throttle) beside the existing bank
  `control_action`, plus an additive read-modify-write helper on the aero-force channel so a thrust
  term composes with — never clobbers — the lift stage's ④ vector.
- The propulsion coupling contract is pinned: `mass`, `propellant`, and `ignited` ride the coupled
  field as named scalars; `commanded_throttle` is world-published exactly like `commanded_bank`;
  pre-ignition carried mass reproduces the corridor's implied `CDA_OVER_M` bundle.
- An **inert A0 propulsion stub stage** (`deep_causality_cfd`) satisfies the contract behind the
  future `PlumeObstruction`/`RetroThrust` seam: at zero throttle it contributes nothing (no force,
  no scalars mutated, no log entries); at nonzero throttle it exercises thrust-RMW, mass depletion,
  and the Jarvinen–Adams force-channel (A0) drag decrement via existing physics kernels. Swapping it
  for the M3 production stages changes no consumer. The M1 verdict
  (`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, measured 2026-07-17, AMBER) pins
  **A0 as the drag authority** — the stub's correlation channel is not a placeholder but the
  committed closure — and the marched-layer imprint seam it fronts already exists in the library
  (`ForcingRegion`, landed by `plasma-retropulsion-de-risk`) for M3's state-realism use.
- `SafetyEnvelope` gains optional, inactive-by-default burn axes (throttle floor/ceiling, dynamic
  C_T cap, ignition dynamic-pressure window, propellant floor, descent-rate bound) enforced by the
  same `CyberneticCorrect` sense → clamp → `Err` pattern; with the axes inactive the gate is
  bit-identical to today.
- The shared `ATMOSPHERE` table extends from its 30 km floor to ~0 km with US-1976-shaped rows
  appended below the existing five (which stay byte-identical).
- A weather-table loader seam in `avionics_examples::shared` reads `weather_table.csv` through the
  existing `deep_causality_file` typed reader, sorts by `d_temp`, rejects duplicate keys,
  interpolates at the measured dT by value-bracketing, and clamps out-of-range dT with a marker the
  flight side stamps into provenance.
- A **corridor inheritance guard**: the corridor example re-runs bit-identically with the appended
  atmosphere, and a harness proves a corridor-class coupled march with the inert stub composed is
  bit-identical to one without it.

No breaking changes: every new channel/axis defaults to absent/inactive, and existing couplings are
unaffected by construction.

## Capabilities

### New Capabilities

- `powered-descent-envelope`: the extended safety envelope for powered descent — optional burn axes
  on `SafetyEnvelope`, their enforcement semantics in `CyberneticCorrect`, and the
  inactive-axes-change-nothing guarantee.
- `full-descent-atmosphere`: the shared atmosphere table extended to ~0 km — US-1976 shape, row
  ordering/format contract, existing rows byte-identical, sampler clamp behavior preserved.
- `weather-table-consumption`: the onboard loader — typed read of the recorded dispersion table,
  sort/dedupe/bracket-by-value/clamp contract, dT interpolation, and the clamp-provenance marker.
- `corridor-inheritance-guard`: the standing bit-identity gate — the flown corridor reproduces its
  committed witnesses after this change (and future milestones re-run it), and the inert propulsion
  stub is proven to be a no-op in a coupled march.

### Modified Capabilities

- `blackout-coupling-interface`: the coupling seam gains the second typed command channel
  (throttle), the additive aero-force read-modify-write contract, the propulsion state scalars +
  `commanded_throttle` publication, and the inert A0 propulsion stub stage requirement (mirroring
  the existing stub-stage requirement).

## Impact

- `deep_causality_cfd`: `types/flow/coupling.rs` (`CoupledField` — throttle channel, force RMW
  helper), `types/flow/corridor.rs` (`SafetyEnvelope` burn axes, `CyberneticCorrect` enforcement),
  new `types/flow/retropulsion.rs` (burn-axis types + A0 propulsion stub), root re-exports in
  `lib.rs`, mirrored tests + `tests/BUILD.bazel` registration.
- `deep_causality_physics`: consumed only (existing propulsion kernels); no changes.
- `examples/avionics_examples`: `src/shared/constants.rs` (`ATMOSPHERE` rows below 30 km), new
  `src/shared/weather_table.rs` loader module + `shared/mod.rs` registration; the corridor and
  weather examples themselves are untouched except for re-verification.
- Downstream: M3 (`RetroThrust`/`PlumeObstruction`), M4 (`ThrottleGuidance`), M5 (example wiring)
  build against these seams unchanged; M1 (`plasma-retropulsion-de-risk`, implemented and
  measured 2026-07-17) had no file overlap, pinned the shared `"commanded_throttle"` name
  identically, and decided the depth behind the stub seam: **A0** (verdict AMBER on imprint
  fidelity; fork economics and rank green — see `derisk-verdict.md`).
