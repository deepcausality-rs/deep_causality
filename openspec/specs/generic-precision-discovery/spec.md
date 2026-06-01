# generic-precision-discovery Specification

## Purpose
TBD - created by archiving change real-field-discovery. Update Purpose after archive.
## Requirements
### Requirement: CDL pipeline is generic over precision

Every stage of the CDL pipeline (load, clean, preprocess, feature-select, discover, analyze, finalize) and the typestate that threads them SHALL be generic over a precision type `T: RealField`. The precision SHALL be selectable at the call site by a single type parameter, as in the wider numerical stack.

#### Scenario: A pipeline at f64 reproduces current behavior
- **WHEN** the CDL pipeline runs end to end at `T = f64` on a recorded input
- **THEN** the produced rankings, decomposition, and rendered report are identical to before the change

#### Scenario: The same pipeline at higher precision compiles and runs
- **WHEN** the identical pipeline is parameterized at `T = Float106`
- **THEN** it compiles and runs, changing only the precision type

#### Scenario: Switching precision is a one-line change
- **WHEN** a user changes the precision alias used to build the pipeline
- **THEN** no other pipeline code requires modification

### Requirement: Stage traits and configs carry the precision parameter

The stage traits (`DataLoader`, `DataCleaner`, `DataPreprocessor`, `FeatureSelector`, `CausalDiscovery`, `ProcessResult`), the configuration types, and the typestate states SHALL be generic over `T: RealField`, with no `dyn` trait objects introduced.

#### Scenario: Implementing a stage at a chosen precision compiles
- **WHEN** a stage is implemented for a concrete `T`
- **THEN** it compiles and integrates into the typestate at that precision

#### Scenario: Discovery returns a precision-parameterized result
- **WHEN** the SURD discovery stage runs at precision `T`
- **THEN** it returns `SurdResult<T>`

### Requirement: Data loaders produce the chosen precision

The CSV and Parquet loaders SHALL produce `CausalTensor<Option<T>>`, converting parsed source numbers into `T`.

#### Scenario: One file loads at two precisions
- **WHEN** the same numeric CSV is loaded at `T = f64` and at `T = Float106`
- **THEN** both load successfully, each value converted into the requested precision

#### Scenario: Missing values remain optional
- **WHEN** a source cell is empty or non-numeric and cleaning maps it to `None`
- **THEN** the loaded tensor carries `None` at that position regardless of `T`

### Requirement: f64 pipeline behavior is preserved

The generification SHALL preserve CDL results at `T = f64`, and `f64` SHALL remain the default precision so existing pipelines compile unchanged.

#### Scenario: End-to-end f64 output is identical
- **WHEN** the SURD-based CDL pipeline runs at `f64` before and after the change on the same input
- **THEN** the final report is identical

#### Scenario: Existing pipeline compiles without naming the precision
- **WHEN** an existing pipeline is built without specifying a precision type
- **THEN** it defaults to `f64` and compiles unchanged

