---
title: Causal Discovery Language
description: A typestate-builder DSL for going from raw observational data to an executable causal model.
section: concepts
order: 8
---

The Causal Discovery Language (CDL) is the DSL that bridges raw observational data and an executable causal model. It lives in the [`deep_causality_discovery`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery) crate and uses Rust's typestate pattern to encode the pipeline stages in the type system.

The library's other concepts assume you already have a Causaloid. The CDL is for the case where you do not.

## The problem it solves

Discovering causal structure from data is not one operation. It is a pipeline: load the data, clean it, select the features that carry signal, run a discovery algorithm, analyze the result, and finalize an executable model. Each stage has its own configuration, its own failure modes, and its own outputs that the next stage depends on. Doing this by hand in a notebook ends in fragile glue code; doing this with a generic pipeline framework loses the type safety that makes Rust worth using here.

The CDL keeps the type safety. Each stage of the pipeline returns a new typestate. You cannot accidentally call the discovery operation before the feature-selection operation. You cannot finalize a model from an incomplete pipeline. The compiler refuses.

## The pipeline

The pipeline has seven stages:

1. **Configure**: [`CdlConfig`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery/src/types/config) is the builder for the whole pipeline. Set the data source, the cleaning strategy, the feature-selection criterion, the discovery algorithm, the analysis pass.
2. **Load**: read observations from CSV, Parquet, or an in-memory matrix.
3. **Clean**: handle missing values, outliers, type coercions.
4. **Select features**: pick the most informative subset of variables. The default is MRMR; SURD is available for information-theoretic decomposition.
5. **Discover**: run the chosen causal-discovery algorithm against the selected features.
6. **Analyze**: produce stability, sparsity, and significance metrics on the discovered structure.
7. **Finalize**: emit a `Causaloid` (or `CausaloidGraph`) that downstream code can consume directly.

The output of step 7 is the input to the rest of DeepCausality. The pipeline ends where the inference workflow begins.

## What the code looks like

The pipeline is a monadic sequence over `CdlEffect<T>`. `CdlBuilder::build()` lifts an empty `CDL<NoData>` typestate into the effect; every `.bind(|cdl| ...)` advances the typestate one stage and threads any error or warning through the chain. The full runnable version is in [`deep_causality_discovery/examples/main.rs`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_discovery/examples/main.rs):

```rust
use deep_causality_discovery::{
    CdlBuilder, MaxOrder, OptionNoneDataCleaner, mrmr_features_selector, surd_states_cdl,
};

let target_index = 3;

let result_effect = CdlBuilder::build()
    .bind(|cdl| cdl.load_data(&file_path, target_index, vec![]))
    .bind(|cdl| cdl.clean_data(OptionNoneDataCleaner))
    .bind(|cdl| {
        cdl.feature_select(|tensor| {
            mrmr_features_selector(tensor, 3, target_index)
        })
    })
    .bind(|cdl| {
        cdl.causal_discovery(|tensor| {
            surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)
        })
    })
    .bind(|cdl| cdl.analyze())
    .bind(|cdl| cdl.finalize());

result_effect.print_results();
```

[`CdlEffect<T>`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_discovery/src/types/cdl_effect/mod.rs) is:

```rust
pub struct CdlEffect<T> {
    pub inner: Result<T, CdlError>,
    pub warnings: CdlWarningLog,
}
```

It carries either the next-stage `CDL<...>` typestate or a `CdlError`, plus a list of accumulated warnings. The HKT witness `CdlEffectWitness<CdlError, CdlWarningLog>` implements `Functor`, `Pure`, `Applicative`, and `Monad` from [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft); `CdlBuilder` plugs into the `Effect3` machinery and fixes the error and warning channels. `bind` short-circuits on the first error, concatenates warnings on success, and lets the typestate inside `CDL<...>` advance one stage per step.

Two layers of safety run at the same time. The outer `CdlEffect` monad sequences and short-circuits. The inner `CDL<State>` typestate enforces stage order: the method that runs causal discovery only exists on the typestate that has features selected, so calling it before `feature_select` is a compile error, not a runtime error.

## When to reach for it

You want the CDL when one of these is true:

- The causal structure is not known up front. You have data and you want the library to find the structure.
- You want a reproducible, type-safe pipeline rather than an exploratory notebook.
- You want to swap one algorithm for another without rewriting the surrounding glue.

You want to write Causaloids directly when one of these is true:

- The causal structure is known. You are encoding domain expertise, not discovering it.
- The rules need to do something a discovery algorithm cannot produce (custom conditionals, side-effecting actions, calls into other libraries).
- Performance constraints rule out the discovery phase. You want the runtime form without the upfront search.

Most production systems use both. The CDL produces an initial Causaloid graph from historical data; the operator hand-edits the graph and adds rules the data does not justify on its own. The result is a hybrid.

## The relationship to other concepts

The CDL is a *producer* of [Causaloid graphs](/docs/concepts/causaloid/). It is not a separate inference engine; the model the `finalize` stage emits uses the same `Causaloid` and `Context` types as a hand-written model. Once the pipeline finishes, the discovered model is indistinguishable from a hand-built one and feeds the rest of the framework directly.

The CDL does not invent a Context. A `Context` is the engineer's job: assemble the Contextoids the discovered Causaloids should evaluate against and hand them in. The pipeline produces the rules; you supply the world they read from.

## Where to look next

The runnable end-to-end walkthrough is [`deep_causality_discovery/examples/main.rs`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_discovery/examples/main.rs). The API reference lives on docs.rs at [`deep_causality_discovery`](https://docs.rs/deep_causality_discovery). The underlying MRMR and SURD primitives are in [`deep_causality_algorithms`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms), which you can use directly when the full pipeline is more than you need.
