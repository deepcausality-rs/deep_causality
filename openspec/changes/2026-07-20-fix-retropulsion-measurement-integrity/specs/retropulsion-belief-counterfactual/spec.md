## ADDED Requirements

### Requirement: The uninformed world is flown

The belief counterfactual SHALL march a second world whose guidance carries the standard-day margin,
and the gate MUST compare a flight outcome between the two worlds. Subtracting two interpolations of
one table produces a number that is invariant to the entire flight, so it cannot establish that the
table changed the flight.

#### Scenario: Both beliefs produce a flown world

- **WHEN** the example runs
- **THEN** an informed world and an uninformed world are each marched from the same baseline, and both
  carry the context-alternation record naming the world they replace

#### Scenario: The gate reads a flight difference

- **WHEN** the belief gate evaluates
- **THEN** it compares a flown outcome — commit step, propellant at touchdown, or miss to the aim
  point — rather than the difference between two table lookups

### Requirement: The ignition margin is sized so it can bind

The margin the commit demands SHALL be sized against the navigated uncertainty the flight actually
achieves. A margin two orders of magnitude above the navigated sigma admits every commit under either
belief, so the two worlds fly identically and the counterfactual is empty.

#### Scenario: The two beliefs separate, or the finding is recorded

- **WHEN** the informed and uninformed margins are applied to the same measured atmosphere
- **THEN** either the commits differ and the gate measures that difference, or the run records that
  the table does not change this flight and the gate states that finding

### Requirement: A table clamp is stamped into provenance and gated

An interpolation that clamped to the nearest tabulated row SHALL be recorded in the provenance log and
read by a gate. The table type is pure by design and leaves the stamping to the flight side; the
flight side currently decorates a console line instead.

#### Scenario: A measured departure outside the table is visible in the run

- **WHEN** the measured temperature departure falls outside the tabulated range
- **THEN** the provenance log carries the clamp with the departure and the row it clamped to, and a
  gate reports it

### Requirement: The interpolated row is the single source for the day's belief

Every quantity the day's belief describes SHALL be read from the interpolated row, and no quantity the
belief prints may be duplicated as a hand-set constant. A printed value that the coupling does not
receive is removed from the output or passed to the coupling.

#### Scenario: The flown atmosphere matches the printed belief

- **WHEN** the day's density scale is printed
- **THEN** it is the value the flown atmosphere was built from

#### Scenario: A printed sensor departure is flown

- **WHEN** the accelerometer bias departure is printed as part of the day flown
- **THEN** the coupling's inertial model was constructed with that departure
