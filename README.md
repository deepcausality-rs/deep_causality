[//]: # (---)
[//]: # (SPDX-License-Identifier: MIT)
[//]: # (---)

# DeepCausality

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Audit][audit-url]
![Benchmarks][benches-url]
![Clippy][clippy-url]
![Tests][test-url]
[![OpenSSF Best Practices][ossf-badge]][ossf-url]
[![codecov][codecov-badge]][codecov-url]

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

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg
[benches-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_benches.yml/badge.svg
[clippy-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust-clippy.yml/badge.svg
[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

Web: https://deepcausality.com

DeepCausality is a hyper-geometric computational causality library that enables fast and deterministic context-aware
causal reasoning over complex multi-stage causality models. Deep Causality adds only minimal overhead and thus is
suitable for real-time applications without additional acceleration hardware. 

[How is deep causality different from deep learning?](deep_causality/docs/difference.md)

## 🤔 Why DeepCausality?

1) DeepCausality is written in Rust with safety, reliability, and performance in mind.
2) DeepCausality provides recursive causal data structures that concisely express arbitrary complex causal
   structures.
3) DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored
   within (multiple) context-hyper-graphs.
4) DeepCausality simplifies modeling of complex tempo-spatial patterns.
5) DeepCausality comes with [Causal State Machine (CSM)](deep_causality/docs/causal_state_machine.md)

## 📚 Docs

