# Deep Causality

## About 

Deep Causality is a hyper-geometric computational causality library that enables fast context
aware causal reasoning across arbitrary complex causal models. Deep Causality induces only
minimal overhead and thus is suitable for deployment on low power (IoT) devices 
or for real-time applications. 

**Why?**

> "ANSR hypothesizes that several of the limitations in ML today are a consequence of the inability to incorporate contextual
> and background knowledge, and treating each data set as an independent, uncorrelated input.
> In the real world, observations are often correlated and a product of an underlying causal mechanism,
> which can be modeled and understood" - [ANSR](https://www.darpa.mil/program/assured-neuro-symbolic-learning-and-reasoning)

For more background information, see the [motivation document](/doc/motivation.md).

Also, take a look at [how is deep causality different from deep learning?](/doc/difference.md)

## Install

```Bash
@TODO
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

## Author

* Marvin Hansen
* Github key ID: 4AEE18F83AFDEB23
* GPG key ID: 210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC

## Licence

* [MIT Licence](LICENSE)
* Software is "as is" without any warranty. 
