# Tasks

Ordered so that each group leaves the workspace green. Library seams land before the example that
consumes them, and every group ends with a verification step that must pass before the next begins.

Defect IDs (R*, G*, P*, S*) refer to the validated register in `design.md` §1.

## 0. Decisions before code — RESOLVED 2026-07-20 (see `design.md` §3)

- [x] 0.1 Resolve **D1** (`CDA_OVER_M` versus `PLUME_S_REF_M2`, implied `C_d = 4.287`). Record the
      chosen option and its blast radius in `design.md`. No later task changes either constant without
      this answer.
- [x] 0.2 Resolve **D2** (roster values inside the admissible band `[0.15, 0.44]`, given the A0
      correlation saturates above `C_T ≈ 0.46`).
- [x] 0.3 Resolve **D3** (implement weather-row jobs 1 and 3, or strike the claim in `main.rs:14-15`
      and `README.md`).
- [x] 0.4 Resolve **D4** (accept that bands are re-earned from the corrected run rather than
      constraining the fix to preserve the current `output.txt`).

## 1. Library: typed witnesses (S1, S2, G1, G3, G4)

- [x] 1.1 `ThrottleGuidance` publishes `"ignition_commit_step"`, `"ignition_commit_mach"`,
      `"ignition_commit_q"`, `"ignition_commit_sigma"` at the latching step, alongside the existing
      log entry (`throttle_guidance.rs:322-332`)
- [x] 1.2 `CarrierPause::re_seeds()` — a counter incremented where `ReadyMarch::from` emits its
      re-seed entry (`coupled_march.rs:92-96`)
- [x] 1.3 `RegimeClassify` maintains a monotone `"regime_transitions"` field scalar from the `changed`
      decision it already computes and discards (`regime.rs:307`)
- [x] 1.4 `Report::alternation_applied() -> bool`, true only on the applied path, false on the
      carrier's refusal path (`carrier.rs:537-543`, `carrier.rs:596-598`)
- [x] 1.5 `Report::peak_bond()` — the measured rank of the final marched state, recorded at
      `finish_report`
- [x] 1.6 `ForkEconomics` samples `Arc::strong_count` once before the `scoped_map` fan-out so the value
      is reproducible under the `parallel` feature; add the post-fork bond growth that design note
      §10(4d) requires (`ForkEconomics::fork_peak_bond` + `Report::bond_growth`)
- [x] 1.6a **`is_o1()` narrowed.** Moving the sample before the fan-out made the pre-existing
      `fluid_refs > 1` conjunct fail, which is the audit's G2 finding realized: the count was only
      above one because it was read on the line after the branch's own `Arc::clone`. The conjunct is
      removed and `is_o1()` is now `shares_fluid && shares_field`, documented as a source-change guard
      rather than a run-time measurement. Two tests in `fork_tests.rs` asserted the old tautology and
      were rewritten to the corrected semantics — the API changed, so the tests follow it
- [ ] 1.6b **Deferred to group 5:** the trunk-relative step-cost ratio. It needs wall-clock timing,
      which does not belong in the solver crate; the example times its own trunk leg and fan-out
      instead. Tracked as task 5.6
- [x] 1.7 Tests in `deep_causality_cfd/tests/types/flow/`: each witness is published, each counter
      increments once per event, `alternation_applied` is false on the refusal path, `peak_bond`
      reports a rank below the cap for a low-rank state, and two fork runs record identical economics
- [x] 1.8 **Verify:** `cargo test -p deep_causality_cfd`

> Commit: `feat(deep_causality_cfd): typed witnesses for commit, re-seeds, transitions, bond, fork economics`

## 2. Library: SRP closure validity (P1, P2, P3)

- [x] 2.1 **BREAKING** — `PlumeObstruction::new(thrust, s_ref)` drops the `q_inf` parameter; the stage
      reads the sensed dynamic-pressure scalar each step with a configurable field name matching
      `CyberneticCorrect::with_burn_sensing`. An absent or non-positive value is an `Err`
      (`retropulsion.rs:371-404`)
- [x] 2.1a **`p_inf` needed a producer.** The carrier published `freestream_n` but no freestream
      temperature, so no stage could form the ambient static pressure. `CompressibleMarchRun` now
      publishes `"freestream_temperature"` from the schedule row it already holds, and `FlightSensors`
      derives `"p_inf" = n∞·k_B·T∞` beside the `q∞` it already produced
- [x] 2.1b **`PlumeNozzle` narrowed.** `p_inf` and `mach_inf` are removed from the record: they are
      sensed per step now, and leaving them as fields would let a caller re-freeze them. Only
      `gamma_inf`, a property of the ambient gas rather than of the flight, remains
