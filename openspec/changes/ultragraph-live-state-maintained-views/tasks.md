## 0. Ground rules for every group

Each implementation group runs the five phases of `openspec/specs/unified-math-tdd-protocol/spec.md`
in order (design D7) and records them in `openspec/notes/ultragraph/tdd/live-group-<n>.md`: the A–K
table with a test named per row, the phase-2 failing run with its count, the phase-3 defect audit and
the phase-5 mutation report. Measurements follow the three-run protocol (design D6) and are recorded
under `openspec/notes/ultragraph/BenchStorage/`. Each group ends with `bazel test //...` green, clippy
clean with no `allow`, and a commit message shown to the user; the user commits.

- [ ] 0.1 Confirm `ultragraph-stable-ids-keyed-edges` is implemented and its after-benchmark `openspec/notes/ultragraph/BenchAfterChange1StableId/BenchAfterChange1StableId.md` exists;
      it is the incumbent every decision here measures against

## 1. Benchmark workloads for the decisions

- [ ] 1.1 Add the workloads of `ultragraph-edge-storage-benchmark` that the incumbent can run: keyed
      update and removal on low-degree nodes and on a hub of 10⁵ edges, and dynamic traversal over the
      ordered and shuffled chains; add the `#[doc(hidden)]` memory accounting in
      `src/utils/utils_tests.rs` with its tests, and record the incumbent's memory per edge
- [ ] 1.2 `cargo bench -p ultragraph -- --test` passes for every new group; record the incumbent's
      three runs and aggregate them
- [ ] 1.3 Commit message for group 1

## 2. The two challengers: the snapshot-log arrays and the indexed lists

- [ ] 2.1 Phase 1: a crate-internal storage trait over the Dynamic state's edge lists, implemented by
      the incumbent, and the snapshot-log `EdgeArray` (design D5) with `unimplemented!()` bodies,
      selected by the temporary Cargo feature `edge-log`
- [ ] 2.2 Fill the A–K table for the candidate (rows E: the log at, below and above its trigger; F: a
      weight of 0; K: the four payload kinds)
- [ ] 2.3 Phase 2: tests through the graph's public API, which is all the repository's tests reach,
      against the model of edges as `(target, key)` to payload, driving a node through every operation
      and across every compaction trigger; the complete change-1 suite runs with `--features edge-log`;
      observe failing; record
- [ ] 2.4 Phase 3: inject the design D7 defects of this group (a duplicate insert kept by an all-insert
      compaction; a zero weight read as a removal; compaction one entry early or late; a `Remove`
      suppressing a newer `Put`; snapshot entries to a tombstone kept) and the nine generic classes;
      record; discard the throwaway
- [ ] 2.5 Phase 4: implement the candidate; `cargo test -p ultragraph` passes without and with
      `--features edge-log` with the same executed count; full coverage under each
- [ ] 2.6 Phase 5: `scripts/mutants.sh` over the candidate's files; resolve every survivor
- [ ] 2.7 Commit message for the snapshot-log candidate
- [ ] 2.8 Phase 1: the indexed-lists candidate (design D5b) behind the storage trait with
      `unimplemented!()` bodies, selected by the temporary Cargo feature `edge-index`; `edge-log` and
      `edge-index` together are a `compile_error!`
- [ ] 2.9 Fill the A–K table for the candidate (rows E: a list at `HUB_DEGREE`, one below and one
      above, and falling back through half of it; K: the four payload kinds)
- [ ] 2.10 Phase 2: tests through the public API against the same model, driving a node across the
      threshold in both directions with keyed inserts, removals, `remove_edge`, `compact_ids`,
      `from_parts`, `freeze` and `unfreeze`, the memory accounting showing the index present and
      absent; the complete change-1 suite runs with `--features edge-index`; observe failing; record
- [ ] 2.11 Phase 3: inject the design D7 index defects (an entry not re-recorded after a swap-remove;
      the index built one below or above the threshold or not dropped below half of it; a stale index
      after `compact_ids` or `unfreeze`; a lookup answered by the index where the list disagrees) and
      the nine generic classes; record; discard
- [ ] 2.12 Phase 4: implement the candidate; `cargo test -p ultragraph` passes without a feature, with
      `--features edge-log` and with `--features edge-index`, all with the same executed count; full
      coverage under each
- [ ] 2.13 Phase 5: `scripts/mutants.sh` over the candidate's files; resolve every survivor
- [ ] 2.14 Commit message for the indexed-lists candidate

## 3. The edge-storage decision

- [ ] 3.1 Sweep `MIN_LOG` and `HUB_DEGREE` for memory per edge and hub update time; fix each by the
      1.25× memory gate of `ultragraph-edge-storage-benchmark`; three runs per candidate at its value;
      aggregate into `BenchStorage/`
