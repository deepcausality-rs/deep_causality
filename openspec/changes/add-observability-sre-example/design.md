## Context

The existing `examples/` tree has `csm_examples` (sensor-driven CSM), `tokio_example` (async runtime integration), and `avionics_examples` (5-stage health pipeline). All three are observability-adjacent but none of them is framed as a service-telemetry use case. SREs and platform engineers — a sizeable engineering cohort — see causal-reasoning libraries through that lens. This change adds the example they would look for.

The use case is a cascading-failure root-cause analysis. The incident window contains four signals: database p99 latency, queue depth, retries-per-second, and frontend error rate. Each signal is wrapped in a Causaloid. The chain composes them with directional dependencies (DB lag → queue full → retry storm → frontend 5xx), and the analysis walks the chain backward to find the earliest predicate that held inside the window.

The example must read as a real post-mortem aid, not a toy.

## Goals / Non-Goals

**Goals:**
- A compilable example crate at `examples/sre_examples/cascading_failure_rca/`.
- A 60-to-150-line `main.rs` that builds the four service-level Causaloids, composes them as a `CausaloidGraph` (or chained `PropagatingEffect`s), runs them against a synthetic incident window, and prints a structured root-cause attribution.
- Uses the same idiom as the other example crates.
- Includes a synthetic incident-window generator inside the crate.
- Builds and runs in CI alongside the other `examples/` crates.

**Non-Goals:**
- Real telemetry ingest. The example uses a synthetic window.
- A full observability pipeline. No OpenTelemetry, no Prometheus, no log ingestion. Those belong to a follow-up integration example, not this one.
- A general RCA framework. The example demonstrates the pattern at the smallest size that still reads as RCA.
- ML-based anomaly detection. The Causaloids encode thresholds, not learned models.

## Decisions

### D1. Signal model
Four service-level Causaloids:
- **`db_lag`** — true when DB p99 > threshold inside the window.
- **`queue_backlog`** — true when queue depth > 0.85 × capacity.
- **`retry_storm`** — true when retries-per-second > 4 × baseline.
- **`frontend_5xx`** — true when frontend error rate > 5%.

Each Causaloid is built with `Causaloid::from_causal_fn` over an `IncidentWindow` input type. Composition uses `Causaloid::from_causal_graph` with edges `db_lag → queue_backlog → retry_storm → frontend_5xx` to encode the directional dependency.

### D2. Backward reasoning
The example calls `evaluate` to confirm the chain fires on the full window, then walks the graph in reverse to find the earliest predicate whose `EffectLog` entry has the smallest timestamp. The earliest one is the root cause. The example prints the full chain of causation, not just the root, so a reader can see why.

If the public API does not expose a clean backward-walk helper, the example computes it from the `EffectLog` directly — the log entries carry enough information.

### D3. Data
A deterministic synthetic `IncidentWindow` generator that bakes in a known root cause (DB lag) and lets the other signals fire downstream of it after a fixed delay. The example output should attribute the cause to `db_lag` at the smoke-test level.

### D4. State and context
- **State**: none. The example uses stateless `PropagatingEffect`, not `PropagatingProcess`. State adds noise without paying for itself here.
- **Context**: thresholds live in a small `BaseContext` Datoid set, demonstrating how a deployment-specific threshold profile would live in a real system. Mutating the threshold mid-run is optional and out of scope unless it fits in 10 lines.

### D5. Style and length
- One `main.rs`, factored into `src/model.rs` if it exceeds 150 lines.
- Snippet for the landing page is 12–18 lines of the most representative chunk.

### D6. Crate layout
```
examples/
  sre_examples/
    Cargo.toml
    README.md
    cascading_failure_rca/
      main.rs
      model.rs    # if needed
      types.rs    # if needed
```
Sibling SRE examples (deploy regression, capacity planning, etc.) can land later under `sre_examples/`; this change ships one.

### D7. Tests
- Unit test: the detector identifies `db_lag` as the root cause on the seeded synthetic window.
- Unit test: deterministic output for a fixed seed.

### D8. Website integration
Out of scope here. A follow-up touch to the Astro site adds a new card (does not replace the CSM one; the CSM example stands on its own). The MDX detail page lives at `src/content/examples/en/sre-cascading-failure-rca.mdx`.

## Risks / Trade-offs

- **Risk**: the synthetic window is too cooperative. **Mitigation**: include a unit test where the planted root cause is `queue_backlog` rather than `db_lag` and assert the detector still picks the right one.
- **Risk**: backward reasoning bleeds into a separate API surface the library does not yet support. **Mitigation**: compute the answer from the `EffectLog` directly in the example. If a `reason_back` helper later lands in the library, this example becomes a candidate caller.
- **Risk**: the example becomes a strawman for an OpenTelemetry integration. **Mitigation**: explicit non-goal; defer OT to a separate change.
- **Trade-off**: graph composition (`Causaloid::from_causal_graph`) vs flat `bind` chain. Graph composition reads better for this domain because the directional dependency is the whole point of RCA. Decision: graph.

## Migration Plan

1. Author the example crate; verify `cargo run -p sre_examples --bin cascading_failure_rca` prints a correct attribution.
2. Add it to the workspace; verify `cargo test --workspace` passes.
3. Add the Bazel target alongside the sibling examples.
4. CI green.
5. (Follow-up edit, not in this change's diff) add the landing card and MDX detail page on the Astro site.

## Open Questions

- Should the example use `Causaloid::from_causal_graph` or chained `PropagatingEffect::bind`? Decision: graph, because the directional dependency is the point. Confirm during implementation that the graph API in the current `deep_causality` version supports the shape cleanly.
- Should the example show counterfactual reasoning ("what if we had increased queue capacity?")? Tempting but probably out of scope; keep RCA focused.