- [x] 2.1c **`PRESERVED_DRAG_FRACTION_FIELD` exported**, and the stand-down path *takes* the scalar
      off the field rather than leaving a stale value that reads as a live measurement
- [x] 2.2 Add the Jarvinen–Adams Mach applicability bound: outside the band the stage applies no
      decrement, publishes no geometry, and logs once per crossing (`retropulsion.rs:394`)
- [x] 2.3 The Cordell–Braun call takes the sensed freestream Mach and static pressure instead of
      `PLUME_MACH_INF` and `PLUME_P_INF`, so the kernel's own envelope check tests the flight
      (`retropulsion.rs:423-437`)
- [x] 2.4 Tests: closure and gate produce the same `C_T` on one step; an absent `q_inf` errors; the
      stage stands down below the Mach floor and resumes above it; geometry differs between two
      altitudes at one throttle; the kernel's envelope rejection surfaces as a step error
- [x] 2.5 **Verify:** `cargo test -p deep_causality_cfd`, and confirm the only workspace caller of
      `PlumeObstruction::new` is `examples/avionics_examples/src/shared/world.rs`

> Commit: `fix(deep_causality_cfd)!: PlumeObstruction reads sensed q-infinity and bounds the A0 correlation by Mach`

## 3. Example: score from the flown state (R1, R2, R3, R7, R8, P5)

- [x] 3.1 Add a witness stage to `examples/avionics_examples/src/shared/stages.rs` publishing
      `"realized_throttle"`, `"axial_accel"` (positive for deceleration), `"dv_actual"` and
      `"dv_frozen"`; compose it after `RetroThrust` in both coupling builders (`world.rs:392`)
- [x] 3.2 `score_branch` reads `final_realized_throttle`, `final_axial_accel`, `final_dv_actual`,
      `final_dv_frozen` from the report; delete the `case.throttle · RETRO_THRUST_N` term
      (`model.rs:320`)
- [x] 3.3 Delete `BASE_AXIAL_DRAG_N` (`constants.rs:56`) and `BURN_CORRIDOR_Q_INF`
      (`shared/constants.rs:254`)
- [x] 3.4 Remove the hardcoded coast fraction and its false justifying comment (`model.rs:307-313`);
      every branch reads its published fraction
- [x] 3.5 `BranchRow` and its `SCHEMA` carry the realized throttle beside the commanded one, so the
      recorded table and the printed table both show what flew
- [x] 3.6 `utils_print::print_branches` prints both throttles and labels the deceleration column as
      read off the force channel
- [x] 3.7 **Verify:** run the example; confirm the coast branch reports a positive deceleration, that
      any two branches with equal realized throttle have equal scored rows, and that the recorded
      propellant is consistent with the recorded realized throttle

> Commit: `fix(examples): score retropulsion branches from the flown force channel`

## 4. Example: fail loudly (S3, S4, S5, S6)

- [x] 4.1 Split `final_scalar` into a sign-preserving single-cell read and a peak-over-cells read;
      neither defaults, both return `Err` on an absent series through `reduce`'s existing closure
      bound (`model.rs:287-296`)
- [x] 4.2 Delete the `mass.max(1.0)` floor; a missing or non-positive mass is an `Err` (`model.rs:316`)
- [x] 4.3 Reject non-finite witnesses at the read, so `main.rs:128`'s ordering cannot see a
      not-a-number
- [x] 4.4 Replace `v.rows()[0]` with a checked first-element read in all nine leg gates
      (`model.rs:565-675`)
- [x] 4.5 **Verify:** falsification tests — a branch report missing `preserved_drag_fraction` errors
      rather than passing gate (4b); a missing `mass` errors rather than inflating gate (4c); a
      negative preserved-drag fraction survives the read with its sign

> Commit: `fix(examples): retropulsion scoring fails on absent, non-finite, and negative witnesses`

## 5. Example: gates that can fail (G1, G4, G5, G6, G7, G8, G9, R1, S10, S11)

- [x] 5.1 Gate (0): carry each leg's error into `LegSet` instead of returning early on three of four;
      name the failing leg (`main.rs:83,108,162,196`)
- [x] 5.2 Gate (1): **reformulated — see `design.md` §5 N5.** Comparing against the *corridor's*
      recorded window is incompatible with §4: this example flies the measured cold day, which
      ionizes earlier and dwells longer by construction. The gate compares the flown window against
      what the dispersion table predicts **for this temperature departure** (onset error 0.04 s,
      dwell error 0.19 s), which also puts the table's window columns under test
