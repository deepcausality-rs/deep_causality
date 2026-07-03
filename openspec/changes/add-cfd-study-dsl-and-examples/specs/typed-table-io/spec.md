## ADDED Requirements

### Requirement: One-call table construction from columns and rows

`NumericTable` SHALL offer `from_columns`, constructing a table from an array of
`(name, unit)` pairs and a row collection in one call, with the same rectangularity
validation and round-trip guarantees as the existing constructor.

#### Scenario: Columns and rows assemble in one call

- **WHEN** a table is built with `from_columns([("mach", "-"), ("q", "kPa")], rows)`
- **THEN** it equals the table built through explicit `TableColumn` construction, and a
  ragged row collection is rejected the same way
