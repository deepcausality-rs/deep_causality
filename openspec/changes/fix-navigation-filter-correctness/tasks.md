## 1. Baseline the navigation figures before touching the filter

Design D5: measure first, so every later movement is attributable and no gate is re-derived from a
number nobody recorded.

- [x] 1.1 Record the corridor example's navigation figures: dead-reckoning drift through blackout, terminal reacquisition error, and the variance witnesses. Current post-`fix-ramc` baseline (the μ correction moved the blackout window, so these are **not** the older 0.3467 → 2.5448 → 0.2775): `err → drift → post-reacquisition` = **0.1823 → 1.5637 → 0.2802 m**, variance **2.290e-1 → 2.671e1 → 3.184e-1 m²**. Re-run to confirm before touching the filter.
      **MEASURED 2026-07-24** (fresh `--release` run, 39.7 s): `0.1823 → 1.5637 → 0.2802 m`, variance
      `2.2897e-1 → 2.6711e1 → 3.1840e-1 m²` — exact match to the committed `output.txt`. All gates PASS.
- [x] 1.2 Record the weather example's per-case drift mean/σ and terminal mean/max over its draw ensemble.
      **MEASURED** (committed `output.txt`, 6 worlds × 8 draws, 184 s): drift(dark) mean±sd / terminal mean(max), m —
      standard `41.60 ± 1.97 / 0.133 (0.195)`; hot `48.64 ± 2.40 / 0.141 (0.212)`; cold `50.68 ± 2.93 / 0.132 (0.206)`;
      polar `58.71 ± 2.27 / 0.151 (0.219)`; thin `41.28 ± 1.94 / 0.136 (0.188)`; dense `43.05 ± 1.71 / 0.141 (0.207)`.
      Gates: (4) cold factor 1.41× (≥1.2×); (4b) polar−standard 17.11 m at 5.7σ (≥2σ); (5) worst-draw terminal <1.0 m across 48 descents.
- [x] 1.3 Record `ins_gnss_blackout`'s reported figures. **MEASURED** (fresh `--release` run): open-loop 374544 m;
      closed-loop pre-outage 0.0 m / peak-in-outage 3 m / post-reacquire 0.0 m; clock carry 3532.3 ns (relativistic) vs
      3663.3 ns (naive); 264 fixes, 2 regime changes; all 5 gates PASS.
      **FINDING — outside this change's reach:** `ins_gnss_blackout` is a **standalone P-controller model**
      (`ins_vel_err`, `ins_residual_bias`, `gnss_gain`); it uses **neither `NavFilter` nor `ReentryNavEngine`** and never calls
      `predict`/`update_scalar`/`correct_position`. The proposal/design/tasks name it as a `predict` call site — that is
      incorrect and is corrected in group 6 (task 6.1a0). Its figures cannot move under this change; 5.3 will confirm they are unchanged.
