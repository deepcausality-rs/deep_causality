# 000-Pre-Specs: Arity-5 Causal Discovery Language (CDL) Example for Sepsis Case Study

This document illustrates a conceptual application of an arity-5 Causal Discovery Language (CDL) pipeline, specifically tailored to the ICU Sepsis Prediction case study. The goal is to demonstrate how the enhanced effect system can provide a richer, more comprehensive, and auditable output beyond just the causal discovery results and fatal errors.

---

## Recap of Arity-5 CDL Channels

As discussed, an arity-5 effect system would allow the CDL to track five distinct types of information simultaneously:

1.  **Primary Value (`T`):** The successful result of a pipeline stage (e.g., a processed tensor, a causal graph, or the final formatted report).
2.  **Fatal Error (`CdlError`):** A single, critical error that causes the pipeline to short-circuit.
3.  **Non-Fatal Warnings (`CdlWarningLog`):** A collection of issues encountered that do not halt execution but provide important context.
4.  **Performance Telemetry (`PerformanceLog`):** Metrics related to the execution time and resource consumption of each stage.
5.  **Data Provenance & Audit Trail (`AuditTrail`):** A detailed, immutable record of decisions, transformations, and parameters applied throughout the pipeline.

---

## Mock Example: Sepsis Causal Discovery Pipeline (`src/stage_3_hkt_pipeline.rs`)

This conceptual Rust code demonstrates how each stage of the Sepsis causal discovery process would contribute to these five channels, and how the final, enriched `CdlEffect` object would be consumed.

