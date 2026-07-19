# weather-table-consumption Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-cfd-contracts. Update Purpose after archive.
## Requirements
### Requirement: Value-bracketed keyed interpolation is a reusable library type

`deep_causality_cfd` SHALL provide a schema-agnostic `KeyedTable<R>` lookup type — the N-column
generalization of `DescentSchedule::sample` — carrying value-bracketed linear interpolation with
end clamping, so the measured-parameter lookup the weather example needs is a tested library
primitive rather than example code. (House rule: examples are example code only; general-purpose
logic lives in a lib crate and is tested there. This supersedes the original placement of the
loader in `avionics_examples::shared` — design D7/D8 — for the reusable core.) `KeyedTable::new`
MUST sort rows ascending by key, reject duplicate keys as an error, and reject ragged column
counts; `KeyedTable::interpolate` MUST bracket the query **by key value** (never by insertion
order), interpolate every column linearly, and clamp a query outside the tabulated range to the
nearest end row. The type MUST perform no logging (it is a pure config-time consumer).

#### Scenario: A mid-range key interpolates its true value neighbors

- **WHEN** a table built from unsorted keys (the committed weather run order 0, +20, −25, −40, −5,
  +5 K) is interpolated at `dT = −15`
- **THEN** the result brackets the −25 K and −5 K rows (not insertion-order neighbors), and every
  interpolated column lies between its bracketing values

#### Scenario: Duplicate keys and ragged rows are rejected

- **WHEN** a table is built with two rows sharing a key, or with rows of differing column counts
- **THEN** construction returns an error and no table is produced

### Requirement: The clamp is a marker carried, not logged

`KeyedTable::interpolate` SHALL report a query outside the tabulated key range through a `clamped`
marker on its result (alongside the interpolated values and the bracketing row indices), and SHALL
itself perform no logging — the `EffectLog` lives on the coupled field, which this config-time type
never sees. Stamping a clamped result into flight provenance is the consuming stage's
responsibility (the M5 example), so a day flown outside the tabulated dispersion range is auditable
without coupling the lookup to the flight stack.

#### Scenario: An out-of-range key clamps with the marker set

- **WHEN** the committed table is interpolated at `dT = −60` (below the −40 K row)
- **THEN** the result equals the −40 K row with `clamped` set, and the lookup writes no log entry

### Requirement: The M5 example binds the reusable lookup to the weather CSV

The M5 retropulsion example SHALL load the recorded dispersion table
(`cfd/plasma_blackout/weather/weather_table.csv`) through the existing `deep_causality_file` typed
reader (`read_rows` with a `FromTableRow` consumption row type bound to the `WorldRow::SCHEMA`
column names), feed the parsed rows into `KeyedTable`, and stamp a clamped interpolation into the
flight `EffectLog`. A missing required column MUST surface as the reader's named-column error, and
a malformed cell as a loading error — never a default value. This binding is example glue (example
code only, no example-crate tests) and is deferred to M5, which owns the retropulsion example
folder; M2 delivers and tests the reusable `KeyedTable` core it stands on.

#### Scenario: Schema drift is a named error

- **WHEN** the M5 loader reads a table missing the `drift_sd` column
- **THEN** `read_rows` returns an error naming `drift_sd`, and no partial result is produced

