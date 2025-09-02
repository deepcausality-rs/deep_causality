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


DeepCausality is a hypergeometric computational causality library that enables fast, context-aware causal reasoning over complex multi-stage causality models. DeepCausality pioneers uniform reasoning across deterministic and
probabilistic modalities by implementing the unified effect propagation process.

In the effect propagation process, cause, and effect are folded into one single entity, the causaloid, that takes
a propagating effect as input and returns another propagating effect as its output. A causaloid models causal relations
as a functional dependency of the previous propagating effect on the current propagating effect via a causal function. The
key difference from conventional classical causality, which models a causal relationship as a temporal order,
comes down to two properties of the causal function. One, the functional dependency is
independent of temporal order and therefore can handle non-Euclidean representation and relativistic effects. Second,
the causal function is unconstrained and therefore can be deterministic, probabilistic, a support vector machine or even a non-deterministic method such as a neural net. As long as the computed effect can be expressed as a propagating
effect, the causal function is valid and can be stored in a Causaloid.

A propagating effect can be a deterministic (causal), a probabilistic value, a probabilistic distribution, or an
arbitrarily complex type stored as a contextual reference. DeepCausality provides reasoning for deterministic and
probabilistic modalities, whereas reasoning over arbitrarily complex types requires custom implementation. To streamline
data sharing, those complex types are stored and loaded from a context attached to the causal model.

DeepCausality supports multiple contexts that can store complex spatio-temporal data as well as data distributions to
account for uncertainty. The adjustable mechanism enables dynamic data updates, for example, from real-time data streams
or sensor data. A hypergraph represents each context and thus enables flexible data relations, i.e., a point in
spacetime may link to multiple sensor readings and a reference data distribution for each sensor to detect data
anomalies. The context, therefore, supports sophisticated reasoning across advanced causal structures. Out of the box,
DeepCausality supports multi-modal causal reasoning across singleton, causal collection, and causal hyper-graph
structures. For causal collections, multiple modes of aggregation are supported, whereas the causal hypergraph
implements the effect propagation process in which the reasoning engine traverses the graph, applies the previous
propagating effect to the current causaloid, and then takes that propagating effect and applies it to the next causaloid
until the graph traversal ends. To support flexible reasoning over geometric causal structures, DeepCausality supports
common path algorithms, i.e., shortest path, start from a node, and path between two nodes.

Once a final conclusion has been reached, the causal state machine enables the explicit linking between the conclusion
and a specific action to be taken as a result. However, because dynamic reasoning over a dynamic context may not always
result in a predictable outcome, DeepCausality has developed the EffectEthos, a programmable ethos, to encode contextual
operational rules the causal state machine can check to ensure that a proposed action is safe and within the pre-defined
rules. The effect ethos can access the same context as the causal model that has led to the insight that triggered a
proposed action and can therefore retrieve relevant and timely data to decide whether the action should be taken. One
key aspect of the effect ethos is its ability to resolve conflicting rules via an internal algorithm that gives
precedent to a rule with higher priority, or a higher authority, to ensure the final rule set is correctly applied.
Furthermore, a tagging system enables efficient re-use and selection of applicable rules.

DeepCausality applies state-of-the-art performance optimization, such as its custom compact sparse representation (CSR)
hypergraph implementation that delivers sub-second traversal time on graphs with ten million nodes or more. Furthermore,
static dispatching in all critical hot paths ensures significant performance even on moderate hardware and thus is
suitable for real-time applications without additional acceleration hardware.

In terms of applications, DeepCausality enables a number of advanced use cases, such as real-time sensor fusion,
real-time contextual risk monitoring, and contextual interaction i.e. in robotics safeguarded by its effect ethos.

DeepCausality is hosted as a sandbox project in the [LF AI & Data Foundation](https://landscape.lfai.foundation/).

## 🤩 Why DeepCausality?

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

* [Datastructures](https://github.com/deepcausality-rs/deep_causality/tree/main/dcl_data_structures/README.md)
* [Ultragraph](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph/README.md)
* [Macros](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_macros/README.md)
* [Uncertainty](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain/README.md)

## 🛠️ Build & Test

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over several additional tools you for linting and formatting.
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

* ["Probability Theories with Dynamic Causal Structure"](docs/papers)- Lucian Hardy
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](docs/papers/ddic.pdf) - Forbus et. al.
* ["NWHy: A Framework for Hypergraph Analytics"](docs/papers/nwhy.pdf) - Lumsdaine et. al.
* ["Uncertain T: A First-Order Type for Uncertain Data"](docs/papers/uncertain_t.pdf) - Bornholt et. al.

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

![JetBrains logo](https://resources.jetbrains.com/storage/products/company/brand/logos/jb_beam.svg)

[JetBrains](https://www.jetbrains.com/), the premier software development tool provider, has granted a
free [all-product license](https://www.jetbrains.com/all/) under
its [open-source community support program](https://www.jetbrains.com/community/opensource/#support) to the
DeepCausality project. The project team expresses its gratitude towards JetBrains generous contribution. Thank you for
your commitment to OSS development!
