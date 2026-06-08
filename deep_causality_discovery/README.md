# deep_causality_discovery 

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/crates/v/deep_causality_discovery.svg

[crates-url]: https://crates.io/crates/deep_causality_discovery

[docs-badge]: https://docs.rs/deep_causality_discovery/badge.svg

[docs-url]: https://docs.rs/deep_causality_discovery

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## Introduction

`deep_causality_discovery` is a Rust crate that provides a Causal Discovery Language (CDL) for the DeepCausality
project. It offers a modular, type-safe pipeline to move from raw observational data to actionable causal insights. By
abstracting the statistical and algorithmic steps, it lets you define and run causal discovery workflows that ultimately
inform the construction of causal models.

## Algorithms

CDL hosts two discovery algorithms as peer pipelines:

* **SURD** (Synergistic, Unique, Redundant Decomposition): an information-theoretic decomposition of how a set of source
  variables drive a target, computed from a single dataset.
* **BRCD** (Bayesian Root-Cause Discovery): ranks the variables whose conditional mechanism changed between a *normal*
  and an *anomalous* regime, given a causal graph over the variables. The graph can be supplied as a CPDAG, or learned
  from the normal data via BOSS when none is given.

## Workflow

The CDL is a builder over Rust's typestate pattern: the pipeline's state is encoded in the type system, so the compiler
guarantees the stages run in a valid order. The two algorithms are **compile-time-isolated lineages** that converge on a
shared analyze/finalize tail. Calling a BRCD stage on a SURD pipeline (or the reverse) does not compile.

### 1. Build the run config (the single source of truth)

`CdlConfigBuilder` is a staged typestate builder. Required fields are enforced at compile time (`build()` only exists
once they are all set), and `build()` additionally verifies that the referenced files exist:

* `CdlConfigBuilder::build_surd_config::<T>()` → `SurdLoaderConfig<T>`: the dataset path, target index, MRMR feature
  count, max interaction order, and analysis thresholds (optional: exclude indices, CSV options).
* `CdlConfigBuilder::build_brcd_config()` → `BrcdLoaderConfig<T>`: the normal-dataset path, anomalous-dataset path, and
  the reused algorithm `BrcdConfig<T>` (optional: CPDAG path, CSV options). No CPDAG path means the structure is learned
  via BOSS.

### 2. Run a lineage

`CdlBuilder::build_surd(&cfg)` / `CdlBuilder::build_brcd(&cfg)` seed the pipeline with the config. Every stage reads its
parameters from the config, so the chain itself is parameterless:

* **SURD**: `surd_load_input → clean_data → feature_select → surd_discover → surd_analyze → finalize`
* **BRCD**: `brcd_load_input → brcd_discover → brcd_analyze → finalize`

Each stage is a method on the pipeline effect, so the chain reads top to bottom with no per-line wrapper. The `CdlEffect`
monad short-circuits on the first error and threads warnings through; `print_results()` renders the final `CdlReport`
(or the error). The discovery result is carried as a `CdlDiscoveryOutcome` (`Surd` or `Brcd`) and the report's `Display`
renders the matching section.

## Installation

Add `deep_causality_discovery` to your `Cargo.toml` file:

```bash
cargo add deep_causality_discovery
```

## Usage

### SURD: information-theoretic decomposition

```rust
use deep_causality_discovery::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A CSV with columns: s1, s2, s3, target.
    let config = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path("./data.csv")
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()?; // compile-checked fields + file-exists check

    CdlBuilder::build_surd(&config)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner)
        .feature_select() // MRMR, using the config's feature count + target
        .surd_discover()  // SURD-states, using the config's max order
        .surd_analyze()   // using the config's thresholds
        .finalize()
        .print_results();

    Ok(())
}
```

### BRCD: root-cause ranking from two regimes

```rust
use deep_causality_discovery::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path("./normal.csv")
        .with_anomalous_path("./anomalous.csv")
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .with_cpdag_path("./cpdag.csv") // optional; omit to learn the graph via BOSS
        .build()?;

    CdlBuilder::build_brcd(&config)
        .brcd_load_input() // loads both datasets (+ CPDAG) inside the pipeline
        .brcd_discover()
        .brcd_analyze()
        .finalize()
        .print_results();

    Ok(())
}
```

The CPDAG file is the typed-endpoint CSV format `load_cpdag_csv` / `save_cpdag_csv` read and write: a `# … vertices=N`
header followed by `src,dst,mark_src,mark_dst` rows, where each mark is `Tail`, `Arrow`, or `Circle` (`Tail,Arrow` is a
directed arc, `Tail,Tail` an undirected edge).

## Error Handling

The crate defines a specific error type for each stage of the pipeline (for example `DataLoadingError`,
`FeatureSelectError`, `CausalDiscoveryError`, `CpdagError`, `BrcdLoadError`), all funneled into `CdlError`. This allows
precise identification and handling of issues, and the `CdlEffect` monad short-circuits on the first error.

## From Discovery to Model: Connecting CDL to DeepCausality

The `deep_causality_discovery` crate acts as a bridge, transforming observational data into the foundational elements for
building executable causal models with the DeepCausality library.

* **SURD → `CausaloidGraph` structure and logic.** Strong **unique** influences suggest direct causal links
  (`Causaloid(Source) -> Causaloid(Target)`). **Synergistic** influences indicate that multiple sources are jointly
  required to cause an effect, guiding many-to-one connections and the choice of `AggregateLogic` within a
  `CausaloidCollection` (strong synergy → `AggregateLogic::All`; unique/redundant → `AggregateLogic::Any`).
  State-dependent maps from the SURD analysis provide conditional logic for a `Causaloid`'s `causal_fn`.
* **BRCD → fault localization.** Given a normal and an anomalous window over a known service/dependency graph, BRCD ranks
  which node's mechanism changed, pointing the operator at the root cause of an incident rather than the collateral.

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC
