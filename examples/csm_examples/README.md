# Causal State Machine (CSM) Examples

These examples demonstrate the **Causal State Machine** (CSM) pattern. In a CSM, causal reasoning (Cause -> Effect -> New State) decides which actions fire, and normative rules can constrain them.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p csm_examples --example <example_name>
```

---

## Examples Overview

| Example | Pattern | Description |
|---------|---------|-------------|
| [csm_basic](csm_basic/README.md) | **Basic CSM** | A monitoring system (Sensor -> Action) built on the State-Causaloid-Action loop. |
| [csm_context](csm_context/README.md) | **Contextual CSM** | Shares mutable data (`BaseContext`) with the causal model through `Arc<RwLock>` to fuse several sensor readings. |
| [csm_effect_ethos](csm_effect_ethos/README.md) | **Ethical CSM** | Pairs a CSM with an `EffectEthos` that applies **Deontic Logic** (Obligation, Permission, Prohibition) to decide whether the CSM's action is permissible. |

---

## Common Patterns

### 1. The Causaloid
The core unit of a CSM is the `Causaloid`. Beyond a state transition function, a Causaloid holds:
- **Causal Function**: The logic $f(data) \to bool$.
- **Description**: A human-readable explanation of *why* this causal link exists.

### 2. Contextual State
Decisions often depend on a global context (e.g., total system power, user permissions) as well as local inputs. The `csm_context` example shares such a context safely with the causal model.

### 3. Normative Reasoning
The `csm_effect_ethos` example adds a normative layer: for an action that is *causally* triggered, the system checks whether it is *ethically* permissible.

---

## Run Commands

| Example | Command |
|---------|---------|
| Basic CSM | `cargo run -p csm_examples --example csm_example` |
| Contextual CSM | `cargo run -p csm_examples --example csm_context_example` |
| Ethical CSM | `cargo run -p csm_examples --example csm_effect_ethos_example` |
