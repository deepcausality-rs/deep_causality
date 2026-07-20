# Design — retropulsion measurement integrity

## 1. Validated defect register

Every row was found by a dimension-specific audit and then attacked by an independent verifier whose
default was refutation. `Verdict` records the verifier's answer, not the finder's claim. Arithmetic is
reproduced from the recorded run (`output.txt`, `retropulsion_branches.csv`) unless noted.

### 1.1 Fabricated and non-computed results

| # | Defect | Site | Verdict |
|---|---|---|---|
| R1 | `score_branch` forms thrust from the roster constant `case.throttle · RETRO_THRUST_N`, not the flown throttle. The gate clamps to `3·2234.804·4.6/70000 = 0.44057566423106215` (bit-matching the logged clamp), so 0.45 / 0.55 / 0.80 fly one trajectory and are scored 8.42 / 10.56 / 15.90 m·s⁻². | `model.rs:320` | CONFIRMED |
| R2 | `net_deceleration` is documented "realized on the force channel"; the function never reads `aero_force`. | `model.rs:239,320` | CONFIRMED |
| R3 | `BASE_AXIAL_DRAG_N = −18_000 N` is invented. Channel drag at the fork is `5.8e-3 × 2234.8 × 3397.8 = 44.0 kN`; ratio 2.443×. | `constants.rs:56` | CONFIRMED |
| R4 | Coast `preserved_fraction` is the literal `1.0`. Without it the coast branch reads the fraction its one firing step published — `C_T = 0.4396976·70000/(12000·4.6) = 0.5576`, on the same saturated plateau `0.21719849383465803` — so the collapse becomes ≈ 0.000 and gate (4b) fails. The comment justifying the literal ("the plume stage is inert there and never publishes") is false; the coast branch fired. | `model.rs:307-313` | CONFIRMED (arithmetic corrected: the first-pass figure 0.462 − 0.217 = 0.245 was wrong; 0.4624 belongs to the θ = 0.25 branch) |
| R5 | Gate (4a)'s spread does **not** depend on the literal. `final_density` is untouched by it; removing the literal leaves spread at 0.011673 ≥ 0.01. The 0.00097 figure belongs to a different counterfactual — dropping the coast **row**. | `model.rs:470` | **REFUTED as first stated.** Restated: gate (4a) is carried entirely by the coast row, and the three saturated branches contribute nothing. |
| R6 | The coast branch burns 1.11 kg at throttle 0. `RetroThrust` reads the throttle channel, which at fork time holds the trunk's last clamp `0.4396975665524148`; `ThrottleGuidance` overwrites it later in the same step. `0.4396976·70000/(282·9.80665)·0.1 = 1.1129671` vs recorded `1.11296707870315`. | `retropulsion.rs:266`, `world.rs:384-407` | CONFIRMED |
| R7 | Gate (4c)'s difference cancels the thrust term identically: `net − frozen = (pf − ff)·D/m`. Its maximum, 4.2326730730280743, is the coast row alone. | `model.rs:522` | PARTIALLY CORRECT. For the four burning rows `pf` and `m` are run outputs, so the term is not fabricated end to end — but it is scaled by the invented `D`, and the gate still passes without the coast row (1.416 ≥ 0.5). |
| R8 | Sign inconsistency. `thrust` is positive and `BASE_AXIAL_DRAG_N` negative, though both act along −v̂ in the coupling. The coasting branch — where aerodynamic deceleration is largest — reports −5.2976. | `model.rs:320-322` | CONFIRMED. Selection order is unaffected: with consistent signs the ranking is still monotone and `nominal` still wins. |
| R9 | `MEASURED_RHO_SCALE = 1.15` is hand-set while the table interpolates to `1.2 − 0.53333·0.1 = 1.1466667` at dT = −32. `{:.2}` renders the interpolated value as "1.15", hiding the 0.29% divergence. | `constants.rs:18`, `utils_print.rs:48` | CONFIRMED |

### 1.2 Gates that cannot fail

