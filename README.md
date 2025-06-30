[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![CodeFactor][codefactor-badge]][codefactor-url]
![Benchmarks][benches-url]
![Tests][test-url]
[![OpenSSF Best Practices][ossf-badge]][ossf-url]
[![codecov][codecov-badge]][codecov-url]


[codefactor-badge]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causality/badge

[codefactor-url]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causality

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

[benches-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_benches.yml/badge.svg

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg


---

<div align="center">

[<img src="img/logo_color.png">](https://deepcausality.com)

</div>

---

<div style="display: flex; flex-wrap: wrap; justify-content: center; align-items: center; text-align: center;">

[Website](https://deepcausality.com) | [Getting started](https://deepcausality.com/getting-started/) | [Documentation](https://deepcausality.com/docs/intro/) | [Blog](https://deepcausality.com/blog/) | [Discord](https://discord.gg/Bxj9P7JXSj) | [Crates](https://crates.io/crates/deep_causality)

</div>


DeepCausality is a hyper-geometric computational causality library that enables fast and deterministic context-aware
causal reasoning over complex multi-stage causality models. Deep Causality adds only minimal overhead and thus is
suitable for real-time applications without additional acceleration hardware. DeepCausality is hosted as a sandbox project in the [LF AI & Data Foundation](https://landscape.lfai.foundation/).

## 🤔 Why DeepCausality?

1) DeepCausality is written in Rust with safety, reliability, and performance in mind.
2) DeepCausality provides recursive causal data structures that concisely express arbitrary complex causal
   structures.
3) DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored
   within (multiple) context-hyper-graphs.
4) DeepCausality simplifies modeling of complex tempo-spatial patterns.
5) DeepCausality comes with Causal State Machine (CSM)

## 📚 Docs

* [API Docs](https://docs.rs/deep_causality/0.2.4/deep_causality/)
* [Documentation](https://deepcausality.com/docs/intro/)
* [Changelog](CHANGELOG.md)
* [Slides](docs/slides/LF_2023/DeepCausality.pdf)

## 🌎 Community

* [Discord](https://discord.gg/Bxj9P7JXSj)
* [GH Discussions](https://github.com/orgs/deepcausality-rs/discussions)
* [LF Email Lists](https://deepcausality.com/community/)

## 🚀 Getting Started

In your project folder, just run in a terminal:

```bash
cargo add deep_causality
```

See the [starter example](https://deepcausality.com/getting-started/).

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

# Smoking inference
cargo run --release --bin example-smoking

# Getting started example
cargo run --release --bin starter
```

## 📦 Sub-Crates

* [Datastructures](https://github.com/deepcausality-rs/deep_causality/tree/main/dcl_data_structures/README.md)
* [Ultragraph](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph/README.md)
* [Macros](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_macros/README.md)

## ⭐ Usage

Docs:

* [Introduction](https://deepcausality.com/docs/intro/)
* [Architecture](https://deepcausality.com/docs/architecture/)
* [Background](https://deepcausality.com/docs/background/)
* [Concepts](https://deepcausality.com/docs/concepts/)

Code:

* [Benchmarks](deep_causality/benches/benchmarks)
* [Example code](examples)
* [Tests](deep_causality/tests)

## 🛠️ Build & Test 

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over several additional tools you for linting and formatting. 
To check and install missing tools, please run the following command:

```bash 
    make install
```

You find the install script in the [script folder.](scripts/install_deps.sh)

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

The scripts called by each make command are located in the [script folder.](scripts)

##  Bazel

In addition to Cargo and related tools, the entire mono-repo is configured to build and test with Bazel.
Please [install bazelisk ](https://github.com/bazelbuild/bazelisk)as it is the only requirement to build the repo with Bazel.  

To query available crate aliases with Bazel, run:

```bash 
    bazel query "kind('alias', //alias/...)"
```

Note, this project uses Bazel aliases extensively for dependencies to ensure if crates relocate, only its alias in [alias/BUILD.bazel](alias/BUILD.bazel) needs to be update to let the build resume. 

To build all targets with Bazel, run:

```bash 
    bazel build //...
```

To build only a specific target and its dependencies, run 

```bash 
    bazel build //alias:deep_causality
```

To test all targets with Bazel, run:

```bash 
    bazel test //...
```

To test only a specific target, run:

```bash 
    bazel test //deep_causality/...
```

To query all available tests to find, for example, all spacetime  tests, run:

```bash 
    bazel query "kind('rust_test', //...)" | grep spacetime
```

To explore all dependencies of a specific crate, run:

```bash 
    bazel query "deps(//alias:dcl_data_structures)"
```

To find all reverse dependencies, i.e. packages that depends on a specific crate, run:

```bash 
    bazel query "rdeps(//..., //alias:dcl_data_structures, 1)"
```

If you were to refactor the dcl_data_structures crate, the rdepds tell you 
upfront were its used and thus helps you to estimate upfront the blast radius of braking changes. 

To query available vendored external dependencies with Bazel, run:

```bash 
    bazel query "kind('rust_library', //thirdparty/...)"
```

Note, these vendored external dependencies are shared across all crates.

To visualize all dependencies of the top level crate deep_causality, run 

```bash 
   bazel query 'deps(//alias:deep_causality, 3) ' --output graph --noimplicit_deps  | dot -Tpng -o graph.png
   
   open graph.png # Works on Mac. 
```

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

## 🎁 Sponsors

![JetBrains logo](https://resources.jetbrains.com/storage/products/company/brand/logos/jb_beam.svg)

[JetBrains](https://www.jetbrains.com/), the premier software development tool provider, has granted a
free [all-product license](https://www.jetbrains.com/all/) under
its [open-source community support program](https://www.jetbrains.com/community/opensource/#support) to the
DeepCausality project. The project team expresses its gratitude towards JetBrains generous contribution. Thank you for
your commitment to OSS development!

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
