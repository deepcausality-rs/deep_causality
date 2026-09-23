## ADDED Requirements

### Requirement: One record per engine variant, monomorphic and named

The store crate SHALL declare `NodeRecord`, `DataRecord`, `TimeRecord`, `SpaceRecord`,
`SpaceTimeRecord`, `ContextRecord`, `ContextoidRecord`, `RelationRecord`, `ExtraContextSnapshot`
and `ContextSnapshot`, with every scalar field `f64`, every tick `u64`, every identifier a store
alias, and every field named.

The correspondence to the engine is exact:

| Record | Variants | Mirrors |
|---|---|---|
| `NodeRecord` | `Root`, `Data(DataRecord)`, `Time(TimeRecord)`, `Space(SpaceRecord)`, `SpaceTime(SpaceTimeRecord)` | `ContextoidType` less its phantom arm |
| `TimeRecord` | `Euclidean { scale, value }`, `Lorentzian { scale, value }`, `Discrete { scale, tick }`, `Entropic { tick }` | `TimeKind` |
| `SpaceRecord` | `Geo { lat, lon, alt, datum }`, `Ecef { x, y, z }`, `Euclidean { x, y, z }`, `Ned { north, east, down }` | `SpaceKind` |
| `SpaceTimeRecord` | `Euclidean { x, y, z, t, scale }`, `Lorentzian { x, y, z, t, scale }`, `Tangent { x, y, z, t, dt, dx, dy, dz, metric: [[f64; 4]; 4] }` | `SpaceTimeKind` |

`NodeRecord::Root` is an ordinary record: a root is stored, linked and hydrated like any other
node. `ContextRecord` holds `id` and `name`; `ContextoidRecord` holds `id` and `node`;
`RelationRecord` holds `from`, `to` and `kind`; `ExtraContextSnapshot` holds `id`, `name`, `nodes`
and `edges`; `ContextSnapshot` holds `version`, `context`, `nodes`, `edges` and `extras`. Struct
fields are private with constructors and getters; enum variant fields are named. Every record
derives `Debug`, `Clone` and `PartialEq`, and `Copy` where no field forbids it.

`SymbolicTime` has no record because `TimeKind` has no arm for it; `TimeScale::Symbolic` stays
because a `DiscreteTime` can be scaled by it. `NoSpaceTime` has no record because it holds no
coordinate.

#### Scenario: Variant counts match the engine

- **WHEN** the variants of `SpaceRecord`, `TimeRecord` and `SpaceTimeRecord` are counted
- **THEN** they number four, four and three, matching `SpaceKind`, `TimeKind` and `SpaceTimeKind`,
  and a projection test in the context crate maps every engine variant to a distinct record variant

#### Scenario: A position is not positional

- **WHEN** a `SpaceRecord::Ned` is constructed
- **THEN** `north`, `east` and `down` are named at the construction site, and no record type
  carries a `[f64; 3]`

### Requirement: `DataRecord` is a closed value tree

`DataRecord` SHALL have exactly the variants `Number(f64)`, `Count(u64)`, `Integer(i64)`,
`Flag(bool)`, `Text(String)`, `Reference(SubstrateRef)`, `List(Vec<DataRecord>)` and
`Fields(Vec<(String, DataRecord)>)`, so that a data payload of any shape reaches a store as a
record the store crate can hold without knowing the payload's type.

`Fields` is an ordered list of named values rather than a map, so a snapshot's equality is
well defined. Which variants a backend accepts is the backend's policy.

#### Scenario: A struct payload is representable

- **WHEN** a payload with a floating-point field, a counter field, a text field and a nested list
  is written as a `DataRecord`
- **THEN** it is one `Fields` value whose entries are `Number`, `Count`, `Text` and `List`, and
  reading the entries back in order yields the same values

#### Scenario: Equality is structural and ordered

- **WHEN** two `Fields` records hold the same entries in different orders
- **THEN** they are not equal, and two holding the same entries in the same order are

### Requirement: The snapshot carries a version and refuses a newer one

The store crate SHALL declare `pub const RECORD_VERSION: u16 = 1;`, `ContextSnapshot::new` SHALL
set the snapshot's `version` to it, and `Context::restore` SHALL refuse a snapshot whose version is
greater than `RECORD_VERSION` with `ProjectionError::Version { found, supported }`.

#### Scenario: A current snapshot restores

- **WHEN** a snapshot with `version == RECORD_VERSION` is restored
- **THEN** the version check passes

#### Scenario: A newer snapshot is refused

- **WHEN** a snapshot carrying `version == RECORD_VERSION + 1` is restored
- **THEN** the result is `Err(ProjectionError::Version { found: RECORD_VERSION + 1, supported:
  RECORD_VERSION })` and no node is built

### Requirement: The identifier reserve is opaque except for `next`

`IdReserve` SHALL hold identifiers a store has made unique and expose `new(Vec<ContextoidId>)`,
`next(&mut self) -> Option<ContextoidId>` as its `Iterator` implementation, and `remaining(&self)
-> usize`, and nothing that says how an identifier was made unique.

#### Scenario: A reserve yields each identifier once

- **WHEN** a reserve of three identifiers is drained with `next`
- **THEN** it yields the three in order, `remaining` counts down from three to zero, and a fourth
  `next` returns `None`

### Requirement: `ProjectionError` names the node and the rule

The store crate SHALL declare `ProjectionError` in the repository's error convention: a public
tuple struct around a public `ProjectionErrorEnum`, with one constructor function per variant, a
`kind()` accessor, and `Debug`, `Clone`, `PartialEq`, `Display` and `std::error::Error` on the
struct. The enum's variants are `WrongVariant { id, expected, found }`, `WrongPayload { id,
expected, found }`, `MissingField { id, field }`, `Unrecordable { id, kind }`, `Scalar { id,
value }`, `Identity { id, rule }` and `Version { found, supported }`.

`expected`, `found`, `field`, `kind` and `rule` are `&'static str`; `id` is a `ContextoidId`, or a
`ContextId` where the rule concerns a container.

#### Scenario: Every variant displays its identifier

- **WHEN** each variant is formatted with `Display`
- **THEN** the output contains the identifier it carries and, for `WrongVariant` and
  `WrongPayload`, both the expected and the found name
