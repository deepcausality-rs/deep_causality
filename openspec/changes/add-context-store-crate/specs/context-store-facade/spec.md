## ADDED Requirements

### Requirement: `ContextStore<S>` composes projection and storage

`deep_causality_context` SHALL declare `pub struct ContextStore<S: ContextStorage>` with
`pub const fn new(storage: S) -> Self`, `pub const fn storage(&self) -> &S`, and the asynchronous
methods `hydrate`, `reserve`, `store_branch`, `create_node_via` and `hydrate_via`, each returning
`Result<_, StoreError<…>>`.

`StoreError<E, B = core::convert::Infallible>` has the variants `Storage(E)`, `Substrate(B)` and
`Projection(ProjectionError)`, and implements `Debug`, `Display` and `std::error::Error` where its
parameters do.

#### Scenario: The store is thin

- **WHEN** a caller uses the storage directly through `storage()` alongside the store's own methods
- **THEN** both paths see the same backend state, because the store holds the backend by value and
  adds no cache

### Requirement: `hydrate` restores the container a slice names

`ContextStore::hydrate<D, S_, T, ST>(&self, spec: &S::Slice)` SHALL call the backend's `hydrate`
and pass the snapshot to `Context::restore`, so the context holds the container's nodes and edges
as its base graph and each referenced container as an extra under that container's identifier and
name.

#### Scenario: A stored context with references hydrates whole

- **WHEN** a container holding a root, two data nodes and an edge, and referencing a second
  container of three nodes, is hydrated
- **THEN** the context's base graph holds the four nodes and the edge, and one extra under the
  second container's identifier and name holds the three nodes

### Requirement: `store_branch` stores a world as a new context

`store_branch` SHALL store a branch as a new context that links what it shares and creates what it
changed. `ContextStore::store_branch<D, S_, T, ST>(&self, name: &str, branch: &Context<D, S_, T,
ST>) -> Result<ContextId, _>` snapshots the branch, looks up every node, creates what the store
does not hold, re-creates under a fresh reserved identifier every node the store holds under a
different record, remaps the branch's edges accordingly, creates the edges, creates the container
under `name`, links every node, and returns the container's identifier. Each extra of the branch is
stored the same way as a new container under the extra's own name and attached to the new one.

The in-memory branch is unchanged by the call. A stored branch is a record of a world, not a
continuation of one; to keep exploring it, hydrate it.

#### Scenario: A stored branch hydrates equal

- **WHEN** a context is hydrated, extended with two nodes from a reserve and one edge, stored with
  `store_branch`, and the returned container is hydrated
- **THEN** the second hydration's snapshot equals the extended branch's snapshot up to the
  extras' identifiers, which are the new containers'

#### Scenario: A changed value is a new node

- **WHEN** a hydrated branch replaces node 5's data in place through `update_node` and is stored
- **THEN** the store still holds the original record under 5, holds the changed record under a
  fresh identifier, the new container links the fresh identifier and not 5, every edge the branch
  had at 5 is stored at the fresh identifier, and the origin container is unchanged

#### Scenario: A shared node is linked, not copied

- **WHEN** a branch is stored whose nodes are all held by the store under the same records
- **THEN** `create_node` is called with an empty slice or not at all, and the new container links
  every node

#### Scenario: Extras become attached containers

- **WHEN** a branch with two named extras is stored
- **THEN** the store holds two new containers under those names, each attached to the new base
  container, and hydrating the base yields both as extras

### Requirement: `reserve` is a pass-through

`ContextStore::reserve(&self, n: usize)` SHALL return the backend's reserve wrapped in
`StoreError::Storage` on failure.

#### Scenario: A reserve outlives a branch

- **WHEN** a reserve of ten is taken, a branch uses three and is scrapped, and another branch uses
  three more and is stored
- **THEN** the stored branch's new nodes carry the fourth to sixth identifiers and `create_node`
  accepts them

### Requirement: The substrate extensions externalise values on the way in and internalise them on the way out

`ContextStore` SHALL provide `create_node_via` and `hydrate_via`, so that a value-holding context
reaches a store that holds references alone. `create_node_via<B: Substrate>(&self, substrate: &B,
nodes: &[ContextoidRecord])` deposits every data payload that is not already a `Reference` and
replaces it by the returned reference before `create_node` sees it. `hydrate_via<B: Substrate, D,
S_, T, ST>(&self, substrate: &B, spec: &S::Slice)` resolves every `Reference` payload into the
record the substrate returns before `restore` sees it. Both return `StoreError<S::Error,
B::Error>`.

#### Scenario: A value-typed context reaches a reference-holding store

- **WHEN** a `BaseContext` with two data nodes is snapshotted, its nodes are created through
  `create_node_via` over the in-memory substrate, and the container is hydrated back through
  `hydrate_via` as a `BaseContext`
- **THEN** the data nodes hold their original `f64` values, and the store's `lookup` shows
  `Reference` payloads for both

#### Scenario: A struct payload reaches a reference-holding store

- **WHEN** a context over a `Data<_>` whose payload is a user struct with a `Storable`
  implementation goes through `create_node_via` and `hydrate_via`
- **THEN** the struct comes back equal, and the graph held only references

#### Scenario: A payload the type cannot hold is loud

- **WHEN** a container whose substrate resolves a reference to a `Count` is hydrated through
  `hydrate_via` as a `BaseContext`
- **THEN** the result is `Err(StoreError::Projection(ProjectionError::WrongPayload { .. }))` naming
  the node

### Requirement: The persisted shape has a name

`deep_causality_context` SHALL declare `SubstrateContext` as `Context<Data<SubstrateRef>,
SpaceKind<f64>, TimeKind<f64>, SpaceTimeKind<f64>>` and `SubstrateContextoid` as the matching
`Contextoid`, the four parameters for which the projection is total and whose data node holds a
reference.

#### Scenario: The alias snapshots without a substrate

- **WHEN** a `SubstrateContext` holding one node of each kind is snapshotted and its nodes are
  created through the plain `create_node`
- **THEN** every call succeeds without a substrate
