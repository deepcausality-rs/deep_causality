## ADDED Requirements

### Requirement: Cylinder-wake validation harness and composition correctness
The change SHALL provide a cut-cell cylinder validation harness as an example
(`examples/avionics_examples/dec_cylinder_validation`: west `Inflow` / east `Outflow` / far-field
`SlipWall` top-bottom / immersed cut cylinder, composed via `with_zones`), per the tests-fast /
examples-verify split, streaming CSV per step so an aborted run keeps its partial curve. The gate for
this Stage-4 change is the **harness plus composition correctness**: the composed primitive stack
SHALL march stably and stay interior-divergence-free to the projection tolerance at every resolution.

#### Scenario: The composed stack marches divergence-free
- **WHEN** the cylinder harness is run at `Re = 100`
- **THEN** the march is stable and the interior divergence residual stays at the projection's solve
  tolerance every step (the global residual is just the open-boundary inlet flux)

### Requirement: 2D Re=100 shedding validation
At `Re = 100` the harness SHALL develop a von-Kármán street (with a symmetry-breaking trigger) and
recover a Strouhal number within a few percent of the Williamson reference (`St ≈ 0.164`), and a drag
coefficient that moves toward the reference (`C_d ≈ 1.24`, pressure + friction). The high-Re 3D
Re-ladder (Re ≈ 200–300 transition, Re ≈ 3900 DNS) versus Lehmkuhl et al. (2013) is **compute-bound
and deferred** to a follow-up validation change.

#### Scenario: Shedding frequency matches the reference at Re 100
- **WHEN** the cylinder harness is run at `Re = 100` to a developed shedding state
- **THEN** the measured Strouhal number is within a few percent of `0.164`
- **AND** the aperture-resolved body sheds at a coarser grid (~16 cells/D) than the staircase body
  (which sheds only marginally at ~24 cells/D) — the `aperture-resolved-noslip` capability

### Requirement: Cheap CI regressions for cut geometry and stability
The CI suite SHALL carry fast regressions that do not run the heavy march: the geometric
exactness of cube ⋂ analytic-primitive cuts, the axis-aligned-cut versus Stage-3-wall-clip
consistency gate, and the small-cut-cell stability smoke test. These SHALL run within the
project's fast-test budget.

#### Scenario: Fast regressions guard the substrate in CI
- **WHEN** the CI test suite runs
- **THEN** the cut-geometry exactness, axis-aligned consistency, and small-cell stability regressions
  pass quickly without executing the cylinder-wake march
