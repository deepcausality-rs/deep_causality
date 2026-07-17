## Context

Roadmap M2 (design-note Stage 0): the contract layer between the finished physics kernels
(archived `close-plasma-retropulsion-physics-gaps`) and the future burn stages (M3), guidance (M4),
and example (M5). Everything here is seam work against verified anchors in the current tree:

- `CoupledField` (`deep_causality_cfd/src/types/flow/coupling.rs:134-142`) carries `ambient`,
  named `scalars`, `aero_force: Option<[R;3]>`, `control_action: Option<R>` (the bank),
  `regime`, `nav`, `log`. `set_aero_force` **overwrites** (`coupling.rs:215-217`).
- The corridor stack (`examples/avionics_examples/src/shared/world.rs:121-181`) orders
  `BankSteeredLift` (writes the full ④ vector, `corridor.rs:522`) before `SuttonGravesLoads`,
  `TruthGnss`, `TrajectoryNav` — the three force consumers.
- `SafetyEnvelope` (`corridor.rs:535-553`) holds three limits; `CyberneticCorrect`
  (`corridor.rs:637-712`) senses configurable scalar fields, runs one `CyberneticLoop`
  observe/decide, clamps the bank channel, and `Err`s on unrecoverable breach.
- `ATMOSPHERE` (`shared/constants.rs:78-84`) floors at 30 km; `DescentSchedule::sample`
  clamps to the *table ends* (`compressible_march_config.rs:125-150`), not to a literal.
- `publish_constant` lands world constants on the field at the top of every `pre_step`
  (`compressible_march_run.rs:195-199`); `commanded_bank` is the proven counterfactual seam.
- The weather table (`cfd/plasma_blackout/weather/weather_table.csv`) is written by
  `write_rows` with `WorldRow::SCHEMA` (12 columns); rows arrive in **run order**
  (0, +20, −25, −40, −5, +5 K), not temperature order. `avionics_examples` already depends on
  `deep_causality_file`, whose `read_rows::<T: FromTableRow>` delivers cells in schema order.

Constraint that shapes everything: the corridor example is a flown, gated artifact with a
committed `output.txt`. This change must leave it **bit-identical**.

## Goals / Non-Goals

**Goals:**

- Land every seam M3–M5 build against: throttle channel, force RMW, propulsion state scalars,
  burn envelope axes, full-height atmosphere, weather loader.
- Prove inertness: with all seams landed and the stub composed at zero throttle, a coupled
  march is bit-identical to one without the stub, and the corridor example reproduces its
  committed witnesses.
- Keep the A0/A coupling-depth decision open: the stub realizes A0 (force-channel correlation);
  M1's verdict decides what M3 puts behind the same seam. Nothing here commits either way.

**Non-Goals:**

- No production `RetroThrust`/`PlumeObstruction` (M3), no `ThrottleGuidance` or live envelope
  sensing wired into a flying stack (M4), no retropulsion example folder (M5).
- No Mach/thrust/touchdown classifier axes (M3 owns `RegimeClassify`).
- No compressible forcing region (M1 owns that seam and its go/no-go).
- No corridor or weather example behavior change of any kind.

## Decisions

**D1 — Throttle is a second typed channel, not a scalars entry.**
`CoupledField` gains `throttle_action: Option<R>` with `throttle_action()`/`set_throttle_action()`,
mirroring `control_action` exactly. Rationale: commands are per-step actuating values consumed
one stage downstream (the bank precedent); a typed channel keeps the two-axis command bus explicit
and `None`-defaulted (existing couplings untouched). Alternative — a `"throttle"` scalars entry —
rejected: scalars are field data, and the bank/throttle asymmetry would leak into every consumer.
The bank channel keeps its name (`control_action`); renaming it is churn with no behavior gain.

**D2 — Propulsion *state* rides the named scalars.**
`mass`, `propellant`, `ignited` are named scalar fields (`set_scalar`), pinned by name in the
spec. Rationale: they are carried state, exactly what `scalars` exists for; the pause/fork
snapshot serializes them automatically (`coupling.rs:178-180`), which the M5 state-fork needs for
free. `commanded_throttle` is a world-published constant read each `pre_step`, the exact
`commanded_bank` mechanism — so a counterfactual branch's throttle intervention needs zero new
machinery.

**D3 — Force composition is an additive helper, not a stage convention.**
`CoupledField::add_aero_force(delta: [R;3])` reads-modifies-writes the force channel (treating
`None` as zero). Rationale: `set_aero_force` overwrites; leaving RMW as an idiom every thrust
stage hand-rolls invites a silent clobber bug the survey already flagged. One helper makes the
contract enforceable and testable. Stage order is pinned by spec: additive force producers
compose after `BankSteeredLift` and before `SuttonGravesLoads`/`TruthGnss`/`TrajectoryNav`.

**D4 — The stub is a library stage in a new `types/flow/retropulsion.rs`.**
`PropulsionStub` follows the `AeroBlackoutStub` precedent (a library stub behind the coupling
contract, `coupling.rs`). New module rather than more mass in `corridor.rs`: M3's production
stages land in the same file family. Semantics: reads `commanded_throttle` (published) into the
throttle channel's default when no guidance stage wrote one; at throttle ≤ 0 it does *nothing*
(no force write, no scalar mutation, no log entry — inertness is the contract); at throttle > 0
it depletes propellant via `propellant_mass_flow`, adds `−T/m·v̂` via `add_aero_force`, and
applies the A0 drag decrement through the existing `srp_thrust_coefficient` /
`srp_preserved_drag_fraction` kernels. It exists to *validate the seams*, not to fly: M3 replaces
it behind the same contract.

