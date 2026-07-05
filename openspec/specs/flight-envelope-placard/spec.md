# flight-envelope-placard Specification

## Purpose
TBD - created by archiving change add-cfd-study-dsl-and-examples. Update Purpose after archive.
## Requirements
### Requirement: The flight-envelope placard table is a self-verifying example

`examples/avionics_examples/cfd/flight_envelope_placard` SHALL read a Mach-altitude test
matrix through the group-1 table reader, compute dynamic pressure, the Rankine-Hugoniot
post-shock stagnation temperature, and Sutton-Graves stagnation-point heating per grid point
with existing cited kernels, and write one placard table through the group-1 writer. Gates
SHALL check every grid point against the stated q-max and temperature placards and SHALL
report which points, if any, sit outside the envelope. The example SHALL run no march and own
no manifold (the pointwise study path), compute in its `FloatType` alias, and exit nonzero on
regression.

#### Scenario: The placard grid is computed and gated pointwise

- **WHEN** the example runs its recorded Mach-altitude matrix
- **THEN** every grid point carries q, post-shock stagnation temperature, and heating, the
  placard gates pass on the recorded matrix, and the table writes with named and
  unit-carrying columns

#### Scenario: An out-of-envelope point is named, not averaged away

- **WHEN** a matrix containing one point beyond the q-max placard is run
- **THEN** the placard gate fails naming that Mach-altitude point and the example exits
  nonzero

