# context-projection Specification

## Purpose
Defines the projection between the context crate's node types and the records: `Recordable<Rec>` beside every node type, the `Storable` payload trait and its built-in implementations, `Context::snapshot` and `Context::restore`, and the precision rule that narrows every scalar to `f64`.
## Requirements
### Requirement: `Recordable` is generic over the record and fallible in both directions

The store crate SHALL declare `pub trait Recordable<Rec>: Sized` with `fn to_record(&self) ->
Result<Rec, ProjectionError>` and `fn from_record(id: ContextoidId, record: Rec) -> Result<Self,
ProjectionError>`, and every implementation SHALL live in `deep_causality_context` beside the
type it converts, in a file named `recordable.rs` under that type's module.

The record parameter, rather than an associated type, is what lets `NoSpaceTime<R>` implement the
trait for `SpaceRecord` and for `SpaceTimeRecord`, one for each `Context` slot it fills.

#### Scenario: A shape change fails beside the type

- **WHEN** a field is added to `GeoSpace<R>` without the matching change to `SpaceRecord::Geo` and
  to `recordable.rs` beside it
- **THEN** `cargo build -p deep_causality_context` fails in that `recordable.rs`

### Requirement: `Storable` is the payload's translation, declared in the context crate

`deep_causality_context` SHALL declare `pub trait Storable: Sized` with `fn to_record(&self) ->
DataRecord` and `fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self,
ProjectionError>`, and SHALL implement `Recordable<DataRecord>` for every `Data<T>` where
`T: Storable + Default + Clone + PartialEq` through one blanket implementation.

The crate ships `Storable` for `f32`, `f64`, `u64`, `i64`, `bool`, `String`, `SubstrateRef`,
`Vec<T: Storable>`, `Option<T: Storable>`, `Float106` and `BFloat16`:

| Type | Record | Reading back |
|---|---|---|
| `f32`, `f64` | `Number` | any other variant is `WrongPayload` |
| `u64` | `Count` | any other variant is `WrongPayload` |
| `i64` | `Integer` | any other variant is `WrongPayload` |
| `bool` | `Flag` | any other variant is `WrongPayload` |
| `String` | `Text` | any other variant is `WrongPayload` |
| `SubstrateRef` | `Reference` | any other variant is `WrongPayload` |
| `Vec<T>` | `List` of `T::to_record` | each element through `T::from_record` |
| `Option<T>` | `List` of zero or one element | a longer list is `WrongPayload` |
| `Float106` | `Fields [("hi", Number), ("lo", Number)]` | exact; a missing entry is `MissingField` |
| `BFloat16` | `Number`, widened exactly | rounded as its arithmetic rounds |

The trait is declared in the context crate rather than the store crate because the two software
scalars are foreign to both, and only a crate that declares the trait may implement it for a
foreign type. Backends never name the trait; they see `DataRecord`. `UncertainData<R>` and
`UncertainBoolData` implement nothing here.

#### Scenario: A user struct becomes persistable with one implementation

- **WHEN** a test defines a struct with a float, a counter and a text field, implements `Storable`
  for it as a `Fields` record, and puts it in a `Data<_>` inside a `Context`
- **THEN** the context snapshots and restores equal, with no change to either crate

#### Scenario: Every shipped payload round-trips

- **WHEN** each type in the table is written with `to_record` and read back with `from_record`
  under an identifier
- **THEN** the result equals the original, `Float106` to every bit, and a `Vec<f64>` of a hundred
  values comes back in order

#### Scenario: A wrong payload names the node

- **WHEN** `f64::from_record(4, DataRecord::Count(1))` is called
- **THEN** it returns `Err(ProjectionError::WrongPayload { id: 4, expected: "Number", found:
  "Count" })`

#### Scenario: A missing field names the field

- **WHEN** `Float106::from_record(9, Fields [("hi", Number(1.0))])` is called
- **THEN** it returns `Err(ProjectionError::MissingField { id: 9, field: "lo" })`

### Requirement: The projection is total on the uniform kinds and refuses loudly on the concrete types

`deep_causality_context` SHALL implement `Recordable` as follows, with `R: RealField + Into<f64> +
FromPrimitive` on every scalar-bearing implementation and `R: RealField` on `NoSpaceTime<R>`:

| Type | Record | Behaviour |
|---|---|---|
| `SpaceKind<R>` | `SpaceRecord` | total: four variants, four arms |
| `TimeKind<R>` | `TimeRecord` | total |
| `SpaceTimeKind<R>` | `SpaceTimeRecord` | total |
| `GeoSpace<R>`, `EcefSpace<R>`, `EuclideanSpace<R>`, `NedSpace<R>` | `SpaceRecord` | writes its own variant; reads any other as `WrongVariant` |
| `EuclideanTime<R>`, `LorentzianTime<R>`, `DiscreteTime`, `EntropicTime` | `TimeRecord` | writes its own variant; reads any other as `WrongVariant` |
| `EuclideanSpacetime<R>`, `LorentzianSpacetime<R>`, `TangentSpacetime<R>` | `SpaceTimeRecord` | writes its own variant; reads any other as `WrongVariant` |
| `NoSpaceTime<R>` | `SpaceRecord` and `SpaceTimeRecord` | `to_record` is `Unrecordable`; `from_record` is `WrongVariant` |
| `Data<T: Storable>` | `DataRecord` | through `Storable`, see above |
| `Contextoid<D, S, T, ST>` | `NodeRecord` | dispatches to `D`, `S`, `T`, `ST`; `Root` ↔ `NodeRecord::Root`; the phantom arm is `Unrecordable` |

