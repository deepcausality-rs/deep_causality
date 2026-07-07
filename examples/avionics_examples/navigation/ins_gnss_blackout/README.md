# INS / GNSS-Blackout Clock Holdover (real Galileo data)

A general **GPS-denial** scenario, run on **real Galileo E14** GNSS products. Temporary loss of GNSS is a
routine, realistic hazard — jamming, spoofing, an urban canyon, a tunnel, deep terrain shadowing — and the
problem at its core is the same in every case: when GNSS is lost, **hold the relativistic clock forward and
dead-reckon the INS**, so that reacquisition on the far side is fast. This example demonstrates that
holdover loop end-to-end on real data, in one auditable `CausalFlow`.

```bash
cargo run -p avionics_examples --example ins_gnss_blackout
```

## What it shows

A vehicle (aircraft, UAV, or ground vehicle) crosses a region that **denies GNSS** for a window. The
simulation drives the navigation loop from real satellite products and runs the holdover:

1. **`deep_causality_file`** loads the real **E14** SP3 orbit + `.clk` clock (the GNSS signal) through
   the **haft IO monad** — lazy `IoAction`s composed with `and_then`/`map`, performed once at the edge.
2. **`relativistic_clock_drift_rate_kernel`** (shipped in `deep_causality_physics`) predicts the clock
   rate `dτ/dt − 1` from the **real orbit geometry**. This is the model that is **carried** across the
   outage.
3. The **grmhd `select_metric` regime detector** flips GNSS available ↔ denied from a **denial indicator**
   (an interference / jamming / signal-shadowing level) vs a critical threshold — the **two regime
   changes** (blackout entry, exit).
4. The **`alternate_value` / `branch_with`** corrective loop applies the GNSS fix when available and is
   **withheld** during the blackout: the chain runs **open-loop** (drift) through the dark, then snaps
   back on reacquisition. Every regime change and every intervention is recorded in the **`EffectLog`**.

Two runs side by side make the lever concrete (the airplane-INS insight: a continuously GPS-recalibrated
INS survives a short gap; a *pure* INS drifts away over hours):

| | Open loop (no GNSS coupling) | Closed loop (regime-gated `alternate_value`) |
|---|---|---|
| INS position error | ~**375 km** (full-day pure dead-reckoning) | **bounded** (~m), snaps back after the outage |
| Clock across the outage | undisciplined | **relativistic carry beats naive last-rate hold** vs the real measured E14 clock |

## Why it matters (the core of the GPS-denial problem)

GNSS gives **position and time**; during a blackout the position has an INS prior, but the **clock has no
external aid** — it free-runs, and its error converts directly to position error on reacquisition
(1 ns ≈ 0.3 m, and the bias is common-mode across all satellites). With a flight-grade oscillator the
**relativistic term dominates the carried-clock error**, so getting it right — the kernel, carried across
the dark — is what keeps reacquisition fast. This example is that mechanism, on real data: a
**regime-gated GNSS denial + a carried relativistic clock + corrective reacquisition, in one auditable
`CausalFlow`** — the navigation/timing core of any GPS-denied flight, independent of *why* GNSS was lost.

## Notes

- **Real data:** GFZ MGEX week 1877, day 0 (`gbm18770.sp3` / `.clk`), satellite **E14** (Galileo IOV,
  eccentric orbit → a clean relativistic clock signature), bundled under
  `examples/chronometric_examples/data/gnss/` and loaded via `deep_causality_file`.
- **Timescale:** the GNSS products are at ~5 min orbit cadence, so the modelled outage is an *extended*
  GNSS gap. The same holdover mechanism scales down to a brief denial (a tunnel, a jamming pocket) and up
  to a long one (a sustained contested-environment outage) — the bundled real data simply sets the cadence
  here.
- **Precision:** the working scalar is the single `FloatType` alias in `model.rs` (`f64` by default).
- **Self-verifying:** five gates (two regime changes; closed-loop bounded + snap-back; GNSS coupling is
  the lever; relativistic carry ≤ naive hold and bounded; intervene loop fired); the example exits
  nonzero on any regression.

## Code map

- `main.rs` — orchestration: load real data → build stream → run open/closed loops → report + gate.
- `model.rs` — the `Epoch` stream prep from real SP3/CLK, and the `CausalFlow` stages (`advance`,
  `detect_regime` = the grmhd pattern, `gps_fix`/`apply_fix` = the `alternate_value` correction).
- `utils_print.rs` — all console output and the gate evaluation.