* [API Docs](https://docs.rs/deep_causality/0.2.4/deep_causality/)
* [Docs](deep_causality/docs/README.md)
* [Slides](deep_causality/docs/slides/)

## 🚀 Install

In your project folder, just run in a terminal:

```bash
cargo add deep_causality
```

## How to run the example code

```bash
git clone https://github.com/deepcausality-rs/deep_causality.git

cd deep_causality

make example
```

You can also run the example code from the project root with cargo:

```bash
# make sure you're in the project root folder
cd deep_causality

# CSM (Causal State Machine)
cargo run --release --bin example-csm

# CTX (Context) 
cargo run --release --bin example-ctx

# Smoking inference
cargo run --release --bin example-smoking
```

## 📦 Sub-Crates

* [Datastructures](https://github.com/deepcausality-rs/deep_causality/tree/main/dcl_data_structures/README.md)
* [Ultragraph](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph/README.md)
* [Macros](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_macros/README.md)

## ⭐ Usage

See:

* [Benchmarks](deep_causality/benches/benchmarks)
* [Examples](deep_causality/examples)
* [Tests](deep_causality/tests)

## Causal Graph Reasoning 

DeepCausality reasons using the causaloid as its central data structure. 
A causaloid encodes a causal relation as a causal function that maps input data
to an output decision determining  whether the causal relation applied to the input data holds true.

The causaloid, however, can be a singleton, a collection, or a graph. The causaloid-graph, however, is a hypergraph with each node being a causaloid. This recursive structure means a sub-graph can be encapsulated as a causaloid which then becomes a node of a graph. A HashMap of causes can be encapsulated as a causaloid and embedded into the same graph. 
Then, the entire causaloid-graph can be analyzed in a variety of ways, for example:

* Reason over the entire graph
* Reason only over a specific causaloid
* Reason over all causaloids between a start and stop causaloid.
* Reason over the shortest path between two causaloids.

As long as causal mechanisms can be expressed
as a hyper-graph, the graph is guaranteed to evaluate them. That means, any combination of
single cause, multi cause, or partial cause can be expressed across many layers.
Also note, once activated, a causaloid stays activated until a different dataset evaluate its
negatively which will then deactivate the causaloid. Therefore, if parts of the dataset remain
unchanged, the corresponding causaloids will remain active.

By default, the causaloid ID is matched to the data index. For example, the root causaloid at index 0 will match to the data at index 0 and the data from index 0 will be used to
evaluated the root causaloid. If, for any reason, the data set is ordered differently,
an optional data_index parameter can be specified that is basically is a hashmap that maps
the causaloid ID to a custom data index position.

Reasoning performance for basic causality functions is a guaranteed sub-second for graphs below 10k nodes
and microseconds for graphs below 1k nodes. However, graphs with well above 100k nodes may require a large amount of memory (> 10GB) because of the underlying sparse matrix representation.

See tests as code examples:

* [Causaloid](deep_causality/tests/types/reasoning_types/causaloid_tests.rs)
* [Causal Graph](deep_causality/tests/types/reasoning_types/causality_graph_tests.rs)
* [Causal Graph Reasoning](deep_causality/tests/types/reasoning_types/causality_graph_reasoning_tests.rs)
* [Causal Graph Explaining ](deep_causality/tests/types/reasoning_types/causality_graph_explaining_tests.rs)


## Contextual Causal Model 

DeepCausality enables context aware causality reasoning through combining contexts and causal structures in a causal model. A context consists of multiple contoids with each one storing relevant contextual information. For example, a temporal context is comprised of contextoids storing time information. Note that the temporal structure of a context is usually a time hypergraph in which time information is stored as nodes in the context. 

Next, the context may also define specific data that occurs either at a certain time, a certain place, or at a certain place at a certain time (space-time). In each of these cases, the context defines a separate data object, called a dataoid that contains all the data and which is then linked to the corresponding tempoid in case of a time context. 

When data is generated in such a way that it varies in space and in time, for example from a sensor attached to a drone, the measurements of the sensor are then stored in a custom dataoid whereas the meta-data of the dataid is stored in a custom space-tempoid that represents the time and location of the data. The reason for this separation is that, at any point in space/time, more than one measurement may occur. In the drone example, let us assume that the drone may have 5 different sensors that are all read in the same interval, the space-time node links to all five sensor dataoids.

A context can either be static or dynamic depending on the specific situation. When building and updating a dynamic context, 
it is possible that the underlying hypergraph may grow very large and therefore it is necessary to implement a pruning mechanism that removes old branches from the context graph at regular intervals.

Causal models can be built either without a context or with a context. 
In the latter case, an immutable reference to the context is passed into the constructor of the causaloid. The idea is that the causal model never modifies the context. When constructing a causaloid which reasons over observed data in relation to the context, the causal function with context is used instead of the regular causal function. Within that causal function, you can access the full context graph. 

However, if the context is generated dynamically, you might want to use a dynamic secondary index to determine the actual index of any contextoid in the context. 
The dynamic secondary index could be an algorithm to calculate the index based on certain parameters derivable from the data, i.e. temporal arithmetic based 
on timestamps. To make the secondary index accessible from within the causal function, you may extend the context with an extension trait 
and corresponding implementation.

* [Contexts](deep_causality/docs/context.md)
* [Customizing context](deep_causality/docs/customizing_context.md)
* [Contextoid](deep_causality/src/types/context_types/contextoid/contextoid.rs)
* [Context Graph](deep_causality/src/types/context_types/context.rs)
* [Example of contextualized causal model](deep_causality/examples/ctx/src/run.rs)

## Causal State Machine

A causal state machine models a context-free system where each cause maps to a known effect. The example below models a sensor network that screens an industry site for smoke, fire, and explosions. Because the sensors are reliable, an alert will be raised whenever the sensor exceeds a certain threshold. You could implement this kind of system in many ways,
but as the example shows, the causal state machine makes the system relatively easy to maintain. New sensors, for example, from a drone inspection, can be added and evaluated
dynamically.

* [CSM](deep_causality/src/types/csm_types/mod.rs)
* [CsmAction](deep_causality/src/types/csm_types/csm_action.rs)
* [Csm CausalState](deep_causality/src/types/csm_types/csm_state.rs)
* [CSM example code](deep_causality/examples/csm)

## 🛠️ Cargo & Make

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over several additional tools you may have to install
before all make commands work:

* [clippy](https://github.com/rust-lang/rust-clippy)
* [nextest](https://nexte.st/)
* [outdated](https://github.com/kbknapp/cargo-outdated)
* [udeps](https://crates.io/crates/cargo-udeps)
* [audit](https://crates.io/crates/cargo-audit)
* [llvm-cov](https://github.com/taiki-e/cargo-llvm-cov#installation)

```bash
    make build          Builds the code base incrementally (fast).
    make bench          Runs all benchmarks across all crates.
    make check          Checks the code base for security vulnerabilities.
    make coverage       Checks test coverage and generates a html report.
    make example        Runs the code examples.
    make fix            Auto-fixes linting issues as reported by cargo and clippy.
    make test           Runs all tests across all crates.
```

## 👩‍👩‍👧‍👦 Community

* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Support](SUPPORT.md)

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, open an issue and ask. For more significant code contributions,
please run make test and make check locally before opening a PR.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT license without additional terms or conditions.

For details:

* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Contributing](CONTRIBUTING.md)
* [Release](RELEASE.md)

## 🙏 Credits

The project took inspiration from several researchers and their projects in the field:

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html) at UCLA
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy) at the Perimeter Institute
* [Kenneth O. Stanley](https://www.kenstanley.net/home) at OpenAI
* [Ilya Shpitser](https://www.cs.jhu.edu/~ilyas/) at Johns Hopkins University
* [Miguel Hernan](https://www.hsph.harvard.edu/miguel-hernan/), [Causal Lab](https://causalab.sph.harvard.edu/) at
  Harvard University
* [Elias Bareinboim](https://causalai.net/) at Columbia University
* [Causality and Machine Learning](https://www.microsoft.com/en-us/research/group/causal-inference/) at Microsoft
  Research
* [Causal ML](https://github.com/uber/causalml) at uber.

Parts of the implementation are inspired by:

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

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