```rust
// In a real implementation, these types would be part of the haft-enabled CDL.
// We define them here conceptually for the example.

use std::time::Duration; // For conceptual timing data

// ---
1. Define the 5 Channels of the Effect System ---

/// The primary value of a successful computation at each stage.
/// e.g., Cdl<WithData>, Cdl<WithFeatures>, etc.
type Value<T> = Option<T>; 

/// Channel 1: Fatal Error (as before)
/// A fatal, short-circuiting error.
type FatalError = Option<String>; // Simplified CdlError for example

/// Channel 2: Non-Fatal Warnings
/// A log of issues that don't stop the pipeline.
#[derive(Debug, Default)]
struct CdlWarningLog { pub entries: Vec<String> }

/// Channel 3: Performance Telemetry
/// A log of performance metrics from each stage.
#[derive(Debug, Default)]
struct PerformanceLog { pub entries: Vec<String> }

/// Channel 4: Data Provenance / Audit Trail
/// An immutable record of how the result was produced.
#[derive(Debug, Default)]
struct AuditTrail { pub entries: Vec<String> }

/// The Arity-5 Effect Monad: CdlEffect
/// This single struct encapsulates the entire result of a computation.
#[derive(Debug, Default)]
struct CdlEffect<T> {
    pub value: Value<T>,
    pub error: FatalError,      // Arity 1: Error
    pub warnings: CdlWarningLog, // Arity 2: Warnings
    pub performance: PerformanceLog, // Arity 3: Performance
    pub audit: AuditTrail,       // Arity 4: Audit
    // Arity 5 is the primary value `T` itself.
}

// Mocked monadic `bind` for chaining operations.
// In a real scenario, this would be provided by the `Monad` trait implementation
// for a concrete HKT Witness (e.g., `SepsisCdlEffectWitness`).
impl<T> CdlEffect<T> {
    fn bind<U, F>(self, mut f: F) -> CdlEffect<U>
    where
        F: FnMut(T) -> CdlEffect<U>,
    {
        // If there's an error, short-circuit and just pass the effects along.
        if self.error.is_some() || self.value.is_none() {
            return CdlEffect {
                value: None,
                error: self.error,
                warnings: self.warnings,
                performance: self.performance,
                audit: self.audit,
            };
        }

        // Otherwise, run the next function `f` with the value.
        let mut next_effect = f(self.value.unwrap());
        
        // Combine the logs from the previous step with the new one.
        let mut combined_warnings = self.warnings;
        combined_warnings.entries.extend(next_effect.warnings.entries);
        next_effect.warnings = combined_warnings;

        let mut combined_performance = self.performance;
        combined_performance.entries.extend(next_effect.performance.entries);
        next_effect.performance = combined_performance;

        let mut combined_audit = self.audit;
        combined_audit.entries.extend(next_effect.audit.entries);
        next_effect.audit = combined_audit;
        
        next_effect
    }
}


/// Main function for the Sepsis case study using the HKT-based CDL.
pub fn run_sepsis_discovery_hkt() {
    println!("--- Running Sepsis Causal Discovery with Arity-5 CDL ---");
    
    // The path to the sepsis-only data, as identified in the experiment plan in notes/experiment_draft.md.
    let data_path = "examples/case_study_icu_sepsis/data/seperated/seps_true.parquet";
    
    // ---
2. Execute the Monadic Pipeline ---
    // The pipeline starts by lifting the initial data path into the `CdlEffect`
    // and is then chained with `bind`.
    // Each `bind` closure represents a pipeline stage.
    
    let final_effect: CdlEffect<String> = CdlEffect { value: Some(data_path.to_string()), ..Default::default() }
        .bind(|path| {
            // ---
Stage 1: Load Data ---
            println!("-> Stage: Loading data from {}...", path);
            let mut effect = CdlEffect::default();

            // Simulate loading data and collecting effects.
            effect.performance.entries.push("load_data_stage: 3.5s".to_string());
            effect.audit.entries.push(format!("Loaded 112,050 records from '{}'.", path));
            
            // This references a finding from the case study notes (stage_2_mrmr.rs).
            effect.warnings.entries.push("Column 'EtCO2' was string-encoded; parsed to f64. Potential data corruption.".to_string());

            // Simulate a potential fatal error (e.g., file not found).
            // if path == "non_existent.parquet" {
            //    effect.error = Some("File not found error".to_string());
            // }

            // Pass the loaded data (mocked as a string for simplicity) to the next stage.
            effect.value = Some("loaded_sepsis_tensor".to_string());
            effect
        })
        .bind(|_tensor| {
            // ---
Stage 2: Feature Selection (MRMR) ---
            println!("-> Stage: Selecting features using mRMR...");
            let mut effect = CdlEffect::default();
            effect.performance.entries.push("feature_select_stage: 15.2s".to_string());
            effect.audit.entries.push("Ran mRMR feature selection for 39 candidate features against target 'SepsisLabel'.".to_string());
            effect.audit.entries.push("Excluded column 'Patient_ID' (index 42) from feature set as it is an administrative ID.".to_string());

            // This warning is derived directly from the case study findings (notes/stage_two_findings.md).
            effect.warnings.entries.push("Dominant Feature Warning: 'ICULOS' (ICU Length of Stay) has disproportionately high importance. It may be masking other critical clinical variables. Consider context-specific analysis.".to_string());
            
            effect.value = Some("feature_selected_sepsis_tensor".to_string());
            effect
        })
        .bind(|_tensor| {
            // ---
Stage 3: Causal Discovery (SURD) ---
            println!("-> Stage: Running causal discovery with SURD-states...");
            let mut effect = CdlEffect::default();
            effect.performance.entries.push("causal_discovery_stage: 25.8s".to_string());
            effect.audit.entries.push("Executed SURD-states algorithm with max_order=3 for synergistic, unique, and redundant influences.".to_string());
            effect.audit.entries.push("Identified primary causal drivers for SepsisLabel.".to_string());

            effect.value = Some("raw_surd_result".to_string());
            effect
        })
        .bind(|_surd_result| {
            // ---
Stage 4: Analyze Causal Results & Finalize Report ---
            println!("-> Stage: Analyzing causal results and formatting report...");
            let mut effect = CdlEffect::default();
            effect.performance.entries.push("analysis_and_finalize_stage: 0.5s".to_string());
            effect.audit.entries.push("Translated numerical SURD results into human-readable causal interpretations.".to_string());
            effect.audit.entries.push("Generated Diagnostic Rapport for Patient_IDs in the 'seps_true' dataset subset.".to_string());

            // This final report is the actual "value" produced by the pipeline.
            let final_report = "Causal Graph for Sepsis Onset: (Lactate -> SepsisLabel - Unique Influence), (WBC, HR -> SepsisLabel - Synergistic Influence)".to_string();
            
            effect.value = Some(final_report);
            effect
        });

    // ---
3. Unpack and Display the Final, Rich Result ---
    // This part demonstrates how a consumer (e.g., a doctor, an analyst) would interact
    // with the comprehensive output of the arity-5 CDL.
    
    println!("\n--- PIPELINE EXECUTION SUMMARY ---\n");

    // Check for fatal errors first
    if let Some(err) = final_effect.error {
        println!("❌ [FATAL ERROR]: Pipeline execution failed: {}", err);
    } else if let Some(result_value) = final_effect.value {
        println!("✅ [SUCCESS]: Causal Discovery Pipeline Completed.");
        println!("\n--- Primary Causal Discovery Result ---");
        println!("{}", result_value);
    } else {
        println!("⚠️ [PIPELINE WARNING]: Pipeline completed without fatal errors, but no primary result value was generated. (This should be rare and indicate an issue).");
    }

    println!("\n--- 🩺 Diagnostic Rapport (from Audit Trail) ---");
    if final_effect.audit.entries.is_empty() {
        println!("- No audit entries recorded.");
    } else {
        for entry in final_effect.audit.entries {
            println!("- {}", entry);
        }
    }

    println!("\n--- ⚠️ Warnings & Diagnostics (from Warning Log) ---");
    if final_effect.warnings.entries.is_empty() {
        println!("- No non-fatal warnings issued during execution.");
    } else {
        for entry in final_effect.warnings.entries {
            println!("- {}", entry);
        }
    }

    println!("\n--- ⏱️ Performance Breakdown (from Telemetry) ---");
    if final_effect.performance.entries.is_empty() {
        println!("- No performance metrics recorded.");
    } else {
        for entry in final_effect.performance.entries {
            println!("- {}", entry);
        }
    }
}
```

