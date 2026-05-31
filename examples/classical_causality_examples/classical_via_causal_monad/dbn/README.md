# DBN via the Causal Monad

Umbrella World as a Dynamic Bayesian Network, implemented directly on `PropagatingProcess<f64, WeatherState, WeatherContext>`. The notable property: **all three carrier channels are exercised**, and the conditional probability tables are alternable via [`alternate_context`](https://docs.rs/deep_causality_core/latest/deep_causality_core/trait.AlternatableContext.html), so a climate-regime change becomes one method call instead of a graph rebuild.

## How to run

```bash
cargo run -p classical_causality_examples --example dbn_via_monad
```

## What the chain looks like

| Channel | Type | Role |
|---|---|---|
| **Value** | `f64` | Today's rain probability emitted by each step. |
| **State** | `WeatherState` | Markov state: `rained_yesterday`, `day` counter, `rainy_days`, `umbrellas_carried`. Evolves through every `bind`. |
| **Context** | `WeatherContext` | The CPTs for the current climate regime (P(rain \| rained_yesterday), P(rain \| dry_yesterday)). Constant within a regime; alternated when the regime changes. |

One bind = one day. The `step_day` closure reads the climate from the Context, the previous day's outcome from the State, then emits today's probability, updates the State, and returns.

## The regime-change demonstration

Two 10-day simulations run side by side:

1. **Baseline all the way** (`run_baseline_only`). The dry-leaning baseline climate (P(rain | rain) = 0.40, P(rain | dry) = 0.20) for the full 10 days.
2. **Regime change mid-stream** (`run_regime_change`). Baseline for days 1-5, then `process.alternate_context(monsoon_climate())` switches to the monsoon regime (P(rain | rain) = 0.95, P(rain | dry) = 0.60), and days 6-10 run under the alternated CPTs.

The Markov state (`rained_yesterday`, the running counters) threads through the regime change untouched. The `EffectLog` contains exactly one `!!ContextAlternation!!` entry recording the switch.

## What `alternate_context` adds over the Causaloid version

In the [Causaloid version](../../classical_via_causaloid/dbn), CPT changes mean rebuilding the Causaloid (or the graph) with the new probability function inside its closure. The Context is used for tracking historical state, not for the model parameters. The model and its parameters live together inside the Causaloid.

In the monad version, the model is the `bind` closure; its parameters are the `WeatherContext`. Swapping parameters is `alternate_context(new_cpts)`. The chain, the State, and the umbrella accounting all continue uninterrupted. A regime change is a one-line operator, not a graph rebuild.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/dbn` | `classical_via_causal_monad/dbn` |
|---|---|---|
| CPTs live in | `Causaloid` closure body (hard-coded) | `WeatherContext` (alternable) |
| Previous-day state lives in | `BaseContext` Datoid (manually updated each tick via `RwLock`) | `WeatherState` (threaded by `bind`) |
| Regime change mechanism | Rebuild Causaloid or rewrite the closure | `.alternate_context(new_cpts)` mid-loop |
| Sampling | `deep_causality_rand::rng()` (non-deterministic) | Deterministic `p > 0.5` rule for reproducibility |
| Lines of code | ~200 across 3 files (main + model + types) | ~160 in a single file |

The two implementations make different design trade-offs: the causaloid version mirrors the textbook DBN architecture (Context = world state, Causaloid = the probability function); the monad version separates *model parameters* (Context) from *model state* (State), which is what makes the regime-change operator clean.

## Reference

For the conceptual background, see the [Dynamic Causality concept page](https://docs.deepcausality.com/concepts/dynamic-causality/) and the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
