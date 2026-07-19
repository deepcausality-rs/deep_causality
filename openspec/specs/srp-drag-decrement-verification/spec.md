# srp-drag-decrement-verification Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-de-risk. Update Purpose after archive.
## Requirements
### Requirement: The coupled layer is gated against Jarvinen–Adams

A self-verifying measurement target SHALL march the compressible layer with the plume coupling
at the correlation's anchor condition (freestream M∞ = 2.0, γ = 1.4, central-nozzle geometry)
across a thrust-coefficient sweep spanning the collapse band and beyond, contract the
preserved-drag fraction per point, and print it beside the digitized correlation
(`srp_preserved_drag_fraction`) at every point. The binary MUST follow the family pattern:
PASS/FAIL per gate printed, nonzero exit on any FAIL, and its representative output committed
beside it. Geometry differences between the harness and the wind-tunnel configuration (2-D
plane vs axisymmetric, smoothing skirt, blockage, the single marched γ) MUST be disclosed in
the printed output, the way the incompressible immersed validation discloses periodic blockage.

The target is `studies/srp_momentum_jet/` (momentum-carrying jet: a nozzle-exit patch pinned
to a supersonic exit state through the `ForcingRegion` seam, the plume formed in the marched
field, strip force tail-averaged over a settled window). It supersedes the original
`verification/srp_drag_decrement/` (whole-envelope pinned-state imprint, terminal-snapshot
read), which measured the weaker model class on the same harness and is retained under
`deep_causality_cfd/reverted/` — detached from Cargo, its first-run output preserved as the
provenance of the amber call. The supersession and its reasons are recorded in
`derisk-verdict.md`.

#### Scenario: Every sweep point is compared against the correlation

- **WHEN** the measurement sweeps C_T and contracts the preserved-drag fraction at each point
- **THEN** the output tabulates the harness fraction beside the correlation's value per point,
  with the realized throttle state, the tail-stationarity witness, the interface position, and
  the peak bond, and the exit code reflects the regression gates

#### Scenario: A regression fails loudly

- **WHEN** any regression gate on the measured behavior breaks on a later run
- **THEN** the binary prints the failing gate with both values and exits nonzero

### Requirement: The collapse and the sign-flip band are measured signatures

The measurement SHALL evaluate the two structural signatures on every run and print them
beside the correlation's values: (a) **central-nozzle drag collapse** — whether the contracted
preserved-drag fraction falls below 0.10 by C_T ≈ 1 (the transition constant
`JARVINEN_ADAMS_TRANSITION_CT_M2`); (b) **the sign-flip band** — whether the total axial force
(`C_T + f(C_T)·C_A0`) is non-monotone in C_T. When a signature is present it SHALL be gated;
when a measured run finds a signature absent, the binary MUST report the miss prominently as
the recorded de-risk finding (naming `derisk-verdict.md` as the authority) and MUST gate the
*measured* structure instead, so the run never silently passes and never permanently fails on
physics the harness cannot produce.

The recorded outcomes of 2026-07-17, both on the same harness: the static-obstruction imprint
shields monotonically like a drag-reduction spike (fraction 1.21 → 0.65) with no collapse and
no dip; the momentum-carrying jet produces monotone drag *augmentation* (annulus fraction
1.03 → 3.61 across C_T 0.25 → 8) with the stagnation interface frozen at the face — the
dissipation floor (ν = ½·s_ref·Δx) prevents jet penetration, so neither coupling model can
host the collapse mechanism at this fidelity. The gated measured structure is therefore the
augmentation (monotone non-decreasing annulus fraction; a pinned sweep-top band; the
frozen-interface witness), and the collapse/dip comparison is the reported finding.

#### Scenario: An absent signature is a reported finding, not a silent pass

- **WHEN** the sweep completes without the collapse or the dip
- **THEN** the output carries a FINDING block naming the miss, the measured values, and the
  verdict note, and the exit code reflects only the regression gates on the measured behavior

#### Scenario: The measured structure is the regression net

- **WHEN** a later default-configuration run breaks the monotone-augmentation structure, the
  pinned sweep-top band, or the frozen-interface band
- **THEN** the binary fails with the offending gate named

### Requirement: Bands are earned, then regressed

The measurement's absolute bands SHALL be pinned from the first measured run (recorded as
constants in the binary with the pin's provenance in comments and in the verdict note), not
tuned by anticipation; subsequent runs regress against the pinned bands. A band MUST NOT be
re-pinned without recording the re-pin and its reason in the verdict note. Environment-dialed
companion runs (bond cap, grid level, sweep subset) measure and disclose; only the default
configuration regresses.

#### Scenario: First run pins, later runs regress

- **WHEN** the first measured run completes and its bands are recorded
- **THEN** a later run producing values outside those bands fails the gate rather than silently
  re-pinning

