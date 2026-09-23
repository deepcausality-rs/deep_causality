# TDD protocol record: `deep_causality_context_store`

The record the `unified-math-tdd-protocol` asks each stage to keep. One section per task group,
one subsection per phase, with the observed output and counts.

## Task groups 3 and 4: aliases, constants, errors, records, traits, `block_on`

### Phase 1: API-only

Every source under `src/alias`, `src/constants`, `src/errors`, `src/types/records`,
`src/types/id_reserve`, `src/types/context_event`, `src/traits` and `src/utils_test/block_on.rs`
was landed with its full signature and every function body reduced to `unimplemented!()`. The
traits have no bodies to reduce. `cargo build -p deep_causality_context_store` succeeded; the only
warnings were unused parameters and imports, which is what a body that never reads its arguments
produces. 42 bodies were reduced.

### Phase 2: the suite, observed failing

Suite: 25 test files, 102 test functions, mirroring `src/` file for file, each registered up its
`mod.rs` chain and declared to Bazel by folder. Each file's module doc carries its corner-case
table (rows A to K) naming the test that covers each row or stating why the row is n/a, and the
provenance of every expected value (the literal handed to the constructor, or the constant the
constants test pins independently). Every `ProjectionErrorEnum` variant is constructed through its
constructor and asserted by variant in `tests/errors/projection_error_tests.rs`.

`cargo test -p deep_causality_context_store --test mod` against the API-only crate:

```
test result: FAILED. 40 passed; 62 failed; 0 ignored; 0 measured; 0 filtered out
```

All 62 failures are the `not implemented` panic, and every panic location is under
`deep_causality_context_store/src/`; none is a compile error, a missing import or a panic from
elsewhere. The 40 that pass exercise nothing a body could get wrong:

- 17 are the vocabulary tests moved in task group 2 (`relation_kind`, `time_scale`,
  `vertical_datum`, `substrate_ref`), whose code predates this stage.
- 2 exercise an alias and a constant, which have no body (`alias_tests`, `constants_tests`).
- 21 exercise only derived `PartialEq`, `Clone`, `Copy` and `Debug` on the record enums, or a
  test-local implementor of a trait (`Probe`, `NullError`, `EchoSubstrate`), never a function of
  the crate. They are kept because they pin the derives the specs require, and they are listed
  here so the count is honest.

The full run is kept beside this note during the session as `phase2-failing-run.txt` and
summarised above.

### Corner-case enumeration

Rows A to K, per file, in each test file's module doc. Rows that apply to a record vocabulary:
A empty (`List`, `Fields`, an empty reserve, an empty snapshot, an empty stream), B single, C
coinciding values under different variants and coinciding identifiers under different payloads, D
index boundaries (`IdReserve` past its end; an off-diagonal metric entry), E the version
threshold, F zero, G negative, H exact boundaries (`u64::MAX`, `i64::MIN`, `u16::MAX`,
`u8::MAX`), I non-finite (`NaN` breaks equality, infinity does not). J overflow and K precision
are n/a: the records hold `f64` and `u64` by design and perform no arithmetic.

### Error variants

`WrongVariant`, `WrongPayload`, `MissingField`, `Unrecordable`, `Scalar`, `Identity`, `Version`:
each constructed and matched by variant in `projection_error_tests.rs`. None is unreachable.

### Phase 3: the defect audit

Eleven defects were injected one at a time into the implementation, the suite run, the failing
tests collected, and the file restored byte for byte (checked by `diff -rq` against the snapshot
taken before the audit). Each row names the defect class, the injection and the test whose subject
it is.

