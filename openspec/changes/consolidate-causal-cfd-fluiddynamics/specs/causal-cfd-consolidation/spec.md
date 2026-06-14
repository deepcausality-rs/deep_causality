## ADDED Requirements

### Requirement: Dedicated `causal_cfd` crate
CFD code SHALL be consolidated into a dedicated `causal_cfd` crate with `publish = false`, built
from the no-external-dependency line (external dependencies only where genuinely required, such as
file output in examples). The crate SHALL follow the structure: `src/{errors, extensions, traits,
types, solvers, theories}`, `tests/` mirroring `src/`, `benches/`, `examples/`, `validation/`, and
`docs/{prompts, openspecs}`. The fluid-dynamics theories, the DEC Navier–Stokes solver, the CFD
benches, and the validation examples SHALL be migrated into it with numerics preserved.

#### Scenario: Migrated validation reproduces reference results
- **WHEN** a migrated validation case (Taylor–Green, lid-driven cavity, graded MMS, cut-cell cylinder) is run in `causal_cfd`
- **THEN** it reproduces the pre-migration reference result to the same tolerance

#### Scenario: The crate is not published
- **WHEN** the crate manifest is inspected
- **THEN** `publish = false` is set
