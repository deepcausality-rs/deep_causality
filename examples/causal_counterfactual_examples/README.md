# Causal Counterfactual Examples

Five worked examples showing counterfactual intervention with the causal
monad: what `intervene` gives you beyond `bind`.

These follow the Judea Pearl tradition. Hold a factual world in mind,
hypothetically substitute a different value, read off the difference. The
intervention is a one-shot, retrospective, estimand-defining value
substitution.

This crate sits alongside
[`causal_correction_examples`](../causal_correction_examples),
[`causal_discovery_examples`](../causal_discovery_examples) and
[`causal_uncertain_examples`](../causal_uncertain_examples).

## What `intervene` does that `bind` cannot

`Intervenable::intervene(self, new_value)` replaces the value carried by
an in-flight chain. Several properties follow from that contract; `bind`
chains can fake any of them with branching code or external state, but
the monad makes them properties of the type instead of properties of the
program.

| Property | Demonstrated by |
|---|---|
| Same chain, different worlds. Replay the same `bind` pipeline under several interventions without branching code. | All five examples |
| Audit trail in `EffectLog`. Every `intervene` appends `!!Intervention!!: <old> replaced with <new>`. The intervention history becomes inspectable. | `counterfactual_envelope_fault`, `counterfactual_cascade_failure`, `counterfactual_resection_intervention` |
| Error-state preservation. If the chain is already in error, `intervene` is a no-op. Intervention cannot paper over an upstream bug. | (Property of the trait; not the punchline of a dedicated example.) |
| State and Context preservation. On `PropagatingProcess`, only the value swaps. Accumulated `State` and read-only `Context` are untouched. | `counterfactual_envelope_fault`, `counterfactual_cascade_failure` |
| Intervention site encodes a causal claim. Where in the chain you intervene says what counts as upstream of the manipulated quantity. | `counterfactual_treatment_options` |
| Comparative evaluation as the estimand. The difference between factual and counterfactual is the causal quantity you wanted. | `counterfactual_treatment_effect`, `counterfactual_treatment_options`, `counterfactual_cascade_failure` |
| Interventions compose into chains. A cascade is a chain of interventions, each applied to the result of the previous one. | `counterfactual_cascade_failure` |

## Examples

The intervention is a one-shot value substitution. The chain is evaluated
on the factual world and on one or more counterfactual worlds; the
difference is the quantity of interest.

| Example | Monad | What it shows | Command |
|---|---|---|---|
| `counterfactual_treatment_effect` | `PropagatingEffect` (stateless) | CATE as `do(T=1) − do(T=0)` on one chain. Age strata show treatment-effect heterogeneity. | `cargo run -p causal_counterfactual_examples --example counterfactual_treatment_effect` |
| `counterfactual_envelope_fault` | `PropagatingProcess<_, FlightState, AircraftConfig>` | A stall airspeed injected mid-chain. Same aircraft, same configuration, different value, different verdict. | `cargo run -p causal_counterfactual_examples --example counterfactual_envelope_fault` |
| `counterfactual_treatment_options` | `PropagatingEffect` | One patient, two intervention sites. Beta-blocker (intervene on blood pressure) versus surgical clip (intervene on wall shear stress). | `cargo run -p causal_counterfactual_examples --example counterfactual_treatment_options` |
| `counterfactual_cascade_failure` | `PropagatingProcess<_, NetworkState, NetworkConfig>` | A fluid network's N-k contingency analysis written as a chain of interventions. One trigger leads to total collapse; another is absorbed gracefully. | `cargo run -p causal_counterfactual_examples --example counterfactual_cascade_failure` |
| `counterfactual_resection_intervention` | `PropagatingEffect` | Epilepsy surgery screening as `do(connectome = resected_at_R)` for each candidate region. The hub-node resection comes out as the most curative. | `cargo run -p causal_counterfactual_examples --example counterfactual_resection_intervention` |

## Suggested reading order

1. `counterfactual_treatment_effect` is the simplest case. The estimand
   is defined as a difference of two interventional chains; that is the
   entire example.
2. `counterfactual_envelope_fault` is the same idea on a stateful
   `PropagatingProcess`. It establishes that intervention swaps the
   value while `State` and `Context` stay put.
3. `counterfactual_treatment_options` puts two interventions in the
   same chain at different sites. The lesson is that where you
   intervene encodes a claim, not just a numerical override.
4. `counterfactual_cascade_failure` is the cascade centrepiece. Each
   cascade step is an intervention applied to the result of the
   previous one; the `EffectLog` accumulates into a forensic timeline.
5. `counterfactual_resection_intervention` is the audit-trail example.
   In a clinical setting the artefact that matters is which region was
   resected, not just the simulator's number. The log records that for
   you.

For the closed-loop, real-time counterpart of these one-shot
interventions, see
[`causal_correction_examples`](../causal_correction_examples).

## How this differs from the existing intervene examples

The codebase already has small demonstrations of `intervene` in:

* [`starter_example`](../starter_example), which walks Pearl's three
  rungs on a smoking-tar-cancer chain.
* [`classical_causality_examples/scm`](../classical_causality_examples/scm),
  which writes the same three rungs as separate files.
* [`core_examples/propagating_effect_counterfactual`](../core_examples/examples/propagating_effect_counterfactual.rs)
  and `propagating_process_counterfactual`, the smallest possible
  stateless and stateful demos.
* [`avionics_examples/geometric_tcas`](../avionics_examples/geometric_tcas),
  where pilot override is modelled as intervention.
* [`material_examples/structural_health_monitor`](../material_examples/structural_health_monitor),
  which asks what would have happened without the observed crack.

Those are the *what is `intervene`* examples. The five here are the
*what does `intervene` give you that `bind` cannot* examples, drawn from
domains that were not built around intervention from the start: clinical
trial design, avionics, hemodynamics, distribution-network reliability,
and surgical planning.

## Adding New Examples

1. Create directory: `<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` following the [standard template](../physics_examples/README.md)
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "your_example/main.rs"
   ```
