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

# Overview

DeepCausality is a hypergeometric computational causality library that enables fast, context-aware causal reasoning
over complex multi-stage causality models. DeepCausality pioneers uniform reasoning across deterministic and
probabilistic modalities by implementing the unified effect propagation process.

### The "Classical" Way of Thinking About Causality

Imagine a simple thermostat.

* **Cause:** The room temperature drops below 68 degrees Fahrenheit.
* **Effect:** The furnace turns on.

A typical classical causal model works because it relies on three fundamental assumptions:

1. **Time is a straight line.** The temperature *always* drops *before* the furnace turns on. There's a clear "
   happen-before" relationship.
2. **The causal rules are fixed.** The law "if temp < 68, then turn on furnace" is static and unchanging. It will be the
   same rule tomorrow as it is today.
3. **Context is implicit** Context is assumed as the implicit background and therefore all data are
   captured in variables relative to implicit context. Here, it is important that the data collection and the causal model
   assume the same implicit background context otherwise errors are very hard to find. 

All previous computational causality frameworks (like those pioneered by Judea Pearl) are built on these three powerful
assumptions. They provide the foundation to discover and reason about these fixed causality in a world where time moves
forward predictably, the rules remains the same, and adding some variables capture the implicit context.

### The Problem: A Dynamic World Breaks Classical Causality

Next, imagine a more complex system, like a financial market or a fleet of autonomous wildfire-fighting drones and you
see that reality operates differently:

1. **Time is NOT a straight line.** In a trading system, events happen on nanosecond scales, but the market context
   relies on different time scales i.e. the hourly high price, the previous days close price
   or the daily trade volume. Time becomes multi-layered, multi-scaled and complex.

2. **The rules can change .** This is the most important point. During a normal market day, "low interest rates cause
   stock prices to rise." But during a market crash (a "regime shift"), that rule breaks down entirely, and a new rule
   like "high fear causes all assets to fall" takes over. The causal relationships within a system have changed
   dynamically.

3. **Context changes dynamically.** The reason causal rules may change is because a system's context is changing
   dynamically. For an autonomous drone relying on a GPS signal navigation might be valid, but the moment the drone
   enters a drone, the GPS signal gets temporarily lost and with it the drones ability to navigate. Here, the context is
   particularly important because the computer vision system almost certainly identified the tunnel entrance, but
   without a workable context, the information cannot be used.

DeepCausality was created from the ground up to model dynamic causality in a system where context changes continuously,
multi-scaled time, and the causal rules themselves may change in response its changing context.

### The Core Idea of DeepCausality: A Radical Simplification

DeepCausality rethinks causality from the ground up based on single single foundation:

**"Causality is a spacetime-agnostic functional dependency."**

* **"Functional dependency":** This just means `Effect2 = function(Effect1)`. Instead of "cause and effect," think of a
  chain reaction where one event triggers a causal function that produces the next event. The focus is on the *process*
  of event propagation.
* **"Spacetime-agnostic":** This is the radical part. Time and space are just another piece of contextual data for the
  causal function.
* **"Explicit Context":** Because the causal function is independent of spacetime, any time or space related data needs
  to be provided via a context. A powerful hypergraph enables flexible context modelling and DeepCausality enables a
  model
  to access and use multiple contexts.

The core of the idea is similar to a ripple in a pond. One ripple (an effect) propagates outward and creates the next
ripple (another effect). DeepCausality is a framework for defining the rules of how those ripples spread. For more
information
about the underlying effect propagation process, see the [Deep Dive document.](README_DEEP_DIVE.md).

### The Three Pillars of DeepCausality

DeepCausality has three main components to make all this work:

#### 1. The Causaloid:

* **What it is:** A self-contained, single unit of causality.
* **What it does:** It holds a single causal function (`E2 = f(E1)`). It receives an incoming effect, runs its causal
  function, and emits a new, outgoing effect.

#### 2. The Context:

* **What it is:** The explicit environment where the Causaloids operate. It holds all the factual data.
* **What it does:** The Context is a super-flexible data structure (a hypergraph) that holds all the facts about the
  world: the current time, sensor readings, locations on a map, etc.

#### 3. The Effect Ethos

* **What it is:** A programmable ethos, to encode and verify operational rules
* **What it does:** A Causaloid might reason, "Based on the data, the most logical action is X." But before action X can
  be taken, the Effect Ethos steps in and checks against a set of rules. It answers the question "**Should this happen?
  **"

In summary, the EPP is a framework for building systems that can reason about cause and effect in complex, dynamic
environments where the rules themselves can change. It achieves this by treating causality as a process of **effect
propagation** between simple, composable **Causaloids** that operate on an explicit, flexible **Context**, all governed
by a verifiable safety layer called the **Effect Ethos**. DeepCausality is hosted as a sandbox project in
the [LF AI & Data Foundation](https://landscape.lfai.foundation/).

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

* ["Probability Theories with Dynamic Causal Structure"](docs/papers)
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](docs/papers/ddic.pdf)
* ["NWHy: A Framework for Hypergraph Analytics"](docs/papers/nwhy.pdf) 
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

![JetBrains logo](https://resources.jetbrains.com/storage/products/company/brand/logos/jb_beam.svg)

[JetBrains](https://www.jetbrains.com/), the premier software development tool provider, has granted a
free [all-product license](https://www.jetbrains.com/all/) under
its [open-source community support program](https://www.jetbrains.com/community/opensource/#support) to the
DeepCausality project. The project team expresses its gratitude towards JetBrains generous contribution. Thank you for
your commitment to OSS development!
