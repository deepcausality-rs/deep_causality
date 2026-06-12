# corrective_ddos_detector

Volumetric DDoS detection and mitigation as a closed-loop corrective
`intervene` over the causal monad.

```
cargo run -p causal_correction_examples --example corrective_ddos_detector
```

## What it shows

A virtual network interface carries traffic from a deterministic generator.
An array-backed sliding window, carried as Markovian `State`, holds the
recent throughput as a rolling baseline. Each second the new measured
throughput is scored as a z-score against that baseline. When the score
stays above the parametric threshold (`sigma_threshold`, 3σ) for
`trigger_slots` consecutive seconds (5), a volumetric attack is declared and
the loop fires `.intervene(THROTTLE_ON)`. The NIC's regulator reads that
command from the value channel and rate-limits the interface to the throttle
ceiling, mitigating the flood from the next tick.

Unlike the other four corrective examples, this one runs **closed loop
only**; there is no open-loop variant. The point is not a setpoint
controller. It is a stateful, real-time anomaly detector embedded in the
correction loop.

## The three channels

| Channel | Type | Carries |
|---|---|---|
| Value | `ThrottleState` (`u8`) | the NIC regulator command (`THROTTLE_OFF` / `THROTTLE_ON`); the intervention flips it |
| State | `DetectorState` | the sliding-window baseline plus per-tick accounting (counters, detection markers, history) |
| Context | `DetectorConfig` | read-only baseline, attack schedule, thresholds, and mitigation ceiling |

```rust
type DetectorProcess<T> = PropagatingProcess<T, DetectorState, DetectorConfig>;
```

## Detecting a *sustained* surge

The detector scores each incoming sample against the baseline and admits
only non-anomalous samples to the window. That withholding is deliberate.
The naive alternative pushes every sample, then tests whether the window max
exceeds the window's own mean + 3σ. That self-masks. As flood samples
accumulate they inflate the window's mean and σ, so the z-score of the max
collapses back under the threshold within a few ticks, and "3σ for 5
consecutive seconds" never holds. Keeping the baseline clean lets the flood
read as anomalous for its full duration.

A wider window (30 one-second samples, capacity over-allocated 2× to 60)
keeps the baseline mean and σ steady; the attack starts only after the window
has filled with clean traffic.

## Control flow

The whole loop is one fluent `iterate_n` over the monad:

```rust
CausalFlow::from(initial_process())
    .iterate_n(N_TICKS as usize, |tick| {
        tick.bind(analyze_tick).branch_with(
            // trigger: 5 consecutive anomalies, and not already throttled
            |throttle, state, ctx| {
                let cfg = ctx.expect("DetectorConfig present");
                state.consecutive_anomalies >= cfg.trigger_slots && *throttle == THROTTLE_OFF
            },
            // mitigate: record it, then intervene the throttle ON
            |anomaly| anomaly.update_state(record_mitigation).intervene(THROTTLE_ON),
            // no detection: business as usual
            |normal| normal,
        )
    })
    .into_process()
```

The `throttle == OFF` guard makes the mitigation fire exactly once.

## Reading the output

The run prints the per-tick throughput, z-score, and throttle trajectories, a
summary line, and the per-tick `EffectLog` (including the `!!Intervention!!`
entry). With the default configuration:

- the baseline holds ~385–415 Mbps (z ≈ 0) while nominal;
- the attack begins at tick 40 and throughput climbs to ~900 Mbps;
- the z-score reads 10.9 → 46.2 across the surge (the withheld samples keep
  the baseline clean);
- the 5th consecutive anomalous second trips the trigger: `first anomaly at
  tick 42`, `detected/mitigated at tick 46`;
- from tick 47 the regulator clamps throughput to the 420 Mbps ceiling;
- overload is bounded to 5 ticks, inside the 8-tick service objective.

Onset (`first_anomaly_at`, tick 42) and confirmed detection (the trigger,
`mitigated_at`, tick 46) are reported separately; they sit `trigger_slots`
apart by construction.

## Extending it: standing down after the attack abates

The example latches the throttle ON and leaves it on. A production detector
would also *release* mitigation once the attack is over. The robust rule is
to confirm abatement on the **offered (raw inbound) load**, not the throttled
delivered rate. A rate-limiter forces the delivered signal normal, so judging
recovery on it is circular. A scrubbing appliance still observes the
true inbound rate while limiting what it forwards; release the throttle once
that raw signal has been normal for, say, twice the detection duration
(`2 × trigger_slots`). The exact policy depends on the deployment and is
intentionally left out of this minimal example.