| # | Defect | Site | Verdict |
|---|---|---|---|
| G1 | Gate (7): `committed_bond` returns the literal `CAP`; the gate asserts `(1..=CAP).contains(&CAP)`. Nothing re-quantizes. | `model.rs:711,654` | CONFIRMED. Refinement: it is a density-presence check, since `final_density` of 0.0 yields `usize::MAX`. |
| G2 | Gate (4d): `ForkEconomics` is built from `Arc::ptr_eq(&clone, &original)` on the line after `Arc::clone`, with `strong_count` read after that clone. All four `is_o1` conjuncts hold unconditionally. | `carrier.rs:547-554`, `report.rs:73` | CONFIRMED as unfalsifiable at run time. It retains value as a source-change guard: an edit that materializes instead of sharing would flip `ptr_eq`. |
| G3 | `fluid_refs` is `Arc::strong_count` sampled while five branches run concurrently under `scoped_map` with the `parallel` feature on. Recorded 3, 2, 4, 5, 6. The committed CSV is not reproducible. | `carrier.rs:552` | CONFIRMED |
| G4 | Gate (4e): the carrier writes `!!ContextAlternation!!` into a fresh log unconditionally before marching. No row can reach the gate without it. The same substring also matches the carrier's refusal message `"!!ContextAlternation!!: not applied (errored run cannot be repaired)"`, so the check cannot distinguish an applied alternation from a refused one. | `carrier.rs:537-543,596-598`, `model.rs:538` | CONFIRMED, with the refusal-match defect added |
| G5 | Gate (5) subtracts two interpolations of one CSV at dT = −32 and dT = 0, computed before any march. `uninformed` reaches no coupling stack. The commit logged `sigma 0.38159182534497615 m` against margins of 73.41 m and 52.98 m — 139× and 194× slack — so the margin binds under neither belief and both worlds fly identically. | `main.rs:66-71`, `model.rs:625` | CONFIRMED |
| G6 | Gate (1) asserts `re_seeds >= 1`, grepped from a rendered log. `ReadyMarch::from` emits that line unconditionally, and the example makes four march calls, so the count is structurally 3. No window, anchor band, drift, or reacquisition witness is read. | `model.rs:576`, `coupled_march.rs:92-96` | CONFIRMED |
| G7 | Gate (2) re-checks `IGNITION_MACH_MIN/MAX` and `IGNITION_Q_MIN/MAX` — the same four constants `IgnitionCorridor` uses as its precondition — so `in_band && in_window` is entailed by the commit line existing. The gate reduces to `commit_step > 0`. Its message additionally claims the post-fix nav state and the margin were checked; neither is. | `model.rs:589`, `world.rs:297-306`, `throttle_guidance.rs:278-303` | CONFIRMED |
| G8 | Gate (0): `onset.error()` and `burn.error()` are unreachable-false because `main.rs` returns `Err` on both first; `burn_out` is absent from the disjunction. The gate is live for one leg of four. `error_text` reads only the terminal leg. | `main.rs:83,108,162,196,215` | CONFIRMED |
| G9 | Gate (4b) substitutes a monotone preserved-drag collapse for the design note's "sign-flip found: deceleration vs throttle non-monotone". The substituted property follows from the digitized table being monotone in `C_T` and `C_T` being linear in throttle. Three of four windows are satisfied by equality, since the correlation saturates at `0.21719849383465803` across 0.45–0.80. | `model.rs:491`, note §3.2/§10 | CONFIRMED. The substitution is disclosed in-comment; the disclosure does not make the substituted gate informative. |

### 1.3 Physics

