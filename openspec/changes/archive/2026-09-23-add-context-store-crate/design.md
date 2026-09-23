## Context

`Context` is an in-memory hypergraph: a base graph of `Contextoid` nodes with `RelationKind`
edges, any number of extra graphs keyed by `ContextId`, four index maps a running model sets, and
an allocator for extra-context identifiers. Every field is private. The only ways in are
`with_capacity`, `ContextuableGraph` and `ExtendableContextuableGraph`; the only ways out are the
same traits, by graph index. Nothing writes a context to storage or reads one back.

Facts verified against the tree at `9d11c0114` that fix the shape of the solution:

1. **The context crate has seven internal dependencies** (`algebra`, `core`, `data_structures`,
   `metric`, `num`, `uncertain`, `ultragraph`) and sits at Tier 6. A backend written against it
   links all seven, including the tensor and algebra stacks, to read five node kinds and four edge
   kinds.
2. **Nine commits marked breaking touched the context layer** between the extraction and the
   working tree. Every one changed a struct a reader would have serialised.
3. **`ultragraph` exposes what a walk needs.** `get_last_index`, `get_node(i)` and
   `get_edges(i)` together yield every live node with its index and every edge with its weight, so
   no addition to `ultragraph` is required.
4. **The extra-context allocator collides.** `extra_ctx_add_new` computes
   `number_of_extra_contexts + 1`, and `extra_ctx_add_new_with_id` increments the count without
   moving the allocator, so an explicit identifier above the count is reached and the `expect`
   inside `extra_ctx_add_new` panics.
5. **`NoSpaceTime` fills two slots of one `Context`.** A context with a clock and no position is
   `Context<D, NoSpaceTime<R>, T, NoSpaceTime<R>>`, and the type's own docstring reports that most
   contexts in the workspace are of this shape. Any projection trait with one associated record
   type cannot be implemented for it twice.
6. **A root is an inert node.** `Context` has no root field; the engine's only notion of a root is
   a node whose `ContextoidType` is `Root`. Library code touches that variant in three places, all
   inside the enum itself. Two example files create a `Root`; neither adds an edge to it.
7. **Extra contexts have no name and no user.** No call to any `extra_ctx_*` method exists outside
   four test files in the context crate. An extra is numbered 1, 2, 3 by creation order and carries
   nothing else.
8. **Data payloads are open.** `Data<T>` bounds `T: Default + Clone + PartialEq` and nothing more.
   The tree carries `Data<f64>`, `Data<u64>` and, in the granger example, `Data<Vec<FloatType>>`,
   two whole time series in one node.
9. **`SubstrateRef` is an untracked draft** in `deep_causality_context/src/types/context_types/`,
   not registered in any `mod.rs` and not compiled.
10. **No library crate has an asynchronous surface.** A search of every `src/` tree for
    `impl Future` and `async fn` finds nothing; only the tokio example is asynchronous.
11. **The existing specs pin what this change touches.** `context-crate-identity` lists the
    crate's dependencies exactly and names `RelationKind` and `TimeScale` as owned;
    `context-id-parameter` states that the identifier aliases appear only in the context crate.
    Both need MODIFIED deltas. No spec pins the extra-context API.

## Goals / Non-Goals

**Goals:**

- A backend links one crate with no dependencies and implements one trait to persist a `Context`.
- A change to a node type's shape is a compile failure in the context crate, beside the type,
  before any backend sees data.
- A data payload of any shape becomes persistable by implementing one trait on the payload, with
  nothing added to either crate.
- `BaseContext`, `UniformContext` and every context over the `Kind` enums round-trips through a
  snapshot; a context over a narrow concrete type fails at the first node it cannot hold, naming it.
- A context stored in a backend can reference other stored contexts, many to many, and hydrating
  it materialises the referenced ones as its extra contexts.
- The contract is asynchronous and adds no runtime dependency to either crate.
- The whole contract is exercised end to end, in this repository, through an in-memory backend.

**Non-Goals:**

- A concrete persistent backend of any kind. Each is a separate crate over the trait, outside this
  repository, because the smallest embedded database in reach brings over seventy transitive
  crates.
- Traversal, query or search operations on the trait. The trait is a persistence contract, and one
  that grew queries would be a second graph API with the engine as its owner.
