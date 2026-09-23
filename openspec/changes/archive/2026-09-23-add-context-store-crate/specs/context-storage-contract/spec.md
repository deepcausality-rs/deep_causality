## ADDED Requirements

### Requirement: `ContextStorage` declares thirteen asynchronous operations

The store crate SHALL declare `ContextStorage` with `type Error: Debug + Display`, `type Slice`,
and thirteen operations, each returning `impl Future<Output = Result<…, Self::Error>> + Send`:

```rust
fn reserve(&self, n: usize) -> … Result<IdReserve, _>;
fn create_context(&self, name: &str) -> … Result<ContextId, _>;
fn retract_context(&self, context: ContextId) -> … Result<(), _>;
fn create_node(&self, nodes: &[ContextoidRecord]) -> … Result<(), _>;
fn retract_node(&self, node: ContextoidId) -> … Result<(), _>;
fn create_edge(&self, edges: &[RelationRecord]) -> … Result<(), _>;
fn retract_edge(&self, from: ContextoidId, to: ContextoidId) -> … Result<(), _>;
fn link(&self, context: ContextId, nodes: &[ContextoidId]) -> … Result<(), _>;
fn unlink(&self, context: ContextId, nodes: &[ContextoidId]) -> … Result<(), _>;
fn attach(&self, context: ContextId, extra: ContextId) -> … Result<(), _>;
fn detach(&self, context: ContextId, extra: ContextId) -> … Result<(), _>;
fn lookup(&self, ids: &[ContextoidId]) -> … Result<Vec<Option<ContextoidRecord>>, _>;
fn hydrate(&self, spec: &Self::Slice) -> … Result<ContextSnapshot, _>;
```

No operation name repeats "persistent" or "store"; the crate's name says every operation is.
The trait adds no runtime dependency; the `Send` bound is what a request handler needs and what
`async fn` in a trait cannot state. The trait's documentation states that under edition 2024 a
returned future may hold the `&self` borrow for its life.

#### Scenario: A backend compiles against the trait with no runtime

- **WHEN** the in-memory backend implements every operation by returning `core::future::ready`
- **THEN** it compiles with the store crate's empty dependency list, and a test drives each
  operation to completion with the crate's `block_on`

#### Scenario: The futures are `Send`

- **WHEN** a generic function bounded `S: ContextStorage + Sync` moves each operation's future into
  a `fn assert_send<F: Send>(f: F)` call
- **THEN** it compiles

### Requirement: Every backend keeps the store invariants

A backend implementing `ContextStorage` SHALL keep these invariants, and the in-memory backend
SHALL pin each with a test:

- `reserve(n)` returns `n` identifiers, none of which the store holds or has handed out before.
- `create_node` is refused for an identifier the store did not hand out and for an identifier
  already created under a different record. It is idempotent for an identifier already created
  under the same record. A `Root` record is created like any other; a backend that cannot hold a
  root declines it with its own error and documents the policy.
- `create_edge` is refused when either end is not held. It is idempotent for an identical
  `(from, to, kind)` and refused for a different `kind` between the same pair, because a relation
  exists once between two nodes.
- `create_context` returns an identifier that is never 0 and never reused.
- `retract_context` retracts the container, its links and every reference to it or from it; every
  contextoid it linked and every container it referenced survives.
- `retract_node` retracts the node, removes it from every container that linked it, and removes
  every edge incident to it.
- `link` is idempotent and refused for a node or a container the store does not hold; `unlink` of
  a node not linked is not an error.
- `attach` is idempotent, refused for a container the store does not hold and for a self-reference;
  `detach` of a reference not held is not an error. A container may be referenced by many and may
  reference many, and a cycle of references is permitted.
- `lookup` returns, for each identifier in order, the record held under it or `None`.
- `hydrate` returns the container named by the slice with its nodes and the edges among them, and
  one `ExtraContextSnapshot` per container it references, carrying that container's identifier,
  name, nodes and edges. References of the referenced containers are not followed.

#### Scenario: An identifier not from a reserve is refused

- **WHEN** `create_node` is called with an identifier the backend never handed out
- **THEN** it returns the backend's error and `lookup` of that identifier returns `None`

#### Scenario: A node is immutable under its name

- **WHEN** a node is created under a reserved identifier and `create_node` is called again with
  the same identifier and a different record
- **THEN** the second call is refused and `lookup` still returns the first record

#### Scenario: A node survives its container

- **WHEN** two containers link one node and one of the containers is retracted
- **THEN** `lookup` still returns the node and `hydrate` of the other container still holds it

#### Scenario: A retracted node leaves every container

- **WHEN** a node linked from two containers and joined by an edge is retracted
- **THEN** `hydrate` of either container holds neither the node nor the edge

#### Scenario: A reference materialises one level

- **WHEN** container A references B, B references C, and A is hydrated
- **THEN** the snapshot's extras hold exactly B with B's name, nodes and edges, and not C

#### Scenario: References are many to many and may cycle

- **WHEN** A and B both reference C, and C references A
- **THEN** every `attach` succeeds, `hydrate` of A holds C as an extra, `hydrate` of B holds C, and
  `hydrate` of C holds A

#### Scenario: A retracted container leaves no dangling reference

- **WHEN** A references B and B is retracted
- **THEN** `hydrate` of A holds no extra, and A's nodes are unchanged

### Requirement: `Substrate` holds values and hands back references

The store crate SHALL declare `Substrate` with `type Error: Debug + Display` and two operations
returning `impl Future<…> + Send`: `deposit(&self, node: ContextoidId, value: &DataRecord) ->
Result<SubstrateRef, _>` and `resolve(&self, reference: &SubstrateRef) -> Result<DataRecord, _>`.

`deposit` is given any `DataRecord` other than a `Reference`; a substrate refuses a `Reference`
with its own error. `resolve` returns the record the reference names and refuses a reference it
does not hold. The store crate declares the trait and implements it only in `utils_test`.

#### Scenario: A value round-trips through a substrate

- **WHEN** a `Fields` record with three entries is deposited under node 7 and the returned
  reference is resolved
- **THEN** the result equals the deposited record

#### Scenario: A reference is not a value

- **WHEN** a `DataRecord::Reference` is passed to `deposit`
- **THEN** the substrate returns its error and holds nothing new
