# 000-Pre-Specs: HKT-Based Causal Discovery Language (CDL)

## 1. Introduction

The current Causal Discovery Language (CDL) in `deep_causality_discovery` provides a type-state driven pipeline for causal analysis. It ensures a correct sequence of operations (e.g., `load_data` -> `feature_select`) through Rust's type system. Error handling is primarily achieved via `Result<_, CdlError>` and the `?` operator, which provides immediate short-circuiting on failure.

While robust, this approach has limitations:
*   It primarily handles a single failure channel (an `Err` of type `CdlError`). There's no inherent mechanism to accumulate non-fatal issues like warnings or informational messages throughout the pipeline without resorting to side-effects (e.g., logging to a global logger) or manual aggregation outside the `Result` type.
*   The composability, while effective with `?`, could be further formalized and extended using functional programming paradigms, enabling richer effect tracking.

This specification proposes an enhancement to the CDL by integrating concepts from Higher-Kinded Types (HKT) and monadic composition, leveraging the `deep_causality_haft` crate. This will allow for the development of a "type-encoded effect system" within the CDL, enabling more expressive and robust pipeline execution with explicit, composable side-effect management.

## 2. Core Concepts from `deep_causality_haft`

The `deep_causality_haft` crate offers the following foundational traits:

*   **Higher-Kinded Types (HKT)**: Traits like `HKT`, `HKT2`, `HKT3`, `HKT4`, `HKT5` allow us to abstract over type constructors (e.g., `Option<T>`, `Result<T, E>`). This enables writing generic code that works across different container types without knowing their inner type `T`.
*   **Functor**: Defines the `fmap` operation, which applies a function to the value *inside* a container, preserving its structure.
*   **Applicative**: Extends `Functor` by providing `pure` (to lift a pure value into a container) and `apply` (to apply a function *within* a container to a value *within* a container).
*   **Monad**: Extends `Applicative` with a `bind` operation. `bind` is the core sequencing primitive; it takes a container of a value and a function that returns another container, flattening the nested containers. This is crucial for chaining computations that might have side-effects.
*   **CoMonad**: The dual of a `Monad`. It provides `extract` to get the value at the focus of the context, and `extend` to derive a new comonadic context by applying a function that observes the original context. Useful for inspecting and transforming contexts based on their content.
*   **Traversable**: Enables "sequencing" a structure of monadic values (`F<M<A>>`) into a monadic value of a structure (`M<F<A>>`). This is powerful for orchestrating multiple effectful computations, allowing uniform error propagation and result collection across a collection.
*   **Type-Encoded Effect Systems (`EffectN`, `MonadEffectN`)**: These traits are designed to define custom "effect monads" that can explicitly track multiple types of side-effects (e.g., errors, warnings, logs, metrics) alongside a primary value, all within the type system. An `EffectN` trait defines the fixed types for `N-1` effect channels, and `MonadEffectN` provides `pure` and `bind` implementations for that specific effect system.

## 3. Proposed Architecture: The `CdlEffect` Monad

We will introduce a new core type, `CdlEffect`, which will act as our custom effect monad. This type will wrap the `CDL`'s intermediate states (e.g., `CDL<WithData>`) and explicitly carry a single fatal error (if any) and an accumulated log of warnings. This design aligns with existing effect system implementations within the `deep_causality` project, emphasizing short-circuiting on fatal errors while collecting non-fatal issues.

### 3.1. `CdlWarningLog` Type Definition

To manage accumulated warnings, we introduce a dedicated log type:

// CdlWarning Definition
#[derive(Debug, Clone, PartialEq)]
pub enum CdlWarning {
    DataIssue(String),
    FeatureIssue(String),
    ModelIssue(String),
    Generic(String),
}

impl From<&str> for CdlWarning {
    fn from(s: &str) -> Self {
        CdlWarning::Generic(s.to_string())
    }
}

// CdlWarningLog Definition
#[derive(Debug, Clone, Default)]
pub struct CdlWarningLog {
    pub entries: Vec<CdlWarning>,
}

// Implement traits from deep_causality_haft::effect_system::effect_log