| Class | Injection | Subject test | Result |
|---|---|---|---|
| guard removed | `IdReserve::next` never advances `taken` | `id_reserve_tests::test_a_reserve_yields_each_identifier_once` | rejected, 4 tests fail |
| off-by-one | `remaining` counts one short | `id_reserve_tests::test_a_single_identifier` | rejected, 3 |
| plausible neighbour | `RelationRecord::to` returns `from` | `relation_record_tests::test_new_and_getters` | rejected, 2 |
| constant changed | `ContextSnapshot::new` at `RECORD_VERSION + 1` | `context_snapshot_tests::test_new_is_at_the_current_version` | rejected, 2 |
| flipped label | `DataRecord::Count` reports `"Integer"` | `data_record_tests::test_eight_variants_with_distinct_names` | rejected, 2 |
| flipped label | `SpaceRecord::Ecef` reports `"Euclidean"` | `space_record_tests::test_four_variants_with_distinct_names` | rejected, 2 |
| dropped field | `WrongVariant` display loses the node | `projection_error_tests::test_wrong_variant` | rejected, 1 |
| plausible neighbour | `ExtraContextSnapshot::name` returns `""` | `extra_context_snapshot_tests::test_new_and_getters` | rejected, 2 |
| off-by-one | `ContextRecord::id` adds one | `context_record_tests::test_new_and_getters` | rejected, 6 |
| value replaced | `ContextoidRecord::id` returns 0 | `contextoid_record_tests::test_new_and_getters` | rejected, 6 |
| early return | `block_on` stops after two polls | `block_on_tests::test_a_pending_future_is_polled_again` | **missed**: the suite's only pending future was ready on the second poll |

The missed defect widened the suite: `test_a_future_pending_several_times_completes` drives a
future that is pending five times, and the same injection then fails that test (1 failure).
Eleven of eleven rejected after the widening. Tolerance loosening is n/a: the crate has no
tolerance. Input variety: the coinciding-value rows (C) are supplemented in every record file by a
case where the values differ, per the tables.

### Phase 4: implementation against the audited suite

The implementation is the snapshot the audit was run on. `cargo test -p
deep_causality_context_store`: 103 passed, 0 failed. `cargo clippy --all-targets -- -D warnings`
clean after two lints were fixed by rewriting: a `ContextEventItem` alias replaces the nested
return type of `ContextEvents::next`, and `IdReserve::next` is the `Iterator` implementation. `cargo
llvm-cov -p deep_causality_context_store --tests`:

```
TOTAL  regions 255/255 100%  functions 53/53 100%  lines 274/274 100%
```

`bazel test //deep_causality_context_store/...`: 26 targets pass, one per test file plus the doc
test; 25 test files, 25 suite targets. Cargo counts test functions (103) and Bazel counts files,
so the raw numbers differ by unit and only the file coverage is comparable.

### Phase 5: mutation testing

`cargo mutants -p deep_causality_context_store -j 8` over the whole crate, 6 minutes:

```
90 mutants tested: 45 caught, 0 missed, 45 unviable
```

Every viable mutant was caught. The 45 unviable mutants are `Default::default()` substitutions
that do not compile: inside a `const fn` body (the getters of the record structs, the
`ProjectionError` constructors, `ContextRecord::id`), or on a type with no `Default`
(`ProjectionErrorEnum`, `ContextRecord`, `ContextoidRecord`, `NodeRecord`,
`ExtraContextSnapshot`). No survivor, so no entry was added to `.cargo/mutants.toml`.

## Task group 5: the in-memory backend

### Phase 1: API-only

`MemoryStorage`, `MemoryState`, `MemoryEvents`, `MemorySubstrate`, their trait implementations
and the two error types were landed with every signature and `unimplemented!()` bodies (a
future-returning body as `ready(unimplemented!())`). The error types are declarations with no
logic, like the record enums of groups 3 and 4, and were complete at this phase. The crate built.

### Phase 2: the suite, observed failing

Ten new test files, 50 test functions, mirroring `src/utils_test/` and `src/errors/`, each with
its corner-case table and provenance note. Against the API-only crate:

```
test result: FAILED. 109 passed; 44 failed   (153 in all; 103 from groups 3 and 4)
```

All 44 failures are the `not implemented` panic from `deep_causality_context_store/src/`. The six
new tests that pass are the two error types' constructor and display tests, which exercise no
body reduced in phase 1.

Every `MemoryStorageErrorEnum` variant is provoked through the public API: `UnknownContext`,
`UnknownNode`, `UnknownEdge`, `IdentityNotReserved`, `NodeConflict`, `EdgeConflict`,
`SelfReference`, `EventNotApplicable`, `UnknownCursor`, in `context_storage_tests.rs` and
`context_storage_stream_tests.rs`; both `MemorySubstrateErrorEnum` variants in
`substrate_tests.rs`.

