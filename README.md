# Deep Causality

## About

Deep Causality is a hyper-geometric computational causality library that enables fast and deterministic context aware causal reasoning over complex multi-stage causality models. Deep Causality adds only minimal overhead,
and thus is suitable for deployment on low-power (IoT) devices or real-time applications without additional acceleration hardware.

**Documents:**

* [Motivation](docs/motivation.md)
* [How is deep causality different?](docs/difference.md)
* [Concepts](docs/concept_guide.md)

## Install

Add the following to your Cargo.toml

```toml
deep_causality = { git = "https://github.com/deepcausality/deep_causality.git", tag = "0.2.1" }
```

## Usage:

See:

* [Benchmark](deep_causality/benches/benchmarks)
* [Example](deep_causality/examples/smoking/run.rs)
* [Test](deep_causality/tests)

## Cargo & Make

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over a number of additional tools you may have to install
before all make scripts work:

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

## Licence

This project is licensed under the [MIT license](deep_causality/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed as MIT, without any additional terms or conditions.

## Author

* Marvin Hansen
* Github key ID: 4AEE18F83AFDEB23
* GPG key ID: 210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
