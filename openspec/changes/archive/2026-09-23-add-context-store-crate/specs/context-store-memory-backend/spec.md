## ADDED Requirements

### Requirement: An in-memory backend under `utils_test` implements both traits

`deep_causality_context_store::utils_test` SHALL declare `MemoryStorage`, implementing
`ContextStorage` with `type Slice = ContextId` and `type Error = MemoryStorageError`, and
`ContextStorageStream` with `type Cursor = usize` and `type Events = MemoryEvents`, with no
dependency beyond `std`.

`MemoryStorageError` is an enum with one variant per refusal the contract names, implementing
`Debug`, `Display` and `std::error::Error`. State lives behind a `std::sync::Mutex`; a poisoned
lock is recovered with `PoisonError::into_inner`. Identifiers are handed out from a counter that
starts past every identifier held; container identifiers start at 1. Every mutating operation
appends the events it emits to a log, `subscribe` from a cursor replays the log to that position
into a fresh state, and `apply_batch` applies to a clone of the state and commits only on success.
`MemoryEvents` yields every node and edge event and the membership, attachment and retraction
events of the container its slice names and of the containers that container referenced at
subscription, and ends at the end of the log.

Every function of `utils_test` counts toward the crate's coverage and is tested to the floor.

#### Scenario: The thirteen operations hydrate what they built

- **WHEN** a container is built through `create_context`, `create_node`, `create_edge`, `link` and
  `attach` with identifiers from `reserve`, and `hydrate` is called with its identifier
- **THEN** the snapshot holds exactly those nodes and edges, ordered as `Context::snapshot` orders
  them, and one extra per attached container with that container's identifier and name

#### Scenario: The backend refuses what the contract refuses

- **WHEN** each refusal named in the storage contract is attempted
- **THEN** each returns its `MemoryStorageError` variant and leaves the state unchanged

### Requirement: `MemorySubstrate` implements `Substrate`

`utils_test` SHALL declare `MemorySubstrate`, implementing `Substrate` over a map keyed by the
reference's `key`, with `source` fixed to `"memory"`, refusing a `Reference` on `deposit` and an
unknown reference on `resolve`.

#### Scenario: Deposit and resolve

- **WHEN** a `Fields` record is deposited under node 2 and resolved through the returned reference
- **THEN** the result equals the record and the reference's `source` is `"memory"`

### Requirement: `block_on` drives a future without a runtime

`utils_test` SHALL declare `pub fn block_on<F: Future>(future: F) -> F::Output`, polling with a
context built on `Waker::noop()` until the future is ready, using no `unsafe` and no dependency.

#### Scenario: A ready future completes in one poll

- **WHEN** `block_on(core::future::ready(7))` is called
- **THEN** it returns 7

#### Scenario: A pending future is polled again

- **WHEN** `block_on` is given a future that returns `Pending` once and then `Ready`
- **THEN** it returns the ready value
