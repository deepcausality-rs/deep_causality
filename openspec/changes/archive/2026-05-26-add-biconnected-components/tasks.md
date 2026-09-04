## 0. Stage gate (applies between every numbered section below)

Each section ends with a hard gate. Do **not** start the next section until all four checks pass on the current state of the crate. If any check fails, fix the cause before proceeding — do not stack work on a broken stage.

- `cargo build -p ultragraph` — compiles clean, zero warnings escalated by the crate's lint config.
- `cargo test -p ultragraph` — every test in the crate passes (not just the ones added in the current section).
- `cargo clippy -p ultragraph --all-targets -- -D warnings` — zero clippy findings. Fix the code, do not `#[allow]` the lint (per project memory).
- `cargo fmt -p ultragraph -- --check` — zero formatting diffs.

Each section below ends with an explicit `Gate` checkbox that asserts all four passed before moving on.

## 1. Trait surface

- [x] 1.1 Extend `StructuralGraphAlgorithms<N, W>` in `ultragraph/src/traits/graph_algo_structural.rs` with three new method signatures: `articulation_points`, `bridges`, `biconnected_components` (no default bodies; signatures per spec).
- [x] 1.2 Verify no out-of-tree implementer of `StructuralGraphAlgorithms` exists in the monorepo (`rg "impl .* StructuralGraphAlgorithms"`). Confirm only `CsmGraph` (+ `UltraGraph` facade) implements it.
- [x] 1.3 Add `unimplemented!()` stub bodies on the `CsmGraph` impl so the crate still compiles. (These are replaced in §§3–5 and must never be left in place past the gate of their owning section.)
- [x] 1.4 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 2. Shared undirected-view helper

- [x] 2.1 Add private helper module `ultragraph/src/types/storage/graph_csm/graph_csm_biconnectivity_common.rs` exposing a function that builds a symmetric CSR adjacency `(offsets: Vec<usize>, targets: Vec<usize>, edge_id: Vec<usize>)` from `self.forward_edges`, dropping self-loops and deduplicating parallel reverse edges. Each entry in `targets` carries a stable `edge_id` so paired half-edges share an id (used for parent-edge tracking in bridges / biconnected components).
- [x] 2.2 Register the new module in the `graph_csm` `mod.rs`.
- [x] 2.3 Unit-test the helper directly (it lives in `src/`, so per `AGENTS.md` it must reach 100% coverage): empty graph, isolated vertices only, self-loop only, single directed edge, both directions stored, multi-edges in input, mixed.
- [x] 2.4 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 3. articulation_points implementation

- [x] 3.1 Create `ultragraph/src/types/storage/graph_csm/graph_csm_algo_articulation_points.rs` with the `articulation_points` impl on `CsmGraph<N, W>`.
- [x] 3.2 Use iterative DFS over the symmetric adjacency (helper from §2). Track `dfs_num`, `low_link`, parent **edge id**, and child counts. Mark `u` as articulation if it is a DFS root with ≥2 tree children, or a non-root with any child `v` where `low_link[v] >= dfs_num[u]`.
- [x] 3.3 Sort the result ascending; deduplicate; return.
- [x] 3.4 Register the new file in `graph_csm/mod.rs`.
- [x] 3.5 Add a minimal smoke test for `articulation_points` (path graph + cycle graph) so the algorithm is exercised at this stage's gate. Full scenario coverage lands in §8.
- [x] 3.6 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 4. bridges implementation

- [x] 4.1 Create `ultragraph/src/types/storage/graph_csm/graph_csm_algo_bridges.rs` with the `bridges` impl on `CsmGraph<N, W>`.
- [x] 4.2 Iterative DFS over the symmetric adjacency, parent **edge id** tracking. For tree edge `u -> v`, after the child returns, if `low_link[v] > dfs_num[u]` emit `(min(u,v), max(u,v))`.
- [x] 4.3 Sort the resulting `Vec<(usize, usize)>` lexicographically before returning.
- [x] 4.4 Register the new file in `graph_csm/mod.rs`.
- [x] 4.5 Add a minimal smoke test for `bridges` (tree + cycle).
- [x] 4.6 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 5. biconnected_components implementation