impl LogAddEntry for CdlWarningLog {
    fn add_entry(&mut self, message: &str) {
        self.entries.push(CdlWarning::from(message));
    }
}

impl LogAppend for CdlWarningLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

impl LogSize for CdlWarningLog {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

// Marker trait implementation
impl LogEffect for CdlWarningLog {}


### 3.2. `CdlEffect` Type Definition

The core `CdlEffect` structure will be:

```rust
pub struct CdlEffect<T> { // E and W types fixed at the HKT witness level
    pub inner: Result<T, CdlError>, // Enforces valid state: either a Value or an Error.
    pub warnings: CdlWarningLog,    // Accumulated warnings, always present.
}

impl<T> CdlEffect<T> {
    /// Convenience method to print accumulated warnings.
    /// Prints "No Warnings" if empty, typically to stdout.
    pub fn print_warnings(&self) {
        if self.warnings.entries.is_empty() {
            println!("No Warnings");
        } else {
            println!("Pipeline Warnings:");
            for warning in &self.warnings.entries {
                println!(" - {:?}", warning);
            }
        }
    }
}
```
*   `T`: The type of the successful result (e.g., `CDL<WithData>`).
*   `CdlError`: The error type.
*   `CdlWarningLog`: The accumulated warnings.

### 3.3. `CdlEffectWitness` and `CdlBuilder`

### 3.3. `CdlEffectWitness` and `CdlBuilder`

To integrate `CdlEffect` with HAFT and `deep_causality`'s conventions, we define the Witness and Builder types and implement the necessary traits.

```rust
use std::marker::PhantomData;

// --- 3.3.1 CdlEffectWitness Definition ---

// The Witness struct holding the fixed types (Error and WarningLog)
// and a phantom generic placeholder.
pub struct CdlEffectWitness<E, W_Log>(PhantomData<(E, W_Log)>);

// Implement HKT3: "If you give me a T, I will give you back a CdlEffect<T> with fixed E and W_Log"
impl<E, W_Log> HKT for CdlEffectWitness<E, W_Log> {}

impl<E, W_Log> HKT3<E, W_Log, dyn Any> for CdlEffectWitness<E, W_Log>
where
    E: 'static,
    W_Log: 'static,
{
    // The associated type that this witness "witnesses"
    type Type<T> = CdlEffect<T>;
}


// --- 3.3.2 CdlBuilder Definition (Effect3) ---

// The Builder struct connecting the Effect system to the Witness
pub struct CdlBuilder;

// Implement Effect3: Fixing the Error and Warning types for the system.
impl Effect3 for CdlBuilder {
    type Fixed1 = CdlError;
    type Fixed2 = CdlWarningLog;
    type HktWitness = CdlEffectWitness<Self::Fixed1, Self::Fixed2>;
}
```

#### 3.3.3. Monad Implementation

This implements `Functor`, `Applicative`, and `Monad` for the Witness, allowing `CdlEffect` to be used in generic HKT contexts.

```rust
// 1. Functor: fmap
impl<E, W_Log> Functor<CdlEffectWitness<E, W_Log>> for CdlEffectWitness<E, W_Log>
where
    E: Clone,
    W_Log: Clone, // Minimal requirement for preserving warnings
{
    fn fmap<A, B, F>(m_a: CdlEffect<A>, f: F) -> CdlEffect<B>
    where
        F: Fn(A) -> B,
    {
        CdlEffect {
            inner: m_a.inner.map(f),
            warnings: m_a.warnings, // Warnings are preserved
        }
    }
}

// 2. Applicative: pure and apply
impl<E, W_Log> Applicative<CdlEffectWitness<E, W_Log>> for CdlEffectWitness<E, W_Log>
where
    E: Clone,
    W_Log: Clone + LogAppend + Default,
{
    fn pure<T>(value: T) -> CdlEffect<T> {
        CdlEffect {
            inner: Ok(value),
            warnings: W_Log::default(),
        }
    }

    fn apply<A, B, F>(
        f_ab: CdlEffect<F>,
        mut m_a: CdlEffect<A>,
    ) -> CdlEffect<B>
    where
        F: Fn(A) -> B,
    {
        let mut combined_warnings = f_ab.warnings;
        combined_warnings.append(&mut m_a.warnings);

        let new_inner = match (f_ab.inner, m_a.inner) {
            (Ok(func), Ok(val)) => Ok(func(val)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        };

        CdlEffect {
            inner: new_inner,
            warnings: combined_warnings,
        }
    }
}

// 3. Monad: bind
impl<E, W_Log> Monad<CdlEffectWitness<E, W_Log>> for CdlEffectWitness<E, W_Log>
where
    E: Clone,
    W_Log: Clone + LogAppend + Default,
{
    fn bind<A, B, F>(m_a: CdlEffect<A>, f: F) -> CdlEffect<B>
    where
        F: Fn(A) -> CdlEffect<B>,
    {
        match m_a.inner {
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: m_a.warnings,
            },
            Ok(val) => {
                let mut m_b = f(val);
                let mut combined_warnings = m_a.warnings;
                // Append warnings from the result of the bound function
                combined_warnings.append(&mut m_b.warnings);

                CdlEffect {
                    inner: m_b.inner,
                    warnings: combined_warnings,
                }
            }
        }
    }
}

// Impl simplified new() for builder
impl CdlBuilder {
    pub fn new() -> CdlEffect<CDL<NoData>> {
         Self::pure(CDL::default())
    }
}
```


## 4. Pipeline Flow with `CdlEffect`

The CDL pipeline would be refactored to operate within this `CdlEffect` monad. Each stage would consume a `CdlEffect<CDL<CurrentState>>` and produce a `CdlEffect<CDL<NextState>>`.

1.  **Initialization**:
    // CdlBuilder::new() initializes with default configuration.
    // Determining specific config would be done via methods on the builder or the CDL object itself before binding.
    // For this example, we assume new() is sufficient or we can pass config to new().
    let initial_effect: CdlEffect<CDL<NoData>> = CdlBuilder::new();

2.  **Chaining Stages with `bind`**:
    Each subsequent stage would then be chained using the `bind` method. This `bind` is an explicit operation that transforms the value inside the `CdlEffect` and automatically handles the effect channels (fatal error propagation and warning accumulation).