| # | Defect | Site | Verdict |
|---|---|---|---|
| P1 | Two dynamic pressures normalize one `C_T` in one step. Closure: `C_T = 0.440578·70000/(12000·4.6) = 0.5587`. Gate: `C_T = 0.440578·70000/(2234.8·4.6) = 3.0000`. Ratio 5.370. Back-solving the recorded `0.21719849383465803` through the J–A table gives `C_T = 0.4746`, confirming the 12 000 Pa branch is what flew. The sensed-q branch would give `C_T = 3.0 → fraction ≈ −0.0606`, a drag reversal. | `retropulsion.rs:371-404`, `gate.rs:274-283` | CONFIRMED |
| P2 | The Cordell–Braun kernel enforces a Mach envelope [2, 4] and is handed the frozen `PLUME_MACH_INF = 2.2`, so the check tests a constant. Flown Mach is 2.00 → 0.60 on the burn leg and 0.01 at touchdown. `PLUME_P_INF = 1500 Pa` is true at ≈ 28–29 km only: ≈ 798 Pa at the 32.71 km ignition (1.88× high), ≈ 6630 Pa at the 18.95 km handover (4.4× low), ≈ 101 600 Pa at sea level (68× low). The `P_exit/P∞ ≥ 7` transition gate never fires under either value, so the frozen number shifts geometry silently. | `world.rs:494-509`, `plume.rs:350-364,389-399` | CONFIRMED |
| P3 | The A0 decrement stays composed through the terminal subsonic leg. `powered_descent_coupling_with`'s `terminal` flag switches only the envelope and the guidance; `PlumeObstruction::apply` gates on throttle alone. At Mach 0.01 and throttle 0.36, `C_T = 0.4565 → fraction ≈ 0.2211`: ~78% of aerodynamic drag deleted by a bow-shock-displacement correlation, below its dataset's Mach floor. `shared/constants.rs:358-371` argues this exact point for the `C_T` cap; the cap was released and the correlation was not. | `world.rs:317-391`, `retropulsion.rs:394` | CONFIRMED |
| P4 | `CDA_OVER_M · VEHICLE_MASS_KG = 5.8e-3 × 3400 = 19.72 m²`, so `PLUME_S_REF_M2 = 4.6` implies `C_d = 4.287` against a capsule class of 1.0–1.7. The two constants feed one expression chain — `S_ref` sets `C_T` and the fraction, `CDA_OVER_M` sets the drag that fraction multiplies — and describe different vehicles. | `shared/constants.rs:111,214,230` | CONFIRMED |
| P5 | The force channel is a specific force (m·s⁻²) throughout the coupling and is consumed as one by `ks_strang_step` and `g_load`. | `branch.rs:156`, `retropulsion.rs:291`, `stages.rs:74,124` | **REFUTED as a dimensional bug.** Restated: the name `aero_force` misleads, and `score_branch` works in newtons against a channel in m·s⁻², which is the real mismatch (see R3). |

### 1.4 Silent failure, leaks, conformance