- [ ] 3.2 Apply the rule of `ultragraph-edge-storage-benchmark`; record the outcome and its rows in
      `openspec/notes/ultragraph/RadixGraphUpdate.md` §Benchmark
- [ ] 3.3 Ask the user to approve removing the losing candidates; remove them and their features, and
      keep the trait only if it still serves the Live state
- [ ] 3.4 Commit message for group 3

## 4. The Live state, its transitions and freeze_clone

- [ ] 4.1 Phase 1: `GraphState::Live`, `is_live`, the transition into Live, Live's handling in every
      `GraphView`, `GraphMut` and `GraphTraversal` method and in the adjacency trait,
      `GraphError::GraphIsLive` for `compact_ids` in Live, and `freeze_clone`, all with
      `unimplemented!()` bodies where new
- [ ] 4.2 Fill the A–K table for Live (rows A–F, H, K of design D7)
- [ ] 4.3 Phase 2: a test for every scenario of `ultragraph-three-states`; extend the model-based test
      with every transition, `freeze_clone` and mutations in Live; add the Live-versus-freshly-frozen
      agreement test; observe failing; record
- [ ] 4.4 Phase 3: inject the design D7 Live defects (a rebuilt row without spare capacity; a relocated
      row's record not updated; the mask bit not cleared; stale entries read through a cleared mask;
      `freeze_clone` changing the source; `compact_ids` accepted in Live) and the nine generic classes; record; discard
- [ ] 4.5 Phase 4: implement the arena, records and mask, every transition, `freeze_clone`, and the
      algorithms through the adjacency trait in Live; full coverage; matching `cargo test` and
      `bazel test //...` counts
- [ ] 4.6 Phase 5: `scripts/mutants.sh` over the group's files; resolve every survivor
- [ ] 4.7 Commit message for group 4

## 5. The Live decision

- [ ] 5.1 Add the Live workloads (algorithms in Live, Live updates in place and with rebuilds, the
      transitions, and k = 1, 16 and 256 keyed writes followed by `strongly_connected_components` in
      Live and through Dynamic → `freeze` → algorithm → `unfreeze`, at 1,000 nodes and on the hub)
      and run three times; aggregate into `BenchStorage/`
- [ ] 5.2 Apply the Live rule of `ultragraph-edge-storage-benchmark` and record it with its rows
- [ ] 5.3 If Live fails its rule, stop and ask the user before removing it and before group 6
- [ ] 5.4 Commit message for group 5

## 6. materialize and maintained views

- [ ] 6.1 Phase 1: `materialize` on the projection view and the container's `ParallelEdges` mode
      (design D8); the container's `P: Projection<N, W>`
      parameter with its default; `maintain`, `view` and `ViewId`, with `unimplemented!()` bodies;
      confirm `UltraGraph<T>` and `UltraGraphWeighted<T, W>` still compile unchanged in every calling
      crate
- [ ] 6.2 Fill the A–K table for views (rows A–C, F, H, K of design D7)
- [ ] 6.3 Phase 2: a test for every scenario of `ultragraph-materialized-projection`,
      `ultragraph-maintained-views` and the `ultragraph-projection-view` delta; extend the model-based test with four maintained views of
      different tests and modes, checked after every step against a projection rebuilt from the model;
      observe failing; record
- [ ] 6.4 Phase 3: inject the design D7 view defects (a view not updated on an edge mutation, on a node
      payload change, or on `clear`; a view updated outside its scope; `freeze_clone` copying views;
      `materialize` sharing storage; a materialised `Distinct` copy answering under `Merge`; a
      transition dropping a view; `compact_ids` leaving a view on the old positions) and the nine generic classes; record; discard
- [ ] 6.5 Phase 4: implement `materialize`, view registration, eager maintenance on every mutation in
      the Dynamic and Live states, the view rebuild in `compact_ids`, and reads in every state; full coverage; matching counts
- [ ] 6.6 Phase 5: `scripts/mutants.sh` over the group's files; resolve every survivor
- [ ] 6.7 Commit message for group 6

## 7. Measurement and close-out

- [ ] 7.1 Add the view-upkeep workload (0, 1, 4 and 16 maintained views) and run the full benchmark
      three times; aggregate and compare with the incumbent; explain every move beyond a row's spread
- [ ] 7.2 Record the outcome of this change in `openspec/notes/ultragraph/RadixGraphUpdate.md`
      (§Design 4, 6, 7 and §Benchmark) as it was built
- [ ] 7.3 Full workspace: `make format && make fix`, `make build`, `bazel test //...`,
      `make check_examples`
- [ ] 7.4 Commit message for group 7, with the migration notes for external callers (`GraphState::Live`
      and the container's projection type parameter) in its body
