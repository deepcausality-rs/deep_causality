# statistics-consumer-migration Specification

## Purpose
TBD - created by archiving change unified-math-next. Update Purpose after archive.
## Requirements
### Requirement: Migration follows the completed statistics crate gate

Consumer migration SHALL begin only after task 4.15 is complete. The recorded coverage exceptions in 4.16 SHALL remain explicit; the gate SHALL NOT be described as measured full coverage.

#### Scenario: The prerequisite is satisfied
- **WHEN** a consumer migration begins
- **THEN** the stats suite has passed under Cargo and Bazel, with the coverage exceptions recorded
- **AND** affected consumer tests establish a baseline before their implementation changes

### Requirement: Cargo declares dependencies and rules-rs supplies them to Bazel

Each migrated consumer SHALL declare `deep_causality_stats` through the workspace Cargo dependency and select its required features explicitly. The workspace declaration disables default features.

Existing Bazel targets obtain Cargo dependencies through `all_crate_deps(...)` from `@crates//:defs.bzl`. Library targets use normal dependencies; example binaries also select normal development dependencies. Migration SHALL use this rules-rs mechanism rather than add duplicate explicit stats labels to targets. D7 prohibits a stats dependency in tensor.

#### Scenario: A consumer gains its first stats call
- **WHEN** the dependency is added to the consumer's Cargo manifest and the repository's dependency resolution is refreshed as required
- **THEN** the existing target macro supplies that dependency to Bazel
- **AND** Cargo and Bazel both resolve the migrated call

#### Scenario: Tensor remains independent
- **WHEN** dependency declarations are checked
- **THEN** tensor has no dependency on stats

### Requirement: Tests define migration behavior before implementation

Each library migration SHALL establish tests for its numerical contract, boundary cases and error mapping before replacing arithmetic. New shared APIs SHALL first have a declared surface with unimplemented bodies, followed by failing tests and a test review before implementation.

Expected numerical values SHALL come from closed forms, published results or independent calculations. Where no published result is used, calculated reference fixtures SHALL retain a reproducible Python derivation using appropriate precision. The migrated implementation SHALL NOT generate its own expected values.

#### Scenario: A numerical result changes
- **WHEN** a migration changes an output
- **THEN** the comparison records the old value, new value, independent reference and applicable tolerance
- **AND** the result is classified as rounding drift or an intentional contract change
- **AND** existing expected values are not replaced merely to make the migration pass

#### Scenario: Policy branches are tested
- **WHEN** a library wrapper introduces error mapping, a threshold or a fallback
- **THEN** tests exercise each applicable branch, including values on both sides of thresholds and at equality
- **AND** targeted mutation testing checks meaningful arithmetic and policy branches after implementation

### Requirement: SURD shares entropy arithmetic while preserving tensor and presence policies

Both SURD paths SHALL delegate entropy arithmetic to stats in bits. Marginalization remains in algorithms. Conditional entropy may retain its existing subtraction of two delegated entropies; it need not repeat marginalization to call the slice-shaped stats `conditional_entropy`.

The plain path uses no additional normalization and skips zero probabilities. The optional path filters absent values, preserves its all-absent zero result, preserves the strict `sum.abs() < epsilon` mass check for valid probabilities, and includes only normalized probabilities strictly greater than epsilon. Stats' `BySum` check uses `sum <= floor`; passing epsilon as its floor without adaptation does not reproduce the caller's equality boundary.

The four helper signatures SHALL carry the stats-required `FromPrimitive` bound. Algorithms SHALL map `StatsError` locally to `CausalTensorError`; it cannot implement a foreign `From` trait for two foreign error types. Tensor SHALL NOT gain a dependency to host a conversion.

#### Scenario: Valid optional inputs preserve their policies
- **WHEN** optional marginals contain absent entries, zero mass, or positive mass below, equal to and above epsilon
- **THEN** presence filtering and the existing strict mass threshold are preserved
- **AND** normalized entries below, equal to and above epsilon follow the existing strict inclusion threshold

#### Scenario: Invalid probabilities receive a mapped error
- **WHEN** a marginal contains a negative or non-finite probability
- **THEN** it produces a mapped error rather than silently dropping the invalid entry
- **AND** this stricter validation is recorded separately from rounding drift

