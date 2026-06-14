## ADDED Requirements

### Requirement: 3D cylinder wake validation
The change SHALL validate the cut-cell solver against flow around a 3D cylinder over
Re 100–3900, comparing the Strouhal number and drag coefficient against Lehmkuhl et al.
(2013) and the Williamson lineage, and documenting the resolution at which the agreement
holds. The heavy march SHALL live in an example
(`examples/avionics_examples/dec_cylinder_wake`) per the tests-fast / examples-verify split,
streaming CSV per step so an aborted run keeps its partial curve.

#### Scenario: Shedding frequency matches the reference at Re 100
- **WHEN** the cylinder example is run at Re 100 to a developed shedding state
- **THEN** the measured Strouhal number matches the published value within the documented tolerance

#### Scenario: Drag matches the reference across the range
- **WHEN** the cylinder example is run at representative Reynolds numbers up to 3900
- **THEN** the drag coefficient and Strouhal number agree with Lehmkuhl et al. (2013) within the documented tolerance at the stated resolution

### Requirement: Cheap CI regressions for cut geometry and stability
The CI suite SHALL carry fast regressions that do not run the heavy march: the geometric
exactness of cube ⋂ analytic-primitive cuts, the axis-aligned-cut versus Stage-3-wall-clip
consistency gate, and the small-cut-cell stability smoke test. These SHALL run within the
project's fast-test budget.

#### Scenario: Fast regressions guard the substrate in CI
- **WHEN** the CI test suite runs
- **THEN** the cut-geometry exactness, axis-aligned consistency, and small-cell stability regressions pass quickly without executing the cylinder-wake march
