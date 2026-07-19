# srp-derisk-verdict Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-de-risk. Update Purpose after archive.
## Requirements
### Requirement: A checked-in go/no-go verdict synthesizes the measurements

The change SHALL record its outcome in a checked-in verdict note,
`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, authored from the measured runs and
carrying: the three risk answers with their measured numbers (imprint fidelity vs the
correlation, fork economics of the plume-coupled state, rank behavior Cartesian vs blend
metric), the pinned bands and where they were pinned, and a single **green / amber / red**
call. The verdict MUST state its downstream consequences explicitly: **green** — M3 builds
`PlumeObstruction` at coupling depth A (plume imprinted on the marched layer) and M5's
centerpiece is the state-fork counterfactual; **amber/red** — M3 carries the A0 force-channel
depth, M5's centerpiece pivots to the parameter-fork design, and the state-fork result is
recorded as a measured limitation. Later milestones (M3, M5) MUST cite the verdict rather than
re-litigating the measurement.

#### Scenario: The verdict is the single authority downstream

- **WHEN** M3 or M5 is proposed after this change archives
- **THEN** its design cites `derisk-verdict.md` for the coupling depth and centerpiece choice,
  and the verdict note contains the measured numbers, the bands, and the explicit call

#### Scenario: An amber or red outcome is a result, not a failure

- **WHEN** the measurements come back degraded (imprint infidelity, fork degradation, or rank
  blowup in both coordinates)
- **THEN** the verdict records the finding with its numbers and pivots the downstream design
  per the consequence table — the roadmap proceeds without a rewrite

