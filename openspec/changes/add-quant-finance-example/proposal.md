## Why

The new deepcausality.com landing page leads with six code-example cards drawn from real example crates under `examples/`. Five of the six target domains have a real example today (aerospace, sensor monitoring, async/Tokio, biomedical, physics, counterfactual reasoning). **Quantitative finance does not.** That gap matters because finance is one of the highest-signal acquisition channels for a causal-reasoning framework, and the existing examples skew toward physical and biological systems. A real, compilable quant example closes the gap.

The use case is **causal regime change detection on a price series**: a small Causaloid chain that recognizes a transition from one market regime (low-volatility trending) to another (high-volatility mean-reverting) and emits a regime-change effect. The example must use the correct DeepCausality API (`PropagatingEffect::pure(...).bind(...)`, `Causaloid`, optional `Context`) and must compile under the existing CI matrix.

## What Changes

- Add a new example crate at `examples/finance_examples/regime_change/` modeling causal regime detection on a synthetic OHLC price series. Compilable, tested, runnable via `cargo run -p finance_examples --bin regime_change`.
- Register `finance_examples` as a workspace member in the appropriate Cargo manifest and BUILD.bazel file.
- Update the deepcausality.com Astro site (separate, follow-up edit) so the landing-page `async-event-inference` placeholder slot is replaced by `quant-finance-regime-change`, with a real snippet and a real source link.
- Add the new example to the docs reference page for `deep_causality` if the umbrella crate is the relevant entry point.

## Capabilities

### New Capabilities
- `finance-example-regime-change`: A compilable example crate demonstrating causal regime-change detection on a price series with the current DeepCausality API.

### Modified Capabilities
<!-- None. The website's landing capability already exists; updating the slot is an editorial change handled in a follow-up touch, not a capability change. -->

## Impact

- **New directory**: `examples/finance_examples/regime_change/` with its own `Cargo.toml`, `src/main.rs`, and any model modules the example needs.
- **Workspace registration**: appropriate Cargo and Bazel registration for the new crate.
- **CI**: the new crate compiles and tests run alongside the rest of `examples/` (same matrix the other example crates already use).
- **Website**: a follow-up edit swaps the landing card. Until that lands, the existing `async-event-inference` card remains.
- **No impact** on production library crates. The example is an additive consumer.
