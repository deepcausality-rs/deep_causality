# Classical Causality Examples

Five classical causal-inference methods (CATE, DBN, Granger, RCM, SCM), each implemented **twice** in the DeepCausality framework so the two implementation styles can be read side by side.

```bash
cargo run -p classical_causality_examples --example <example_name>
```

## The two approaches

| Folder | Mechanism                                                                                                                   | Where the alternation lives |
|---|-----------------------------------------------------------------------------------------------------------------------------|---|
| [`classical_via_causaloid/`](classical_via_causaloid/) | `Causaloid` + `CausaloidGraph` + manual Context construction                                                                | Contextoid clones outside the carrier |
| [`classical_via_causal_monad/`](classical_via_causal_monad/) | `PropagatingProcess` + `bind` chain + the [`Alternatable`](../../deep_causality_core/src/traits/alternatable/mod.rs) traits | `alternate_value` / `alternate_context` / `alternate_state` on the carrier itself |

### A short detour: what "Pearl's abduction" is and why both approaches work without it

Counterfactual reasoning is the rung-3 question: "given the world as it actually is, what would have happened if X had been different?" Answering it cleanly requires a way to hold the *actual* world fixed while changing one thing in it.

Pearl's Structural Causal Model handles this with a three-step procedure: **abduction → action → prediction**.

- **Abduction** is the inference step. An SCM expresses the world as a fixed graph of variables with hidden exogenous noise. To pin down "the world as it actually is," you must *infer* the noise from the factual observations: compute the posterior over the unobserved noise variables given the data you saw. This step is the well-known bottleneck — for general non-linear or neural SCMs it is often intractable, which is why much of the deep-causal-model literature (e.g., Pawlowski, Castro & Glocker, NeurIPS 2020) exists specifically to work around it.
- **Action** applies the `do(X := x)` operator: replace the structural equation for `X` with a constant.
- **Prediction** re-runs the modified SCM, with the abduced noise held fixed, to compute the counterfactual outcome.

DeepCausality takes a different foundational move: it **separates the causal law (the `Causaloid` or the `bind` closures) from the world state (the `Context`).** Once the world state is an explicit data structure rather than implicit noise, the counterfactual question becomes: *clone the world, change a piece of it, re-evaluate the same causal law again*.

That "clone + modify + re-evaluate" pattern is **contextual alternation**, and it is the mechanism both approaches use. The difference is only where it happens:

- **`via_causaloid`**: alternation happens *outside* the carrier. You clone a `BaseContext`, modify a `Contextoid` (a `Datoid` value, a `Spaceoid` location, a `Tempoid` time slice, etc.), then evaluate a `Causaloid` against the new Context.
- **`via_monad`**: alternation happens *on the carrier itself*. The `Context` is a Rust struct carried through the `PropagatingProcess`; `.alternate_context(other)`, `.intervene(value)`, or `.alternate_state(state)` swap one channel mid-chain and emit a distinctive audit-log entry recording the switch.

Either way, both approaches sidestep abduction because the world state is explicitly stated upfront. For engineering systems where state is what your software already records (sensor logs, patient histories, network topology), this is a straight win.

## Picking an approach: a practitioner guide

Both approaches produce identical numbers for the same problem. The question is which fits your team and your codebase better. The five worked examples give concrete data points.

### Lines of code per implementation

| Method | `via_causaloid` LOC | `via_monad` LOC | Reduction |
|---|---:|---:|---:|
| RCM     | 161 | 147 |  8% |
| DBN     | 205 | 183 | 10% |
| CATE    | 233 | 174 | 25% |
| Granger | 240 | 142 | 40% |
| SCM     | 334 | 182 | 45% |
| **Total** | **1 173** | **828** | **29%** |

LOC counts include all `.rs` files in the example directory (main, model, supporting modules). The spread is more informative than the average: the monad's advantage scales with how much **scaffolding** the causaloid version would otherwise need (multiple contextual `Causaloid` instances, manual `Contextoid` / `Datoid` construction, multi-file rung splits). When the causaloid version is already a single tight `CausaloidGraph` with one Context (RCM, DBN), the two approaches are close to parity.

### What each approach is naturally good at

