[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![CodeFactor][codefactor-badge]][codefactor-url]
![Tests][test-url]
[![OpenSSF Best Practices][ossf-badge]][ossf-url]
[![codecov][codecov-badge]][codecov-url]
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fdeepcausality-rs%2Fdeep_causality.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2Fdeepcausality-rs%2Fdeep_causality?ref=badge_shield&issueType=license)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fdeepcausality-rs%2Fdeep_causality.svg?type=shield&issueType=security)](https://app.fossa.com/projects/git%2Bgithub.com%2Fdeepcausality-rs%2Fdeep_causality?ref=badge_shield&issueType=security)


[codefactor-badge]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causality/badge

[codefactor-url]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causalityl

[codecov-badge]: https://codecov.io/gh/deepcausality-rs/deep_causality/branch/main/graph/badge.svg?token=W9TA1VVJ7O

[codecov-url]: https://codecov.io/gh/deepcausality-rs/deep_causality

[ossf-badge]: https://bestpractices.coreinfrastructure.org/projects/7568/badge

[ossf-url]:https://bestpractices.coreinfrastructure.org/projects/7568

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality/latest/deep_causality/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg


---

<div align="center">

[<img src="img/logo_color.png">](https://deepcausality.com)

</div>

---

<div style="display: flex; flex-wrap: wrap; justify-content: center; align-items: center; text-align: center;">

[Website](https://deepcausality.com) | [Getting started](https://deepcausality.com/getting-started/) | [Documentation](https://deepcausality.com/docs/intro/) | [Blog](https://deepcausality.com/blog/) | [Discord](https://discord.gg/Bxj9P7JXSj) | [Crates](https://crates.io/crates/deep_causality)

</div>

# DeepCausality

**A hypergeometric computational causality library for building systems that reason about cause and effect.**

DeepCausality pioneers uniform reasoning across deterministic and probabilistic modalities, supporting both static and dynamic contextual causal models. The project is hosted as a sandbox project in the [Linux Foundation for Data & AI](https://landscape.lfai.foundation/).

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| **Effect Propagation Monads** | `PropagatingEffect` and `PropagatingProcess` for composable causal computations |
| **Geometric Algebra** | Universal Clifford Algebra via `CausalMultiVector` for physics and quantum mechanics |
| **Differential Topology** | `Manifold`, `SimplicialComplex`, and discrete exterior calculus |
| **Tensor Operations** | `CausalTensor` with Einstein summation (`ein_sum`) and linear algebra |
| **Causaloid Graphs** | Recursive, composable causal structures with graph-based reasoning |
| **Effect Ethos** | Deontic guardrails using defeasible calculus for safe AI actions |
| **Causal Discovery** | SURD and MRMR algorithms for learning causal structure from data |
| **Multi-Physics** | Quantum mechanics, thermodynamics, electromagnetism, fluids, relativity |

---

## 🏗️ Architecture

DeepCausality's architecture is built on a unified foundation: **causality as monadic dependency** (`E₂ = f(E₁)`). From this axiom, the framework derives its core abstractions.

### The Causal Monad & PropagatingEffect

At the heart of DeepCausality is the **Causal Monad** pattern, implemented through two primary types:

| Type | Purpose | State |
|------|---------|-------|
| `PropagatingEffect<T>` | Stateless effect propagation | Value + Error + Log |
| `PropagatingProcess<T>` | Stateful effect propagation | Value + State + Context + Error + Log |

These monads enable **composable causal computations** where effects flow through a pipeline of transformations wth the following key properties:
- **Error propagation**: Errors short-circuit the chain automatically
- **Logging**: Each step can append to an audit trail
- **Counterfactuals**: `bind` supports hypothetical "what-if" reasoning

### The Three Pillars

#### 1. The Causaloid
A self-contained unit of causality that holds a causal function, receives an incoming effect, and emits a new outgoing effect. Causaloids compose into graphs for complex reasoning.

#### 2. The Context
An explicit hypergraph data structure that holds all factual data about the operational environment: sensor readings, temporal structures, spatial locations, and more.

#### 3. The Effect Ethos
A programmable deontic layer that verifies whether proposed actions align with safety and mission objectives before execution.

---

## 🚀 Getting Started

Add DeepCausality to your project:

```bash
cargo add deep_causality_core
```

### Counterfactual & Intervention Example

```rust
use deep_causality_core::{Intervenable, PropagatingEffect};

fn main() {
    // Causal chain: Dose → Absorption → Metabolism → Response
    let observed = PropagatingEffect::pure(10.0_f64)
        .bind(|dose, _, _| PropagatingEffect::pure(dose * 0.8))   // Absorption: 8.0
        .bind(|level, _, _| PropagatingEffect::pure(level - 2.0)) // Metabolism: 6.0
        .bind(|level, _, _| {
            let response = if level > 5.0 { "Effective" } else { "Ineffective" };
            PropagatingEffect::pure(response)
        });
    // Result: "Effective"

    // Intervention: Replace value MID-CHAIN with do(BloodLevel := 3.0)
    let intervened = PropagatingEffect::pure(10.0_f64)
        .bind(|dose, _, _| PropagatingEffect::pure(dose * 0.8))   // Absorption: 8.0
        .intervene(3.0)  // ← Force BloodLevel to 3.0, preserving log
        .bind(|level, _, _| PropagatingEffect::pure(level - 2.0)) // Metabolism: 1.0
        .bind(|level, _, _| {
            let response = if level > 5.0 { "Effective" } else { "Ineffective" };
            PropagatingEffect::pure(response)
        });
    // Result: "Ineffective" — intervention changed the outcome

    println!("Observed:   {:?}", observed.value());   // "Effective"
    println!("Intervened: {:?}", intervened.value()); // "Ineffective"
}
```

This demonstrates **Pearl's Ladder of Causation**:
1. **Association** (Rung 1): Observing dose=10 correlates with "Effective"
2. **Intervention** (Rung 2): `intervene(3.0)` forces a value mid-chain
3. **Counterfactual** (Rung 3): Same chain, different outcome due to intervention

---

## 📂 Examples

Run examples with:

```bash
# Classical Causality (CATE, DBN, Granger, SCM)
cargo run -p classical_causality_examples --example scm_example

# Medicine & Life Sciences
cargo run -p medicine_examples --example protein_folding
cargo run -p medicine_examples --example mri_tissue_classification

# Physics (Quantum, Electromagnetism, Relativity)
cargo run -p physics_examples --example maxwell_example
cargo run -p physics_examples --example geometric_tilt_example
cargo run -p physics_examples --example quantum_counterfactual
cargo run -p physics_examples --example gravitational_wave

# Causal State Machine
cargo run -p csm_examples --example csm_effect_ethos_example
```

For more examples, See [examples/README.md](examples/README.md)

---

## 📦 Crate Ecosystem

### Core Framework
| Crate | Description |
|-------|-------------|
| [`deep_causality`](deep_causality/README.md) | Main library with Causaloid, Context, CSM, and Model types |
| [`deep_causality_core`](deep_causality_core/README.md) | `PropagatingEffect`, `PropagatingProcess`, and `CausalEffectSystem` |
| [`deep_causality_ethos`](deep_causality_ethos/README.md) | Deontic reasoning with `EffectEthos` and `Teloid` |

### Mathematics & Physics
| Crate | Description |
|-------|-------------|
| [`deep_causality_multivector`](deep_causality_multivector/README.md) | Clifford Algebra with `CausalMultiVector` and `HilbertState` |
| [`deep_causality_tensor`](deep_causality_tensor/README.md) | N-dimensional tensors with Einstein summation |
| [`deep_causality_topology`](deep_causality_topology/README.md) | `Manifold`, `SimplicialComplex`, `Graph`, `Hypergraph`, `PointCloud` |
| [`deep_causality_physics`](deep_causality_physics/README.md) | Quantum, thermodynamics, electromagnetism, fluids, relativity kernels |
| [`deep_causality_sparse`](deep_causality_sparse/README.md) | Compressed sparse row matrices |
| [`deep_causality_num`](deep_causality_num/README.md) | Complex numbers, division algebras, numeric traits |

### Data & Algorithms
| Crate | Description |
|-------|-------------|
| [`deep_causality_discovery`](deep_causality_discovery/README.md) | Causal Discovery Language (CDL) with SURD and MRMR |
| [`deep_causality_algorithms`](deep_causality_algorithms/README.md) | Feature selection and causal discovery algorithms |
| [`deep_causality_data_structures`](deep_causality_data_structures/README.md) | Specialized data structures |
| [`ultragraph`](ultragraph/README.md) | High-performance hypergraph backend |

### Utilities
| Crate | Description |
|-------|-------------|
| [`deep_causality_haft`](deep_causality_haft/README.md) | Higher-kinded types and abstract functional types |
| [`deep_causality_uncertain`](deep_causality_uncertain/README.md) | `Uncertain<T>` and `MaybeUncertain<T>` for uncertainty propagation |
| [`deep_causality_rand`](deep_causality_rand/README.md) | Random number generation and statistical distributions |
| [`deep_causality_macros`](deep_causality_macros/README.md) | Procedural macros |
| [`deep_causality_ast`](deep_causality_ast/README.md) | Generic abstract syntax tree |

---



## 🛠️ Build & Test

```bash
# Optimized build with SIMD
RUSTFLAGS='-C target-cpu=native' cargo build --release

# Run all tests
cargo test --all

# Run benchmarks
cargo bench
```

### Using Make

```bash
make install   # Install dependencies
make build     # Build incrementally
make test      # Run all tests
make example   # Run examples
make check     # Security audit
```

### Using Bazel

The repository also supports Bazel builds. Install [bazelisk](https://github.com/bazelbuild/bazelisk) and run:

```bash
bazel build //...
bazel test //...
```

---

## 📚 Documentation

| Resource | Link |
|----------|------|
| API Reference | [docs.rs/deep_causality](https://docs.rs/deep_causality/latest/deep_causality/) |
| Core Concepts | [docs/CORE.md](docs/CORE.md) |
| Introduction | [docs/INTRO.md](docs/INTRO.md) |
| Deep Dive | [docs/DEEP_DIVE.md](docs/DEEP_DIVE.md) |
| Architecture | [deepcausality.com/docs/architecture](https://deepcausality.com/docs/architecture/) |
| Concepts | [deepcausality.com/docs/concepts](https://deepcausality.com/docs/concepts/) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

---

## 👨‍💻 Contributing

Contributions are welcome! Please read:

* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Project Charter](DeepCausalityProjectCharter.pdf)

```bash
# Before submitting a PR
make test
make check
```

---

## 🙏 Credits

Inspired by research from:
* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html) - Causal inference
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy) - Causaloid framework
* [Elias Bareinboim](https://causalai.net/) - Causal AI
* [Microsoft Research](https://www.microsoft.com/en-us/research/group/causal-inference/) - Causality and ML

Implements research from:
* ["An Algebraic Roadmap of Particle Theories"](docs/papers/algebraic_physics.pdf) 
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](docs/papers/ddic.pdf)
* ["NWHy: A Framework for Hypergraph Analytics"](docs/papers/nwhy.pdf)
* ["Observational causality by states and interaction type for scientific discovery"](docs/papers/surd-state.pdf)
* ["Probability Theories with Dynamic Causal Structure"](docs/papers/causaloid.pdf)
* ["Uncertain T: A First-Order Type for Uncertain Data"](docs/papers/uncertain_t.pdf)

---

## 🌐 Community

* [Discord](https://discord.gg/Bxj9P7JXSj)
* [GitHub Discussions](https://github.com/orgs/deepcausality-rs/discussions)
* [LF Email Lists](https://deepcausality.com/community/)

---

## 📜 License

This project is licensed under the [MIT license](LICENSE).

## 👮 Security

See [SECURITY.md](SECURITY.md) for security policies.

---

## 🎁 Sponsors

[![JetBrains logo.](https://resources.jetbrains.com/storage/products/company/brand/logos/jetbrains.svg)](https://jb.gg/OpenSource)

[JetBrains](https://www.jetbrains.com/) provides the project with an all-product license.
