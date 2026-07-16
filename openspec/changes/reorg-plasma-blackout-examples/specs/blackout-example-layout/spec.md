## ADDED Requirements

### Requirement: One family folder, one subfolder per example

The plasma-blackout example family SHALL live under
`examples/avionics_examples/cfd/plasma_blackout/`, with exactly one subfolder per example. The
corridor example SHALL live at `cfd/plasma_blackout/corridor/` and the weather-dispersion
example at `cfd/plasma_blackout/weather/`. A later sibling (e.g. the retropulsion descent) MUST
join as a new subfolder of the same parent rather than as a flat folder under `cfd/`.

#### Scenario: The two examples sit in the family folder

- **WHEN** the repository is inspected after the reorganization
- **THEN** `examples/avionics_examples/cfd/plasma_blackout/corridor/main.rs` and
  `examples/avionics_examples/cfd/plasma_blackout/weather/main.rs` exist, and the old flat
  folders `cfd/plasma_blackout_corridor/` and `cfd/plasma_blackout_weather/` do not

#### Scenario: Git history survives the move

- **WHEN** `git log --follow --oneline` is run on a moved file (e.g.
  `cfd/plasma_blackout/corridor/main.rs`)
- **THEN** the pre-move history of that file is listed

### Requirement: Example binary names are stable across the move

The example binary names SHALL remain `plasma_blackout_corridor` and
`plasma_blackout_weather`; only the `[[example]] path` entries in
`examples/avionics_examples/Cargo.toml` change. Every documented run command MUST work
unchanged.

#### Scenario: The corridor runs green under its unchanged name

- **WHEN** `cargo run --release -p avionics_examples --example plasma_blackout_corridor` is
  executed after the move
- **THEN** the run completes with exit code 0 and its full gate suite passes, with the same
  gate witnesses as the pre-move run

#### Scenario: The weather table runs green under its unchanged name

- **WHEN** `cargo run --release -p avionics_examples --example plasma_blackout_weather` is
  executed after the move
- **THEN** the run completes with exit code 0 and all eight table gates plus the wall-clock
  gate pass

### Requirement: Recorded artifacts live inside their example's subfolder

Each example SHALL write its recorded artifacts into its own subfolder: the corridor's branch
table to `cfd/plasma_blackout/corridor/corridor_branches.csv`, the weather example's dispersion
table to `cfd/plasma_blackout/weather/weather_table.csv` and its audit logs under
`cfd/plasma_blackout/weather/audit/`.

#### Scenario: The corridor's branch table lands beside its sources

- **WHEN** the corridor example runs after the move
- **THEN** `corridor_branches.csv` is (re)written at
  `examples/avionics_examples/cfd/plasma_blackout/corridor/corridor_branches.csv`

#### Scenario: The weather artifacts land beside their sources

- **WHEN** the weather example runs after the move
- **THEN** `weather_table.csv` is (re)written at
  `examples/avionics_examples/cfd/plasma_blackout/weather/weather_table.csv` and the per-branch
  audit files land under `examples/avionics_examples/cfd/plasma_blackout/weather/audit/`

### Requirement: Live references resolve; archives stay untouched

All live references to the example folders (Cargo.toml paths, embedded
`CARGO_MANIFEST_DIR`-relative path constants, README links, live openspec notes) SHALL point at
the new layout. Archived changes and archived notes MUST NOT be edited.

#### Scenario: No stale live path references remain

- **WHEN** the repository is searched for `cfd/plasma_blackout_corridor` or
  `cfd/plasma_blackout_weather`, excluding `openspec/changes/archive/`,
  `openspec/notes/archive/`, and `target/`
- **THEN** zero matches remain

#### Scenario: README cross-links resolve

- **WHEN** the links in `examples/avionics_examples/README.md` and the corridor↔weather README
  cross-links are followed
- **THEN** each linked file exists at its target path, and the corridor README names the shared
  library module as `avionics_examples::shared`