- Records for `UncertainData`, `UncertainBoolData`, `SymbolicTime`, `CausalSetSpacetime` and
  `ConformalSpacetime`. A distribution is a computation over samples, not a row; the symbolic
  types are outside the `Kind` enums the projection covers.
- `no_std` for the store crate. Settled by the author.
- Persisting the four index maps or the frozen form of the graph. Both are run-time state.
- A stream that widens its own scope. A subscription's scope is fixed; see Decision 15.
- Bounding the causal monad's `Context` channel. Unchanged from the extraction.

## Decisions

### 1. The contract is a leaf crate the engine owns

`deep_causality_context_store` sits at Tier 0 with no dependencies. The context crate depends on
it and implements the projection; a backend depends on it alone.

*Alternative: put the records and trait inside `deep_causality_context`.* Rejected. A backend
would then link the context crate and its seven dependencies to read a handful of enums, and every
release of the algebra or tensor stacks would move the backend's lock file.

*Alternative: let each backend vendor the shapes it reads.* Rejected. A vendored copy learns that
the engine moved only after the engine has moved, and only if someone reruns a comparison. A crate
the engine owns tells the engine first, in its own build.

### 2. Records are monomorphic, closed and named

Every scalar in a record is `f64`; every tick and identifier is `u64`. The storage formats in reach
hold an IEEE double and a 64-bit integer, and a record generic over the scalar would push a
conversion into every backend. Fields are named, never positional: a `[f64; 3]` for a position is
an ordering nobody declared.

There is one record variant per engine variant. `SpaceRecord` has four arms because `SpaceKind`
has four; `SpaceTimeRecord` three because `SpaceTimeKind` has three; `TimeRecord` four because
`TimeKind` has four. A variant added to the engine adds one to the record, moves the crate's major
version, and tells every backend by its build.

### 3. A root is an ordinary record

`NodeRecord::Root` is stored, linked, looked up and hydrated like any other node. Fact 6 is the
reason: nothing in the engine reads a root, so there is nothing to synthesise and no anchor to
join it to. `create_context` takes a name alone, `hydrate` takes a slice alone, and a round trip is
total with no refusal rule about roots.

*Alternative: never store a root; synthesise one on hydration and join it to an anchor the
container names.* This was the mechanism of the design notes, written for a backend whose container
plays the root's role. That backend still never holds a root: it declines `Root` records as its own
policy, the same way a reference-holding backend declines values (Decision 11). The engine-side
mechanism was removing a parameter's worth of machinery for nothing the engine uses.

### 4. `DataRecord` is a value tree

```rust
pub enum DataRecord {
    Number(f64), Count(u64), Integer(i64), Flag(bool), Text(String),
    Reference(SubstrateRef),
    List(Vec<DataRecord>),
    Fields(Vec<(String, DataRecord)>),
}
```

A data node's payload is whatever a model needs (fact 8), and the record has to hold any of them
without the store crate knowing the type. `Fields` is an ordered list of named values, not a map,
so a snapshot stays canonical. A property-graph backend maps `Fields` onto node properties one to
one; a numeric-only backend refuses `Text` as its policy; a graph store that holds references alone
accepts `Reference` and refuses the rest.

*Alternative: three scalar arms and a substrate generic in the value type for everything else.*
Rejected. It made every non-scalar payload need a substrate and a per-substrate implementation,
and it put the payload outside the record vocabulary, where no backend could hold it directly.

### 5. The vocabulary moves down and is re-exported

`RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef` mention nothing but themselves,
are named by the records, and are returned by the context crate's own API (`GeoSpace::datum`,
`Temporal::time_scale`, the edge weight of every graph method). They move into the store crate by
`git mv`, and `deep_causality_context` re-exports them.

*Alternative: duplicate the four enums and map between them.* Rejected: two enums with one meaning
need a mapping nothing but a test pins. *Alternative: move without re-export.* Rejected: a consumer
calling `time_scale()` would need a second crate to name the return type of a method on a type
they already hold. The precedent is `Identifiable`, re-exported from core for the same reason.

### 6. The identifier width is declared in both crates, and the projection pins them equal

