# DeepCausality Examples Overview

This directory contains a collection of examples demonstrating various features and applications of the `DeepCausality`
library. Each example showcases how to model and reason about causal relationships in different scenarios.

## Table of Contents

- [EPP Example: Conditional Average Treatment Effect (CATE)](#epp-example-conditional-average-treatment-effect-cate)
- [EPP Example: Causal State Machine (CSM)](#epp-example-causal-state-machine-csm)
- [EPP Example: Causal State Machine with Effect Ethos](#epp-example-causal-state-machine-with-effect-ethos)
- [EPP Example: Dynamic Bayesian Network (DBN)](#epp-example-dynamic-bayesian-network-dbn)
- [EPP Example: Granger Causality](#epp-example-granger-causality)
- [EPP Example: Rubin Causal Model (RCM)](#epp-example-rubin-causal-model-rcm)
- [EPP Example: Pearl's Ladder of Causation](#epp-example-pearls-ladder-of-causation)
- [Starter Example](#starter-example)
- [Tokio Integration Example](#tokio-integration-example)

---

## EPP Example: Conditional Average Treatment Effect (CATE)

**Location:** `examples/epp_cate`

This example demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), can
model and calculate the Conditional Average Treatment Effect (CATE). It answers the question: "What is the average
effect of a new medication on blood pressure, specifically for the subgroup of patients over the age of 65?"

**Key Concepts:**

- **Rubin Causal Model (RCM)**: Showcases potential outcomes and contextual reasoning.
- **Contextual Alternation**: Simulating parallel realities (treatment vs. control) by modifying context.
- **Causal Logic (`Causaloid`)**: Encapsulating the drug's effect.
- **Population & Subgroup Selection**: Filtering `BaseContext` instances.

**How to Run:**
From within the `examples/epp_cate` directory:

```bash
cargo run --bin example-cate
```

This example highlights a core strength of the EPP: the explicit separation of causal logic from the state of the world,
enabling powerful counterfactual reasoning.

## EPP Example: Causal State Machine (CSM)

**Location:** `examples/epp_csm`

This example demonstrates how the `DeepCausality` library can be used to build a Causal State Machine (CSM). It models a
simple industrial monitoring system with three sensors (smoke, fire, explosion) that trigger corresponding actions.

**Key Concepts:**

- **Pearl's Rung 2 (Intervention)**: Provides a formal bridge between causal reasoning and deterministic intervention.
- **`Causaloid`s**: Encapsulate sensor trigger conditions.
- **`CausalState`s**: Represent individual sensors.
- **`CausalAction`s**: Define interventions to be executed.
- **`CSM`**: Orchestrates the evaluation and action-firing process.

**How to Run:**
From within the `examples/epp_csm` directory:

```bash
cargo run --bin example-csm
```

The CSM acts as a powerful bridge between the abstract world of causal reasoning and the concrete world of action and
intervention.

## EPP Example: Causal State Machine with Effect Ethos

**Location:** `examples/epp_csm_effect_ethos`

This example demonstrates the integration of a **Causal State Machine (CSM)** with an **Effect Ethos** to add a layer of
deontic (normative) reasoning to a reactive system. It models a temperature monitoring system where alerts are subject
to predefined norms.

**Key Concepts:**

- **`CausalState` & `CausalAction`**: Standard CSM components.
- **`EffectEthos`**: A deontic reasoning engine that evaluates `ProposedAction`s against norms.
- **`Teloid` (Norm)**: A single rule within the ethos (e.g., making an alert impermissible).

**How to Run:**
From the root of the `deep_causality` project:

```bash
cargo run --release --bin example_effect_ethos
```

The `EffectEthos` successfully intercepts and blocks actions that would otherwise be executed, providing a powerful
mechanism for building safer and more robust systems.

## EPP Example: Dynamic Bayesian Network (DBN)

**Location:** `examples/epp_dbn`

This example demonstrates how `DeepCausality` can model a simple Dynamic Bayesian Network (DBN), specifically the "
Umbrella World" scenario. It showcases the library's ability to model temporal causal processes.

**Key Concepts:**

- **Temporal Causal Processes**: Modeling processes over discrete time slices.
- **Dynamic Context**: Representing the timeline as a single, dynamic `Context`.
- **`Causaloid`s**: Encapsulating conditional probability tables (CPTs) for state variables.
- **`CausaloidGraph`**: Modeling causal dependencies.

**How to Run:**
From within the `examples/epp_dbn` directory:

```bash
cargo run --bin example-dbn
```

The EPP provides a flexible alternative to traditional DBNs by externalizing time into a dynamic `Context`, simplifying
the modeling of complex temporal dependencies.

## EPP Example: Granger Causality

**Location:** `examples/epp_granger`

This example demonstrates how `DeepCausality` can model a Granger Causality test. It answers: "Do past changes in oil
prices Granger-cause future changes in shipping activity?"

**Key Concepts:**

- **Counterfactual Reasoning**: Comparing predictive accuracy under different contexts.
- **Factual vs. Counterfactual Contexts**: Creating alternate realities (with/without oil price history).
- **`Causaloid`**: Encapsulating the predictive model.

**How to Run:**
From within the `examples/epp_granger` directory:

```bash
cargo run --bin example-granger
```

The EPP's explicit separation of causal logic from the state of the world makes it trivial to perform the powerful
counterfactual reasoning required for a Granger Causality test.

## EPP Example: Rubin Causal Model (RCM)

**Location:** `examples/epp_rcm`

This example demonstrates how to implement a simple Rubin Causal Model (RCM) scenario using `DeepCausality`. It
showcases **Contextual Alternation** to directly compute potential outcomes and determine an Individual Treatment
Effect (ITE).

**Key Concepts:**

- **RCM**: Comparing potential outcomes (Y(1) vs. Y(0)).
- **Contextual Alternation**: Simulating hypothetical scenarios by modifying contexts.
- **`CausaloidGraph`**: Representing the flow of causal calculation.

**How to Run:**
From the root of the `deep_causality` project:

```bash
cargo run -p example-rcm
```

The EPP's separation of the causal model from its context enables the computational simulation of both potential
outcomes, directly addressing the RCM's fundamental challenge.

## EPP Example: Pearl's Ladder of Causation

**Location:** `examples/epp_scm`

This example demonstrates how `DeepCausality` models the three rungs of Judea Pearl's Ladder of Causation, each
representing a different level of causal reasoning.

**Key Concepts:**

- **Rung 1: Association (`rung1_association.rs`)**: Demonstrates observational inference (`P(Y|X)`) via `CausaloidGraph`
  evaluation.
- **Rung 2: Intervention (`rung2_intervention.rs`)**: Demonstrates taking an action (`P(Y|do(X))`) using the Causal
  State Machine (CSM).
- **Rung 3: Counterfactuals (`rung3_counterfactual.rs`)**: Demonstrates reasoning about alternate possibilities via
  Contextual Alternation.

**How to Run:**
From within the `examples/epp_scm` directory:

```bash
cargo run --bin example-scm
```

The EPP's architecture provides a robust framework for addressing each rung of Pearl's Ladder of Causation.

## Starter Example

**Location:** `examples/starter`

This is a basic example demonstrating fundamental `DeepCausality` concepts. It shows how to build a causal graph,
perform full and partial reasoning, and explain reasoning paths.

**Key Concepts:**

- **`CausaloidGraph`**: Building and freezing a causal graph.
- **`PropagatingEffect`**: Unified data and control-flow container.
- **Reasoning**: `evaluate_subgraph_from_cause`, `evaluate_shortest_path_between_causes`.
- **Explanation**: `explain_shortest_path_between_causes`.

**How to Run:**
From within the `examples/starter` directory:

```bash
cargo run --bin example-starter
```

This example serves as an excellent starting point for understanding the core functionalities of the `DeepCausality`
library.

## Tokio Integration Example

**Location:** `examples/tokio`

This example demonstrates how to integrate `DeepCausality` with the `tokio` asynchronous runtime for performing
background inference.

**Key Concepts:**

- **Asynchronous Processing**: Using `tokio` for concurrent operations.
- **`EventHandler`**: Managing inference tasks.
- **`BaseModel`**: Encapsulating the causal model.
- **Concurrency Primitives**: `Arc<RwLock>` for shared ownership.

**How to Run:**
From within the `examples/tokio` directory:

```bash
cargo run --bin example-tokio
```

This example illustrates how `DeepCausality` can be effectively used in an asynchronous environment for concurrent and
scalable causal inference.