| # | Defect | Site | Verdict |
|---|---|---|---|
| S1 | Six witnesses are recovered by rendering an `EffectLog` and splitting strings, including `{:?}`-formatted floats. | `model.rs:380-419,609,327` | CONFIRMED. Failure mode corrected: a format change yields zeros, which makes the gates **fail with a misattributed message**, not pass silently. |
| S2 | Four of those six have no typed accessor anywhere in the crate — commit step / Mach / q, leg re-seed count, regime-transition count, applied-alternation flag. They are library gaps. The commit-*fired* boolean does have one: `IGNITION_LATCH_FIELD = "ignition_committed"`, which the example ignores. | `carrier.rs:429-455`, `coupling.rs:275`, `throttle_guidance.rs:65` | CONFIRMED |
| S3 | `final_scalar` returns 0.0 for a missing series and `mass` is floored at 1.0 kg. Missing `preserved_drag_fraction` → 0.0 → gate (4b) **passes** (collapse 1.0). Missing `mass` → 1.0 kg → net deceleration ~10³ m·s⁻² and gate (4c) **passes harder**. Missing `n_tot` is caught. | `model.rs:287-296,316` | CONFIRMED, with the per-field consequences corrected |
| S4 | The peak fold seeds at 0.0, so a negative single-cell scalar reads 0.0. This is not hypothetical: `srp.rs:141-142` documents *"slightly negative values past C_T ≈ 2 are the measured wake-type forebody force"*, and that is exactly the sign-flip physics gate (4b) was meant to find. | `model.rs:287` | CONFIRMED |
| S5 | `score_branch` has no `Err` path. | `model.rs:299` | PARTIALLY CORRECT. The `Result` is imposed by `StudyEffect::reduce`'s closure bound and is not removable. The defect is that no detectable failure is routed into the channel that already exists. |
| S6 | Panic sites: `main.rs:128` `max_by(...partial_cmp().unwrap())` is **reachable** (`net_deceleration` derives from an unchecked report series). `model.rs:503` `sort_by` is unreachable (throttles are literals). `v.rows()[0]` in all nine leg gates is unreachable here but unsound for reuse, since `StudyView::of` accepts an empty slice. `day_belief`'s `v[8]`/`v[9]` are guarded upstream by `from_cells`, not locally. | `main.rs:128`, `model.rs:503,565-675,135` | PARTIALLY CORRECT, per-site |
| S7 | The printed "IMU bias departure 1.32" is the interpolated table value; every coupling is constructed with the literal `1.0`, and the parameter is load-bearing (`IMU_ACCEL_BIAS[i] * imu_bias_departure`). | `utils_print.rs:48`, `main.rs:77,96,151,176` | CONFIRMED |
| S8 | Design note §4 gives the row three jobs. Only job 2 exists. `PROPELLANT_FLOOR_KG` is a fixed 40.0 read only by gate (6), never derived from the dispersion, and there is no targeting computation. `main.rs:14-15` states the row "sizes the ignition margin and the propellant reserve"; the second clause is false. | `main.rs:14`, `shared/constants.rs:256` | CONFIRMED |
| S9 | `DayBelief::clamped` is documented — and required by note §4 and by the `KeyedTable` docstring — to be stamped into provenance. Its only reader is a `println!`. No gate reads it; `LegSet` has no field for it. Latent on this run, since dT = −32 is interior to [−40, +20]. | `model.rs:101,144`, `keyed_table.rs:17` | CONFIRMED |
| S10 | Band headroom: `FROZEN_DRAG_SEPARATION_MIN` 0.5 vs 4.2327 (8.5×, and its docstring cites a superseded 1.610); `BELIEF_SEPARATION_MIN_M` 1.0 vs 20.43 (20×); `TOUCHDOWN_SINK_MAX` 5.0 vs 2.0 (2.5×). | `constants.rs:69,75,101` | CONFIRMED with two refinements: `WALL_CLOCK_BUDGET_S` (7.3×) is a runaway detector and appropriate as written; `TOUCHDOWN_SINK_MAX`'s defect is that a one-sided `≤ 5.0` bound cannot fail an undershoot or a hover, which is what its docstring claims it now measures. |
| S11 | `README.md:113` and `main.rs:27` state the fork's "branches spread with the intervention"; three of five branches are bit-identical to 17 significant figures. | `README.md:113` | CONFIRMED |

## 2. Derived fixes

### F1 — Score from the flown state (R1, R2, R3, R7, R8, P5)

Add an example-local witness stage composed after `RetroThrust`, matching the `shared/stages.rs`
precedent that example wiring lives beside the library physics rather than inside it. Each step it
publishes:

- `"realized_throttle"` — `field.throttle_action()`, the value the gate clamped on the previous step
  and the propulsion stages actually flew.
- `"axial_accel"` — `−(a · v̂)` from the summed force channel, positive for deceleration. This is the
  quantity `net_deceleration` claims to be.
- `"dv_actual"` — running `Σ axial_accel · dt`, a trajectory integral.
- `"dv_frozen"` — running `Σ (a_thrust + f_fork · a_drag) · dt` with the preserved-drag fraction held
  at the value passed in at fork time. This is the design note's frozen-drag foil as a trajectory
  quantity rather than an algebraic restatement.

`score_branch` then reads `final_realized_throttle`, `final_axial_accel`, `final_dv_actual`,
`final_dv_frozen`. `BASE_AXIAL_DRAG_N` is deleted. The sign convention becomes one convention:
positive is deceleration along −v̂, so a coasting branch reports a positive number.

`dv_actual` and `dv_frozen` are monotone accumulators, so the existing peak fold reads them correctly;
`realized_throttle` and `axial_accel` are single-cell and must be read at cell 0 (see F6).