---

## Discussion and Interpretation

This mock example, grounded in the Sepsis case study, vividly demonstrates the power of an arity-5 CDL:

1.  **Integrated Effect Management**: Instead of just getting a causal graph or an error message, the output is a single `CdlEffect` object containing *all* relevant information. The monadic `bind` operation ensures that `warnings`, `performance` metrics, and `audit` trails are seamlessly accumulated across all stages.

2.  **Rich Diagnostic Rapport (Audit Trail)**:
    *   **"Which data were used?"**: The audit trail records the exact dataset used (`seps_true.parquet`) and details like the number of records loaded.
    *   **"Which features were considered important?"**: It notes the execution of `mRMR` feature selection, the number of features considered, and the exclusion of `Patient_ID` (a crucial step in the Sepsis study due to its confounding nature).
    *   **"What algorithms were used?"**: It explicitly logs the execution of the `SURD-states` algorithm and its configuration (`max_order=3`).
    *   **Actionable Insights**: The final "Diagnostic Rapport" would summarize the high-level causal insights extracted (e.g., "Lactate -> SepsisLabel - Unique Influence"), echoing the kind of information an ICU doctor would find actionable.

3.  **Proactive Warning System (Warning Log)**:
    *   **Data Quality Issues**: The warning about `EtCO2` being string-encoded highlights a real-world data challenge mentioned in `stage_two_findings.md`. The CDL can now alert users to such issues without halting execution.
    *   **Methodological Concerns**: The warning about `ICULOS`'s dominance in feature selection directly addresses a key finding from `notes/stage_two_findings.md`. This is crucial for interpreting the causal graph correctly, as administrative data might mask clinical insights if not handled carefully.

