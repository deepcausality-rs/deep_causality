# generic-precision-algorithms Specification

## Purpose
TBD - created by archiving change real-field-discovery. Update Purpose after archive.
## Requirements
### Requirement: SURD is generic over RealField

The SURD algorithm SHALL be generic over a precision type `T: RealField`, returning `SurdResult<T>`. Instantiated at `T = f64`, it SHALL produce results identical to the previous `f64` implementation.

#### Scenario: f64 results are preserved
- **WHEN** SURD runs at `T = f64` on a recorded input
- **THEN** the synergistic, unique, and redundant components match the recorded `f64` output within numerical tolerance

#### Scenario: Higher precision compiles and runs
- **WHEN** SURD is instantiated at `T = Float106` on the same input
- **THEN** it compiles and produces a `SurdResult<Float106>`

#### Scenario: No f64 leaks into the generic path
- **WHEN** the SURD source is compiled
- **THEN** the algorithm uses only `RealField` operations and constructed constants, with no `f64`-typed intermediate in the generic code path

### Requirement: MRMR is generic over RealField

The MRMR feature selector SHALL be generic over `T: RealField` (with `FromPrimitive` where sample-count conversion is required), returning a generic result. Instantiated at `T = f64`, it SHALL produce results identical to the previous `f64` implementation.

#### Scenario: f64 selection is preserved
- **WHEN** MRMR runs at `T = f64` on a recorded input
- **THEN** the selected feature ranking matches the recorded `f64` output

#### Scenario: Lower precision compiles and runs
- **WHEN** MRMR is instantiated at `T = f32` on the same input
- **THEN** it compiles and returns a ranking

### Requirement: No precision regression at f64

The generification SHALL NOT change any algorithm output at `T = f64`. The default precision SHALL remain `f64` so existing callers compile unchanged.

#### Scenario: Existing caller compiles without naming the precision
- **WHEN** a caller uses the algorithms without specifying a precision type
- **THEN** the precision defaults to `f64` and the call compiles

#### Scenario: Golden output is stable
- **WHEN** the full algorithm suite runs at `f64` before and after the change
- **THEN** all recorded outputs are identical within tolerance

