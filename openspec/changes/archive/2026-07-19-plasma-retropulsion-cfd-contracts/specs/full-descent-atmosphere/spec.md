## ADDED Requirements

### Requirement: Atmosphere rows extend to the ground

The shared `ATMOSPHERE` table (`examples/avionics_examples/src/shared/constants.rs`) SHALL
extend from its current 30 km floor to 0 km with US-1976-shaped rows in the existing
four-column format `(altitude m, n_tot m⁻³, T K, a m/s)`, inserted below the existing rows in
ascending-altitude order. The existing five rows MUST remain byte-identical, and each new row
MUST carry a comment citing its US Standard Atmosphere 1976 pinpoint. The new rows MUST be
internally consistent: sound speed agrees with `a = √(γ R_s T)` at `γ = 1.4` within transcription
tolerance, and number density decreases monotonically with altitude across the whole table.

#### Scenario: The table reaches the ground with its original rows intact

- **WHEN** the extended `ATMOSPHERE` is inspected
- **THEN** its first row is at 0 m, its rows ascend in altitude, the five pre-existing rows are
  byte-identical to the committed table, and every new row's `(n_tot, T, a)` triple is
  consistent with US-1976

### Requirement: The sampler's clamp moves by data alone

The atmosphere extension SHALL require no change to `DescentSchedule::sample`: the sampler
already clamps to the table ends, so appending rows below 30 km relocates the low clamp to 0 km
purely through data. Above 30 km, sampling the extended table MUST return values identical to
sampling the original table, because interpolation between the original rows is untouched.

#### Scenario: Above the old floor, nothing changes

- **WHEN** the schedule is sampled at altitudes spanning 30–90 km against both the original and
  the extended table
- **THEN** every sampled row is identical between the two tables

#### Scenario: Below the old floor, the atmosphere is live

- **WHEN** the schedule is sampled at 15 km against the extended table
- **THEN** the returned state interpolates the new rows (denser and warmer than the 30 km row's
  frozen values) instead of clamping to the 30 km row
