# DeepCausality, distilled: the context and its store

Source: every file in `deep_causality_context/` (0.1.2) and `deep_causality_context_store/`
(0.1.2), read in full on 2026-09-30: `src/`, `tests/`, `Cargo.toml`, `BUILD.bazel` and the
READMEs. The relation to the causal engine is checked against `deep_causality/` (see `dc.md`).

## The finding

A context is a directed graph of typed, identified nodes: data, time, space, spacetime and root.
The causal engine never reads it. A causaloid holds a context behind `Arc<RwLock<C>>` and passes it,
as `Option<C>`, to the user's contextual function; what the function reads there is the only way
time, space or external data reach a cause. That is how "causality is a spacetime-agnostic
functional dependency" is kept: the dependency is a function, and spacetime is a value handed to it.

The context crate has two halves of unequal age. The first is the in-memory graph with its node
types and the "extra context" side graphs. The second, newer half makes the context persistent: a
zero-dependency contract crate (`deep_causality_context_store`) states what a store must do, and
`Context::snapshot`, `Context::restore`, `Context::apply` and `ContextStore` project a context onto
it and back. Branching a context, storing the branch, and keeping a context current from a stream
of store events all work and are tested end to end against an in-memory backend.

Nothing outside the two crates uses the second half. No crate, example or test in the workspace
calls `snapshot`, `restore`, `apply`, `ContextStore` or any `extra_ctx_*` method. The persistence
layer and the extra-context model are complete as specified and unexercised by the engine.

## 1. What a context is

```rust
pub struct Context<D, S, T, ST> {                 // D: Datable, S: Spatial, T: Temporal, ST: SpaceTemporal
    id: ContextId,                                // u64
    name: String,
    base_context: UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>,
    id_to_index_map: IdMap<ContextoidId, usize>,  // node identifier -> graph index, base graph only
    extra_contexts: Option<IdMap<ContextId, ExtraContext<D, S, T, ST>>>,
    extra_context_id: ContextId,                  // the current extra; 0 = none
    highest_extra_context_id: ContextId,          // allocation high-water mark
    current_data_map, previous_data_map,          // IdMap<usize, usize>
    current_index_map, previous_index_map,        // IdMap<usize, usize>
}
```

`deep_causality_context/src/types/context_types/context_graph/mod.rs`. `IdMap` is `HashMap` under
`std` and `BTreeMap` without it (`src/utils/id_map.rs`).

- A **contextoid** is `Contextoid { id, vertex_type: ContextoidType<D, S, T, ST> }`. The type enum
  has five variants, `Datoid(D)`, `Tempoid(T)`, `Root(Root)`, `Spaceoid(S)`, `SpaceTempoid(ST)`,
  and a hidden `_Marker(PhantomData<()>)`. `ContextKind` names the five.
- An **edge** carries a `RelationKind`: `Datial` (default), `Temporal`, `Spatial`, `SpaceTemporal`.
  Edges are directed. The kind is stored, so two edges between nodes of the same types can differ.
- **The base graph has two states.** Mutable, it takes changes; `freeze` builds a compressed
  sparse row form in O(V + E) over which `outbound_edges` and `inbound_edges` list neighbours, and
  every change errors until `unfreeze`. Freezing drops removed nodes' slots, so indices shift;
  the context moves `id_to_index_map` and the four index maps with them. Each graph holds a
  contextoid identifier at most once: `add_node`, `update_node` and `extra_ctx_add_node` refuse a
  held one.
- **Extra contexts** are named side graphs of the same node type, held beside the base graph. One
  is "current" at a time; every `extra_ctx_*` operation acts on the current one.
- **Four index maps** hold run-time bookkeeping: current and previous data index (keys 1 and 0),
  current and previous time index keyed by `TimeScale as usize` for minute through year. They are
  plain key-value slots; the crate's own logic never reads them. The DBN example
  (`examples/classical_causality_examples/classical_via_causaloid/dbn/model.rs`) is their one
  user in the workspace.

The four parameters make a context a product of four independent choices. Three context aliases
fix them:

| Alias | Data | Space | Time | Spacetime |
|---|---|---|---|---|
| `BaseContext` | `Data<f64>` | `EuclideanSpace<f64>` | `NewtonianTime<f64>` | `NewtonianSpacetime<f64>` |
| `UniformContext` | `Data<u64>` | `SpaceKind<f64>` | `TimeKind<f64>` | `SpaceTimeKind<f64>` |
| `SubstrateContext` | `Data<SubstrateRef>` | `SpaceKind<f64>` | `TimeKind<f64>` | `SpaceTimeKind<f64>` |