- [x] 5.3 Gate (2): check the aided navigation mode and the sigma against the margin; remove the
      re-check of the four constants the commit predicate guarantees; correct the message
      (`model.rs:589`)
- [x] 5.4 Gate (4b): asserts **both** the monotone preserved-drag collapse and the **sign flip** —
      net deceleration is non-monotone in realized throttle, minimum at the 0.24 branch (5.031 m/s2
      against 10.644 coasting). The design note's stated (4b) is met rather than substituted
- [x] 5.5 Gate (4c): compare `dv_actual` against `dv_frozen` (`model.rs:522`)
- [x] 5.6 Gate (4d): keeps `is_o1` as a source-change guard and reads `Report::bond_growth()`,
      banded at `MAX_BOND_GROWTH` (measured 0)
- [ ] 5.6a **Still open:** the trunk-relative step-cost ratio. Bond growth and sharing are gated;
      the wall-clock ratio design note §10(4d) also asks for is not
- [x] 5.7 Gate (4e): read `Report::alternation_applied()` (`model.rs:538`)
- [x] 5.8 Gate (4f) **new**: roster non-degeneracy on realized throttle
- [x] 5.9 Gate (6): bound the descent rate on both sides of `CONTACT_SPEED_MS` (`model.rs:637`)
- [x] 5.10 Gate (7): read `Report::peak_bond()`; delete `committed_bond` (`model.rs:711`)
- [x] 5.11 Gates (8) and (9): relabel the messages as runaway detectors; thresholds unchanged
- [x] 5.12 Delete every rendered-log recovery from the example — `commit_witness`, `leg_witnesses`,
      the `"regime ->"` count, the marker search — replacing each with its typed accessor from group 1
- [x] 5.13 **Verify:** every gate has a test supplying an input on which it fails

> Commit: `fix(examples): rebuild retropulsion gates on typed measurements that can fail`

## 6. Example: the belief counterfactual (G5, S8, S9, R9, S7)

- [ ] 6.1 March the uninformed world with `uninformed.margin_m` from the same baseline, marked as a
      context alternation
- [ ] 6.2 Gate (5) compares a flown outcome between the two worlds (`model.rs:625`)
- [ ] 6.3 Size the margin so it can bind against the navigated sigma the flight achieves; if the two
      beliefs cannot separate at any honest sizing, record that finding in the gate message
- [ ] 6.4 Stamp `DayBelief::clamped` into the `EffectLog` and gate it (`model.rs:144`)
- [ ] 6.5 Read `MEASURED_RHO_SCALE` from the interpolated row; delete the hand-set constant
      (`constants.rs:18`)
- [ ] 6.6 Pass `informed.bias_departure` to both coupling builders, or remove it from the printed
      output (`main.rs:77,96,151,176`)
- [ ] 6.7 Apply the **D3** decision on weather-row jobs 1 and 3
- [ ] 6.8 **Verify:** run with a measured departure outside `[-40, +20]` and confirm the clamp appears
      in provenance and trips its gate

> Commit: `fix(examples): fly the retropulsion belief counterfactual and stamp the table clamp`

## 7. Constants coherence and prose (P4, S11, S8)

- [ ] 7.1 Apply the **D1** decision on `CDA_OVER_M` / `PLUME_S_REF_M2`
- [x] 7.2 Re-earn every band from the corrected run; each docstring records the measured value and
      states whether the band binds; remove the superseded figure in `FROZEN_DRAG_SEPARATION_MIN`
      (`constants.rs:69`)
- [ ] 7.3 Correct `README.md` and the `main.rs` module doc where they state a measurement the code no
      longer makes — in particular the claim that branches "spread with the intervention"
      (`README.md:113`, `main.rs:27`)
- [x] 7.4 Regenerate `output.txt` and `retropulsion_branches.csv`
- [x] 7.5 **Verify:** run twice and diff both artifacts; they must be identical

> Commit: `docs(examples): re-earn retropulsion bands and correct the claims the run no longer supports`

## 8. Close-out

- [x] 8.1 `make format && make fix`
- [ ] 8.2 `bazel test //...`
- [x] 8.3 Re-run the corridor and weather examples; confirm their recorded outputs are unchanged, or
      re-earn their bands if **D1** option (b) was chosen
- [ ] 8.4 Record in `design.md` which gates failed on the first corrected run and what each failure
      revealed. A gate that failed is a finding, not a threshold to loosen
- [ ] 8.5 Re-run the audit's confirmation arithmetic against the new `output.txt`: the clamp identity,
      the coast propellant identity, and the branch bit-identity check must all now resolve differently

> Commit: `chore: close out retropulsion measurement-integrity change`
