## ADDED Requirements

### Requirement: CdlConfigBuilder is the single source of truth

The run configs (`SurdLoaderConfig<T>`, `BrcdLoaderConfig<T>`) SHALL have no public
constructor. `CdlConfigBuilder` SHALL be the only way to build them, via staged
builders started by `build_surd_config::<T>()` and `build_brcd_config()`.

#### Scenario: Configs are built only through the builder

- **WHEN** a caller needs a `SurdLoaderConfig` or `BrcdLoaderConfig`
- **THEN** the only available constructor is the corresponding `CdlConfigBuilder` staged builder

### Requirement: Compile-time required fields, runtime file-exists check

Each staged builder SHALL expose `build()` only once every required field is set,
so omitting a required field is a compile error. SURD requires `path`,
`target_index`, `num_features`, `max_order`, and `analyze`; BRCD requires
`normal_path`, `anomalous_path`, and `brcd_config` (the reused algorithm config,
mandatory — no hidden default). Optional fields (SURD `exclude_indices`/`csv`,
BRCD `cpdag_path`/`csv`) SHALL default. `build()` SHALL verify the referenced files
exist, returning a `CdlError` if not.

#### Scenario: Building with all required fields and existing files succeeds

- **WHEN** `build()` is called after all required fields are set and the files exist
- **THEN** it returns the product config

#### Scenario: Missing file is rejected at build time

- **WHEN** `build()` is called and a referenced dataset or CPDAG file does not exist
- **THEN** it returns a `CdlError::ReadDataError`

#### Scenario: The reused algorithm config is mandatory and explicit

- **WHEN** a caller builds a BRCD config
- **THEN** `with_brcd_config(BrcdConfig::…)` is required, carrying that exact `deep_causality_algorithms::BrcdConfig<T>` unchanged into the run config

### Requirement: BRCD input is loaded in-pipeline

Loading the two datasets and optional CPDAG SHALL be an in-pipeline stage
(`brcd_load_input`) that invokes the internal loader, not a separate user-facing
call. The two datasets SHALL be validated to be 2-D and to share the same number
of variables; a CPDAG, when present, SHALL be validated to have a matching vertex
count.

#### Scenario: Both datasets loaded and aligned

- **WHEN** `brcd_load_input` runs with a config naming two datasets over the same variables
- **THEN** it produces a bundle whose normal and anomalous tensors have equal column counts

#### Scenario: Mismatched variable counts are rejected

- **WHEN** the two datasets have different numbers of columns
- **THEN** `brcd_load_input` yields a `CdlError` wrapping a BRCD load error

### Requirement: Absent CPDAG defers to BOSS

When the config has no CPDAG path, the loaded bundle's CPDAG SHALL be `None`, and
the loader SHALL NOT run BOSS itself; passing `None` to `brcd_run` is what triggers
BOSS CPDAG learning inside the algorithm.

#### Scenario: No CPDAG path yields a None cpdag

- **WHEN** a BRCD config has no CPDAG path
- **THEN** the loaded bundle's cpdag is `None` and no structure learning occurs in the loader

### Requirement: CPDAG CSV format faithful to the typed-endpoint model

The CPDAG file SHALL be a CSV recording each edge as the canonical pair plus both
endpoint marks (`Mark ∈ {Tail, Arrow, Circle}`, full-word tokens), with the vertex
count in a `# … vertices=N` comment header and rows of `src,dst,mark_src,mark_dst`.
The format SHALL round-trip directed, undirected, bidirected, and circle-marked
edges, and SHALL preserve isolated vertices. `load_cpdag_csv` and `save_cpdag_csv`
SHALL be public.

#### Scenario: Round-trip preserves structure

- **WHEN** a `MixedGraph` is written with `save_cpdag_csv` and read back with `load_cpdag_csv`
- **THEN** the loaded graph has the same vertex count and the same edges with identical endpoint marks

#### Scenario: Malformed content is rejected

- **WHEN** a CPDAG file is missing the `vertices=N` header, has an unknown mark token, or names an out-of-range vertex
- **THEN** `load_cpdag_csv` returns the corresponding `CpdagError`
