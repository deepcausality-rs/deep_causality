# deep_causality_context_store

The persistence contract for the [DeepCausality](https://deepcausality.com/) context: the records
a `Context` projects onto, the storage trait a backend implements, and the vocabulary both sides
name. Written for someone implementing a backend.

## What this crate is for

`deep_causality_context` holds a `Context` in memory. A store that keeps one needs a description of
a context that does not change when the engine's structs change. This crate is that description.
It depends on nothing, so a backend links this crate alone and never the context crate or its
dependencies. The context crate depends on this one and does the projection; a backend never sees
a `Context`, only records.

```toml
[dependencies]
deep_causality_context_store = "0.1"
```

## The records

A context is nodes, edges among them, and references to other contexts. Every identifier is a
`u64` the store hands out (`ContextId` for a container, `ContextoidId` for a node).

| Record | Holds |
|---|---|
| `ContextRecord` | a container: identifier and name |
| `ContextoidRecord` | a node: identifier and `NodeRecord` |
| `RelationRecord` | an edge: `from`, `to`, `RelationKind` |
| `ContextSnapshot` | one container as of a consistent point: version, `ContextRecord`, nodes, edges, extras |
| `ExtraContextSnapshot` | a referenced container as the snapshot carries it: identifier, name, nodes, edges |

`NodeRecord` has one variant per kind of node: `Root`, `Data(DataRecord)`, `Time(TimeRecord)`,
`Space(SpaceRecord)` and `SpaceTime(SpaceTimeRecord)`. `SpaceRecord` is `Geo`, `Ecef`,
`Euclidean` or `Ned`; `TimeRecord` is `Euclidean`, `Lorentzian`, `Discrete` or `Entropic`;
`SpaceTimeRecord` is `Euclidean`, `Lorentzian` or `Tangent`, the last carrying the tangent vector
and the local metric. Every variant is a struct variant with named fields, so a backend that maps
records to columns or properties reads the names off the type.

`DataRecord` is a value tree:

```text
Number(f64) | Count(u64) | Integer(i64) | Flag(bool) | Text(String)
Reference(SubstrateRef)
List(Vec<DataRecord>)
Fields(Vec<(String, DataRecord)>)
```

A scalar payload is one leaf, a sequence is a `List`, a struct is `Fields` with one entry per
field, and `Reference` names a value that lives outside the store (see Substrate below). A
backend stores the tree as it is; it never interprets it.

`ContextSnapshot::version()` is the record layout's version, `RECORD_VERSION`. A backend stores
the number with the snapshot and hands it back; the context crate refuses a version newer than
its own.

## Precision

Every scalar field in a record is `f64` and every tick is `u64`. The context crate's node types
carry their scalar as a type parameter, and the projection narrows it to `f64` on the way out and
lifts it on the way in. A backend therefore stores one scalar width. The one exception is a data
payload of `Float106`, which the context crate stores as `Fields [("hi", Number), ("lo", Number)]`
and restores to every bit; a backend sees two ordinary numbers.

## The fourteen operations

`ContextStorage` is the trait a backend implements. Every operation returns
`impl Future<Output = Result<_, Self::Error>> + Send`, so a backend that awaits a network never
blocks, and a synchronous backend returns `core::future::ready`. The trait brings no runtime.

| Operation | Effect |
|---|---|
| `reserve(n)` | `n` identifiers made unique by the store, as an `IdReserve`; a lease with no return |
| `create_context(name)` | a container under a name; returns its identifier, never 0 and never reused |
| `retract_context(id)` | the container, its links and its references go; every node and every referenced container survives |
| `create_node(&[ContextoidRecord])` | nodes under reserved identifiers |
| `retract_node(id)` | the node leaves every container, with every edge incident to it |
| `create_edge(&[RelationRecord])` | relations between nodes the store holds |
| `retract_edge(from, to)` | one relation goes |
| `link(context, &[id])` | nodes join a container |
| `unlink(context, &[id])` | nodes leave a container |
| `attach(context, extra)` | a container references another |
| `detach(context, extra)` | the reference goes |
| `commit(&[ContextWrite])` | the writes in order, all or none; returns the identifiers of the containers it created |
| `lookup(&[id])` | the record under each identifier, or `None` |
| `hydrate(&slice)` | one container as a `ContextSnapshot`, its references materialised one level deep |

Every backend keeps these invariants; the trait's documentation states each in full:

- **A context is a set of links.** `link` and `unlink` change membership and nothing else.
- **A node is immutable under its identifier.** `create_node` is idempotent for the same record
  and refused for a different one. A changed value is a new node under a fresh identifier.
- **Identity is the store's.** `create_node` is refused for an identifier that no `reserve` handed
  out.
- **A relation exists once between two nodes.** `create_edge` is idempotent for an identical edge,
  refused for a different kind between the same pair, and refused when either end is not held.
- **A reference is a link between containers.** `attach` is idempotent, refused for a self-reference
  and for a container the store does not hold; `detach` of a reference not held is not an error.
  References are many to many and may form a cycle.
- **A commit is all or nothing.** `commit` performs a sequence of `ContextWrite`s (`CreateContext`,
  `CreateNode`, `CreateEdge`, `Link`, `Attach`) under each operation's own refusals, or none of
  them. A `ContainerRef::Created(i)` names the container the commit's `i`-th `CreateContext` made,
  so a commit can link into and attach a container it creates. A refused commit leaves the store
  as it was and reports no event.
- **`hydrate` materialises one level.** The snapshot holds the container's nodes, the edges among
  them, and one `ExtraContextSnapshot` per container it references. Their own references are not
  followed.
- **`Root` is an ordinary record.** A backend that cannot hold one declines it with its own error
  and documents that.

## References between containers

Every context is a container. What the context crate shows as an extra context is, in the store, a
reference from one container to another, made by `attach` and removed by `detach`. Hydrating a
container brings each referenced container along under its own identifier and name, and the
context crate restores it as an extra under that identifier. One container may be referenced by
many, so a node linked into a shared container appears in every context that references it.

## Implementing a backend

A backend is a type that implements `ContextStorage` with two associated types:

- `Error`: the backend's own refusals. It must be `Debug + Display`. A projection failure never
  reaches a backend; the context crate reports it as `ProjectionError` on its side.
- `Slice`: what names a context to hydrate. The in-memory backend uses `ContextId`; a backend
  with its own query language may use a query.

Identifiers come from the store: `reserve` hands out node identifiers once, and a container's
identifier is never 0 and never reused.

Two further traits are optional, and a backend that implements neither is complete:

- `ContextStorageStream: ContextStorage` adds `subscribe(slice, from)`, which returns a snapshot
  and a stream of every `ContextEvent` to the named container and the containers it referenced at
  that moment, from that point on, and `apply` / `apply_batch`, which perform the operation an
  event names and return the cursor after it. The scope of a subscription is fixed for its life; a
  container created or attached afterwards is reached by a new subscription from the cursor
  reached. `ContextEvent` has one variant per mutating operation plus `NodeEntered` and `NodeLeft`
  for a view whose answer moves.
- `Substrate` is for a store that holds structure and not values. `deposit(node, &record)` stores
  a value and returns a `SubstrateRef`; `resolve(&reference)` returns the value. `deposit` is
  idempotent per node: a deposit under a node replaces the value held for it and returns the same
  reference, so the substrate holds at most one value per node. The context crate's
  `create_node_via` and `hydrate_via` run every data payload through it, so the store sees only
  `DataRecord::Reference`.

## The in-memory backend

`utils_test` ships `MemoryStorage`, `MemoryEvents` and `MemorySubstrate`, an in-memory
implementation of all three traits, and `block_on`, which polls a future to completion with no
runtime. They exist for tests on both sides of the contract, and their tests state the invariants
above as behaviour a backend can compare against.

## Contents

* `ContextStorage`, `ContextStorageStream`, `ContextEvents`, `Substrate` and `Recordable`.
* `ContextRecord`, `ContextoidRecord`, `RelationRecord`, `ContextSnapshot`, `ExtraContextSnapshot`,
  `NodeRecord`, `DataRecord`, `SpaceRecord`, `TimeRecord`, `SpaceTimeRecord`, `ContextEvent`,
  `ContextWrite`, `ContainerRef` and `IdReserve`.
* `ContextId`, `ContextoidId`, `IdentificationValue` and `RECORD_VERSION`.
* `ProjectionError`, `MemoryStorageError` and `MemorySubstrateError`.
* `RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef`, the vocabulary the records and
  the context node types share. `deep_causality_context` re-exports all four.

## Licence

MIT. See [LICENSE](LICENSE).
