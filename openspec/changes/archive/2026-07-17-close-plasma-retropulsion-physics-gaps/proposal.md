## Why

The plasma-retropulsion descent (designed in
`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-descent.md`) needs a propulsion
physics family that the repo does not have: a workspace-wide audit confirmed zero thrust, Isp,
propellant, rocket-equation, SRP, or powered-descent code anywhere. This change is Stage 1 of
the note's build order — the pointwise kernels land in `deep_causality_physics`, cited and
validated in isolation, before any CFD stage or example consumes them. Everything here is a
prerequisite for Stage 2's measured de-risking (the SRP drag-decrement verification target and
the plume rank / fork-economics study).

## What Changes

- A new kernel family `deep_causality_physics/src/kernels/propulsion/` with five topic files
  (performance, nozzle, srp, plume, descent), a `wrappers.rs` of `PropagatingEffect` lifts, new
  quantities under `src/quantities/propulsion/`, and digitized published coefficients under
  `src/constants/propulsion.rs`.
- **Rocket performance**: propellant mass flow from Isp (`ṁ = T/(Isp·g₀)`) and the Tsiolkovsky
  Δv relation (sizes the propellant reserve the weather-table margin demands).
- **Nozzle exit state**: an inverse area-Mach kernel (branch-selected; the forward relation
  exists as `area_mach_ratio_kernel`) and an exit-state composition over the existing
  Mach-parameterized isentropic kernels — the unstated input that makes the momentum-flux ratio
  computable from a commanded throttle.
- **SRP similarity and correlation**: freestream-normalized thrust coefficient `C_T`
  (deliberately named apart from the duct solver's nozzle `C_f`), jet-to-freestream
  momentum-flux ratio, the digitized Jarvinen–Adams central-nozzle preserved-drag correlation
  with its low-C_T drag-collapse structure, the unpowered baseline axial coefficient from the
  same dataset, the total-axial-force composition, and the stability margin against the
  published C_T ≈ 3 bow-shock instability bound.
- **Cordell–Braun plume geometry**: the analytic plume-as-effective-obstruction kernel (max
  plume radius, penetration length, terminal-shock standoff) from exit state plus freestream,
  on-axis, with a typed `PlumeGeometry` output — the geometry the CFD-side `PlumeObstruction`
  stage will imprint per step.
- **Powered-descent kinematics**: stopping distance / ignition altitude (rejecting net
  deceleration ≤ 0, the thrust-to-weight ≤ 1 case) and the suicide-burn commanded deceleration
  `a = v²/(2h) + g`.
- Source PDFs land in `deep_causality_physics/papers/`; every kernel docstring carries the full
  reference with table/equation pinpoints.

Out of scope: all CFD-side stages (`RetroThrust`, `PlumeObstruction`, classifier axes,
envelope, guidance — later changes per the note's §9); the atmosphere table extension
(example-local per the note); `thrust = throttle · T_max` (trivial stage arithmetic, not a
kernel); the adjacent hand-roll cleanup the audit surfaced in the duct solver and nozzle
example (pressure-parameterized isentropics, (p, ρ) sound speed, normal-shock ratio set) —
noticed, listed in the design, deliberately a separate change; Lean/THEOREM_MAP witnesses.

## Capabilities

### New Capabilities

- `propulsion-performance-kernels`: rocket performance and nozzle exit state — mass flow from
  Isp, Tsiolkovsky Δv, inverse area-Mach, and the exit-state composition.
- `srp-aero-kernels`: supersonic-retropropulsion similarity numbers, the Jarvinen–Adams
  drag correlation family, the stability bound, and the Cordell–Braun plume-boundary geometry.
- `powered-descent-kernels`: closed-form powered-descent kinematics — stopping distance,
  ignition altitude, and the suicide-burn deceleration command.

### Modified Capabilities

None. The family is greenfield; no existing kernel's requirements change, and the existing
compressible-flow kernels are reused, never duplicated.

## Impact

- **Affected code**: `deep_causality_physics` only — new modules under `src/kernels/propulsion/`,
  `src/quantities/propulsion/`, `src/constants/propulsion.rs`, registration in `kernels/mod.rs`,
  `quantities/mod.rs`, `constants/mod.rs`, `lib.rs`; mirrored tests under
  `tests/kernels/propulsion/` and a new `rust_test_suite` in `tests/BUILD.bazel`. No other crate
  is touched; no new external dependencies; `no_std` discipline (core/alloc) holds.
- **Downstream (unblocked, not modified)**: the Stage-2 verification target
  (`verification/srp_drag_decrement`), the plume rank study, and the CFD-side stages all consume
  this family later.
- **Verification**: `cargo test -p deep_causality_physics` (pointwise vs published values, limit
  cases, every rejection path; 100% coverage of added code), `bazel test //...`,
  `make format && make fix`.