#### Scenario: The existing corpus remains numerically consistent
- **WHEN** SURD's existing corpus is evaluated after migration
- **THEN** results agree within its established numerical tolerances
- **AND** differences from per-term log2 versus a final division by ln(2) are recorded without requiring bit identity

#### Scenario: Public and helper empty cases are distinguished
- **WHEN** empty-input tests are reviewed
- **THEN** public driver rejection is distinguished from the optional helper's all-absent zero result
- **AND** the latter remains zero without passing an empty slice to stats

### Requirement: BRCD reductions preserve model-specific policies

BRCD SHALL delegate its three log-sum-exp computations, two Gaussian log-density computations, slice mean and corrected variance to stats. Tensor's corresponding computations are excluded by D7.

Density calls SHALL pass variance, not standard deviation. BRCD's variance floor and fitted residual-variance floor of 1e-12 SHALL remain caller policies. Its empty mean of zero and variance of one for fewer than two observations SHALL remain explicit wrapper policies. Invalid-input error mappings SHALL be tested rather than replaced with unconditional unwraps.

#### Scenario: Degenerate regimes preserve the reference contract
- **WHEN** a BRCD regime has zero or one observation at an existing sentinel-bearing call site
- **THEN** its documented zero-mean and unit-variance fallbacks remain
- **AND** the existing single-row unit-variance regression test continues to pass

#### Scenario: Gaussian parameterization is preserved
- **WHEN** a BRCD density is evaluated
- **THEN** the model's variance-floor policy is applied before calling stats
- **AND** a positive non-unit variance fixture detects accidental variance-to-standard-deviation conversion

#### Scenario: Log-sum-exp edge cases remain covered
- **WHEN** the three BRCD replacements run on ordinary, empty where applicable, and non-finite inputs
- **THEN** tests establish compatibility with the shipped stats functions for each case

### Requirement: BRCD fitting migration preserves objectives and resource behavior

Materialized ridge SHALL delegate to stats and retain a local prediction adapter where required. The shared solver is `deep_causality_linear::solve`, LU with partial pivoting. Model variance floors remain outside the shared fit.

The shipped stats streaming entry collects its iterator into a vector. An adapter from BRCD's shared columns and row indices alone therefore does not preserve BRCD's streaming memory behavior. Streaming migration SHALL first provide a shared path that avoids retaining the full design, with the API and tests specified before implementation; until then the existing consumer path remains.

BRCD logistic leaves the intercept unpenalized, whereas the shipped stats fit penalizes every coefficient. Its boolean labels, stopping rule and last-iterate return at the iteration cap also require explicit adaptation. Migration SHALL NOT substitute a different objective or silently change convergence behavior. Any shared API extension SHALL preserve the existing stats entry's contract and receive tests before implementation. Sigmoid can migrate independently.

#### Scenario: Ridge retains numerical fidelity
- **WHEN** materialized and streaming ridge are migrated
- **THEN** independent non-diagonal pivoting fixtures verify coefficients, predictions and residual variance
- **AND** BRCD reference-parity tests pass within their established tolerances
- **AND** the streaming path does not retain a full materialized design

#### Scenario: Logistic compatibility is a prerequisite
- **WHEN** the BRCD logistic fit is delegated
- **THEN** tests already pin its unpenalized intercept, label conversion, stopping criterion, iteration cap and singular-system behavior
- **AND** the shared implementation reproduces those policies through an explicitly specified API

#### Scenario: Remaining local solve users determine cleanup
- **WHEN** a fit is migrated
- **THEN** the remaining users of brcd_linalg are checked before removing its implementation
- **AND** any retained module documentation accurately describes LU with partial pivoting

### Requirement: mRMR delegates pairwise-complete Pearson in the working scalar

mRMR SHALL use `pearson_pairwise_complete` with the required `RealField + FromPrimitive` bounds. `Float` supplies arithmetic but does not imply the algebraic law bounds of `RealField`.

