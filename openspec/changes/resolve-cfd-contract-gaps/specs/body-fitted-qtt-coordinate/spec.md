## MODIFIED Requirements

### Requirement: Shock-aligned / body-fitted curvilinear coordinate

`deep_causality_cfd` SHALL provide a smooth curvilinear coordinate map that aligns the body surface and the
bow-shock surface to **coordinate surfaces** (e.g. a wall-normal / shock-normal coordinate), carried as a
**low-rank Jacobian** in tensor-train form. The map SHALL be analytic and invertible over the computational
domain, and the metric/Jacobian SHALL be computed from the geometry (dynamic-by-construction — no hardcoded
metric components).

Invertibility SHALL be **enforced**, not asserted. A map whose Jacobian determinant changes sign or
approaches zero over the sampled domain is not invertible there, and SHALL be rejected at construction
rather than producing a metric the caller cannot distinguish from a valid one. Every division by the
Jacobian determinant SHALL be guarded, so a degenerate geometry cannot yield `inf`/`NaN` or
unbounded-magnitude metric entries behind a successful construction.

`BlendedMap`'s documentation currently claims the constructor "rejects a fold" and that one-signed
`det J_λ` holds "by construction"; no such check exists in `BlendedMap::new`, and the justification
offered is the `qtt_blend_metric` **measurement** for one specific geometry rather than an argument
covering the inputs the constructor accepts. All four inverse-metric components and the volume factor
divide by `det_at(ξ, η)` unguarded. Either the guarantee is enforced or it is withdrawn; a documented
guarantee that nothing checks is the defect.

#### Scenario: A curved shock becomes axis-aligned and low-rank
- **WHEN** a curved (spherical-shell) shock field is expressed in the body-fitted coordinate where it is a
  function of the wall-/shock-normal coordinate only, and QTT-encoded
- **THEN** its bond dimension is `O(10)` and independent of resolution — matching the measured
  `qtt_rank_study` / `qtt_rank_3d` result (vs `χ ~ √side` in Cartesian)

#### Scenario: Jacobian is low-rank
- **WHEN** the coordinate Jacobian is QTT-encoded
- **THEN** its bond dimension is bounded and small (the smooth stretch does not itself inflate rank)

#### Scenario: A folded map is rejected at construction
- **WHEN** a map is constructed whose Jacobian determinant changes sign over the sampled domain
- **THEN** construction fails with an error naming the violation, rather than returning a map whose
  inverse metric is unbounded

#### Scenario: A near-singular map is rejected rather than silently amplified
- **WHEN** `|det J|` falls below a documented floor relative to the geometric scale
- **THEN** construction fails, rather than producing metric entries of ~1e15 magnitude that a caller
  cannot distinguish from a valid map

#### Scenario: The documented guarantee matches what is enforced
- **WHEN** the map's documentation states a validity property
- **THEN** that property is checked in code, or the documentation does not state it — and any appeal to
  a measurement is identified as evidence for a specific geometry rather than a guarantee over the
  accepted input range
