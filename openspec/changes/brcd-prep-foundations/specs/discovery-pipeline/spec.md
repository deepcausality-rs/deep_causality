## ADDED Requirements

### Requirement: Two-dataset carriage through the discovery stage

The discovery stage SHALL accept a primary dataset and an optional second aligned dataset. A single-dataset algorithm SHALL read only the primary dataset and SHALL be unaffected by the presence or absence of the second.

#### Scenario: Single-dataset algorithm is unaffected
- **WHEN** SURD runs through the pipeline with only a primary dataset
- **THEN** it produces the same result as before the change

#### Scenario: Two-dataset algorithm receives both datasets
- **WHEN** an algorithm that requires normal and anomalous data runs through the pipeline
- **THEN** both aligned datasets are available to it at the discovery stage

### Requirement: Algorithm-specific discovery result without dynamic dispatch

The pipeline SHALL represent discovery output as a closed enum `DiscoveryOutcome<T>` of algorithm-specific result types, parameterized over the precision `T: RealField` introduced by the prerequisite `real-field-discovery`, with no `dyn` trait objects. The analyzer and the formatter SHALL handle each variant by exhaustive match.

#### Scenario: SURD result flows through as its own variant
- **WHEN** SURD completes
- **THEN** its result is carried as the SURD variant and rendered by the analyzer and formatter exactly as before

#### Scenario: Adding an algorithm is a compile-checked change
- **WHEN** a new result variant is introduced in a later change
- **THEN** the exhaustive matches in the analyzer and formatter fail to compile until the variant is handled

### Requirement: User-supplied domain graph input

The discovery stage SHALL accept an optional user-supplied graph over the variables. Algorithms that do not consume a graph SHALL ignore it; algorithms that require one SHALL read it from the stage input.

#### Scenario: Supplied graph is available to the algorithm
- **WHEN** a domain graph is passed into the discovery stage
- **THEN** the discovery algorithm can read it

#### Scenario: Absent graph is permitted
- **WHEN** no graph is supplied
- **THEN** algorithms that do not need one run unchanged

### Requirement: SURD behavior is preserved

The change SHALL preserve SURD's numerical results and report content. Generalizing the result type and the dataset carriage SHALL NOT alter SURD output for the same input.

#### Scenario: SURD output is identical before and after
- **WHEN** the SURD pipeline runs on the same input before and after this change
- **THEN** the rankings, the SURD decomposition, and the rendered report are identical