### Phase 3: the defect audit

Twelve defects, injected one at a time into the implementation and restored byte for byte:

| Class | Injection | Subject test | Result |
|---|---|---|---|
| off-by-one | `reserve` hands out one identifier short | `test_reserve_hands_out_fresh_identifiers` | rejected, 27 |
| guard removed | `create_node` accepts unreserved identifiers | `test_an_identifier_not_from_a_reserve_is_refused` | rejected, 3 |
| flipped comparison | equal record conflicts, different passes | `test_a_node_is_immutable_under_its_name` | rejected, 2 |
| early return | retracted container leaves dangling references | `test_a_retracted_container_leaves_no_dangling_reference` | **missed**: `hydrate` filters unknown references, so the dangling entry was invisible there |
| constant changed | identifiers start at 0 | `test_reserve_hands_out_fresh_identifiers` | rejected, 3 |
| guard loosened | `hydrate` keeps edges with one end outside | `test_hydrate_holds_only_edges_among_members` | rejected, 2 |
| value replaced | `scope` drops the references | `test_scope_holds_the_references_at_subscription` | rejected, 3 |
| off-by-one | `apply` returns the cursor before its event | `test_cursors_are_log_positions` | rejected, 2 |
| guard removed | the stream ignores its scope | `test_membership_events_of_other_containers_are_withheld` | rejected, 3 |
| early return | a failed batch commits its prefix | `test_apply_batch_is_atomic` | rejected, 1 |
| off-by-one | replay includes the event at the cursor | `test_a_subscription_resumes_from_a_cursor` | rejected, 5 |
| guard removed | `resolve` ignores the reference's source | `test_an_unknown_reference_is_refused` | rejected, 1 |

The missed defect widened the suite: `test_scope_drops_a_retracted_reference` observes the
reference through the subscription scope, where a dangling entry is visible, and the same
injection then fails it. Twelve of twelve rejected after the widening. Tolerance loosening is n/a.

### Phase 4: implementation against the audited suite

`cargo test -p deep_causality_context_store`: 155 passed. Clippy and fmt clean. Coverage:

```
TOTAL  regions 1254/1267 98.97%  functions 155/155 100%  lines 933/936 99.68%
```

The three uncovered lines are the `NodeEntered` and `NodeLeft` alternatives of two match arms,
`MemoryState::fold` (`memory_state.rs`, two lines) and `MemoryEvents::in_scope`
(`memory_events/mod.rs`, one line). They are unreachable by construction: the log is appended only
by this backend's operations, none of which emits either variant, and `apply` refuses both, so no
event of these variants can be in a log. The arms exist because the match over `ContextEvent` must
be exhaustive, and they are grouped with `NodeLinked` and `NodeUnlinked` because that is what the
streaming spec says a view's answer means to a context.

`bazel test //deep_causality_context_store/...`: 36 targets pass, 35 test files, 35 suite
targets.

## Task group 6: named extra contexts

### Phase 1: API-only

`ExtendableContextuableGraph` gained the `name` parameter on `extra_ctx_add_new` and
`extra_ctx_add_new_with_id` and the new `extra_ctx_get_name`; the extra-context map's value became
the private `ExtraContext { name, graph }`. The getter's body was `unimplemented!()`, the
allocator and the zero check were left as they were, and the 53 call sites in the crate's own four
test files gained a name argument. The crate built and its 455 existing tests passed.

### Phase 2: the suite, observed failing

`tests/types/context_types/context_graph/extra_context_tests.rs`, six tests with its corner-case
table, against the phase 1 crate:

```
test result: FAILED. 0 passed; 6 failed
```

Three fail with the unimplemented panic from `extendable_contextuable_graph.rs` (the name tests),
three on their assertions: identifier 0 accepted, and the count-based allocator answering 1 and 4
where 8 and 42 are required.

### Phase 3: the defect audit

| Class | Injection | Subject test | Result |
|---|---|---|---|
| off-by-one | allocator lands two past the highest | `test_sequential_numbering_is_unchanged` | rejected, 5 |
| constant changed | allocator starts at 0 when empty | `test_sequential_numbering_is_unchanged` | rejected, 32 |
| regression | allocator counts extras again | `test_an_explicit_identifier_is_never_reached` | rejected, 2 |
| guard removed | identifier 0 accepted | `test_identifier_zero_is_refused` | rejected, 1 |
| value replaced | the name getter answers `None` | `test_a_name_is_kept_with_the_extra` | rejected, 3 |

