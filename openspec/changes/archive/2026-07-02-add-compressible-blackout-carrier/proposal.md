## Why

The archived plasma-blackout corridor (`2026-07-02-add-plasma-blackout-corridor`) rides an
*incompressible* QTT carrier: the post-shock state is a recovery-temperature reconstruction over an
imposed Rankine-Hugoniot jump, every flight station carries hand-computed `n_tot` / `pressure_atm` /
`gamma_eff` constants, the blackout window is station-switched rather than flow-resolved, and the
bounded bank command is demonstrative (it steers nothing, so branch miss distances fall back to a
t²-law proxy). The compressible two-temperature machinery that removes all of this already exists
as validated lib code (`CompressibleMarcher3dFitted`, `EulerStateTt3d`, `FittedNormalShock`,
`Park2tClosure`, the body-fitted shock-rank studies); what is missing is a `CfdFlow` host for it,
plus a small 3-DOF lift stage so the clamped bank command actually steers. Peak `n_e` accuracy is
already at the RAM-C II anchor (1.03e19 vs 1e19); this change upgrades the *provenance and
structure* of that result, not the number.

## What Changes

- Host the compressible two-temperature marcher in the `CfdFlow` DSL: a config/builder, a coupled
  run (`run_coupled`), and the resumable loop (`run_until` → pause → O(1) fork →
  `alternate_context` → `continue_march`) over `EulerStateTt` state, mirroring the QTT
  incompressible host's machinery.
- The coupled loop publishes *evolved* state projections into the `CoupledField` (`"speed"`,
  `"T_tr"`, `"T_ve"` come from the marched state), so the corridor drops the
  recovery-temperature reconstruction and the per-step vibrational-lag stage; ionization reads the
  evolved controller.
- Station constants collapse to freestream plus geometry: `n_tot`, `pressure_atm`, and `gamma_eff`
  become post-shock *outputs* of the marched state instead of hand-computed inputs.
- **`BankSteeredLift`, a dedicated stage**: 3-DOF point-mass aerodynamics with an `L/D` value; the
  bank angle read from the control channel (the one `CyberneticCorrect` already clamps) rotates the
  lift vector about the velocity vector. The ④ channel becomes a full 3-vector (drag + steered
  lift), so guidance genuinely steers both the truth vehicle and the navigation, and branch miss
  distances become trajectory-derived.
- The flagship becomes **one continuous descent** over a time-varying freestream schedule: blackout
  onset and exit are flow-resolved (the marched sheath state crosses the L1 cutoff), replacing the
  three-leg station-switch structure.
- Out of scope, explicitly: 6-DOF attitude dynamics (no RAM-C-equivalent flight-data anchor exists
  for the 6-DOF side; not worth the validation burden), aero moment/coefficient databases, and any
  new chemistry beyond the existing Park-2T closure.

## Capabilities

### New Capabilities
- `compressible-flow-host`: the `CfdFlow` DSL host for the compressible two-temperature marcher —
  owned config + builder, the coupled per-step loop over `EulerStateTt` state with evolved-state
  projections into the `CoupledField`, and the full resumable/forkable counterfactual machinery
  (`run_until`, `MarchPause`, `fork`, `alternate_context`, `continue_march`).
- `bank-steered-lift`: the 3-DOF bank-steered lift stage — point-mass drag + lift with the lift
  vector rotated about the velocity vector by the clamped bank command, actuating the ④ aero
  channel for both truth and navigation.

### Modified Capabilities
- `blackout-coupling-interface`: the ④ physics→navigation channel is extended from a drag-only
  kick to a full 3-vector aero acceleration with a bank-steered lift component; the control channel
  becomes an *actuating* input (read by the lift stage), not just an audited output.
- `plasma-blackout-flagship`: the flagship's required structure changes from three station-switched
  legs to one continuous descent on the compressible carrier — flow-resolved blackout onset/exit,
  evolved (not reconstructed) `T_tr`/`T_ve`/`n_e`, and trajectory-derived branch miss distances.

## Impact

- `deep_causality_cfd/src/types/flow_config/`: new compressible march config + builder (owned
  container, same config→run split).
- `deep_causality_cfd/src/types/flow/`: new compressible run host + pause/fork (mirroring
  `qtt_march_run.rs` / `qtt_march_pause.rs`, either generalized over a small solver seam or as a
  sibling); `coupling.rs` ④ channel extension; new `BankSteeredLift` stage in `corridor.rs`.
- `deep_causality_cfd/src/solvers/qtt/compressible/`: consumed as-is; a body-fitted wall-heat-flux
  observable may be added beside the existing observables.
- `examples/avionics_examples/cfd/plasma_blackout_corridor/`: rewired to the compressible host and
  the continuous descent; station constants shrink to freestream + geometry; gates re-pinned (the
  RAM-C II `n_e` anchor gate is retained).
- Runtime risk: the 3-D fitted marcher's per-step cost at corridor grid sizes must satisfy the
  minutes-not-hours budget; de-risked by a study run before the host lands (Task 0).
- No new external dependencies; no changes to `deep_causality_physics` kernels.
