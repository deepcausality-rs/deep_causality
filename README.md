# 💡 DeepCausality

DeepCausality is a hyper-geometric computational causality library that enables fast and deterministic context aware
causal reasoning over complex multi-stage causality models. Deep Causality adds only minimal overhead,
and thus is suitable for deployment on low-power (IoT) devices or real-time applications without additional acceleration
hardware.

## 🤔 Why DeepCausality?

1) DeepCausality is written in Rust with production grade safety, reliability, and performance in mind.
2) DeepCausality provides recursive causal data-structures that enable concise expression of arbitrary complex causal
   structures.
3) DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored
   within (multiple) context-hyper-graphs.
4) DeepCausality simplified modelling of complex tempo-spatial patterns.
5) DeepCausality is small, efficient, and fast, really fast. Run [benchmarks](deep_causality/benches) and see it.

## 📚 Docs:

* [Motivation](docs/motivation.md)
* [How is deep causality different?](docs/difference.md)
* [Causal Structure](docs/causal_structure.md)
* [Concepts](docs/concepts.md)
* [Context](docs/context.md)
* [Heilmeier Questions](docs/heilmeier_questions.md)

## 🚀 Install

Add the following to your Cargo.toml

```toml
deep_causality = { git = "https://github.com/deepcausality/deep_causality.git", tag = "0.2.1" }
```

## ⭐ Usage:

See:

* [Benchmark](deep_causality/benches/benchmarks)
* [Example](deep_causality/examples/smoking/run.rs)
* [Test](deep_causality/tests)

## 🛠️ Cargo & Make:

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over a number of additional tools you may have to install
before all make commands work:

* [clippy](https://github.com/rust-lang/rust-clippy)
* [nextest](https://nexte.st/)
* [outdated](https://github.com/kbknapp/cargo-outdated)
* [udeps](https://crates.io/crates/cargo-udeps)
* [audit](https://crates.io/crates/cargo-audit)

```bash 
    make build          Builds the code base incrementally (fast).
    make bench          Runs all benchmarks across all crates.
    make check          Checks the code base for security vulnerabilities.
    make example        Runs the default example: Smoking.
    make fix            Auto-fixes linting issues as reported by cargo and clippy.
    make test           Runs all tests across all crates.
```

## 🙏 Credits

The project took inspiration by work from several researchers and their teams:

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html) at UCLA
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy) at the Perimeter Institute
* [Kenneth O. Stanley](https://www.kenstanley.net/home) at OpenAI
* [Ilya Shpitser](https://www.cs.jhu.edu/~ilyas/) at Johns Hopkins University
* [Miguel Hernan](https://www.hsph.harvard.edu/miguel-hernan/), [Causal Lab](https://causalab.sph.harvard.edu/) at
  Harvard University
* [Elias Bareinboim](https://causalai.net/) at Columbia University
* [Causality and Machine Learning](https://www.microsoft.com/en-us/research/group/causal-inference/) at Microsoft
  Research

Parts of the implementation are inspired by:

* [Differentiable Types](https://github.com/tensorflow/swift/blob/main/docs/DifferentiableTypes.md)
* [Extension Trait](http://xion.io/post/code/rust-extension-traits.html)
* [Storage API](https://github.com/petgraph/petgraph/issues/563)

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask. For larger code contributions, please
run make check locally before opening a PR and please add tests to make the PR merge
relatively straight forward.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 💻 Author

* Marvin Hansen
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