### F2 — Roster inside the admissible band, plus a degeneracy gate (R1)

The envelope's ceiling at the fork is 0.4406 and decays with `q∞` to 0.3742 over the continuation.
Re-pin the roster to values the envelope admits, spanning the band the A0 correlation resolves rather
than the band it saturates over: the audit shows the correlation is flat at `0.21719849383465803`
above `C_T ≈ 0.46`, so distinct branches must sit below it.

Add gate **(4f) roster non-degeneracy**: fail when any two branches' `final_realized_throttle` agree
within a pinned tolerance. This is the structural guard — it converts "the roster silently collapsed"
from an invisible condition into a run failure, and it would have caught the present defect.

### F3 — One dynamic pressure (P1)

`PlumeObstruction::new` loses its `q_inf` parameter and reads the sensed `"q_inf"` scalar each step,
with a configurable field name matching `CyberneticCorrect::with_burn_sensing`. `BURN_CORRIDOR_Q_INF`
is deleted. An absent or non-positive `"q_inf"` is an `Err`, not a fallback: a silent fallback is how
the present defect survives.

Consequence to expect and record: at the flown condition the closure moves from `C_T = 0.559` to
`C_T = 3.0`, and the A0 table returns a **negative** fraction there. That is the physics the design
note's §3.2 sign-flip gate was written for, and F6 must stop rectifying it.

### F4 — Bound the closure by its validity envelope (P2, P3)

`PlumeObstruction` gains an applicability bound on the sensed `"flight_mach"`. Outside the
Jarvinen–Adams band it applies no decrement, publishes no geometry, and logs once per crossing that it
stood down. The Cordell–Braun call is fed the sensed freestream Mach and static pressure instead of
`PLUME_MACH_INF` and `PLUME_P_INF`, so the kernel's own envelope check tests the flight and refuses
when the flight leaves it. Both frozen constants are deleted.

### F5 — Typed witnesses (S1, S2, G1, G2, G3, G4, G6, G7)

Library additions, each replacing a string grep:

| Witness | Addition |
|---|---|
| commit step / Mach / q / sigma | `ThrottleGuidance` publishes `"ignition_commit_step"`, `"ignition_commit_mach"`, `"ignition_commit_q"`, `"ignition_commit_sigma"` alongside the existing log entry |
| leg re-seed count | `CarrierPause::re_seeds()`, incremented where `ReadyMarch::from` emits its line |
| regime transitions | `RegimeClassify` maintains a monotone `"regime_transitions"` counter; `RegimeClass` already computes `changed` and discards it |
| alternation applied | `Report::alternation_applied() -> bool`, set `true` only on the applied path, `false` on the carrier's refusal path |
| peak bond | `Report::peak_bond()`, recorded by the carrier at `finish_report` from the final state's actual rank |
| per-branch step cost | `ForkEconomics` gains the trunk-relative step-cost ratio and post-fork bond growth that §10(4d) requires |

`ForkEconomics` samples `Arc::strong_count` once before the fan-out rather than inside each concurrent
branch, so the recorded CSV becomes reproducible. If a deterministic sample is not achievable at that
seam, the column is dropped from the recorded artifact and kept only in the gate message.

### F6 — Fail loudly (S3, S4, S5, S6)

- Split `final_scalar` into `final_cell0` (single-cell scalars, sign-preserving) and `final_peak`
  (field quantities). Neither defaults: a missing series returns `Err` through `reduce`'s existing
  closure bound.
- Delete the `mass.max(1.0)` floor; a missing or non-positive mass is an `Err`.
- Reject non-finite scalars at the point of read, so `main.rs:128`'s `max_by` cannot see a NaN.
- Replace `v.rows()[0]` with a checked `first()` in all nine leg gates.

### F7 — Gates that can fail (G1, G4, G5, G6, G7, G8, G9, S10, S11)

