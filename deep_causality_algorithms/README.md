# deep_causality_algorithms

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_algorithms

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_algorithms/latest/deep_causality_algorithms/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

Computational causality algorithms for the [DeepCausality](https://github.com/deepcausality-rs/deep_causality)
project: causal decomposition (SURD), root-cause localization (BRCD) and feature selection (mRMR).

`surd_states` implements the **SURD-states algorithm** from "Observational causality by states and interaction type for
scientific discovery" (Martínez-Sánchez and Lozano-Durán, 2025). It decomposes the mutual information between a set of
source variables and a target variable into **S**ynergistic, **U**nique, and **R**edundant (**SURD**) components, which
shows how multiple variables interact rather than only whether they correlate.

`brcd` implements **Bayesian Root Cause Discovery (BRCD)** from "Root Cause Analysis of Failures in Microservices via
Bayesian Root Cause Discovery" (Lee, Zhou, and Kocaoglu, 2026). SURD decomposes how a set of sources jointly influence a
target. BRCD takes a normal dataset, an anomalous dataset, and a causal graph over the same variables, scores every
candidate root-cause set with a Bayesian posterior, and ranks the candidates from most to least probable.

## Key Features

* **SURD-states port**: A Rust port of the SURD-states algorithm that follows the published mathematics.
* **Causal Decomposition**: Decomposes the total causal influence into:
    * **Redundant (R)**: Overlapping information provided by multiple sources.
    * **Unique (U)**: Information provided by a single source independently.
    * **Synergistic (S)**: New information that emerges only from the combination of sources.
* **State-Dependent Analysis**: State-dependent maps show how causal influences change with the system's current state.
* **Information Leak Quantification**: Calculates the "information leak": the influence of unobserved variables or
  inherent randomness in the system.
* **Incomplete Data Handling (CDL Variant)**: `surd_states_cdl` handles missing or undefined probability values (`None`
  in `CausalTensor<Option<f64>>`) by ignoring `None` values in calculations and propagating uncertainty, so datasets
  with gaps still yield causal results.
* **Minimum Redundancy Maximum Relevance (mRMR) Feature Selection**: Selects features that are maximally relevant to a target variable and minimally redundant among themselves, and returns them ranked with normalized importance scores (between 0.0 and 1.0).
* **Bayesian Root Cause Discovery (BRCD)**: `brcd_run` localizes the root cause of an anomaly. Given two aligned datasets (a normal regime and an anomalous regime) and a CPDAG over the shared variables, it augments the graph with a soft-intervention F-node, scores each candidate root-cause set with a plug-in ridge-Gaussian (continuous) or Dirichlet (discrete) likelihood, and ranks the candidates by their posterior probability `p(R | D)`. It ranks on the log-posterior directly, so the ranking stays stable when a single fault dominates.
* **Performance**:
    * **Algorithmic Capping**: The `MaxOrder` enum limits the analysis to a tractable number of interactions (
      e.g., pairwise), reducing complexity from exponential `O(2^N)` to polynomial `O(N^k)`.
    * **Parallel Execution**: With the `parallel` feature flag, the main SURD decomposition loop, the mRMR feature selection loops, and BRCD's per-candidate structural enumeration and per-family likelihood scoring run across all available CPU cores using `rayon`. Candidates and families are independent, and each candidate derives its own RNG seed, so parallel and sequential results are identical.

## Installation

```bash
cargo add deep_causality_algorithms
```

## Usage

The primary function is `surd_states`, which takes a `CausalTensor` representing a joint probability distribution and
returns a `SurdResult`.

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

### Handling Incomplete Data with `surd_states_cdl`

`surd_states_cdl` handles datasets with missing values. It operates on `CausalTensor<Option<f64>>`, ignores `None`
values in calculations, and propagates uncertainty.

```rust
use deep_causality_algorithms::{surd_states_cdl, MaxOrder};
use deep_causality_data_structures::CausalTensor;

// Create a joint probability distribution with missing data (None values).
// Shape: [target_states, source1_states, source2_states] = [2, 2, 2]
let data_with_nones = vec![
    Some(0.1), Some(0.2), // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
    None,      Some(0.2), // P(T=0, S1=1, S2=0) is missing, P(T=0, S1=1, S2=1)
    Some(0.3), None,      // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1) is missing
    Some(0.1), Some(0.1), // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
];
let p_raw_with_nones = CausalTensor::new(data_with_nones, vec![2, 2, 2]).unwrap();

// Perform a full decomposition with None handling
let full_result_cdl = surd_states_cdl(&p_raw_with_nones, MaxOrder::Max).unwrap();

// Print the detailed decomposition
println!("CDL Result: {}", &full_result_cdl);

// Access specific results
println!("CDL Information Leak: {:.3}", full_result_cdl.info_leak());
```

### Minimum Redundancy Maximum Relevance (mRMR) Feature Selection

mRMR selects the subset of features that are maximally relevant to a target variable and minimally redundant among
themselves, which reduces dimensionality and focuses causal analysis on the most informative variables. The
implementation follows the formulation of Zhao, Anand, and Wang (2019) and returns a ranked list of features with
normalized importance scores (between 0.0 and 1.0).

```rust
use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;

let data = vec![
    10.0, 12.0, 1.0, 11.0,
    20.0, 21.0, 5.0, 22.0,
    30.0, 33.0, 2.0, 31.0,
    40.0, 40.0, 8.0, 43.0,
    50.0, 55.0, 3.0, 52.0,
];
let mut tensor = CausalTensor::new(data, vec![5, 4]).unwrap();

// Select 2 features, with the target variable in column 3.
let selected_features_with_scores = mrmr_features_selector(&mut tensor, 2, 3).unwrap();

println!("Selected Features and Scores: {:?}", selected_features_with_scores);
```

A higher mRMR score (and thus a higher normalized importance score)
means the feature is relevant to the target and adds information that the
features already chosen do not carry. The score measures a feature's
contribution to predicting the target within the selected feature set.

### Bayesian Root Cause Discovery (BRCD)

SURD and mRMR describe how variables influence a target; BRCD asks which variable most likely caused an observed
anomaly. `brcd_run` takes a normal dataset, an anomalous dataset, a CPDAG over
the shared variables, and a `BrcdConfig`. It returns a `BrcdResult` whose `ranks()` list the candidate root-cause sets
from most to least probable, and whose `top()` returns the single most probable set.

```rust
use deep_causality_algorithms::brcd::{brcd_run, BrcdConfig};
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

// A linear-Gaussian chain X -> Y -> Z. `y_intercept` shifts Y's own mechanism.
fn chain(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let noise = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = noise.sample(&mut rng);
        let y = y_intercept + 1.5 * x + noise.sample(&mut rng);
        let z = 2.0 * y + noise.sample(&mut rng);
        data.extend_from_slice(&[x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

// Two aligned regimes over the variables [X, Y, Z]. Only Y's mechanism changes
// between them (its intercept jumps), so Y is the true root cause.
let normal = chain(120, 0.0, 1);
let anomalous = chain(120, 4.0, 2);

// The CPDAG over the three variables: the undirected chain X — Y — Z.
let unit = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
let mut cpdag = MixedGraph::new(3, unit, 0).unwrap();
cpdag.add_undirected(0, 1).unwrap();
cpdag.add_undirected(1, 2).unwrap();

// Run BRCD with the continuous (ridge-Gaussian) family, seed 7, single cause.
let result = brcd_run(&normal, &anomalous, &cpdag, &BrcdConfig::continuous(7)).unwrap();

println!("{result}");
println!("Top root cause: {:?}", result.top()); // Some([1]) = Y
```

`BrcdConfig::continuous(seed)` selects the ridge-Gaussian family for continuous data; `BrcdConfig::discrete(seed)`
selects the Dirichlet family for categorical data. The `num_root_causes` field sets how many simultaneous root causes a
candidate set holds (`k`).

## From Discovery to Model: Connecting SURD to DeepCausality

`surd_states` connects observational data to executable DeepCausality causal models.

### 1. Mapping Causal Links to `CausaloidGraph` Structure

The aggregate SURD results inform the structure of the `CausaloidGraph`.

* A strong **unique** influence from `S1` to `T` suggests a direct edge: `Causaloid(S1) -> Causaloid(T)`.
* A strong **synergistic** influence from `S1` and `S2` onto `T` suggests a many-to-one connection where `Causaloid(S1)`
  and `Causaloid(S2)` both point to `Causaloid(T)`.
* A high **information leak** suggests that the `Causaloid` for `T` should model a high degree of internal randomness or
  dependency on an unobserved `Context`.

### 2. Mapping State-Dependency to `Causaloid` Logic

The state-dependent maps supply the conditional logic for a `Causaloid`'s `causal_fn`. If SURD shows that `S1`'s
influence on `T` is strong only when `S1 > 0`, that condition goes directly into the `Causaloid`.

### 3. Modeling Multiple Causes with `CausaloidCollection`

SURD detects multi-causal relationships; a `CausaloidCollection` models the interplay of multiple factors. The SURD
results guide the choice of the collection's `AggregateLogic`:

* **Strong SYNERGY** (e.g., A and B are required for C) maps to `AggregateLogic::All` (Conjunction).
* **Strong UNIQUE or REDUNDANT** influences (e.g., A or B can cause C) maps to `AggregateLogic::Any` (Disjunction).
* **Complex mixed influences** (e.g., any two of three factors cause C) maps to `AggregateLogic::Some(k)` (Threshold).

`surd_states` supplies the evidence that identifies a multi-causal structure; the DeepCausality primitives turn that
structure into an executable model.

## Example: Decomposing Causal Structure

The `example_surd` example shows how to call `surd_states` and how to interpret its output. It runs several cases
with different underlying causal structures (e.g., synergistic, noisy, random) and explains each part of the output.

To run the example:

```bash
cargo run --example example_surd
```

For a detailed walkthrough of the output, see the [example's README](examples/README.md).

## References

The algorithms in this crate are Rust implementations of the following published work. Credit for the methods belongs to
their original authors.

* **SURD** — Álvaro Martínez-Sánchez and Adrián Lozano-Durán. "Observational causality by states and interaction type for
  scientific discovery." arXiv:2505.10878 (2025). <https://arxiv.org/abs/2505.10878>
* **mRMR** — Zhenyu Zhao, Radhika Anand, and Mallory Wang. "Maximum Relevance and Minimum Redundancy Feature Selection
  Methods for a Marketing Machine Learning Platform." arXiv:1908.05376 (2019). <https://arxiv.org/abs/1908.05376>
* **BRCD** — Kenneth Lee, Zihan Zhou, and Murat Kocaoglu. "Root Cause Analysis of Failures in Microservices via Bayesian
  Root Cause Discovery." International Conference on Machine Learning (ICML), 2026.
  <https://icml.cc/virtual/2026/poster/65359>

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
