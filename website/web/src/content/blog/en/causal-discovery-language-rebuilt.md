---
title: "The Causal Discovery Language, Rebuilt: Two Algorithms, One Type-Safe Pipeline"
description: "Six months ago the Causal Discovery Language shipped with one algorithm and a single linear pipeline. Adding a second algorithm, BRCD, exposed four assumptions the first design had quietly baked in. Rather than patch around them, the CDL was rebuilt from the ground up into two compile-time-isolated sub-pipelines, a design that is more expressive, more concise, and more powerful than before."
date: 2026-06-09
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## The original idea

Building a causal model requires knowledge of causal structure: which variables influence which, and how. In a real system, that structure is rarely obvious, and finding it is far from trivial. You load a CSV with an ad-hoc script, clean the missing values by hand, run several statistical tests in isolation, then try to isolate confounders, construct a causal graph, and verify it.

Six months ago, `deep_causality_discovery` shipped the **Causal Discovery Language (CDL)** to streamline the chore. It treated discovery as a typed pipeline: a chain of stages, with the order enforced by the type system, so you could not select features before loading data or discover structure before selecting features. The chain ran inside `CdlEffect`, a small monad that threaded the data tensor between stages, short-circuited on the first error, and accumulated warnings. The first causal discovery algorithm was SURD, an information-theoretic decomposition that separates what a variable explains *uniquely* from what it shares *redundantly* with others and what emerges only from their *synergy*. For that job, the design was clean and it worked.

## What the first design got right

The good parts have survived, and they are worth naming. The monadic pipeline was the right backbone: a fluent chain that reads top to bottom, carrying its error handling and audit log as it goes. The typestate ordering was the right guarantee: invalid stage orders failed to compile. SURD plus MRMR feature selection was a genuinely useful pairing. The otherwise complex and cumbersome process of causal discovery became a simple pipeline:

```rust
let result = CdlBuilder::build()
    .bind(|cdl| cdl.load_data(&file_path, target_index, vec![]))
    .bind(|cdl| cdl.clean_data(OptionNoneDataCleaner))
    .bind(|cdl| cdl.feature_select(|tensor| mrmr_features_selector(tensor, 3, target_index)))
    .bind(|cdl| cdl.causal_discovery(|tensor| surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)))
    .bind(|cdl| cdl.analyze())
    .bind(|cdl| cdl.finalize());
```
The rigidity was harder to see, because nothing in the design pointed at it. Read that chain with one algorithm in mind and it is fine. Read it asking what *any* algorithm must accept, and four assumptions surface:

1. There is exactly one dataset, carried as one tensor through every stage.
2. Every algorithm wants MRMR feature selection and `Option` cleaning before discovery.
3. There is one discovery result type, `SurdResult`, hardcoded through the whole typestate.
4. Structure is something to *discover*, never something you *supply*.

While SURD was the only algorithm, all four assumptions were fine. SURD reads one dataset; SURD benefits from feature selection; SURD produces a `SurdResult`; SURD takes no graph. An assumption that the only algorithm never violates is invisible. It is not examined, because nothing forces the question.

## The algorithm that did not fit

The question arrived with a second algorithm. **BRCD** (Bayesian Root-Cause Discovery) answers a different kind of causal query. Given a system that was healthy and is now failing, BRCD ranks the variables whose conditional mechanism changed between the two regimes, pointing an operator at the root cause of an incident. It is the discovery method you want when a microservice falls over and forty other telemetry metrics move all at once.

BRCD violates all four assumptions at once:

1. **It needs two datasets, not one.** A normal window and an anomalous window, aligned column for column.
2. **It must not pass through feature selection.** MRMR drops and reorders columns. BRCD ranks variables by index and reads a graph whose vertices are those same indices, so reindexing the columns would silently desync the ranks and the graph. 
3. **It produces a different result.** A `BrcdResult`, a ranked posterior over candidate root-cause sets, not a synergy-unique-redundant decomposition.
4. **It can take a supplied graph.** BRCD accepts an optional CPDAG over the variables, a causal structure you already know from the service topology; when none is given, it learns one from the normal data via the BOSS algorithm before ranking.



The tempting fix is to add an optional second dataset to the carrier. Add a flag to skip feature selection. Widen the result type to an enum. Thread an optional graph through every stage. Each change is small on its own.

Together all those steps are corrosive. A single pipeline that carries an optional second dataset, an optional graph, a skippable feature-selection stage, and a polymorphic result is a pipeline whose type no longer says what it does. SURD would carry fields it never reads; BRCD would inherit a stage it must refuse at runtime; a reader could write a chain that loads two datasets, selects features, and then asks for a root-cause ranking, and nothing would stop them until the numbers came out wrong. The assumptions were not a layer of grime to wipe off. They were load-bearing, and they were specific to SURD. 
The honest move was to rebuild.