| Gate | Change |
|---|---|
| (0) integrity | Capture each leg's error into `LegSet` rather than early-returning, so all four legs are covered and the failing leg is named |
| (1) inheritance | Compare Act-0/1 witnesses against the corridor example's recorded values — onset step, exit step, dwell, drift, reacquisition — and assert equality |
| (2) ignition | Check the two conditions the corridor predicate does **not** enforce (aided nav mode, sigma against the margin), and drop the re-check of the four constants the predicate already guarantees. Correct the message to state what is checked |
| (4b) | With F3 and F6 the preserved fraction reaches its negative branch. Assert the sign flip from `dv_actual` versus throttle if it is present, and record the band location against the correlation. If the corrected run shows no flip, record that as a measured finding and say so in the gate message |
| (4c) | Compare `dv_actual` against `dv_frozen` — two trajectory integrals |
| (4d) | Keep `is_o1` as a source-change guard and add the step-cost ratio and bond-growth bands §10(4d) requires |
| (4e) | Read `Report::alternation_applied()`. Reject the refusal path explicitly |
| (4f) | New: roster non-degeneracy on realized throttle |
| (5) | Fly the uninformed world (F8) |
| (6) | Two-sided bound around `CONTACT_SPEED_MS`, so an undershoot or a hover fails |
| (7) | Read `Report::peak_bond()` |
| (8), (9) | Unchanged in threshold; messages relabelled as runaway detectors rather than regression gates |

Every band is re-earned from the corrected run and each docstring records the measured value it was
pinned from and whether it binds.

### F8 — Fly the belief counterfactual (G5, S8, S9)

Run the uninformed world as a second march with `uninformed.margin_m` and gate on a flight difference —
commit step, propellant at touchdown, or miss to the aim point. The present margin is 139× the
navigated sigma and cannot bind, so the margin's `k` or the navigated uncertainty must be sized so the
two beliefs can separate. If they cannot separate at any honest sizing, the finding is that the table
does not change this flight, and gate (5) records that instead of asserting the opposite.

Stamp `DayBelief::clamped` into the `EffectLog` and gate it, as the note and the `KeyedTable`
docstring require.

### F9 — Constants coherence (R9, P4, S7, S8)

- `MEASURED_RHO_SCALE` is read from the interpolated row, deleting the second source of truth.
- The printed IMU bias departure is passed to the coupling, or the line is removed. Passing it is the
  honest option and matches what the weather example's INS thermal model is for.
- `main.rs`'s doc claim about the propellant reserve is either implemented or struck.

## 3. Decisions (resolved 2026-07-20)

**D1 — `CDA_OVER_M` versus `PLUME_S_REF_M2` (P4). RESOLVED: derive `S_ref` from `CDA_OVER_M`.**
`CDA_OVER_M = 5.8e-3` is the corridor's flown, gated ballistic bundle and stays authoritative.
`PLUME_S_REF_M2` becomes `CDA_OVER_M · VEHICLE_MASS_KG / C_d` with a cited capsule `C_d`, so the
reference area the thrust coefficient normalizes against and the drag that coefficient's fraction
scales describe one vehicle. Blast radius is the retropulsion example only; the corridor and weather
examples are untouched. The chosen `C_d` and its citation are pinned in `shared/constants.rs`.

**D2 — Roster values (F2). RESOLVED: resolve the correlation's live range.** The roster is re-pinned
low in the admissible band `[0.15, 0.44]` so every branch flies a distinct realized throttle and a
distinct preserved-drag fraction. This is a weaker burn than the example currently advertises, and the
README says so. Gate (4f) enforces the property structurally, so a future envelope change that
re-collapses the roster fails the run instead of silently reproducing the present defect.

**D3 — Weather-row jobs 1 and 3 (S8). RESOLVED: implement job 3, strike job 1.** The propellant
reserve is sized from the interpolated dispersion (`drift_mean + k · drift_sd`) rather than the fixed
`PROPELLANT_FLOOR_KG = 40.0`, which makes the table load-bearing in a second place. The deorbit/entry
targeting shift is removed from the `main.rs` and README claims; the design note named it a stretch
goal and nothing implements it.

