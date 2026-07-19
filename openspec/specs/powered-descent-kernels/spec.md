# powered-descent-kernels Specification

## Purpose
Closed-form powered-descent kinematics kernels in `deep_causality_physics`: stopping distance,
ignition altitude (with a caller-supplied navigation-dispersion margin), and the suicide-burn
deceleration command. These are the Tier-A terminal-guidance closed forms of the
plasma-retropulsion descent, rejecting the thrust-to-weight ≤ 1 and ground-contact cases;
Apollo polynomial and convex powered-descent guidance are the named upgrade path.
## Requirements
### Requirement: Stopping distance and ignition altitude

The crate SHALL provide `stopping_distance_kernel` (`d = v²/(2·a_net)`) and
`ignition_altitude_kernel` (`h_ign = v²/(2·(a_T − g)) + margin`), the closed-form Tier-A
ignition solution of the retropulsion note, with the navigation-dispersion margin taken as an
input (the weather-table row supplies it downstream, not this crate). Both kernels MUST reject
`a_net ≤ 0` — a vehicle with thrust-to-weight at or below one cannot stop — and negative speed,
with typed errors; the docstrings cite Klumpp (1974) and Açıkmeşe–Ploen (2007) as the guidance
upgrade path beyond these closed forms.

#### Scenario: Kinematic identity

- **WHEN** the stopping distance is evaluated for representative (v, a_net) pairs
- **THEN** `d = v²/(2·a_net)` holds exactly, and the ignition altitude equals the stopping
  distance against `(a_T − g)` plus the supplied margin

#### Scenario: Thrust-to-weight rejection

- **WHEN** `a_T ≤ g` (net deceleration non-positive)
- **THEN** the kernel returns a typed `PhysicsError` instead of an infinite or negative
  altitude

### Requirement: Suicide-burn deceleration command

The crate SHALL provide `suicide_burn_deceleration_kernel` computing the commanded
deceleration `a_cmd = v²/(2h) + g` that just nulls velocity at the surface — the closed-form
feedback the terminal-guidance stage clamps into the envelope. The kernel MUST reject `h ≤ 0`
with a typed error and never panic.

#### Scenario: Command nulls at touchdown

- **WHEN** the command is integrated in a simple pointwise test from representative (v, h)
  initial conditions under constant g
- **THEN** velocity and altitude reach zero together within the integration tolerance

#### Scenario: Ground-contact rejection

- **WHEN** `h ≤ 0`
- **THEN** the kernel returns a typed `PhysicsError`

### Requirement: Family conventions hold

The kernels SHALL follow the crate's conventions (generics, typed errors, `PropagatingEffect`
wrappers, flat exports, mirrored tests with the Bazel suite, cited docstrings), matching the
rest of the propulsion family.

#### Scenario: Coverage and gates

- **WHEN** `cargo test -p deep_causality_physics` runs
- **THEN** every added kernel's nominal path, limit cases, and every rejection path are
  exercised (the crate's 100%-coverage rule for added code)

