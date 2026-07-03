## Context

The plasma-blackout corridor shipped on the incompressible QTT carrier
(`2026-07-02-add-plasma-blackout-corridor`): the post-shock state is reconstructed
(`RecoveryTemperatureStage` + `VibrationalLagStage` over an imposed RH jump), each flight station
hand-carries `n_tot`/`pressure_atm`/`gamma_eff`, the blackout window switches at station
boundaries, and the clamped bank command audits but does not steer. The compressible side already
exists as validated lib code: `CompressibleMarcher3dFitted` marches the two-temperature
`EulerStateTt3d` on the `BodyFittedCoordinate3d` shell, `FittedNormalShock` provides the exact RH
interface, `Park2tClosure` the calibrated chemistry controller, and the shock-rank studies
measured the body-fitted rank lever at χ ~ O(10). The `CfdFlow` coupled-loop machinery
(`run_coupled`, `run_until`, `MarchPause`, O(1) fork, the verbatim alternation vocabulary) is
bound to the incompressible `QttMarchRun`; the stage stack itself is marcher-agnostic (stages read
named `CoupledField` scalars).

Constraints: no `dyn`, no `unsafe`, precision as a parameter, workspace lints, tests mirror src
with Bazel registration, papers cited in kernel docstrings, and the corridor's minutes-not-hours
runtime budget.

## Goals / Non-Goals

**Goals:**
- One coupled-loop machinery, two carriers: host the compressible two-temperature marcher behind
  the same config→run split, coupling stack, and pause/fork/alternation contract.
- Evolved-state provenance: `T_tr`, `T_ve`, and per-cell `n_tot` are read off the marched state;
  the reconstruction stages disappear from the compressible corridor.
- Station constants collapse to freestream plus geometry; the flagship becomes one continuous
  descent with flow-resolved blackout onset and exit.
- `BankSteeredLift`: the clamped bank command actuates a 3-DOF lift vector, steering truth and
  navigation; branch miss distances become trajectory-derived.
- Keep the RAM-C II peak-`n_e` anchor gate; add onset/exit and steering gates.

**Non-Goals:**
- 6-DOF attitude dynamics, aero moment models, or coefficient databases (no RAM-C-equivalent
  flight anchor for the 6-DOF side; the validation burden buys nothing for the blackout story).
- New chemistry beyond the existing Park-2T closure and LER stages.
- DEC-solver or physics-kernel changes; GPU/parallel acceleration.
- Real-time (uncompressed) descent duration; the corridor remains a compressed-time demonstration
  with the mapping labeled.

## Decisions

**D1 — Extract a marcher seam; do not fork the pause machinery.** The resumable-loop code
(`MarchPause`/`MarchFork`, CoW `Arc`s, error-channel capture, alternation markers) is the
subtlest code in the DSL; two diverging copies would be a maintenance defect factory. Introduce a
small `pub(crate)` seam trait (working name `CoupledMarcher`): associated `State`, `advance`,
`publish` (state projections into the `CoupledField`), `transport` (the carried scalar), and
`finish` (final report fields). `QttMarchRun` refactors onto the seam with bit-identical behavior
(the existing DSL-equivalence and pause tests catch drift); the compressible host is a second
implementation. Alternative considered: a sibling copy of the host files — rejected for the
divergence risk; the seam is four methods.

**D2 — Evolved projections replace reconstruction.** The compressible host publishes per-cell
`"speed"`, `"T_tr"` (EOS), `"T_ve"` (from the vibrational energy), and `"n_tot"` (from density).
The corridor stack drops `RecoveryTemperatureStage` and `VibrationalLagStage`; a thin
`RateControllerStage` computes `"T_a" = √(T_tr·T_ve)` from the two evolved fields.
`IonizationStage` gains per-cell density support (`with_density_field("n_tot")`), keeping the
scalar-config constructor for the QTT surrogate path. Sheath renewal becomes optional on this
carrier: real compressible through-flow renews parcels physically; whether the explicit renewal
mode is still needed is settled empirically at re-pin (see Open Questions).

**D3 — Continuous descent as a freestream schedule closed through the truth vehicle.** A small,
cited US-Standard-Atmosphere table (altitude → `ρ_∞, T_∞, a_∞`) plus the truth vehicle's actual
position/speed drive the marcher's freestream each step: navigation feeds flow, flow feeds
navigation — the corridor's two-way coupling made real. The marcher's inflow is fixed at
construction, so the host rebuilds the solver when the scheduled freestream drifts past a
configured tolerance (assembly is cheap relative to marching; rebuild count is logged to
provenance). Alternative considered: a `set_freestream` mutator on the marcher — more invasive to
a validated solver, and rebuild-on-drift keeps the solver immutable.

