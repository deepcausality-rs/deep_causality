## Context

The existing `examples/` tree contains ten example crates spanning aerospace, biomedical, physics, classical causality, materials, chronometry, async runtime integration, and the starter Pearl-ladder walkthrough. None of them addresses quantitative finance. The landing page for the new deepcausality.com explicitly wants finance in its six-card grid because the engineering audience for a causal-reasoning framework includes a sizeable quant cohort.

This change adds one example crate. The use case is **regime change detection**, picked because it exercises the library's strengths (composing stage-gated rules, threading a small amount of state, optionally consulting a Context that updates over time) and stays short enough to read in one sitting.

The example must be honest. It uses the real API. It does not fake a backtest. It does not pretend to be an alpha strategy. It demonstrates the library on a real domain problem at the smallest size that still reads as a quant example.

## Goals / Non-Goals

**Goals:**
- A compilable example crate at `examples/finance_examples/regime_change/`.
- A 60-to-150-line `main.rs` that builds a small Causaloid chain (or `PropagatingEffect::bind` sequence), runs it over a synthetic OHLC series, and prints regime-change events.
- Uses the same idiom as the other example crates: `PropagatingEffect::pure(...).bind(...)` chains; `Causaloid::from_causal_fn` or `from_contextual_causal_fn` for any rule that needs to read from a Context.
- Includes a synthetic data generator inside the crate so the example is self-contained; no external file inputs.
- Builds and runs in CI alongside the other `examples/` crates.

**Non-Goals:**
- Real market data. The example uses a deterministic synthetic series.
- A backtest framework. The example processes one pass over the series and stops.
- Performance benchmarking. Cleanliness of the example matters more than throughput.
- A reusable library of finance primitives. Anything reusable can be promoted later; this change is one crate.
- Network IO, file IO, or any external dependency beyond what the rest of the example tree already uses.

## Decisions

### D1. Regime model
Two regimes:
- **Trending low-vol**: realized volatility below threshold AND directional move (close-to-close drift) above threshold.
- **Mean-reverting high-vol**: realized volatility above threshold AND autocorrelation of log returns negative.

Each regime is a Causaloid. The transition rule is a third Causaloid that fires when both legs hold over a window. The output is a `PropagatingEffect<RegimeEvent>` where `RegimeEvent` is `{ Enter(Regime), Exit(Regime), NoChange }`.

This is small enough to read at landing-page snippet length and rich enough that the rules are not trivially boolean.

### D2. Data
Synthetic OHLC series generated inside the crate via a seeded deterministic generator. Two regimes baked in: a quiet trending segment for the first half, a noisy mean-reverting segment for the second half. The example output should show one Enter and one Exit event near the midpoint, which is the smoke test for the rules firing correctly.

### D3. State and context
- **State**: a small `RegimeState` struct holding the most recent realized vol, drift, and autocorrelation estimates plus the current regime. Threaded through the chain as the `state` field of a `PropagatingProcess`.
- **Context**: optional. The thresholds (vol, drift, autocorrelation cutoffs) live in a `BaseContext` Datoid. Mutating the threshold mid-run demonstrates the dynamic-causality angle without requiring the example to be longer. Skip the Context if it adds more than ~20 lines of setup; cap on length wins.

### D4. Style and length
- One `main.rs`, with model code factored into `src/model.rs` if `main.rs` exceeds 150 lines.
- The snippet that lands on the website's landing page is 12–18 lines of the most representative chunk.
- Comments where the why is non-obvious. No tutorial prose inside the source — the website's detail page carries the walkthrough.

### D5. Crate layout
Mirror the existing example crates (`avionics_examples`, `medicine_examples`, `physics_examples`):
```
examples/
  finance_examples/
    Cargo.toml
    README.md           # one paragraph; points at the docs detail page
    regime_change/
      main.rs
      model.rs          # if needed
      types.rs          # if needed
```
Sibling examples (basket pricing, options, etc.) can land later under `finance_examples/`; this change ships one.

### D6. Tests
Add a unit test or two in the crate that asserts:
- The synthetic generator produces deterministic output for a fixed seed.
- The regime detector emits `Enter(MeanRevertingHighVol)` somewhere in the back half of the series.

These keep CI honest without bloating the example.

### D7. Website integration
Out of scope here. A follow-up touch to `website/web/src/components/home/examples.ts` swaps the landing card from `async-event-inference` to `quant-finance-regime-change`. The MDX detail page lives at `src/content/examples/en/quant-finance-regime-change.mdx`. Both are mechanical changes once the example crate compiles.

## Risks / Trade-offs

- **Risk**: the example reads as "fake finance" if the rules are too crude. **Mitigation**: keep the rules grounded in textbook regime indicators (realized vol, drift, autocorrelation) rather than invented quantities. Add a one-line citation in `README.md`.
- **Risk**: the synthetic generator is too cooperative and the regime detector always fires correctly, masking the example's robustness. **Mitigation**: include a unit test with a deliberately noisy seed and assert the detector handles it.
- **Risk**: scope creep into a backtest framework. **Mitigation**: hard cap at one `main.rs` plus model. Backtesting is a separate change.
- **Trade-off**: using a `Context` adds power but also adds reading length. The decision is to use one only if it stays cheap; otherwise stay stateless.

## Migration Plan

1. Author the example crate; verify `cargo run -p finance_examples --bin regime_change` produces a sensible regime-change event stream.
2. Add it to the workspace; verify `cargo test --workspace` passes.
3. Add the Bazel target alongside the sibling examples.
4. CI green.
5. (Follow-up edit, not in this change's diff) swap the landing card and add the MDX detail page on the Astro site.

## Open Questions

- Should the example use `PropagatingProcess` (stateful) or `PropagatingEffect` (stateless)? Recommendation: stateless first; promote to `PropagatingProcess` if `RegimeState` makes the chain cleaner. Decide during implementation.
- Add an `Effect Ethos` guard? Possible but probably out of scope; it would make the example longer without adding finance content. Defer.
