# Classical Causality Examples

This directory contains examples demonstrating **Traditional Causal Inference** methods implemented using the `deep_causality` framework. These examples bridge the gap between standard econometrics/statistics and the Causal State Machine approach.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p classical_causality_examples --example <example_name>
```

---

## Examples Overview

| Example | Method | Description |
|---------|--------|-------------|
| [cate](cate/README.md) | **CATE** (Conditional Average Treatment Effect) | Models how treatment effects vary across different subgroups (heterogeneity). |
| [dbn](dbn/README.md) | **DBN** (Dynamic Bayesian Network) | Models probabilistic dependencies across time steps (e.g., Umbrella World). |
| [granger](granger/README.md) | **Granger Causality** | predictive causality test for time series data. |
| [rcm](rcm/README.md) | **RCM** (Rubin Causal Model) | Potential Outcomes framework for estimating causal effects from observational data. |
| [scm](scm/README.md) | **SCM** (Structural Causal Model) | Pearl's "Ladder of Causation": Association, Intervention, and Counterfactuals. |

---

## Common Patterns

### 1. The Inverse Pattern
Unlike deep causal chains which propagate *forward* (Result -> Effect), classical methods often look *backward* or infer hidden parameters from data. These examples show how to wrap such statistical inferences into `PropagatingEffect` containers to make them composable with the rest of the system.

### 2. Contextual Data
Many of these examples (especially **CATE** and **RCM**) rely heavily on `Context` to store population data, covariats, or historical time series, demonstrating how `CausalEffectPropagationProcess` handles stateful context.

---

## Run Commands

| Example | Command |
|---------|---------|
| CATE | `cargo run -p classical_causality_examples --example cate_example` |
| DBN | `cargo run -p classical_causality_examples --example dbn_example` |
| Granger | `cargo run -p classical_causality_examples --example granger_example` |
| RCM | `cargo run -p classical_causality_examples --example rcm_example` |
| SCM | `cargo run -p classical_causality_examples --example scm_example` |