The store crate declares `pub type IdentificationValue = u64;` with `ContextId` and
`ContextoidId` as aliases of it. The context crate keeps its own two aliases. The two meet in
`to_record`, which writes an engine identifier into a record field: a widening on one side alone is
a type error in the context crate's build. A consumer that glob-imports both crates and names
`ContextId` gets an ambiguity error and imports one path. Recorded in the `context-id-parameter`
delta.

### 7. `Recordable` is generic over the record, and `to_record` is fallible

```rust
pub trait Recordable<Rec>: Sized {
    fn to_record(&self) -> Result<Rec, ProjectionError>;
    fn from_record(id: ContextoidId, record: Rec) -> Result<Self, ProjectionError>;
}
```

Fact 5 forces the type parameter: `NoSpaceTime<R>` implements `Recordable<SpaceRecord>` and
`Recordable<SpaceTimeRecord>`, one for each slot it fills. `to_record` returns a `Result` for the
same type: a `NoSpaceTime` node in a graph is constructible and has no record, and the honest answer
is `ProjectionError::Unrecordable` naming the node. `Contextoid`'s implementation reaches the
phantom `_Marker` arm the same way instead of panicking.

### 8. `Storable` is the payload's translation, declared beside the node types

```rust
pub trait Storable: Sized {
    fn to_record(&self) -> DataRecord;
    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError>;
}
```

A payload type implements it once and `impl<T: Storable> Recordable<DataRecord> for Data<T>`
plugs it into `snapshot`, `restore`, `store_branch`, `hydrate` and the stream. The context crate
ships implementations for `f32`, `f64`, `u64`, `i64`, `bool`, `String`, `SubstrateRef`,
`Vec<T: Storable>`, `Option<T: Storable>`, `Float106` and `BFloat16`. `Float106` is two `f64`
halves and stores as `Fields { hi, lo }`, so it round-trips exactly; `BFloat16` widens exactly and
restores rounded. A struct with named fields maps to `Fields`.

**The trait lives in `deep_causality_context`, and the orphan rule leaves no choice.** The two
software scalars live in `deep_causality_num`. A trait declared in the store crate could be
implemented for them nowhere: the store crate cannot depend on `num` and stay dependency-free,
`num` must not depend on a persistence crate, and the context crate would be a third party to both
trait and type. Declared in the context crate the trait is local, so the context crate implements
it for the foreign scalars and a user's crate implements it for its own structs. Backends never see
the trait; they see `DataRecord`.

A blanket `impl<R: RealField> Storable for R` cannot stand beside `impl Storable for String`,
because coherence cannot prove `String` will never be a `RealField`. So the scalars get one
implementation each, and a fifth shipped scalar adds one line in the context crate. That is a
stated exception to "zero concrete types named", confined to data payloads.

### 9. Precision is spent at the projection, as a bound

Every scalar-bearing `Recordable` implementation is `impl<R: RealField + Into<f64> +
FromPrimitive>`. The narrowing is `Into<f64>` on the way out and `FromPrimitive::from_f64` on the
way back, stated once on the implementation. A scalar that cannot narrow to a double has no
implementation and cannot reach a store. `from_f64` returning `None` is
`ProjectionError::Scalar { id, value }`. Coordinates and times at `Float106` therefore store as
doubles; data payloads at `Float106` store exactly through Decision 8. Settled by the author.

### 10. Storage is asynchronous without a runtime

Every operation returns `impl Future<Output = Result<…, Self::Error>> + Send`. Plain Rust since
1.75; it states the `Send` a request handler needs, which `async fn` in a trait cannot state; and
it costs no dependency. The in-memory backend returns `core::future::ready`, and tests drive
futures with a poll loop on `Waker::noop()`, stable since 1.85 and free of `unsafe`.

*Alternative: a synchronous trait.* Rejected. Every network backend would block a runtime thread
inside a request handler.

Under edition 2024 a return-position `impl Trait` captures the `&self` lifetime. The in-memory
futures are ready before they are returned and borrow nothing; a backend that awaits inside a
method holds the borrow for the future's life. Documented on the trait.

### 11. A data node in a reference-holding store is a reference, and the substrate holds the value

Which `DataRecord` variants a backend accepts is the backend's policy. A store that holds structure
and not values refuses everything but `Reference`, because a value beside the reference is a second
copy that disagrees the moment either is corrected. The way past that is `Substrate`, monomorphic
over the record:

