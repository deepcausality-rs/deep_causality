---
title: Causal Discovery Language
description: A typestate-builder DSL for going from raw observational data to an executable causal model.
section: concepts
order: 8
---

The Causal Discovery Language (CDL) is the DSL that bridges raw observational data and an executable causal model. It lives in the `deep_causality_discovery` crate and uses Rust's typestate pattern to encode the pipeline stages in the type system.

The library's other concepts assume you already have a Causaloid. The CDL is for the case where you do not.

## The problem it solves

Discovering causal structure from data is not one operation. It is a pipeline: load the data, clean it, select the features that carry signal, run a discovery algorithm, analyze the result, and finalize an executable model. Each stage has its own configuration, its own failure modes, and its own outputs that the next stage depends on. Doing this by hand in a notebook ends in fragile glue code; doing this with a generic pipeline framework loses the type safety that makes Rust worth using here.

The CDL keeps the type safety. Each stage of the pipeline returns a new typestate. You cannot accidentally call the discovery operation before the feature-selection operation. You cannot finalize a model from an incomplete pipeline. The compiler refuses.

## The pipeline

The current stages (drawn from `deep_causality_discovery::src/types/`):

1. **Configure** — `CdlConfig` is the builder for the whole pipeline. Set the data source, the cleaning strategy, the feature-selection criterion, the discovery algorithm, the analysis pass.
2. **Load** — read observations from CSV, Parquet, or an in-memory matrix.
3. **Clean** — handle missing values, outliers, type coercions.
4. **Select features** — pick the most informative subset of variables. The default is MRMR; SURD is available for information-theoretic decomposition.
5. **Discover** — run the chosen causal-discovery algorithm against the selected features.
6. **Analyze** — produce stability, sparsity, and significance metrics on the discovered structure.
7. **Finalize** — emit a `Causaloid` (or `CausaloidGraph`) that downstream code can consume directly.

The output of step 7 is the input to the rest of DeepCausality. The pipeline ends where the inference workflow begins.

## What the code looks like

```rust
use deep_causality_discovery::*;

let model = CdlConfig::new()
    .with_loader(CsvLoader::new("observations.csv"))
    .with_cleaner(Cleaner::default())
    .with_feature_selector(MrmrSelector::new(20, mrmr::Criterion::Difference))
    .with_discoverer(Pc::default())
    .with_analyzer(StabilityAnalyzer::default())
    .build()?
    .load()?
    .clean()?
    .select_features()?
    .discover()?
    .analyze()?
    .finalize()?;

// `model` is a CausaloidGraph wired to a Context built from the observations.
let effect = model.evaluate(&fresh_observation)?;
```

Each method returns a new typestate. The compiler will not let you call `.discover()` before `.select_features()`; the method does not exist on the previous typestate. The pipeline either compiles, or it does not have the right shape.

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

The CDL is a *producer* of [Causaloid graphs](/docs/concepts/causaloid/). It is not a separate inference engine; the model it produces uses the same `Causaloid` and `Context` types as a hand-written model. Once the pipeline finalizes, the discovered model is indistinguishable from a hand-built one.

The CDL builds a `Context` for you. The Context holds the observed feature space, the relationships discovered between features, and any side metadata (sample sizes, confidence intervals) the analyzer surfaced. You can replace the Context after the fact; the discovered Causaloid graph does not care where the Context came from.

The CDL respects the AI Styleguide and the rebrand: outputs are framed as Causaloids and Contexts. The pipeline does not export private types.

## What it is not

The CDL is not a graphical interface. It is a Rust DSL. If you want a notebook front end, embed the pipeline in your notebook of choice and run it from there.

The CDL is not a full statistical workbench. It does discovery and analysis. It does not replace your data-cleaning library or your modelling stack.

The CDL is not coupled to a particular algorithm. The discoverer is a trait; ship a custom impl if the bundled ones do not fit.

## Where to look next

The end-to-end walkthrough is in the [CDL pipeline guide](/docs/guides/cdl-pipeline/). The crate reference is at [deep_causality_discovery](/docs/reference/deep_causality_discovery/). The bioinformatics [example](/examples/bioinformatics-signal/) uses the underlying MRMR primitive directly, without the full pipeline, and is the simplest end-to-end exposure to the algorithms layer.