    ```rust
    let final_effect = initial_effect
        .bind(|cdl_no_data| {
            // Inside bind, cdl_no_data is CDL<NoData>.
            // The load_data function itself would return a CdlEffect<CDL<WithData>>.
            // Any fatal error from load_data would be wrapped in the CdlEffect's error field.
            // Any warnings from load_data would be wrapped in the CdlEffect's warnings log.
            cdl_no_data.load_data(DataLoader, "path/to/data.csv")
                // A stage could also explicitly add warnings, e.g.:
                // .with_warning(CdlWarning::MinorDataInconsistency)
        })
        .bind(|cdl_with_data| cdl_with_data.preprocess(DataDiscretizer))
        .bind(/* ... other stages ... */)
        .bind(|cdl_runner| cdl_runner.run()); // Final stage produces CdlEffect<ProcessFormattedResult>
    ```

3.  **Intermediate Stage Output**:
    Each stage's function (e.g., `load_data`, `preprocess`) would no longer return `Result<CDL<State>, CdlError>` but rather `CdlEffect<CDL<State>>`. This means they can signal a fatal error (which short-circuits) and accumulate warnings.

4.  **Final Extraction**:
    At the end of the entire pipeline, the consumer retrieves the result from the final `CdlEffect` object. This involves checking the `error` field and the `warnings` log:
    ```rust
    match final_effect.inner {
        Err(fatal_error) => {
            println!("--- Pipeline Failed with Fatal Error ---");
            println!("Error: {:?}", fatal_error);
        },
        Ok(result) => {
             println!("Causal Discovery Result: {:?}", result);
        }
    }
    }
    // Always report warnings using the convenience method.
    final_effect.print_warnings();
    ```

## 5. Detailed Stage-by-Stage Considerations (Conceptual)

Each existing CDL stage would be modified to return a `CdlEffect` instance, encapsulating its outcome.

*   **`load_data`**:
    *   **Errors**: `DataLoadingError` (e.g., `FileNotFound`, `PermissionDenied`, `CsvError`, `ParquetError`) would be converted into a `CdlError` and wrapped in `CdlEffect::error`. This would typically short-circuit the pipeline.
    *   **Warnings**: Data-specific issues like empty lines, non-numeric values converted to `NaN` (if handled leniently), or columns being skipped, could be reported as `CdlWarning` entries in `CdlEffect::warnings`.

*   **`preprocess`**:
    *   **Errors**: `PreprocessError` (e.g., `BinningError` with invalid configuration, `ImputeError` if imputation is impossible) would be wrapped in `CdlEffect::error`.
    *   **Warnings**: Information about columns with high numbers of imputed values, or columns skipped from binning due to low variance, would be added to `CdlEffect::warnings`.

*   **`feature_select`**:
    *   **Errors**: `FeatureSelectError` (e.g., `TooFewFeatures`, `MrmrError` indicating algorithm failure) would be wrapped in `CdlEffect::error`.
    *   **Warnings**: Could include messages about highly correlated features that were implicitly removed, or a low number of features selected, suggesting a potentially poor model.

*   **`causal_discovery`**:
    *   **Errors**: `CausalDiscoveryError` (e.g., `TensorError` due to inconsistent data post-selection, or algorithm non-convergence) would be wrapped in `CdlEffect::error`.
    *   **Warnings**: Messages about weak statistical significance of some discovered relationships, or potential instability in results, could be added to `CdlEffect::warnings`.

*   **`analyze`**:
    *   **Errors**: `AnalyzeError` (e.g., `EmptyResult` if discovery yielded nothing, `AnalysisFailed` due to internal logic) would be wrapped in `CdlEffect::error`.
    *   **Warnings**: Thresholds for synergy/redundancy/uniqueness being barely met, or the presence of many near-zero influence values, would be added to `CdlEffect::warnings`.

*   **`finalize`**:
    *   **Errors**: `FinalizeError` (e.g., `FormattingError` if the output cannot be rendered) would be wrapped in `CdlEffect::error`.
    *   **Warnings**: Minor formatting adjustments or truncations for display could be noted here.

## 6. Benefits of HKT-Based CDL

*   **Explicit Effect Handling**: The type signature of `CdlEffect<T>` (with its fixed `CdlError` and `CdlWarningLog` types) makes it immediately clear that a CDL computation can produce a value `T`, a single fatal error, and a log of non-fatal warnings. This forces developers to consider and handle all these effects.
*   **Enhanced Composability**: The `bind` operation provides a clean, consistent, and mathematically sound way to chain pipeline stages. The underlying monadic implementation handles the boilerplate of fatal error propagation (short-circuiting) and warning accumulation, leading to more readable and maintainable code.
*   **Increased Robustness**: Non-fatal issues (warnings) no longer need to halt pipeline execution. The pipeline can proceed, accumulating diagnostic information that is invaluable for debugging and understanding results.
*   **Improved Diagnostics**: Users receive a comprehensive report at the end, detailing not only the final result (or the primary fatal error) but also a full history of all warnings encountered, ordered by the stage they occurred in.
*   **Greater Flexibility**: The `CdlEffect` can be extended to include other effect channels (e.g., performance metrics, audit trails) without fundamentally altering the pipeline's structure.

## 7. Future Considerations and Advanced Usage

### 7.1. Orchestrating Multiple Pipelines with `Traversable`

The `Traversable` trait offers a powerful mechanism for managing collections of `CdlEffect` instances. If we have multiple independent causal discovery pipelines (e.g., for different datasets or configurations), each yielding a `CdlEffect<ProcessFormattedResult>`, `Traversable` can be used to aggregate their results.

For example, given a `Vec<CdlEffect<ProcessFormattedResult>>`, applying `Traversable::sequence` (via `VecWitness`) could transform this into `CdlEffect<Vec<ProcessFormattedResult>>`. This means:
*   If any individual `CdlEffect` in the vector contains a fatal error, the resulting `CdlEffect<Vec<ProcessFormattedResult>>` will contain that fatal error (the first one encountered).
*   All warnings from all individual `CdlEffect`s (whether they succeeded or failed) will be accumulated into the final `CdlEffect`'s `CdlWarningLog`.
*   If all individual `CdlEffect`s succeed, the final `CdlEffect` will contain a `Vec<ProcessFormattedResult>` along with all accumulated warnings.

This provides a unified and type-safe way to manage error propagation and warning collection across batch processing or parallel execution of CDL tasks.

### 7.2. Inspecting and Deriving from `CdlEffect` with `CoMonad`

The `CoMonad` trait, with its `extract` and `extend` operations, offers capabilities for observing and deriving new contexts from a `CdlEffect`.

*   **`extract`**: After a pipeline has run and produced a `CdlEffect<ProcessFormattedResult>`, `extract` could be used to directly retrieve the `ProcessFormattedResult` (if present) for immediate use, while the full `CdlEffect` (with errors and warnings) remains available for inspection.
*   **`extend`**: This allows creating new `CdlEffect` instances (e.g., an `CdlEffect<SummaryReport>`) by observing a completed `CdlEffect<ProcessFormattedResult>`. The observation function could analyze the `ProcessFormattedResult` along with its associated warnings and errors to generate a high-level summary or derive a new pipeline configuration. This could be particularly useful for adaptive systems that react to the outcomes of previous analyses.

### 7.3. General Considerations

*   **Customizable Error/Warning Policies**: Users could configure whether a certain type of warning should be elevated to an error, or if certain errors are degradable to warnings.
*   **Parallel Processing Integration**: Parallel execution is already supported via the `parallel` feature flag, which compiles all algorithms to run in parallel using `rayon`. Monadic structures naturally compose with this existing capability.
* 
### 7.3. Advanced Reporting: The `CdlReport` System

To provide structured, readable access to all accumulated information from the pipeline, we define a dedicated `CdlReport` type. This struct aggregates metadata and the specific result types from feature selection (`MrmrResult`) and causal discovery (`SurdResult`).

#### 7.3.1. `CdlReport` Definition

```rust
use std::fmt::{self, Display, Formatter};
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::causal_discovery::surd::SurdResult;

/// Aggregates all significant findings from a CDL pipeline execution.
#[derive(Debug)]
pub struct CdlReport {
    // 1. Data Metadata
    pub dataset_path: String,
    pub records_processed: usize,

    // 2. Feature Selection Result
    pub feature_selection: MrmrResult,

    // 3. Causal Discovery Result (assuming f64 precision for this example)
    pub causal_analysis: SurdResult<f64>,
}

impl Display for CdlReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "==========================================================")?;
        writeln!(f, "               DEEP CAUSALITY: ANALYSIS REPORT             ")?;
        writeln!(f, "==========================================================")?;
        
        writeln!(f, "\n[1] DATASET SUMMARY")?;
        writeln!(f, "    File: .............. {}", self.dataset_path)?;
        writeln!(f, "    Records: ........... {}", self.records_processed)?;

        writeln!(f, "\n[2] FEATURE SELECTION (MRMR)")?;
        // Delegate to MrmrResult's Display implementation
        write!(f, "{}", self.feature_selection)?;

        writeln!(f, "\n[3] CAUSAL DISCOVERY (SURD)")?;
        // Delegate to SurdResult's Display implementation
        write!(f, "{}", self.causal_analysis)?;
        
        writeln!(f, "\n==========================================================")?;
        Ok(())
    }
}
```

#### 7.3.2. Integration with `finalize`

The `finalize` stage of the pipeline transforms the internal `CDL` state (which holds the results from previous steps) into this user-friendly `CdlReport`.

```rust
// Conceptual implementation of finalize within the CDL pipeline
/*
impl CDL<Analyzed> {
    pub fn finalize(self) -> CdlReport {
        CdlReport {
            dataset_path: self.config.data_path.clone(),
            records_processed: self.data.len(),
            // Extract the MrmrResult stored in the state (from feature_select step)
            feature_selection: self.feature_selection_result,
            // Extract the SurdResult stored in the state (from determine_causality/analyze step)
            causal_analysis: self.causal_result,
        }
    }
}
*/
```

By adopting an HKT-based, monadic approach with the added power of `Traversable` and `CoMonad`, the CDL can become a more powerful, flexible, and transparent framework for causal discovery.

## 8. Migration Guide: Updating Existing Examples

This section demonstrates how to rewrite the existing example found in `deep_causality_discovery/examples/main.rs` to use the new HKT-based API.

### 8.1. Before (Current Type-State API)

The current API relies on a heavy configuration object (`CdlConfig`) and uses `Result` chaining with `.expect()` for error handling.

```rust
// deep_causality_discovery/examples/main.rs (Simplified)

fn main() {
    let file_path = "test_data.csv";

    // 1. Extensive Configuration
    let cdl_config = CdlConfig::new()
        .with_data_loader(DataLoaderConfig::Csv(CsvConfig::default()))
        // Config: Select 2 features, target index 3
        .with_feature_selector(FeatureSelectorConfig::Mrmr(MrmrConfig::new(2, 3)))
        // Config: SURD with Max order, target index 3
        .with_causal_discovery(CausalDiscoveryConfig::Surd(SurdConfig::new(Max, 3)))
        .with_analysis(AnalyzeConfig::new(0.1, 0.1, 0.1));

    // 2. Execution with Type-State Chaining and Panic-on-Error (.expect)
    let discovery_process = CDL::with_config(cdl_config)
        .load_data(CsvDataLoader, &file_path).expect("Fail")
        .feature_select(MrmrFeatureSelector).expect("Fail")
        .causal_discovery(SurdCausalDiscovery).expect("Fail")
        .analyze(SurdResultAnalyzer).expect("Fail")
        .finalize(ConsoleFormatter).expect("Fail")
        .build().expect("Fail");

    // 3. Run and Handle Result
    let result = discovery_process.run();
    match result {
        Ok(res) => println!("Result: {}", res),
        Err(e) => println!("Error: {}", e),
    }
}
```

### 8.2. After (HKT CdlEffect API)

The new API moves configuration inline for conciseness and uses monadic binding to handle errors and warnings essentially.

```rust
// New HKT-based Implementation

use deep_causality_discovery::*;

fn main() {
    let file_path = "test_data.csv";
    
    // 1. Initialization (No complex config upfront)
    let pipeline = CdlBuilder::new();

    // 2. Monadic Chain with Inline Config
    let effect = pipeline
        // load_data directly takes arguments (path, target_idx, excluded_cols)
        // Assuming target index 3, no exclusions for this simple CSV
        .bind(|cdl| cdl.load_data(file_path, 3, vec![]))
        
        // feature_select takes a closure using the specific algorithm
        // mrmr_features_selector(tensor, num_features, target_col)
        .bind(|cdl| cdl.feature_select(|tensor| {
            mrmr_features_selector(tensor, 2, 3)
        }))
        
        // causal_discovery takes a closure using the specific algorithm
        // surd_states_cdl(tensor, max_order)
        .bind(|cdl| cdl.causal_discovery(|tensor| {
             surd_states_cdl(tensor, MaxOrder::Max)
        }))
        
        // analyze uses default or inline config
        .bind(|cdl| cdl.analyze()) // Config like (0.1, 0.1, 0.1) could be passed here if needed
        
        // finalize converts to the output format
        .bind(|cdl| cdl.finalize());

    // 3. Explicit Effect Handling (Result + Warnings)
    match effect.inner {
        Ok(formatted_result) => println!("Result: {}", formatted_result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 4. Convenience Warning Check
    effect.print_warnings();
}
```

### 8.3. Key Changes Summary

1.  **Configuration**: Moved from a pre-built `CdlConfig` struct to *inline arguments* passed directly to the stage methods (or the algorithms they invoke). This reduces boilerplate.
2.  **Flow Control**: Replaced `Result::expect` chaining (which panics) with `Monad::bind` (which safely propagates errors and accumulates warnings).
3.  **Algorithms**: Replaced abstract enum variants (`FeatureSelectorConfig::Mrmr`) with direct calls to algorithm functions (`mrmr_features_selector`), increasing transparency.
4.  **Diagnostics**: Added `print_warnings()` to explicitly surface non-fatal issues that were previously lost or required side-channel logging.

## 9. Comprehensive Testing Strategy

To ensure robustness and reliability, the CDL implementation must strictly adhere to the project's testing conventions as outlined in `AGENTS.md`. We aim for **100% test coverage** across all components.

### 9.1. Test Structure and Organization

Following the "mirroring" convention, tests will be placed in a `tests/` directory that replicates the `src/` structure.

*   `src/cdl/builder.rs` -> `tests/cdl/builder/builder_tests.rs` (Tests `CdlBuilder` initialization)
*   `src/cdl/effect_impl.rs` -> `tests/cdl/effect_hkt/effect_tests.rs` (Tests `CdlEffect` logic: bind, map, apply)
*   `src/cdl/report.rs` -> `tests/cdl/report/report_tests.rs` (Tests `CdlReport` creation and formatting)
*   `tests/integration_tests.rs` (Full pipeline verification)

### 9.2. Unit Testing Strategy

#### 9.2.1. `CdlEffect` Monad Logic
*   **Monadic Laws**: Verify `Left Identity`, `Right Identity`, and `Associativity` for `bind`.
*   **Effect Propagation**:
    *   **Success Path**: Verify values transform correctly through a chain of `map` and `bind` calls.
    *   **Error Path**: Verify that a single error in the chain short-circuits execution and is returned as the final result.
    *   **Warning Accumulation**: Verify that warnings from *each* step are preserved and aggregated in the correct order, regardless of whether the final result is `Ok` or `Err`.

#### 9.2.2. `CdlBuilder`
*   **Initialization**: Ensure `CdlBuilder::new()` returns a clean state with no errors or warnings.

#### 9.2.3. `CdlReport`
*   **Display Formatting**: Verify that the `Display` implementation produces the exact expected string output for a given report, including proper handling of empty lists (e.g., no ignored features).

### 9.3. Integration Testing with Mock Algorithms

To test the CDL pipeline logic without relying on expensive real-world algorithms (like true MRMR or SURD computations) or external data files, we will use **Mock Algorithms** in our integration tests.

*   **Mock Feature Selector**: A closure that returns a hardcoded list of selected features (e.g., indices `[1, 2]`) or a specific error.
*   **Mock Causal Discovery**: A closure that returns a hardcoded `SurdResult` or error.

**Example Test Scenario**:
```rust
#[test]
fn test_full_pipeline_success() {
    let pipeline = CdlBuilder::new();
    let result = pipeline
         // Mock Load: Returns dummy tensor
        .bind(|_| mock_load_data()) 
         // Mock Feature Select: Returns fixed MrmrResult
        .bind(|t| mock_feature_select(t))
         // Mock Causal Discovery: Returns fixed SurdResult
        .bind(|t| mock_causal_discovery(t)) 
        .bind(|s| mock_analyze(s))
        .bind(|a| mock_finalize(a));

    assert!(result.inner.is_ok());
    assert_eq!(result.warnings.len(), 0);
    // Assert specific fields in the final CdlReport match the mock data
}
```

### 9.4. Error and Warning Scenarios

We must explicitly test edge cases:
1.  **Early Failure**: Pipeline failing at the `load_data` step. Assert subsequent steps are NOT executed.
2.  **Mid-Pipeline Failure**: Pipeline failing at `causal_discovery`. Assert warnings from previous steps (`load_data`, `feature_select`) are still preserved in the final failure result.
3.  **Warning-Only Run**: A pipeline that succeeds but accumulates multiple warnings (e.g., "Feature X ignored", "Low data variance"). Assert `inner` is `Ok` and `warnings` contains all messages.

By decoupling the pipeline logic from the heavy numerical algorithms via closures, we can achieve 100% coverage of the CDL framework itself efficiently.