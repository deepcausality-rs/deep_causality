## MODIFIED Requirements

### Requirement: Typed table reading as a lazy IO action

`deep_causality_file` SHALL provide a reader for delimited numeric tables (test matrices,
atmosphere tables, flow-rate schedules) expressed over the `deep_causality_haft::IoAction`
pattern: constructing the reader performs no side effect, and `.run()` executes the read
exactly once. The reader SHALL parse column names from the first row, units from an optional
`#units`-prefixed second row, and numeric cells through the `TableScalar` codec into the
caller's precision-generic scalar `R`, so a cell written by the codec recovers `R` exactly at
every supported precision (`f64`, `f32`, `Float106`) and a plain decimal literal lifts through
exact `f64`. Malformed input (ragged rows, non-numeric cells, missing header) SHALL produce a
descriptive loading error naming the path and location, never a default value.

#### Scenario: A test matrix loads typed and lazily

- **WHEN** a Mach-altitude test matrix CSV is described by the reader and then `.run()`
- **THEN** no filesystem access happens before `.run()`, the result carries the column names
  and units alongside the typed rows, and every value round-trips into `R` through the codec

#### Scenario: Malformed input is an error, not a guess

- **WHEN** a table with a ragged row or a non-numeric cell is loaded
- **THEN** the reader returns an error naming the file and the offending row, and no partial
  table is returned

### Requirement: Result tables carry their column semantics

`deep_causality_file` SHALL provide a result-table writer, precision-generic over any
`R: TableScalar`, that emits the same two-row-header shape the reader consumes: a column-name
row, a `#units` row, then data rows. A table written by the writer SHALL be readable by the
typed table reader with names, units, and bit-identical values preserved at the written
precision (`read(write(t)) == t`). The writer SHALL NOT downcast a wider scalar to `f64`. The
shape SHALL be the table payload the future self-describing results archive adopts unchanged.

#### Scenario: Write-read round trip preserves semantics and bits at every precision

- **WHEN** a result table of `R` values with named, unit-annotated columns is written and read
  back, for each of `R = f64`, `R = f32`, `R = Float106`
- **THEN** the recovered names, units, and values are identical, with values equal bit-for-bit
  at the written precision

## ADDED Requirements

### Requirement: One symmetric precision codec bounds reader and writer

`deep_causality_file` SHALL define a `TableScalar` trait that both the reader and the writer
bound on, providing a cell encoder and a cell parser such that parsing an encoded cell recovers
the exact source bits. `TableScalar` SHALL be implemented for `f64` and `f32` (shortest
round-trip formatting) and for `Float106` (an exact two-component `hi|lo` pair cell), and every
implementation's parser SHALL also accept a plain decimal literal, lifted through exact `f64`,
so a hand-authored specification table loads at any precision.

#### Scenario: Float106 survives a text round trip

- **WHEN** a `Float106` value is encoded to a cell and parsed back
- **THEN** the recovered value equals the original in both the `hi` and `lo` components

#### Scenario: A hand-authored decimal loads at any precision

- **WHEN** a table cell containing a plain decimal literal is read as `R = Float106`
- **THEN** the value lifts through exact `f64` into `R` without error

### Requirement: The row schema lives on the row type

`deep_causality_file` SHALL define a `TableRow` trait carrying an associated `Scalar: TableScalar`,
a static `SCHEMA` of `(name, unit)` pairs in column order, and a `cells()` projection; and a
`FromTableRow` trait carrying `from_cells()`. The writer verb `write_rows` SHALL accept a slice
of `T: TableRow` and emit the schema's names and units once. The reader verb `read_rows` SHALL
accept `T: FromTableRow`, match the file header names to `SCHEMA` by name, deliver each row's
cells to `from_cells` in schema order regardless of file column order, and report a missing
column by name.

#### Scenario: Column names appear once, on the row type

- **WHEN** a `Vec<T: TableRow>` is written with `write_rows`
- **THEN** the emitted header carries the schema's names and units, and the program declares
  those names only in the `TableRow` impl

#### Scenario: read_rows reorders file columns to schema order

- **WHEN** a file whose columns are ordered differently from `SCHEMA` (with an extra column) is
  read with `read_rows::<T>`
- **THEN** each `from_cells` call receives cells in schema order, and an absent required column
  is reported as an error naming that column

### Requirement: Named column access

`NumericTable::column(name)` SHALL return the named column's values in row order as a
`Result`, and SHALL return an error naming the column when it is absent, replacing positional
row indexing.

#### Scenario: A missing column names itself

- **WHEN** `column("airspeed")` is requested on a table that has no such column
- **THEN** the call returns an error naming `"airspeed"`, not a panic or an empty column