## The rebuild

The new CDL hosts SURD and BRCD as type-encoded **peers**. Within the discovery pipeline, the chosen configuration selects one of two sub-pipelines, one for SURD and one for BRCD; the two share no state until they converge at `finalize`, and the compiler keeps them apart.

```
build_surd(&cfg) ► load ► clean ► feature_select ► discover ► analyze ─┐
                                                                       ├► finalize ► report
build_brcd(&cfg) ► load ───────────────────────────► discover ► analyze ─┘
```

The `surd_*` stages exist only on SURD states, the `brcd_*` stages only on BRCD states, and `feature_select` and `clean_data` only on SURD states. Asking a BRCD pipeline to select features is impossible. The assumption that wrecked the old design, that every algorithm wants feature selection, is now unrepresentable, which eliminates an entire class of API-misuse errors.
The old design assumed that building a configuration for a discovery pipeline would always yield a valid one. In practice, that does not hold.

The new design treats the configuration as the single source of truth for a run, and builds it with a staged typestate builder. Each required field is its own stage, so `build()` becomes callable only once every required field is set; omitting one is a compile error. `build()` then checks that the referenced files exist, failing before the pipeline starts. With the config built, the stages take no parameters; they read what they need from the explicit configuration alone. Whether the run fails or returns the expected result, the configuration that drove the pipeline is explicit, so you can verify each parameter quickly. Taken together, `CdlConfigBuilder` and `CdlBuilder` form one coherent discovery process:

```rust
let config = CdlConfigBuilder::build_surd_config::<f64>()
    .with_path("./data.csv")
    .with_target_index(3)
    .with_num_features(3)
    .with_max_order(MaxOrder::Max)
    .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
    .build()?;

CdlBuilder::build_surd(&config)
    .surd_load_input()
    .clean_data(OptionNoneDataCleaner)
    .feature_select()
    .surd_discover()
    .surd_analyze()
    .finalize()
    .print_results();
```

The BRCD sub-pipeline reads the same way, with its own stages. Loading both datasets and the optional CPDAG happens inside the pipeline, so the chain stays uniform:

```rust
let config = CdlConfigBuilder::build_brcd_config()
    .with_normal_path("./normal.csv")
    .with_anomalous_path("./anomalous.csv")
    .with_brcd_config(BrcdConfig::<f64>::continuous(0))
    .with_cpdag_path("./cpdag.csv") // optional; omit this line to learn the graph from the data via BOSS
    .build()?;

CdlBuilder::build_brcd(&config)
    .brcd_load_input()
    .brcd_discover()
    .brcd_analyze()
    .finalize()
    .print_results();
```

Omitting `with_cpdag_path` automatically configures the pipeline to learn the required graph structure using the BOSS algorithm. The two sub-pipelines meet only at the end, where a closed `CdlDiscoveryOutcome` enum carries either result into one shared `analyze` and `finalize` tail. 

## BRCD verification

The BRCD sub-pipeline was verified on the RCAEval Sock Shop benchmark: forty-four service metrics, a normal window and an anomalous window, and the supplied service-call CPDAG. The pipeline ranks `shipping_latency` as the top root cause, matching both the case's recorded expectation and the standalone reference run of the algorithm. Remove the CPDAG configuration entry and the same case runs through the BOSS path, learning the structure from the normal telemetry before ranking. 

## Gains

The new design of the CDL brought several gains.

The new CDL is more **expressive**. It hosts two genuinely different discovery methods as equals, an information decomposition and a root-cause ranker, and the closed-enum convergence leaves room for more without disturbing the ones already there.

The new CDL is more **concise**. The config is the single source of truth, the stages are parameterless, and the mid-pipeline closures are gone. A pipeline reads as a list of verbs, and a run's parameters live in one place you can check before anything executes.

The new CDL is more **powerful**. Required fields are enforced at compile time, files are checked before the pipeline starts, crossing the two sub-pipelines does not compile, and a missing graph quietly becomes a BOSS-learned one. 

Six months ago the Causal Discovery Language made one algorithm pleasant to run. The assumptions that made that possible were invisible until a second algorithm pressed on them. Rebuilding the CDL turned a single-algorithm pipeline into a foundation built to grow as new algorithms are added.

## Getting started

* Explore the [code on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery).
* Run the examples this post draws from:
  * `git clone https://github.com/deepcausality-rs/deep_causality.git && cd deep_causality`
  * `cargo run -p causal_discovery_examples --example example_surd_discovery`
  * `cargo run -p causal_discovery_examples --example example_brcd_discovery`
  * `cargo run -p causal_discovery_examples --example example_brcd_boss_discovery`
* Join the [community](https://www.deepcausality.com/community/) or the [Discord server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