- [x] 1.4 Record every tuned `q_diag` in the examples, with the `dt` it was tuned at — these values change meaning under a stated discretisation.
      **MEASURED:** one tuned diagonal, `Q_DIAG` (`avionics_examples/src/shared/constants.rs:174`), fed to **both** the
      `ImuModel` and `TrajectoryNav` in `world.rs` (the IMU's copy wins via `.with_imu`), tuned **per coupled step at the
      fixed `DT_FLIGHT = 0.1 s`** (`world.rs:105,477 .flight_dt(DT_FLIGHT)`). Values (per block): pos `1e-4`, vel `1e-4`,
      att `1e-12`, accel-bias `1e-12`, gyro-bias `1e-14`, clock-bias `1e-12`, clock-drift `1e-14`. `P0_DIAG` is the initial
      covariance, **not** process noise — it is not a `dt`-scaled quantity and is left untouched. Test-local `q` values
      (`[1.0;17]`, `[100.0;17]`, IMU grades in `closed_loop_tests`) are recorded where each test asserts a covariance.

## 2. Validate the covariance at its entry points (D3)

Lands first: it makes the degenerate-update path unreachable rather than merely guarded.

- [x] 2.1 Reject a non-finite, negative-variance or asymmetric covariance in `NavFilter::new`. Returns
      `Result<Self, PhysicsError>`; a pure diagonal is symmetric by construction, so `new` checks finiteness
      and non-negativity via the shared `validate_covariance` (`eskf.rs`).
- [x] 2.2 Same in `NavFilter::restore`, so a snapshot cannot reintroduce one — full-matrix finiteness, symmetry, and diagonal non-negativity.
- [x] 2.3 Choose and document the symmetry tolerance — `|cov[i][j] − cov[j][i]| ≤ √ε · (1 + max(|cov[i][j]|, |cov[j][i]|))`.
      `√ε` (≈1.5e-8 f64 / 3.4e-4 f32) is scale-relative with a unit absolute floor; documented on `validate_covariance`. The
      `restore_admits_float_level_asymmetry_within_tolerance` test pins a one-ULP asymmetry as admitted.
- [x] 2.4 Add tests for each rejection, and a snapshot round-trip test proving a valid filter restores exactly.
      Added to `eskf_tests.rs`: `new_rejects_a_negative_variance`, `new_rejects_a_non_finite_variance`,
      `new_accepts_a_zero_variance_on_the_boundary`, `restore_rejects_an_asymmetric_covariance`,
      `restore_rejects_a_negative_diagonal`, `restore_rejects_a_non_finite_entry`,
      `restore_admits_float_level_asymmetry_within_tolerance`, `a_valid_filter_snapshots_and_restores_exactly` — all pass.
- [x] 2.5 Confirm no shipped configuration is now refused. The only shipped covariances are `P0_DIAG` (all-positive,
      finite → valid; `world.rs` uses `.expect` on it) and snapshot round-trips (exactly symmetric by the Joseph averaging →
      valid). Neither is refused; corridor/weather re-run in group 5 confirms end-to-end.
      **NOTE (D3 scope):** validation is the necessary conditions (finiteness, symmetry, non-negative diagonal) per design D3,
      not a full-matrix LDLᵀ PSD test; the sufficient PSD guarantee against run-time round-off is the guarded update (group 3).

## 3. Guard the measurement update (D2)

- [x] 3.1 Give `update_scalar` a rejection channel; it now returns `Result<(), PhysicsError>` (was `()`).
- [x] 3.2 Reject a zero, negative or non-finite innovation covariance `s = h·P·hᵀ + r` before dividing —
      `if s <= R::zero() || !s.is_finite()` (written `<=`/`!finite`, not `!(s > 0)`, to keep `neg_cmp_op_on_partial_ord` clean).
- [x] 3.3 Reject a negative measurement variance `r` — `if r < R::zero() || !r.is_finite()`, before it enters `s`.
- [x] 3.4 Rejection is **atomic**: both guards precede any mutation in `update_scalar`, so a refused single
      update leaves state+covariance untouched. Across the three per-axis folds, `correct_position` snapshots the
      filter (`self.filter.clone()`) and rolls back on any rejection, so a refusal on axis 2 never leaves axis 1 applied.
- [x] 3.5 Propagate the rejection through `ReentryNavEngine::correct_position` (now returns `Result`); the
      `TrajectoryNav` stage threads the engine back and surfaces the error rather than continuing as if the fix folded.
- [x] 3.6 `a_degenerate_update_is_refused_and_leaves_the_filter_untouched` (`P[0][0]=0`, `r=0` → `s=0`) asserts
      `is_err()` and that state+covariance are `assert_eq!`-unchanged. `a_refused_update_leaves_the_run_able_to_continue`
      confirms a later valid fold still folds.
- [x] 3.7 `a_valid_update_is_bit_identical_after_the_guard` pins `cov[0][0]` and the position variance to exact
      `f64` values reproduced in the Joseph update's own operation order (`EXPECTED_COV00` const) — the guard is a pure
      early-return before that arithmetic, so a valid fold is byte-unchanged.

## 4. Give the process noise a discretisation (D1)

Landed alone so its effect on the examples is attributable.
**FINDING (measured in group 5, isolated to group 4):** group 4 **on its own is bit-identical** on the
examples — because they run at a **single fixed `dt = 0.1 s`** and the re-tune (4.5) is the
dimensionally-correct `÷dt` conversion, `Q_c·dt` reproduces the calibrated per-step `Q` bit-for-bit. So
the process-noise discretisation is *not* what moves the numbers here (it would, for a filter run at a
different `dt` — the proposal's general statement holds). What actually moves them is **group 6's
attitude injection** (see group 6): the proposal's "figures move" is confirmed, just via the applied
attitude correction rather than the `Q` re-tune. Recorded honestly per D5.

- [x] 4.1 Scale the process noise by `dt` in `NavFilter::predict` (`Q_d = Q_c·dt`) — `q_d[i] = process_noise_diag[i] * dt`.
- [x] 4.2 Document at the API boundary that the supplied diagonal is a continuous-time spectral density
      (units `state²/s`), with the discretisation named (first-order / Euler–Maruyama) and cited (Groves 2013 §14.2.4),
      and the Van Loan within-step coupling explicitly declined. Done on `NavFilter::predict`.
- [x] 4.3 `covariance_growth_is_invariant_under_step_refinement`: a leaf-state (accel-bias) variance over a
      fixed horizon at `dt` and `dt/2` agrees to `1e-12` and equals `T·Q_c` — exact, not merely to O(dt), because a
      leaf accumulates `P += Q_c·dt` with no propagation. Would double under the old per-step `Q`.
- [x] 4.4 `changing_dt_alone_does_not_retune_the_filter`: the position witness (position-only spectral density,
      a leaf) is `dt`-invariant to `1e-12` over a fixed horizon and equals `T·Q_c`.
- [x] 4.5 Re-tune `Q_DIAG` from per-step to per-second: `Q_c = Q_step / DT_FLIGHT` (×10 at `DT_FLIGHT = 0.1 s`),
      stated in the `Q_DIAG` doc. Magnitudes now read 10× the pre-fix per-step diagonal; `Q_c·dt` at the configured step
      reproduces the calibrated per-step process noise. No test-local `q` needed re-tuning (all qualitative; the group-3
      golden uses `update_scalar`, not `predict`).

## 5. Measure, then re-derive the gates (D5)

Do not edit a navigation gate before this group's measurements are written down.

- [x] 5.1 Re-ran the corridor (`--release`). Measured **at end of group 4** (attitude not yet in): bit-identical
      to the 1.1 baseline (`0.1823 → 1.5637 → 0.2802 m`, var `2.2897e-1 → 2.6711e1 → 3.1840e-1 m²`). **Final, after group 6**
      (the migration plan puts attitude last): leg4 error `0.2802 → 0.2804 m` and leg2 variance `2.6711e1 → 2.6710e1 m²`
      (see the group-6 finding — an applied attitude correction). Everything else (all CFD/plasma/miss figures) bit-identical.
      Gate (3) PASS; all 13 corridor gates PASS. `corridor/output.txt` regenerated to `0.2804 / 2.6710e1`.
- [x] 5.2 Re-ran the weather example. **Bit-identical at display precision** vs 1.2 baseline in every world
      (standard `41.60 ± 1.97 / 0.133`, polar `58.71 ± 2.27 / 0.151`, …). `drift(dark)` is dead-reckoned (no fixes → no
      attitude injection → exactly unchanged); the terminal (3-sig-fig) hides the sub-mm group-6 shift. Gates (4) 1.41×,
      (4b) 5.7σ, (5) worst <1.0 m — all PASS. `weather/output.txt` unchanged.
- [x] 5.3 `ins_gnss_blackout` is causally independent of the filter (group-1 finding); no code it uses changed,
      so its figures are unchanged by construction (open 374544 m; closed 0.0/3/0.0 m; clock 3532.3 vs 3663.3 ns; 5 gates PASS).
- [x] 5.4 **No navigation gate fails.** The corridor gate is relational (drift up through blackout, collapse on
      reacquisition) and the weather gates are ratio/σ-based; the tiny group-6 shift leaves all of them PASS.
- [x] 5.5 **Nothing to re-derive** — no gate bound was touched. The corridor gate remains satisfied by the relational
      pattern (`0.1823 → 1.5637 → 0.2804`, collapse on reacquisition); no bound was widened to restore a figure (D5 honoured).
- [x] 5.6 The corrected filter does **not** report worse drift — the dead-reckoned drift (leg2 error `1.5637 m`) is
      unchanged (it is deterministic, independent of the covariance). Group 4's `÷dt` re-tune is calibration-preserving at the
      fixed `dt` (bit-identical); the only movement is group 6's applied attitude correction (leg4/leg2-var, ≲0.07%),
      recorded as the honest result — the proposal's "figures move" is confirmed via that correction, not the `Q` re-tune.

## 6. Resolve the attitude-error inconsistency (D4)

Last, because it needs an owner decision and the options differ greatly in size.

**FINDING — (a) does move the example numbers a little, and that is the correction working.** The design
called (a) "primarily a covariance-legitimacy fix" for the point-mass examples on the reasoning that the
attitude *mean* stays ≈ 0 (`IMU_GYRO_BIAS = 0`, no true rotation). That is right for the **predict** path
(ω̂ = 0 ⇒ the nominal integrates to *exactly* identity — pinned by `a_zero_rate_leaves_the_nominal_exactly_at_identity`),
but it missed a second path: a **position fix** folds a small nonzero `δψ` through the position↔attitude
cross-covariance the `−[f]×` block builds over the descent, and (a) now **injects** that `δψ` into the
nominal. So after the first fix the nominal is no longer exactly identity, `C(q) ≠ I`, and the specific
force fed to the filter shifts by a hair. Measured, deterministic (two runs identical), fully isolated:
corridor leg4 `0.2802 → 0.2804 m`, leg2 variance `2.6711e1 → 2.6710e1 m²`; **every other figure
bit-identical**; weather changes below display precision (its `drift(dark)` has no fixes). All gates hold.
This is exactly the applied correction (a) exists to deliver, and it confirms the proposal's "figures move".

Resolution revised 2026-07-22 after testing the deferral against the tree: **(a) here; (b) dropped.**
`Quaternion` with `from_axis_angle`, `to_rotation_matrix`, `normalize` and `slerp` already ships in
`deep_causality_num_complex`, an existing `deep_causality_cfd` dependency, so (a) is a field plus two
call sites rather than a feature. (b) was justified solely by (a)'s cost and carried a real price — an
attitude block that is no longer an error about the current nominal.

- [x] 6.1 Added a nominal attitude `Quaternion<R>` field to `ReentryNavEngine`; `new` initialises it to
      identity, `restore` takes it as an argument (and it round-trips through the snapshot). Re-exported `Quaternion`
      from `deep_causality_cfd` since it is now in the engine's public API.
- [x] 6.1a0 **Added the angular-rate input.** `ReentryNavEngine::predict` gains `angular_rate: [R; 3]` (`ω̂`).
      **CALL-SITE CORRECTION (found against the tree):** the artifacts named `world.rs` and the `ins_gnss_blackout`
      example as `predict` call sites — **both wrong**. `world.rs` builds the engine via `ReentryNavEngine::new` (no
      `predict`); the only in-crate `predict` caller is the **`TrajectoryNav` stage**, which the corridor/weather examples
      drive. `ins_gnss_blackout` is a **standalone P-controller model** that never touches the engine. Actual call sites
      updated: `TrajectoryNav::apply`, `reentry_nav_tests`, `closed_loop_tests`. `magnav` (2-D filter) untouched.
      `ω̂` supplied by the examples = `imu.sense_angular_rate([0;3])` = gyro bias = **`[0,0,0]`** (`IMU_GYRO_BIAS = 0`,
      no true rotation), so the *predict* path leaves the nominal exactly at identity — but a **fix** still injects a
      small `δψ` from the position↔attitude cross-covariance (see the group-6 FINDING above), which is why the corridor
      numbers shift a hair.
- [x] 6.1a Integrate the gyro each step: `self.attitude = (self.attitude * Quaternion::from_axis_angle(ω̂, |ω̂|·dt)).normalize()`.
      `from_axis_angle` returns identity for a zero-length axis, so `ω̂ = 0` leaves the nominal exactly at identity (⊗ identity
      and normalise are both exact on the unit quaternion).
- [x] 6.1b Inject `δψ` in `correct_position`: `self.attitude = (Quaternion::from_axis_angle(δψ, |δψ|) * self.attitude).normalize()`
      **before** `reset_navigation_error()`. The reset is now legitimate — proven by `a_fix_injects_the_attitude_error_into_the_nominal_then_resets_it`.
- [x] 6.1c Feed `C(q)·f_body` (via `attitude.to_rotation_matrix()` and the new `mat3_vec` helper) to `filter.predict`,
      replacing `C ≈ I`. Computed in the caller (`ReentryNavEngine::predict`); `nav_transition_matrix`/`propagate` byte-unchanged (task 7.9).
- [x] 6.2 No interim departure to document: `δψ` is injected, so the attitude block *is* an error about the
      current nominal — textbook ESKF bookkeeping holds, (b)'s caveat is not incurred. (Recorded in the design D4.)
- [x] 6.3 `repeated_fixes_keep_correcting_the_nominal_rather_than_claiming_free_confidence`: 100 predict+fix cycles
      with a standing gyro bias — the nominal keeps being corrected (rotates away from identity) and the attitude error is
      reset (`< 1e-9`) after every fix, so the covariance reductions are matched by applied corrections, not claimed for free.
- [x] 6.4 The `−[f]×` coupling is unchanged in structure (task 7.9); the coast/reacquire tests
      (`closed_loop_tracks_then_drifts_then_reacquires`, `position_fix_reacquires_and_stays_on_orbit`) confirm the
      propagation stays stable and the attitude error still couples into velocity error as intended.
- [x] 6.5 `a_nonzero_rate_integrates_and_stays_normalised_across_a_long_march` (5000 steps, unit to `1e-9`) and
      `a_zero_rate_leaves_the_nominal_exactly_at_identity` (exact `assert_eq!`) cover normalisation and the zero-input identity.

## 7. Verify

- [x] 7.1 `covariance_growth_is_invariant_under_step_refinement` — leaf-state variance agrees to `1e-12` at `dt` and `dt/2`.
- [x] 7.2 `a_degenerate_update_is_refused_and_leaves_the_filter_untouched` — `is_err()` + state/covariance `assert_eq!`-unchanged.
- [x] 7.3 `new_rejects_*` / `restore_rejects_*` — negative variance, asymmetry, non-finite refused at both entry points.
- [x] 7.4 `a_fix_injects_the_attitude_error_into_the_nominal_then_resets_it` — a nonzero `δψ` is injected (nominal ≠ identity) before the block is zeroed.
- [x] 7.5 `a_valid_update_is_bit_identical_after_the_guard` (exact `f64`) + `a_resume_package_round_trips_through_disk_bit_exact` — valid path byte-unchanged.
- [x] 7.6 `cargo test -p deep_causality_cfd --release` → **895 passed, 0 failed, 1 ignored** (was 828 baseline + the new tests). No regression.
- [x] 7.7 `cargo clippy -p deep_causality_cfd -p avionics_examples --all-targets --all-features -- -D warnings` clean
      (one `needless_range_loop` was fixed by rewriting to iterators, not suppressed); `cargo fmt` applied; **no new `#[allow]`** (diff-verified).
- [x] 7.8 The only gate whose *witness* moved is the corridor drift/reacquire gate (`0.2802 → 0.2804 m`), traceable to
      group 6's applied attitude correction. It is a **relational** gate (no numeric bound), so nothing was widened; it still PASSES.
      No bound anywhere was changed to restore a pre-correction figure (D5 honoured).
- [x] 7.9 **Verified byte-unchanged:** `ins_error_state.rs` (propagate + 17-state composition) has an empty diff;
      `nav_transition_matrix`'s body is unchanged (the eskf.rs hunk is only the *added* `validate_covariance`/`max_abs`);
      the Joseph arithmetic has no added/removed lines (guards precede it, `Ok(())` follows). The sole error-dynamics
      behaviour change is `ReentryNavEngine::predict` feeding `C(q)·f_body` to `filter.predict` — the caller, matching the
      design's documented Non-Goal exception.
