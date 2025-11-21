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

```rust
// Conceptual CdlWarningLog (similar to ModificationLog or CausalEffectLog)
#[derive(Debug, Clone, Default)]
pub struct CdlWarningLog {
    pub entries: Vec<CdlWarning>, // Assuming CdlWarning is a simple enum or struct
}

impl CdlWarningLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.extend(other.entries.drain(..));
    }
}
```
*   `CdlWarning`: A new enum or struct defining specific warning types (e.g., `MissingHeaders`, `ColumnSkipped`).

### 3.2. `CdlEffect` Type Definition

The core `CdlEffect` structure will be:

```rust
pub struct CdlEffect<T> { // E and W types fixed at the HKT witness level
    pub value: Option<T>,        // The successful result of the computation. None if a fatal error occurred.
    pub error: Option<CdlError>, // The first fatal error encountered, which short-circuits computation.
    pub warnings: CdlWarningLog, // Accumulated warnings, non-fatal.
}
```
*   `T`: The type of the successful result of a CDL stage (e.g., `CDL<WithData>`, `ProcessFormattedResult`).
*   `CdlError`: The existing `CdlError` enum from `deep_causality_discovery`.
*   `CdlWarningLog`: The newly defined log for warnings.

### 3.3. `CdlEffectWitness` and `CdlEffectImpl`

To integrate `CdlEffect` with HAFT and `deep_causality`'s conventions, we would define:

*   **`CdlEffectWitness<CdlError, CdlWarningLog>`**: An HKT witness struct (`Placeholder, CdlError, CdlWarningLog`) that implements `HKT3<CdlError, CdlWarningLog>`, specifying `type Type<T> = CdlEffect<T>;`. This fixes the error and warning log types, leaving `T` as the generic parameter.
*   **`CdlEffectImpl`**: A unit struct that implements `Effect3` for `CdlEffectWitness` (fixing the error and warning types). It would then implement `Functor`, `Applicative`, and `Monad` for its `CdlEffectWitness`, defining how `fmap`, `pure`, `apply`, and `bind` behave for our custom effect.

#### 3.3.1. `Monad::bind` Implementation Logic

The `bind` implementation for `CdlEffect` is critical for ensuring correct short-circuiting and effect accumulation:

```rust
// Conceptual Monad::bind for CdlEffectWitness
impl<E, W_Log> Monad<CdlEffectWitness<E, W_Log>> for CdlEffectWitness<E, W_Log>
where
    E: Clone, // Errors need to be cloneable for propagation
    W_Log: Clone + LogAppend + Default, // Warning logs need to be cloneable, appendable, and default-constructible
{
    fn bind<A, B, Func>(m_a: CdlEffect<A, E, W_Log>, mut f: Func) -> CdlEffect<B, E, W_Log>
    where
        A: SomeValue, // Assuming A has a trait indicating it's the 'value' part
        Func: FnMut(A) -> CdlEffect<B, E, W_Log>,
    {
        // 1. Short-circuit on fatal error:
        // If the initial effect `m_a` already contains an error,
        // prevent further computation on the value `A`.
        // The error and accumulated warnings from `m_a` are directly propagated.
        if m_a.error.is_some() {
            return CdlEffect {
                value: None, // No meaningful value on error
                error: m_a.error,
                warnings: m_a.warnings,
            };
        }

        // 2. Proceed only if a valid value is present:
        if let Some(val) = m_a.value {
            let mut m_b = f(val); // Execute the next function `f` with the unwrapped value `A`.

            // 3. Combine warnings:
            // Append warnings from `m_a` to `m_b`'s warnings.
            let mut combined_warnings = m_a.warnings;
            combined_warnings.append(&mut m_b.warnings);

            // 4. Construct the new CdlEffect:
            CdlEffect {
                value: m_b.value,   // The value from the result of `f`.
                error: m_b.error,   // The error from the result of `f` (short-circuits next bind).
                warnings: combined_warnings, // All warnings combined.
            }
        } else {
            // This case implies `m_a.error` is `None` but `m_a.value` is `None`.
            // This is an unexpected or invalid state for a healthy pipeline.
            // It should ideally be an internal error or handled by the previous stage.
            // For robustness, we propagate the current (empty) state with a new error.
            CdlEffect {
                value: None,
                error: Some(E::from_string("Unexpected empty value in successful state")), // Conceptual: E needs From<String>
                warnings: m_a.warnings,
            }
        }
    }
}
```

