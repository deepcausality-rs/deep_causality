# retropulsion-branch-scoring Specification

## Purpose
Scoring of retropulsion counterfactual branches from the state each branch actually flew.

## Requirements

### Requirement: Branch rows are scored from the flown state

Every scored branch quantity SHALL be read from that branch's report, and no scored quantity may be
re-derived from a roster constant or a hardcoded force. The commanded throttle is an input to the
world; the realized throttle is what the safety gate admitted, and only the realized value may enter
a score. `BASE_AXIAL_DRAG_N` is removed, because the aerodynamic drag the branch flew is already
carried on the force channel.

#### Scenario: A clamped branch is scored at the throttle it flew

- **WHEN** a branch publishes a commanded throttle above the envelope's dynamic thrust-coefficient
  ceiling, and the gate clamps it
- **THEN** the branch's recorded throttle is the clamped value, and its recorded axial deceleration
  is consistent with that clamped value rather than with the commanded one

#### Scenario: Two branches that fly the same throttle score the same

- **WHEN** two roster entries are both clamped to the same realized throttle
- **THEN** their recorded deceleration, propellant and flow columns are equal, so the recorded table
  cannot present one flight as several

### Requirement: The axial deceleration witness is read off the force channel

A per-step stage SHALL publish the along-velocity component of the summed force channel as a field
scalar, positive for deceleration, and the branch score MUST read it. The channel carries a specific
force in m·s⁻², so the published witness is in m·s⁻² and no newton-valued constant participates.

#### Scenario: The coasting branch reports positive deceleration

- **WHEN** a branch flies at zero throttle through atmospheric drag
- **THEN** its recorded axial deceleration is positive, and it is the largest aerodynamic
  deceleration in the roster

#### Scenario: Thrust and drag carry one sign convention

- **WHEN** thrust and aerodynamic drag both act along the negative flight-velocity direction
- **THEN** both contribute with the same sign to the published witness

### Requirement: The frozen-drag foil is a trajectory integral

The frozen-drag comparison SHALL accumulate two along-velocity velocity increments over the
continuation — the realized one, and one computed with the preserved-drag fraction held at its value
at the fork — so the comparison is between two trajectories. An algebraic difference whose thrust term
cancels does not satisfy this requirement.

#### Scenario: The foil differs from the realized trajectory only through the drag closure

- **WHEN** a branch continues from the fork under a thrust schedule
- **THEN** the frozen-drag increment uses that same realized thrust schedule with the fork's drag
  fraction, and the two increments are both reported in m·s⁻¹

### Requirement: The coast branch's preserved-drag fraction is measured

The preserved-drag fraction SHALL be read from the branch's report for every branch, including a
branch commanded to zero throttle. No branch may be assigned a literal fraction.

#### Scenario: A branch that fires for one step reports the fraction it published

- **WHEN** a branch inherits a non-zero throttle on the channel at fork time and its propulsion stages
  run for one step before the guidance overwrites the command
- **THEN** the recorded fraction is the one the plume stage published on that step, and the recorded
  propellant consumption accounts for that step

### Requirement: A degenerate roster fails the run

A gate SHALL fail when two branches' realized throttles agree within a pinned tolerance, so a roster
silently collapsed by the safety envelope is a run failure rather than an invisible condition.

#### Scenario: A roster collapsed by the dynamic ceiling is rejected

- **WHEN** three roster entries are all clamped to the same realized throttle
- **THEN** the roster non-degeneracy gate fails and names the colliding branches