Five of five rejected. Sources restored byte for byte.

### Phase 4: implementation against the audited suite

`cargo test -p deep_causality_context`: 461 passed. Clippy and fmt clean. The `expect` on the
allocator's insertion stays: the identifier it inserts is fresh and non-zero by construction, and
the method returns a `ContextId` with no error channel.

### Phase 5 (group 5): mutation testing

`cargo mutants -p deep_causality_context_store -j 8` over the whole crate, 7 minutes:

```
203 mutants tested: 104 caught, 1 missed, 92 unviable, 6 timeouts
```

- The miss, `replace != with == in MemoryState::fold` at the `NodeRetracted` arm, kept an edge
  into a retracted node and dropped unrelated edges. Every test had observed edges through
  `hydrate`, which filters by membership, so the lingering edge was invisible. The suite was
  widened with `test_a_retracted_node_takes_only_its_own_edges`, which observes edges through
  `retract_edge`. A re-run over `memory_state.rs` after the widening: 63 mutants, 45 caught, 18
  unviable, none missed.
- The 6 timeouts are five `IdReserve` mutants that stop the iterator advancing (`taken` fixed at
  0 or 1, `advance` emptied, `+=` to `*=`, `next` returning a constant) and the `MemoryEvents::next`
  position update mutated to a multiplication. Each makes a draining loop in the suite run
  forever, so the harness detects the mutant by non-termination rather than by an assertion. They
  are not survivors and need no equivalence entry.
- The 92 unviable mutants are `Default::default()` substitutions on types without `Default`
  (`Self`, `MemoryStorageError`, the records) and inside `const fn` bodies.

### Phase 5 (group 6): mutation testing

`cargo mutants -p deep_causality_context` over `extendable_contextuable_graph.rs` and
`extra_context.rs`, 11 minutes:

```
39 mutants tested: 37 caught, 1 missed, 1 unviable
```

The miss was `+= to *=` on `number_of_extra_contexts`, a counter the count-based allocator read
and the new allocator does not, which left the field write-only. The field, its initialiser, its
clone and its increment were removed rather than pinned: a value nothing reads is not a decision
a test can defend. The re-run after the removal is recorded below.

Re-run over the same two files after the removal, 5 minutes:

```
37 mutants tested: 36 caught, 1 unviable, 0 missed
```

## Task groups 7 and 8: `Storable`, the projection, `snapshot` and `restore`

One stage for both groups: the projection and the two `Context` methods share a suite and a
mutation run.

### Phase 1: API-only

`Storable` was declared in `deep_causality_context/src/traits/storable/`, its eleven
implementations under `src/extensions/storable/`, `Recordable` beside every node type (four
space, five time, four spacetime, the absent spacetime twice, `Data<T: Storable>`, `Contextoid`),
and `Context::snapshot` and `Context::restore` under `context_graph/`, every body
`unimplemented!()`. The crate built; the 461 existing tests passed.

### Phase 2: the suite, observed failing

27 new test files, 115 test functions, each file with its corner-case table and provenance note:
`tests/traits/storable/`, `tests/extensions/storable/`, `recordable_tests.rs` beside every node
type, `snapshot_tests.rs` and `restore_tests.rs`. Against the API-only crate:

```
test result: FAILED. 461 passed; 115 failed
```

All 115 failures are the `not implemented` panic from `deep_causality_context/src/`; no new test
passed. The scenarios of `context-projection` are each covered: the shape-change compile failure
is a property of the layout rather than a test; every other scenario names its test in a table.

### Phase 3: the defect audit

Fourteen defects, injected one at a time and restored byte for byte:

