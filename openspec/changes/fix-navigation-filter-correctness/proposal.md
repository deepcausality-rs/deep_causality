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
  **BREAKING (result-level):** every currently-tuned `q_diag` changes meaning, so the corridor and
  weather examples' navigation figures move.
- **Guard the innovation covariance** and validate the inputs that reach it, so a degenerate
  measurement cannot poison the state and covariance with `NaN`.
- **Resolve the attitude-error inconsistency**: an error state is injected before it is reset, or it is
  not reset. Which of those is achieved is a design decision (see below), but the invariant is the
  requirement.
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
  guard), `new` / `restore` (covariance validation).
- `deep_causality_cfd/src/navigation/reentry_nav.rs` — `correct_position` (the injection/reset gap).
- `deep_causality_cfd/src/navigation/ins_error_state.rs` — `reset_navigation`, which currently zeroes
  the attitude block unconditionally.
- Possibly the engine's state: resolving the attitude gap by injection requires a nominal attitude,
  which `ReentryNavEngine` does not currently carry.

**Evidence**
- `examples/avionics_examples/cfd/plasma_blackout/{corridor,weather}` — every navigation figure:
  dead-reckoning drift through blackout, terminal reacquisition error, and the variance witnesses. Both
  examples gate on these.
- `examples/avionics_examples/navigation/ins_gnss_blackout` — the dedicated navigation example.
- Any tuned `q_diag` in those examples, which changes meaning under a stated discretisation.

**Risk**
- Re-tuning is required, not optional: with `Q` scaled by `dt` the existing diagonals produce different
  covariance growth, so the examples' drift and reacquisition numbers move and their gates must be
  re-derived from the corrected filter rather than restored.
- The attitude resolution may be larger than a bug fix. Injecting `δψ` needs a nominal attitude the
  engine does not have; the alternatives are narrower but each has a cost. See the design.
- No public API signature change is forced, though `update_scalar` returning `()` is what prevents it
  from rejecting bad input today.
