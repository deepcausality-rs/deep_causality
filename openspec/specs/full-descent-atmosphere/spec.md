# full-descent-atmosphere Specification

## Purpose
TBD - created by archiving change plasma-retropropulsion-cfd-contracts. Update Purpose after archive.
## Requirements
### Requirement: Atmosphere rows extend to the ground

The shared `ATMOSPHERE` table (`examples/avionics_examples/src/shared/constants.rs`) SHALL hold
U.S. Standard Atmosphere 1976 rows from 0 km to 90 km at 1 km spacing in the four-column format
`(altitude m, n_tot m⁻³, T K, a m/s)`, in ascending-altitude order, generated from the standard's
defining constants (its Table 2 constants and Table 4 layer gradients) and cited to them. Between
86 km and 90 km the rows SHALL extend isothermally at the 86 km temperature, hydrostatic in geometric
altitude, and the table SHALL state that extension. No row MAY be scaled or pinned to a flight
condition: every row describes US-1976 at its own altitude. The rows MUST be internally consistent:
`n_tot = P/(kT)`, `a = √(γ R* T / M₀)` at `γ = 1.4`, and number density decreases monotonically with
altitude across the whole table.

#### Scenario: The table is US-1976 at its own altitudes

- **WHEN** the `ATMOSPHERE` rows at 61 km, 71 km and 72 km are inspected
- **THEN** their temperature, pressure and density agree with US-1976 Table I at those altitudes
  within 0.1 % (Table I, 61 km: 244.274 K, 19.157 Pa, 2.7321e-4 kg m⁻³; 71 km: 216.846 K, 4.4795 Pa,
  7.1966e-5 kg m⁻³)

#### Scenario: The table reaches the ground and its top

- **WHEN** the table is inspected
- **THEN** its first row is at 0 m with the US-1976 sea-level state, its last row is at 90 km, and
  its rows ascend in altitude at 1 km spacing

### Requirement: The sampler's clamp moves by data alone

`DescentSchedule::sample` SHALL clamp to the table ends and, between rows, SHALL interpolate the
logarithm of `n_tot` linearly in altitude, with temperature and sound speed interpolated linearly. A
table whose number density falls exponentially between two rows MUST be sampled exactly at every
altitude between them, so a coarse table cannot overstate density between rows.

#### Scenario: An exponential layer is reproduced between rows

- **WHEN** a two-row table holds `n_tot = n₀` at 0 m and `n₀·e⁻²` at 14 km and is sampled at 7 km
- **THEN** the sampled `n_tot` equals `n₀·e⁻¹` to round-off, and the sampled temperature is the
  linear midpoint

#### Scenario: The ends clamp

- **WHEN** the schedule is sampled below its first row or above its last row
- **THEN** it returns that end row unchanged