| Class | Injection | Subject test | Result |
|---|---|---|---|
| guard removed | `f32` accepts a double it cannot hold | `scalars_tests::test_an_f32_that_cannot_hold_the_double_is_refused` | rejected, 3 |
| guard loosened | a longer list read as an `Option` | `collections_tests::test_a_longer_list_is_not_an_option` | rejected, 3 |
| plausible neighbour | `Float106` halves swapped on read | `software_scalars_tests::test_float106_round_trips_to_every_bit` | rejected, 4 |
| plausible neighbour | `GeoSpace` writes `lon` into `lat` | `geo_space::recordable_tests::test_round_trip` | rejected, 6 |
| plausible neighbour | `NedSpace` reads `east` as `north` | `ned_space::recordable_tests::test_round_trip` | rejected, 9 |
| early return | `TangentSpacetime` keeps the default metric | `test_the_stored_metric_is_restored_not_the_default` | rejected, 3 |
| flipped label | an `Ecef` record lands in `Euclidean` | `space_kind::recordable_tests::test_every_variant_round_trips` | rejected, 3 |
| constant changed | a restored root gets identifier 0 | `contextoid::recordable_tests::test_a_root_is_an_ordinary_record` | rejected, 4 |
| constant changed | `DiscreteTime` writes `NoScale` | `discrete_time::recordable_tests::test_round_trip` | rejected, 6 |
| early return | `snapshot` leaves edges unsorted | `snapshot_tests::test_a_snapshot_is_canonical` | rejected, 3 |
| flipped comparison | the current version is refused | `restore_tests::test_a_newer_snapshot_is_refused` | rejected, 13 |
| early return | the current extra stays set after restore | `restore_tests::test_run_time_state_starts_empty` | rejected, 3 |
| guard removed | a duplicate node identifier is accepted | `restore_tests::test_a_duplicate_identifier_is_refused` | rejected, 3 |
| guard removed | a duplicate edge is accepted | `restore_tests::test_a_duplicate_edge_is_refused` | rejected, 1 |

Fourteen of fourteen rejected. Tolerance loosening is n/a.

### Phase 4: implementation against the audited suite

`cargo test -p deep_causality_context`: 577 passed. Clippy and fmt clean. Two facts of the tree
changed the implementation or a test during this phase, each recorded here:

- `ultragraph` accepts parallel edges, so the "edge could not be added" arm of `restore` is
  unreachable with valid indices. A relation exists once between two nodes in the storage
  contract, so `restore` refuses a duplicate `(from, to)` itself, with
  `Identity(from, "an edge is carried twice")`, and a test provokes it.
- `TangentSpacetime::new` builds its default metric with `c²` held exactly in `Float106`'s two
  halves. The metric narrows to `f64` in the record like every other field, so the restored
  metric is the narrowed one. The test had expected the exact one; its expectation was corrected
  to the rule the spec states, not weakened.
- `deep_causality_num` narrows a huge double to an infinite `f32` rather than refusing. The `f32`
  `Storable` refuses a finite double whose `f32` value is not finite with `Scalar`, so the
  spec's scenario holds. The generic coordinate implementations rely on `FromPrimitive::from_f64`
  and inherit the crate's behaviour for `f32`, which never returns `None`.

Two scenarios joined the suite during phase 4, both from facts found while implementing: the
duplicate-edge refusal above, and a snapshot taken after `remove_node`, whose tombstoned index the
walk steps over (`snapshot_tests::test_a_removed_node_is_not_recorded`, which coverage showed
missing). Final count: 578 tests in the crate, 117 of them from this stage.

Coverage over the stage's files: every `Storable` and `Recordable` file and `snapshot.rs` at
100% of lines. `restore.rs` has two uncovered lines, the `map_err` arm for an edge the graph
refuses to add. It is unreachable by construction: both indices come from the map the same
function filled, `ultragraph` accepts parallel edges, and duplicates are refused before the call.
The matching arm for a node the graph refuses to add is unreachable for the same reason on a
fresh dynamic graph and is a closure coverage reports as a missed function.

`bazel test //deep_causality_context/...`: 90 targets pass, 89 test files, 89 suite targets.

### Phase 5: mutation testing

`cargo mutants -p deep_causality_context` over `src/extensions/**`, every `recordable.rs`,
`snapshot.rs` and `restore.rs`, 7 minutes:

```
84 mutants tested: 26 caught, 0 missed, 58 unviable, 0 timeouts
```

