# Granger via the Causal Monad

Runs Granger's predictive-causality test on `PropagatingProcess<f64, (), SeriesContext>` with the [`AlternatableContext`](../../../../deep_causality_core/src/traits/alternatable_context/mod.rs) trait.

## How to run

```bash
cargo run -p classical_causality_examples --example granger_via_monad
```

## The test

The Granger question: does past oil-price history improve the prediction of next-period shipping activity? One chain makes two predictions:

1. **Factual:** predict from a Context that carries both `shipping_activities` and `oil_prices`.
2. **Counterfactual:** the same chain, but `.alternate_context(no_oil_ctx)` swaps in a Context with an empty `oil_prices` vector before the bind runs.

The example compares each prediction's error against the actual Q5 shipping value. If the factual prediction is closer, the oil series Granger-causes shipping.

## The mechanism

```rust
let factual = factual_series()?;
let counterfactual = without_oil(&factual)?;

let factual_pred = value_of(&run(factual.clone()))?;  // start + bind, then read the f64
let counter_pred = value_of(
    &start(factual)
        .alternate_context(counterfactual)            // swap world
        .bind(predict_shipping),                      // same predictor
)?;
```

`value_of` returns the value the chain carries, or the error that ended it, and `?` propagates that error out of `main`.

The single-stage `predict_shipping` bind reads the series and the model's coefficients from the Context. It averages past shipping, adds the `SHIPPING_TREND` coefficient, and adjusts by `(mean(oil_prices) - OIL_BASELINE) * OIL_COEFFICIENT` *only when* the oil series is non-empty, so the counterfactual prediction omits the oil adjustment. The actual Q5 value the predictions are scored against stays in `main`, outside the predictor's Context.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/granger` | `classical_via_causal_monad/granger` |
|---|---|---|
| Time-series data lives in | `GrangerContext` Datoid nodes with `OIL_PRICE_ID` / `SHIPPING_ACTIVITY_ID` tags, one `DiscreteTime` node per quarter | `SeriesContext` with one `Data<Quantity>` node per series; quarters are vector positions |
| Model coefficients live in | Three Datoids at the front of each `GrangerContext` | Three `Quantity::Scalar` nodes in each `SeriesContext` |
| Counterfactual world built by | Iterate factual Context, skip every `OIL_PRICE_ID` Datoid | `without_oil(&factual)` rebuilds the world from the factual shipping series and coefficients with an empty oil series |
| Two-world plumbing | Two separate contextual `Causaloid` instances, each bound to its own `Arc<RwLock<GrangerContext>>` | One chain; `.alternate_context(no_oil)` switches worlds |
| Lines of code | ~300 across 2 files | ~255 in a single file |

Both versions produce identical numbers for the same fixture.

## Reference

For background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the RCM example, which introduces the single-chain, two-context pattern.