Pairwise deletion and the surviving-pair count SHALL remain. Zero variance SHALL continue to produce zero correlation. The perfect-correlation F-statistic sentinel is a separate policy and SHALL remain tested. Existing public f64 result boundaries SHALL remain unless separately redesigned; the precision claim applies to arithmetic before that conversion.

#### Scenario: Pearson delegates in the working scalar
- **WHEN** mRMR evaluates two columns
- **THEN** it uses stats' pairwise-complete Pearson in the caller's scalar
- **AND** any conversion to the existing public f64 result occurs after that computation

### Requirement: Algorithm consumers use algebraic numeric bounds

The mRMR selector, Pearson helper and F-statistic helper SHALL replace their legacy Float bounds with RealField + FromPrimitive. Discovery's MrmrFeatureSelector and both SurdCleaned CDL implementations SHALL remove the Float bounds inherited from mRMR; existing Precision bounds already supply the numeric requirements where present. Other algorithms SHALL be checked for explicit or indirect Float constraints, selecting Real for analytic operations without field division and RealField for real-field computations.

Float remains a low-level num capability. The existing FloatOption trait itself requires Float and SHALL be removed from mRMR's bounds without introducing a replacement trait. The implementation SHALL use ordinary Option values and standard Copy + Into<Option<F>> bounds to accept both scalar and optional inputs. It SHALL preserve None and filter NaN/Some(NaN) locally while passing infinities to the statistics validation. Neither a RealOption trait nor a num dependency on algebra SHALL be introduced.

#### Scenario: Optional input does not reintroduce Float
- **WHEN** mRMR accepts plain or optional real scalars
- **THEN** its numeric and standard option-conversion bounds do not require Float
- **AND** tests cover finite values, None, NaN, Some(NaN) and both infinities

#### Scenario: The numeric abstraction is compile-checked
- **WHEN** a generic caller is declared using RealField + FromPrimitive and the necessary nonnumeric container bounds only
- **THEN** it can call mRMR without adding Float
- **AND** discovery's selector and cleaned CDL entry points compile with their corresponding algebraic bounds

#### Scenario: The algorithm sweep is complete
- **WHEN** runtime sources in algorithms and the affected discovery call chain are checked
- **THEN** no Float trait import or bound remains in that scope
- **AND** enum variants such as parquet Field::Float are not mistaken for numeric trait usage
- **AND** stale bound documentation is corrected against the implemented interfaces

#### Scenario: FloatOption has no remaining consumers
- **WHEN** a post-migration workspace search finds no FloatOption consumers beyond its own definition, implementations, re-export and dedicated tests
- **THEN** FloatOption and its dedicated support code are flagged for removal with their file locations and public API impact
- **AND** speculative compatibility alone is not a reason to retain it
- **AND** removal is reported for follow-up rather than performed by the usage check

#### Scenario: Missing and degenerate columns retain their policies
- **WHEN** columns contain missing pairs, too few valid pairs, constant values or perfect correlation
- **THEN** each case exercises its corresponding existing policy or mapped stats error
- **AND** constant columns are not confused with perfect correlation

#### Scenario: Precision improves before the public conversion
- **WHEN** Pearson is computed at Float106 on a fixture with an independently known result
- **THEN** intermediate arithmetic retains Float106 accuracy
- **AND** the test separately identifies the final public f64 conversion

#### Scenario: Changed numerical behavior is explicit
- **WHEN** centered accumulation differs from the former raw-sums calculation or non-finite input is rejected
- **THEN** tests and migration notes identify the difference and its reference result

### Requirement: Discovery adopts the shared binning contract explicitly

Discovery SHALL delegate equal-width and equal-frequency binning and migrate compatible mean imputation. Error conversion SHALL preserve useful preprocessing context.

Shared binning rejects empty data, fewer than two bins, more bins than observations and non-finite entries. Existing discovery helpers already reject fewer than two bins and NaN; newly rejected cases include empty input, excess bins and infinities.

Equal-frequency binning SHALL adopt stats' tied-block largest-share assignment, with ties between shares assigned to the lower bin. Its rank boundaries differ from discovery's rounded partition boundaries. Equal-width migration SHALL test rounding at boundaries and the change from an epsilon-based constant-range check to an exact check; value identity is not assumed.