The unviable mutants are `Default::default()` substitutions on return types with no `Default`:
the node types behind `Result<Self, _>`, `ProjectionError`, `ContextSnapshot`, and the
record-vector pairs `walk` returns. No survivor, so no entry was added to `.cargo/mutants.toml`.

## Task groups 9 and 10: `ContextStore`, `StoreError`, the aliases, `Context::apply` and `subscribe`

One stage for both groups: the store handle and the streaming block share the in-memory backend,
one suite and one mutation run.

### Phase 1: API-only

`StoreError<E, B = Infallible>` in `deep_causality_context/src/errors/store_error.rs` after the
physics convention (a public struct around `StoreErrorEnum`, per-variant constructors, `kind`,
`Display`, `Error`, `From<ProjectionError>`); `ContextStore<S>` under
`src/types/context_types/context_store/` with `new`, `storage`, `reserve`, `hydrate`,
`store_branch`, `create_node_via`, `hydrate_via` and, in a block bounded on
`ContextStorageStream`, `subscribe`; `Context::apply` in `context_graph/apply.rs`; the aliases
`SubstrateContext` and `SubstrateContextoid` in `src/alias/mod.rs`. Every method body
`unimplemented!()`. The error type and the aliases carry no logic beyond construction and so were
complete in this phase. The crate built; the 578 existing tests passed.

### Phase 2: the suite, observed failing

Eight new test files, 37 test functions, each file with its corner-case table and provenance note:
`tests/errors/store_error_tests.rs`, `tests/alias/alias_tests.rs`,
`tests/types/context_types/context_store/{context_store,hydrate,store_branch,substrate,subscribe}_tests.rs`
and `tests/types/context_types/context_graph/apply_tests.rs`. Against the API-only crate:

```
test result: FAILED. 584 passed; 31 failed
```

All 31 failures are the `not implemented` panic from `deep_causality_context/src/`. Six new tests
passed already: the four `StoreError` tests and the alias test exercise construction alone, and
`context_store_tests::test_the_store_is_thin` reaches the backend through `storage()`, a getter.
Every scenario of `context-store-facade` names its test; of `context-store-streaming` the
context-side scenarios do (the five stream-mechanics scenarios were closed in the store crate
in group 6), and the store-only backend scenario is the `compile_fail` doctest on the
`subscribe` block, which `cargo test --doc` runs.

### Phase 3: the defect audit

Twenty defects, injected one at a time by `audit910.py` and restored byte for byte:

| Class | Injection | Subject test | Result |
|---|---|---|---|
| early return | `store_branch` stores edges unremapped | `store_branch_tests::test_a_changed_value_is_a_new_node` | rejected, 1 |
| plausible neighbour | a remapped node linked under its old identifier | same | rejected, 1 |
| flipped guard | nodes never linked into the container | four tests | rejected, 4 |
| swapped arguments | extras attached the wrong way round | three tests | rejected, 3 |
| flipped comparison | a held equal node re-created | four tests | rejected, 4 |
| guard removed | a `Reference` payload deposited again | `substrate_tests::test_an_empty_slice_deposits_nothing` | rejected, 1 |
| early return | extras' references left unresolved | `substrate_tests::test_extras_are_resolved_too` | rejected, 1 (after widening) |
| constant changed | `hydrate_via` drops the snapshot version | four tests | rejected, 4 |
| early return | `hold` skips the base identifier index | `apply_tests::test_a_membership_event_is_self_contained` | **missed, then rejected, 1** |
| early return | `NodeRetracted` leaves the extras | `apply_tests::test_unlink_left_and_retract` | rejected, 1 |
| early return | an edge joined in the base alone | `apply_tests::test_an_edge_lands_wherever_both_ends_are_held` | rejected, 1 |
| early return | the current extra not reset on drop | `apply_tests::test_attach_and_detach_on_the_held_context` | rejected, 1 |
| constant changed | an attached container under 0 accepted | `apply_tests::test_an_attached_container_under_zero_is_refused` | rejected, 1 |
| guard removed | an attachment on another container adds an extra | `apply_tests::test_attach_and_detach_on_the_held_context` | rejected, 1 |
| constant changed | the unknown-container rule reworded | `apply_tests::test_an_unknown_container_is_refused` | rejected, 1 |
| flipped guard | `sever` removes only an absent edge | `apply_tests::test_an_edge_lands_wherever_both_ends_are_held` | rejected, 1 |
| constant changed | retracting the held context accepted | `apply_tests::test_context_retracted` | rejected, 1 |
| swapped variant | a projection failure in `hydrate` reported as another | `hydrate_tests::test_a_projection_failure_names_the_node` | rejected, 1 |
| swapped label | `Display` names the substrate variant "storage" | `store_error_tests::test_substrate_variant` | rejected, 1 |
| off by one | `reserve(n)` asks the backend for `n + 1` | five tests | rejected, 5 |