```rust
pub trait Substrate {
    type Error: core::fmt::Debug + core::fmt::Display;
    fn deposit(&self, node: ContextoidId, value: &DataRecord)
        -> impl Future<Output = Result<SubstrateRef, Self::Error>> + Send;
    fn resolve(&self, reference: &SubstrateRef)
        -> impl Future<Output = Result<DataRecord, Self::Error>> + Send;
}
```

`ContextStore::create_node_via` deposits every non-`Reference` payload and hands the store the
reference; `hydrate_via` resolves every reference before `restore` sees it. Because every storable
payload is already a `DataRecord` (Decision 8), the substrate needs no type parameter and a
deployment writes no per-payload code for it.

### 12. Thirteen operations

```rust
fn reserve(&self, n: usize) -> … IdReserve;
fn create_context(&self, name: &str) -> … ContextId;
fn retract_context(&self, context: ContextId) -> … ();
fn create_node(&self, nodes: &[ContextoidRecord]) -> … ();
fn retract_node(&self, node: ContextoidId) -> … ();
fn create_edge(&self, edges: &[RelationRecord]) -> … ();
fn retract_edge(&self, from: ContextoidId, to: ContextoidId) -> … ();
fn link(&self, context: ContextId, nodes: &[ContextoidId]) -> … ();
fn unlink(&self, context: ContextId, nodes: &[ContextoidId]) -> … ();
fn attach(&self, context: ContextId, extra: ContextId) -> … ();
fn detach(&self, context: ContextId, extra: ContextId) -> … ();
fn lookup(&self, ids: &[ContextoidId]) -> … Vec<Option<ContextoidRecord>>;
fn hydrate(&self, spec: &Self::Slice) -> … ContextSnapshot;
```

The design notes listed ten. `lookup` is added because `store_branch` must know, for every node of
a branch, whether the store holds it, holds it under the same record, or under a different one; a
refusal from `create_node` arrives as an opaque error after the fact, and one read by identifier
answers before the plan. `attach` and `detach` are added for Decision 13.

### 13. A stored context references other contexts, and hydration materialises them

Every context is a container. An extra context in the store is a reference from one container to
another, created by `attach`, removed by `detach`, and dropped in both directions by
`retract_context`. Many containers may reference one, one may reference many, so the references
form an M:N relation; cycles are allowed. `hydrate` of a container materialises each referenced
container into an `ExtraContextSnapshot` carrying that container's identifier, name, nodes and
edges, and `restore` plugs each into the extra-context map under the referenced container's own
identifier and name. Materialisation is one level deep: the engine's extras are flat graphs, so a
referenced container's own references are not followed.

Three consequences:

- **An extra has a name**, because a container has one and an extra is a container. This changes
  `ExtendableContextuableGraph`: `extra_ctx_add_new(name, capacity, default)`,
  `extra_ctx_add_new_with_id(id, name, capacity, default)` and `extra_ctx_get_name(id)`. The name
  is the container's, so it is the same in every context that references it; a per-reference alias
  would let two contexts call one container two things.
- **A hydrated extra carries the store's identifier.** In memory an extra is numbered locally; in
  the store every container has one identifier unique across the store. Keying the extra by the
  store's identifier means an event naming that container routes to the right graph with no
  translation table, and two engines hydrating one container agree on its number. The engine's own
  numbering survives only for extras built in memory and never stored.
- **`extra_ctx_add_new` allocates past any held identifier** (fact 4). A hydrated context holds
  store identifiers the count-based allocator would reach and panic on. Sequential numbering is
  unchanged. Identifier 0 is refused for an extra, because 0 is the engine's "no extra is current"
  sentinel.

A slice therefore names one container; the composition is stored, so the composite slice of the
design notes is unnecessary.

### 14. `store_branch` stores a world as a new context and returns its identifier

`store_branch(name, branch)` snapshots the branch, looks up every node, creates what the store does
not hold, re-creates under a fresh reserved identifier every node the store holds under a different
record and remaps the branch's edges to it, creates the edges, creates the container, links every
node, and returns the container's identifier. Each extra of the branch becomes a new container
under the extra's own name, filled the same way, and is attached to the new one. Nodes are shared or
re-created by the node rule; containers are cheap, so no comparison against the store is made to
re-attach an existing one. A stored branch is a record of a world, not a continuation of one: to
keep exploring it, hydrate it. Settled by the author.