#### Scenario: Binning policy changes are visible
- **WHEN** fixtures contain repeated values, an observation count not divisible by the bin count, near-constant ranges or values on bin edges
- **THEN** expected assignments follow the shared contract
- **AND** old and new assignments are recorded for changed fixtures

#### Scenario: Invalid binning input maps to preprocessing errors
- **WHEN** input is empty, contains NaN or infinity, or has an invalid bin count
- **THEN** the appropriate preprocessing error is returned and tested

#### Scenario: Mean imputation retains its missing-data policy
- **WHEN** mean imputation is migrated
- **THEN** extraction of valid observations and the all-missing-column policy are tested before delegation

### Requirement: Physics entropy exposes bits in both public names

The physics entropy kernel and causal wrapper SHALL be renamed to state bits and SHALL delegate to stats with `EntropyConfig::bits()`. Their bounds SHALL include `FromPrimitive`. Physics SHALL provide an appropriate `StatsError` conversion to its own error type.

#### Scenario: The published unit is bits
- **WHEN** a uniform distribution over four outcomes is evaluated through either public entry
- **THEN** entropy is two bits
- **AND** both names, callers and documentation identify bits
- **AND** the public rename and unit change are recorded as breaking

#### Scenario: Invalid probabilities are reported
- **WHEN** input contains negative or non-finite probabilities
- **THEN** physics returns the mapped error
- **AND** changed validation behavior is documented and tested

### Requirement: The inventory distinguishes compatible reductions from retained computations

Every inventory entry SHALL identify its actual source, owning package, computation, migration disposition and verification. Totals SHALL be derived from entries using a stated counting unit, not from stale section headings.

D7 retains all tensor statistics. Quantum's shot bridge retains its frequency-weighted reduction over usize outcomes and u64 counts without expanding one sample per shot. Candle tensor operations and lazy Uncertain expression reductions remain in their respective execution models. Population variance SHALL NOT be replaced by corrected variance.

Uncertain's plain-slice `from_samples`, its other statistics helpers and discovery mean imputation SHALL receive explicit dispositions based on their actual implementations. A plain-slice helper cannot inherit the weighted bridge's exclusion merely because both return Uncertain. Any proposed uncertainty dependency change SHALL include its derived tier effects and its empty, singleton and non-finite policies before implementation.

#### Scenario: Completeness is source-based
- **WHEN** migration completeness is checked
- **THEN** every inventory entry is migrated or explicitly retained with its mathematical or architectural reason
- **AND** no superseded local arithmetic remains at migrated sites
- **AND** retained implementations are not counted as unresolved duplicates

### Requirement: Compatible examples migrate with the library consumers

Compatible example reductions SHALL delegate to stats, including both separate math_utils files, DDoS, weather, Granger, CATE, ML score and chronometric sites. The population-variance standardizer SHALL have an explicit retained disposition unless a separately specified population API is added.

Each owning Cargo package SHALL gain the dependency when needed. Existing scalar aliases and shared-helper types SHALL be respected. Example code SHALL use existing lift utilities rather than introduce conversion helpers.

#### Scenario: Example edge conventions are explicit
- **WHEN** a helper previously returned NaN for empty input or zero standard deviation for one observation
- **THEN** its migration explicitly preserves that wrapper policy or records an approved interface change
- **AND** it does not silently replace sample variance with population variance or vice versa

#### Scenario: Examples are verified by execution
- **WHEN** an example migration is complete
- **THEN** the affected example runs under its supported build workflow
- **AND** no unit-test modules are added to example binaries

### Requirement: Group completion verifies consumers and dependency documentation

Group completion SHALL require affected Cargo tests, workspace Bazel tests, formatting and lint checks, unused-dependency checks for changed consumers, affected example execution and reconciliation of dependency documentation.

#### Scenario: The group closes
- **WHEN** group 5 is marked complete
- **THEN** `bazel test //...` passes and changed consumers have no unused stats dependency
- **AND** the inventory, task list and spec agree on completed and retained sites
- **AND** changed numerical results and any verification limitations are recorded
- **AND** dependency tiers and documentation agree with the manifests

