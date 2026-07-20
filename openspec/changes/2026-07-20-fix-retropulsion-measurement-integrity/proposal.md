## Why

The plasma-retropulsion example flies a correct descent and then reports a counterfactual study that
does not measure the flight. An audit of `examples/avionics_examples/cfd/plasma_blackout/retropulsion/`
against its recorded run (`output.txt`, `retropulsion_branches.csv`) and its design note
(`openspec/notes/archive/cfd-plasma-retropulsion/plasma-retropulsion-descent.md`) found, and an
independent adversarial pass confirmed, twenty-two defects. Four are load-bearing:

1. **Three of five roster branches never flew their commanded throttle.** `CyberneticCorrect` clamps
   every command to the dynamic thrust-coefficient ceiling `max_ct·q∞·S_ref/T_ref`. At the fork's
   sensed `q∞ = 2234.804 Pa` that ceiling is `3 × 2234.804 × 4.6 / 70000 = 0.44057566423106215`,
   bit-matching the logged clamp. Roster entries 0.45, 0.55 and 0.80 therefore fly one trajectory.
   The recorded CSV confirms it: those three rows are bit-identical in `preserved_fraction`
   (`0.21719849383465803`), `final_density` (`1116693100279057600000000`) and `propellant_used`
   (`122.86207959045669`), yet are scored at 8.42 / 10.56 / 15.90 m·s⁻².

2. **`net_deceleration` is documented as "realized on the force channel" and never reads it.**
   `score_branch` computes `(case.throttle·RETRO_THRUST_N + preserved_fraction·BASE_AXIAL_DRAG_N)/mass`
   from the roster constant and a hardcoded drag. `BASE_AXIAL_DRAG_N = −18_000 N` is 2.44× below the
   drag the run actually flies (`q·CDA_OVER_M·m = 2234.8 × 5.8e-3 × 3397.8 = 44.0 kN`), and carries a
   sign opposite to thrust, so the coasting branch is reported as accelerating at −5.30 m·s⁻².

3. **The A0 drag closure and the safety envelope normalize the same `C_T` against different dynamic
   pressures, 5.37× apart.** `PlumeObstruction` takes `q∞ = BURN_CORRIDOR_Q_INF = 12_000 Pa` at
   construction; `CyberneticCorrect` takes the sensed `q∞ ≈ 2234.8 Pa`. In the same step the closure
   evaluates the correlation at `C_T = 0.559` while the gate judges the vehicle to be exactly at the
   `C_T = 3.0` stability limit. The closure also stays composed through the subsonic terminal leg,
   deleting ~78% of aerodynamic drag at Mach 0.01 with a correlation whose dataset spans Mach 0.4–2.0
   and whose mechanism is bow-shock displacement.

4. **Six gates cannot fail.** Gate (7) returns the literal `CAP` and asserts `(1..=CAP).contains(&CAP)`.
   Gate (4d) reads `Arc::ptr_eq(&clone, &original)` evaluated on the line after `Arc::clone`. Gate (4e)
   greps for a marker the carrier writes unconditionally, and its substring also matches the carrier's
   *"not applied"* refusal message. Gate (5) subtracts two interpolations of one CSV before any march
   and reports "the table changed the flight"; the margin it sizes is 139× the navigated uncertainty
   and never binds. Gate (1) is named "corridor inheritance" and counts leg-boundary log lines, which
   the four march calls fix at three. Gate (2) re-checks the four constants the commit predicate
   already enforced.

The audit also corrected two of its own first-pass claims, recorded here so the change does not
inherit them: gate (4a)'s spread is carried by the coast branch's genuinely measured density and does
not depend on the hardcoded fraction; and the force channel is dimensionally consistent as a specific
force throughout the coupling, so the mismatch is between the coupling and the example's
newton-valued scoring, not inside the coupling.

## What Changes

- **Score branches from the flown state.** Publish the realized throttle and the realized axial
  specific force as field scalars, and read both from each branch's report. Delete
  `BASE_AXIAL_DRAG_N`. The drag term becomes the drag the simulation carried.
- **Make the frozen-drag foil a trajectory quantity.** Accumulate two along-velocity Δv integrals per
  branch — the realized one, and one with the preserved-drag fraction held at its fork value — so
  gate (4c) compares trajectories rather than restating gate (4b) algebraically.
- **Re-pin the roster inside the envelope's admissible band**, and add a degeneracy gate that fails
  when two branches fly the same realized throttle within tolerance.
- **One dynamic pressure.** `PlumeObstruction` reads the sensed `"q_inf"` each step. Remove
  `BURN_CORRIDOR_Q_INF`.
- **Bound the SRP closure by its validity envelope.** `PlumeObstruction` goes inert and logs outside
  the Jarvinen–Adams Mach band, and the Cordell–Braun geometry is fed the sensed freestream Mach and
  static pressure so the kernel's own envelope check tests the flight.
- **Typed witnesses replace log grepping.** Add the commit witnesses, a leg re-seed counter, a regime
  transition counter, an applied-alternation flag, and a recorded peak bond to the library, then
  delete every `format!("{log}")` + `contains`/`split` recovery from the example.