### 15. A subscription's scope is fixed; a wider scope is a new subscription

`subscribe(slice, from)` returns the container's snapshot and a stream of every change to it and
to the containers it references, from that point. The scope is fixed for the life of the stream. A
container that comes into existence afterwards, by this engine's `store_branch` or by any writer,
is outside every existing subscription. The host reaches it by subscribing again with a slice that
names it, passing the cursor it has reached, so no event between the two subscriptions is lost.
Settled by the author.

*Alternative: a stream handle that widens its own standing query and delivers the new container's
contents in-band.* Rejected for now: a new operation, a new event variant and backend support for
mutating a standing query, to save one hydration.

### 16. Every event names what it changes, and a membership event carries the record

`ContextEvent` has one variant per mutating operation and two for a view's answer moving. Three
corrections to the design notes, each from tracing what `Context::apply` needs:

- `NodeLinked { context, node: ContextoidRecord }` and `NodeEntered { context, node }`. A
  subscriber adding a node to a hydrated graph needs the payload; an event with only an identifier
  forces either a journal of every `NodeCreated` seen or a `lookup` round trip inside `apply`.
  `NodeCreated` therefore changes nothing on a `Context`: creation is a fact about the store,
  membership is a fact about a context.
- `apply` and `apply_batch` take the event alone. Membership and container events already name
  their container; node and edge events belong to none.
- `ContextAttached { context, extra: ContextRecord }` and `ContextDetached { context, extra }` join
  the vocabulary for Decision 13.

`Context::apply` routes by the container an event names: the held context's identifier is the base
graph, an extra's identifier is that extra, anything else is `ProjectionError::Identity`. Every
path is idempotent, so an echo of the host's own write lands on a state it already produced.
`ContextAttached` on the held context adds an empty extra under the attached container's
identifier and name and is the host's cue to resubscribe (Decision 15); `ContextDetached` drops
the extra; `ContextRetracted` of the held context itself is refused.

Events are applied between evaluations, never during one. The host drains the stream when no
evaluation runs, evaluates, then sends what it decided to keep. There is no journal on the engine
side: a host that knows what it changed calls `apply`; a host that explored a branch calls
`store_branch`. Settled by the author.

### 17. The in-memory backend lives in the store crate's `utils_test`

It references records alone, so it belongs beside them, and it is then reachable by both crates'
tests. Its state is a fold over its own event log: every mutating operation appends the events it
emits, `subscribe` from a cursor replays the log to that position into a fresh state, and
`apply_batch` applies to a clone and commits on success. A `std::sync::Mutex` gives interior
mutability behind `&self`; a poisoned lock is recovered with `PoisonError::into_inner`. Its slice is
a `ContextId`. `MemorySubstrate` implements `Substrate` over a map keyed by the reference's `key`.
`block_on` polls a future to completion on `Waker::noop()`. All of it is tested to the coverage
floor, because `utils_test` counts toward the crate's coverage.

### 18. Errors

Every error type follows the repository's convention, as `PhysicsError` shows it: a public tuple
struct around a public enum (`ProjectionError(pub ProjectionErrorEnum)`), one constructor
function per variant, a `kind()` accessor, and `Display` and `std::error::Error` on the struct, so
the classification can grow without the struct changing.

`ProjectionErrorEnum` in the store crate: `WrongVariant { id, expected, found }`, `WrongPayload {
id, expected, found }`, `MissingField { id, field }`, `Unrecordable { id, kind }`, `Scalar { id,
value }`, `Identity { id, rule }`, `Version { found, supported }`. `MissingField` is for a
`Storable` implementation reading a `Fields` record that lacks a name it expects.

`StoreError<E, B = core::convert::Infallible>` in the context crate wraps `StoreErrorEnum<E, B>`:
`Storage(E)`, `Substrate(B)`, `Projection(ProjectionError)`. The default makes the plain paths read
`StoreError<S::Error>`. The in-memory backend's `MemoryStorageError` wraps `MemoryStorageErrorEnum`
the same way.

### 19. Snapshots are canonical

