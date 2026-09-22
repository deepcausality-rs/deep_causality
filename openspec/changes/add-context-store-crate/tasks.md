Groups 3 onward follow the `unified-math-tdd-protocol`: API-only, suite observed failing, defect
audit, implementation, mutants. The record is `tdd-notes.md` beside this file.

## 1. Scaffold `deep_causality_context_store`

- [x] 1.1 Create `deep_causality_context_store/` with `Cargo.toml`: version 0.1.0,
      workspace-inherited edition, rust-version, license, repository and homepage, crates.io
      metadata, the standard `exclude` list, `[lints] workspace = true`, and no `[dependencies]`
      table
- [x] 1.2 Add `deep_causality_context_store` to `members` in the root `Cargo.toml` and add
      `deep_causality_context_store = { path = "deep_causality_context_store", version = "0.1" }`
      to `[workspace.dependencies]`
- [x] 1.3 Add `BUILD.bazel` modelled on `deep_causality_context/BUILD.bazel`: `rust_library` with
      `all_crate_deps(normal = True)` (resolves empty), `rust_doc`, `rust_doc_test`; test suites are
      added as the test folders appear
- [x] 1.4 Add `README.md` and `LICENSE`; do not create `CHANGELOG.md`
- [x] 1.5 Create `src/lib.rs` with the SPDX header and the module skeleton (`alias`, `constants`,
      `errors`, `traits`, `types`, `pub mod utils_test`); confirm `cargo build -p
      deep_causality_context_store` succeeds and `source scripts/crates.sh` lists the crate

## 2. Move the vocabulary down

- [x] 2.1 `git mv deep_causality_context/src/types/context_types/{relation_kind,time_scale,vertical_datum}`
      to `deep_causality_context_store/src/types/`, and `mv` the untracked
      `deep_causality_context/src/types/context_types/substrate_ref` beside them; register all four
      in `src/types/mod.rs` and export them from `lib.rs`
- [x] 2.2 `git mv deep_causality_context/tests/types/context_types/{relation_kind,time_scale,vertical_datum}`
      to `deep_causality_context_store/tests/types/`; add `tests/types/substrate_ref/substrate_ref_tests.rs`
      covering constructor, getters, `Display`, `Default`, `Hash` and equality; register every file
      in its `mod.rs` chain
- [x] 2.3 Add `deep_causality_context_store = { workspace = true }` to
      `deep_causality_context/Cargo.toml`; replace the four removed modules in
      `deep_causality_context/src/lib.rs` with `pub use deep_causality_context_store::{RelationKind,
      SubstrateRef, TimeScale, VerticalDatum};` above a comment stating why they are re-exported
- [x] 2.4 Remove the three vocabulary `rust_test_suite` targets from
      `deep_causality_context/BUILD.bazel`; add the matching suites to the store crate's
      `BUILD.bazel`. Confirm `cargo build --workspace` passes with no import changed anywhere else

## 3. Aliases, constants, errors, records

- [x] 3.1 `src/alias/mod.rs`: `IdentificationValue = u64`, `ContextId`, `ContextoidId`, each with
      a docstring stating that the width is the record's and that the projection pins it to the
      engine's
- [x] 3.2 `src/constants/mod.rs`: `pub const RECORD_VERSION: u16 = 1;`
- [x] 3.3 `src/errors/projection_error.rs`: `ProjectionError(pub ProjectionErrorEnum)` in the
      repository's error convention with the seven variants, a constructor per variant, `kind()`,
      `Display` naming the identifier and both names, `std::error::Error`; tests for every variant's
      display and equality. Corrected during apply: struct-around-enum, as `PhysicsError` shows it
- [x] 3.4 `src/types/records/data_record/mod.rs`: the eight-variant value tree with `Fields` as an
      ordered `Vec<(String, DataRecord)>`; `Debug`, `Clone`, `PartialEq`