| Axis | `via_causaloid` strength | `via_monad` strength |
|---|---|---|
| **Topology** | Real graphs: diamonds, joins, conditional sub-graphs. `CausaloidGraph::add_edge` and graph traversals are first-class. | Sequential or near-sequential pipelines expressed as `bind` chains. |
| **Number of causal units** | Dozens to hundreds of first-class `Causaloid` values; register, hot-swap, store in collections. | A handful of stages inlined as `bind` closures. |
| **Context heterogeneity** | Multiple `Contextoid` types in one `BaseContext` (`Datoid` + `Spaceoid` + `Tempoid` + `SpaceTempoid` + `Symboid`). | A single `Context` Rust struct carries the world. |
| **Alternation granularity** | Coarse, infrequent world rebuilds (clone Context, modify, build a new contextual `Causaloid`). | Fine, frequent channel substitutions (`alternate_value`, `alternate_context`, `alternate_state`) emitted with one method call. |
| **Audit trail style** | Structural attribution: which `Causaloid` produced which effect. | Linear log of alternation events with distinctive markers (`!!Intervention!!`, `!!ContextAlternation!!`, `!!StateAlternation!!`) emitted automatically. |
| **Reuse across models** | The same `Causaloid` can appear in many graphs. | The chain is defined once per problem; closures are inlined. |
| **Ceremony per pipeline** | Graph construction (`new`, `add_causaloid`, `add_edge`, `freeze`) plus an evaluation strategy. | `start(ctx).bind(...).bind(...)`. |
| **Type system carries the world** | Contextoid IDs + `Data::get_data()` lookups (runtime). | Plain Rust struct fields (compile-time). |

### Concrete decision rules

Pick **`via_causaloid`** when **any** of these is true:

- The causal model is a real graph: diamond patterns, parallel branches, joins, conditional sub-graphs.
- The world state is genuinely heterogeneous and benefits from typed `Contextoid` kinds (`Datoid` + `Spaceoid` + `Tempoid` + `SpaceTempoid` + `Symboid`).
- Causal units need to be first-class values: registered in a catalogue, hot-swapped at runtime, traversed by graph algorithms.
- The same model needs multiple evaluation strategies (full graph, subgraph from a cause, shortest path between causes).
- Downstream tooling (Effect Ethos, custom traversals) expects to attribute findings to named structural units.

Pick **`via_monad`** when **all** of these are true:

- The pipeline is sequential or near-sequential.
- The world state fits in a single Rust struct (no need for multiple `Contextoid` kinds).
- The audit trail of *alternation events* matters more than structural attribution to named units.
- You want minimal ceremony and the alternation operator visible at the call site (`.intervene(x)`, `.alternate_context(c)`, `.alternate_state(s)`).
- The team prefers plain Rust structs and `bind` chains over graph construction.

### Hybrid is a real option

The two approaches share the carrier (`PropagatingEffect` / `PropagatingProcess`), so a single system can mix them per stage. A Causaloid graph emits a `PropagatingEffect` that a `bind` chain consumes; a `bind` chain produces input for a Causaloid evaluation. The [flight envelope monitor](../avionics_examples/flight_envelope_monitor/) demonstrates this: a Causaloid collection over five sensor checks, a three-step `bind` chain for state estimation, and a Causaloid hypergraph of six envelope protections, all running through one `PropagatingProcess<T, FlightState, AircraftConfig>` with state and audit log threaded across every stage.

If a system's high-level structure is a graph but its individual nodes contain sequential reasoning, the hybrid is often the right shape: graph at the top, `bind` chains inside. Each approach is at its best where it is structurally most natural.

## Example matrix

| Method | `via_causaloid` | `via_monad` |
|---|---|---|
| **CATE** (heterogeneity) | [cate](classical_via_causaloid/cate/README.md) | [cate](classical_via_causal_monad/cate/README.md) |
| **DBN** (Umbrella World) | [dbn](classical_via_causaloid/dbn/README.md) | [dbn](classical_via_causal_monad/dbn/README.md) |
| **Granger** (time-series predictive causality) | [granger](classical_via_causaloid/granger/README.md) | [granger](classical_via_causal_monad/granger/README.md) |
| **RCM** (Rubin potential outcomes) | [rcm](classical_via_causaloid/rcm/README.md) | [rcm](classical_via_causal_monad/rcm/README.md) |
| **SCM** (Pearl's Ladder) | [scm](classical_via_causaloid/scm/README.md) | [scm](classical_via_causal_monad/scm/README.md) |

## Run commands

### Causaloid approach

| Example | Command |
|---|---|
| CATE | `cargo run -p classical_causality_examples --example cate_via_causaloid` |
| DBN | `cargo run -p classical_causality_examples --example dbn_via_causaloid` |
| Granger | `cargo run -p classical_causality_examples --example granger_via_causaloid` |
| RCM | `cargo run -p classical_causality_examples --example rcm_via_causaloid` |
| SCM | `cargo run -p classical_causality_examples --example scm_via_causaloid` |

### Causal-Monad approach

| Example | Command |
|---|---|
| RCM | `cargo run -p classical_causality_examples --example rcm_via_monad` |
| SCM | `cargo run -p classical_causality_examples --example scm_via_monad` |
| DBN | `cargo run -p classical_causality_examples --example dbn_via_monad` |
| CATE | `cargo run -p classical_causality_examples --example cate_via_monad` |
| Granger | `cargo run -p classical_causality_examples --example granger_via_monad` |

