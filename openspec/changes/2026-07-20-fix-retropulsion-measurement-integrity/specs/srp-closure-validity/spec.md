## MODIFIED Requirements

### Requirement: The thrust coefficient is normalized against one dynamic pressure

`PlumeObstruction` SHALL read the freestream dynamic pressure from the sensed field scalar each step,
using the same field name the safety gate's burn sensing reads, so that the drag closure and the
envelope's dynamic thrust-coefficient cap describe one physical state. The construction-time `q_inf`
parameter is removed, and `BURN_CORRIDOR_Q_INF` is deleted.

#### Scenario: Closure and envelope agree on C_T within a step

- **WHEN** the plume stage and the safety gate both evaluate the thrust coefficient on the same step
- **THEN** both use the same sensed dynamic pressure and produce the same `C_T` for the same thrust

#### Scenario: An absent dynamic-pressure sensor fails the step

- **WHEN** the plume stage runs under an active throttle on a field carrying no dynamic-pressure
  scalar, or a non-positive one
- **THEN** the stage returns an error rather than substituting a default, because a silent fallback
  reintroduces a second normalization

### Requirement: The A0 correlation applies only inside its validated Mach band

`PlumeObstruction` SHALL stand down outside the Jarvinen–Adams Mach band read from the sensed flight
Mach: it applies no drag decrement, publishes no plume geometry, and records one provenance entry per
crossing. The correlation's mechanism is bow-shock displacement, and there is no bow shock to displace
below the dataset's Mach floor.

#### Scenario: The subsonic terminal leg carries no A0 decrement

- **WHEN** the terminal leg burns at a positive throttle below the band's Mach floor
- **THEN** the aerodynamic drag on the force channel is unmodified by the plume stage, and provenance
  records that the closure stood down

#### Scenario: Re-entering the band resumes the closure

- **WHEN** the sensed flight Mach crosses back inside the band under an active throttle
- **THEN** the decrement resumes and a second provenance entry records the crossing

### Requirement: The plume geometry is built from the sensed freestream

The Cordell–Braun plume boundary SHALL take its freestream Mach number and static pressure from the
flown state rather than from constants fixed at construction, so the kernel's own validity envelope
tests the flight. `PLUME_MACH_INF` and `PLUME_P_INF` are deleted.

#### Scenario: The kernel's envelope check tests the flight

- **WHEN** the flown freestream Mach leaves the kernel's documented envelope
- **THEN** the kernel's rejection surfaces as a step error rather than being masked by a constant
  inside the envelope

#### Scenario: Geometry tracks the descent

- **WHEN** the vehicle descends through two altitudes with materially different static pressure at one
  throttle
- **THEN** the published plume radius and penetration differ between the two steps
