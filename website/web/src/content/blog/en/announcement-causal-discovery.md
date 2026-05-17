---
title: "DeepCausality Introduces the Causal Discovery Language (CDL)"
description: "This post introduces deep_causality_discovery, a new crate providing a type-safe, monadic pipeline for discovering causal structures in observational data."
date: 2025-12-12
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

The DeepCausality project announces the release of `deep_causality_discovery`, a new crate designed to bridge the gap between raw data and causal models. This release introduces the **Causal Discovery Language (CDL)**, a Domain Specific Language (DSL) that allows researchers and engineers to define robust, type-safe pipelines for extracting causal insights from observational data.

## The Problem: Causal Discovery

Building a causal model requires knowing *structure*: which variables influence which, and how? In complex systems, such as in finance, biology, or systems engineering, this structure is rarely obvious.

Traditionally, finding these links involves a messy process of "data munging":
1.  Loading CSVs or Parquet files with ad-hoc scripts.
2.  Manually cleaning `NaN`s and outliers.
3.  Running statistical tests (like Granger Causality or Mutual Information) in isolation.
4.  Subjectively interpreting the numerical output to draw a graph.

This approach lacks structure, formal verification, is prone to pipeline errors (e.g., analyzing uncleaned data), and often fails to distinguish between **synergy** (variables that act together) and **redundancy** (variables that mimic each other).

## The Solution: The CDL Pipeline

The **Causal Discovery Language (CDL)** solves this by treating the discovery process as a **Monadic Pipeline**.

Built on the [DeepCausality HAFT](/blog/announcement-haft-hkt/) foundation, the CDL uses a type-safe Builder pattern to ensure that every stage of the discovery process happens in the correct order. You cannot run a discovery algorithm before selecting features; you cannot select features before loading data.

The pipeline is wrapped in a `CdlEffect`, a monadic structure that handles:
*   **State Propagation**: Passing the data tensor between stages.
*   **Error Handling**: Short-circuiting on IO or algorithmic errors.
*   **Audit Logging**: accumulating warnings and decisions made at each step.

## The CDL Pipeline Stages

The CDL workflow consists of sequential stages, each strictly typed:

1.  **Data Loading**: Ingests data from CSV or Parquet sources into a high-performance `CausalTensor`.
2.  **Data Cleaning**: Applies strategies like `OptionNoneDataCleaner` to explicitly handle missing values or outliers without silent failures.
3.  **Feature Selection**: Uses algorithms like **MRMR (Max-Relevance Min-Redundancy)** to identify the subset of variables most relevant to your target, filtering out noise early.
4.  **Causal Discovery**: The core engine. It currently leverages the **SURD (Synergistic, Unique, Redundant Decomposition)** algorithm. Unlike simple correlation, SURD quantifies how much information a variable contributes *uniquely*, how much is *redundant* with others, and how much arises only from *synergistic* interaction.
5.  **Analysis**: Interprets the raw information bits from SURD against configurable thresholds to generate semantic recommendations (e.g., "Strong unique influence detected").
6.  **Finalization**: Formats the results into a human-readable report or a structured artifact for model generation.

## Code Example

Here is how the CDL looks in practice. Notice the clean, functional flow enabled by the `.bind()` operator.

```rust
use deep_causality_discovery::*;
use std::{fs::File, io::Write};

fn main() {
    let file_path = "raw_data.csv";
    let target_index = 3; // We want to find causes for this column

    // The CDL Pipeline
    let result_effect = CdlBuilder::build()
        // 1. Load Data
        .bind(|cdl| cdl.load_data(&file_path, target_index, vec![]))
        // 2. Clean Data (Handle NaNs explicitly)
        .bind(|cdl| cdl.clean_data(OptionNoneDataCleaner))
        // 3. Feature Selection (MRMR)
        .bind(|cdl| {
            cdl.feature_select(|tensor| {
                // Select top 3 features relevant to target_index
                mrmr_features_selector(tensor, 3, target_index)
            })
        })
        // 4. Causal Discovery (SURD)
        .bind(|cdl| {
            cdl.causal_discovery(|tensor| {
                // Decompose influences up to maximum order interactions between all variables
                surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)
            })
        })
        
        // 5. Analyze & Format
        .bind(|cdl| cdl.analyze())
        .bind(|cdl| cdl.finalize());

    // Output the report
    result_effect.print_results();
}
```

The result is a comprehensive report detailing not just *if* variables are related, but *how*—identifying whether a driver acts alone (Unique) or requires other conditions to be met (Synergistic). These insights can then be directly mapped to `Causaloid` logic in the main DeepCausality library.

For larger data sets (over half a million records), the parallel flag executes all algorithms in parallel on all available 
CPU cores. However, CDL is not ready to handle big data volume with millions or billions of rows as this would require enhanced
 hardware acceleration (i.e. on GPU) to parallelize especially the SURD algorithm further. In practice, this might be less
of an issue because data scientists routinely work on smaller data samples before validating a model on a larger dataset. 

## Current State

The `deep_causality_discovery` has resulted from ongoing research in streamlining causal discovery and modeling. Despite its
early stage, it offers a promising path towards streamlined causal discovery. The design of the CDL embraces extensibility,
meaning a CDL user can write a custom data cleaner and inject it into the clean_data step. Also, medical and life science 
data often require custom data loaders. In that case, a user can write a custom data loader that returns a CausalTensor
and then construct the CDL pipeline using the load_tensor() as a starting point to bypass the default data loader. 
That said, most testing and usage will expose some rough edge cases. If you encounter any issues, please an issue.. 

Get Started with Causal Discovery today!

*   Explore the [code on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery).
*   Run the [examples](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery/examples).
*   Join the [community](https://www.deepcausality.com/community/).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
