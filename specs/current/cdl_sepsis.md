# Sepsis Case Study: HKT CDL Implementation Mockup

This document mocks up the "ICU Sepsis" case study using the proposed HKT-based Causal Discovery Language (CDL). It demonstrates the concise syntax with inline configuration and the direct usage of existing algorithms from `deep_causality_algorithms`.

## 1. Setup and imports

```rust
// Use the unified single crate import
use deep_causality_discovery::*;
```

## 2. The Pipeline (Monadic Flow)

We merge configuration directly into the pipeline steps for conciseness. We also inject the specific algorithms defined in the `deep_causality_algorithms` crate.

```rust
fn run_sepsis_cdl() -> CdlEffect<ProcessFormattedResult> {
    // 1. Initialize empty CDL state
    let initial_state = CdlBuilder::new();

    initial_state
        // 2. Load Data with Inline Config
        // Signature: load_data(file_path, target_column_index, exclude_column_indices)
        .bind(|cdl| cdl.load_data(
            "examples/case_study_icu_sepsis/data/all/dataset.parquet", 
            41,       // SepsisLabel is at index 41
            vec![42]  // Patient_ID is at index 42 (excluded)
        ))
        
        // 3. Preprocess (Optional)
        // Passes a preprocessor implementation
        .bind(|cdl| cdl.preprocess(DataDiscretizer::default()))
        
        // 4. Feature Selection using MRMR from deep_causality_algorithms
        // We pass a closure that calls the specific algorithm with its required configuration.
        // mrmr_features_selector(tensor, num_features, target_col)
        .bind(|cdl| cdl.feature_select(|tensor| {
            // Configuration: Select 39 features, Target is at index 41 (adjusted internally usually, but passed here)
            // Note: The tensor passed here typically has excluded columns removed already by load_data/preprocess.
            mrmr_features_selector(tensor, 39, 41) 
        }))
        
        // 5. Causal Discovery using SURD from deep_causality_algorithms
        // surd_states_cdl(tensor, max_order)
        .bind(|cdl| cdl.causal_discovery(|tensor| {
            // Configuration: MaxOrder::Max implies using all available variables up to the limit
            surd_states_cdl(tensor, MaxOrder::Max)
        }))
        
        // 6. Analyze Results
        .bind(|cdl| cdl.analyze())
        
        // 7. Finalize and Format
        .bind(|cdl| cdl.finalize())
}
```

## 3. Execution and Handling Effects

The execution block remains focused on handling the `Result` and `CdlWarningLog`.

```rust
fn main() {
    let result_effect = run_sepsis_cdl();

    // 1. Check for Fatal Errors or Success
    match result_effect.inner {
        Ok(formatted_result) => {
            println!("✅ Sepsis Analysis Completed Successfully!");
            println!("----------------------------------------");
            println!("{}", formatted_result);
        }
        Err(e) => {
            eprintln!("❌ Pipeline Failed!");
            eprintln!("Error: {:?}", e);
        }
    }

    // 2. Inspect Accumulated Warnings
    // print_warnings() prints "No Warnings" or lists them in order.
    result_effect.print_warnings();
}
```
