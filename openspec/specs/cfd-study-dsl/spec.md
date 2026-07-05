# cfd-study-dsl Specification

## Purpose
TBD - created by archiving change add-cfd-study-dsl-and-examples. Update Purpose after archive.
## Requirements
### Requirement: A parameter sweep is one combinator, deterministic and output-generic

`deep_causality_cfd` SHALL provide `sweep`, an order-preserving map over a slice of case
inputs collecting `Result`s with first-error-wins semantics. Under the `parallel` feature the
sweep SHALL run on scoped threads and produce results bit-identical to the sequential run;
without the feature it SHALL be a plain map. The output type SHALL be generic: a study may
return row arrays, reports, or domain structs, and a study that runs no march at all (a
pointwise table) SHALL be expressible with the same combinator.

#### Scenario: A swept study is order-preserving and bit-identical

- **WHEN** the same sweep runs with and without the `parallel` feature
- **THEN** the collected outputs are identical bit for bit and appear in input order

#### Scenario: The first error stops the sweep

- **WHEN** one case in a sweep returns an error
- **THEN** the sweep returns that error and no partial result set

### Requirement: Acceptance gates are one builder

`deep_causality_cfd` SHALL provide `Gates`: a builder accumulating `(label, pass, detail)`
entries under a title, whose `finish` prints one `[PASS]` or `[FAIL]` line per gate plus a
closing verdict line and returns whether all gates passed. The builder SHALL NOT exit the
process, format numbers, or add output beyond those lines, so the caller keeps process
control and precision display and the existing hand-rolled gate blocks can migrate by
substitution.

#### Scenario: A failing gate is visible and the verdict is false

- **WHEN** a `Gates` set with one failing entry finishes
- **THEN** that entry prints as `[FAIL]` with its label and detail, the verdict line reports
  the regression, and `finish` returns false

### Requirement: One-shot geometry on the march pipeline

The march pipeline SHALL offer `run_owned`, which materializes the
case's geometry internally, runs, and returns the report, for sweep bodies where each case
owns a fresh grid. The caller-owned-geometry form (`materialize` plus `.on`) SHALL remain
unchanged as the primary API for geometry reuse.

#### Scenario: A sweep body needs no manifold ceremony

- **WHEN** a swept case is run with `run_owned`
- **THEN** the report equals the one produced by the explicit `materialize`-and-`.on` form
  for the same case

