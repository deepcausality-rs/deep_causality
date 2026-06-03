## MODIFIED Requirements

### Requirement: BRCD root-cause ranking from two datasets and a CPDAG

The BRCD entry point `brcd_run` SHALL take the CPDAG as an **optional** argument (`Option<&MixedGraph<N>>`): a normal dataset, an anomalous dataset (both `CausalTensor<T>`, `T: RealField`, aligned columns), an optional CPDAG, and a `BrcdConfig`. When `Some(cpdag)` is supplied it SHALL be used directly. When `None` is supplied, `brcd_run` SHALL first learn a CPDAG from the observational (normal) data via BOSS (capability `brcd-bootstrap`) as a preprocessing step, then proceed with the identical ranking. **BREAKING:** the `cpdag` parameter changes from `&MixedGraph<N>` to `Option<&MixedGraph<N>>`; existing call sites pass `Some(&cpdag)`.

#### Scenario: Supplied CPDAG is used directly

- **WHEN** `brcd_run` is called with `Some(cpdag)` and one true injected root cause
- **THEN** it returns a normalized posterior over the candidate variables and the true root cause is ranked first, without invoking structure learning

#### Scenario: Absent CPDAG triggers structure learning

- **WHEN** `brcd_run` is called with `None` for the CPDAG
- **THEN** it learns a CPDAG from the observational data via BOSS and returns a ranking, rather than returning an error

#### Scenario: Misaligned datasets are rejected

- **WHEN** the normal and anomalous tensors do not share the same columns
- **THEN** BRCD returns an error rather than producing a ranking
