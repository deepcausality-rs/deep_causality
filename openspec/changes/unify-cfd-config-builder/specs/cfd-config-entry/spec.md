## ADDED Requirements

### Requirement: One public entry per configuration family

`CfdConfigBuilder` SHALL be the only public entry to every owned CFD configuration, and SHALL expose
exactly one method per configuration family:

| Method | Starts | Name |
|---|---|---|
| `dec_ns()` | the DEC incompressible solver config | — |
| `march::<D, R>(name)` | the DEC marching case | required |
| `qtt_march::<R>(name)` | the QTT 2-D incompressible marching case | required |
| `compressible_march::<R>(name)` | the compressible coupled marching case | required |
| `duct::<R>(name)` | the quasi-1-D duct case | required |
| `verify::<R, M>(name, manufactured)` | the MMS-verification config | required |
| `uncertain_march::<R>(name)` | the sensor-fed uncertain-inflow case (`std`) | required |

Every case-family entry SHALL take the case name as its first argument, and the name SHALL be
required rather than defaulted, so a study sweep cannot produce two reports that cannot be told
apart. The solver-config entry (`dec_ns`) takes no name, because it configures a solver rather than
a case.

Each entry SHALL return a fluent builder whose terminal `build()` validates and returns
`Result<_, PhysicsError>`, and each builder's constructor SHALL be crate-private.

#### Scenario: Every family is reachable from one entry

- **WHEN** a caller configures any of the seven families
- **THEN** the configuration starts at a `CfdConfigBuilder` method, and no other public path to that
  configuration's builder exists

#### Scenario: A case carries the name it was given

- **WHEN** a case configuration is built through its entry and run
- **THEN** the returned `Report` is labelled with the name supplied at the entry

### Requirement: No configuration type exposes a public bypass constructor

A configuration type SHALL NOT expose a public constructor that bypasses its `CfdConfigBuilder`
entry. Specifically, the builder constructors (`new`), their `Default` implementations, and the
owned configs' field constructors SHALL be crate-private, and a type whose only public purpose was
to host such a constructor SHALL NOT be exported.

Validation SHALL live in the builder's `build()`, so the owned configuration is unconstructible in
an invalid state from outside the crate.

#### Scenario: A bypass constructor is not reachable

- **WHEN** a consumer outside the crate attempts to construct a configuration builder directly, or
  through `Default`
- **THEN** the code does not compile, because the constructor and the `Default` implementation are
  crate-private

#### Scenario: Validation is not skippable

- **WHEN** an invalid configuration is assembled through the entry
- **THEN** `build()` returns `PhysicsError` naming the violated condition, and there is no public
  path that produces the owned configuration without that check

### Requirement: Configuration types hold private fields

Every public configuration type SHALL hold private fields and provide access through a builder
method, a validated constructor, or a getter. A configuration SHALL NOT be assembled as a struct
literal from outside the crate.

Types absorbed into a builder method (their fields becoming the method's arguments) satisfy this by
construction; types that remain standalone (a plume-nozzle geometry, an imprint spec) SHALL gain a
validated constructor and getters.

A tabular **data row** — a value whose sites are literal table data and whose validity is checked by
the container that consumes the whole table — is not a configuration under this requirement and MAY
keep public fields, provided the consuming container validates it.

#### Scenario: A configuration cannot be struct-literalled

- **WHEN** a consumer outside the crate writes a struct literal for a configuration type
- **THEN** the code does not compile, because the fields are private

#### Scenario: A geometry spec validates its documented envelope

- **WHEN** a plume-nozzle geometry is constructed with a jet ratio of specific heats outside the
  documented Cordell validity envelope
- **THEN** construction returns `PhysicsError`, rather than a value whose validity is stated only in
  prose

### Requirement: The boundary between a configuration and a configuration input is stated

The crate SHALL distinguish a **configuration** — something `CfdFlow` runs, started by a
`CfdConfigBuilder` entry — from a **configuration input** — something a configuration reads, carried
as a value type with its own validated constructor and fluent overrides.

`Mesh`, `Body`, `Seed`, `Observe`, `QttObserve`, `DescentSchedule`, `ForcingRegion` and the blended
coordinate-map configuration SHALL be inputs, not entries. An input SHALL NOT gain a
`CfdConfigBuilder` entry, and an input with more than three constructor arguments SHALL provide a
fluent builder rather than a positional constructor.

#### Scenario: An input keeps its own constructor

- **WHEN** a case reads a descent schedule, a forcing region, or a coordinate map
- **THEN** that value is constructed through its own validated constructor or builder, and no
  `CfdConfigBuilder` entry exists for it

#### Scenario: A wide positional constructor is replaced by a builder

- **WHEN** a configuration input requires more than three parameters
- **THEN** it is assembled through a fluent builder whose `build()` validates, rather than a
  positional constructor

### Requirement: Solver-level harnesses are a declared exemption

A verification or study binary SHALL be permitted to construct a solver directly when its subject is
a solver, kernel, or codec **below** the case layer, and SHALL declare that exemption in its module
documentation with the reason.

The reason SHALL name why the case layer does not apply: the binary measures construction itself,
the binary exercises an operator with no marching case, or no configuration family exists for the
solver in question. A binary that duplicates an existing configuration family by hand SHALL NOT
claim this exemption.

#### Scenario: An exempt harness says why

- **WHEN** a reader opens a verification binary that constructs a solver directly
- **THEN** its module documentation states the exemption and the reason, so the direct construction
  reads as a decision rather than an omission

#### Scenario: A duplicated family is not exempt

- **WHEN** a binary hand-assembles the mesh, solver, zones, seed and observables of an existing
  configuration family
- **THEN** it is configured through that family's `CfdConfigBuilder` entry, and its bespoke per-step
  diagnostics run on the pipeline's per-step hook
