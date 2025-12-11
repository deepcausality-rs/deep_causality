# DeepCausality Examples Overview

This directory contains examples demonstrating various features and applications of the DeepCausality library. Each example showcases how to model and reason about causal relationships using the Effect Propagation Process (EPP) and PropagatingEffect monads.

## Example Categories

| Category | Description |
|----------|-------------|
| [Classical Causality](#classical-causality-examples) | Traditional causal inference methods (CATE, DBN, Granger, RCM, SCM) |
| [CSM Examples](#csm-examples) | Causal State Machine patterns |
| [Core Examples](#core-examples) | PropagatingEffect and PropagatingProcess fundamentals |
| [Physics Examples](#physics-examples) | Multi-physics simulations with Geometric Algebra |
| [Medicine Examples](#medicine-examples) | Biomedical and life sciences applications |
| [Starter Example](#starter-example) | Basic introduction to DeepCausality |
| [Tokio Example](#tokio-example) | Async integration with tokio runtime |

---

## Classical Causality Examples

**Location:** `examples/classical_causality_examples`

### CATE (Conditional Average Treatment Effect)

Models the average effect of a treatment on a specific subgroup using contextual alternation.

```bash
cargo run -p classical_causality_examples --example cate_example
```

### DBN (Dynamic Bayesian Network)

Models temporal causal processes with the "Umbrella World" scenario.

```bash
cargo run -p classical_causality_examples --example dbn_example
```

### Granger Causality

Tests whether past changes in one variable predict future changes in another.

```bash
cargo run -p classical_causality_examples --example granger_example
```

### RCM (Rubin Causal Model)

Demonstrates potential outcomes and Individual Treatment Effect calculation.

```bash
cargo run -p classical_causality_examples --example rcm_example
```

### SCM (Pearl's Ladder of Causation)

Models all three rungs of Pearl's Ladder: Association, Intervention, and Counterfactuals.

```bash
cargo run -p classical_causality_examples --example scm_example
```

---

## CSM Examples

**Location:** `examples/csm_examples`

### CSM Basic

Simple industrial monitoring system with sensors and actions.

```bash
cargo run -p csm_examples --example csm_example
```

### CSM with Context

Contextual causaloids with shared mutable state via `Arc<RwLock<BaseContext>>`.

```bash
cargo run -p csm_examples --example csm_context_example
```

### CSM with Effect Ethos

Integrates deontic reasoning with CSM for normative action evaluation.

```bash
cargo run -p csm_examples --example csm_effect_ethos_example
```

---

## Core Examples

**Location:** `examples/core_examples`

Fundamental examples demonstrating the monadic API.

### PropagatingEffect Examples

Basic monadic composition with value, error, and log propagation.

```bash
cargo run -p core_examples --example propagating_effect_example
cargo run -p core_examples --example propagating_effect_counterfactual_example
```

### PropagatingProcess Examples

Stateful monadic composition with state and context.

```bash
cargo run -p core_examples --example propagating_process_example
cargo run -p core_examples --example propagating_process_counterfactual
```

### Control Flow Examples

Builder patterns and strict ZST control flow.

```bash
cargo run -p core_examples --example control_flow_builder
cargo run -p core_examples --example control_flow_strict_zst
```

---

## Physics Examples

**Location:** `examples/physics_examples`

Multi-physics simulations using Geometric Algebra, Tensor operations, and Topology.

| Example | Domain | Command |
|---------|--------|---------|
| Maxwell's Unification | Electromagnetism | `cargo run -p physics_examples --example maxwell_example` |
| GRMHD | Relativity | `cargo run -p physics_examples --example grmhd_example` |
| Geometric Tilt | Robotics/IMU | `cargo run -p physics_examples --example geometric_tilt_example` |
| Algebraic Scanner | Abstract Algebra | `cargo run -p physics_examples --example algebraic_scanner` |
| Multi-Physics Pipeline | Particle Physics | `cargo run -p physics_examples --example multi_physics_pipeline` |
| Quantum Counterfactual | Quantum | `cargo run -p physics_examples --example quantum_counterfactual` |
| IKKT Matrix Model | Quantum Gravity | `cargo run -p physics_examples --example ikkt_matrix_model` |
| Gravitational Wave | Relativity | `cargo run -p physics_examples --example gravitational_wave` |

See [physics_examples/README.md](physics_examples/README.md) for detailed documentation.

---

## Medicine Examples

**Location:** `examples/medicine_examples`

Biomedical and life sciences applications using causal monads.

| Example | Domain | Command |
|---------|--------|---------|
| Protein Folding | Biophysics | `cargo run -p medicine_examples --example protein_folding` |
| MRI Tissue Classification | Medical Imaging | `cargo run -p medicine_examples --example mri_tissue_classification` |

See [medicine_examples/README.md](medicine_examples/README.md) for detailed documentation.

---

## Starter Example

**Location:** `examples/starter_example`

Basic introduction demonstrating:
- Building and freezing a `CausaloidGraph`
- Using `PropagatingEffect` for data/control flow
- Graph evaluation and path explanation

```bash
cargo run -p starter_example --example starter_example
```

---

## Tokio Example

**Location:** `examples/tokio_example`

Demonstrates integration with the tokio async runtime:
- Asynchronous causal inference
- `EventHandler` pattern
- `Arc<RwLock>` for shared state

```bash
cargo run -p tokio_example --example tokio_example
```

---

## 📜 License

All examples are licensed under the [MIT license](LICENSE).