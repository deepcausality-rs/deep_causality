## Why

A `Context` exists only in the process that built it. `deep_causality_context` has no operation
that writes a context out or reads one back, so a context that must outlive its process, be shared
between processes, or be read later has no supported path. A persistence layer written today would
have to target the engine's structs directly, and those structs have moved in nine breaking commits
since the crate was extracted: the scalar became a parameter, the value parameters became associated
types, the symbolic dimension went, the quaternion space went, `MinkowskiSpacetime` merged into
`LorentzianSpacetime`, the datum landed on `GeoSpace`, and the identifiers were taken over. A reader
written against the old shape compiles unchanged and writes a node without its datum. The failure
is discovered by the next reader, not by the build.

The fix is a contract the context crate owns and compiles against: a record vocabulary, a storage
trait any backend implements, a projection from the node types onto the records, and a store that
composes the three. When a node type gains a field, the projection beside it stops compiling, and
the engine says what the record has to say about it before anything downstream learns of the
change.

## What Changes

- **New crate `deep_causality_context_store` 0.1.0** at the repository root, Tier 0, with no
  dependencies of any kind. It declares the record types (`ContextSnapshot`, `ContextRecord`,
  `ContextoidRecord`, `NodeRecord`, `DataRecord`, `TimeRecord`, `SpaceRecord`, `SpaceTimeRecord`,
  `RelationRecord`, `ExtraContextSnapshot`), `IdReserve`, `RECORD_VERSION`, the identifier aliases
  `IdentificationValue`, `ContextId` and `ContextoidId`, the `ProjectionError` type, and five traits:
  `ContextStorage`, `Recordable`, `Substrate`, `ContextEvents` and `ContextStorageStream`. It ships
  an in-memory backend under `utils_test` and a runtime-free `block_on` for driving the asynchronous
  contract in tests.
- **`DataRecord` is a value tree, not three scalars.** `Number`, `Count`, `Integer`, `Flag`, `Text`,
  `Reference`, `List` and `Fields` let a data payload of any shape, a scalar, a time series or a
  sensor reading with named fields, reach a store without the store crate knowing the type.
- **Four vocabulary types move down.** `RelationKind`, `TimeScale`, `VerticalDatum` and
  `SubstrateRef` are declared in the store crate, because the records name them and the store crate
  depends on nothing. `deep_causality_context` re-exports all four, so no consumer import changes.
  `SubstrateRef` exists today only as an untracked, unregistered draft in the context crate and is
  moved rather than duplicated.
- **`deep_causality_context` depends on the store crate** and gains the projection: a `Storable`
  trait that a data payload implements once to become persistable, with implementations for the
  primitives, the collections and the two software scalars; `Recordable` implementations for the
  three `Kind` enums, the eleven concrete space, time and spacetime types, `NoSpaceTime`, every
  `Data<T: Storable>` and `Contextoid`; `Context::snapshot`, `Context::restore` and
  `Context::apply`; the `ContextStore<S>` type with `hydrate`, `reserve`, `store_branch`,
  `create_node_via`, `hydrate_via` and `subscribe`; `StoreError`; and the `SubstrateContext` /
  `SubstrateContextoid` aliases naming the shape a reference-holding store accepts.
- **BREAKING: extra contexts take a name.** `extra_ctx_add_new` and `extra_ctx_add_new_with_id`
  gain a `name` parameter and `ExtendableContextuableGraph` gains `extra_ctx_get_name`. A stored
  extra context is a container of its own, referenced by name and identifier, and an unnamed extra
  had nothing to be referenced by. The only callers are four test files in the context crate.
- **The extra-context allocator stops colliding.** `extra_ctx_add_new` derives the next identifier
  from the highest identifier the context holds rather than from the count of extra contexts. Today
  `extra_ctx_add_new_with_id(7)` followed by six calls to `extra_ctx_add_new` panics on the seventh,
  and a hydrated context, whose extras carry store identifiers, would panic on its first
  allocation. Numbering in the sequential case is unchanged.
- **The first asynchronous surface in a library crate.** Every storage operation returns
  `impl Future<Output = …> + Send`, with no runtime dependency. A network backend cannot block a
  request thread, and a synchronous trait would force every such backend to.
- **Out of scope, stated so it is not assumed:** any concrete persistent backend. A file store, a
  database store or a network store is a separate crate over `ContextStorage`, outside this
  repository. `UncertainData`, `UncertainBoolData`, `SymbolicTime` and the symbolic spacetimes get
  no record; `no_std` for the store crate is not pursued.

## Capabilities

### New Capabilities

- `context-store-crate-identity`: the crate, its position, its empty dependency list, its standard
  infrastructure, the vocabulary it takes over and the identifier aliases it declares.
- `context-store-records`: the record vocabulary including the `DataRecord` value tree, the
  snapshot, the identifier reserve, the record version and the projection error.
- `context-storage-contract`: the `ContextStorage` and `Substrate` traits, their thirteen plus two
  operations, their asynchronous form and the invariants every backend keeps.
- `context-projection`: `Recordable` and `Storable`, their implementations in the context crate,
  where the projection is total and where it fails loudly, the scalar narrowing, and
  `Context::snapshot` / `restore`.
- `context-named-extra-contexts`: the name on an extra context, the allocator rule, and how a
  stored context references others.
- `context-store-facade`: `ContextStore<S>`, the branch rule, the substrate extensions,
  `StoreError` and the persisted-shape aliases.
- `context-store-streaming`: `ContextEvent`, `ContextEvents`, `ContextStorageStream`, the fixed
  scope of a subscription, `Context::apply` and `ContextStore::subscribe`.
- `context-store-memory-backend`: the in-memory backend and the `block_on` helper under
  `utils_test`, which is how both crates test the contract without a network.

### Modified Capabilities

- `context-crate-identity`: the crate no longer declares `RelationKind` and `TimeScale` (it
  re-exports them), its dependency list gains `deep_causality_context_store`, and the list of
  capabilities that deliberately change a signature gains `context-named-extra-contexts`.
- `context-id-parameter`: the store crate declares the identifier aliases the records carry, so
  the context crate is no longer the only declaration site; the two are pinned equal by the
  projection.

## Impact

- **New crate:** `deep_causality_context_store/` with `Cargo.toml`, `BUILD.bazel`, README, SBOM
  pair, `src/` and `tests/`. Root `Cargo.toml` gains a member and a `[workspace.dependencies]`
  entry at two-digit precision.
- **`deep_causality_context`:** one new dependency; three `git mv` of vocabulary modules and their
  tests; roughly fifty new source files (the projection, `Storable` and its implementations, the
  three `Context` methods, the store, the errors, the aliases) and their tests; the named-extra
  change to `ExtendableContextuableGraph` and its four test files; `BUILD.bazel` loses three test
  suites and gains one.
- **No other crate changes.** `deep_causality`, `deep_causality_ethos` and the examples import the
  vocabulary through the context crate, call no `extra_ctx_*` method, and are unaffected.
- **Documentation:** `AGENTS.md` (crate count, root list, tier block), both crate READMEs.
- **Toolchain:** `impl Future` in trait return position and `Waker::noop()` are stable well below
  the workspace floor of 1.98.
- **Versions** are left to release-plz. The store crate carries an explicit 0.1.0 because a crate
  cannot publish without one, and it publishes before the context crate.