**D5 — Burn axes are an optional sub-struct, not six new required fields.**
`SafetyEnvelope` gains `burn: Option<BurnEnvelope<R>>`; the existing 3-arg `new()` yields
`burn: None`. `BurnEnvelope` carries throttle floor/ceiling, `max_ct` (the dynamic cap:
admissible throttle ceiling is the static ceiling min'd with the throttle at which
C_T = max_ct given sensed q∞, thrust reference, and S_ref), the ignition q window, the
propellant floor, and the descent-rate bound. `CyberneticCorrect` enforcement: with
`burn: None` **or** no throttle channel present, the gate's behavior is bit-identical to today
(the inheritance guard checks this); with axes active it senses via configurable field names
(the `heat_flux_field` pattern), clamps the throttle channel, and returns the same
`Err(PhysicalInvariantBroken)` on unrecoverable breach (propellant below floor, descent rate
above bound). M2 validates enforcement with synthetic scalar fields in unit tests; M4 wires
live sensing. Alternative — a second gate stage — rejected: the design note pins "the same
`CyberneticCorrect` pattern", and one gate owning one envelope keeps the breach semantics single.

**D6 — Atmosphere rows extend the existing array; values verified against US-1976.**
Six new rows (0, 5, 10, 15, 20, 25 km) prepend the array in ascending order; the existing five
rows stay byte-identical. `DescentSchedule::sample` needs no change — it clamps to table ends,
so the clamp point moves from 30 km to 0 km by data alone. The corridor's marched legs sample
well above 30 km (terminal state 47.6 km), so its sampling is untouched; the guard proves it.
Row values are transcribed from the published US Standard Atmosphere 1976 (n_tot, T, a
consistent: a = √(γRT) at γ = 1.4), each row commented with its table pinpoint.

**D7 — The loader uses `typed-table-io`, not the note's "~20-line std parser".**
Deviation from design-note §7, documented here: `avionics_examples` already depends on
`deep_causality_file`, and the table was *written* by `write_rows` with `WorldRow::SCHEMA` —
reading it back through `read_rows` with a `FromTableRow` consumption row type reuses the
schema-order, missing-column-by-name machinery the repo already verifies. Hand-rolling a parser
would duplicate a tested seam (AGENTS.md: land at the right abstraction level). The loader lives
in `avionics_examples::shared` (new `shared/weather_table.rs`) because the retropulsion example
folder only arrives in M5 and the guard/tests need the loader now.

**D8 — The loader is pure; provenance stamping stays with the flight side.**
The loader returns an interpolation result carrying the interpolated row, the bracketing pair,
and a `clamped` marker; it performs no logging. Rationale: `EffectLog` lives on the coupled
field, which the loader (a config-time artifact consumer) never sees; returning the marker keeps
the "clamp stamped into provenance" requirement testable at both layers without coupling the
loader to the flight stack. Contract (from the note §4, verified against the real CSV): sort
ascending by `d_temp` after load; reject duplicate keys; bracket **by value**; clamp
out-of-range dT to the nearest row with the marker set.

**D9 — The inheritance guard has two prongs.**
Prong A (example-level): re-run `plasma_blackout_corridor` after the atmosphere append; its gate
witnesses must equal the committed `output.txt` values. Prong B (harness-level): a
`deep_causality_cfd` test marches a corridor-class coupled world twice — plain stack vs. stack
with `PropulsionStub` composed at zero throttle — and asserts the reports, final fields, and
logs are bit-identical. Rationale: the full Acts-0/1 inheritance gate belongs to the
retropulsion example (M5, gate 1); M2 can and must prove the two things it changes globally
(atmosphere data, stub availability) are invisible. Prong B is what makes "strictly inert at
zero throttle" a tested contract instead of a comment.

## Risks / Trade-offs

- [Atmosphere append shifts the sampler's low clamp from 30 km to 0 km] → The corridor never
  samples below ~47 km at its terminal state, and prong A of the guard catches any surprise
  bit-for-bit. If prong A fails, the append is wrong (or the corridor secretly sampled the old
  clamp), and the failure is the finding.
- [Envelope extension touches the corridor's live gate] → `burn: None` default plus prong B; the
  gate's existing three-limit path is not re-written, only extended behind the option.
- [Stub semantics might quietly diverge from M3's production stages] → The stub is spec'd as
  contract-validation only; M3's spec inherits the same seam requirements, and the swap-changes-
  no-consumer scenario (mirroring the existing stub requirement in `blackout-coupling-interface`)
  is the regression net.
- [US-1976 transcription error] → Each row cites its pinpoint; a consistency test asserts
  a = √(γRT) within tolerance and monotone density/altitude ordering.
- [Weather CSV schema drift breaking the loader] → The consumption row type binds to
  `WorldRow::SCHEMA` column names through `read_rows`' by-name matching; a missing column is a
  named error by the `typed-table-io` contract, and a loader test reads the committed CSV.

## Migration Plan

Additive throughout; no API is removed or re-signed. Land order: coupling channels + RMW helper
→ envelope types → stub stage → atmosphere rows → loader → guard prongs. Each lands with its
tests green under `bazel test //deep_causality_cfd/...` before the next. Rollback is dropping
the change; nothing downstream depends on it until M3.

## Open Questions

- None blocking. The A0-vs-A depth behind the stub seam is deliberately open pending M1's
  verdict; this change is shaped identically either way (roadmap §2).
