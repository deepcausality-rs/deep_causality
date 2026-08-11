## MODIFIED Requirements

### Requirement: Configuration is separated from workflow composition
The crate SHALL separate **configuration** (the "what") from **workflow composition** (the "how"),
mirroring the Discovery `CdlConfigBuilder` → `CdlBuilder` split. A single `CfdConfigBuilder` entry
SHALL start each owned, validated configuration — the solver config (`dec_ns`) and every marching /
verification case container (`march`, `qtt_march`, `compressible_march`, `duct`, `verify`,
`uncertain_march`) — and the `CfdFlow` facade SHALL compose those configs onto a caller-owned
geometry and run them. The entry set SHALL be complete: no configuration family SHALL be reachable
through a public constructor outside `CfdConfigBuilder`. Configuration objects SHALL hold no geometry
borrow; the geometry SHALL be lent to the run via `.on(&manifold)` (the B1 borrow model) and SHALL NOT
escape the run.

#### Scenario: A marching case is configured, then composed and run
- **WHEN** a marching case is built with `CfdConfigBuilder::march(...)` and run with `CfdFlow::march(&config).on(&manifold)`
- **THEN** the configuration carries no manifold borrow, the geometry is lent for the run only, and a `Report` is returned

#### Scenario: Every configuration family starts at the one entry
- **WHEN** any of the seven configuration families is built
- **THEN** it is started by a `CfdConfigBuilder` method, and the family's own builder constructor is
  crate-private

## ADDED Requirements

### Requirement: The seed catalogue covers symmetry breaking

`Seed` SHALL provide a uniform free-stream variant carrying a superposed transverse perturbation, so
a case whose discretisation, geometry and inflow are all symmetric can be tipped off the symmetric
branch without hand-building its own vertex field.

The variant SHALL carry the perturbation's offset from the body, its width, and its amplitude
relative to the free-stream speed. `Seed` SHALL remain `Copy` (no boxed closures), so a case stays
cheap to clone for a counterfactual run. The perturbed field SHALL be made divergence-free by the
same seed projection every other variant uses.

#### Scenario: A symmetric wake case sheds

- **WHEN** a cylinder case above the shedding Reynolds number is seeded with the perturbed variant
- **THEN** the march develops the von Kármán street, and the wake probe carries a periodic signal
  from which a Strouhal number is read

#### Scenario: The perturbed seed is divergence-free

- **WHEN** the perturbed seed is applied
- **THEN** the resulting field passes the solver's divergence check to the same tolerance as the
  unperturbed uniform seed
