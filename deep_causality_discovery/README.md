# deep_causality_discovery 

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[crates-badge]: https://img.shields.io/crates/v/deep_causality_discovery.svg

[crates-url]: https://crates.io/crates/deep_causality_discovery

[docs-badge]: https://docs.rs/deep_causality_discovery/badge.svg

[docs-url]: https://docs.rs/deep_causality_discovery

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

## Introduction

`deep_causality_discovery` provides the Causal Discovery Language (CDL) for DeepCausality: a typed pipeline that
turns observational data into causal findings. You define and run a discovery workflow; its results inform how you build a
causal model.

## Algorithms

CDL hosts two discovery algorithms as peer pipelines:

* **SURD** (Synergistic, Unique, Redundant Decomposition): decomposes, in information-theoretic terms, how a set of
  source variables drives a target, computed from a single dataset.
* **BRCD** (Bayesian Root-Cause Discovery): ranks the variables whose conditional mechanism changed between a *normal*
  and an *anomalous* regime, given a causal graph over the variables. You supply the graph as a CPDAG, or BOSS learns it
  from the normal data.

## Workflow

The CDL is a typestate builder: the type system encodes the pipeline's state, so the compiler
guarantees the stages run in a valid order. The two algorithms are **compile-time-isolated sub-pipelines** that share a
finalize tail. Calling a BRCD stage on a SURD pipeline (or the reverse) does not compile.

### 1. Build the run config (the single source of truth)

`CdlConfigBuilder` is a staged typestate builder. The compiler enforces required fields (`build()` exists only
once all are set), and `build()` checks that the referenced files exist:

* `CdlConfigBuilder::build_surd_config::<T>()` → `SurdLoaderConfig<T>`: the dataset path, target index, MRMR feature
  count, max interaction order, and analysis thresholds (optional: exclude indices, CSV options).
* `CdlConfigBuilder::build_brcd_config()` → `BrcdLoaderConfig<T>`: the normal-dataset path, anomalous-dataset path, and
  the reused algorithm `BrcdConfig<T>` (optional: CPDAG path, CSV options). No CPDAG path means the structure is learned
  via BOSS.

### 2. Run a sub-pipeline

`CdlBuilder::build_surd(&cfg)` / `CdlBuilder::build_brcd(&cfg)` seed the pipeline with the config. Every stage reads its
parameters from the config, so the chain takes no arguments:

* **SURD**: `surd_load_input → clean_data → feature_select → surd_discover → surd_analyze → finalize`
* **BRCD**: `brcd_load_input → brcd_discover → brcd_analyze → finalize`

Each stage is a method on the pipeline effect, so the chain reads top to bottom. The `CdlEffect`
monad short-circuits on the first error and carries warnings along; `print_results()` renders the final `CdlReport`
(or the error). A `CdlDiscoveryOutcome` (`Surd` or `Brcd`) holds the discovery result, and the report's `Display`
renders the matching section.

## Installation

Add `deep_causality_discovery` to your `Cargo.toml`:

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

The CPDAG file uses the typed-endpoint CSV format that `load_cpdag_csv` / `save_cpdag_csv` read and write: a `# … vertices=N`
header followed by `src,dst,mark_src,mark_dst` rows, where each mark is `Tail`, `Arrow`, or `Circle` (`Tail,Arrow` is a
directed arc, `Tail,Tail` an undirected edge).

## Error Handling

Each pipeline stage has its own error type (for example `DataLoadingError`, `FeatureSelectError`,
`CausalDiscoveryError`, `CpdagError`, `BrcdLoadError`); all convert into `CdlError`, so a caller can match on the stage
that failed.

## From Discovery to Model: Connecting CDL to DeepCausality

The discovery results map onto the building blocks of an executable DeepCausality model.

* **SURD → `CausaloidGraph` structure and logic.** Strong **unique** influences suggest direct causal links
  (`Causaloid(Source) -> Causaloid(Target)`). **Synergistic** influences indicate that multiple sources are jointly
  required to cause an effect, guiding many-to-one connections and the choice of `AggregateLogic` within a
  `CausaloidCollection` (strong synergy → `AggregateLogic::All`; unique/redundant → `AggregateLogic::Any`).
  State-dependent maps from the SURD analysis provide conditional logic for a `Causaloid`'s `causal_fn`.
* **BRCD → fault localization.** Given a normal and an anomalous window over a known service/dependency graph, BRCD ranks
  which node's mechanism changed, pointing the operator at the root cause of an incident.

## Contribution

Contributions are welcome, especially documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

