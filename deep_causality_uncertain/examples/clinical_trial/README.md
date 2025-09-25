# Aspirin Headache Trial Analysis Example

This example demonstrates how to use the `MaybeUncertain<T>` type to model and analyze data from a clinical trial, specifically focusing on the effectiveness of Aspirin in reducing headache pain. It showcases how to handle situations where the presence of data itself is uncertain, alongside the uncertainty of the data's value.

## Scenario

We simulate a clinical trial where patients report their headache pain reduction on a scale of 0-10 after taking either Aspirin or a placebo. Due to various real-world factors (e.g., patient drop-out, missed measurements, subjective reporting), some patients' pain reduction data might be missing, or their reported reduction might be uncertain.

This example illustrates:

-   **Modeling Probabilistic Data Presence**: Using `MaybeUncertain<f64>` to represent pain reduction scores, where the data might not always be available.
-   **Diverse Patient Data**: Creating `MaybeUncertain` instances using various constructors to represent different patient scenarios (certainly present/absent, probabilistically present).
-   **`sample()` Method**: Demonstrating how to sample from `MaybeUncertain` to get `Option<f64>` results.
-   **`is_some()` and `is_none()`**: Obtaining `Uncertain<bool>` to assess the probability of data presence.
-   **Arithmetic Operations**: Performing calculations (e.g., average pain reduction) and observing how `None` values propagate.
-   **`lift_to_uncertain()` for Data Reliability**: Using this method as a probabilistic gate to determine if there's sufficient statistical evidence to consider a patient's data reliably present for further analysis.
-   **Demonstrating Drug Effectiveness**: Comparing the Aspirin group's pain reduction against the control group, even with inherent uncertainties and missing data, to show the drug's efficacy.

## How to Run

To run this example, navigate to the root of the `deep_causality` project and execute the following command:

```bash
cargo run --example clinical_trial -p deep_causality_uncertain
```

This will compile and run the `clinical_trial.rs` example, displaying the results of the pain reduction analysis, including group comparisons and conclusions about Aspirin's effectiveness under uncertainty.