Nineteen of twenty rejected on the first pass; one missed. `hold` added a node to the base
graph without recording it in the identifier index, and every assertion read the graph through
`snapshot`, which walks the graph and never consults the index. The test now reads the linked node
back through `get_node_index_by_id` and removes it through `remove_node`, which goes through the
index, and the unlink test asserts the index forgets a node at once and is left alone by an event
on an extra. The `sever` anchor had drifted under `rustfmt` and was re-anchored; the defect was then
rejected. Tolerance loosening is n/a.

### Phase 4: implementation against the audited suite

`cargo test -p deep_causality_context`: 618 passed, 40 of them from this stage; the `compile_fail`
doctest passes. Clippy and fmt clean. Facts of the tree that shaped the implementation or a test:

- The `apply` fixture first gave an extra both ends of a stored edge and not the edge. A store
  holds edges between nodes, not per container, so a container holding both ends holds the edge,
  and the echo of `EdgeCreated` rightly added it. The fixture was corrected to a snapshot a store
  can produce; the routing rule was not changed.
- `store_branch` creates every remapped edge, including edges between two shared nodes the store
  already holds. The storage contract makes `create_edge` idempotent for an identical edge, so no
  comparison against the store is made for edges.
- `store_graph` refuses, with `ProjectionError::Identity(id, "the backend's reserve held fewer
  identifiers than asked")`, a reserve that answers short. No conforming backend does; the arm
  is reached by `store_branch_tests::test_a_short_reserve_is_refused` through a wrapper around the
  in-memory backend whose `reserve` answers empty.
- `Context` has no freeze, and every index `apply` passes to the graph comes from a scan of that
  same graph, so the `map_err` arms for a node or edge the graph refuses to add or remove are
  unreachable. They are the uncovered lines of `apply.rs` (the `add_edge` arm) and its missed
  closures, as in `restore.rs`.

Two tests joined the suite during phase 4 from coverage: `hydrate_via` over a container that
references another (the extras loop and the non-reference pass-through were never run), and the
context-side form of the fixed-scope scenario in which the host's own `store_branch` creates a
container outside its subscription (`subscribe_tests::test_a_new_container_is_outside_an_existing_subscription`).

Coverage over the stage's files: `hydrate.rs`, `mod.rs`, `subscribe.rs` and `substrate.rs` at
100% of lines; `store_error.rs` and `store_branch.rs` with one region each that coverage counts
as a missed line and lists no line for; `apply.rs` with the unreachable arms above.

`bazel test //deep_causality_context/... //deep_causality_context_store/...`: 134 targets pass;
the eight new test files each have a suite target (`alias_tests`, `error_types_tests`,
`ctx_graph_types_tests`, `ctx_types_context_store_tests`).

### Phase 5: mutation testing

`cargo mutants -p deep_causality_context` over `store_error.rs`, `context_store/*.rs` and
`apply.rs`, two runs of 3 and 2 minutes:

```
52 mutants tested: 40 caught, 1 missed, 11 unviable     (first run)
52 mutants tested: 41 caught, 0 missed, 11 unviable     (after the widening)
```

The survivor was `==` to `!=` in `Context::release` at the test that decides whether the base
identifier index is updated. The unlink test released a node from the base and then from an extra,
so the flipped test still emptied the index by the end. The test now asserts the index forgets the
node at the base release and is untouched by an extra's release. No entry was added to
`.cargo/mutants.toml`. The unviable mutants are `Default::default()` substitutions on return
types with no `Default`: `StoreError`, `&S`, `IdReserve`, `Context`, the `(Context, Events)`
pair and `ContextoidRecord`.
