## ADDED Requirements

### Requirement: Per-algorithm pipeline entry points

The CDL pipeline SHALL provide two entry points on `CdlBuilder`, one per discovery
algorithm: `build_surd(&SurdLoaderConfig<T>)` for SURD and
`build_brcd(&BrcdLoaderConfig<T>)` for BRCD. Each SHALL seed an algorithm-specific
configured state carrying its run config, and SHALL fix the pipeline precision `T`
from the config so no downstream turbofish is needed.

#### Scenario: SURD entry produces a configured SURD state

- **WHEN** a caller invokes `CdlBuilder::build_surd(&config)` with a `SurdLoaderConfig<T>`
- **THEN** the pipeline begins in a `CDL<SurdConfigured<T>>` carrying that config

#### Scenario: BRCD entry produces a configured BRCD state

- **WHEN** a caller invokes `CdlBuilder::build_brcd(&config)` with a `BrcdLoaderConfig<T>`
- **THEN** the pipeline begins in a `CDL<BrcdConfigured<T>>` carrying that config

### Requirement: Compile-time isolation of the two lineages

The SURD and BRCD lineages SHALL share no typestate before the converged analysis
state. Each algorithm's stage methods SHALL be implemented only on that algorithm's
states, so applying a method from one lineage to a state of the other fails to
compile.

#### Scenario: BRCD discover is unavailable on a SURD state

- **WHEN** code attempts to call `brcd_discover` on a SURD-lineage state
- **THEN** compilation fails with a no-such-method error

#### Scenario: SURD analyze is unavailable on a BRCD state

- **WHEN** code attempts to call `surd_analyze` on a BRCD-lineage state
- **THEN** compilation fails with a no-such-method error

#### Scenario: Feature selection is unavailable to BRCD

- **WHEN** code attempts to call `feature_select` on a BRCD-lineage state
- **THEN** compilation fails with a no-such-method error

### Requirement: Config-driven, parameterless stages

The pipeline stages SHALL take no algorithm parameters; each SHALL read what it
needs from the carried run config. The SURD lineage SHALL run
`surd_load_input → clean_data → feature_select → surd_discover → surd_analyze →
finalize`, where `feature_select` uses the config's feature count and target index,
`surd_discover` uses the config's max order, and `surd_analyze` uses the config's
thresholds. The BRCD lineage SHALL run `brcd_load_input → brcd_discover →
brcd_analyze → finalize`.

#### Scenario: SURD stages read from config

- **WHEN** a SURD pipeline runs `surd_load_input().clean_data(..).feature_select().surd_discover().surd_analyze()`
- **THEN** the dataset path, MRMR feature count, target index, max order, and thresholds all come from the `SurdLoaderConfig`, with no inline parameters

#### Scenario: BRCD stages read from the bundle

- **WHEN** a BRCD pipeline runs `brcd_load_input().brcd_discover()`
- **THEN** the two datasets, optional CPDAG, and `BrcdConfig` all come from the loaded bundle

### Requirement: Fluent effect surface

Each stage SHALL be invocable directly on the pipeline effect, so a pipeline reads
as a single method chain without a per-line monadic-bind wrapper, while still
short-circuiting on error and threading the warning log.

#### Scenario: Chain without bind wrappers

- **WHEN** a caller writes `CdlBuilder::build_surd(&cfg).surd_load_input().clean_data(..)..`
- **THEN** it compiles and runs without any `.bind(|cdl| cdl. …)` wrapper, short-circuiting if any stage errors

### Requirement: Closed discovery-outcome carrier

The pipeline SHALL carry the discovery result through a closed enum
`CdlDiscoveryOutcome<T>` with variants `Surd(Box<SurdResult<T>>)` and
`Brcd(BrcdResult<T>)`, using no dynamic dispatch. Each `*_analyze` method SHALL
wrap its concrete result into the matching variant when transitioning to the shared
`WithAnalysis<T>` state.

#### Scenario: SURD result is carried as the Surd variant

- **WHEN** `surd_analyze` completes
- **THEN** the resulting state carries `CdlDiscoveryOutcome::Surd(..)` and `Some(MrmrResult)`

#### Scenario: BRCD result is carried as the Brcd variant

- **WHEN** `brcd_analyze` completes
- **THEN** the resulting state carries `CdlDiscoveryOutcome::Brcd(..)` and `None` for feature selection

### Requirement: Per-algorithm analyzers

Each algorithm SHALL have its own analyzer typed to its own result via the
`ProcessResultAnalyzer` contract's associated input/config types, both producing a
common `ProcessAnalysis` rendered by the existing formatter.

#### Scenario: SURD analyze uses SURD thresholds

- **WHEN** `surd_analyze` runs
- **THEN** the produced `ProcessAnalysis` reflects the config's synergy/unique/redundancy thresholds

#### Scenario: BRCD analyze reports ranked candidates

- **WHEN** `brcd_analyze` runs
- **THEN** the produced `ProcessAnalysis` lists the top-ranked candidate root-cause sets with their posterior weights

### Requirement: Shared finalize and generalized report

`finalize` SHALL be implemented once on `WithAnalysis<T>` and SHALL produce a
`CdlReport<T>` carrying `CdlDiscoveryOutcome<T>` and `Option<MrmrResult>`. The
report's `Display` SHALL render the correct algorithm section by matching the
variant, omitting the feature-selection section for BRCD.

#### Scenario: Report renders the SURD section

- **WHEN** a SURD pipeline reaches `finalize`
- **THEN** the report displays the dataset summary, the MRMR feature selection, and the SURD decomposition

#### Scenario: Report renders the BRCD section

- **WHEN** a BRCD pipeline reaches `finalize`
- **THEN** the report displays the dataset summary and the BRCD root-cause ranking, with no feature-selection section

### Requirement: SURD behavior is preserved

Re-designing the pipeline SHALL NOT change SURD's discovery behavior: the full SURD
chain SHALL still load, clean, MRMR-select, run SURD-states, analyze, and produce a
SURD-variant report on the same input.

#### Scenario: SURD regression holds

- **WHEN** the SURD chain runs end to end on a fixed dataset after the re-design
- **THEN** it produces a `CdlReport` whose outcome is the SURD variant, carrying the feature-selection result, and renders the SURD section

### Requirement: Generalized discovery error

`CausalDiscoveryError` SHALL gain a `Brcd(BrcdError)` variant so a BRCD failure
propagates through the pipeline as a `CdlError`.

#### Scenario: BRCD failure surfaces as a pipeline error

- **WHEN** `brcd_run` returns a `BrcdError`
- **THEN** `brcd_discover` yields a `CdlError` wrapping `CausalDiscoveryError::Brcd(..)`
