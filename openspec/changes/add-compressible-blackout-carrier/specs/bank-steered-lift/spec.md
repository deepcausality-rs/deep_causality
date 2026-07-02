## ADDED Requirements

### Requirement: A 3-DOF bank-steered lift stage actuates the aero channel

`deep_causality_cfd` SHALL provide a `BankSteeredLift` corridor stage implementing point-mass
(3-DOF) entry aerodynamics: drag along the negative velocity direction from the dynamic-pressure
bundle, and lift `L = (L/D)·D` rotated about the velocity vector by the bank angle read from the
control channel — the same channel `CyberneticCorrect` clamps. The lift-plane basis SHALL derive
from the local radial (up) direction at the truth position. The stage SHALL write the resulting
full 3-vector acceleration into the ④ aero channel consumed by both the truth propagation and the
navigation predict. 6-DOF attitude dynamics SHALL NOT be modeled (no flight-data anchor exists;
explicitly out of scope).

#### Scenario: The clamped bank command steers the trajectory
- **WHEN** the corridor runs with a nonzero clamped bank command
- **THEN** the ④ channel carries drag plus a lift component rotated by that command, the truth and
  navigation trajectories curve accordingly, and a zero-bank run produces a measurably different
  trajectory

#### Scenario: Actuation uses the bounded command with a one-step lag
- **WHEN** the guidance commands a bank beyond the envelope cap
- **THEN** the lift stage actuates the *clamped* value from the gate (one between-step lag,
  documented), never the raw command

### Requirement: Branch outcomes carry trajectory-derived miss distances

Coupled reports SHALL expose the terminal truth and navigation states, and branch scoring SHALL
compute `miss_distance` as the distance from the branch's terminal position to the configured aim
point — replacing the t²-law proxy, which SHALL remain only as a printed cross-check. Distinct
bank-angle branches SHALL yield distinct, trajectory-derived miss distances.

#### Scenario: Bank branches disperse on real trajectories
- **WHEN** counterfactual bank-angle branches continue from the shared blackout-onset fork
- **THEN** each branch's report exposes its terminal states, the miss distances derive from those
  trajectories, and at least two branches differ in miss distance
