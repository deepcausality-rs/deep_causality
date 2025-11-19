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

# Overview

DeepCausality is a hypergeometric computational causality library that enables fast, context-aware causal reasoning over
complex multi-stage causality models. DeepCausality pioneers uniform reasoning across deterministic and probabilistic
modalities across static and dynamic contextual causal models. DeepCausality comprises of three core pillars:
The Context is an explicit and dynamic hypergraph that models the operational environment, supporting non-Euclidean and
non-linear temporal structures. The Causaloid is a polymorphic, composable unit that can encapsulate a causal function.
The Effect Ethos is a deontic guardrail that uses a defeasible calculus to ensure a system's actions remain verifiably
aligned with its safety and mission objectives. The DeepCausality project is hosted as a sandbox project
in the [Linux Foundation for Data & Ai](https://landscape.lfai.foundation/).

### The Three Pillars of DeepCausality

DeepCausality comprises of three main components:

#### 1. The Causaloid:

* **What it is:** A self-contained, single unit of causality.
* **What it does:** It holds a single causal function that receives an incoming effect, runs its causal
  function, and emits a new, outgoing effect.

#### 2. The Context:

* **What it is:** The explicit environment where the Causaloids operate. It holds all the factual data.
* **What it does:** The Context is a super-flexible data structure (a hypergraph) that holds all the facts about the
  world: the current time, sensor readings, locations on a map, etc.

#### 3. The Effect Ethos

* **What it is:** A programmable ethos, to encode and verify operational rules.
* **What it does:** A Causaloid might reason, "Based on the data, the most logical action is X." But before action X can
  be taken, the Effect Ethos steps in and checks against a set of rules. It answers the question: "Should this happen?"

DeepCausality is a framework for building systems that can reason about cause and effect in complex, dynamic
environments. It achieves this by treating causality as a process of **effect propagation** between simple, composable *
*Causaloids** that operate on an explicit, flexible **Context**, all governed by a verifiable safety layer called the *
*Effect Ethos**. For an introduction to DeepCausality, see the [introduction document](README_INTRO.md) and
the more detailed [deep dive.](README_DEEP_DIVE.md)

## The Foundation of DeepCausality

DeepCausality implements the Effect Propagation Process (EPP), which uses a single axiomatic foundation of causality as
a functional dependency (`E2 = f(E1)`). From this single axiom, it derives the context to support Euclidean,
non-Euclidean and non-linear, structures, the causaloid for handling static and dynamic causal structures, and the
effect ethos. The full text of the EPP monograph is accessible via the the [publication repository](https://github.com/deepcausality-rs/papers).

## Why DeepCausality?

1) DeepCausality is written in Rust with production-grade safety, reliability, and performance thanks to
   its [UltraGraph backend](https://deepcausality.com/blog/announcement-ultragraph-0-8).
2) DeepCausality provides recursive causal data structures that concisely
   express [arbitrary complex causal structures](https://deepcausality.com/docs/concepts/#structural-conceptualization-of-causation).
3) DeepCausality enables context awareness across complex data stored
   in [multiple contexts](https://deepcausality.com/blog/announcement-multiple-contexts).

4) DeepCausality simplifies modeling of complex tempo-spatial patterns
   and [non-Euclidean geometries](https://deepcausality.com/blog/announcement-non-euclidean).
5) DeepCausality supports [adaptive reasoning](https://deepcausality.com/blog/announcement-adaptive-reasoning).
6) DeepCausality comes with Causal State Machine (CSM).
7) DeepCausality
   supports [programmable ethics via the EffectEthos](https://deepcausality.com/blog/announcement-effect-ethos).

## 📚 Docs

* [API Docs](https://docs.rs/deep_causality/0.2.4/deep_causality/)
* [Documentation](https://deepcausality.com/docs/intro/)
* [Changelog](CHANGELOG.md)
* [Slides](docs/slides/LF_2023/DeepCausality.pdf)

* [Introduction](https://deepcausality.com/docs/intro/)
* [Architecture](https://deepcausality.com/docs/architecture/)
* [Background](https://deepcausality.com/docs/background/)
* [Concepts](https://deepcausality.com/docs/concepts/)

## 🌎 Community

* [Discord](https://discord.gg/Bxj9P7JXSj)
* [GH Discussions](https://github.com/orgs/deepcausality-rs/discussions)
* [LF Email Lists](https://deepcausality.com/community/)

## 🚀 Getting Started

In your project folder, just run in a terminal:

```bash
cargo add deep_causality
```

* [Starter Example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter)
* [More Examples](examples)
* [Tests](deep_causality/tests)

## How to run the example code

```bash
git clone https://github.com/deepcausality-rs/deep_causality.git

cd deep_causality

make example
```

## 📦 Sub-Crates

* `deep_causality`: The main DeepCausality library.
* `deep_causality_algorithms`: Provides algorithms for the DeepCausality library.
* `deep_causality_ast`: Provides utils to work on AST for the DeepCausality library.
* `deep_causality_data_structures`: Provides data structures for the DeepCausality library.
* `deep_causality_discovery`: A custom DSL for causal discovery.
* `deep_causality_haft`: Higher-Order Abstract Functional Traits a.k.a HKT. 
* `deep_causality_macros`: Provides macros for the DeepCausality library (_deprecated_).
* `deep_causality_num`: Numerical traits and utils used across the other crates.
* `deep_causality_rand`: Random number generator and statistical distributions used in deep_causality_tensor and other
* `deep_causality_tensor`: A custom tensor type used in deep_causality_algorithms and deep_causality_discovery crates.
* `deep_causality_uncertain`: Provides a custom crate for handling uncertainty.
* `examples`: A collection of example code.
* `ultragraph`: A hyper-graph library used as a backend in the deep_causality library.

## 🛠️ Build & Test

Cargo works as expected for build, test, and examples. For best performance, 
please build with the CPU profile that matches your platform target to enable various
auto-SIMD optimizations. To do so, use the RUSTFLAGS as shown below:

```bash 
    RUSTFLAGS='-C target-cpu=native' cargo  build 
```

You get the list of all CPU targets supported by your Rust compiler by running:

```bash 
    rustc --print target-list
```

In addition to cargo, a makefile exists that abstracts over several additional tools you for linting and formatting.
To check and install missing tools, please run the following command:

```bash 
    make install
```

You find the install script in the [script folder.](build/scripts/install_deps.sh)

The script tests and tries to install all required developer dependencies.
if the automatic install fails, the script will show a link with further installation instructions.

After all dependencies have been installed, the following commands are ready to use.

```bash 
    make build          Builds the code base incrementally (fast) for dev.
    make bench          Runs all benchmarks across all crates.
    make check          Checks the code base for security vulnerabilities.
    make example        Runs the example code.
    make fix            Fixes linting issues as reported by clippy
    make format         Formats call code according to cargo fmt style
    make install        Tests and installs all make script dependencies
    make start          Starts the dev day with updating rust, pulling from git remote, and build the project
    make test           Runs all tests across all crates.
```

The scripts called by each make command are located in the [script folder.](build/scripts)

In addition to Cargo and related tools, the entire mono-repo is configured to build and test with Bazel.
Please [install bazelisk ](https://github.com/bazelbuild/bazelisk) as it is the only requirement to build the repo with
Bazel. For more details on working with Bazel, see the [Bazel](Bazel.md) document.

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, open an issue and ask. For more significant code contributions,
please run make test and make check locally before opening a PR.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT license without additional terms or conditions.

For details:

* [Charta](DeepCausalityProjectCharter.pdf)
* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Contributing](CONTRIBUTING.md)
* [Release](RELEASE.md)
* [Support](SUPPORT.md)

## 🙏 Credits

The project took inspiration from several researchers and their projects in the field:

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html) at UCLA
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy) at the Perimeter Institute
* [Kenneth O. Stanley](https://www.kenstanley.net/home)
* [Ilya Shpitser](https://www.cs.jhu.edu/~ilyas/)
* [Miguel Hernan](https://www.hsph.harvard.edu/miguel-hernan/), [Causal Lab](https://causalab.sph.harvard.edu/) at
  Harvard University
* [Elias Bareinboim](https://causalai.net/) at Columbia University
* [Causality and Machine Learning](https://www.microsoft.com/en-us/research/group/causal-inference/) at Microsoft
  Research
* [Causal ML](https://github.com/uber/causalml) at uber.

DeepCausality implements the following research publications:

* ["An Algebraic Roadmap of Particle Theories"](docs/papers/algebraic_physics.pdf) 
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](docs/papers/ddic.pdf)
* ["NWHy: A Framework for Hypergraph Analytics"](docs/papers/nwhy.pdf)
* ["Observational causality by states and interaction type for scientific discovery"](docs/papers/surd-state.pdf)
* ["Probability Theories with Dynamic Causal Structure"](docs/papers/causaloid.pdf)
* ["Uncertain T: A First-Order Type for Uncertain Data"](docs/papers/uncertain_t.pdf)

Parts of the implementation are also inspired by:

* [Differentiable Types](https://github.com/tensorflow/swift/blob/main/docs/DifferentiableTypes.md)
* [Extension Trait](http://xion.io/post/code/rust-extension-traits.html)
* [Storage API](https://github.com/petgraph/petgraph/issues/563)
* [gTime: Time Graph](https://youtu.be/dIeYjLtg6s4)

Finally, inspiration, especially related to the hypergraph structure, was derived from reading
the [Quanta Magazine](https://www.quantamagazine.org/).

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 🎁 Sponsors

[![JetBrains logo.](https://resources.jetbrains.com/storage/products/company/brand/logos/jetbrains.svg)](https://jb.gg/OpenSource)

[JetBrains](https://www.jetbrains.com/), the premier software development IDE provider, has granted a
free [all-product license](https://www.jetbrains.com/all/) to the DeepCausality project.
The project team expresses its gratitude towards JetBrains generous support.
