# Feature Specification: Causal Discovery DSL

**Feature Branch**: `003-causal-disovery-dsl`  
**Created**: Friday, September 19, 2025  
**Status**: Draft  
**Input**: User description: "Causal-disovery-dsl Currently, the DeepCausality project does have some algorithms for causal discovery, but lacks a process that simplifies causal discovery. The idea is a build a new crate called: deep_causality_discovery That depends on the following internal crates: deep_causality_algorithms deep_causality_tensor External dependencies may entail: csv - to read csv files parquet - to read parquet files The deep_causality_discovery provides a Domain Specific Language (DSL) for causal discovery that uses monadic composition to streamline the composition of steps in the discovery process. The high level workflow causal discovery is: 1) Load date from file (CSV or parquet) 2) Run feature selection (using mrmr) 3) Run causal discover (using SURD) 4) Interpret results 5) Format results The main type is called CQD (For Causal Qualities Discovery) and its used to attach multiple steps to be executed in sequence. For example, in pseudo Rust:
```rust
Let path = "data/file.csv"
Let discovery_process = CQD::new()
.start(read_csv(path))
.feat_select(mrmr())
.causal_discovery(surd())
.analyze(analyse_surd_results())
.finalize(format(console_print()))
.build()?;
Let result = discovery_process.run()?;
println(&results);
```
The start step takes a function of type (trait) ProcessDataLoader that loads data and returns a CausalTensor. This would be great to be implemented using the SansIO design pattern. Then feat_select() applies the feature selector that implements trait FeatureSelector passed as parameter to the tensor, and returns the processed tensor to the next step. Likewise, causal_discovery takes a CcausalTensor as input, applies a discovery algorithm that implements the CausalDiscovery trait to the input tensor, and returns The analyze step takes a function of type (trait) ProcessResultAnalyzer, which takes a CausalTensor as as input, analysis the tensor, recommends how to translate the results into a causal structure and retur... [truncated]

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

### For AI Generation
When creating this spec from a user prompt:
1. **Mark all ambiguities**: Use [NEEDS CLARIFICATION: specific question] for any assumption you'd need to make
2. **Don't guess**: If the prompt doesn't specify something (e.g., "login system" without auth method), mark it
3. **Think like a tester**: Every vague requirement should fail the "testable and unambiguous" checklist item
4. **Common underspecified areas**:
   - User types and permissions
   - Data retention/deletion policies  
   - Performance targets and scale
   - Error handling behaviors
   - Integration requirements
   - Security/compliance needs

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
A user wants to perform causal discovery on their data, from loading to formatting results, using a streamlined, composable DSL.

### Acceptance Scenarios
1. **Given** a CSV file with data, **When** the user defines a CQD process to load, feature select, causal discover, analyze, and format, **Then** the process runs successfully and prints formatted causal insights.
2. **Given** a Parquet file with data, **When** the user defines a CQD process to load, feature select, causal discover, analyze, and format, **Then** the process runs successfully and prints formatted causal insights.
3. **Given** a CQD process with a `feat_select` step using mRMR, **When** the process is executed, **Then** mRMR is applied to the data before causal discovery.
4. **Given** a CQD process with a `causal_discovery` step using SURD, **When** the process is executed, **Then** SURD is applied to the data.
5. **Given** SURD results, **When** the `analyze` step is executed, **Then** recommendations for converting SURD results to Causaloids are generated.
6. **Given** analyzed results, **When** the `finalize` step is executed with a console printer, **Then** the results are printed to the console in a human-readable format.

