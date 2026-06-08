## ADDED Requirements

### Requirement: Per-algorithm pipeline entry points

The CDL pipeline SHALL provide two distinct entry methods on `CDL<NoData>`, one
per discovery algorithm: `load_surd_input` for SURD and `load_brcd_input` for
BRCD. Each entry SHALL produce an algorithm-specific typestate that begins that
algorithm's lineage.

#### Scenario: SURD entry produces a SURD state

- **WHEN** a caller invokes `load_surd_input::<T>(path, target_index, exclude_indices)` on `CDL<NoData>`
- **THEN** the pipeline transitions to `CDL<SurdData<T>>` with the loaded dataset and recorded record count

#### Scenario: BRCD entry produces a BRCD state

- **WHEN** a caller invokes `load_brcd_input(input)` on `CDL<NoData>` with a `BrcdInput<T>` bundle
- **THEN** the pipeline transitions to `CDL<BrcdLoaded<T>>` carrying the two datasets, the optional CPDAG, and the `BrcdConfig<T>`

### Requirement: Compile-time isolation of the two lineages

The SURD and BRCD lineages SHALL share no typestate before the converged
analysis state. Each algorithm's `*_discover` and `*_analyze` methods SHALL be
implemented only on that algorithm's states, so that applying a method from one
lineage to a state of the other fails to compile.

#### Scenario: BRCD discover is unavailable on a SURD state

- **WHEN** code attempts to call `brcd_discover` on any SURD-lineage state (`SurdData`, `SurdCleaned`, `SurdFeatures`, or `SurdResults`)
- **THEN** compilation fails with a no-such-method error

#### Scenario: SURD analyze is unavailable on a BRCD state

- **WHEN** code attempts to call `surd_analyze` on a `BrcdResults<T>` state
- **THEN** compilation fails with a no-such-method error

#### Scenario: Feature selection is unavailable to BRCD

- **WHEN** code attempts to call `feature_select` or `clean_data` on a BRCD-lineage state
- **THEN** compilation fails with a no-such-method error

### Requirement: SURD lineage stages

The SURD lineage SHALL run `load_surd_input → clean_data → feature_select →
surd_discover → surd_analyze → finalize`, preserving the existing cleaning, MRMR
feature selection, and `surd_states_cdl` discovery behavior. The intermediate
SURD states SHALL be named `SurdData<T>`, `SurdCleaned<T>`, `SurdFeatures<T>`,
and `SurdResults<T>`. `preprocess` and `filter_cohort` SHALL remain available on
`SurdData<T>`.

#### Scenario: SURD chain runs end to end

- **WHEN** a caller runs `load_surd_input` then `clean_data`, `feature_select`, `surd_discover`, `surd_analyze`, and `finalize`
- **THEN** a `CdlReport<T>` is produced containing the SURD decomposition and the MRMR feature-selection result

### Requirement: BRCD lineage stages

The BRCD lineage SHALL run `load_brcd_input → brcd_discover → brcd_analyze →
finalize`, with no cleaning or feature-selection stage. `brcd_discover` SHALL
invoke `brcd_run` with the bundle's `normal`, `anomalous`, `cpdag`, and
`brcd_config`, and SHALL surface a BRCD failure as a pipeline error.

#### Scenario: BRCD chain runs end to end with a supplied CPDAG

- **WHEN** a caller runs `load_brcd_input` with a bundle whose `cpdag` is `Some(graph)`, then `brcd_discover`, `brcd_analyze`, and `finalize`
- **THEN** a `CdlReport<T>` is produced containing the BRCD ranked candidate root-cause sets

#### Scenario: BRCD chain runs with BOSS-learned CPDAG

- **WHEN** a caller runs the BRCD chain with a bundle whose `cpdag` is `None`
- **THEN** `brcd_discover` passes `None` to `brcd_run`, which learns the CPDAG via BOSS, and the chain produces a ranked report

### Requirement: Closed discovery-outcome carrier

The pipeline SHALL carry the discovery result through a closed enum
`DiscoveryOutcome<T>` with variants `Surd(SurdResult<T>)` and
`Brcd(BrcdResult<T>)`, using no dynamic dispatch. Each `*_analyze` method SHALL
wrap its concrete result into the matching variant when transitioning to the
shared `WithAnalysis<T>` state.

#### Scenario: SURD result is carried as the Surd variant

- **WHEN** `surd_analyze` completes
- **THEN** the resulting `WithAnalysis<T>` carries `DiscoveryOutcome::Surd(..)` and `Some(MrmrResult)`

#### Scenario: BRCD result is carried as the Brcd variant

- **WHEN** `brcd_analyze` completes
- **THEN** the resulting `WithAnalysis<T>` carries `DiscoveryOutcome::Brcd(..)` and `None` for feature selection

### Requirement: Per-algorithm analyzers with typed configs

Each algorithm SHALL have its own analyzer and its own analyze configuration.
`surd_analyze` SHALL use the SURD analyzer with a `SurdAnalyzeConfig`;
`brcd_analyze` SHALL use a BRCD analyzer with a `BrcdAnalyzeConfig`. Both SHALL
produce a `ProcessAnalysis` consumed unchanged by the existing formatter. When an
algorithm's analyze config is absent from `CdlConfig`, the method SHALL apply a
sensible default.

#### Scenario: SURD analyze uses SURD thresholds

- **WHEN** `surd_analyze` runs with a `SurdAnalyzeConfig` of synergy/unique/redundancy thresholds
- **THEN** the produced `ProcessAnalysis` reflects those thresholds

#### Scenario: BRCD analyze reports ranked candidates

- **WHEN** `brcd_analyze` runs with a `BrcdAnalyzeConfig` specifying a top-k
- **THEN** the produced `ProcessAnalysis` lists the top-k ranked candidate root-cause sets with their posterior weights

#### Scenario: Default analyze config when none supplied

- **WHEN** `surd_analyze` or `brcd_analyze` runs and no matching analyze config is present in `CdlConfig`
- **THEN** the method applies its default configuration rather than failing

### Requirement: Shared finalize and generalized report

`finalize` SHALL be implemented once on `WithAnalysis<T>` and SHALL produce a
`CdlReport<T>` that carries `DiscoveryOutcome<T>` and `Option<MrmrResult>`. The
report's `Display` SHALL render the correct algorithm section by matching the
`DiscoveryOutcome` variant.

#### Scenario: Report renders the SURD section

- **WHEN** a SURD pipeline reaches `finalize`
- **THEN** the report displays the dataset summary, the MRMR feature selection, and the SURD decomposition

#### Scenario: Report renders the BRCD section

- **WHEN** a BRCD pipeline reaches `finalize`
- **THEN** the report displays the dataset summary and the BRCD root-cause ranking, with no feature-selection section

### Requirement: SURD behavior is preserved

Re-designing the pipeline SHALL NOT change SURD's numeric output or rendered
report on the same input. The rankings, the SURD decomposition, and the report
text SHALL be identical before and after the change.

#### Scenario: SURD regression holds

- **WHEN** the SURD chain runs on a fixed reference dataset after the re-design
- **THEN** the rankings, decomposition, and rendered report match the pre-change output exactly

### Requirement: Generalized discovery error

`CausalDiscoveryError` SHALL gain a `Brcd(BrcdError)` variant so a BRCD failure
propagates through the pipeline as a `CdlError`, alongside the existing tensor
error variant.

#### Scenario: BRCD failure surfaces as a pipeline error

- **WHEN** `brcd_run` returns a `BrcdError` (for example, a dimension mismatch)
- **THEN** `brcd_discover` yields a `CdlError` wrapping `CausalDiscoveryError::Brcd(..)`
