---
title: Causal Discovery Language
description: A typestate-builder DSL for going from raw observational data to a discovery report that informs the construction of an executable causal model.
sidebar:
  order: 9
---

The Causal Discovery Language (CDL) is the DSL that bridges raw observational data and an executable causal model. It lives in the [`deep_causality_discovery`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery) crate and uses Rust's typestate pattern to encode the pipeline stages in the type system.

The library's other concepts assume you already have a Causaloid. The CDL is for the case where you do not.

## Two algorithms, two lineages

CDL hosts two discovery algorithms as **compile-time-isolated lineages** that converge on a shared analyze/finalize tail:

- **SURD** (Synergistic, Unique, Redundant Decomposition): from a single dataset, decomposes how a set of source variables drive a target. It is the shipped implementation of [SURD-states](/concepts/algorithms/).
- **BRCD** (Bayesian Root-Cause Discovery): given a *normal* and an *anomalous* dataset over the same variables plus a causal graph, ranks which variable's conditional mechanism changed — the root cause of a regime shift. The graph is supplied as a CPDAG, or learned from the normal data via BOSS.

The lineages share no typestate until the converged analysis state, so calling a BRCD stage on a SURD pipeline (or the reverse) is a compile error, not a runtime one.

## The problem it solves

Discovering causal structure from data is not one operation. It is a pipeline: load the data, prepare it, run a discovery algorithm, analyze the result, and finalize a report that informs how the causal model is constructed. Each stage has its own configuration, its own failure modes, and its own outputs that the next stage depends on. Doing this by hand in a notebook ends in fragile glue code; doing this with a generic pipeline framework loses the type safety that makes Rust worth using here.

The CDL keeps the type safety. Each stage returns a new typestate, and a stage method only exists on the typestate that precedes it. You cannot run discovery before preparing the data, and you cannot finalize an incomplete pipeline. The compiler refuses.

## The pipeline

A run config built by `CdlConfigBuilder` is the **single source of truth**. It is a staged typestate builder: each required field is its own stage, so `build()` is reachable only once every required field is set (omitting one is a compile error), and `build()` then verifies that the referenced data files exist.

```rust
// SURD config: dataset path, target, MRMR feature count, max order, thresholds.
let cfg = CdlConfigBuilder::build_surd_config::<f64>()
    .with_path("./data.csv")
    .with_target_index(3)
    .with_num_features(3)
    .with_max_order(MaxOrder::Max)
    .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
    .build()?;

// BRCD config: two datasets + the algorithm config; CPDAG optional (None => BOSS).
let cfg = CdlConfigBuilder::build_brcd_config()
    .with_normal_path("./normal.csv")
    .with_anomalous_path("./anomalous.csv")
    .with_brcd_config(BrcdConfig::<f64>::continuous(0))
    .with_cpdag_path("./cpdag.csv")
    .build()?;
```

`CdlBuilder::build_surd(&cfg)` / `build_brcd(&cfg)` seed the pipeline with the config and fix the precision. The stages then read their parameters from the config, so the chain is parameterless:

- **SURD**: `surd_load_input → clean_data → feature_select → surd_discover → surd_analyze → finalize`
- **BRCD**: `brcd_load_input → brcd_discover → brcd_analyze → finalize`

The final stage emits a `CdlReport` carrying a `CdlDiscoveryOutcome` (the `Surd` or `Brcd` result); its `Display` renders the matching section with edge-construction recommendations (for example, "Strong unique influence: Recommended Direct edge in `CausaloidGraph`", or a ranked list of root-cause candidates). The report is where the pipeline ends and the model-construction workflow begins.

## What the code looks like

The pipeline is a monadic sequence over `CdlEffect<T>`. Each stage is a method on the effect, so the chain reads top to bottom with no per-line wrapper; under the hood an `FnOnce` `and_then` threads the value and merges warnings, short-circuiting on the first error. The full runnable version is in [`examples/causal_discovery_examples/cdl/surd_discovery/main.rs`](https://github.com/deepcausality-rs/deep_causality/blob/main/examples/causal_discovery_examples/cdl/surd_discovery/main.rs):

```rust
use deep_causality_discovery::*;

let config = CdlConfigBuilder::build_surd_config::<f64>()
    .with_path(&file_path)
    .with_target_index(3)
    .with_num_features(3)
    .with_max_order(MaxOrder::Max)
    .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
    .build()?;

let result_effect = CdlBuilder::build_surd(&config)
    .surd_load_input()
    .clean_data(OptionNoneDataCleaner)
    .feature_select()
    .surd_discover()
    .surd_analyze()
    .finalize();

result_effect.print_results();
```

[`CdlEffect<T>`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_discovery/src/types/cdl_effect/mod.rs) is:

```rust
pub struct CdlEffect<T> {
    pub inner: Result<T, CdlError>,
    pub warnings: CdlWarningLog,
}
```

It carries either the next-stage `CDL<...>` typestate or a `CdlError`, plus accumulated warnings. The HKT witness `CdlEffectWitness<CdlError, CdlWarningLog>` implements `Functor`, `Pure`, `Applicative`, and `Monad` from [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft); `CdlBuilder` plugs into the `Effect3` machinery and fixes the error and warning channels.

Two layers of safety run at once. The outer `CdlEffect` monad sequences and short-circuits. The inner `CDL<State>` typestate enforces stage order and algorithm isolation: `surd_discover` exists only on the SURD-features state and `brcd_discover` only on the BRCD-loaded state, so crossing the lineages — or running discovery before the data is ready — is a compile error.

## When to reach for it

You want the CDL when one of these is true:

- The causal structure is not known up front. You have data and you want the library to find the structure (SURD), or to localize a fault across a known graph (BRCD).
- You want a reproducible, type-safe pipeline rather than an exploratory notebook.
- You want one explicit, compile-checked config to drive the run.

You want to write Causaloids directly when one of these is true:

- The causal structure is known. You are encoding domain expertise, not discovering it.
- The rules need to do something a discovery algorithm cannot produce (custom conditionals, side-effecting actions, calls into other libraries).
- Performance constraints rule out the discovery phase.

Most production systems use both. The CDL produces an initial discovery report from historical data; the operator constructs the `CausaloidGraph` from those recommendations and adds rules the data does not justify on its own.

## The relationship to other concepts

The CDL is a *producer of recommendations* for [Causaloid graphs](/concepts/causaloid/). SURD's unique/synergistic/redundant findings guide which edges and `AggregateLogic` to wire into the graph; BRCD's ranking points at the node whose mechanism changed, the root cause to act on. The constructed model uses the same types as a hand-written one and feeds the rest of the framework directly.

A `Context` is the engineer's job: assemble the Contextoids the discovered Causaloids should evaluate against and hand them in. The pipeline produces the recommendations; you supply the world they read from.

## Where to look next

The runnable walkthroughs are in [`examples/causal_discovery_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_discovery_examples): `cdl/surd_discovery` for SURD, and `cdl/brcd_discovery` / `cdl/brcd_boss_discovery` for BRCD on a real Sock Shop case. The API reference lives on docs.rs at [`deep_causality_discovery`](https://docs.rs/deep_causality_discovery). The underlying MRMR, SURD, and BRCD primitives are in [`deep_causality_algorithms`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms), usable directly when the full pipeline is more than you need.
