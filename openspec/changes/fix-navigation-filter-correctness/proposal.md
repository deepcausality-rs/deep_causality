# Fix the error-state Kalman filter's correctness defects

## Why

The GNSS-denial navigation stack is the crate's estimation layer — the thing that decides how far a
vehicle drifts through a comms blackout and how well it reacquires. Three defects in it are
independent of each other and all three affect what the filter reports.

**The process noise is not a discretisation of anything.** `NavFilter::predict` takes `dt`, uses it to
propagate the state and build the transition matrix, then adds `Q` with no `dt` factor at all:

```rust
self.cov = mat_add(&fpft, &diag(&process_noise_diag));
```

Covariance therefore grows with **step count, not elapsed time**. Halving `dt` over a fixed horizon
doubles the accumulated process noise. The supplied diagonal is not a continuous-time spectral density
(`Q_d ≈ Q_c·dt`) nor any other stated discretisation, so the filter's tuning is silently bound to one
step size and any change of `dt` re-tunes the filter as a side effect.

**The innovation covariance is divided by unguarded.** `update_scalar` computes
`s = h·P·hᵀ + r` and then `k[i] = ph[i] / s` with no positivity check, and it returns `()` — it has no
channel through which to reject. The path is reachable from the public API: `NavFilter::new` accepts an
arbitrary covariance diagonal with no validation, `NavFilter::restore` accepts an arbitrary matrix, and
`ReentryNavEngine::correct_position` passes `r_var` straight through. With `P[i][i] = 0` and
`r_var = 0`, `s = 0`, `k = 0/0 = NaN`, and the NaN is written into both the state and every entry of
the covariance.

**An error state is zeroed without ever being injected.** `correct_position` runs three sequential
scalar updates on the position axes. Each one shrinks the *whole* covariance through the Joseph form,
including the attitude block, via the position↔attitude cross-covariance — so the filter becomes more
confident about attitude. It then injects only `δp` and `δv` into the nominal and calls
`reset_navigation_error()`, which zeroes position, velocity **and attitude** error.

The attitude correction is therefore estimated, credited in the covariance, and discarded. And it
cannot be applied: **`ReentryNavEngine` carries no nominal attitude** — its fields are `position`,
`velocity`, `filter`, `tau_offset`, `elapsed` — and `attitude_error` is read nowhere in the crate. In a
correct ESKF, resetting the error state after injection is sound precisely *because* the error was
transferred to the nominal. Zeroing an error that was never transferred discards the estimate while
keeping the covariance reduction it justified, leaving the filter permanently overconfident in
attitude.

To the code's credit, `update_scalar`'s covariance update is a correct **Joseph form** with
re-symmetrisation, citing Groves (2013) §3.4.3 — the audit confirmed this positively. The defects are
around it, not in it.

Audit `AUDIT-REPORT.md` §4b and §9 Phase 2 item 9. The navigation module is one of three rated
`not-ready`.

## What Changes

- **Give the process noise a stated discretisation.** `Q` becomes a documented continuous-time
  spectral density scaled by `dt` (or another explicitly stated and cited discretisation), so
  covariance growth tracks elapsed time and the filter's tuning survives a change of step size.
  Every currently-tuned `q_diag` changes *meaning* — but the examples run at a single fixed `dt`, and the
  re-tune is the dimensionally-correct `÷dt` conversion, so `Q_c·dt` reproduces the calibrated per-step
  `Q` **bit-for-bit**. This change alone therefore does not move the example figures (measured); it would
  for a filter run at a different step size, which is the point.
- **Guard the innovation covariance** and validate the inputs that reach it, so a degenerate
  measurement cannot poison the state and covariance with `NaN`.
