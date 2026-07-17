## ADDED Requirements

### Requirement: The imprinted layer is gated against Jarvinen–Adams

A self-verifying verification target, `verification/srp_drag_decrement/`, SHALL march the
compressible layer with the plume imprint at the correlation's anchor condition (freestream
M∞ = 2.0, γ = 1.4, central-nozzle geometry) across a thrust-coefficient sweep spanning the
collapse band and beyond (C_T ≈ 0.25 to 4), contract the preserved-drag fraction per point,
and print it beside the digitized correlation (`srp_preserved_drag_fraction`) at every point.
The binary MUST follow the family pattern: PASS/FAIL per gate printed, nonzero exit on any
FAIL, and its representative output committed beside it. Geometry differences between the
harness and the wind-tunnel configuration (2-D plane vs axisymmetric, smoothing skirt,
blockage, the single marched γ) MUST be disclosed in the printed output, the way the
incompressible immersed validation discloses periodic blockage.

#### Scenario: Every sweep point is compared against the correlation

- **WHEN** the verification sweeps C_T and contracts the preserved-drag fraction at each point
- **THEN** the output tabulates the harness fraction beside the correlation's value per point,
  with the plume geometry and the peak bond, and the exit code reflects the regression gates

#### Scenario: A regression fails loudly

- **WHEN** any regression gate on the measured behavior breaks on a later run
- **THEN** the binary prints the failing gate with both values and exits nonzero

### Requirement: The collapse and the sign-flip band are measured signatures

The verification SHALL measure the two structural signatures on every run and print them beside
the correlation's values: (a) **central-nozzle drag collapse** — whether the contracted
preserved-drag fraction falls below 0.10 by C_T ≈ 1 (the transition constant
`JARVINEN_ADAMS_TRANSITION_CT_M2`); (b) **the sign-flip band** — whether the total axial force
(`C_T + f(C_T)·C_A0`) is non-monotone in C_T. When a signature is present it SHALL be gated;
when the first measured run finds a signature absent — the recorded outcome of 2026-07-17: the
static-obstruction imprint shields monotonically like a drag-reduction spike (fraction
1.21 → 0.65 across the sweep) but does not collapse (0.895 at C_T ≈ 1) and produces no dip —
the binary MUST report the miss prominently as the **amber de-risk finding** (naming
`derisk-verdict.md` as the authority) and MUST gate the *measured* structure instead
(monotone non-increasing fraction; a pinned shielding-depth ceiling at the sweep top), so the
run never silently passes and never permanently fails on physics the harness cannot produce.

#### Scenario: An absent signature is a reported finding, not a silent pass

- **WHEN** the sweep completes without the collapse or the dip
- **THEN** the output carries a FINDING block naming the miss, the measured values, and the
  verdict note, and the exit code reflects only the regression gates on the measured behavior

#### Scenario: The measured shielding structure is the regression net

- **WHEN** a later run's preserved-drag fraction is non-monotone in C_T, or its sweep-top
  fraction exceeds the pinned ceiling
- **THEN** the binary fails with the offending gate named

### Requirement: Bands are earned, then regressed

The verification's absolute bands SHALL be pinned from the first measured run (recorded as
constants in the binary with the pin's provenance in comments and in the verdict note), not
tuned by anticipation; subsequent runs regress against the pinned bands. A band MUST NOT be
re-pinned without recording the re-pin and its reason in the verdict note.

#### Scenario: First run pins, later runs regress

- **WHEN** the first measured run completes and its bands are recorded
- **THEN** a later run producing values outside those bands fails the gate rather than silently
  re-pinning
