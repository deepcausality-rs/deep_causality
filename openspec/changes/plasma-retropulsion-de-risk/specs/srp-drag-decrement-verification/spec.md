## ADDED Requirements

### Requirement: The imprinted layer is gated against Jarvinen–Adams

A self-verifying verification target, `verification/srp_drag_decrement/`, SHALL march the
compressible layer with the plume imprint at the correlation's anchor condition (freestream
M∞ = 2.0, γ = 1.4, central-nozzle geometry) across a thrust-coefficient sweep spanning the
collapse and beyond (C_T from 0 to ≈ 4), contract the preserved-drag fraction per point, and
gate it against the digitized correlation (`srp_preserved_drag_fraction`) within a pinned band.
The binary MUST follow the family pattern: PASS/FAIL per gate printed, nonzero exit on any
FAIL, and its representative output committed beside it. Geometry differences between the
harness and the wind-tunnel configuration (2-D plane vs axisymmetric, smoothing skirt,
blockage) MUST be disclosed in the printed output, the way the incompressible immersed
validation discloses periodic blockage.

#### Scenario: The sweep tracks the correlation

- **WHEN** the verification sweeps C_T and contracts the preserved-drag fraction at each point
- **THEN** every point lies within the pinned band of the correlation's value, and the run
  exits zero

#### Scenario: A regression fails loudly

- **WHEN** any swept point leaves its pinned band
- **THEN** the binary prints the failing gate with both values and exits nonzero

### Requirement: The collapse and the sign-flip band are structural gates

The verification SHALL gate the two structural signatures independently of the absolute bands:
(a) **central-nozzle drag collapse** — the contracted preserved-drag fraction falls below 0.10
by C_T ≈ 1 (the transition constant `JARVINEN_ADAMS_TRANSITION_CT_M2`); (b) **the sign-flip
band** — the total axial force (thrust plus contracted preserved drag, composed as the
correlation's `C_T + f(C_T)·C_A0`) is non-monotone in C_T, dipping below the unpowered
baseline in the low-C_T band, with the dip's C_T location within tolerance of the
correlation's predicted location. These are the physics the M5 counterfactual study must later
find from trajectory outcomes; the verification pins where they live on the imprinted layer.

#### Scenario: Lighting the engine gently loses deceleration

- **WHEN** the total axial force is evaluated across the low-C_T band of the sweep
- **THEN** it dips below the unpowered value before recovering as thrust dominates, and the
  minimum's C_T location agrees with the correlation within the pinned tolerance

#### Scenario: The collapse is present

- **WHEN** the contracted preserved-drag fraction is read at the transition thrust coefficient
- **THEN** it is below 0.10 — the central-nozzle collapse, corroborating the survey's
  "~10% of the no-jet value"

### Requirement: Bands are earned, then regressed

The verification's absolute bands SHALL be pinned from the first measured run (recorded as
constants in the binary with the pin's provenance in comments and in the verdict note), not
tuned by anticipation; subsequent runs regress against the pinned bands. A band MUST NOT be
re-pinned without recording the re-pin and its reason in the verdict note.

#### Scenario: First run pins, later runs regress

- **WHEN** the first measured run completes and its bands are recorded
- **THEN** a later run producing values outside those bands fails the gate rather than silently
  re-pinning
