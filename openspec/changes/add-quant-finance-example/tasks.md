## 1. Crate scaffold

- [ ] 1.1 Create `examples/finance_examples/` directory with `Cargo.toml`, `README.md`, and `regime_change/` subdirectory containing the binary
- [ ] 1.2 Register `finance_examples` as a workspace member in the appropriate Cargo manifest
- [ ] 1.3 Add `BUILD.bazel` mirroring the sibling example crates' Bazel layout
- [ ] 1.4 Verify `cargo build -p finance_examples` compiles cleanly with no warnings under the standard lint profile

## 2. Synthetic data

- [ ] 2.1 Implement a deterministic OHLC generator seeded from a constant; first half trending low-vol, second half mean-reverting high-vol
- [ ] 2.2 Add a unit test asserting byte-identical output for a fixed seed

## 3. Causal logic

- [ ] 3.1 Define `RegimeEvent` enum: `Enter(Regime)`, `Exit(Regime)`, `NoChange`. Define `Regime` enum: `TrendingLowVol`, `MeanRevertingHighVol`
- [ ] 3.2 Define a `RegimeState` carrying realized vol, drift, autocorrelation, and current regime
- [ ] 3.3 Build the trending Causaloid: returns true when realized vol < threshold AND drift > threshold
- [ ] 3.4 Build the mean-reverting Causaloid: returns true when realized vol > threshold AND log-return autocorrelation < 0
- [ ] 3.5 Build the transition Causaloid: emits `Enter` / `Exit` / `NoChange` based on current vs proposed regime
- [ ] 3.6 Wire the three Causaloids into a `PropagatingEffect`-based chain (or `PropagatingProcess` if the state field carries its weight)

## 4. Optional Context layer

- [ ] 4.1 Decide during implementation: do thresholds live in a `BaseContext` Datoid, or stay as constants? Keep whichever option is shorter. If Context, add a small demonstration mid-run mutation showing the dynamic-causality angle
- [ ] 4.2 If Context is used, ensure the example still fits the length budget

## 5. Tests

- [ ] 5.1 Unit test: detector emits `Enter(MeanRevertingHighVol)` somewhere in the back half of the series
- [ ] 5.2 Unit test: deterministic output for a fixed seed
- [ ] 5.3 Verify `cargo test -p finance_examples` passes locally

## 6. Documentation

- [ ] 6.1 Write `examples/finance_examples/README.md`: one paragraph on the use case, one paragraph on the rules, link to the docs detail page on deepcausality.com
- [ ] 6.2 Verify the source file the website snippet will quote stays at a stable byte range (the website extracts a 12–18-line chunk; mark it with a clear comment if necessary)

## 7. CI

- [ ] 7.1 Run `make build && make test` from the repo root; confirm green
- [ ] 7.2 Confirm the new crate appears in the CI matrix output

## 8. Website integration (follow-up, separate diff)

- [ ] 8.1 Swap `async-event-inference` slot in `website/web/src/components/home/examples.ts` for `quant-finance-regime-change`, pointing at the real crate path
- [ ] 8.2 Author `website/web/src/content/examples/en/quant-finance-regime-change.mdx` as a thin pointer to the new crate
- [ ] 8.3 Update `examples.ts` glyph map for the new slug
