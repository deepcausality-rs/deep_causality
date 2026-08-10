# constant-and-test-traceability Specification

## Purpose
TBD - created by archiving change reconcile-cfd-docs-and-traceability. Update Purpose after archive.
## Requirements
### Requirement: A load-bearing constant carries a source, its units, and a papers entry

A load-bearing constant SHALL carry, at its definition, a source, its units, and — where a publication
fixes its value — a `papers/` entry with a full author-year citation. A constant is load-bearing when the
reported result depends materially on its value; such a constant SHALL NOT stand as a bare literal with no
provenance.

The mask-smoothing width `SMOOTH_CELLS = 2.0` moves the reported immersed-cylinder `C_d` by 6.1×
(`AUDIT-REPORT.md` §4b) and carries only "Mask smoothing width in cells." at both definitions
(the `SMOOTH_CELLS` constants in `verification/qtt_cylinder_verification/config.rs` and
`verification/qtt_park2t_blackout/config.rs`). The `qtt_park2t_blackout` `ETA = 0.016` (the `ETA`
constant in the same file) carries no source, while the cylinder site (`ETA` in
`verification/qtt_cylinder_verification/config.rs`) already carries a wall-error-target derivation from
`close-qtt-solver-envelope`. The Mach-1.05 shock floor (the `shock_floor` branch in
`src/types/flow/compressible_march_run.rs`) explains its branch but not the 0.05 buffer above
sonic. The `papers/` folder holds no penalization reference; Angot / Bruneau & Fabrie (1999) is cited in
harness text only (`render_ladders` in `verification/qtt_cylinder_verification/print_utils.rs`).

#### Scenario: A load-bearing constant states its source and units at the definition

- **WHEN** a reviewer reads a load-bearing constant's definition
- **THEN** its source, units, and — where a publication backs it — a `papers/` citation are present there

#### Scenario: The penalization method is cited from a papers entry

- **WHEN** the immersed-body penalization method is described in the harness
- **THEN** its reference (Angot / Bruneau & Fabrie 1999) is present in `papers/` and cited at the site, not
  named in text alone

#### Scenario: Every papers PDF is accounted for

- **WHEN** the `papers/` folder is inventoried
- **THEN** each PDF is cited by author-year from the code it supports, or is flagged for the owner — none
  is removed without owner approval

### Requirement: A load-bearing test references an independent truth

A load-bearing test SHALL check the shipped code path against a reference derived independently of that
path — an analytic solution, a published value, or a hand-derivation — not against a value pinned from the
code's own prior output, and not against a re-implementation of the code under test. The shipped path SHALL
be the path the test drives.

No test drives the shipped QTT convection path `rate_pair` (`src/solvers/qtt/incompressible_2d.rs:105`)
with a nonzero velocity: the two `scalar_rate` unit tests pass `u = v = 0`, and the full-solver
Taylor–Green test's convection is a pure gradient the projection annihilates, so a sign flip in the shipped
convection is invisible. The Taylor–Green verification harness checks convection with `u,v ≠ 0` but
re-assembles the operator from `gradient_x`/`gradient_y` (`verification/qtt_taylor_green_verification/main.rs:83-126`)
rather than calling `rate_pair`, so it gates a copy.

#### Scenario: The shipped convection path is exercised with a nonzero velocity

- **WHEN** the QTT convection path is tested
- **THEN** a test drives the shipped `rate_pair` with `u,v ≠ 0` against an analytic reference the
  projection does not annihilate, and the test fails under a sign flip in the shipped convection

#### Scenario: A gate drives the shipped path, not a re-implementation

- **WHEN** the Taylor–Green harness checks the convection operator
- **THEN** it routes the check through the shipped `rate_pair`, not a `gradient_x`/`gradient_y` re-assembly,
  and its reported convection error is unchanged by the re-route