- [x] 5.1 Create `ultragraph/src/types/storage/graph_csm/graph_csm_algo_biconnected_components.rs` with the `biconnected_components` impl on `CsmGraph<N, W>`.
- [x] 5.2 Iterative DFS over the symmetric adjacency. Maintain an edge stack of `(u, v, edge_id)`. When an articulation condition `low_link[v] >= dfs_num[u]` fires (or on DFS completion of a root), pop the edge stack down to the entry pushed when `(u, v)` was traversed and emit the set of distinct endpoints as one biconnected component.
- [x] 5.3 For each emitted component: collect vertices via a small per-component `HashSet<usize>` (or vertex bit-set sized to V), sort ascending, push to the result.
- [x] 5.4 Exclude isolated vertices (degree 0 in the symmetric view) — they belong to no component per spec.
- [x] 5.5 Register the new file in `graph_csm/mod.rs`.
- [x] 5.6 Add a minimal smoke test for `biconnected_components` (cycle + bow-tie).
- [x] 5.7 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 6. UltraGraph facade pass-through

- [x] 6.1 Add three forwarder methods in `ultragraph/src/types/ultra_graph/graph_algo.rs` delegating to the inner `CsmGraph` exactly as `strongly_connected_components` is forwarded today.
- [x] 6.2 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 7. Public re-exports

- [x] 7.1 Confirm `StructuralGraphAlgorithms` is re-exported from `ultragraph/src/lib.rs` (it already is via the existing path); no new exports needed since the methods land on the existing trait.
- [x] 7.2 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green.

## 8. Tests

- [x] 8.1 Create test directory `ultragraph/tests/types/storage/graph_csm/` (matching src layout if not already present); add `mod.rs` files at each level and register them up to the crate test root.
- [x] 8.2 Add `graph_csm_algo_articulation_points_tests.rs` covering every scenario in `specs/ultragraph-biconnectivity/spec.md` under "articulation_points API" plus: K4 (no articulation), star graph (center is articulation), self-loop-only graph (empty result), disconnected graph.
- [x] 8.3 Add `graph_csm_algo_bridges_tests.rs` covering every "bridges API" scenario plus: K4 (no bridges), star graph (every edge is a bridge), parallel-edge input verifying canonical `(min, max)` form.
- [x] 8.4 Add `graph_csm_algo_biconnected_components_tests.rs` covering every "biconnected_components API" scenario plus: K4 (single component `{0,1,2,3}`), nested-cycle graph, ensure isolated vertices excluded.
- [x] 8.5 Add a `graph_csm_algo_biconnectivity_cross_consistency_tests.rs` that, for each fixture used above, asserts the two cross-API invariants from the spec ("articulation iff in ≥2 components", "bridge iff 2-vertex component"). Use a small property-style loop over the fixture set; do not introduce a new dependency (use plain Rust assertions).
- [x] 8.6 Add an UltraGraph-facade test verifying the three forwarders return the same values as calling the underlying `CsmGraph` directly.
- [x] 8.7 Register every new test file in the corresponding `mod.rs` with `#[cfg(test)]` and in `ultragraph/tests/BUILD.bazel`.
- [x] 8.8 **Gate**: `build` + `test` + `clippy -D warnings` + `fmt --check` all green; smoke tests from §§3.5/4.5/5.6 either superseded (remove if duplicated) or retained.

## 9. Final verification

- [x] 9.1 Crate-local gate re-run: `cargo build -p ultragraph` + `cargo test -p ultragraph` + `cargo clippy -p ultragraph --all-targets -- -D warnings` + `cargo fmt -p ultragraph -- --check` — all green.
- [x] 9.2 Monorepo-wide check (per `AGENTS.md`: required when ≥3 crates change, optional here, but cheap): `make format && make fix` clean, `make build` green, `make test` green.
- [x] 9.3 Coverage check on the four new `src/` files (3 algorithms + 1 common helper) reaches 100% line coverage; any unreachable branches documented in-file.
- [x] 9.4 Cross-API consistency assertions in §8.5 pass on every fixture.
- [x] 9.5 No `unimplemented!()` / `todo!()` / `#[allow(clippy::...)]` left in the new code (grep before sign-off).
- [x] 9.6 Prepare commit message referencing issue #581 and present it to the user for commit (per project Golden Rule: never `git commit`).