Each has a matching `*Contextoid` alias (`BaseContextoid`, `UniformContextoid`,
`SubstrateContextoid`). `f64` is `FloatType`; `Data<u64>` is
`Data<NumberType>` from `deep_causality_core`.

## 2. The node vocabulary

Every coordinate-carrying node is generic in one scalar `R: RealField`. No concrete float is named in
`src/`; `tests/types/context_types/context_graph/scalar_parameter_tests.rs` builds the same context at
`BFloat16` and at `Float106` and shows the two disagree at `0.1`, which is the negative control that
the scalar reaches the node.

| Kind | Types | Notes |
|---|---|---|
| Data | `Data<T>`, `UncertainData<R>`, `UncertainBoolData<R>` | `T: Default + Clone + PartialEq`; `Copy` is asked only by `Adjustable`, so `Data<Vec<f64>>` is a valid node. |
| Space | `EuclideanSpace`, `EcefSpace`, `GeoSpace`, `NedSpace`, `NoSpace`; `SpaceKind` over the first four | `GeoSpace` carries a `VerticalDatum` (WGS84, EGM96, EGM2008, ISA, Terrain); its distance is the WGS 84 geocentric straight line (IOGP 373-7-2 §2.2.1), NaN unless both datums are `WGS84`; `GeoSpace::new` refuses a latitude outside [−90, 90]. |
| Time | `NewtonianTime` (absolute time), `MinkowskiTime` (inertial-frame coordinate time), `DiscreteTime`, `EntropicTime` (`u64` tick), `SymbolicTime` (labelled `i64`), `NoTime`; `TimeKind` over the first four | `TimeKind` lifts ticks into `R` with `lift_count`. |
| Spacetime | `GalileanSpacetime` and `NewtonianSpacetime` (spatial metric (0,+,+,+), `Metric::PGA(4)`), `MinkowskiSpacetime` (−,+,+,+), `TangentSpacetime` (event, tangent vector and a validated Lorentzian 4×4 metric tensor); `SpaceTimeKind` over the four; `CausalSetSpacetime` (a causal order, carried as a `Data` payload, not in this slot) | Coordinates are time first, `0 => t`. Each reports its own `Metric` through `MetricSignature`. Galilean distance exists only between simultaneous events. |
| Root | `Root { id }` | An ordinary node; stored as `NodeRecord::Root`. |
| Absence | `NoSpace<R>`, `NoTime`, `NoSpaceTime<R>` | Zero-sized; fill the spatial, temporal and spacetime slots of a context that holds no node of that kind. |

Three design choices recur across the node types:

- **The node, not the context, carries the metric.** `SpaceTimeKind::metric` forwards to the variant,
  so one context can hold a Newtonian and a relativistic node side by side
  (`mixed_spacetime_tests.rs`). `TangentSpacetime` reports `Lorentzian(4)` however its tensor is
  replaced, because a signature does not change under continuous evolution.
- **Units are converted where physics needs them.** `MinkowskiSpacetime::time()` converts every
  `TimeScale` to seconds before the interval is formed; a test shows seconds, minutes and
  milliseconds give the same interval.
- **Mismatches are recorded, not solved.** `space/mod.rs` states that a geodetic datum is lost when a
  position is converted to any Cartesian type, and `GeoSpace::distance` asserts equal datums only
  in debug builds.

`Adjustable<T>` updates a node in place from an `ArrayGrid`: `update` replaces, `adjust` adds, and
both refuse non-finite results for space and spacetime types. `UncertainAdjustable` replaces an
uncertain value wholesale.

## 3. How the context meets the causal engine

`deep_causality` depends on the context crate and uses exactly four things from it: the `Context`
type as the `C` of a causaloid, the traits `Datable`/`Spatial`/`Temporal`/`SpaceTemporal` as bounds,
the node types inside the generative `Operation` AST, and the test builders.

```text
Causaloid<I, O, PS, C>                         C = Arc<RwLock<Context<D, S, T, ST>>>
  └─ context_causal_fn: fn(CausalEffect<I>, PS, Option<C>) -> PropagatingProcess<O, PS, C>
        evaluate          → called with the causaloid's stored context
        evaluate_stateful → called with the incoming process's context, else the stored one
```

- The engine hands the context to the function and reads nothing from it. Collection and graph
  reasoning (`dc.md` §3) never inspect a node.
- The monad threads the context forward, and a continuation's returned context replaces the old one
  (`bind`, `dc.md` §2.2). A function can therefore swap the context it passes downstream; it cannot
  mutate the shared `Arc<RwLock<_>>` except by taking the write lock itself.
