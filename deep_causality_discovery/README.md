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

## Key Features

* **Modular Pipeline Design (CDL)**: The core of `deep_causality_discovery` is its Causal Discovery Language (CDL),
  implemented as a typestate-driven builder pattern. This ensures that causal discovery workflows are constructed in a
  valid sequence at compile-time, enhancing robustness and developer experience.
* **Flexible Data Loading**: Supports loading tabular data from various sources, including CSV and Parquet files, into
  the project's `CausalTensor` format, ready for analysis.
* **Intelligent Feature Selection**: Integrates algorithms like Minimum Redundancy Maximum Relevance (MRMR) to identify
  the most relevant features for a given target variable, reducing dimensionality and focusing causal analysis.
* **Advanced Causal Discovery**: Leverages the high-performance `surd_states` algorithm from `deep_causality_algorithms`
  to decompose causal influences into Synergistic, Unique, and Redundant components, providing a nuanced understanding
  of multi-variable interactions.
* **Actionable Causal Analysis**: Translates complex numerical results from causal discovery algorithms into
  human-readable reports, offering recommendations for building `CausaloidGraph` structures and `Causaloid` logic within
  the broader DeepCausality framework.
* **Compile-Time Safety**: The typestate pattern guarantees that each step of the causal discovery pipeline is correctly
  configured and executed, preventing common errors and ensuring a robust workflow.

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

## Installation

Add `deep_causality_discovery` to your `Cargo.toml` file:

```bash
cargo add deep_causality_discovery
```

## Usage

Here's a basic example demonstrating how to use the CDL pipeline to discover causal relationships from a CSV file:

```rust
use deep_causality_algorithms::surd::MaxOrder;
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

    // 2. Configure the Causal Discovery Language (CDL) pipeline
    let cdl_config = CdlConfig::new()
        // Data Loader: CSV with headers, comma delimiter, no skipped rows
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(true, b',', 0, None)))
        // Feature Selector: MRMR, select 2 features, target column index 3
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(2, 3)))
        // Causal Discovery: SURD, full decomposition (MaxOrder::Max), target column index 3
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            3,
        )))
        // Analysis: Define thresholds for interpreting synergistic, unique, and redundant influences
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));

    // 3. Build and run the CDL pipeline
    let discovery_process = CDL::with_config(cdl_config)
        .start(CsvDataLoader, file_path)?
        .feat_select(MrmrFeatureSelector)?
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