- **Gates that can fail.** Rebuild gates (0), (1), (2), (4b), (4c), (4d), (4e), (5), (6) and (7)
  against measurements, re-earn every band from the corrected run, and record which bands bind.
- **Fly the belief counterfactual.** Run the uninformed world as a second march and gate on a flight
  outcome; size the ignition margin so it can bind.
- **Stop silent substitution.** A missing report series, a missing mass, or a non-finite scalar
  becomes an `Err` through the `reduce` closure's existing error channel. The peak fold no longer
  rectifies the negative preserved-drag branch the SRP kernel documents past `C_T ≈ 2`.
- **Reconcile or delete the duplicated constants.** `MEASURED_RHO_SCALE` is read from the table row;
  the printed IMU bias departure is either flown or not printed; `PLUME_S_REF_M2` and `CDA_OVER_M`
  are made one vehicle.
- **Stamp the table clamp into provenance** and gate it, as the design note and the `KeyedTable`
  docstring both require.
- **Implement or strike the two unimplemented weather-row jobs.** `main.rs` claims the interpolated
  row sizes the propellant reserve; nothing does.

## Capabilities

### New Capabilities

- `retropulsion-branch-scoring`: branch rows scored from the flown state — realized throttle, realized
  axial specific force, trajectory Δv, and the frozen-drag Δv foil.
- `srp-closure-validity`: one dynamic-pressure source for the thrust coefficient, and Mach-bounded
  applicability for the A0 correlation and the Cordell–Braun geometry.
- `retropulsion-gate-integrity`: every gate reads a measurement that can differ from its threshold,
  every gate message describes what the gate checks, and every band is re-earned.
- `retropulsion-typed-witnesses`: typed accessors for the commit, leg re-seeds, regime transitions,
  alternation application, and peak bond, replacing rendered-log recovery.
- `retropulsion-belief-counterfactual`: the uninformed world is flown, and the table's value is gated
  on a flight difference.
- `retropulsion-scoring-robustness`: absent, non-finite, and negative scalars fail loudly rather than
  defaulting.

### Modified Capabilities

- `plume-obstruction-stage` (`openspec/changes/archive/2026-07-19-add-retropulsion-coupled-stages/`):
  the construction-time `q_inf` parameter is replaced by sensed-field reads, and the stage gains a
  Mach applicability bound. **BREAKING** for `PlumeObstruction::new`.
- `throttle-guidance-stage`: `ThrottleGuidance` publishes the commit witnesses as field scalars in
  addition to the log entry.
- `flight-regime-classifier`: `RegimeClassify` publishes a monotone regime-transition counter.

## Impact

- **Modified (library, `deep_causality_cfd`):** `types/flow/retropulsion.rs` (`PlumeObstruction`
  signature and applicability), `types/flow/throttle_guidance.rs` (commit witness scalars),
  `types/flow/corridor/regime.rs` (transition counter), `types/flow/carrier.rs` (re-seed counter,
  peak bond, deterministic fork economics, applied-alternation flag), `types/flow/report.rs`
  (`peak_bond`, `alternation_applied`, per-branch step cost), `types/flow/coupled_march.rs`.
- **Modified (example):** the whole retropulsion example, `examples/avionics_examples/src/shared/`
  (`constants.rs`, `world.rs`, `stages.rs`), and both example READMEs where they state a measurement.
- **BREAKING:** `PlumeObstruction::new(thrust, q_inf, s_ref)` loses its `q_inf` parameter. The only
  caller in the workspace is `shared/world.rs`. `BASE_AXIAL_DRAG_N` and `BURN_CORRIDOR_Q_INF` are
  removed.
- **Blast radius:** the corridor and weather examples share `avionics_examples::shared`. Reconciling
  `CDA_OVER_M` with `PLUME_S_REF_M2` would change the corridor's flown trajectory and every band it
  has earned. That reconciliation is held behind an explicit decision in `design.md` and is not
  applied without it.
- **Regression artifacts:** `retropulsion_branches.csv` currently records `Arc::strong_count` sampled
  while five branches run concurrently under the `parallel` feature (recorded values 3, 2, 4, 5, 6),
  so the committed file is not reproducible. It becomes deterministic or the column is dropped.
- **Tests:** `deep_causality_cfd/tests/types/flow/retropulsion_tests.rs`,
  `throttle_guidance_tests.rs`, `corridor/regime_tests.rs`, `report_tests.rs`, `carrier_tests.rs`.
- **Expected outcome:** several gates will fail on first re-run. That is the point of the change; the
  bands are re-earned from the corrected run and the failures are recorded as findings rather than
  tuned away.

## Non-goals

- Replacing the A0 correlation with a decrement contracted from the marched field. The M1 de-risk
  verdict is AMBER on imprint fidelity and that stands.
- Off-axis SRP, thrust vectoring, or 6-DOF.
- Upgrading the Tier-A stopping-distance guidance to Klumpp or convex powered-descent guidance.