**D4 — Expected gate failures. RESOLVED: re-earn from the corrected run.** F3's move onto the sensed
`q∞` is expected to push the preserved fraction onto its negative branch at the flown `C_T`, changing
the trajectory, the touchdown state, and several bands. The corrected run becomes the reference, every
band is re-pinned from it, and each first-run gate failure is recorded in §5 as a finding. No fix is
shaped to reproduce the present `output.txt`.

## 5. First-run findings

A gate that failed on the corrected run is a measurement, not a threshold to loosen.

### N1 — The two SRP models' validity envelopes are nearly disjoint **[new]**

Feeding the Cordell–Braun kernel the sensed freestream (F4) made it refuse on the first burn step:

```
Physical Invariant Broken: Freestream Mach outside the Cordell model's validated envelope [2, 4]
```

The two models this example composes are validated over different flight regimes:

| Model | Role | Validated range |
|---|---|---|
| Jarvinen–Adams | preserved-drag correlation, the in-flight drag authority | Mach 0.4 – 2.0 |
| Cordell–Braun | analytic plume boundary, drives the marched-layer imprint | Mach 2 – 4 |

They meet at a single point. The ignition corridor commits at Mach 2.0 and the burn descends from
there, so the geometry model is outside its envelope for essentially the whole burn. This was
invisible before: the kernel was handed a frozen `PLUME_MACH_INF = 2.2` while the vehicle flew the
Jarvinen–Adams band, so its own envelope check tested the constant and always passed.

Resolution: both bands are declared explicitly at the call site
(`with_mach_band` / `with_geometry_mach_band`), each stands down where it does not apply and records
the crossing, and a kernel refusal still propagates *inside* a declared band. The consequence for the
example is real and is not papered over — the marched-layer imprint the geometry drives can only be
live in the instant around Mach 2, so the flow-spread witness measures throttle→trajectory→density
rather than a plume imprint. **[measured]**

### N2 — A shut-down engine carried a live drag decrement **[new]**

`PlumeObstruction`'s inert path returned before touching the field, so a branch that stopped burning
kept the trunk's last preserved-drag fraction on the field and republished it as its own. The coast
branch's first corrected run reported a fraction of −0.0606 at zero throttle. The inert path now
*takes* the scalar off, and `BranchRow::preserved_fraction` is an `Option` — absent means the closure
applied nothing, which is a state rather than a missing measurement. Gate (4b) compares only branches
that applied a decrement. **[fixed]**

### N3 — The sign flip is present, and the old normalization was hiding it

With the closure on the sensed `q∞` and the read sign-preserving, the correlation reaches its
**negative** branch — the wake-type forebody force past `C_T ≈ 2` that §3.2 names as the effect the
study exists to find. The first corrected roster measured fractions of 0.131, −0.001, −0.029, −0.047
across realized throttles 0.16 → 0.38. Two defects were jointly concealing it: the 12 kPa
normalization kept `C_T` in the correlation's shallow range, and `final_scalar`'s peak-from-zero fold
rectified any negative to 0. **[measured]**

### N5 — Gate (1) as the design note specifies it is incompatible with §4 **[new]**

Note §10 requires Acts 0-1 to reproduce the corridor's blackout window **bit-identically**. Built that
way, the gate failed:

```
blackout onset step 105 vs corridor 119; dwell 59.60 vs 58.40 s; drift 47.19 vs 2.54 m
```

The divergence is the example working. §4's whole premise is that this descent flies the **measured
cold day**, and a colder, denser atmosphere ionizes earlier and dwells longer — which is exactly what
the dispersion table records. The note's §2 argument for bit-identity ("the extension only appends")
covers the *atmosphere table extension below 30 km*; it does not cover the *weather dispersion*, which
changes every altitude the corridor samples. Two requirements of the same note are in conflict, and
the bit-identity one is the one that has to give.

Reformulated, and stronger for it: the flown window is compared against the `onset` and `dwell` the
dispersion table records **for this temperature departure**. The table was generated by the sibling
weather example from the same coupling stack, so agreement says the stack still behaves as it did —
and it puts the table's window columns under test rather than only its drift row.

