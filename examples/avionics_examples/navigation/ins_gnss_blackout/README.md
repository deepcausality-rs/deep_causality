# INS / GNSS-Blackout Clock Holdover (real Galileo data)

This example runs a **GPS-denial** holdover loop end-to-end on **real Galileo E14** GNSS products, in one auditable
`CausalFlow`. Temporary loss of GNSS is a routine hazard (jamming, spoofing, an urban canyon, a tunnel, deep terrain
shadowing), and the core problem is the same in every case: when GNSS is lost, **hold the relativistic clock forward
and dead-reckon the INS**, so that reacquisition on the far side is fast.

```bash
cargo run -p avionics_examples --example ins_gnss_blackout
```

## What it shows

A vehicle (aircraft, UAV, or ground vehicle) crosses a region that **denies GNSS** for a window. The
simulation drives the navigation loop from real satellite products and runs the holdover:

1. **`deep_causality_file`** loads the real **E14** SP3 orbit + `.clk` clock (the GNSS signal) through
   the **haft IO monad**: `read_gnss_single_satellite` returns a lazy `IoAction`, run once at the edge.
2. **`relativistic_clock_drift_rate_kernel`** (from `deep_causality_physics`) predicts the clock
   rate `dτ/dt − 1` from the **real orbit geometry**. The loop **carries** this model across the
   outage.
3. The **grmhd `select_metric` regime detector** flips GNSS available ↔ denied by comparing a **denial
   indicator** (an interference / jamming / signal-shadowing level) with a critical threshold, producing
   the **two regime changes** (blackout entry, exit).
4. The **`alternate_value_if` / `branch_with`** corrective loop applies the GNSS fix when available and
   **withholds** it during the blackout: the chain runs **open-loop** (drift) through the dark, then snaps
   back on reacquisition. The **`EffectLog`** records every regime change and every intervention.

Two runs side by side show the effect (as in aircraft INS, a continuously GPS-recalibrated INS survives a
short gap; a *pure* INS drifts away over hours):

| | Open loop (no GNSS coupling) | Closed loop (regime-gated `alternate_value_if`) |
|---|---|---|
| INS position error | ~**375 km** (full-day pure dead-reckoning) | **bounded** (~m), snaps back after the outage |
| Clock across the outage | undisciplined | **relativistic carry beats naive last-rate hold** vs the real measured E14 clock |

## Why it matters (the core of the GPS-denial problem)

GNSS gives **position and time**. During a blackout the position has an INS prior, but the **clock has no
external aid**: it free-runs, and its error converts directly to position error on reacquisition
(1 ns ≈ 0.3 m, and the bias is common-mode across all satellites). With a flight-grade oscillator the
**relativistic term dominates the carried-clock error**, so carrying the kernel across the dark keeps
reacquisition fast. The example implements that mechanism on real data: a **regime-gated GNSS denial + a
carried relativistic clock + corrective reacquisition, in one auditable `CausalFlow`**. It forms the
navigation and timing core of any GPS-denied flight, whatever caused the GNSS loss.

## Notes

- **Real data:** GFZ MGEX week 1877, day 0 (`gbm18770.sp3` / `.clk`), satellite **E14** (Galileo IOV,
  eccentric orbit → a clean relativistic clock signature), bundled under
  `examples/physics_examples/chronometric_gm_recovery/data/gnss/` and loaded via `deep_causality_file`.
- **Timescale:** the GNSS products are at ~5 min orbit cadence, so the modelled outage is an *extended*
  GNSS gap. The same holdover mechanism scales down to a brief denial (a tunnel, a jamming pocket) and up
  to a long one (a sustained contested-environment outage); the bundled data sets the cadence here.
- **Precision:** the working scalar is the single `FloatType` alias in `main.rs` (`f64` by default).
- **Self-verifying:** five gates (two regime changes; closed-loop bounded + snap-back; GNSS coupling
  bounds the error; relativistic carry ≤ naive hold and bounded; intervene loop fired); the example exits
  nonzero on any regression.

## Code map

- `main.rs` — orchestration: load real data → build stream → run open/closed loops → report + gate.
- `model.rs` — the `Epoch` stream prep from real SP3/CLK, and the `CausalFlow` stages (`advance`,
  `detect_regime` = the grmhd pattern, `gps_fix`/`apply_fix` = the `alternate_value_if` correction).
- `utils_print.rs` — all console output and the gate evaluation.
