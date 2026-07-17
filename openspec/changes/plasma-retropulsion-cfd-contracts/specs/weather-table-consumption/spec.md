## ADDED Requirements

### Requirement: The weather table loads through the typed reader

The shared library (`avionics_examples::shared`) SHALL provide a weather-table loader that reads
the recorded dispersion table (`cfd/plasma_blackout/weather/weather_table.csv`) through the
existing `deep_causality_file` typed reader (`read_rows` with a `FromTableRow` consumption row
type bound to the `WorldRow::SCHEMA` column names), not a hand-rolled parser. A missing required
column MUST surface as the reader's named-column error, and a malformed cell as a loading error
— never a default value.

#### Scenario: The committed table loads typed

- **WHEN** the loader reads the committed `weather_table.csv`
- **THEN** it returns six typed rows whose `d_temp`, `onset`, `exit`, `dwell`, `drift_mean`,
  and `drift_sd` values equal the recorded cells

#### Scenario: Schema drift is a named error

- **WHEN** a table missing the `drift_sd` column is loaded
- **THEN** the loader returns an error naming `drift_sd`, and no partial result is returned

### Requirement: Interpolation is value-bracketed over sorted unique keys

The loader SHALL sort the loaded rows ascending by `d_temp` after load (the recorded rows arrive
in run order, not temperature order), MUST reject duplicate `d_temp` keys as an error, and SHALL
interpolate at the measured temperature departure `dT` by selecting the bracketing pair **by
value** and interpolating every numeric column linearly between them. A `dT` outside the
tabulated range SHALL clamp to the nearest row. Bracketing by file order is prohibited: it would
select non-adjacent temperatures for most inputs, and the row feeds load-bearing margins.

#### Scenario: A mid-range dT interpolates its true neighbors

- **WHEN** the committed table (run order 0, +20, −25, −40, −5, +5 K) is loaded and interpolated
  at `dT = −15`
- **THEN** the result interpolates between the −25 K and −5 K rows, and every interpolated
  column lies between its bracketing values

#### Scenario: Duplicate keys are rejected

- **WHEN** a table containing two rows with the same `d_temp` is loaded
- **THEN** the loader returns an error identifying the duplicate key

### Requirement: The clamp is a marker the flight side stamps into provenance

The interpolation result SHALL carry the interpolated row, the bracketing pair (or the single
clamped row), and a `clamped` marker; the loader itself SHALL perform no logging. When the
flight side consumes a clamped result, the clamp MUST be stamped into the `EffectLog`
provenance, so a day flown outside the tabulated dispersion range is auditable.

#### Scenario: An out-of-range dT clamps with the marker set

- **WHEN** the committed table is interpolated at `dT = −60` (below the −40 K row)
- **THEN** the result equals the −40 K row with `clamped` set, and a consumer stamping the
  result produces a provenance entry recording the clamp and the requested dT
