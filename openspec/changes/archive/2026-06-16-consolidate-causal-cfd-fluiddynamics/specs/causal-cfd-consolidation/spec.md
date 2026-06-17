## ADDED Requirements

### Requirement: Dedicated `deep_causality_cfd` crate
CFD code SHALL be consolidated into a dedicated `deep_causality_cfd` crate with `publish = false`,
built from the no-external-dependency line (external dependencies only where genuinely required, such
as file output in examples). The crate SHALL follow the structure: `src/{errors, extensions, traits,
types, solvers, theories}`, `tests/` mirroring `src/`, `benches/`, `examples/`, `validation/`, and
`docs/{prompts, openspecs}`. The fluid-dynamics theories and the DEC Navier–Stokes solver SHALL be
**moved out of** `deep_causality_physics` entirely (no published back-compat to preserve; downstream
importers updated). The CFD benches and the validation examples SHALL be migrated into the crate with
numerics preserved.

#### Scenario: Migrated validation reproduces reference results
- **WHEN** a migrated validation case (Taylor–Green, lid-driven cavity, graded MMS, cylinder wake/validation) is run in `deep_causality_cfd`
- **THEN** it reproduces the pre-migration reference result to the same tolerance

#### Scenario: The solver no longer lives in the physics crate
- **WHEN** `deep_causality_physics` is inspected after migration
- **THEN** the fluid-dynamics theories and the DEC NS solver are absent, and downstream importers depend on `deep_causality_cfd` instead

#### Scenario: The crate is not published
- **WHEN** the crate manifest is inspected
- **THEN** `publish = false` is set