4.  **Transparent Performance Monitoring (Performance Telemetry)**:
    *   **Resource Allocation**: Each stage's execution time is logged, enabling developers to identify performance bottlenecks (e.g., `causal_discovery` is often the most intensive step) and allocate computational resources effectively. This replaces ad-hoc timing mechanisms with a structured, integrated approach.

5.  **Enhanced Trust and Reproducibility**:
    *   By integrating all these aspects into a single, structured output, the CDL fosters greater trust in its results. Any causal claim comes with its full context: how it was derived, what challenges were faced, and what resources were consumed.
    *   This dramatically improves reproducibility, as every aspect of a pipeline run is encapsulated in the `CdlEffect` object, making it easier to share, verify, and build upon past analyses.

In essence, the arity-5 CDL transforms the Sepsis case study from merely generating a causal graph to producing a fully transparent, auditable, and diagnostically rich understanding of the causal relationships at play. This moves the `deep_causality` library closer to delivering on its promise of providing actionable, context-aware causal reasoning in complex, high-stakes domains.

---

### Merging Configuration and Execution: A Cohesive Block-Based Pipeline API

This design enables two key improvements:
1.  **Elimination of Upfront Monolithic `CdlConfig`**: Instead of building a large `CdlConfig` object separately and then passing it, each operator in the pipeline chain can take its specific parameters directly.
2.  **Seamless Integration of Custom Operators**: Users can define their own reusable operators (functions) and plug them directly into the pipeline using a generic `.then()` method, maintaining the same fluent, block-based style.

#### 1. Core API Components (Conceptual)

To enable this, we would define the following conceptual structures and implementations:

```rust
// Define simplified config structs needed for the mock Cdl<State> implementations
#[derive(Debug, Clone)]
pub struct CsvConfig; impl CsvConfig { pub fn default() -> Self { Self } }
#[derive(Debug, Clone)]
pub struct MrmrConfig { num_features: usize, target_col: usize }
impl MrmrConfig { pub fn new(num_features: usize, target_col: usize) -> Self { Self { num_features, target_col } } pub fn num_features(&self) -> usize { self.num_features } pub fn target_col(&self) -> usize { self.target_col } }
#[derive(Debug, Clone, Copy)]
pub enum MaxOrder { Max, Some(usize) }
#[derive(Debug, Clone)]
pub struct SurdConfig { max_order: MaxOrder, target_col: usize }
impl SurdConfig { pub fn new(max_order: MaxOrder, target_col: usize) -> Self { Self { max_order, target_col } } pub fn max_order(&self) -> MaxOrder { self.max_order } }
#[derive(Debug, Clone)]
pub struct AnalyzeConfig { synergy_threshold: f64 }
impl AnalyzeConfig { pub fn new(synergy_threshold: f64, _u:f64, _r:f64) -> Self { Self { synergy_threshold } } pub fn synergy_threshold(&self) -> f64 { self.synergy_threshold } }

// Re-using CdlEffect and its internal logs from the previous section.
// (CdlEffect, CdlWarningLog, PerformanceLog, AuditTrail are defined above in cdl-5.md)

// These are necessary helper structs/enums for the mock code to compile conceptually.
// They are assumed to be defined earlier in the document or in a `lib.rs`
// as part of the CdlEffect definition. For the self-contained nature of this code block,
// we re-declare them as mocks here.
#[derive(Debug, Default, Clone)]
pub struct CdlWarningLog { pub entries: Vec<String> }
#[derive(Debug, Default, Clone)]
pub struct PerformanceLog { pub entries: Vec<String> }
#[derive(Debug, Default, Clone)]
pub struct AuditTrail { pub entries: Vec<String> }
#[derive(Debug, Default, Clone)]
pub struct CdlEffect<T> {
    pub value: Option<T>,
    pub error: Option<String>,      
    pub warnings: CdlWarningLog, 
    pub performance: PerformanceLog, 
    pub audit: AuditTrail,       
}

impl<T: Clone> CdlEffect<T> {
    fn bind<U, F>(self, mut f: F) -> CdlEffect<U>
    where
        F: FnMut(T) -> CdlEffect<U>,
    {
        if self.error.is_some() || self.value.is_none() {
            return CdlEffect { value: None, error: self.error, warnings: self.warnings, performance: self.performance, audit: self.audit };
        }
        let mut next_effect = f(self.value.unwrap());
        let mut combined_warnings = self.warnings; combined_warnings.entries.extend(next_effect.warnings.entries.drain(..)); next_effect.warnings = combined_warnings;
        let mut combined_performance = self.performance; combined_performance.entries.extend(next_effect.performance.entries.drain(..)); next_effect.performance = combined_performance;
        let mut combined_audit = self.audit; combined_audit.entries.extend(next_effect.audit.entries.drain(..)); next_effect.audit = combined_audit;
        next_effect
    }
    fn clone_effects_without_value(&self) -> CdlEffect<()> {
        CdlEffect {
            value: None,
            error: self.error.clone(),
            warnings: self.warnings.clone(),
            performance: self.performance.clone(),
            audit: self.audit.clone(),
        }
    }
}


// Mock of the internal Cdl states for the example
#[derive(Debug, Clone)] pub struct NoData;
#[derive(Debug, Clone)] pub struct WithData;
#[derive(Debug, Clone)] pub struct WithFeatures;
#[derive(Debug, Clone)] pub struct WithCausalResults;
#[derive(Debug, Clone)] pub struct WithAnalysis;
#[derive(Debug, Clone)] pub struct Finalized;


// A simplified mock of the Cdl<State> type which actually holds the data/results.
// In reality, this would transition from Cdl<NoData> to Cdl<WithData>, etc.
#[derive(Debug, Clone)]
pub struct Cdl<State> {
    phantom: std::marker::PhantomData<State>,
    data_representation: String, // Simplified to a String for mocking purposes
}

impl<State: Clone + 'static> Cdl<State> { // State needs to be Clone for CdlEffect<Cdl<State>> and 'static for FnOnce
    // Internal mock methods that would perform the actual work of each stage
    // and return the next Cdl<State>.
    fn load_parquet_impl(path: &str) -> Cdl<WithData> {
        println!("  [Internal]: Loading Parquet from {}", path);
        Cdl { phantom: std::marker::PhantomData, data_representation: format!("ParquetDataLoaded from {}", path) }
    }

    fn feature_select_mrmr_impl(_cdl_state: Cdl<WithData>, config: MrmrConfig) -> Cdl<WithFeatures> {
        println!("  [Internal]: Selecting features with mRMR (k={}, target={}).", config.num_features(), config.target_col());
        Cdl { phantom: std::marker::PhantomData, data_representation: format!("FeaturesSelected({:?})", config) }
    }

    fn discover_surd_impl(_cdl_state: Cdl<WithFeatures>, config: SurdConfig) -> Cdl<WithCausalResults> {
        println!("  [Internal]: Discovering causality with SURD (max_order={:?}).", config.max_order());
        Cdl { phantom: std::marker::PhantomData, data_representation: format!("SURDResults({:?})", config) }
    }

    fn analyze_impl(_cdl_state: Cdl<WithCausalResults>, config: AnalyzeConfig) -> Cdl<WithAnalysis> {
        println!("  [Internal]: Analyzing results (synergy_threshold={}).", config.synergy_threshold());
        Cdl { phantom: std::marker::PhantomData, data_representation: format!("AnalyzedResults({:?})", config) }
    }

    fn format_console_impl(_cdl_state: Cdl<WithAnalysis>) -> Cdl<Finalized> {
        println!("  [Internal]: Formatting for console.");
        Cdl { phantom: std::marker::PhantomData, data_representation: "Final Report: Causal Graph Generated".to_string() }
    }
}


// The central builder that carries the CdlEffect and manages state transitions.
// The generic parameter `CurrentCdlState` ensures type-safety and proper ordering of stages.
pub struct CdlPipelineBuilder<CurrentCdlState: Clone + 'static> {
    pub current_effect: CdlEffect<Cdl<CurrentCdlState>>,
    pub data_path: String,
}

impl CdlPipelineBuilder<NoData> {
    // Entry point: takes the initial data path.
    pub fn pipeline(data_path: &str) -> CdlPipelineBuilder<NoData> {
        let initial_cdl_state = Cdl {
            phantom: std::marker::PhantomData,
            data_representation: "Initial_NoData_State".to_string(),
        };
        CdlPipelineBuilder {
            current_effect: CdlEffect {
                value: Some(initial_cdl_state),
                ..Default::default()
            },
            data_path: data_path.to_string(),
        }
    }

    // Standard operator for loading Parquet data.
    // Takes config directly and transitions state.
    pub fn load_parquet(self) -> CdlPipelineBuilder<WithData> {
        println!("-> CdlPipeline: Loading Parquet data...");
        let path_clone = self.data_path.clone(); // Clone for the closure
        let next_effect = self.current_effect.bind(move |cdl_no_data| {
            let next_cdl_state = cdl_no_data.load_parquet_impl(&path_clone);
            let mut effect_updates = CdlEffect::default();
            effect_updates.performance.entries.push("load_parquet_stage: 3.0s".to_string());
            effect_updates.audit.entries.push(format!("Loaded Parquet from {}", path_clone));
            effect_updates.warnings.entries.push("Column 'EtCO2' had to be re-parsed as f64. Data quality warning.".to_string());
            effect_updates.value = Some(next_cdl_state);
            effect_updates
        });
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
    // ... potentially other load methods like load_csv(config)
}

impl CdlPipelineBuilder<WithData> {
    // Standard operator for feature selection using MRMR.
    pub fn feature_select_mrmr(self, config: MrmrConfig) -> CdlPipelineBuilder<WithFeatures> {
        println!("-> CdlPipeline: Selecting features with mRMR...");
        let next_effect = self.current_effect.bind(move |cdl_with_data| { // `move` config
            let next_cdl_state = cdl_with_data.feature_select_mrmr_impl(config.clone());
            let mut effect_updates = CdlEffect::default();
            effect_updates.performance.entries.push("feature_select_mrmr_stage: 10.0s".to_string());
            effect_updates.audit.entries.push(format!("Ran mRMR feature selection with config: {:?}", config));
            effect_updates.warnings.entries.push("Dominant Feature Warning: 'ICULOS' (ICU Length of Stay) has disproportionately high importance.".to_string());
            effect_updates.value = Some(next_cdl_state);
            effect_updates
        });
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
}

impl CdlPipelineBuilder<WithFeatures> {
    // Standard operator for causal discovery using SURD.
    pub fn discover_surd(self, config: SurdConfig) -> CdlPipelineBuilder<WithCausalResults> {
        println!("-> CdlPipeline: Discovering causality with SURD...");
        let next_effect = self.current_effect.bind(move |cdl_with_features| { // `move` config
            let next_cdl_state = cdl_with_features.discover_surd_impl(config.clone());
            let mut effect_updates = CdlEffect::default();
            effect_updates.performance.entries.push("discover_surd_stage: 20.0s".to_string());
            effect_updates.audit.entries.push(format!("Executed SURD-states with config: {:?}", config));
            effect_updates.value = Some(next_cdl_state);
            effect_updates
        });
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
}

impl CdlPipelineBuilder<WithCausalResults> {
    // Standard operator for analysis.
    pub fn analyze(self, config: AnalyzeConfig) -> CdlPipelineBuilder<WithAnalysis> {
        println!("-> CdlPipeline: Analyzing results...");
        let next_effect = self.current_effect.bind(move |cdl_with_results| { // `move` config
            let next_cdl_state = cdl_with_results.analyze_impl(config.clone());
            let mut effect_updates = CdlEffect::default();
            effect_updates.performance.entries.push("analyze_stage: 1.0s".to_string());
            effect_updates.audit.entries.push(format!("Analyzed results with config: {:?}", config));
            effect_updates.value = Some(next_cdl_state);
            effect_updates
        });
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
}

impl CdlPipelineBuilder<WithAnalysis> {
    // Standard operator for formatting the console output.
    pub fn format_console(self) -> CdlPipelineBuilder<Finalized> {
        println!("-> CdlPipeline: Formatting console output...");
        let next_effect = self.current_effect.bind(|cdl_with_analysis| {
            let next_cdl_state = cdl_with_analysis.format_console_impl();
            let mut effect_updates = CdlEffect::default();
            effect_updates.performance.entries.push("format_console_stage: 0.1s".to_string());
            effect_updates.audit.entries.push("Formatted final report for console output.".to_string());
            effect_updates.value = Some(next_cdl_state);
            effect_updates
        });
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
}

impl CdlPipelineBuilder<Finalized> {
    // Final `run` method to execute the pipeline and return the full CdlEffect.
    // The CdlEffect's value will now contain the final report string.
    pub fn run(self) -> CdlEffect<String> {
        println!("-> CdlPipeline: Executing assembled pipeline and retrieving final effect...");
        // This unwraps the Cdl<Finalized> and places its internal String representation
        // (the final report) into the value field of the returned CdlEffect.
        self.current_effect.bind(|cdl_finalized| {
            let mut effect_updates = CdlEffect::default();
            effect_updates.value = Some(cdl_finalized.data_representation); // Extract the final report string
            effect_updates
        })
    }
}

// Global CDL functions to kick off the pipeline builder.
pub mod CDL { // Encapsulate CdlPipelineBuilder creation
    use super::{CdlPipelineBuilder, NoData};
    pub fn pipeline(data_path: &str) -> CdlPipelineBuilder<NoData> {
        CdlPipelineBuilder::pipeline(data_path)
    }
}

// Custom operators can be generic over the state.
impl<CurrentCdlState: Clone + 'static> CdlPipelineBuilder<CurrentCdlState> {
    pub fn then<NextCdlState: Clone + 'static, F>(self, custom_operator: F) -> CdlPipelineBuilder<NextCdlState>
    where
        F: FnOnce(CdlEffect<Cdl<CurrentCdlState>>) -> CdlEffect<Cdl<NextCdlState>>,
    {
        println!("-> CdlPipeline: Executing custom operator...");
        let next_effect = custom_operator(self.current_effect);
        CdlPipelineBuilder { current_effect: next_effect, data_path: self.data_path }
    }
}
```