`snapshot` orders nodes by identifier, edges by `(from, to)`, extras by identifier, and the
`Fields` of a data record as the `Storable` implementation wrote them. Two contexts with the same
content produce equal snapshots regardless of insertion order, which is what lets a test assert
`restore(snapshot(c)).snapshot() == snapshot(c)` without a `PartialEq` on `Context`.

### 20. Backend metadata is a property of the handle

Who wrote a change, when, and under what authority are questions a backend may answer, and the
trait says nothing about them. A backend that records them takes them at construction.

### 21. Module layout

Store crate:

```
src/alias/mod.rs                 IdentificationValue, ContextId, ContextoidId
src/constants/mod.rs             RECORD_VERSION
src/errors/projection_error.rs
src/traits/{context_storage,context_storage_stream,context_events,recordable,substrate}.rs
src/types/{relation_kind,time_scale,vertical_datum,substrate_ref}/mod.rs   (git mv, mv)
src/types/records/{context_record,contextoid_record,node_record,time_record,space_record,
                   space_time_record,relation_record,extra_context_snapshot}.rs
src/types/records/data_record/mod.rs
src/types/records/context_snapshot/mod.rs
src/types/id_reserve/mod.rs
src/types/context_event/mod.rs
src/utils_test/{block_on.rs, memory_storage/…, memory_substrate.rs}
```

Context crate additions:

```
src/traits/storable/mod.rs                                 Storable
src/extensions/storable/{scalars,software_scalars,text,collections,reference}.rs
src/types/context_node_types/**/recordable.rs              one per implementing type
src/types/context_node_types/data/recordable.rs            the Storable blanket
src/types/context_types/contextoid/recordable.rs
src/types/context_types/context_graph/{snapshot,restore,apply}.rs
src/types/context_types/context_graph/extra_context.rs     the named extra
src/types/context_types/context_store/{mod,hydrate,store_branch,substrate,subscribe}.rs
src/errors/store_error.rs
src/alias/mod.rs                                           SubstrateContext, SubstrateContextoid
```

Tests mirror both trees. The context crate's `BUILD.bazel` loses the three vocabulary suites and
gains `ctx_types_context_store_tests` and `traits_storable`; the store crate's `BUILD.bazel`
declares one suite per test folder.

## Risks / Trade-offs

- [The first asynchronous library surface meets edition-2024 lifetime capture] → The trait
  documents that a returned future may hold the `&self` borrow; a test pins that a `ContextStore`
  over the in-memory backend is usable across two awaits.
- [Coordinates at `Float106` lose precision through a store] → Stated on the projection bound and
  in the READMEs; data payloads at `Float106` do not, through `Fields { hi, lo }`.
- [`Storable` names the shipped scalars concretely] → Confined to data payloads and forced by
  coherence; the exception is stated in Decision 8 and in the crate README.
- [Twenty `Recordable` implementations are repetitive] → Each is pinned by a round-trip test at a
  value where every field differs, and each concrete type by a wrong-variant test naming the node.
- [A hydrated context's extra identifiers differ from the ones it had before storing] → Only for
  extras that were stored; documented on `store_branch` and `hydrate`, and the allocator no longer
  collides with them.
- [A no-dependency crate under `all_crate_deps`] → resolves to an empty list; test suites still
  link `:deep_causality_context_store`.
- [Two `ContextId` aliases] → Ambiguity only under a double glob import; see Decision 6.
- [The extra-context allocator changes its answer after an explicit insertion] → Only the case
  that panicked today changes; a test that asserted the count-based number after `_with_id` was
  asserting the defect and is corrected with it.

## Migration Plan

Additive for every consumer except the four context-crate test files that call `extra_ctx_add_new`
or `extra_ctx_add_new_with_id`, which gain a name argument. The vocabulary relocation is transparent
through the re-exports; the context crate's manifest gains one dependency. The store crate publishes
first at 0.1.0, then the context crate with the dependency and the breaking footer, both through
release-plz. Rollback is removing the dependency and the `git mv`, which git reverses.

## Open Questions

Every design question raised while deriving this change was settled with the author on 2026-09-22
and is recorded in the decisions above. One recommendation remains unanswered and is not part of
the change: a `conformance` module in the store crate's `utils_test`, generic over any
`ContextStorage`, that runs every invariant scenario so an external backend can prove itself with
the same suite the in-memory backend passes.