- `UncertainActivationPredicate` takes `&Context`, so a context can gate whether an uncertain
  causaloid fires.
- The generative interpreter creates, updates and deletes contexts and contextoids as AST
  operations. Its `CreateExtraContext` creates a second top-level context and does not use this
  crate's extra-context API (`dc.md` §6, item 7).

A context is therefore the environment a cause is evaluated in. Swapping one `Arc` for another is a
counterfactual: the same causaloid, evaluated against a different world.

## 4. The persistence contract: `deep_causality_context_store`

A crate with no dependencies at all (`std` only gates the in-memory backend under `utils_test`). It
defines records, the storage trait, an optional streaming trait, an optional substrate trait, and a
reference backend.

### 4.1 Records

| Record | Content |
|---|---|
| `ContextRecord` | `id`, `name` |
| `ContextoidRecord` | `id`, `node: NodeRecord` |
| `NodeRecord` | `Root`, `Data(DataRecord)`, `Time(TimeRecord)`, `Space(SpaceRecord)`, `SpaceTime(SpaceTimeRecord)` |
| `DataRecord` | a value tree: `Number(f64)`, `Count(u64)`, `Integer(i64)`, `Flag(bool)`, `Text`, `Reference(SubstrateRef)`, `List`, `Fields(Vec<(String, DataRecord)>)` |
| `SpaceRecord` | `Geo{lat, lon, alt, datum}`, `Ecef`, `Euclidean`, `Ned` |
| `TimeRecord` | `Euclidean{scale, value}`, `Lorentzian{scale, value}`, `Discrete{scale, tick}`, `Entropic{tick}` |
| `SpaceTimeRecord` | `Euclidean`, `Lorentzian`, `Tangent{…, metric: [[f64; 4]; 4]}` |
| `RelationRecord` | `from`, `to`, `kind` |
| `ContextSnapshot` | `version`, the context record, nodes, edges, and one `ExtraContextSnapshot` per referenced container |

Every scalar in a record is `f64`. `RECORD_VERSION = 1`.

### 4.2 The model a store implements

The invariants, stated in the crate's README and pinned by `tests/`:

- **A context is a set of links.** Nodes and edges live in the store once; a container links node
  identifiers. Two containers share a node by linking the same identifier.
- **A node is immutable under its identifier.** Creating an existing identifier with the same record
  is idempotent; with a different record it is a conflict.
- **Identity is the store's.** `reserve(n)` leases identifiers as a non-`Clone` iterator
  (`IdReserve`); creating a node under an unreserved identifier is refused.
- **One relation per ordered pair.** A second edge `a → b` with a different kind is a conflict.
- **Containers reference containers.** `attach` is idempotent, refuses self-reference, and may form
  cycles. `hydrate` resolves references one level deep.
- **`commit` is all-or-nothing.** A batch of `ContextWrite`s (`CreateContext`, `CreateNode`,
  `CreateEdge`, `Link`, `Attach`) names containers it creates through `ContainerRef::Created(i)`.

`ContextStorage` has fourteen async operations, each returning `impl Future + Send`: `reserve`,
`create_context`, `retract_context`, `create_node`, `retract_node`, `create_edge`, `retract_edge`,
`link`, `unlink`, `attach`, `detach`, `commit`, `lookup`, `hydrate`, with associated `Error` and
`Slice` (what `hydrate` accepts).

### 4.3 Streams and substrates

`ContextStorageStream` adds `subscribe(slice, from: Option<Cursor>) -> (ContextSnapshot, Events)`,
`apply(event)` and `apply_batch(events)`. The snapshot and the stream come from one call, so no
change falls between them. A subscription's scope is fixed at its start: the named container and the
containers it referenced then. A container attached later arrives as an empty extra until the host
resubscribes from its cursor (`subscribe_tests.rs`).

`ContextEvent` has twelve variants. Nine mirror an operation. `ContextCreated` is report-only,
because a container identifier always comes from the store. `NodeEntered` and `NodeLeft` report a
view whose rule began or stopped reading a node. A membership event carries the node's record and
every relation between it and the container's members, so a subscriber can apply it to a hydrated
context without having seen earlier events.

`Substrate` separates where a value lives from where the graph lives: `deposit(node, value)` returns
a `SubstrateRef { source, key }` and `resolve(ref)` returns the value. A store may then hold
references only.

The in-memory backend, `MemoryStorage`, is a fold over an event log behind `Arc<Mutex<_>>`. Contexts
and nodes share one identifier counter starting at 1; `commit` and `apply_batch` run on a clone and
swap it in on success. `MemorySubstrate` keys values by node identifier under source `"memory"`.
`block_on` is a no-op-waker poller for tests.

