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

4. Data Cleaning & Feature Selection (`feature_select`):
  * Transition: WithData -> WithFeatures
  * Action: This is a mandatory step that prepares the data and selects the
    most relevant features for analysis.
    * First, it internally uses OptionNoneDataCleaner to convert the tensor
      to CausalTensor<Option<f64>>, which handles missing or NaN values by
      converting them to None. This is crucial for robust statistical
      analysis in the subsequent steps.
    * Then, it applies a feature selection algorithm to reduce
      dimensionality.
  * Implementation: MrmrFeatureSelector (Minimum Redundancy Maximum
    Relevance).

5. Causal Discovery (`causal_discovery`):
  * Transition: WithFeatures -> WithCausalResults
  * Action: Executes the core causal discovery algorithm on the selected
    features.
  * Implementation: SurdCausalDiscovery, which uses the surd_states_cdl
    algorithm to decompose causal influences into Synergistic, Unique, and
    Redundant (SURD) components. The output is a SurdResult<f64>.

6. Analysis (`analyze`):
  * Transition: WithCausalResults -> WithAnalysis
  * Action: Interprets the raw numerical output from the discovery algorithm
    into a human-readable analysis. It uses thresholds from AnalyzeConfig to
    classify the strength of causal influences.
  * Implementation: SurdResultAnalyzer, which generates a report with
    recommendations (e.g., "Strong unique influence... Recommended: Direct
    edge in CausaloidGraph").

7. Finalization (`finalize`):
  * Transition: WithAnalysis -> Finalized
  * Action: Formats the analysis report into a final output string.
  * Implementation: ConsoleFormatter, which prepares the text for printing.

8. Execution (`build` and `run`):
  * The build() method is called on a Finalized pipeline to create an
    executable CDLRunner.
  * The run() method on the CDLRunner executes the process and returns the
    final ProcessFormattedResult.

## Installation

Add `deep_causality_discovery` to your `Cargo.toml` file:

```bash
cargo add deep_causality_discovery
```

## Usage

Here's a basic example demonstrating how to use the CDL pipeline to discover causal relationships from a CSV file:

```rust
use deep_causality_discovery::*;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare test data (create a dummy CSV file)
    let csv_data =
        "s1,s2,s3,target
1.0,2.0,3.0,1.5
2.0,4.1,6.0,3.6
3.0,6.2,9.0,5.4
4.0,8.1,12.0,7.6";
    let file_path = "./test_data.csv";
    let mut file = File::create(file_path)?;
    file.write_all(csv_data.as_bytes())?;

    // 2. Build the CDL configuration
    let cdl_config = CdlConfig::new()
        // Define the data loader as CSV file loader and the corresponding default CSV config
        .with_data_loader(DataLoaderConfig::Csv(CsvConfig::default()))
        // Define the feature selected as MRMR and set its parameters
        .with_feature_selector(FeatureSelectorConfig::Mrmr(MrmrConfig::new(2, 3)))
        // Define the causal discovery as SURD and set its parameters
        .with_causal_discovery(CausalDiscoveryConfig::Surd(SurdConfig::new(Max, 3)))
        // Define the analysis of the SURD results and set its parameters
        .with_analysis(AnalyzeConfig::new(0.1, 0.1, 0.1));

    // 3. Build and run the CDL pipeline
    let discovery_process = CDL::with_config(cdl_config)
        .load_data(CsvDataLoader, &file_path)?
        .feature_select(MrmrFeatureSelector)?
        .causal_discovery(SurdCausalDiscovery)?
        .analyze(SurdResultAnalyzer)?
        .finalize(ConsoleFormatter)?
        .build()?;

    let result = discovery_process.run()?;
    println!("Causal Discovery Result: {}", result);

    // 4. Clean up the dummy file
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
