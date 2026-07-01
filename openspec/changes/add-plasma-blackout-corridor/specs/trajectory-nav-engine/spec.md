## ADDED Requirements

### Requirement: Trajectory + timing engine built once against the coupling interface

Stage 2 SHALL provide the onboard trajectory + navigation engine composed from the Stage-0 primitives: predict =
the KS conformal propagator with the between-step aero kick taken **from the `blackout-coupling-interface` force
channel**; correct = a **17-state tightly-coupled ESKF** (position 3, velocity 3, attitude-error 3, gyro bias 3,
accel bias 3, clock bias + drift 2) followed by the Sp(2,R)/KS constraint projection; and a **two-clock** carry
(the KS fictitious time `s` distinct from proper time `τ`). Because the aero force arrives through the interface,
there SHALL be **no mock/real split** in the engine — the Stage-0 stub and the Stage-1 marcher are interchangeable
behind the contract. The ESKF is example-level; the reusable math is the Stage-0 library.

#### Scenario: Coast exactness and dynamic timing
- **WHEN** the engine runs with a zero aero force and monopole gravity
- **THEN** the trajectory matches the analytic Kepler orbit to round-off, and the carried `dτ/dt = 1 + Φ/c² −
  v²/2c²` (from state) reproduces the GPS split (+45.7 / −7.2 / +38.6 µs/day) within the FS-3 tolerances

#### Scenario: Closed-loop navigation through a coupling-driven blackout
- **WHEN** the interface asserts the blackout flag and the ESKF runs predict-only on the IMU over the RAM-C
  trajectory (~7.65 km/s; 61/71/81 km; ~30 s window)
- **THEN** the estimate tracks truth pre-blackout, the INS-only error grows at the expected `t²`/`t³` rate during
  denial (bounded by the through-plasma optical aid), and re-converges on reacquisition with the carried clock
  matching the FS-3 anchor

#### Scenario: Interface swap is transparent
- **WHEN** the Stage-1 marcher adapter replaces the Stage-0 stub behind the interface
- **THEN** the engine produces the same results modulo the (now real) aero force, with no engine code change

### Requirement: Encke↔Cowell regime-switched integrator

The engine SHALL include the `select_integrator` regime switch (the `grmhd/select_metric` pattern): compute
`ε = a_aero / a_grav` from state (the aero magnitude from the interface, `a_grav = GM/r²`), compare to a config
threshold, and select the perturbed-conformal (matrix-exponential, branch-cheap) integrator when `ε < ε_switch`
and direct integration when `ε ≥ ε_switch`, with hysteresis to prevent chatter. This is part of Stage 2, not a
later phase; it consumes stub ε until Stage 1 supplies real ε.

#### Scenario: Switch agrees in the overlap band and is hysteresis-stable
- **WHEN** `ε` is swept across `ε_switch`
- **THEN** the perturbed-conformal and direct integrators agree within tolerance in an overlap band, the switch
  does not chatter near the threshold, and the matrix-exponential branch-cost advantage is measured in the
  perturbative regime