- [x] 3.5 `src/types/records/<record>/mod.rs`, one folder module per record: `TimeRecord`,
      `SpaceRecord`, `SpaceTimeRecord`, `NodeRecord` as enums with named fields; `ContextRecord`, `ContextoidRecord`, `RelationRecord`,
      `ExtraContextSnapshot` (`id`, `name`, `nodes`, `edges`) as structs with private fields,
      constructors and getters; `Copy` wherever no field forbids it
- [x] 3.6 `src/types/records/context_snapshot/mod.rs`: `ContextSnapshot` (`version`, `context`,
      `nodes`, `edges`, `extras`) with `new` setting `version` to `RECORD_VERSION`, getters, and
      `into_parts` for `restore`
- [x] 3.7 `src/types/id_reserve/mod.rs`: `IdReserve` with `new`, `remaining`, and `next` as its
      `Iterator` implementation in `iterator.rs` (clippy's `should_implement_trait`)
- [x] 3.8 `src/types/context_event/mod.rs`: `ContextEvent` with the twelve variants
- [x] 3.9 Tests under `tests/types/` mirroring every file: constructor and getter round trips, derive
      behaviour, one test per record enum asserting its variant count against the number the spec
      states, the ordered-equality scenario for `Fields`

## 4. Traits

- [x] 4.1 `src/traits/recordable.rs`: `Recordable<Rec>` with `to_record` and `from_record`
- [x] 4.2 `src/traits/context_storage.rs`: `ContextStorage` with `Error`, `Slice` and the thirteen
      operations returning `impl Future<Output = Result<…, Self::Error>> + Send`; docstrings state
      each refusal, the idempotence rules, the one-level materialisation of references and the
      edition-2024 borrow note
- [x] 4.3 `src/traits/substrate.rs`: `Substrate` with `deposit` and `resolve` over `DataRecord`
- [x] 4.4 `src/traits/context_events.rs`: `ContextEvents` with `Error`, `Cursor` and `next`
- [x] 4.5 `src/traits/context_storage_stream.rs`: `ContextStorageStream: ContextStorage` with
      `Cursor`, `Events`, `subscribe`, `apply`, `apply_batch`, no default bodies; docstring states
      the fixed scope of a subscription
- [x] 4.6 Export every trait from `lib.rs`; add `tests/traits/*_tests.rs` with a minimal
      implementor of each trait to pin static dispatch and the `Send` bound through an
      `assert_send` helper

## 5. In-memory backend under `utils_test`

- [x] 5.1 `src/utils_test/block_on.rs`: `block_on<F: Future>(F) -> F::Output` polling on
      `Waker::noop()`; test with `ready`, a once-pending and a five-times-pending future. Pulled
      forward into group 4 because the trait tests drive futures with it
- [ ] 5.2 `src/utils_test/memory_storage/`: `MemoryStorage` holding `Mutex<MemoryState>` and the
      event log; `MemoryState` as a fold over `ContextEvent` with the reserve counter, nodes,
      edges, containers (name, links, references); `MemoryStorageError` with one variant per
      refusal
- [ ] 5.3 Implement `ContextStorage` for `MemoryStorage`: every operation validates, mutates,
      appends its events, and returns `ready(...)`; `hydrate` assembles the container and its
      referenced containers one level deep, ordering nodes and edges canonically
- [ ] 5.4 Implement `ContextStorageStream` for `MemoryStorage`: `MemoryEvents` over
      `Arc<Mutex<…>>` and a position, filtered to the slice's container and the containers it
      referenced at subscription; `subscribe` with replay from a cursor; `apply` dispatching to the
      operation the event names; `apply_batch` on a cloned state committed on success
- [ ] 5.5 `src/utils_test/memory_substrate.rs`: `MemorySubstrate` implementing `Substrate` over a
      map keyed by `key`, `source` fixed to `"memory"`
- [ ] 5.6 Tests under `tests/utils_test/`: one test per invariant in `context-storage-contract`
      including the three reference scenarios, one per refusal variant, subscription with and
      without a cursor, the fixed-scope scenarios, the atomic batch, the stream ending, the
      substrate round trip and refusals. Register the `utils_tests` suite in `BUILD.bazel`

## 6. Named extra contexts in the context crate

- [ ] 6.1 `context_graph/extra_context.rs`: a private `ExtraContext { name, graph }`; the
      extra-context map holds it
- [ ] 6.2 Change `ExtendableContextuableGraph`: `extra_ctx_add_new(name, capacity, default)`,
      `extra_ctx_add_new_with_id(id, name, capacity, default)` refusing identifier 0,
      `extra_ctx_get_name(id) -> Option<&str>`; update the implementation and the four test files
      that call the two constructors
- [ ] 6.3 Change `extra_ctx_add_new` to allocate `max held extra identifier + 1`, 1 when none;
      correct any existing test that asserted the count-based number after an explicit insertion
- [ ] 6.4 Tests: every scenario in `context-named-extra-contexts`

## 7. `Storable` and the projection

- [ ] 7.1 `src/traits/storable/mod.rs`: `Storable` with `to_record` and `from_record`
- [ ] 7.2 `src/extensions/storable/`: implementations for `f32`, `f64`, `u64`, `i64`, `bool`,
      `String`, `SubstrateRef`, `Vec<T>`, `Option<T>`, `Float106` as `Fields { hi, lo }`, and
      `BFloat16`; one file per group; tests under `tests/extensions/storable/` for every round
      trip, every wrong payload, the missing field, and `Float106` to every bit
- [ ] 7.3 `data/recordable.rs`: `impl<T: Storable + Default + Clone + PartialEq>
      Recordable<DataRecord> for Data<T>`; test with the shipped payloads and with a test-local
      struct implementing `Storable` as `Fields`
- [ ] 7.4 `recordable.rs` beside each of `GeoSpace`, `EcefSpace`, `EuclideanSpace`, `NedSpace`
      and `SpaceKind` implementing `Recordable<SpaceRecord>` with `R: RealField + Into<f64> +
      FromPrimitive`; wrong-variant refusals name the node
- [ ] 7.5 `recordable.rs` beside each of `EuclideanTime`, `LorentzianTime`, `DiscreteTime`,
      `EntropicTime` and `TimeKind` implementing `Recordable<TimeRecord>`
- [ ] 7.6 `recordable.rs` beside each of `EuclideanSpacetime`, `LorentzianSpacetime`,
      `TangentSpacetime` and `SpaceTimeKind` implementing `Recordable<SpaceTimeRecord>`;
      `TangentSpacetime::from_record` restores the stored metric through `update_metric_tensor`
- [ ] 7.7 `recordable.rs` beside `NoSpaceTime` implementing both `Recordable<SpaceRecord>` and
      `Recordable<SpaceTimeRecord>` as `Unrecordable` / `WrongVariant`
- [ ] 7.8 `recordable.rs` beside `Contextoid` implementing `Recordable<NodeRecord>`, dispatching to
      the four parameters, `Root` both ways, the phantom arm as `Unrecordable`
- [ ] 7.9 `recordable_tests.rs` beside every implementing type: round trip at a value where every
      field differs; every wrong variant refused with the node's identifier; `Float106` and
      `BFloat16` coordinate round trips on one space type; the root round trip

## 8. `Context::snapshot` and `Context::restore`

- [ ] 8.1 `context_graph/snapshot.rs`: the walk over base and extras through `get_last_index`,
      `get_node`, `get_edges`; canonical ordering; each extra's identifier and name recorded
- [ ] 8.2 `context_graph/restore.rs`: version check, duplicate and dangling-identifier checks,
      extra identifier 0 refused, base and extras rebuilt through the graph traits under the
      snapshot's identifiers and names, current extra identifier left at 0
- [ ] 8.3 `snapshot_tests.rs` and `restore_tests.rs`: the canonical-order scenario, the
      identifier-not-index scenario, the unrecordable-node scenario, the full `UniformContext`
      round trip with two named extras and all four relation kinds, the run-time-state scenario,
      the newer-version refusal, the dangling edge, the duplicate identifier, the
      two-hydrations-agree scenario

## 9. `ContextStore`, errors, aliases

- [ ] 9.1 `src/errors/store_error.rs`: `StoreError<E, B = Infallible>` with `Storage`, `Substrate`,
      `Projection`, `Display`, `std::error::Error`, `From<ProjectionError>`; tests
- [ ] 9.2 `context_store/mod.rs`: `ContextStore<S>` with `new`, `storage`, `reserve`
- [ ] 9.3 `context_store/hydrate.rs`: backend `hydrate` then `restore`
- [ ] 9.4 `context_store/store_branch.rs`: snapshot, `lookup`, partition into create / share /
      remap, `reserve` for remaps, edge remapping, `create_node`, `create_edge`,
      `create_context(name)`, `link`; the same per extra under the extra's name, then `attach`;
      returns the base container's identifier
- [ ] 9.5 `context_store/substrate.rs`: `create_node_via` and `hydrate_via`
- [ ] 9.6 `src/alias/mod.rs`: `SubstrateContext` and `SubstrateContextoid`
- [ ] 9.7 `tests/types/context_types/context_store/*_tests.rs` over `MemoryStorage` and
      `MemorySubstrate` via the store crate's `utils_test`: every scenario in
      `context-store-facade`, including the struct payload through the substrate and the loud
      `WrongPayload`. Add the `ctx_types_context_store_tests` suite to `BUILD.bazel`

## 10. Streaming on the context side

- [ ] 10.1 `context_graph/apply.rs`: `Context::apply` with the routing table from
      `context-store-streaming`; extra removal and empty-extra creation through the private map;
      every path idempotent
- [ ] 10.2 `context_store/subscribe.rs`: `impl<S: ContextStorageStream> ContextStore<S>` with
      `subscribe` restoring the snapshot and returning the stream
- [ ] 10.3 `apply_tests.rs` and `subscribe_tests.rs`: every scenario in `context-store-streaming`
      including both fixed-scope scenarios, plus a `compile_fail` doc test that a `ContextStore`
      over a store-only backend has no `subscribe`

## 11. Documentation

- [ ] 11.1 `AGENTS.md`: crate count 31 to 32, root list gains `_context_store`, Tier 0 gains
      `deep_causality_context_store`, `deep_causality_context`'s entry lists it; re-derive the
      block from the manifests rather than editing by hand
- [ ] 11.2 `deep_causality_context_store/README.md`: what the crate is, the record vocabulary and
      the `DataRecord` value tree, the thirteen operations, references between containers, how a
      backend implements the trait, the precision statement, the in-memory backend for tests;
      written for a backend implementor
- [ ] 11.3 `deep_causality_context/README.md`: a persistence section naming `Storable` with a
      struct example, `ContextStore`, `snapshot` / `restore`, `SubstrateContext`, the named extras
      and the precision statement including the `Float106` exception; the re-exported vocabulary
      listed in Contents
- [ ] 11.4 Crate-level docstring for the store crate's `lib.rs` stating what the code does, with
      no history

## 12. Verification

- [ ] 12.1 `cargo test -p deep_causality_context_store` and `cargo test -p deep_causality_context`
      green; coverage of every added or edited file at the floor, `utils_test` included
- [ ] 12.2 `make format && make fix`; lints fixed by rewriting, not by `#[allow]`
- [ ] 12.3 `bazel test //...` green; every `*_tests.rs` under both crates' `tests/` matched by a
      suite glob
- [ ] 12.4 `cargo build -p deep_causality_core --no-default-features --features no-std` still
      succeeds
- [ ] 12.5 Confirm the store crate's `[dependencies]` is empty and that `cargo tree -p
      deep_causality_context_store` shows the crate alone
- [ ] 12.6 Confirm `grep -rn "u64" deep_causality_context_store/src` finds no identifier position
      outside the `IdentificationValue` declaration
- [ ] 12.7 Run `scripts/sbom.sh` so the new crate's SBOM pair exists
- [ ] 12.8 `openspec validate --changes` passes

## 13. Hand-off

- [ ] 13.1 Prepare commit messages per task group, with the breaking change to
      `ExtendableContextuableGraph`, the behaviour change to `extra_ctx_add_new` and the vocabulary
      relocation stated in the footers for release-plz, and hand them to the user to commit
