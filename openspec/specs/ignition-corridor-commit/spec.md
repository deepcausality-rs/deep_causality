# ignition-corridor-commit Specification

## Purpose
TBD - created by archiving change add-retropulsion-terminal-descent. Update Purpose after archive.
## Requirements
### Requirement: The ignition corridor is a conjunction of four conditions

The ignition-corridor commit SHALL require all four of: the flight Mach inside the configured
ignition band, the sensed dynamic pressure inside the envelope's `[q_min, q_max]` window, a
post-fix navigation state, and a navigated position uncertainty inside the table-sized margin. The
margin MUST be supplied by the caller rather than derived inside the guidance: the ignition-altitude
kernel takes its margin as an input by design, because downstream it is sized from the weather
dispersion table's navigation-drift row, which is not the physics crate's business. Any condition
failing leaves the corridor uncommitted and the commanded throttle at zero.

#### Scenario: All four conditions hold

- **WHEN** the Mach is inside the band, `q∞` is inside the window, the navigation state is aided,
  and the position uncertainty is inside the margin
- **THEN** the corridor commits on that step

#### Scenario: One condition short leaves the corridor uncommitted

- **WHEN** three conditions hold and the dynamic pressure is outside the window
- **THEN** no commit occurs, the commanded throttle stays zero, and no commit entry is logged

### Requirement: The commit is a rising edge and then a one-way latch

The commit SHALL be evaluated on each step until the conjunction first holds, and MUST latch
thereafter, so that a momentary loss of a condition does not extinguish a burn already under way.
The latch MUST ride a field scalar rather than carrier-internal state, because carrier-internal
state is reset at every leg boundary while the coupled field is carried across it. A rising edge
cannot be recovered from the throttle channel alone — the channel is an `Option` that is never
cleared between steps and is carried by clone, so a positive value persists after its producer goes
quiet and a forked branch inherits its parent's last command — so the commit MUST carry its own
edge state rather than inferring one.

#### Scenario: A transient condition loss does not extinguish the burn

- **WHEN** the corridor has committed and the navigation state subsequently degrades to dead
  reckoning
- **THEN** the burn continues, and safety is enforced by the envelope rather than by re-opening the
  commit decision

#### Scenario: The latch survives a leg boundary

- **WHEN** a committed burn crosses into a new march leg
- **THEN** the commit is still latched, because it is carried on the coupled field rather than in
  carrier-internal state

#### Scenario: The commit fires once

- **WHEN** the conjunction holds on many consecutive steps
- **THEN** exactly one commit event is logged, on the first of them

### Requirement: The commit is logged as a transition event

The commit SHALL append one entry to the provenance log naming the step and the four sensed values
that satisfied the corridor, following the transition-only logging discipline the navigation-mode
flip already uses. The entry MUST be emitted on the committing step only, so the log records the
decision rather than the condition.

#### Scenario: The commit appears in provenance

- **WHEN** the corridor commits
- **THEN** one entry naming the commit and its sensed Mach, dynamic pressure, navigation mode, and
  position uncertainty appears in the log, and no further commit entries follow

### Requirement: The commit reads published navigation scalars, never the navigation engine

The commit SHALL read the navigation state through the published `"nav_mode"` and
`"nav_position_variance"` scalars and MUST NOT reach into `CoupledField::nav()`. Navigation is a
one-way sink in the library today — no stage feeds the nav estimate back into physics or control,
and the aero and control stages steer off the carried truth state and flow scalars — so the commit
is the first place a decision is taken on navigation quality. The published scalars exist for
exactly this purpose but have no production consumer: the corridor example reaches **past** them
into the engine's own accessor to gate on position variance, which is the coupling this requirement
exists to avoid. The pattern to follow is the navigation stage's own GNSS gate, which reads the
published regime class rather than another subsystem's internals.
Two properties of those scalars MUST be honored rather than assumed: `"nav_mode"` reports aided
versus dead reckoning and does **not** distinguish a GNSS fix from a through-plasma optical fix, so
a corridor requiring GNSS reacquisition specifically cannot be expressed against it today; and
`"nav_position_variance"` is a covariance **trace in m²**, so a margin stated in metres MUST compare
against its square root.

#### Scenario: The navigation engine is untouched

- **WHEN** the commit evaluates the navigation conditions
- **THEN** it reads only published scalars, and the navigation engine on the field is neither taken
  nor inspected

#### Scenario: The published scalar is the seam, not the engine accessor

- **WHEN** the commit needs the navigated position uncertainty
- **THEN** it reads `"nav_position_variance"` rather than obtaining it from the engine directly, as
  the corridor example currently does

#### Scenario: A metre-valued margin is compared in metres

- **WHEN** the margin is a distance derived from the dispersion table's drift columns
- **THEN** it is compared against the square root of the published variance trace, not against the
  trace itself