## 5. Projection: the context onto records and back

`Recordable<Rec>` (`to_record`, fallible `from_record(id, rec)`) is implemented by every node type
for its record family, by the `*Kind` enums as total four- or three-arm dispatches, and by
`Contextoid` for `NodeRecord`. `Data<T>` is `Recordable<DataRecord>` for every `T: Storable`.
`Storable` is implemented for `f64`, `f32` (refuses a finite double beyond its range), `u64`,
`i64`, `bool`, `String`, `SubstrateRef`, `Vec<T>`, `Option<T>` (a list of zero or one),
`Float106` (as `Fields{hi, lo}`, exact) and `BFloat16`. A user struct becomes persistable with one
`Storable` impl (`tests/traits/storable/storable_tests.rs`).

`ProjectionError` names the node in every variant: `WrongVariant`, `WrongPayload`, `MissingField`,
`Unrecordable` (`NoSpaceTime`, `_Marker`), `Scalar` (the target scalar cannot hold the `f64`),
`Identity`, `Version`, `Rejected` (the record's values break a rule of the node type) and `Frozen` (an event would
change a frozen base graph; this variant names the context).

On `Context`:

- **`snapshot()`** walks the base graph and every extra and returns canonical records: nodes by
  identifier, edges by `(from, to)`, extras by identifier. Indices, the current extra and the four
  index maps are run-time state and are not recorded. A graph with two parallel edges is refused,
  because the store holds one relation per pair.
- **`restore(snapshot)`** rebuilds a context. Every extra becomes a *stored* extra under the
  snapshot's identifier. It refuses a newer version, a duplicated node, extra or edge, a dangling
  edge, and extra identifier 0.
- **`apply(&event)`** applies one store event idempotently. It routes by the container the event
  names: the context's own identifier means the base graph, a stored extra's identifier means that
  extra. Node and edge events apply to every graph holding the node. An event naming a container the
  context does not hold, or a *local* extra, is `Identity`.
  While the base graph is frozen, an event that would change it is `Frozen` and changes nothing,
  extras included; an echo of a state the base graph holds, and an event for an extra alone,
  apply.

The stored/local distinction exists because both the store and `extra_ctx_add_new` count from 1. An
extra created locally is never reached by a container event, so a store container that happens to
receive the same identifier cannot merge into it (`test_a_local_extra_never_takes_a_store_container`).

`ContextStore<S: ContextStorage>` wraps a backend by value and adds no cache:

| Method | Effect |
|---|---|
| `hydrate(slice)` | backend snapshot, then `Context::restore` |
| `reserve(n)` | pass-through |
| `store_branch(name, &context)` | stores the context as a new container in one `commit`; returns its identifier |
| `subscribe(slice, from)` | only when `S: ContextStorageStream`, enforced at compile time |
| `create_node_via(substrate, nodes)` | deposits each non-reference data payload, then `create_node`; idempotent on retry |
| `hydrate_via(substrate, slice)` | resolves every reference before `restore` |

`store_branch` applies one rule over the base graph and every extra together. A node the store holds
under the same record is linked. A node the store does not hold is created under its own
identifier. A node the store holds under a different record is created under a fresh reserved
identifier, and the branch's edges are remapped to it. A record two graphs share keeps one
identifier; two graphs carrying different records under one identifier become two nodes. Each extra
becomes its own container, attached to the new one. The in-memory branch is unchanged. A refusal
leaves nothing behind, because every write goes through one `commit`
(`test_a_refused_store_writes_nothing`).

## 6. The lifecycle the two halves support

```text
store ──hydrate──▶ Context ──Arc<RwLock<_>>──▶ causaloid.evaluate(…) ──▶ effect
                     │
                     ├─ clone, update_node / add_node      (a branch: a counterfactual world)
                     │        └─ store_branch ──▶ new container linking what it shares
                     │
store ──subscribe──▶ (Context, events) ── between evaluations: context.apply(&event)
```

`test_a_context_driven_by_its_stream_converges_to_the_store` drains a stream into a context and
compares its snapshot with a fresh hydrate after each change; they agree, including an edge created
before either end was linked and an unlink-relink cycle.

## 7. Gaps and inconsistencies

Ordered by consequence.

1. **No consumer.** The store, the projection and the extra-context API are used only by their own
   tests. The causal engine reads a context through user functions only, and the generative
   interpreter's `CreateExtraContext` builds an unlinked top-level context instead of an extra.
