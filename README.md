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

[//]: # ([![Miri][miri-badge]][miri-url])

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

[miri-badge]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust_miri.yml/badge.svg

[miri-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust_miri.yml

---

<div align="center">

[<img src="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/logo_background.jpg" width="900">](https://deepcausality.com)

</div>

---

<div style="display: flex; flex-wrap: wrap; justify-content: center; align-items: center; text-align: center;">

[Website](https://deepcausality.com) | [Getting started](https://www.deepcausality.com/docs/getting-started/install/) | [Blog](https://deepcausality.com/blog/) | [Discord](https://discord.gg/Bxj9P7JXSj) | [Crates](https://crates.io/crates/deep_causality)

</div>

# DeepCausality: Dynamic Causality

DeepCausality is the reference implementation of the **Effect Propagation Process (EPP)**, a single axiomatic foundation
for dynamic causality based on Whitehead's process metaphysics, with the consequence that the resulting framework is
general-relativistic-native and quantum-native. Classical computational causality frameworks (Pearl's SCM, Granger
causality, DBNs) assume fixed background spacetime and static causal structure and thus cannot handle dynamic causal
structures; DeepCausality contributes **dynamic, adaptive, and emergent** causality as first-class modalities, with a
programmable deontic layer for verifiable safety. DeepCausality is hosted as a sandbox project at
the [Linux Foundation for Data & AI](https://landscape.lfai.foundation/).

## Support

Dynamic causality can be daunting at first, and if you need more support for a larger or commercial project, please feel free to reach out to the [Center of Dynamic Causality](https://www.causalcenter.com/contact/) that backs the Deep Causality project.

## Getting Started

For LLM-assisted project-building guidance, see [SKILLS.md](./SKILLS.md).

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

This walks **Pearl's Ladder of Causation**:

1. **Association** (Rung 1): `dose=10` correlates with "Effective".
2. **Intervention** (Rung 2): `intervene(3.0)` forces a value mid-chain.
3. **Counterfactual** (Rung 3): Same chain, different outcome under the intervention.

DeepCausality can express
all [major frameworks of classical computation causality](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples).

---

## Examples

```bash
# Regime change: causal structure evolves as the system crosses a physical threshold
cargo run -p physics_examples --example event_horizon_probe

# Compositional pipeline: Causaloid evaluations interleaved with CausalMonad bind
cargo run -p avionics_examples --example flight_envelope_monitor
```

See [examples/README.md](examples/README.md) for the full catalogue of available examples.

---

## What is Unique

|                                 |                                                                                                                                                                                                                               |
|---------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **One axiom, three primitives** | Causaloid, Context, and Causal State Machine derived from a single functional-dependency axiom                                                                                                                                |
| **Three causal modalities**     | Dynamic, adaptive, and emergent causality, going beyond the static-structure assumption                                                                                                                                       |
| **Effect Propagation Monads**   | `PropagatingEffect` and `PropagatingProcess` for composable  causal pipelines                                                                                                                                                 |
| **Effect Ethos**                | Defeasible deontic calculus (after Forbus) that verifies actions against an immutable ethos before execution                                                                                                                  |
| **Uniform mathematics**         | Tensors, MultiVectors, Manifolds, and `PropagatingEffect` share one categorical interface (Functor / Monad / Comonad) via arity-5 HKT in stable Rust; multi-physics pipelines compose across domains in a single monadic flow |
| **Geometric Algebra**           | Clifford algebras (Pauli, spacetime, conformal, projective, Dixon, Spin(10) GUA) with shared metric conventions                                                                                                               |
| **Differential Topology**       | Manifolds, simplicial complexes, lattice gauge theory verified against 24 reference results from Creutz                                                                                                                       |
| **Float106 precision**          | 106-bit float (~32 decimal digits) on stable Rust, several × faster than IEEE binary128                                                                                                                                       |
| **Causal Discovery**            | SURD and MRMR algorithms wrapped in a typestate DSL that closes the loop from data to model                                                                                                                                   |

---

## Architecture

The EPP rests on a single axiom: **`m₂ = m₁ >>= f`**. Effect propagation becomes a monadic dependency, with no
assumption of any background spacetime. Three computable primitives operationalize the axiom, and an optional fourth
provides the safety layer for emergent behaviour.

### The Three Primitives

#### 1. Causaloid and CausalMonad

The monadic axiom admits two isomorphic structural expressions of the same causal computation. Both are first-class
causal entities; neither is more fundamental than the other.

- **Causaloid.** A polymorphic container for the causal function `f` (after Hardy). It carries causal *structure* and is
  isomorphic across three forms (**Singleton**, **Collection**, **Graph**), which lets recursive causal structures be
  composed without changing the calling code.
- **CausalMonad.** The bind side of the axiom, carrying causal *sequencing* through Kleisli composition. `bind`
  short-circuits on error, accumulates the audit log, and supports counterfactual `intervene` operations.

Both inhabit the same propagating-effect carrier:

| Type                    | Purpose                      | Channels                              |
|-------------------------|------------------------------|---------------------------------------|
| `PropagatingEffect<T>`  | Stateless effect propagation | Value · Error · Log                   |
| `PropagatingProcess<T>` | Stateful effect propagation  | Value · State · Context · Error · Log |

Because both consume and produce the same carrier, they compose freely. A Causaloid evaluation can feed a `.bind()`
step. A `.bind()` step can feed a Causaloid evaluation. State and audit log accumulate across both. One pipeline can
mix structural and sequential reasoning, picking the right shape at each stage:

* **Sequential transforms** belong in a CausalMonad bind-chain.
* **Parallel aggregation** belongs in a Causaloid collection.
* **Cross-influencing dependencies** belong in a Causaloid graph.

The [flight envelope monitor](examples/avionics_examples/control/flight_envelope_monitor/) shows all three: a Causaloid
collection over five sensor-health checks, a three-step CausalMonad bind-chain for state estimation, and a Causaloid
hypergraph of six envelope protections, all running through one `PropagatingProcess<T, FlightState, AircraftConfig>`
with state and audit log threaded across every stage.

#### 2. Context

An explicit hypergraph carrying the operational environment: sensor data, temporal structures (linear and non-linear),
spatial locations (Euclidean and non-Euclidean). Detaching causality from a fixed background spacetime requires the
Context to be queryable and dynamic.

#### 3. Causal State Machine (CSM)

The bridge from causal inference to action. The CSM separates state from action so that a proposed action can be
verified before execution.

### The Safety Layer

#### Effect Ethos

An optional, programmable deontic layer that uses a **defeasible deontic calculus** to resolve normative conflicts and
decide whether a CSM-proposed action is permissible under an immutable ethos. Required wherever emergent causality is in
play, since static verifiability is no longer possible there.

### Uniform mathematics

Most scientific-computing stacks force you to bridge silos: one library for tensors, another for geometric algebra, a
third for topology, with glue code in between. The DeepCausality stack lifts every mathematical layer into the same
categorical interface through the `deep_causality_haft` crate's arity-5 higher-kinded types:

| Domain    | Type                   | Categorical role                |
|-----------|------------------------|---------------------------------|
| Mechanics | `CausalTensor<T>`      | Functor (map over field data)   |
| Algebra   | `CausalMultiVector<T>` | Monad (chain operations)        |
| Topology  | `Manifold<T>`          | Comonad (neighborhood analysis) |
| Causality | `PropagatingEffect<T>` | Monad (sequencing + logs)       |

A single `bind`-chain can therefore step from a Tensor (general relativity), through a MultiVector (geometric algebra),
onto a Manifold (topology), and finish in a `PropagatingEffect` (causal logic) without serialisation or adapter code.
The [GRMHD example](examples/physics_examples/grmhd/) does exactly this for relativistic magnetohydrodynamics: Einstein
tensor curvature feeds metric selection, which feeds a multivector Lorentz force, which feeds causal stability analysis,
all in one monadic chain. The [Maxwell example](examples/physics_examples/maxwell/) derives `E` and `B` as bivector
grades of a single electromagnetic field `F = ∇A`, which cuts the scalar count from six to four (~50% compute reduction)
and is directly applicable to 5G/6G phased-array antenna design.

---

## Crate Ecosystem

### Causal Discovery

| Crate                                                              | Description                                                                           |
|--------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| [`deep_causality_discovery`](deep_causality_discovery/README.md)   | Causal Discovery Language (typestate DSL: load → clean → select → discover → analyse) |
| [`deep_causality_algorithms`](deep_causality_algorithms/README.md) | SURD, MRMR, and feature-selection primitives                                          |

### Causal Framework

| Crate                                                            | Description                                                                                          |
|------------------------------------------------------------------|------------------------------------------------------------------------------------------------------|
| [`deep_causality`](deep_causality/README.md)                     | Causaloid (Singleton/Collection/Graph), Context, CSM, Teloid, Effect Ethos integration               |
| [`deep_causality_core`](deep_causality_core/README.md)           | `PropagatingEffect`, `PropagatingProcess`, `CausalMonad`, `CausalEffectSystem`, `ControlFlowBuilder` |
| [`deep_causality_ethos`](deep_causality_ethos/README.md)         | `EffectEthos` and `Teloid` for defeasible deontic reasoning                                          |
| [`deep_causality_uncertain`](deep_causality_uncertain/README.md) | `Uncertain<T>` and `MaybeUncertain<T>` (after Bornholt et al.)                                       |

### Physics

| Crate                                                        | Description                                                                                                                 |
|--------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|
| [`deep_causality_physics`](deep_causality_physics/README.md) | Astrophysics, condensed matter, EM, fluids, MHD, nuclear, photonics, QM, relativity, thermo, waves; generic over float type |

### Mathematics

| Crate                                                                | Description                                                                                                                          |
|----------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| [`deep_causality_tensor`](deep_causality_tensor/README.md)           | N-dim tensors, broadcasting, Einstein summation, Functor/Applicative/Monad/Comonad                                                   |
| [`deep_causality_multivector`](deep_causality_multivector/README.md) | Clifford algebras: Pauli, STA, CGA, PGA(3), Dixon, Spin(10) GUA                                                                      |
| [`deep_causality_topology`](deep_causality_topology/README.md)       | Graphs, hypergraphs, simplicial complexes, manifolds, point clouds, exterior calculus, U(1)/SU(2)/SU(3)/Lorentz lattice gauge fields |
| [`deep_causality_sparse`](deep_causality_sparse/README.md)           | CSR sparse matrices                                                                                                                  |

### Foundation

| Crate                                                      | Description                                                                                       |
|------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| [`deep_causality_num`](deep_causality_num/README.md)       | Magma → Field algebraic hierarchy, `Float106`, Complex/Quaternion/Octonion division algebras      |
| [`deep_causality_haft`](deep_causality_haft/README.md)     | Arity-5 higher-kinded types via witness pattern; Effect / Functor / Applicative / Monad / CoMonad |
| [`deep_causality_metric`](deep_causality_metric/README.md) | Single source of truth for metric signatures (East Coast, West Coast, Cl(p,q,r))                  |
| [`ultragraph`](ultragraph/README.md)                       | Two-phase hypergraph backend for CausaloidGraph and Context                                       |

### Utilities

| Crate                                                                        | Description                                                  |
|------------------------------------------------------------------------------|--------------------------------------------------------------|
| [`deep_causality_data_structures`](deep_causality_data_structures/README.md) | Sliding-window, grid-array, and other specialised structures |
| [`deep_causality_rand`](deep_causality_rand/README.md)                       | RNG and statistical distributions                            |
| [`deep_causality_ast`](deep_causality_ast/README.md)                         | Generic abstract syntax tree                                 |

---

## Build & Test

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
bazel test  //...
```

---

## Contributing

Contributions are welcome! Please read:

* [AI Coding Assistants](AiCodingAssistants.md)
* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)

```bash
# Before submitting a PR
make test
make check
```

---

## Acknowledgement

Inspired by research from:

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html): Structural Causal Models
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy): Causaloid framework
* [Elias Bareinboim](https://causalai.net/): Transportability and data fusion

Implemented research:

**deep_causality**
* ["Probability Theories with Dynamic Causal Structure"](deep_causality/papers/causaloid.pdf), Hardy

**deep_causality_algorithms**
* "[Root Cause Analysis of Failures in Microservices via Bayesian Root Cause Discovery](https://icml.cc/virtual/2026/poster/65359)"
* [Maximum Relevance and Minimum Redundancy Feature Selection](deep_causality_algorithms/papers/mrmr_feature_selector.pdf)
* ["Observational causality by states and interaction type for scientific discovery"](deep_causality_algorithms/papers/surd-state.pdf)

**deep_causality_ethos**
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](deep_causality_ethos/papers/ddic.pdf), Olson & Forbus

**deep_causality_multivector**
* ["An Algebraic Roadmap of Particle Theories"](deep_causality_multivector/papers/algebraic_physics.pdf)

**deep_causality_uncertain**
* ["Uncertain⟨T⟩: A First-Order Type for Uncertain Data"](deep_causality_uncertain/papers/uncertain_t.pdf), Bornholt et al.

**Ultragraph**
* ["NWHy: A Framework for Hypergraph Analytics"](ultragraph/papers/nwhy.pdf)

---

## Community

* [Discord](https://discord.gg/Bxj9P7JXSj)
* [GitHub Discussions](https://github.com/orgs/deepcausality-rs/discussions)
* [LF Email Lists](https://deepcausality.com/community/)

---

## License

This project is licensed under the [MIT license](LICENSE).

## 👮 Security

See [SECURITY.md](SECURITY.md) for security policies.

---

## Sponsors

[![JetBrains logo.](https://resources.jetbrains.com/storage/products/company/brand/logos/jetbrains.svg)](https://jb.gg/OpenSource)

[JetBrains](https://www.jetbrains.com/) provides the project with an all-product license.

<a href="https://www.causalcenter.com">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/causal_center_logo_dark.svg">
    <img src="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/causal_center_logo.svg" alt="Center for Dynamic Causality" width="140">
  </picture>
</a>

The [Center for Dynamic Causality](https://www.causalcenter.com) contributes ongoing research and resources to the DeepCausality project.


---

## Citation

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20195214.svg)](https://doi.org/10.5281/zenodo.20195214)

If you use DeepCausality in your research, please cite it using the metadata in [`CITATION.cff`](CITATION.cff), or
directly as follows.

**APA:**

> Hansen, M. (2026). *DeepCausality* [Computer software]. Zenodo. https://doi.org/10.5281/zenodo.20195214

**BibTeX:**

```bibtex
@software{hansen_deepcausality,
    author = {Hansen, Marvin},
    title = {DeepCausality: A Hypergeometric Computational Causality Library for Rust},
    publisher = {Zenodo},
    url = {https://github.com/deepcausality-rs/deep_causality},
    doi = {10.5281/zenodo.20195214},
    orcid = {0009-0000-1159-8173}
}
```

The DOI above represents all versions, and will always
resolve to the latest one. To cite a specific release, use the version-specific DOI listed on the
project's [Zenodo record](https://doi.org/10.5281/zenodo.20195214).
