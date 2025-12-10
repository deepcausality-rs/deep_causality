# 🔍 deep_causality_discovery 🔍

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
project. It offers a powerful, modular, and type-safe pipeline to move from raw observational data to actionable causal
insights. By abstracting complex statistical and algorithmic steps, it enables users to define and execute causal
discovery workflows with ease, ultimately informing the construction of causal models.

## Workflow 

The core of the CDL is a builder pattern that uses Rust's typestate pattern.
This means the pipeline's state is encoded in the type system, which guarantees
at compile-time that the steps are executed in a valid sequence.

The workflow consists of the following sequential stages:

1. Configuration (`CdlConfig`):
  * The entire pipeline is configured using the CdlConfig struct.
  * This struct uses a builder pattern (with_* methods) to set up
    configurations for each stage, such as data loading, feature selection,
    the discovery algorithm, and analysis thresholds.

2. Initialization (`CDL<NoData>`):
  * The pipeline starts in the NoData state, created via CDL::new() or
    CDL::with_config(config).

3. Data Loading (`load_data`):
  * Transition: NoData -> WithData
  * Action: Loads data from a source (e.g., CSV, Parquet) into a
    CausalTensor<f64>.
  * Implementations: CsvDataLoader, ParquetDataLoader.

4. Data Cleaning (`clean_data`, Optional):
  * Transition: WithData -> WithCleanedData
  * Action: Explicitly cleans data.
  * Implementation: `OptionNoneDataCleaner` (converts `NaN` to `None`).

5. Feature Selection (`feature_select`):
  * Transition: WithData or WithCleanedData -> WithFeatures
  * Action: Selects relevant features.
  * Implementation: MRMR Feature Selector.

6. Causal Discovery (`causal_discovery`):
  * Transition: WithFeatures -> WithCausalResults
  * Action: Executes the core causal discovery algorithm on the selected
    features.
  * Implementation: SurdCausalDiscovery, which uses the surd_states_cdl
    algorithm to decompose causal influences into Synergistic, Unique, and
    Redundant (SURD) components. The output is a SurdResult<f64>.

7. Analysis (`analyze`):
  * Transition: WithCausalResults -> WithAnalysis
  * Action: Interprets the raw numerical output from the discovery algorithm
    into a human-readable analysis. It uses thresholds from AnalyzeConfig to
    classify the strength of causal influences.
  * Implementation: SurdResultAnalyzer, which generates a report with
    recommendations (e.g., "Strong unique influence... Recommended: Direct
    edge in CausaloidGraph").

8. Finalization (`finalize`):
  * Transition: WithAnalysis -> Finalized
  * Action: Formats the analysis report into a final output string.
  * Implementation: ConsoleFormatter, which prepares the text for printing.

9. Report (`print_results`):
  * The `print_results()` method is called on the final `CdlEffect` to display
    errors, warnings, or the successful analysis to the console.

## Installation

Add `deep_causality_discovery` to your `Cargo.toml` file:

```bash
cargo add deep_causality_discovery
```

## Usage

Here's a basic example demonstrating how to use the CDL pipeline to discover causal relationships from a CSV file:

```rust
use deep_causality_discovery::*;
use std::{fs::File, io::Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare test data
    let csv_data = "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6";
    let file_path = "./test_data.csv";
    let mut file = File::create(file_path)?;
    file.write_all(csv_data.as_bytes())?;
    
    let target_index = 3;

    // 2. Run the CDL pipeline (Monadic Flow)
    let result_effect = CdlBuilder::build()
        // Load Data (implicitly creates Config)
        .bind(|cdl| cdl.load_data(file_path, target_index, vec![]))
        // Explicitly Clean Data (Optional but recommended)
        .bind(|cdl| cdl.clean_data(OptionNoneDataCleaner))
        // Feature Selection
        .bind(|cdl| {
            cdl.feature_select(|tensor| {
                 mrmr_features_selector(tensor, 3, target_index)
            })
        })
        // Causal Discovery
        .bind(|cdl| {
            cdl.causal_discovery(|tensor| {
                surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)
            })
        })
        // Analyze & Finalize
        .bind(|cdl| cdl.analyze())
        .bind(|cdl| cdl.finalize());

    // 3. Output results
    result_effect.print_results();

    // 4. Cleanup
    std::fs::remove_file(file_path)?;
    Ok(())
}
```

## Error Handling

The crate employs a comprehensive error handling strategy, defining specific error types for each stage of the CDL
pipeline (e.g., `DataError`, `FeatureSelectError`, `CausalDiscoveryError`). This allows for precise identification and
handling of issues, ensuring robust and reliable causal discovery workflows.

## From Discovery to Model: Connecting CDL to DeepCausality

The `deep_causality_discovery` crate acts as a crucial bridge, transforming observational data into the foundational
elements for building executable causal models with the DeepCausality library. The insights gained from the SURD-states
algorithm directly inform the design of your `CausaloidGraph` and the internal logic of individual `Causaloid`s:

* **Structuring the `CausaloidGraph`**: Strong **unique** influences suggest direct causal links (
  `Causaloid(Source) -> Causaloid(Target)`). Significant **synergistic** influences indicate that multiple sources are
  jointly required to cause an effect, guiding the creation of many-to-one connections.
* **Defining `Causaloid` Logic**: State-dependent maps from the SURD analysis provide precise conditional logic for a
  `Causaloid`'s `causal_fn`, allowing you to programmatically capture how causal influences vary with system states.
* **Modeling Multi-Causal Interactions**: The detection of synergistic, unique, and redundant influences directly
  informs the choice of `AggregateLogic` within `CausaloidCollection`s. For instance, strong synergy might map to
  `AggregateLogic::All` (conjunction), while unique or redundant influences could suggest `AggregateLogic::Any` (
  disjunction).

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
