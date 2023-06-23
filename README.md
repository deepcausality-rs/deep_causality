# Deep Causality

## About

Deep Causality is a hyper-geometric computational causality library that enables fast context aware causal reasoning.
Deep Causality contributes a novel contextualized causality reasoning engine that enables deterministic reasoning over
poly-contextual, complex multi-stage causality models. Deep Causality adds only minimal overhead,
and thus is suitable for deployment on low-power (IoT) devices or real-time applications without additional acceleration
hardware.

**Documents:**

* [Motivation](/doc/motivation.md)
* [How is deep causality different?](/doc/difference.md)
* [Concepts](/doc/concept_guide.md)

## Install

Add the following to your Cargo.toml

```toml
deep_causality = { git = "https://github.com/deepcausality/deep_causality.git", tag = "0.2.1" }
```

## Usage:

See:

* [Benchmark](benches/benchmarks)
* [Example](examples/smoking/run.rs)
* [Test](tests)

## Cargo & Make

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over a number of additional tools you may have to install
before all make scripts work:

* [nextest](https://nexte.st/)
* [outdated](https://github.com/kbknapp/cargo-outdated)
* [udeps](https://crates.io/crates/cargo-udeps)
* [audit](https://crates.io/crates/cargo-audit)

```bash 
    make build          Builds the code base incrementally (fast).
    make bench          Runs all benchmarks across all crates.
    make check          Checks the code base for security vulnerabilities.
    make example        Runs the default example: Smoking.
    make fix            Fixes linting issues as reported by cargo
    make test           Runs all tests across all crates.
```

## Licence

* [MIT Licence](LICENSE)
* Software is "as is" without any warranty.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed as MIT, without any additional terms or conditions.

## Author

* Marvin Hansen
* Github key ID: 4AEE18F83AFDEB23
* GPG key ID: 210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