This `bind` implementation ensures:
*   Fatal errors are short-circuited efficiently.
*   Non-fatal warnings are continuously accumulated across the entire pipeline.
*   The `value` field correctly reflects the presence or absence of a meaningful result.

#### 3.3.2. `Applicative::apply` Implementation Logic

The `apply` method combines two effects, one holding a function and another holding an argument.

```rust
// Conceptual Applicative::apply for CdlEffectWitness
impl<E, W_Log> Applicative<CdlEffectWitness<E, W_Log>> for CdlEffectWitness<E, W_Log>
where
    E: Clone,
    W_Log: Clone + LogAppend + Default,
{
    fn pure<T>(value: T) -> CdlEffect<T, E, W_Log> {
        CdlEffect {
            value: Some(value),
            error: None,
            warnings: W_Log::default(),
        }
    }

    fn apply<A, B, Func>(
        mut f_ab: CdlEffect<Func, E, W_Log>,
        mut m_a: CdlEffect<A, E, W_Log>,
    ) -> CdlEffect<B, E, W_Log>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        // Combine warnings from both effects first.
        let mut combined_warnings = f_ab.warnings;
        combined_warnings.append(&mut m_a.warnings);

        // Propagate the first error encountered.
        if f_ab.error.is_some() {
            return CdlEffect {
                value: None,
                error: f_ab.error,
                warnings: combined_warnings,
            };
        }
        if m_a.error.is_some() {
            return CdlEffect {
                value: None,
                error: m_a.error,
                warnings: combined_warnings,
            };
        }

        // If both are successful, apply the function.
        if let (Some(func), Some(val)) = (f_ab.value, m_a.value) {
            CdlEffect {
                value: Some(func(val)),
                error: None,
                warnings: combined_warnings,
            }
        } else {
            // This implies no error but one or both values were None, which is an inconsistent state.
            CdlEffect {
                value: None,
                error: Some(E::from_string("Inconsistent state: missing value for apply")), // Conceptual: E needs From<String>
                warnings: combined_warnings,
            }
        }
    }
}
```

## 4. Pipeline Flow with `CdlEffect`

The CDL pipeline would be refactored to operate within this `CdlEffect` monad. Each stage would consume a `CdlEffect<CDL<CurrentState>>` and produce a `CdlEffect<CDL<NextState>>`.

1.  **Initialization**:
    The pipeline begins by lifting the initial `CDL<NoData>` state into a `CdlEffect` monad using the `pure` function provided by `CdlEffectImpl`:
    ```rust
    let initial_effect: CdlEffect<CDL<NoData>> = CdlEffectImpl::pure(CDL::with_config(cdl_config));
    ```

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
    match final_effect.error {
        Some(fatal_error) => {
            println!("--- Pipeline Failed with Fatal Error ---");
            println!("Error: {:?}", fatal_error);
        },
        None => {
            if let Some(result) = final_effect.value {
                println!("Causal Discovery Result: {:?}", result);
            } else {
                println!("Pipeline completed but yielded no value (unexpected successful empty result).");
            }
        }
    }
    // Always report final_effect.warnings, regardless of success or failure.
    if !final_effect.warnings.is_empty() {
        println!("--- Warnings Encountered ---");
        println!("{}", final_effect.warnings); // CdlWarningLog should implement Display
    }
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
*   **Parallel Processing Integration**: Monadic and traversable structures are well-suited for integration with asynchronous and parallel execution strategies, further leveraging Rust's concurrency features.
*   **Advanced Reporting**: A dedicated `CdlReport` type could be created from the final `CdlEffect` to provide structured access to all accumulated information.

By adopting an HKT-based, monadic approach with the added power of `Traversable` and `CoMonad`, the CDL can become a more powerful, flexible, and transparent framework for causal discovery.