2. **Symbolic node types sit outside the uniform kinds and the store.** `SymbolicTime` implements
   `Temporal`, so it fills the time slot of a context declared with it, but it is not a `TimeKind`
   variant (the arm is commented out) and has no `Recordable` impl; its `scalar_projector.rs` is a
   commented-out file. `CausalSetSpacetime` implements none of `Spatial`, `Temporal` or
   `SpaceTemporal`, so it cannot fill the spacetime slot; a context holds it as a data payload,
   `Data<CausalSetSpacetime>`, and it has no `Storable` impl, so such a context cannot be
   snapshotted.
3. **`apply` finds nodes by linear scan.** `index_of` walks every graph index for each lookup, in the
   base graph too, although the base graph keeps `id_to_index_map`. Edge events scan every graph.
   Extras have no identifier index at all, and `extra_ctx_*` operations take graph indices.
4. **A context can hold what it cannot store.** The in-memory graph accepts parallel edges;
   `snapshot` then refuses the whole context. A `NoSpaceTime` node, which the type allows in the
   space slot, makes `snapshot` return `Unrecordable`.
5. **Coordinates stop being generic at the store.** Every scalar record is `f64`. A `Float106`
   coordinate loses its low half on snapshot, and the tests pin this ("precision is spent at the
   bound"). A `Data<Float106>` payload keeps both halves, because `Storable` writes it as `Fields`.
6. **`Adjustable` is not uniform.**
   - `MinkowskiSpacetime` chooses indices by the grid's dimensionality; every other space and
     spacetime type reads fixed 3D indices `(0, 0, k)`, so a spacetime needs a grid of depth 4 or more.
   - `NewtonianTime::update` and `MinkowskiTime::update` store the value unchecked, while every
     space type's `update` refuses a non-finite value.
   - `DiscreteTime::adjust` refuses a zero *result*; `EntropicTime::adjust` refuses a zero
     *adjustment*. `DiscreteTime(10) + 0` succeeds and `EntropicTime(42) + 0` fails.
7. **Panics on public paths.** `ContextoidType::kind()` and its `Display` panic on the hidden but
   public `_Marker` variant (tested as `should_panic`). `extra_ctx_add_new` computes
   `highest_extra_context_id + 1` and `expect`s the insert, so an allocation after `u64::MAX` is
   held overflows.
8. **Distance semantics differ by type.** `NewtonianSpacetime::distance` ignores `t`.
   `TangentSpacetime` reports `TimeScale::Second` unconditionally and its `time()` returns the raw
   `t`. `GeoSpace::distance` checks equal datums only under `debug_assertions`.
9. **Minor.** The `UniformContext` docstring calls `NumberType` "typically an alias for a
   floating-point or integer"; it is `u64`. A context-store test is named
   `test_the_thirteen_operations…` while the README lists fourteen.

## 8. File map

```text
deep_causality_context/src/
  alias/                  FloatType, ContextId, ContextoidId, Base/Uniform/Substrate aliases
  errors/                 AdjustmentError, ContextIndexError, IndexError, UpdateError, StoreError<E, B>
  extensions/storable/    Storable impls: scalars, text, reference, collections, Float106, BFloat16
  traits/                 Contextuable (Coordinate, Distance, Metric*, Spatial, Temporal, …),
                          ContextuableGraph, ExtendableContextuableGraph, indexable, Adjustable, Storable
  types/context_node_types/
    data/, data_uncertain/, root/
    space/{ecef, euclidean, geo, ned, no_space, space_kind}
    space_time/{causal_set, conformal, galilean, minkowski, newtonian, no_space_time, tangent, space_time_kind}
    time/{discrete, entropic, minkowski, newtonian, no_time, symbolic, time_kind}
  types/context_types/
    context_graph/        Context, ExtraContext, snapshot.rs, restore.rs, apply.rs
    context_store/        ContextStore: hydrate, store_branch, subscribe, substrate (…_via)
    contextoid/           Contextoid, ContextoidType, ContextKind
  utils/id_map.rs         HashMap under std, BTreeMap without
  utils_test/             get_context, get_base_context, get_test_context, array-grid fixtures

deep_causality_context_store/src/
  types/                  records, ContextEvent, ContextWrite, ContainerRef, IdReserve, SubstrateRef,
                          RelationKind, TimeScale, VerticalDatum
  traits/                 ContextStorage, ContextStorageStream, ContextEvents, Substrate, Recordable
  errors/                 ProjectionError, MemoryStorageError, MemorySubstrateError
  utils_test/             MemoryStorage, MemorySubstrate, block_on
```
