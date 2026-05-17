---
title: DeepCausality Introduces the MaybeUncertain Type
description: This post summarizes the new Uncertain<T> and MaybeUncertain<T> types for robust uncertainty modeling in DeepCausality.
date: 2025-09-25
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

The DeepCausality project announces the introduction of `Uncertain<T>` and the `MaybeUncertain<T>` types within the
`deep_causality_uncertain` crate. These first-class types provide a robust way to model and propagate uncertainty,
ensuring your causal models are reliable.

## 💡 The Problem with Certainty in an Uncertain World

Traditional programming often forces us to represent inherently uncertain data with discrete, "certain" types like `f64`
or `bool`. This approach, however, hides the underlying variance and can lead to:

1. **Ignoring Random Error**: Treating an estimate as a fact overlooks the inherent noise and variability.
2. **Compounding Errors**: Simple arithmetic operations on uncertain values can quickly magnify small errors into
   significant inaccuracies.
3. **Misleading Decisions**: Boolean questions asked of probabilistic data often result in false positives or negatives,
   leading to incorrect conclusions.

## ⚡ Introducing `Uncertain<T>`: First-Class Uncertainty

Based on [foundational research](https://jamesbornholt.com/papers/uncertaint-asplos14.pdf), `Uncertain<T>` is a generic
type that encapsulates a value along with its inherent uncertainty, modeled as a probability distribution. It transforms
how you interact with data by making uncertainty a first-class citizen in your code.

### Key Capabilities of `Uncertain<T>`:

* **Rich Distribution Support**: Easily create uncertain values from `Point` (precise), `Normal` (Gaussian noise),
  `Uniform` (range-bound), and `Bernoulli` (probabilistic boolean) distributions.
* **Intuitive Operator Overloading**: Perform standard arithmetic (`+`, `-`, `*`, `/`), comparison (`>`, `<`, `==`), and
  logical (`&`, `|`, `!`) operations directly on `Uncertain` types. The uncertainty is automatically propagated through
  these operations, building an implicit computation graph.
* **Lazy, Efficient Evaluation**: The underlying computation graph is evaluated lazily using intelligent sampling and
  statistical hypothesis tests (like SPRT), drawing only as many samples as necessary for robust decision-making.
* **Comprehensive Statistical Analysis**: Extract meaningful insights with methods like `expected_value()`,
  `standard_deviation()`, and `estimate_probability()`.
* **Robust Decision Making**: Make informed choices under uncertainty using `to_bool()`, `probability_exceeds()`, and
  `implicit_conditional()`.

## 🌍 Beyond Uncertain: `MaybeUncertain<T>` for Probabilistic Presence

While `Uncertain<T>` handles the uncertainty of a value, it assumes the presence of a value, but real-world data often
presents an even more fundamental challenge: the uncertainty *of a value's very existence*. Think of a sensor that
intermittently fails, or a patient who misses a clinical measurement. Is the value `0.0` or `false` truly the
measurement, or is the data simply absent?

This is where `MaybeUncertain<T>` shines. It's a specialized type designed to explicitly model values that are *
*probabilistically present or absent**. If a value is present, its own value is, of course, uncertain. You can think of
`MaybeUncertain<T>` as a "Probabilistic Option" or "Probabilistic Maybe" type.

### Why `MaybeUncertain<T>` Matters:

* **Explicit Modeling of Missing Values**: Instead of forcing a choice between a definite value and a definite `None`,
  `MaybeUncertain<T>` allows you to model the probability of presence itself. This is crucial for sparse or
  intermittently available data.
* **Robustness**: It prevents uncertainly-present data from propagating through a model as if it were certainly present,
  which could lead to incorrect conclusions.
* **Nuanced Decision-Making**: It enables more sophisticated causal pathways that can adapt to the quality and
  completeness of the available data.

### Key Capabilities of `MaybeUncertain<T>`:

* **Flexible Construction**: Create instances for values that are:
    * Certainly present, certain value (`from_value`).
    * Certainly present, uncertain value (`from_uncertain`).
    * Certainly absent (`always_none`).
    * Probabilistically present with an uncertain value (`from_bernoulli_and_uncertain`).
* **Intelligent Sampling**: The `sample()` method returns an `Option<f64>`, reflecting the probabilistic presence.
* **Presence Probability**: `is_some()` and `is_none()` methods return `Uncertain<bool>`, allowing you to reason about
  the probability of the value's presence or absence.
* **Arithmetic Propagation**: Standard arithmetic operations (`+`, `-`, `*`, `/`, unary `-`) correctly propagate the "
  None" state: if any operand is probabilistically absent, the result will also be probabilistically absent.
* **Probabilistic Gate (`lift_to_uncertain`)**: This powerful method acts as a statistical gate, converting a
  `MaybeUncertain<T>` to a standard `Uncertain<T>` *only if* there is sufficient statistical evidence (above a defined
  threshold and confidence level) that the value is reliably present. Otherwise, it returns an
  `UncertainError::PresenceError`.

## 🧪 Clinical Trial Example: Aspirin Headache Analysis

Let's illustrate the power of `MaybeUncertain<T>` with a real-world scenario: analyzing data from a clinical trial for
Aspirin's effectiveness in reducing headache pain.

In such trials, data can be messy. Patients might drop out, miss measurements, or provide subjective reports.
`MaybeUncertain<T>` allows us to model these complexities directly.

```rust
use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};

fn main() -> Result<(), UncertainError> {
    println!("Aspirin Headache Trial Analysis");
    println!("=====================================");

    // Patient A: Control Group (Placebo Effect), certainly present, certain value
    let patient_a_pain_reduction = MaybeUncertain::<f64>::from_value(0.5);

    // Patient B: Aspirin Group (Strong Responder), certainly present, uncertain value
    let patient_b_pain_reduction =
        MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(6.0, 2.0));

    // Patient C: Aspirin Group (Weak Responder / Intermittent Reporting)
    // Low probability of reporting reduction, and if reported, it's small.
    let patient_c_pain_reduction =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.3, Uncertain::normal(1.0, 0.5));

    // Patient D: Aspirin Group (Moderate Responder / Good Reporting)
    // High probability of reporting reduction, moderate effect.
    let patient_d_pain_reduction =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.8, Uncertain::normal(4.0, 2.5));

    // Patient E: Control Group (Missing Data), certainly absent
    let patient_e_pain_reduction = MaybeUncertain::<f64>::always_none();

    // --- Using lift_to_uncertain() for Data Reliability ---
    println!("--- lift_to_uncertain() for Data Reliability ---");

    // Collect and lift data for Aspirin group (only reliable data)
    let mut aspirin_reductions: Vec<Uncertain<f64>> = Vec::new();

    // Only include data if there's high confidence in its presence
    if let Ok(reduction) = patient_b_pain_reduction.lift_to_uncertain(0.9, 0.95, 0.05, 1000) {
        aspirin_reductions.push(reduction);
    }
    if let Ok(reduction) = patient_d_pain_reduction.lift_to_uncertain(0.7, 0.95, 0.05, 1000) {
        aspirin_reductions.push(reduction);
    }
    // Patient C's data is unlikely to be lifted due to low presence probability (0.3)
    if let Ok(reduction) = patient_c_pain_reduction.lift_to_uncertain(0.8, 0.95, 0.05, 1000) {
        aspirin_reductions.push(reduction);
    }

    let control_reduction = patient_a_pain_reduction
        .clone()
        .lift_to_uncertain(0.9, 0.95, 0.05, 1000)?;

    if aspirin_reductions.is_empty() {
        println!("Not enough reliable Aspirin data to draw conclusions.");
    } else {
        let num_aspirin_reductions = aspirin_reductions.len() as f64;
        let total_aspirin_reduction = aspirin_reductions
            .into_iter()
            .reduce(|acc, r| acc + r)
            .unwrap();
        let avg_aspirin_reduction =
            total_aspirin_reduction / Uncertain::<f64>::point(num_aspirin_reductions);

        println!(
            "Average Aspirin Group Pain Reduction: {:.2}",
            avg_aspirin_reduction.expected_value(1000)?
        );
        println!(
            "Average Control Group Pain Reduction: {:.2}",
            control_reduction.expected_value(1000)?
        );

        // Compare Aspirin vs Control
        let aspirin_better_than_control =
            avg_aspirin_reduction.greater_than(control_reduction.expected_value(1000)?);

        let confidence_aspirin_better =
            aspirin_better_than_control.estimate_probability(1000)? * 100.0;

        println!(
            "Confidence that Aspirin is better than Control: {:.1}%",
            confidence_aspirin_better
        );

        if aspirin_better_than_control.probability_exceeds(0.9, 0.95, 0.05, 1000)? {
            println!("✅ Conclusion: Aspirin reduces headache pain within uncertainty bounds!");
        } else {
            println!(
                "❌ Conclusion: Evidence is insufficient to confidently say Aspirin reduces headache pain."
            );
        }
    }
    Ok(())
}
```

In this example, `MaybeUncertain<f64>` allows us to:

1. **Model diverse patient data**: From certainly present and certain values (`from_value`) to probabilistically present
   and uncertain values (`from_bernoulli_and_uncertain`).
2. **Filter unreliable data**: The `lift_to_uncertain` method acts as a crucial probabilistic gate. It ensures that only
   patient data with sufficient statistical evidence of presence (e.g., 70% probability of being present with 95%
   confidence) is included in the final analysis. This prevents sparse or unreliable data from skewing the results.
3. **Draw robust conclusions**: By combining `MaybeUncertain<T>` with `Uncertain<T>`'s statistical capabilities, we can
   confidently compare the average pain reduction between groups and make a statistically sound conclusion about
   Aspirin's effectiveness, even with real-world data imperfections.

## Conclusion

The `Uncertain<T>` and `MaybeUncertain<T>` types are powerful additions to the DeepCausality ecosystem, providing
developers with the tools to build more robust, accurate, and trustworthy causal models in the face of inherent data
uncertainty and incompleteness. By embracing uncertainty as a first-class concept, DeepCausality empowers you to make
better, more informed decisions.

Get Started with DeepCausality. The Future is Now!

* Explore the [code examples on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain/examples).
* Join the [community](https://www.deepcausality.com/community/).
* Join the [Discord Server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
