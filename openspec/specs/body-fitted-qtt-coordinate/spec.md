# body-fitted-qtt-coordinate Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: Shock-aligned / body-fitted curvilinear coordinate

`deep_causality_cfd` SHALL provide a smooth curvilinear coordinate map that aligns the body surface and the
bow-shock surface to **coordinate surfaces** (e.g. a wall-normal / shock-normal coordinate), carried as a
**low-rank Jacobian** in tensor-train form. The map SHALL be analytic and invertible over the computational
domain, and the metric/Jacobian SHALL be computed from the geometry (dynamic-by-construction — no hardcoded
metric components).

Invertibility SHALL be **enforced**, not asserted. A map whose Jacobian determinant changes sign or
approaches zero over the computational domain is not invertible there, and SHALL be rejected at
construction rather than producing a metric the caller cannot distinguish from a valid one. Every
division by the Jacobian determinant SHALL be guarded, so a degenerate geometry cannot yield
`inf`/`NaN` or unbounded-magnitude metric entries behind a successful construction.

The determinant floor SHALL be relative to the geometric scale rather than an absolute constant, since
the determinant is an area ratio and an absolute bound would mean different quantities at different
geometries. The validity scan SHALL cover the **closed** computational domain, not only the interior
points the metric fields are sampled at: a map may degenerate exactly on the domain boundary, and a
scan of the sampled lattice alone approaches that degeneracy only as `~1/n`.

A documented validity property SHALL be checked in code, or not documented. An appeal to a
measurement — a study reporting the determinant margin for one geometry — is evidence for that
geometry and SHALL NOT be presented as a guarantee over the accepted input range. Where a study gates
on such a property, it SHALL obtain the quantity from the shipped constructor rather than
recomputing the determinant algebra alongside it, so the gate measures the map the crate builds.

#### Scenario: A curved shock becomes axis-aligned and low-rank
- **WHEN** a curved (spherical-shell) shock field is expressed in the body-fitted coordinate where it is a
  function of the wall-/shock-normal coordinate only, and QTT-encoded
- **THEN** its bond dimension is `O(10)` and independent of resolution — matching the measured
  `qtt_rank_study` / `qtt_rank_3d` result (vs `χ ~ √side` in Cartesian)

#### Scenario: Jacobian is low-rank
- **WHEN** the coordinate Jacobian is QTT-encoded
- **THEN** its bond dimension is bounded and small (the smooth stretch does not itself inflate rank)

#### Scenario: A folded map is rejected at construction
- **WHEN** a map is constructed whose Jacobian determinant changes sign over the computational domain
- **THEN** construction fails with an error naming the violation, rather than returning a map whose
  inverse metric is unbounded

#### Scenario: A near-singular map is rejected rather than silently amplified
- **WHEN** `|det J|` falls below a documented floor relative to the geometric scale
- **THEN** construction fails, rather than producing metric entries of ~1e15 magnitude that a caller
  cannot distinguish from a valid map

#### Scenario: The documented guarantee matches what is enforced
- **WHEN** the map's documentation states a validity property
- **THEN** that property is checked in code, and any appeal to a measurement is identified as evidence
  for a specific geometry rather than a guarantee over the accepted input range

#### Scenario: A validity gate measures the shipped map
- **WHEN** a study gates on the map's determinant margin
- **THEN** it obtains that margin from the shipped constructor, so the gate cannot pass on a copy of
  the algebra that has drifted from the code under test

### Requirement: Chain-rule operator transformation

The finite-difference operators (`qtt-codec-3d`) SHALL be transformable to the body-fitted coordinate by the
chain rule using the low-rank Jacobian, so that physical derivatives are computed on the aligned lattice. The
transformed operators SHALL recover the physical gradient/divergence on a smooth field to discretization
order.

#### Scenario: Physical gradient via the transformed operator
- **WHEN** the chain-rule-transformed gradient is applied in the body-fitted coordinate to a field with a
  known physical derivative
- **THEN** the decoded result matches the analytic physical `∇` field to the scheme's order

### Requirement: Rank-lever acceptance gate

The body-fitted coordinate SHALL be demonstrated to hold a representative reentry shock at `χ ~ O(10)`
independent of resolution, where the same shock captured on a Cartesian grid grows as `χ ~ √side`. This is the
measured precondition that makes the 3-D marcher tractable.

#### Scenario: Resolution-independent rank in the fitted coordinate
- **WHEN** the shock rank is measured in the fitted coordinate across a resolution sweep
- **THEN** the bond dimension stays approximately constant (does not grow with side), unlike the Cartesian
  capture