#### 2. The "Single Block" Standard Pipeline in Practice

This demonstrates how a standard Sepsis causal discovery pipeline would look in this new, cohesive API.

```rust
// How it would be used in `main.rs` (or a similar entry point)

// Assume CDL module (which contains pipeline entry and builder) is properly imported from lib.rs
// use deep_causality_discovery::CDL; 

// // Stage-specific configuration structs are also defined above and accessible.
// use deep_causality_discovery::{
//     CsvConfig, MrmrConfig, SurdConfig, AnalyzeConfig, MaxOrder
// };

fn run_sepsis_discovery_standard() {
    println!("--- Running Sepsis CDL (Standard Block-Based API) ---");
    let data_path = "examples/case_study_icu_sepsis/data/seperated/seps_true.parquet";
    
    let final_effect: CdlEffect<String> = CDL::pipeline(data_path)
        .load_parquet() // Operator for Parquet, with no explicit config needed for default
        .feature_select_mrmr(MrmrConfig::new(15, 41)) // Stage-specific configuration
        .discover_surd(SurdConfig::new(MaxOrder::Max, 41)) // Stage-specific configuration
        .analyze(AnalyzeConfig::new(0.1, 0.1, 0.1)) // Stage-specific configuration
        .format_console() // A simple, parameter-less operator
        .run(); // Execute the pipeline and get the final CdlEffect

    println!("\n--- Result of Standard Pipeline ---");
    println!("Value: {:?}", final_effect.value);
    println!("Warnings: {:?}", final_effect.warnings.entries);
    println!("Audit Trail: {:?}", final_effect.audit.entries);
    println!("Performance: {:?}", final_effect.performance.entries);
    println!("Errors: {:?}", final_effect.error);
}
```

