# DBN via the Causal Monad

Models Umbrella World as a Dynamic Bayesian Network on `PropagatingProcess<FloatType, WeatherState, BaseContext>`, where `FloatType` is a local alias for `f64`. The example uses **all three carrier channels**, and [`alternate_context`](../../../../deep_causality_core/src/traits/alternatable_context/mod.rs) swaps the conditional probability tables, so a climate-regime change takes one method call instead of a graph rebuild.

## How to run

```bash
cargo run -p classical_causality_examples --example dbn_via_monad
```

## What the chain looks like

| Channel | Type | Role |
|---|---|---|
| **Value** | `FloatType` (`f64`) | Today's rain probability emitted by each step. |
| **State** | `WeatherState` | Markov state: `rained_yesterday`, `day` counter, `rainy_days`, `umbrellas_carried`. Evolves through every `bind`. |
| **Context** | `BaseContext` | The CPTs for the current climate regime, stored as two Datoid contextoids (P(rain \| rained_yesterday), P(rain \| dry_yesterday)). Constant within a regime; alternated when the regime changes. |

One bind is one day. The `step_day` closure reads the climate from the Context and the previous day's outcome from the State, then emits today's probability and updates the State.

## The regime-change demonstration

Two 10-day simulations run side by side:

1. **Baseline all the way** (`run_baseline_only`). The dry-leaning baseline climate (P(rain | rain) = 0.40, P(rain | dry) = 0.20) for the full 10 days.
2. **Regime change mid-stream** (`run_regime_change`). Baseline for days 1-5, then `process.alternate_context(monsoon_climate())` switches to the monsoon regime (P(rain | rain) = 0.95, P(rain | dry) = 0.60), and days 6-10 run under the alternated CPTs.

The Markov state (`rained_yesterday`, the running counters) passes through the regime change untouched. The `EffectLog` contains one `!!ContextAlternation!!` entry recording the switch.

## What `alternate_context` adds over the Causaloid version

In the [Causaloid version](../../classical_via_causaloid/dbn), changing the CPTs means rebuilding the Causaloid (or the graph) with the new probability function inside its closure. The Context tracks historical state; the model and its parameters live together inside the Causaloid.

In the monad version, the model is the `bind` closure and its parameters are the climate `BaseContext`. `alternate_context(new_cpts)` swaps the parameters while the chain, the State, and the umbrella accounting continue uninterrupted.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/dbn` | `classical_via_causal_monad/dbn` |
|---|---|---|
| CPTs live in | `Causaloid` closure body (hard-coded) | Climate `BaseContext` (alternable) |
| Previous-day state lives in | `BaseContext` Datoid (manually updated each tick via `RwLock`) | `WeatherState` (threaded by `bind`) |
| Regime change mechanism | Rebuild Causaloid or rewrite the closure | `.alternate_context(new_cpts)` mid-loop |
| Sampling | `deep_causality_rand::rng()` (non-deterministic) | Deterministic `p > 0.5` rule for reproducibility |
| Lines of code | ~250 across 3 files (main + model + types) | ~205 in a single file |

The causaloid version mirrors the textbook DBN architecture (Context = world state, Causaloid = the probability function). The monad version separates *model parameters* (Context) from *model state* (State), which reduces the regime change to one operator.

## Reference

For background, see the [Dynamic Causality concept page](https://docs.deepcausality.com/concepts/dynamic-causality/) and the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
