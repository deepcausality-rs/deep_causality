## 1. Crate scaffold

- [ ] 1.1 Create `examples/sre_examples/` directory with `Cargo.toml`, `README.md`, and `cascading_failure_rca/` subdirectory containing the binary
- [ ] 1.2 Register `sre_examples` as a workspace member in the appropriate Cargo manifest
- [ ] 1.3 Add `BUILD.bazel` mirroring the sibling example crates' Bazel layout
- [ ] 1.4 Verify `cargo build -p sre_examples` compiles cleanly with no warnings under the standard lint profile

## 2. Synthetic incident window

- [ ] 2.1 Implement a deterministic `IncidentWindow` generator: `db_p99`, `queue_depth`, `queue_capacity`, `retries_per_sec`, `baseline_rps`, `error_rate`, indexed by timestamp
- [ ] 2.2 Plant DB lag as the seeded root cause; let downstream signals fire after a fixed delay
- [ ] 2.3 Unit test asserting byte-identical output for a fixed seed

## 3. Service-level Causaloids

- [ ] 3.1 `db_lag` — Causaloid returning true when `db_p99 > threshold`
- [ ] 3.2 `queue_backlog` — true when `queue_depth > 0.85 * queue_capacity`
- [ ] 3.3 `retry_storm` — true when `retries_per_sec > 4.0 * baseline_rps`
- [ ] 3.4 `frontend_5xx` — true when `error_rate > 0.05`

## 4. Causal composition

- [ ] 4.1 Build a `CausaloidGraph` with nodes for all four service-level rules
- [ ] 4.2 Add directional edges: `db_lag → queue_backlog → retry_storm → frontend_5xx`
- [ ] 4.3 Wrap the graph as a composed `Causaloid` via `Causaloid::from_causal_graph`
- [ ] 4.4 Evaluate the composed Causaloid against the synthetic incident window

## 5. Root-cause attribution

- [ ] 5.1 Walk the resulting `EffectLog` to find the earliest predicate whose entry timestamp is smallest
- [ ] 5.2 Print the attribution: which predicate fired, when, what value triggered it
- [ ] 5.3 If the public API exposes a `reason_back`-style helper, use it; otherwise compute from the log directly
- [ ] 5.4 Unit test: detector attributes the cause to `db_lag` on the seeded fixture
- [ ] 5.5 Unit test: a second fixture where `queue_backlog` is the planted root yields a `queue_backlog` attribution

## 6. Optional Context layer

- [ ] 6.1 Place the four thresholds in a `BaseContext` Datoid set
- [ ] 6.2 Demonstrate mutating one threshold mid-run if it fits within ~10 lines of source; otherwise skip

## 7. Tests + CI

- [ ] 7.1 `cargo test -p sre_examples` passes locally
- [ ] 7.2 `make build && make test` from the repo root green
- [ ] 7.3 Confirm the new crate appears in the CI matrix output

## 8. Documentation

- [ ] 8.1 Write `examples/sre_examples/README.md`: one paragraph on RCA, one paragraph on the rules, link to the docs detail page on deepcausality.com

## 9. Website integration (follow-up, separate diff)

- [ ] 9.1 Add a new slot in `website/web/src/components/home/examples.ts` for `sre-cascading-failure-rca`, pointing at the real crate path
- [ ] 9.2 Author `website/web/src/content/examples/en/sre-cascading-failure-rca.mdx` as a thin pointer to the new crate
- [ ] 9.3 Add the new slug to the glyph map in `ExampleCard.astro`
- [ ] 9.4 Decide whether to keep `async-event-inference` and `sensor-monitoring-csm` on the landing page alongside the new card, or rotate one out; the grid stays at six cards
