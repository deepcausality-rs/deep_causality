# nozzle-operating-map Specification

## Purpose
TBD - created by archiving change add-cfd-study-dsl-and-examples. Update Purpose after archive.
## Requirements
### Requirement: The nozzle operating map is a self-verifying example

`examples/avionics_examples/cfd/nozzle_operating_map` SHALL sweep back pressure on a
converging-diverging duct through `sweep` and `CfdFlow::duct_march`, and SHALL produce one
operating-map table (back pressure, choking state, shock position, thrust coefficient)
written through the group-1 result-table writer. Every row SHALL be gated against closed
forms: choking against the critical pressure ratio, the shock position against the analytic
normal-shock placement, and the shock-free profiles against the area-Mach relation, with each
band's derivation written next to its constant. The example SHALL compute in its `FloatType`
alias, exit nonzero on any gate regression, and read its back-pressure schedule from a file
through the group-1 table reader.

#### Scenario: The sweep brackets the critical ratio and the map is gated per row

- **WHEN** the example runs its recorded back-pressure schedule
- **THEN** rows above the first critical ratio gate against the area-Mach relation, rows
  below it gate choking and the shock position, the table writes with named and unit-carrying
  columns, and all gates pass with exit code zero

#### Scenario: A wrong usage speaks engineering

- **WHEN** the back-pressure schedule file has a non-numeric cell
- **THEN** the example fails with the reader's error naming the file, row, and column, and no
  partial table is written

