# 🔬 deep_causality_algorithms 🔬

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue
[crates-url]: https://crates.io/crates/deep_causality_algorithms
[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue
[docs-url]: https://docs.rs/deep_causality_algorithms/latest/deep_causality_algorithms/
[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg
[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE
[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

A collection of computational causality algorithms used in the [DeepCausality](https://github.com/deepcausality-rs/deep_causality) project. This crate provides tools for analyzing and decomposing causal relationships in complex systems.

The cornerstone of this crate is `surd_states`, a high-performance Rust implementation of the **SURD-states algorithm**. Based on the paper "Observational causality by states and interaction type for scientific discovery" (martínezsánchez2025), this algorithm decomposes the mutual information between a set of source variables and a target variable into its fundamental components: **S**ynergistic, **U**nique, and **R**edundant (**SURD**).

This decomposition allows for a deep, nuanced understanding of causal structures, moving beyond simple correlations to reveal the nature of multi-variable interactions.

## Key Features

*   **Faithful & Performant Implementation**: A high-performance, mathematically faithful Rust port of the SURD-states algorithm, optimized for speed and memory efficiency.
*   **Rich Causal Decomposition**: Decomposes the total causal influence into:
    *   **Redundant (R)**: Overlapping information provided by multiple sources.
    *   **Unique (U)**: Information provided by a single source independently.
    *   **Synergistic (S)**: New information that emerges only from the combination of sources.
*   **State-Dependent Analysis**: Provides detailed state-dependent maps that reveal how causal influences change based on the system's current state.
*   **Information Leak Quantification**: Explicitly calculates the "information leak," which quantifies the influence of unobserved variables or inherent randomness in the system.
*   **Performance Optimized**:
    *   **Algorithmic Capping**: Use the `MaxOrder` enum to limit the analysis to a tractable number of interactions (e.g., pairwise), reducing complexity from exponential `O(2^N)` to polynomial `O(N^k)`.
    *   **Parallel Execution**: When compiled with the `parallel` feature flag, the main decomposition loop runs in parallel across all available CPU cores using `rayon`.

## Installation

```bash
cargo add deep_causality_algorithms
```

## Usage

The primary function is `surd_states`, which takes a `CausalTensor` representing a joint probability distribution and returns a `SurdResult`.

```rust
use deep_causality_algorithms::{surd_states, MaxOrder};
use deep_causality_data_structures::CausalTensor;

// Create a joint probability distribution for a target and 2 source variables.
// Shape: [target_states, source1_states, source2_states] = [2, 2, 2]
let data = vec![
    0.1, 0.2, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
    0.0, 0.2, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
    0.3, 0.0, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
    0.1, 0.1, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
];
let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();

// Perform a full decomposition (k=N=2)
let full_result = surd_states(&p_raw, MaxOrder::Max).unwrap();

// Print the detailed decomposition
println!("{}", &full_result);

// Access specific results
println!("Information Leak: {:.3}", full_result.info_leak());

// Synergistic information for the pair of variables {1, 2}
if let Some(synergy) = full_result.synergistic_info().get(&vec![1, 2]) {
    println!("Synergistic Info for {{1, 2}}: {:.3}", synergy);
}
```

## From Discovery to Model: Connecting SURD to DeepCausality

The `surd_states` algorithm serves as a bridge from observational data to executable causal models with the DeepCausality.

### 1. Mapping Causal Links to `CausaloidGraph` Structure

The aggregate SURD results inform the structure of the `CausaloidGraph`.
*   A strong **unique** influence from `S1` to `T` suggests a direct edge: `Causaloid(S1) -> Causaloid(T)`.
*   A strong **synergistic** influence from `S1` and `S2` onto `T` suggests a many-to-one connection where `Causaloid(S1)` and `Causaloid(S2)` both point to `Causaloid(T)`.
*   A high **information leak** suggests that the `Causaloid` for `T` should model a high degree of internal randomness or dependency on an unobserved `Context`.

### 2. Mapping State-Dependency to `Causaloid` Logic

The state-dependent maps provide the exact conditional logic for a `Causaloid`'s `causal_fn`. For example, if SURD shows that `S1`'s influence on `T` is strong only when `S1 > 0`, this condition can be programmed directly into the `Causaloid`.

### 3. Modeling Multiple Causes with `CausaloidCollection`

SURD's ability to detect multi-causal relationships is perfectly complemented by the `CausaloidCollection`, which models the interplay of multiple factors. The SURD results guide the choice of the collection's `AggregateLogic`:
*   **Strong SYNERGY** (e.g., A and B are required for C) maps to `AggregateLogic::All` (Conjunction).
*   **Strong UNIQUE or REDUNDANT** influences (e.g., A or B can cause C) maps to `AggregateLogic::Any` (Disjunction).
*   **Complex mixed influences** (e.g., any two of three factors cause C) maps to `AggregateLogic::Some(k)` (Threshold).

In summary, `surd_states` provides the data-driven evidence to identify multi-causal structures, and the DeepCausality primitives provide the formal mechanisms to build an executable model of that precise structure.

## Example: Decomposing Causal Structure

The crate includes a detailed example (`example_surd`) that demonstrates how to use the `surd_states` algorithm and, more importantly, how to interpret its rich output. It runs through several test cases with different underlying causal structures (e.g., synergistic, noisy, random) and explains what each part of the output means.

To run the example:
```bash
cargo run --example example_surd
```
For a detailed walkthrough of the output, see the [example's README](examples/README.md).

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
