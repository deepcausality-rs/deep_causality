# Granger via the Causal Monad

Granger's predictive-causality test on `PropagatingProcess<f64, (), SeriesContext>` using the [`AlternatableContext`](https://docs.rs/deep_causality_core/latest/deep_causality_core/trait.AlternatableContext.html) trait.

## How to run

```bash
cargo run -p classical_causality_examples --example granger_via_monad
```

## The test

The Granger question: does including past oil-price history improve our prediction of next-period shipping activity? Two predictions, one chain:

1. **Factual** — predict from a Context that carries both `shipping_activities` and `oil_prices`.
2. **Counterfactual** — same chain, but `.alternate_context(no_oil_ctx)` swaps to a Context whose `oil_prices` vector is empty before the bind runs.

The error of each prediction is compared against the actual Q5 shipping value. If the factual prediction is closer, the oil series Granger-causes shipping.

## The mechanism

```rust
let factual_pred    = run(factual_series());                                       // start + bind
let counter_pred    = start(factual_series())
    .alternate_context(without_oil(&factual_series()))                             // swap world
    .bind(predict_shipping)                                                        // same predictor
    .value;
```

The single-stage `predict_shipping` bind reads the series from the Context. It averages past shipping, adds a trend, and adjusts by `(mean(oil_prices) - 50.0) * 0.5` *only when* the oil series is non-empty. The counterfactual world emits its prediction without that oil-driven adjustment.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/granger` | `classical_via_causal_monad/granger` |
|---|---|---|
| Time-series data lives in | `BaseContext` Datoid nodes with `OIL_PRICE_ID` / `SHIPPING_ACTIVITY_ID` tags | `SeriesContext { oil_prices: Vec<f64>, shipping_activities: Vec<f64> }` |
| Counterfactual world built by | Iterate factual Context, skip every `OIL_PRICE_ID` Datoid | One-line `without_oil(&factual)` returning a new `SeriesContext` with `oil_prices: vec![]` |
| Two-world plumbing | Two separate contextual `Causaloid` instances, each bound to its own `Arc<RwLock<BaseContext>>` | One chain; `.alternate_context(no_oil)` switches worlds |
| Lines of code | ~160 across 2 files | ~125 in a single file |

Both versions produce identical numbers for the same fixture.

## Reference

For the conceptual background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the RCM example which establishes the single-chain-two-contexts pattern.
