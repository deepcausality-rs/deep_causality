## ADDED Requirements

### Requirement: BRCD loader configuration

A `BrcdLoaderConfig<T>` SHALL describe where BRCD's inputs live and how to run
the algorithm: a `normal_path`, an `anomalous_path`, an optional `cpdag_path`, a
`CsvConfig` for parsing both data files, and the reused algorithm `BrcdConfig<T>`.
The `csv` field SHALL default to `CsvConfig::default()` and the `brcd_config`
field SHALL default to `BrcdConfig::default()` when not overridden.

#### Scenario: Minimal config uses defaults

- **WHEN** a caller builds a `BrcdLoaderConfig` with only `normal_path` and `anomalous_path` set
- **THEN** the `csv` options default to `CsvConfig::default()` and `brcd_config` defaults to `BrcdConfig::default()`

#### Scenario: Reused algorithm config, not a duplicate

- **WHEN** a caller sets `brcd_config` to `BrcdConfig::continuous(seed)` or `BrcdConfig::discrete(seed)`
- **THEN** that exact `deep_causality_algorithms::BrcdConfig<T>` is carried unchanged into the BRCD stage with no conversion

### Requirement: BRCD input bundle

A `BrcdInput<T>` SHALL bundle everything the BRCD stage needs into one struct: the
`normal` and `anomalous` `CausalTensor<T>` matrices, an optional `cpdag`
(`MixedGraph<()>`), and the `BrcdConfig<T>`. It is the value passed to
`load_brcd_input`.

#### Scenario: Bundle feeds the BRCD stage

- **WHEN** `load_brcd_input(input)` is called with a `BrcdInput<T>`
- **THEN** the BRCD stage has access to both datasets, the optional CPDAG, and the config without further loading

### Requirement: BrcdDataLoader builds the bundle

`BrcdDataLoader::load(&BrcdLoaderConfig<T>)` SHALL load both datasets through the
existing CSV loader, cast the loaded `f64` values into the pipeline precision `T`,
load the CPDAG when a path is supplied, and return a `BrcdInput<T>` or a
`BrcdLoadError`. The two datasets SHALL be validated to be 2-D and to share the
same number of variables.

#### Scenario: Both datasets loaded and aligned

- **WHEN** `load` is called with valid `normal_path` and `anomalous_path` over the same variables
- **THEN** it returns a `BrcdInput<T>` whose `normal` and `anomalous` tensors have equal column counts

#### Scenario: Mismatched variable counts are rejected

- **WHEN** the normal and anomalous datasets have different numbers of columns
- **THEN** `load` returns a `BrcdLoadError` and does not produce a bundle

#### Scenario: Non-2-D data is rejected

- **WHEN** either loaded dataset is not a 2-D matrix
- **THEN** `load` returns a `BrcdLoadError`

### Requirement: Absent CPDAG defers to BOSS

When `cpdag_path` is `None`, `BrcdDataLoader::load` SHALL set the bundle's `cpdag`
to `None`. The loader SHALL NOT run BOSS itself; passing `None` to `brcd_run` is
what triggers BOSS CPDAG learning inside the algorithm.

#### Scenario: No CPDAG path yields a None cpdag

- **WHEN** `load` is called with `cpdag_path = None`
- **THEN** the returned `BrcdInput<T>` has `cpdag == None` and the loader performs no structure learning

### Requirement: CPDAG CSV format faithful to the typed-endpoint model

The CPDAG file SHALL be a CSV that records each edge as the canonical pair plus
both endpoint marks, mirroring the `MixedGraph` representation
(`Mark ∈ {Tail, Arrow, Circle}`). The file SHALL carry the vertex count in a
comment header line (`# … vertices=N`) and SHALL use rows of
`src,dst,mark_src,mark_dst` with marks spelled as full words. The format SHALL
round-trip directed, undirected, bidirected, and circle-marked edges.

#### Scenario: Round-trip preserves structure

- **WHEN** a `MixedGraph` is written with `save_cpdag_csv` and read back with `load_cpdag_csv`
- **THEN** the loaded graph has the same vertex count and the same set of edges with identical endpoint marks

#### Scenario: Isolated vertices are preserved

- **WHEN** a graph with a vertex that has no incident edges is saved and reloaded
- **THEN** the reloaded graph reports the same `num_vertices`, including the isolated vertex

### Requirement: CPDAG load builds a unit-payload graph and validates size

`load_cpdag_csv(path)` SHALL build a `MixedGraph<()>` over the `N` vertices read
from the header, applying each edge row via the graph's mutation API. When used
by the BRCD loader, the CPDAG's vertex count SHALL be validated to equal the
datasets' variable count.

#### Scenario: Edge rows applied by marks

- **WHEN** `load_cpdag_csv` reads a row `0,1,Tail,Arrow`
- **THEN** the resulting graph has a directed arc `0 → 1`

#### Scenario: CPDAG size mismatch is rejected

- **WHEN** a CPDAG file declares a vertex count different from the datasets' variable count
- **THEN** the BRCD loader returns a `BrcdLoadError` rather than producing a bundle

#### Scenario: Malformed CPDAG content is rejected

- **WHEN** a CPDAG file contains an unknown mark token, a non-numeric vertex index, or an out-of-range vertex
- **THEN** `load_cpdag_csv` returns a `CpdagError`
