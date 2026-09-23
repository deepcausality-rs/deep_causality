## ADDED Requirements

### Requirement: `ContextEvent` has one variant per mutating operation and two for a view

The store crate SHALL declare `ContextEvent` with the variants `ContextCreated(ContextRecord)`,
`ContextRetracted(ContextId)`, `NodeCreated(ContextoidRecord)`, `NodeRetracted(ContextoidId)`,
`EdgeCreated(RelationRecord)`, `EdgeRetracted { from, to }`, `NodeLinked { context, node:
ContextoidRecord }`, `NodeUnlinked { context, node: ContextoidId }`, `ContextAttached { context,
extra: ContextRecord }`, `ContextDetached { context, extra: ContextId }`, `NodeEntered { context,
node: ContextoidRecord }` and `NodeLeft { context, node: ContextoidId }`, deriving `Debug`, `Clone`
and `PartialEq`.

A membership event carries the container it concerns and, when it adds a node, the node's record,
so a subscriber applies it to a hydrated context without having seen any earlier event. The event
carries no time: order is the stream's cursor, and the time a node describes is the node's own
payload. `NodeEntered` and `NodeLeft` are a view's answer moving with no operation behind it;
`reserve`, `lookup` and `hydrate` change nothing and have no event.

#### Scenario: A membership event is self-contained

- **WHEN** a `NodeLinked` event is applied to a context that does not hold the node
- **THEN** the context holds it afterwards, without any earlier event having been seen

### Requirement: `ContextEvents` is an asynchronous iterator the store crate declares

The store crate SHALL declare `ContextEvents` with `type Error: Debug + Display`, `type Cursor:
Clone + Send` and `fn next(&mut self) -> impl Future<Output = ContextEventItem<Self::Cursor,
Self::Error>> + Send`, where `ContextEventItem<Cursor, Error>` is the alias
`Option<Result<(Cursor, ContextEvent), Error>>`, taking no stream trait from any crate.

`None` means the stream has ended. Each item carries the cursor after the event, so a subscriber
can resume from it.

#### Scenario: A stream ends

- **WHEN** an in-memory stream has delivered every event in the log
- **THEN** `next` returns `None`

### Requirement: `ContextStorageStream` requires `ContextStorage` and takes the event alone

The store crate SHALL declare `ContextStorageStream: ContextStorage` with `type Cursor: Clone +
Send`, `type Events: ContextEvents<Cursor = Self::Cursor, Error = Self::Error> + Send`, and three
operations returning `impl Future<…> + Send`: `subscribe(&self, spec: &Self::Slice, from:
Option<Self::Cursor>) -> Result<(ContextSnapshot, Self::Events), _>`, `apply(&self, event:
&ContextEvent) -> Result<Self::Cursor, _>` and `apply_batch(&self, events: &[ContextEvent]) ->
Result<Self::Cursor, _>`. No operation has a default body.

`subscribe` returns the snapshot and the stream in one call so no change falls between them; with
a cursor, the snapshot is the slice as of that cursor and the stream continues from it. `apply`
performs the operation the event names under the same refusals; `apply_batch` applies all in
order as one transaction and is refused whole if any event would be.

#### Scenario: A store-only backend is complete

- **WHEN** a backend implements `ContextStorage` and not `ContextStorageStream`
- **THEN** it compiles, `ContextStore` over it offers `hydrate`, `reserve` and `store_branch`, and
  a call to `subscribe` on it is a compile error

#### Scenario: No change falls between snapshot and stream

- **WHEN** a subscription is taken, a node is then linked into the subscribed container, and the
  stream is drained
- **THEN** the stream yields exactly the `NodeLinked` event for that node, once

#### Scenario: A batch is atomic

- **WHEN** `apply_batch` is given a valid `NodeLinked` followed by a `NodeCreated` for an
  identifier no reserve handed out
- **THEN** it is refused, the container's links are unchanged, and the log has grown by nothing

#### Scenario: A subscription resumes from a cursor

- **WHEN** a subscription is dropped after cursor `c`, two more events commit, and `subscribe` is
  called with `Some(c)`
- **THEN** the snapshot equals the state as of `c` and the stream yields exactly the two events

### Requirement: A subscription's scope is fixed at its slice

A subscription SHALL deliver the changes to the container its slice names and to the containers
that container referenced when the subscription began, and nothing else, for the life of the
stream. A container created or attached afterwards is reached by a new `subscribe` whose slice
names it, resumed from the cursor the host has reached.

#### Scenario: A new container is outside an existing subscription

- **WHEN** a host subscribed to container A stores a branch as container B and drains its stream
- **THEN** no membership event of B arrives; subscribing to B from the current cursor yields B's
  snapshot and its changes from there

#### Scenario: An attachment is delivered and materialised by resubscribing

- **WHEN** another writer attaches container C to subscribed container A, and the host drains the
  stream into `Context::apply` and then subscribes to A again from the cursor reached
- **THEN** the first drain yields `ContextAttached { A, C }` and the context holds an empty extra
  under C's identifier and name, and the second subscription's snapshot holds C's contents as that
  extra

### Requirement: `Context::apply` applies one event idempotently

`Context<D, S, T, ST>` SHALL provide `pub fn apply(&mut self, event: &ContextEvent) -> Result<(),
ProjectionError>` under the `Recordable` bounds of `snapshot`, routing by the container an event
names: the context's own identifier is the base graph, an extra's identifier is that extra, any
other is `ProjectionError::Identity`.

| Event | Effect |
|---|---|
| `NodeCreated`, `ContextCreated` | none |
| `NodeLinked`, `NodeEntered` | add the record to the named graph; present already: none |
| `NodeUnlinked`, `NodeLeft` | remove from the named graph; absent: none |
| `NodeRetracted` | remove from every graph; absent: none |
| `EdgeCreated` | add to every graph holding both ends; present already: none |
| `EdgeRetracted` | remove from every graph holding it; absent: none |
| `ContextAttached` on the held context | add an empty extra under the attached container's identifier and name; present already: none |
| `ContextDetached` on the held context | drop that extra; absent: none |
| `ContextAttached`, `ContextDetached` on any other container | none |
| `ContextRetracted` | an extra: drop it; the held context: `ProjectionError::Identity`; other: none |

#### Scenario: An echo is harmless

- **WHEN** a context applies `NodeLinked` for a node it holds and `EdgeCreated` for an edge it
  holds
- **THEN** both return `Ok(())` and the snapshot is unchanged

#### Scenario: An edge lands wherever both ends are held

- **WHEN** the base graph and one extra both hold nodes 3 and 4 and `EdgeCreated { 3, 4, Spatial }`
  is applied
- **THEN** both graphs hold the edge and a second extra holding only node 3 does not

#### Scenario: The held context cannot retract itself

- **WHEN** `ContextRetracted(self.id())` is applied
- **THEN** the result is `Err(ProjectionError::Identity { .. })` and the context is unchanged

### Requirement: `ContextStore::subscribe` keeps a context current

`impl<S: ContextStorageStream> ContextStore<S>` SHALL provide the asynchronous method
`subscribe<D, S_, T, ST>(&self, spec: &S::Slice, from: Option<S::Cursor>)`, whose future
resolves to `Result<(Context<D, S_, T, ST>, S::Events), StoreError<S::Error>>`: it awaits the
backend's `subscribe`, restores the snapshot and returns the stream beside the context.

#### Scenario: A subscribed context follows the store

- **WHEN** a context is obtained through `subscribe`, a node is linked into its container through
  the storage, and the stream is drained into `Context::apply`
- **THEN** the context's snapshot equals a fresh `hydrate` of the same slice