#### 3. Customization with User-Defined Operators

The `CdlPipelineBuilder` would also feature a generic `.then()` method, allowing users to define their own custom operators and seamlessly integrate them into the block-based flow. This allows for arbitrary custom logic or replacement of standard steps, while preserving type-safety of the pipeline stages.

```rust
// A custom operator function defined by the user.
// It takes the previous CdlEffect (containing Cdl<WithData>) and custom parameters,
// and returns a new CdlEffect (containing Cdl<WithFeatures>).
// This function must respect the input/output type of the `then` method.
fn custom_feature_selector_with_validation(
    input_effect: CdlEffect<Cdl<WithData>>,
    mrmr_config: MrmrConfig,
    min_feature_count: usize,
) -> CdlEffect<Cdl<WithFeatures>> {
    println!("\n  [Custom Operator]: Executing custom_feature_selector_with_validation...");
    
    let mut next_effect = input_effect.clone_effects_without_value(); 
    
    if let Some(cdl_with_data) = input_effect.value {
        // --- Custom Logic ---
        println!("  [Custom Logic]: Validating minimum feature count before feature selection...");
        if mrmr_config.num_features() < min_feature_count {
            next_effect.error = Some(format!("Custom Error: Requested features ({}) below minimum allowed ({}). Aborting feature selection.", 
                                             mrmr_config.num_features(), min_feature_count));
            next_effect.audit.entries.push("Custom operator aborted due to insufficient feature count.".to_string());
            next_effect.value = None; 
        } else {
            // If validation passes, delegate to the standard feature selection implementation
            let next_cdl_state = cdl_with_data.feature_select_mrmr_impl(mrmr_config.clone());
            next_effect.value = Some(next_cdl_state);
            next_effect.performance.entries.push("custom_feature_select_stage: 12.0s".to_string());
            next_effect.audit.entries.push(format!("Custom operator applied mRMR with config: {:?}", mrmr_config));
            next_effect.warnings.entries.push("Custom warning: Feature validation passed.".to_string());
        }
    } else {
        next_effect.error = input_effect.error;
        next_effect.value = None;
    }
    next_effect
}


// How to use it in `main.rs`
fn run_sepsis_discovery_custom() {
    println!("\n--- Running Sepsis CDL (Custom Operator Example) ---");
    let data_path = "examples/case_study_icu_sepsis/data/seperated/seps_true.parquet";
    
    let final_effect: CdlEffect<String> = CDL::pipeline(data_path)
        .load_parquet()
        // Plug in our custom operator 
        .then(|effect_from_loader| {custom_feature_selector_with_validation(effect_from_loader,  MrmrConfig::new(15, 41), 5)})
        .discover_surd(SurdConfig::new(MaxOrder::Max, 41))
        .analyze(AnalyzeConfig::new(0.1, 0.1, 0.1))
        .format_console()
        .run();

    println!("\n--- Result of Custom Pipeline ---");
    println!("Value: {:?}", final_effect.value);
    println!("Warnings: {:?}", final_effect.warnings.entries);
    println!("Audit Trail: {:?}", final_effect.audit.entries);
    println!("Errors: {:?}", final_effect.error);
}
```

This merged, block-based design offers significant advantages:
*   **Improved Cohesion**: The entire pipeline, from configuration to execution, is expressed in a single, flowing block of code, enhancing readability and maintainability.
*   **Stage-Specific Configuration**: Parameters are passed directly to the operators that consume them, simplifying the configuration process and reducing the need for large, monolithic config objects.
*   **Seamless Customization**: Users can define and reuse their own custom operators that fit perfectly into the fluent API, allowing for powerful extensions without breaking the cohesive flow.
*   **Type-State Preservation**: The `CdlPipelineBuilder<CurrentCdlState>` ensures that the pipeline always moves through valid state transitions, leveraging Rust's type system for compile-time guarantees, even with custom operators.

---