`SymbolicTime`, `CausalSetSpacetime` and `ConformalSpacetime` implement nothing.
`TangentSpacetime::from_record` restores the stored metric tensor, not the default one.

#### Scenario: Every uniform variant round-trips

- **WHEN** each variant of `SpaceKind<f64>`, `TimeKind<f64>` and `SpaceTimeKind<f64>` is built at
  a value where every field differs from every other field, projected with `to_record` and read
  back with `from_record` under its identifier
- **THEN** the result equals the original, including the identifier, the scale, the datum and,
  for `Tangent`, a non-default metric tensor

#### Scenario: A concrete type refuses the wrong variant and names the node

- **WHEN** `EuclideanSpace::<f64>::from_record(9, SpaceRecord::Geo { … })` is called
- **THEN** it returns `Err(ProjectionError::WrongVariant { id: 9, expected: "Euclidean", found:
  "Geo" })`, and the same holds for every other concrete type against every variant it does not
  hold

#### Scenario: A root round-trips as a node

- **WHEN** a `Contextoid` holding `Root` is written with `to_record` and read back
- **THEN** the record is `NodeRecord::Root` and the result is a `Root` contextoid under the same
  identifier

#### Scenario: The absent spacetime cannot be recorded

- **WHEN** `Recordable::<SpaceRecord>::to_record` and `Recordable::<SpaceTimeRecord>::to_record`
  are called on a `NoSpaceTime<f64>`
- **THEN** each returns `Err(ProjectionError::Unrecordable { .. })`, and a `Context<Data<f64>,
  NoSpaceTime<f64>, EuclideanTime<f64>, NoSpaceTime<f64>>` holding a root, a data node and a time
  node still snapshots and restores, because no node of the absent type is in the graph

### Requirement: Precision is projected onto `f64` at the bound

Every scalar-bearing `Recordable` implementation SHALL narrow through `Into<f64>` on the way out
and lift through `FromPrimitive::from_f64` on the way back, and a `from_f64` that returns `None`
SHALL be `ProjectionError::Scalar { id, value }`.

#### Scenario: A wide coordinate stores as a double

- **WHEN** an `EuclideanSpace<Float106>` whose coordinates carry more than 53 bits of significand
  is projected and read back
- **THEN** the record's fields are `f64`, and the restored coordinates equal the originals rounded
  to double precision

#### Scenario: A narrow coordinate restores rounded

- **WHEN** an `EuclideanSpace<BFloat16>` is projected and read back
- **THEN** the record holds the exact widened values and the restored node equals the original

### Requirement: `Context::snapshot` walks every graph into a canonical snapshot

`Context<D, S, T, ST>` SHALL provide `pub fn snapshot(&self) -> Result<ContextSnapshot,
ProjectionError>` under the bounds `D: Datable + Clone + Recordable<DataRecord>`, `S: Spatial +
Clone + Recordable<SpaceRecord>`, `T: Temporal + Clone + Recordable<TimeRecord>` and `ST:
SpaceTemporal + Clone + Recordable<SpaceTimeRecord>`.

The walk reads every live node and every edge of the base graph and of each extra graph through
`get_last_index`, `get_node` and `get_edges`, replacing graph indices with contextoid identifiers.
It orders nodes by identifier, edges by `(from, to)` and extras by identifier, and records each
extra's identifier and name, so two contexts of equal content yield equal snapshots. The four index
maps, the current extra-context identifier and the frozen form of the graph are not recorded.

#### Scenario: A snapshot is canonical

- **WHEN** two contexts hold the same nodes and edges inserted in different orders
- **THEN** their snapshots are equal

#### Scenario: A snapshot names identifiers, never indices

- **WHEN** a context holds nodes with identifiers 100 and 200 at graph indices 0 and 1 joined by
  an edge
- **THEN** the snapshot's edge is `RelationRecord { from: 100, to: 200, .. }`

#### Scenario: An unrecordable node stops the walk

- **WHEN** a context holds a `Spaceoid(NoSpaceTime)` contextoid and `snapshot` is called
- **THEN** it returns `Err(ProjectionError::Unrecordable { .. })` naming that node

### Requirement: `Context::restore` rebuilds a context from a snapshot

`Context<D, S, T, ST>` SHALL provide `pub fn restore(snapshot: ContextSnapshot) -> Result<Self,
ProjectionError>` under the same bounds as `snapshot`. It checks the version, builds the base graph
with `with_capacity(id, name, nodes.len())`, adds every node and edge through `ContextuableGraph`
using a local identifier-to-index map, recreates each extra through `extra_ctx_add_new_with_id`
under its identifier and name and fills it the same way, and leaves no extra context current.

`restore` refuses a snapshot whose nodes carry one identifier twice, whose edges name an identifier
no node carries, or whose extra carries identifier 0, each with `ProjectionError::Identity { id,
rule }`.

#### Scenario: A context restores equal

- **WHEN** a `UniformContext` holding a root, a data node, a time node, a space node, a spacetime
  node, two named extra contexts each with nodes, and edges of all four relation kinds is
  snapshotted and restored
- **THEN** the restored context's snapshot equals the original's, `number_of_nodes` and
  `number_of_edges` match, `get_edge` returns the same relation for every pair, and
  `extra_ctx_get_name` returns each extra's name

#### Scenario: Run-time state starts empty

- **WHEN** a context with a current data index and a current extra context set is snapshotted and
  restored
- **THEN** the restored context reports no current data index, and `extra_ctx_get_current_id`
  returns 0

#### Scenario: A dangling edge is refused

- **WHEN** a snapshot's edge names an identifier no node carries
- **THEN** `restore` returns `Err(ProjectionError::Identity { .. })` naming that identifier
