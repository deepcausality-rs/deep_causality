# typed-table-io Specification

## Purpose
Reading delimited numeric tables into typed, precision-generic rows and writing result tables with headers and units, both as lazy IoAction values in deep_causality_file: the input and output seam of the everyday CFD examples and the table payload of the future self-describing results archive.

## Requirements
### Requirement: Typed table reading as a lazy IO action

`deep_causality_file` SHALL provide a reader for delimited numeric tables (test matrices,
atmosphere tables, flow-rate schedules) expressed over the `deep_causality_haft::IoAction`
pattern: constructing the reader performs no side effect, and `.run()` executes the read
exactly once. The reader SHALL parse column names from the first row, units from an optional
`#units`-prefixed second row, and numeric values as exact `f64` lifted into the caller's
precision-generic scalar `R` at the boundary. Malformed input (ragged rows, non-numeric cells,
missing header) SHALL produce a descriptive loading error naming the path and location, never a
default value.

#### Scenario: A test matrix loads typed and lazily

- **WHEN** a Mach-altitude test matrix CSV is described by the reader and then `.run()`
- **THEN** no filesystem access happens before `.run()`, the result carries the column names
  and units alongside the typed rows, and every value round-trips the exact `f64` literal from
  the file into `R`

#### Scenario: Malformed input is an error, not a guess

- **WHEN** a table with a ragged row or a non-numeric cell is loaded
- **THEN** the reader returns an error naming the file and the offending row, and no partial
  table is returned

### Requirement: Result tables carry their column semantics

`deep_causality_file` SHALL provide a result-table writer that emits the same two-row-header
shape the reader consumes: a column-name row, a `#units` row, then data rows. A table written
by the writer SHALL be readable by the typed table reader with names, units, and bit-identical
`f64` values preserved (round trip). The shape SHALL be the table payload the future
self-describing results archive adopts unchanged.

#### Scenario: Write-read round trip preserves semantics and bits

- **WHEN** a result table with named, unit-annotated columns is written and read back
- **THEN** the recovered names, units, and values are identical, with `f64` values equal
  bit-for-bit
