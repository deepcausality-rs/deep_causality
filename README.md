# DeepCausality

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
![Audit][audit-url]
![Tests][test-url]
![Clippy][clippy-url]


[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg

[clippy-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust-clippy.yml/badge.svg

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

Web: https://deepcausality.com

DeepCausality is a hyper-geometric computational causality library that enables fast and deterministic context-aware
causal reasoning over complex multi-stage causality models. Deep Causality adds only minimal overhead and thus is
suitable for real-time applications without additional acceleration hardware. Take a look
at [how is deep causality different from deep learning?](docs/difference.md)

## 🤔 Why DeepCausality?

1) DeepCausality is written in Rust with production-grade safety, reliability, and performance in mind.
2) DeepCausality provides recursive causal data structures that concisely express arbitrary complex causal
   structures.
3) DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored
   within (multiple) context-hyper-graphs.
4) DeepCausality simplified modeling of complex tempo-spatial patterns.
5) DeepCausality comes with [Causal State Machine (CSM)](docs/causal_state_machine.md)

## 📚 Docs

* [API Docs](https://docs.rs/deep_causality/0.2.4/deep_causality/)
* [Motivation](docs/motivation.md)
* [How is deep causality different?](docs/difference.md)
* [Causal Structure](docs/causal_structure.md)
* [Causal State Machine (CSM)](docs/causal_state_machine.md)
* [Concepts](docs/concepts.md)
* [Context](docs/context.md)
* [Heilmeier Questions](docs/heilmeier_questions.md)
* [Limitations](docs/limitations.md)

## 🚀 Install

Just run:

```bash
cargo add deep_causality
```

Alternatively, add the following to your Cargo.toml

```toml
deep_causality = { git = "https://github.com/deepcausality/deep_causality.git", tag = "0.2.4" }
```

## ⭐ Usage

See:

* [Benchmarks](deep_causality/benches/benchmarks)
* [Examples](deep_causality/examples)
* [Tests](deep_causality/tests)

### Causal State Machine

A causal state machine models a context-free system where each cause maps to a known effect. The example below
models a sensor network that screens an industry site for smoke, fire, and explosions. Because the
sensors are reliable, an alert will be raised whenever the sensor exceeds a certain threshold.

You could implement this kind of system in many different ways, but as the example shows, the causal state machine makes
the system relatively easy to maintain and extend. New sensors, for example, from a drone inspection, can
be added and evaluated dynamically.

[Full example code](deep_causality/examples/csm)

```rust
const SMOKE_SENSOR: usize = 1;
const FIRE_SENSOR: usize = 2;
const EXPLOSION_SENSOR: usize = 3;

pub fn run() {
    let data = [0.0f64];
    let smoke_causloid = get_smoke_sensor_causaloid();
    let smoke_cs = CausalState::new(SMOKE_SENSOR, 1, &data, &smoke_causloid);
    let smoke_ca = get_smoke_alert_action();

    let fire_causaloid = get_fire_sensor_causaloid();
    let fire_cs = CausalState::new(FIRE_SENSOR, 1, &data, &fire_causaloid);
    let fire_ca = get_fire_alert_action();

    let explosion_causaloid = get_explosion_sensor_causaloid();
    let explosion_cs = CausalState::new(EXPLOSION_SENSOR, 1, &data, &explosion_causaloid);
    let explosion_ca = get_explosion_alert_action();

    let state_actions = &[
        (&smoke_cs, &smoke_ca),
        (&fire_cs, &fire_ca),
    ];

    println!("Create Causal State Machine");
    let csm = CSM::new(state_actions);

    let smoke_data = get_smoke_sensor_data();
    let fire_data = get_fire_sensor_data();
    let exp_data = get_explosion_sensor_data();

    println!("Add a new sensor");
    csm.add_single_state(EXPLOSION_SENSOR, (&explosion_cs, &explosion_ca)).expect("Failed to add Explosion sensor");

    println!("Start data feed and monitor senors");
    for i in 0..12 {
        wait();
        csm.eval_single_state(SMOKE_SENSOR, &[smoke_data[i]]).expect("Panic: Smoke sensor failed");

        csm.eval_single_state(FIRE_SENSOR, &[fire_data[i]]).expect("Panic: Fire sensor failed");

        csm.eval_single_state(EXPLOSION_SENSOR, &[exp_data[i]]).expect("Panic: Explosion sensor failed");
    }
}
```

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
If unsure where to start, open an issue and ask. For more significant code contributions, 
please run make test and make check locally before opening a PR.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT license without additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 💻 Author

* Marvin Hansen, [Emet-Labs](https://emet-labs.com/).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
