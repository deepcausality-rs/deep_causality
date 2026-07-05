# duct-march Specification

## Purpose
TBD - created by archiving change add-cfd-study-dsl-and-examples. Update Purpose after archive.
## Requirements
### Requirement: A 1-D compressible duct path in the flow DSL

`deep_causality_cfd` SHALL provide an owned `DuctConfig` (area profile as a table or analytic
variant, inlet stagnation state, back pressure, grid resolution, stop condition) and a
`CfdFlow::duct_march` entry lowering it onto the existing 1-D compressible Euler solver. The
runner SHALL march to a quasi-steady state under the stop condition and return the standard
`Report` carrying the series `"x"`, `"mach_profile"`, and `"pressure_profile"` and the
scalars `"shock_position"` and `"thrust_coefficient"`. A stop condition that expires before
the residual settles SHALL surface as an error, never as a silently unconverged report.

#### Scenario: A choked nozzle reports its shock and thrust

- **WHEN** a converging-diverging duct is marched at a back pressure that places a normal
  shock in the diverging section
- **THEN** the report's Mach profile passes through unity at the throat, the shock position
  falls where the pressure profile steepens, and the thrust coefficient is finite and
  positive

#### Scenario: Convergence failure is loud

- **WHEN** the stop condition expires with the residual above its gate
- **THEN** `duct_march` returns an error naming the residual and the step budget

### Requirement: Cited isentropic duct relations gate the duct march

`deep_causality_physics` SHALL provide the area-Mach relation and the isentropic pressure,
temperature, and density ratios as pure pointwise kernels with full citations and the source
reference in `papers/`, following the shipped kernel contract. Normal-shock relations SHALL
be reused from the existing Rankine-Hugoniot machinery, not duplicated. The duct march's
verification SHALL gate the computed profiles against these closed forms.

#### Scenario: The computed profile matches the closed form where the flow is smooth

- **WHEN** the duct march runs shock-free (back pressure above the first critical)
- **THEN** the computed Mach profile matches the area-Mach relation within the gated band at
  every sampled station

