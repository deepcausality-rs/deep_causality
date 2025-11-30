[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality Core

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue
[crates-url]: https://crates.io/crates/deep_causality_core
[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue
[docs-url]: https://docs.rs/deep_causality_core/latest/deep_causality_core/
[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg
[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE
[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

**Core types and abstractions for the [DeepCausality project](http://www.deepcausality.com).**

This crate provides the foundational building blocks for causal reasoning, effect propagation, and mission-critical control flow construction. It is designed for high-assurance systems, supporting `no_std`, zero-allocation execution, and formal certification requirements.

## Core Capabilities

### 1. Causal Effect Systems
The crate defines the core types for modeling causal systems using **Monadic Effect Systems**.
*   **`CausalEffectSystem`**: The central runtime for managing causal models.
*   **`CausalMonad`**: A monadic interface for chaining causal effects, allowing for composable and testable logic.
*   **`PropagatingEffect` / `PropagatingProcess`**: Types that model how effects ripple through a system, integrated with Higher-Kinded Types (HKT) via `deep_causality_haft`.

### 2. Mission-Critical Control Flow Builder
The **Control Flow Builder** is a specialized tool for constructing **Correct-by-Construction** execution graphs. It is designed for safety-critical applications where runtime wiring errors are unacceptable.

*   **Type-Safe Topology**: The builder uses Rust's type system to enforce that nodes can only be connected if their input/output protocols match. Invalid connections are rejected at **compile time**.
*   **Zero-Fault Execution**: The execution engine is designed to be panic-free and allocation-free (in the hot loop). Errors are returned as static enums, not Strings.
*   **Zero-Allocation Loop**: The `execute` method accepts a pre-allocated queue, ensuring deterministic execution timing suitable for real-time control loops.

## Feature Flags & Safety Implications

This crate is designed to scale from research prototypes to certified flight control systems. The feature flags control the trade-off between flexibility and strict safety guarantees.

| Feature | Default | Description | Safety & Regulatory Implication |
| :--- | :--- | :--- | :--- |
| **`std`** | Yes | Enables standard library support. | **General Purpose**. Suitable for servers, desktops, and research. Not for bare-metal embedded. |
| **`alloc`** | Yes | Enables heap allocation (`Vec`, `Box`). | **Embedded Linux / RTOS**. Required for dynamic graph construction. Most embedded systems use this. |
| **`strict-zst`** | **No** | **Certification Mode**. Enforces Zero-Sized Types. | **Safety-Critical / Hard Real-Time**. See below. |

### The `strict-zst` Feature (Certification Mode)

When `strict-zst` is enabled, the `ControlFlowBuilder` enforces that all user-provided logic must be **Zero-Sized Types (ZSTs)** (i.e., static function items).

*   **Implication**: You cannot use closures that capture environment variables. You must use plain functions.
*   **Benefit**:
    *   **No Hidden State**: The logic is guaranteed to be pure and stateless (or explicitly state-passing).
    *   **No Vtables**: Eliminates dynamic dispatch (`dyn Fn`), simplifying WCET (Worst-Case Execution Time) analysis.
    *   **Formal Verification**: The resulting graph topology is static and easier to model in formal verification tools.
    *   **Zero-Allocation Nodes**: Nodes are stored as function pointers, requiring no heap allocation for the logic itself.

**Recommendation**:
*   Use **default features** for prototyping and general applications.
*   Enable **`strict-zst`** for final deployment in safety-critical environments where dynamic allocation and hidden state are impermissible.

## Usage Examples

### Control Flow Builder (Mission Critical)

```rust
use deep_causality_core::{ControlFlowBuilder, CausalProtocol, FromProtocol, ToProtocol};
use std::collections::VecDeque;

// 1. Define your Domain Protocol
#[derive(Clone, Debug)]
enum MyProtocol {
    Signal(bool),
    Command(u8),
}
// ... impl CausalProtocol, FromProtocol, ToProtocol ...

fn sensor_read(input: bool) -> u8 {
    if input { 100 } else { 0 }
}

fn main() {
    let mut builder = ControlFlowBuilder::<MyProtocol>::default();

    // 2. Add Nodes (Type-checked!)
    let n_sensor = builder.add_node(sensor_read);
    
    // 3. Build Graph
    let graph = builder.build();

    // 4. Execute (Zero-Allocation Loop)
    let mut queue = VecDeque::with_capacity(10); // Pre-allocate memory
    let input = true.to_protocol();
    
    let result = graph.execute(input, 0, 10, &mut queue);
    println!("Result: {:?}", result);
}
```

### PropagatingEffect (Stateless)

`PropagatingEffect` is a monadic container for stateless causal effects. It supports standard functional transformations (`map`, `bind`) via the `Functor` and `Monad` traits.

```rust
use deep_causality_core::{PropagatingEffect, PropagatingEffectWitness};
use deep_causality_haft::{Functor, Applicative};

fn main() {
    // Create a pure effect
    let effect = PropagatingEffectWitness::pure(10);
    
    // Transform value (Functor)
    let mapped = PropagatingEffectWitness::fmap(effect, |x| x * 2);
    
    println!("Result: {:?}", mapped.value); // Value(20)
}
```

### PropagatingProcess (Stateful)

`PropagatingProcess` extends `PropagatingEffect` with **State** and **Context**. It allows you to model Markovian processes where each step can read/write state and access configuration context.

```rust
use deep_causality_core::{PropagatingProcess, PropagatingEffectWitness, EffectValue};
use deep_causality_haft::Applicative;

#[derive(Clone, Default, Debug)]
struct State { count: i32 }

fn main() {
    // Lift a pure effect into a stateful process
    let effect = PropagatingEffectWitness::pure(10);
    let process = PropagatingProcess::with_state(effect, State::default(), None);

    // Chain stateful computation
    let next = process.bind(|val, mut state, ctx| {
        state.count += 1;
        deep_causality_core::CausalEffectPropagationProcess {
            value: EffectValue::Value(val.into_value().unwrap() + 1),
            state,
            context: ctx,
            error: None,
            logs: Default::default(),
        }
    });

    println!("State: {:?}", next.state); // State { count: 1 }
}
```

## non-std Support

To use this crate in a bare-metal `no_std` environment:

```toml
[dependencies]
deep_causality_core = { version = "...", default-features = false, features = ["alloc"] }
```

If you need absolute strictness (no `Box<dyn Fn>`):

```toml
[dependencies]
deep_causality_core = { version = "...", default-features = false, features = ["alloc", "strict-zst"] }
```

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

*   [Marvin Hansen](https://github.com/marvin-hansen).
*   Github GPG key ID: 369D5A0B210D39BC
*   GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
