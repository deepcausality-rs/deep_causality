## ADDED Requirements

### Requirement: No workspace library crate depends on rayon
The root `Cargo.toml` `[workspace.dependencies]` table and the `[dependencies]` table of every workspace library crate SHALL contain no `rayon` entry.
Every `parallel` feature SHALL forward to `deep_causality_par/parallel` instead of `dep:rayon`.
Dev-only and example-only transitive paths (`criterion`, `candle-core`) are outside this requirement.

#### Scenario: Normal dependency graph is rayon-free
- **WHEN** `cargo tree -e normal -i rayon --all-features` runs for each of `deep_causality_topology`, `deep_causality_fft` and `deep_causality_algorithms`
- **THEN** it reports that `rayon` is not in the graph

#### Scenario: No source file imports rayon
- **WHEN** the library `src/` trees of the workspace are searched for `rayon::`
- **THEN** there are no matches

### Requirement: Migrated call sites preserve results
Each migrated parallel call site SHALL return the same values as its serial arm for the same inputs, as it did under rayon.
The algorithms crate's error-returning maps (`surd`, `brcd`, `mrmr`) SHALL return an error when any
element fails. Where several elements fail, the returned error SHALL be the one from the lowest input
index.

#### Scenario: Topology operators agree across feature modes above threshold
- **WHEN** a stencil, bilinear, matvec, wedge, interior-product, de Rham or sharp operator runs on a lattice large enough to cross its parallel threshold
- **THEN** the output equals the serial output bit for bit

#### Scenario: FFT axis passes agree across feature modes
- **WHEN** an N-D FFT whose axis pass crosses `PARALLEL_THRESHOLD` runs forward and inverse
- **THEN** the output equals the serial output bit for bit

#### Scenario: mRMR selection is unchanged
- **WHEN** mRMR runs on the existing test fixtures under `parallel`
- **THEN** it selects the same features with the same scores as the serial path

### Requirement: Parallel thresholds are measured against the in-house primitive
Every size cut-off that gates a migrated fan-out SHALL be set from a benchmark of `deep_causality_par` against the serial loop on the benchmark machine.
The measurement and the machine SHALL be recorded in the constant's doc comment. Above the cut-off
the parallel path SHALL NOT be slower than the serial path.

#### Scenario: Threshold doc comment carries its measurement
- **WHEN** a reader opens a threshold constant in topology or fft
- **THEN** its doc comment states the serial and parallel timings on each side of the cut-off and names the benchmark machine
