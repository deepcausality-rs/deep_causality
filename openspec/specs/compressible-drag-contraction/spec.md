# compressible-drag-contraction Specification

## Purpose
TBD - created by archiving change plasma-retropulsion-de-risk. Update Purpose after archive.
## Requirements
### Requirement: Axial force contracted from the evolved pressure field

The `solvers/qtt` module SHALL provide a compressible axial-force observable that integrates
the evolved pressure over a forebody-strip mask — pressure recovered from the conserved
components (the ideal-gas closure), contracted against the strip mask via the tensor-train
inner product and the cell volume, with no surface reconstruction — yielding the axial-force
coefficient the Jarvinen–Adams dataset measured. It SHALL be a compressible sibling of the
incompressible penalization-force contraction, not a modification of it: the integrand is the
field's pressure, not the forcing deficit, because the quantity of interest is the *preserved
aerodynamic drag*, not the imposed force.

#### Scenario: Axial force is a single strip contraction

- **WHEN** the axial-force coefficient is requested for a marched compressible state
- **THEN** it is computed as the forebody-strip pressure contraction of the evolved field, with
  no cut-cell surface or boundary-fiber reconstruction

### Requirement: Preserved-drag fraction against an unpowered baseline

The observable family SHALL expose the preserved-drag fraction as the ratio of the powered
run's contracted forebody axial force to the unpowered baseline's (same configuration, no
forcing region) — the dimensionless quantity the Jarvinen–Adams correlation tabulates. The
baseline MUST come from a run of the same grid, schedule, and truncation policy, so the ratio
cancels the harness's common geometry biases.

#### Scenario: The fraction is a same-configuration ratio

- **WHEN** a powered (imprinted) run and an unpowered baseline run of the same configuration
  are contracted
- **THEN** the preserved-drag fraction is their forebody axial-force ratio, and an unpowered
  run's own fraction evaluates to one within contraction tolerance