- **Resolve the attitude-error inconsistency**: an error state is injected before it is reset, or it is
  not reset. The invariant is the requirement; the resolution is **option (a), decided 2026-07-24** —
  carry a nominal attitude, integrate the sensed angular rate into it, inject `δψ` on a fix, and only
  then zero the attitude error. This makes the reset legitimate rather than working around it, and it is
  larger than a bug fix (see the design's D4 and the Impact/Risk below).
- **Validate filter construction.** `NavFilter::new` and `restore` accept arbitrary covariance today;
  a covariance must be at least symmetric and positive-semidefinite to be one.

Explicitly **not** in scope: the 17-state composition and the transition matrix (the audit confirmed
the skew-symmetric coupling blocks against the standard ESKF form), the Joseph update itself, the
Encke/Cowell integrator switch, the IMU noise model, and the relativistic clock. Those are separate
findings or were confirmed correct.

## Capabilities

### New Capabilities

- `eskf-filter-correctness`: the error-state Kalman filter's numerical and structural contract — the
  process-noise discretisation is stated rather than implied, the update is guarded against degenerate
  inputs, the covariance is a covariance, and the error-state lifecycle (estimate → inject → reset) is
  closed. This capability exists because all three defects share one root: the filter's *contracts*
  were never written down, so nothing distinguished a tuned constant from a discretisation, or an
  injected error from a discarded one.

### Modified Capabilities

None. The navigation stack has no existing capability spec — which is itself part of why these defects
persisted. This change introduces the first one rather than modifying prose that does not exist.

## Impact

**Code**
- `deep_causality_cfd/src/navigation/eskf.rs` — `predict` (Q scaling), `update_scalar` (innovation
  guard + `Result` return), `new` / `restore` (covariance validation).
- `deep_causality_cfd/src/navigation/reentry_nav.rs` — `predict` (gains an angular-rate argument, and
  computes `C(q)·f_body` for the filter's specific force), `correct_position` (propagates the update
  rejection; injects `δψ` into the nominal attitude), and a **new nominal-attitude `Quaternion<R>`
  field** on `ReentryNavEngine` (which today carries `gm`, `position`, `velocity`, `filter`,
  `tau_offset`, `elapsed` and no attitude).
- `deep_causality_cfd/src/navigation/ins_error_state.rs` — `reset_navigation` stays as-is; zeroing the
  attitude block is now legitimate because `δψ` was injected first.
- `deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs` — the `TrajectoryNav` stage is the
  **only** in-crate `ReentryNavEngine::predict` caller (the corridor and weather examples drive it); it
  gains the new angular-rate argument, sourced from `ImuModel::sense_angular_rate`.
  **CORRECTED against the tree during implementation:** the earlier draft named `world.rs` and the
  `ins_gnss_blackout` example as `predict` call sites. Neither is: `world.rs` only *builds* the engine
  (`ReentryNavEngine::new`), and `ins_gnss_blackout` is a **standalone P-controller model** that never
  uses `NavFilter`/`ReentryNavEngine` at all. `examples/avionics_examples/navigation/magnav` uses a
  **different** 2-D filter (`predict(vel_x, vel_y, dt)`), also unaffected. The test call sites
  (`reentry_nav_tests`, `closed_loop_tests`) take the new argument.

**Evidence**
- `examples/avionics_examples/cfd/plasma_blackout/{corridor,weather}` — every navigation figure:
  dead-reckoning drift through blackout, terminal reacquisition error, and the variance witnesses. Both
  examples gate on these.
- `examples/avionics_examples/navigation/ins_gnss_blackout` — **not affected**: it is a standalone
  P-controller model that uses neither `NavFilter` nor `ReentryNavEngine` (confirmed against the tree).
- Any tuned `q_diag` in those examples, which changes meaning under a stated discretisation.

**Risk**
- Re-tuning is required for the *contract*, but at the examples' fixed `dt` the dimensionally-correct
  `÷dt` re-tune is calibration-preserving, so group 4 alone is **bit-identical** on the examples (measured).
  The numbers that do move come from the attitude correction (a), not the `Q` re-tune.
- The attitude resolution (a) is larger than a bug fix, and the tree makes it larger than the design's
  2026-07-22 revision assumed. Two things beyond "carry a `Quaternion`" were found by verifying against
  the source before implementation:
  - **`predict` has no angular-rate input.** `ReentryNavEngine::predict` and `NavFilter::predict` carry
    no `ω̂`, so integrating a nominal attitude (6.1a) forces a signature change on `ReentryNavEngine::predict`
    and a new argument at its call sites. **CORRECTED against the tree:** those call sites are the
    `TrajectoryNav` stage (which the corridor/weather examples drive) and the `reentry_nav`/`closed_loop`
    tests — **not** `world.rs` (it only builds the engine) and **not** `ins_gnss_blackout` (standalone model).
    The `Quaternion` type ships; the *sensing path* did not exist and was added (`ImuModel::sense_angular_rate`).
  - **The examples are point-mass 3-DOF with no true angular rate** (`IMU_GYRO_BIAS = [0,0,0]`), so the
    *predict* path leaves the nominal exactly at identity. (a) is therefore **primarily** a
    covariance-legitimacy fix — it stops the position fix claiming an attitude-covariance reduction with no
    correction behind it. It is **not purely** a covariance fix, though: a position fix folds a small `δψ`
    through the position↔attitude cross-covariance and (a) injects it, which nudges the nominal off identity
    and shifts the corridor's leg4 error `0.2802 → 0.2804 m` and leg2 variance `2.6711e1 → 2.6710e1 m²`
    (≲0.07%, deterministic, gate-safe — the applied correction doing exactly its job). Everything else is
    bit-identical; the weather shift is below display precision.
- **Public API signatures do change.** `update_scalar` gains a rejection channel (returns a `Result`
  instead of `()`), and `predict` gains the sensed angular rate for (a). Both are in-crate only
  (`publish = false`), and `update_scalar`'s only caller is `correct_position`.
- **`correct_position` computes the nav-frame specific force from the nominal DCM** (`C(q)·f_body`)
  rather than assuming `C ≈ I`. This changes the *value* fed to the `−[f]×` coupling but **not**
  `nav_transition_matrix` or `propagate` themselves, which already take the specific force as an
  argument — so task 7.9's "does not alter `nav_transition_matrix` / the 17-state composition / the
  Joseph update" still holds at the function level, while the error-dynamics *behaviour* changes by
  design. This is a deliberate, documented departure from the Tier-A `C ≈ I` model, recorded as an
  exception in the design's Non-Goals.
