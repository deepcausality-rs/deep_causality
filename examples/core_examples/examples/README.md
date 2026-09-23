# Core Examples: PropagatingEffect and PropagatingProcess

These examples demonstrate the core monadic types of `deep_causality_core`:
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

Counterfactual reasoning with `PropagatingEffect`.

```bash
cargo run -p core_examples --example propagating_effect_counterfactual_example
```

### 3. Propagating Process

Basic usage of `PropagatingProcess` with state management.

```bash
cargo run -p core_examples --example propagating_process_example
```

### 4. Propagating Process with Counterfactuals

Counterfactual reasoning with `PropagatingProcess`.

```bash
cargo run -p core_examples --example propagating_process_counterfactual
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
- Error propagation via a `Result<_, CausalityError>` outcome
- Logging to an `EffectLog`

### Monadic Composition

```rust
let result = PropagatingEffect::pure(initial_value)
    .bind(|v, _, _| step_one(v))
    .bind(|v, _, _| step_two(v))
    .bind(|v, _, _| step_three(v));
```