### Edge Cases
- **What happens when the input file (CSV/Parquet) does not exist or is malformed?** The system will return a `CqdError::ReadDataError` containing a `DataError` variant that specifies the exact nature of the file I/O or parsing issue (e.g., `FileNotFound`, `MalformedCsv`, `InvalidParquetSchema`).
- **How does the system handle cases where feature selection or causal discovery algorithms fail (e.g., insufficient data, numerical instability)?** The system will return a `CqdError::FeatSelectError` or `CqdError::CausalDiscoveryError` respectively, encapsulating specific error variants from `FeatureSelectError` or `CausalDiscoveryError` (e.g., `InsufficientData`, `NumericalInstability`).
- **What happens if the order of steps in the DSL is semantically incorrect (e.g., `feat_select` before `start`)?** The system will prevent invalid step sequences at compile time using the Typestate Pattern.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The system MUST provide a `deep_causality_discovery` crate.
- **FR-002**: The `deep_causality_discovery` crate MUST depend on `deep_causality_algorithms` and `deep_causality_tensor`.
- **FR-003**: The `deep_causality_discovery` crate MUST support reading data from CSV files and returning a `CausalTensor`.
- **FR-004**: The `deep_causality_discovery` crate MUST support reading data from Parquet files and returning a `CausalTensor`.
- **FR-005**: The system MUST provide a `CQD` type for composing causal discovery steps.
- **FR-006**: The `CQD` type MUST support a `start` step that takes a `ProcessDataLoader` trait object and a data loading configuration (e.g., `CsvConfig` or `ParquetConfig`).
- **FR-007**: The `CQD` type MUST support a `feat_select` step that takes a `FeatureSelector` trait object and an `MrmrConfig` for configuring the feature selection algorithm.
- **FR-008**: The `CQD` type MUST support a `causal_discovery` step that takes a `CausalDiscovery` trait object and a `SurdConfig` for configuring the causal discovery algorithm.
- **FR-009**: The `CQD` type MUST support an `analyze` step that takes a `ProcessResultAnalyzer` trait object.
- **FR-010**: The `CQD` type MUST support a `finalize` step that takes a `ProcessResultFormatter` trait object.
- **FR-011**: The `analyze` step MUST generate recommendations for converting SURD results to `Causaloid` structures.
- **FR-012**: The `finalize` step MUST support a console printer for formatted results.
- **FR-013**: The system SHOULD optionally provide static verification of the order of steps in the DSL at compile time.
- **FR-014**: The system MUST map aggregate SURD results to `CausaloidGraph` structure (e.g., unique influence to directed edge, synergistic influence to many-to-one connection).
- **FR-015**: The system MUST map state-dependent SURD results to `Causaloid` logic (e.g., conditional effects based on state).
- **FR-016**: The system MUST support modeling multiple causes using `Causaloid Collection` with `Aggregate Logic` (All, Any, Some(k)) based on SURD synergy, unique, or redundant influences.
- **FR-017**: The system MUST define a comprehensive error handling mechanism using a `CqdError` enum, which encapsulates specific errors from each stage of the causal discovery process (data reading, feature selection, causal discovery, result analysis, and finalization). Each nested error enum MUST provide detailed error cases and meaningful display implementations.
- **FR-018**: The system MUST enforce the semantic order of DSL steps at compile time using the Typestate Pattern, where each step returns a new type representing the valid subsequent states of the causal discovery process.

### Key Entities *(include if feature involves data)*
- **CQD (Causal Qualities Discovery)**: The main type for orchestrating causal discovery workflows.
- **ProcessDataLoader**: Trait for loading data (e.g., from CSV, Parquet) into a `CausalTensor`.
- **FeatureSelector**: Trait for applying feature selection algorithms (e.g., mRMR) to a `CausalTensor`.
- **CausalDiscovery**: Trait for applying causal discovery algorithms (e.g., SURD) to a `CausalTensor`.
- **ProcessResultAnalyzer**: Trait for analyzing causal discovery results and recommending `Causaloid` structures. The `analyze` step takes a `SurdResult` as input.
- **ProcessAnalysis**: The output of the `ProcessResultAnalyzer`, containing recommendations.
- **ProcessResultFormatter**: Trait for formatting `ProcessAnalysis` into a presentable result.
- **ProcessFormattedResult**: The final formatted output.
- **CausalTensor**: A multi-dimensional array for numerical data, used throughout the process.
- **CausaloidGraph**: A graph structure representing causal links.
- **Causaloid**: A self-contained unit of causality with internal logic.
- **Causaloid Collection**: A container for multiple `Causaloids` with aggregate logic.
- **MrmrConfig**: Configuration for the mRMR algorithm, including `num_features` (desired number of features to select) and `target_col` (column index of the target variable).
- **SurdConfig**: Configuration for the SURD algorithm, including `max_order` (maximum order of interactions to compute, represented by `MaxOrder` enum).
- **CsvConfig**: Configuration for CSV data loading, including `has_headers` (boolean), `delimiter` (byte), `skip_rows` (usize), and optional `columns` (vector of strings) to select.
- **ParquetConfig**: Configuration for Parquet data loading, including optional `columns` (vector of strings) to select and `batch_size` (usize) for reading.