| | table @ dT = −32 | flown | error |
|---|---|---|---|
| onset | 10.54 s | 10.50 s | 0.04 s |
| dwell | 59.41 s | 59.60 s | 0.19 s |

Both inside one compressed flight step of the quantity predicted. **[measured]**

### N6 — The trajectory sign flip is present and is now asserted **[new]**

With deceleration read off the force channel rather than re-derived, net deceleration is
**non-monotone** in realized throttle:

| branch | flown | axial m·s⁻² |
|---|---|---|
| coast | 0.00 | 10.64 |
| floor | 0.16 | 5.29 |
| low | 0.24 | **5.03** |
| mid | 0.31 | 6.16 |
| high | 0.38 | 7.48 |

Lighting the engine in the low-`C_T` band buys *less* net deceleration than coasting — the
counter-intuitive result §3.2 names as the analog of the corridor's clamped 40° branch. Gate (4b) now
asserts the dip's presence and reports its location, so the note's stated (4b) is met rather than
substituted. **[measured]**

### N7 — A false performance alarm, and a real artifact defect **[new]**

Verifying the sibling examples reported the weather example failing its wall-clock gate at 739.6 s
against a 600 s budget, and a follow-up "clean" run at 605.1 s against a 312.5 s recorded figure. Both
were **contaminated measurements**: the second run began while the first was still executing, so two
copies of the same all-core example competed for cores. Measured properly the example runs in
**193.5 s and passes** — faster than its recorded figure. There is no regression. The methodology was
the defect, and the lesson is that a wall-clock gate is only as good as the isolation of the run that
feeds it.

Separately, and genuinely: the weather example **appends** to the tracked
`weather/audit/weather.audit.main.log` on every run, 294 lines per run, with absolute machine paths
embedded (`/Users/<name>/...`). A tracked artifact that grows without bound on every run cannot be
diffed as a regression record, and the absolute paths make it machine-specific. Out of scope for this
change; recorded so it is not lost. **[open]**

### N4 — `DRAG_COLLAPSE_MIN` must be re-earned, downward

The band was 0.5, earned when fractions ran 1.000 → 0.217 under the wrong normalization. Corrected,
the collapse across burning branches is ≈ 0.178. The band is re-pinned from the corrected run per
decision **D4**; the earlier figure is superseded by a correction, not relaxed. **[measured]**

## 6. Corrections carried from the audit

Recorded so the change does not re-import claims its own verification pass rejected.

- Gate (4a)'s spread does not depend on the hardcoded coast fraction (R5).
- The coupling's force channel has no dimensional bug; the mismatch is between the channel and the
  example's newton-valued scoring (P5).
- `score_branch`'s `Result` is imposed by `StudyEffect::reduce`'s closure bound and is not removable
  (S5).
- A rendered-log format change makes gates fail with a misattributed message rather than pass
  silently (S1).
- `WALL_CLOCK_BUDGET_S`'s headroom is appropriate for a runaway detector (S10).

## 4. Verification strategy

Each fix carries its own check, and the change is not complete on "it compiles".

1. **Unit level.** Library additions get tests in `deep_causality_cfd/tests/` per the crate's existing
   layout: `PlumeObstruction` sensed-`q∞` and Mach-bound behavior, the commit witness scalars, the
   re-seed and transition counters, `alternation_applied` on both the applied and refused paths, and
   `peak_bond`.
2. **Falsification level.** Every rebuilt gate gets a deliberately broken input proving it fails —
   a roster collapsed to one throttle must fail (4f); a report with `alternation_applied = false` must
   fail (4e); a branch missing `preserved_drag_fraction` must `Err` rather than pass (4b).
3. **Run level.** `cargo run --release -p avionics_examples --example plasma_blackout_retropulsion`,
   re-earn the bands, and diff `retropulsion_branches.csv` across two runs to prove reproducibility.
4. **Non-regression.** The corridor and weather examples must reproduce their recorded outputs unless
   D1 option (b) is chosen, in which case their bands are re-earned too.
5. **Workspace.** `bazel test //...`, then `make format && make fix`.
