# Causal State Machine (CSM) Examples

This directory contains examples demonstrating the **Causal State Machine** pattern. CSMs are systems where state transitions are governed not just by events, but by causal reasoning (Cause -> Effect -> New State) and potentially ethical/normative rules.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p csm_examples --example <example_name>
```

---

## Examples Overview

| Example | Pattern | Description |
|---------|---------|-------------|
| [csm_basic](csm_basic/README.md) | **Basic CSM** | A simple monitoring system (Sensor -> Action) demonstrating the fundamental State-Causaloid-Action loop. |
| [csm_context](csm_context/README.md) | **Contextual CSM** | Demonstrates sharing mutable data (`BaseContext`) across the causal graph using `Arc<RwLock>`, allowing complex state aggregations. |
| [csm_effect_ethos](csm_effect_ethos/README.md) | **Ethical CSM** | Integrates **Deontic Logic** (Obligation, Permission, Prohibition) into the state machine, allowing the system to evaluate the "moral permissibility" of an action before execution. |

---

## Common Patterns

### 1. The Causaloid
The core unit of a CSM is the `Causaloid`. Unlike a simple state transition function, a Causaloid encapsulates:
- **Causal Function**: The logic $f(data) \to bool$.
- **Description**: Human-readable explanation of *why* this causal link exists.
- **Weights**: For probabilistic reasoning.

### 2. Contextual State
In complex systems, decisions often depend on a global context (e.g., total system power, user permissions) rather than just local inputs. The `csm_context` example shows how to thread this context safely through the graph.

### 3. Normative Reasoning
The `csm_effect_ethos` example demonstrates a "Super-Ego" layer for AI agents. Even if an action is *causally* possible, the system checks if it is *ethically* permissible (e.g., "Do not delete root files").

---

## Run Commands

| Example | Command |
|---------|---------|
| Basic CSM | `cargo run -p csm_examples --example csm_example` |
| Contextual CSM | `cargo run -p csm_examples --example csm_context_example` |
| Ethical CSM | `cargo run -p csm_examples --example csm_effect_ethos_example` |