**D4 — `BankSteeredLift` is a lib corridor stage.** Point-mass aerodynamics: drag along `−û`,
lift `L = (L/D)·D` rotated about the velocity vector by the clamped bank angle
(`n̂` from the local radial direction, `b̂ = û × n̂`). It reads the control channel
(`CyberneticCorrect`'s clamped command from the previous step — a one-step actuation lag, the
standard operator split) and the truth state, and writes the full 3-vector into the ④ aero
channel that both `TruthGnss` and `TrajectoryNav` consume. Placement: before the truth/nav stages
in the stack. Plain 3-vector algebra suffices; the `deep_causality_num` quaternion type is
available if the rotation is later generalized, but a single-axis rotation does not need it.

**D5 — Trajectory-derived branch outcomes.** Coupled reports gain the terminal navigation and
truth states (written by the report finisher from the field), so `BranchOutcome.miss_distance`
becomes `|r_terminal − r_aim|` per branch instead of the t²-law proxy. The t²-law remains only as
a cross-check printed beside the real miss.

**D6 — Gates: keep the anchor, add structure.** The RAM-C II peak-`n_e` gate (5× band around
1e19 m⁻³) is retained and must hold as the descent sweeps the 61 km condition. New gates: onset
and exit are *events* found by the run (not station switches), ordered onset → dwell → exit with
nonzero dwell; the committed branch's steered trajectory diverges measurably from the zero-bank
branch (steering is real); miss distances differ across branches; bond stays under the cap;
wall-clock stays inside the minutes budget.

**D0 — De-risk runtime first.** Before any host code: a study run measuring per-step wall-clock
of `CompressibleMarcher3dFitted` at candidate corridor grids/bond caps, with a go/no-go grid
choice recorded in the study output. This is Task 0 because every later estimate hangs on it.

> **D0 outcome (measured, `studies/compressible_carrier_timing`).** The 3-D fitted marcher is
> over budget by 3.6× at the *smallest* candidate (10.7 s/step at 16³/cap-16, 35.6 min
> projected), so the pre-declared fallback applies: **the corridor carrier is
> `CompressibleMarcher2d`** (0.026–0.174 s/step across 32²–64², caps 16–32; the corridor stage
> stack is already `PhysicsStage<2, _>`). The carrier marches the sheath layer in a 2-D chart
> with the exact `FittedNormalShock` Rankine-Hugoniot state as its scheduled inflow — the shock
> jump is the *boundary* of the marched layer (shock-fitted, not reconstructed), and the layer
> structure, transport, and relaxation are evolved. Assembly is ~free (one rebuild ≈ 0.01 steps),
> so the freestream-drift tolerance defaults tight. The 3-D fitted marcher remains the
> stagnation-line/validation tool.

## Risks / Trade-offs

- [3-D fitted marcher too slow for the corridor budget] → Task-0 study picks the grid/bond
  configuration first; the shell grid is small by construction (χ ~ O(10) measured); if the 3-D
  budget fails, the 2-D compressible marcher (`CompressibleMarcher2d`) is the documented fallback
  carrier with the same evolved-state provenance.
- [Freestream rebuild-on-drift thrashes (rebuilds every step)] → tolerance is config; rebuild
  count is logged and gated (a re-pin assertion caps it); worst case amortizes assembly over a
  leg-sized step span.
- [Carried `alpha` double-counts renewal (explicit renewal + real through-flow)] → A/B at re-pin
  with renewal on/off; the report series make the difference visible; keep whichever matches the
  stagnation-closure calibration, document the other.
- [Compressed-time mapping confuses readers (a real descent takes ~100 s; the carrier marches
  far less)] → the schedule mapping is one labeled constant, printed in the intro banner, with
  the same relative-comparison gate philosophy as the shipped corridor.
- [Seam refactor destabilizes the shipped QTT host] → the refactor lands as its own task gated
  by the existing equivalence/pause/alternation tests before any compressible code follows.
- [One-step actuation lag on the bank command] → inherent to the between-step split; documented;
  at corridor `dt` it is far below the guidance timescale.

## Migration Plan

Additive throughout: the QTT host keeps its public API (the seam is `pub(crate)`); the flagship
example is rewired in place with its gates re-pinned; the archived three-leg behavior remains
reproducible from the archive. Rollback is dropping the new host and stage; nothing existing
changes contract.

## Open Questions

- Does the compressible carrier still need explicit sheath renewal, or does real through-flow
  reproduce the residence-limited exposure? (Settled empirically at re-pin; D2.)
- Wall heat flux on the fitted shell: Fourier at the wall from the evolved temperature vs the
  Brinkman integral analog — pick during the observable task based on what the fitted geometry
  exposes cleanly.
- Freestream-drift tolerance default (rebuild cadence): set from the Task-0 study's assembly-cost
  measurement.
