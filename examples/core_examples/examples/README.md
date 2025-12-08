# Core Examples: PropagatingEffect and PropagatingProcess

This directory contains examples demonstrating the core monadic types from `deep_causality_core`:
- **PropagatingEffect**: A pure monadic effect for value propagation
- **PropagatingProcess**: A stateful monadic process with state and context

---

## Examples

### 1. Propagating Effect

Basic usage of `PropagatingEffect` for monadic value propagation.

```bash
cargo run -p core_examples --example propagating_effect_example
```

### 2. Propagating Effect with Counterfactuals

Demonstrates counterfactual reasoning using `PropagatingEffect`.

```bash
cargo run -p core_examples --example propagating_effect_counterfactual_example
```

### 3. Propagating Process

Basic usage of `PropagatingProcess` with state management.

```bash
cargo run -p core_examples --example propagating_process_example
```

### 4. Propagating Process with Counterfactuals

Demonstrates counterfactual reasoning using `PropagatingProcess`.

```bash
cargo run -p core_examples --example propagating_process_counterfactual
```

### 5. Control Flow Builder

Shows the control flow builder pattern for monadic composition.

```bash
cargo run -p core_examples --example control_flow_builder
```

### 6. Control Flow with Strict ZST

Advanced example using Zero-Sized Types for strict control flow.

```bash
cargo run -p core_examples --example control_flow_strict_zst --features strict-zst
```

---

## Key Concepts

### PropagatingEffect vs PropagatingProcess

| Type | Best For | Has State | Has Context |
|------|----------|-----------|-------------|
| `PropagatingEffect<T>` | Pure value propagation | No | No |
| `PropagatingProcess<O, S, C>` | Stateful operations | Yes (`S`) | Yes (`C`) |

Both types support:
- Monadic `bind` for chaining operations
- Error propagation via `Option<CausalityError>`
- Logging via the logs field

### Monadic Composition

```rust
let result = PropagatingEffect::pure(initial_value)
    .bind(|v, _, _| step_one(v))
    .bind(|v, _, _| step_two(v))
    .bind(|v, _, _| step_three(v));
```