### Error Types

'''

     pub Enum DataError{
			     FileNotFound(path: String),
			     PermissionDenied(os_error : String),
			     os_error(os_error: String), // catch all
     }
     
     pub Enum FeatureSelectError{
	     TooFewFeaturs(usize), // print the actual vs. min. Number
	     MRMRError(MrmrError), // from the mrmr algo, 
	     TensorError(CausalTensorError), // catch all
     }
     
     pub Enum CausalDiscoveryError{
	     TensorError(CausalTensorError), // from the surd algo 
     }
     
     pub Enum AnalyzeError{
	    EmptyResult, // In case SURD returned an empty result
	    AnalysisFailed(msg: string), // In case a processing error  
      TensorError(CausalTensorError), // catch all
     }
     
     pub Enum FinalizeError{
	     FormattingError(msg: string)
     }
'''

### Typestate Transitions
- **`CQD<NoData>`**: Initial state, no data loaded.
    - Transition: `.start(loader, config)` -> `CQD<WithData>`
- **`CQD<WithData>`**: Data loaded, ready for feature selection.
    - Transition: `.feat_select(selector, config)` -> `CQD<WithFeatures>`
- **`CQD<WithFeatures>`**: Features selected, ready for causal discovery.
    - Transition: `.causal_discovery(discovery, config)` -> `CQD<WithCausalResults>`
- **`CQD<WithCausalResults>`**: Causal discovery performed, ready for analysis.
    - Transition: `.analyze(analyzer)` -> `CQD<WithAnalysis>`
- **`CQD<WithAnalysis>`**: Results analyzed, ready for finalization/formatting.
    - Transition: `.finalize(formatter)` -> `CQD<Finalized>`
- **`CQD<Finalized>`**: Process finalized, ready to build and run.
    - Transition: `.build()` -> `Result<CQDRunner, CqdError>`
- **`CQDRunner`**: Built pipeline, ready to run.
    - Transition: `.run()` -> `Result<ProcessFormattedResult, CqdError>`

### Heuristics for Translating SURD Results to Causaloid Structures

These heuristics guide the `analyze` step in generating recommendations for building `Causaloid` structures based on the output of the SURD algorithm:

-   **Strong SYNERGY**:
    -   **Rule**: If SURD analysis shows a large synergistic value for a set of source variables (e.g., A, B) influencing a target (Reaction).
    -   **Causaloid Model**: Model with a `Causaloid Collection` containing individual `Causaloid`s for each synergistic source (e.g., `Causaloid(A)`, `Causaloid(B)`). Set the `Aggregate Logic` of the collection to `All` (Conjunction), meaning the collection becomes active only if all constituent `Causaloid`s are active.

-   **Strong UNIQUE or REDUNDANT Influences**:
    -   **Rule**: If SURD analysis shows strong unique contributions from multiple sources (e.g., Virus, Bacteria) influencing a target (Fever).
    -   **Causaloid Model**: Model with a `Causaloid Collection` containing individual `Causaloid`s for each unique/redundant source (e.g., `Causaloid(Virus)`, `Causaloid(Bacteria)`). Set the `Aggregate Logic` of the collection to `Any` (Disjunction), meaning the collection becomes active if at least one constituent `Causaloid` is active.

-   **Complex, Mixed Influences (e.g., multiple synergistic pairs)**:
    -   **Rule**: If SURD analysis shows strong synergistic effects for multiple combinations of sources (e.g., (PS1, PS2), (PS1, PS3), (PS2, PS3)) influencing a target (Server Failure).
    -   **Causaloid Model**: Model with a `Causaloid Collection` with an `Aggregate Logic` of `Some(k)` (Threshold). The value of `k` is determined by the specific rule (e.g., `Some(2)` for "any two of three"). The collection becomes active if at least `k` of the constituent `Causaloid`s are active.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] Written for non-technical stakeholders
- [ ] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous  
- [ ] Success criteria are measurable
- [ ] Scope is clearly bounded
- [ ] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [ ] User description parsed
- [ ] Key concepts extracted
- [ ] Ambiguities marked
- [ ] User scenarios defined
- [ ] Requirements generated
- [ ] Entities identified
- [ ] Review checklist passed

---
