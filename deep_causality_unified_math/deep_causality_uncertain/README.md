# deep_causality_uncertain

[![Crates.io](https://img.shields.io/crates/v/deep_causality_uncertain.svg)](https://crates.io/crates/deep_causality_uncertain)
[![Docs.rs](https://docs.rs/deep_causality_uncertain/badge.svg)](https://docs.rs/deep_causality_uncertain)

[![MIT licensed][mit-badge]][mit-url]
 

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg

 

A Rust library of first-order uncertain data types for computation and decisions under uncertainty.


## Introduction

Applications from sensor data processing and machine learning to probabilistic modeling often treat estimates as precise facts. The result is "uncertainty bugs": random errors go ignored, computations compound them, and probabilistic data drives misleading Boolean decisions (false positives/negatives).

`deep_causality_uncertain` provides `Uncertain<T>`, a programming language abstraction that models and propagates uncertainty explicitly. It follows "Uncertain<T>: A First-Order Type for Uncertain Data" by Bornholt et al. Treating uncertainty as a first-class type makes applications that handle variable data more expressive, accurate, and correct.

## Key Features

*   **Precision is a parameter.** Every type is generic in its scalar, bounded by `RandScalar`
    (`RealField + FromPrimitive`, blanket-implemented). `Uncertain<f64>`, `Uncertain<f32>`,
    `Uncertain<Float106>` and `Uncertain<BFloat16>` all work, and a scalar added to
    `deep_causality_num` tomorrow works with no line changed here. Nothing in `src/` names a
    concrete scalar.
*   **Two carriers over one graph.** `Uncertain<R>` is a real quantity; `UncertainBool<R>` is a
    truth value. A comparison or a Bernoulli leaf yields the Boolean carrier over the *same*
    `R`-carrying graph, because the tree beneath a Boolean root holds reals: a threshold, a
    Bernoulli parameter, an arithmetic operand.
*   **Probabilistic presence (`MaybeUncertain<R>`):** a real quantity that may be absent. Its
    presence channel is an `UncertainBool<R>` and its value channel an `Uncertain<R>`, drawn at one
    sample index so the two always belong to the same draw.
*   **Distributions:** `point`, `normal(mean, std_dev)`, `uniform(low, high)` on the
    real carrier, and `bernoulli(p)` on the Boolean one, every parameter in the caller's scalar.
*   **Operator overloading:** arithmetic (`+`, `-`, `*`, `/`, unary `-`) on the real
    carrier, logic (`&`, `|`, `!`, `^`) on the Boolean one, and comparisons (`greater_than`,
    `less_than`, `equals`, `approx_eq`, `within_range`, and the `*_uncertain` forms) crossing from
    one to the other.
*   **Lazy computation graph:** operations build a graph rather than evaluating, so a quantity used
    twice in one expression is drawn once: `x - x` is exactly zero at every index.
*   **Reproducible, addressed draws:** a draw is a function of three numbers (a `SampleSession`'s
    seed, the sample index, and the leaf's ordinal) and of nothing else. Nothing is stored between
    calls, no global or thread-local is consulted, and two graphs sharing a leaf agree about that
    leaf at the same index. The same seed replays the same values in a later process.
*   **Statistical analysis, in the caller's scalar:** `expected_value`, `standard_deviation`,
    `estimate_probability`, and quasi-Monte-Carlo variants (`expected_value_qmc`,
    `standard_deviation_qmc`, `estimate_probability_qmc`) over a low-discrepancy Sobol sequence.
*   **Decision making:** `to_bool`, `probability_exceeds` and `implicit_conditional` collapse
    a distribution to one verdict by sequential hypothesis testing (SPRT), drawing only as many
    samples as the decision needs; `conditional` implements `if-then-else` on an uncertain
    condition.
*   **Ensembles into a carrier you name:** `materialize::<W>` returns `W::Type<R>` (a
    `DenseVector`, a rank-1 `CausalTensor`, a `Vec`), so the ensemble arrives with that
    container's structure, and this crate declares no ensemble type of its own.
*   **The graph is an `Arrow`:** `Uncertain<R>: Arrow<In = SampleIndex, Out = Result<R, _>>`, so it
    composes with downstream computation through `haft`'s combinators, statically and with no trait
    object anywhere in the crate.

## Installation

Add `deep_causality_uncertain` to your `Cargo.toml` file:

```toml
[dependencies]
deep_causality_uncertain = "0.5" # Or the latest version
```

## Usage

### Creating uncertain values

```rust
use deep_causality_uncertain::{Uncertain, UncertainBool};

// A precise, known value.
let precise = Uncertain::<f64>::point(10.0);

// A sensor reading with Gaussian noise.
let reading = Uncertain::<f64>::normal(25.0, 0.5);

// A value somewhere in a range.
let jitter = Uncertain::<f64>::uniform(-1.0, 1.0);

// An uncertain truth value: a Bernoulli trial.
let coin = UncertainBool::<f64>::bernoulli(0.7);
```

The scalar is a parameter, so the same constructors serve any of them:

```rust
use deep_causality_num::{BFloat16, Float106};
use deep_causality_uncertain::Uncertain;

let wide = Uncertain::<Float106>::normal(Float106::from(0.0), Float106::from(1.0));
let narrow = Uncertain::<BFloat16>::normal(BFloat16::from(0.0), BFloat16::from(1.0));
let single = Uncertain::<f32>::normal(0.0, 1.0);
```

### Sessions: where a draw comes from

Every draw is addressed by a session's seed and a sample index. Hold a `SampleSession` when you
want a run to be reproducible; use the `_from_entropy` forms when you do not.

```rust
use deep_causality_uncertain::{SampleSession, Uncertain};

let reading = Uncertain::<f64>::normal(25.0, 0.5);

// Reproducible: the same seed and index give the same value, in this process or a later one.
let session = SampleSession::seeded(42);
let a = reading.sample_at(&session, 0).unwrap();
let b = reading.sample_at(&session, 0).unwrap();
assert_eq!(a, b);

// Advancing draws a fresh index each time.
let mut walking = SampleSession::seeded(42);
let first = reading.sample_next(&mut walking).unwrap();
let second = reading.sample_next(&mut walking).unwrap();
assert_ne!(first, second);

// No session of your own: nothing about the value can be reproduced afterwards, and the name
// says so.
let once = reading.sample_from_entropy().unwrap();
let _ = (a, b, first, second, once);
```

### Arithmetic and shared leaves

```rust
use deep_causality_uncertain::{SampleSession, Uncertain};

let session = SampleSession::seeded(7);
let x = Uncertain::<f64>::normal(10.0, 2.0);
let y = Uncertain::<f64>::normal(3.0, 0.5);

let total = x.clone() + y.clone();
let scaled = x.clone() * Uncertain::point(2.0);

// A quantity used twice in one expression is one draw, not two.
let difference = x.clone() - x.clone();
assert_eq!(difference.sample_at(&session, 0).unwrap(), 0.0);
let _ = (total, scaled);
```

### Comparisons cross to the Boolean carrier

```rust
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

let session = SampleSession::seeded(7);
let reading = Uncertain::<f64>::normal(100.0, 5.0);

// Every comparison yields an `UncertainBool<R>` over the same graph.
let too_high: UncertainBool<f64> = reading.greater_than(105.0);
let in_band = reading.within_range(95.0, 105.0);
let near = reading.approx_eq(100.0, 1.0);

let target = Uncertain::<f64>::normal(98.0, 3.0);
let exceeds_target = reading.gt_uncertain(&target);

let p: f64 = too_high.estimate_probability(&session, 1000).unwrap();
println!("P(reading > 105) = {:.1}%", p * 100.0);
let _ = (in_band, near, exceeds_target);
```

### Mapping

`map` and `map_to_bool` take a plain function pointer, so the graph stores no trait object. A
captured parameter belongs in the graph rather than in a closure over it.

```rust
use deep_causality_uncertain::{SampleSession, Uncertain};

let session = SampleSession::seeded(7);
let celsius = Uncertain::<f64>::normal(25.0, 2.0);

let fahrenheit = celsius.map(|c| c * 1.8 + 32.0);
let is_hot = celsius.map_to_bool(|c| c > 30.0);

let mean: f64 = fahrenheit.expected_value(&session, 1000).unwrap();
let p: f64 = is_hot.estimate_probability(&session, 1000).unwrap();
println!("{mean:.2} F, P(hot) = {:.1}%", p * 100.0);
```

### Conditional logic

```rust
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

let session = SampleSession::seeded(7);
let heavy_traffic = UncertainBool::<f64>::bernoulli(0.7);
let via_main = Uncertain::<f64>::normal(30.0, 5.0);
let via_back = Uncertain::<f64>::normal(45.0, 2.0);

let travel_time = Uncertain::conditional(heavy_traffic, via_back, via_main);
let mean: f64 = travel_time.expected_value(&session, 1000).unwrap();
println!("Expected travel time: {mean:.1} minutes");
```

### Statistical properties

```rust
use deep_causality_uncertain::{SampleSession, Uncertain};

let session = SampleSession::seeded(7);
let price = Uncertain::<f64>::normal(150.0, 10.0);

let mean: f64 = price.expected_value(&session, 1000).unwrap();
let spread: f64 = price.standard_deviation(&session, 1000).unwrap();

// Quasi-Monte-Carlo: a digitally shifted Sobol sequence, reproducible from its own seed and
// faster-converging on low-dimension static graphs.
let qmc_mean: f64 = price.expected_value_qmc(1024, 0xABCD).unwrap();

println!("{mean:.2} +/- {spread:.2}  (QMC mean {qmc_mean:.2})");
```

### Decision making

The three probabilities are stated in the caller's scalar, and the SPRT draws only as many samples
as the decision needs.

```rust
use deep_causality_uncertain::{SampleSession, UncertainBool};

let session = SampleSession::seeded(7);
let healthy = UncertainBool::<f64>::bernoulli(0.9);

// threshold, confidence, indifference region, sample budget.
if healthy.to_bool(&session, 0.5, 0.95, 0.05, 1000).unwrap() {
    println!("System is healthy at 95% confidence.");
}

// "More likely than not", with those defaults filled in.
if healthy.implicit_conditional(&session).unwrap() {
    println!("System is more likely than not healthy.");
}
```

### Probabilistic presence (`MaybeUncertain<R>`)

```rust
use deep_causality_uncertain::{MaybeUncertain, SampleSession, Uncertain};

let mut session = SampleSession::seeded(7);

// Certainly present, uncertain in value.
let present = MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(10.0, 2.0));
assert!(present.sample(&mut session).unwrap().is_some());

// Certainly absent.
let absent = MaybeUncertain::<f64>::always_none();
assert!(absent.sample(&mut session).unwrap().is_none());

// Present with probability 0.7.
let intermittent = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.7, Uncertain::normal(5.0, 1.0));

// Absence propagates through arithmetic: one absent operand makes the sum absent.
let sum = present.clone() + absent.clone();
assert!(sum.sample(&mut session).unwrap().is_none());

// Collapse to a plain `Uncertain<R>` only if the evidence of presence clears the gate.
let gate = SampleSession::seeded(8);
match intermittent.lift_to_uncertain(&gate, 0.6, 0.95, 0.05, 1000) {
    Ok(value) => {
        let mean: f64 = value.expected_value(&gate, 1000).unwrap();
        println!("Present; expected value {mean:.2}");
    }
    Err(e) => println!("Not enough evidence of presence: {e}"),
}
```

### Ensembles

`materialize` draws `n` samples into a container you name. The crate declares no ensemble type and
depends on neither container crate; the witness decides where the draws land.

```rust,ignore
use deep_causality_linear::{DenseVector, DenseVectorWitness};
use deep_causality_uncertain::{SampleSession, Uncertain};

let mut session = SampleSession::seeded(7);
let quantity = Uncertain::<f64>::normal(10.0, 2.0);

let ensemble: DenseVector<f64> = quantity
    .materialize::<DenseVectorWitness>(&mut session, 1000)
    .unwrap();
```

Two quantities materialised with `materialize_at` from one session are drawn at the **same**
indices, so draw *i* of one belongs with draw *i* of the other and a positional zip pairs them
correctly. `materialize` advances the session instead, which does not correlate. See the crate
documentation for the composition surface, the cartesian-traversal hazard, and the memory
arithmetic for ensembles at scale.

### The graph as an `Arrow`

Evaluating the graph at an address is a pure function, so the graph is an `Arrow` and composes
statically with downstream computation.

```rust
use deep_causality_haft::Arrow;
use deep_causality_uncertain::{SampleIndex, SampleSession, Uncertain};

let session = SampleSession::seeded(7);
let quantity = Uncertain::<f64>::normal(10.0, 2.0);

let at = SampleIndex::at(&session, 0);
assert_eq!(quantity.run(at).unwrap(), quantity.run(at).unwrap());
```

## More Examples

For larger, real-world scenarios, see the `examples` directory:

*   **GPS Navigation (`example_gps_navigation.rs`)**: Simulates GPS readings, propagates uncertainty through distance and time calculations, and makes route decisions.
  
*   **Sensor Data Processing (`example_sensor_processing.rs`)**: Processes sensor data with error handling, sensor fusion, and anomaly detection under uncertainty.

*   **Aspirin Headache Trial Analysis (`example_clinical_trial.rs`)**: Models clinical trial data with probabilistic presence through `MaybeUncertain<T>` and analyzes drug effectiveness under uncertainty.

To run an example:

```bash
cargo run --example example_gps_navigation 

cargo run --example example_sensor_processing 

cargo run --example example_clinical_trial 
```

## Benchmarks

```bash
cargo bench -p deep_causality_uncertain --bench maybe_uncertain_benchmarks

cargo bench -p deep_causality_uncertain --bench uncertain_benchmarks
```

## Acknowledgements

This crate is inspired by the blog post ["Uncertain⟨T⟩"](https://nshipster.com/uncertainty) by [@Mattt](https://github.com/mattt) and his implementation of [Uncertain for Swift](https://github.com/mattt/Uncertain).
Prior art in the [uncertain crate](https://crates.io/crates/uncertain) and the [uncertain-rs](https://crates.io/crates/uncertain-rs) crate inspired some of the implementation and examples.

The Uncertain⟨T⟩ type is based on the research presented in:

*   Bornholt, J., Mytkowicz, T., & McKinley, K. S. (2014). [**Uncertain<T>: A First-Order Type for Uncertain Data**.](https://www.microsoft.com/en-us/research/publication/uncertaint-a-first-order-type-for-uncertain-data-2) *Proceedings of the 19th International Conference on Architectural Support for Programming Languages and Operating Systems (ASPLOS '14)*. ACM, New York, NY, USA, 123-136. ([Download Paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/asplos077-bornholtA.pdf))

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
