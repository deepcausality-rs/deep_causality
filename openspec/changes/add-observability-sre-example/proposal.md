## Why

The new deepcausality.com landing page wants six code-example cards drawn from real example crates. Five domains have a real example today; **observability / SRE does not**. The closest existing example is `csm_examples` (causal state machine for smoke/fire/explosion sensors), which is good but framed as physical sensors rather than service telemetry. A real observability/SRE example — root-cause analysis on a cascading service failure — fills the gap and reaches a different cohort (platform engineers and SREs) than the existing examples.

The use case is **causal root-cause analysis on a cascading failure**. A chain of service-level Causaloids evaluates against a recorded incident window (DB lag, queue depth, retry rate, frontend errors). The chain reasons backward to find the earliest predicate that held, and emits a structured cause attribution. The example must use the correct DeepCausality API and must compile under the existing CI matrix.

## What Changes

- Add a new example crate at `examples/sre_examples/cascading_failure_rca/` modeling causal root-cause analysis on a synthetic incident window. Compilable, tested, runnable via `cargo run -p sre_examples --bin cascading_failure_rca`.
- Register `sre_examples` as a workspace member in the appropriate Cargo manifest and BUILD.bazel file.
- Update the deepcausality.com Astro site (separate, follow-up edit) so the landing-page `sensor-monitoring-csm` placeholder slot — or whichever finance-adjacent slot the user chooses — is **augmented** by a new `sre-cascading-failure-rca` slot. The current `sensor-monitoring-csm` card stays because the CSM example is a legitimate observability primitive; the new card complements it with a service-telemetry framing.
- Add the new example to the docs reference page for `deep_causality` if the umbrella crate is the relevant entry point.

## Capabilities

### New Capabilities
- `sre-example-cascading-failure-rca`: A compilable example crate demonstrating causal root-cause analysis on a synthetic incident window with the current DeepCausality API.

### Modified Capabilities
<!-- None. -->

## Impact

- **New directory**: `examples/sre_examples/cascading_failure_rca/` with its own `Cargo.toml`, `src/main.rs`, and any model modules the example needs.
- **Workspace registration**: appropriate Cargo and Bazel registration for the new crate.
- **CI**: the new crate compiles and tests run alongside the rest of `examples/`.
- **Website**: a follow-up edit adds the landing card and the MDX detail page.
- **No impact** on production library crates. The example is an additive consumer.
