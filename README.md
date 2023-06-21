# Deep Causality

## About



## Concepts

## Install

```Bash
cargo add deep_causality
```


## Usage:

Full code in [Smoking example](examples/smoking/run.rs)

```rust


```

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