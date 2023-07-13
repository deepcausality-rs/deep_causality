## 🛠️ Cargo & Make

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over several additional tools you may have to install
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