## Context

`CsmGraph<N, W>` is the immutable, CSR-backed structural graph in the `ultragraph` crate. Its existing structural-analysis API (`StructuralGraphAlgorithms` in `ultragraph/src/traits/graph_algo_structural.rs`) currently exposes a single method, `strongly_connected_components`, implemented in `ultragraph/src/types/storage/graph_csm/graph_csm_algo_structural.rs` as an iterative Tarjan DFS over `forward_edges.offsets` / `forward_edges.targets`.

Issue #581 asks for three new structural decompositions — articulation points, bridges, biconnected components — to replace an existing O(V·(V+E)) per-candidate reachability sweep used downstream (ServiceRadar fault-impact analysis) with a single O(V+E) Tarjan pass. The proposal locks two upstream decisions: (a) the API is always undirected (no `directed: bool`), and (b) three independent passes rather than one fused pass.

Constraints from `AGENTS.md` and the existing crate layout:

- One algorithm per file under `ultragraph/src/types/storage/graph_csm/`; tests mirror the path under `ultragraph/tests/`, registered in the corresponding `mod.rs` and in `ultragraph/tests/BUILD.bazel`.
- No `unsafe`, no `dyn`, no macros in `src/`, no new external dependencies.
- 100% line coverage on added code.

## Goals / Non-Goals

**Goals:**

- Add `articulation_points`, `bridges`, `biconnected_components` on `CsmGraph<N, W>` with the contract defined in `specs/ultragraph-biconnectivity/spec.md`.
- Each method runs in O(V + E) time and O(V + E) auxiliary space (the symmetric adjacency view dominates; per-pass DFS state is O(V)).
- Match the existing iterative-DFS shape used by `strongly_connected_components` so the three new files are stylistically isomorphic and reviewable side-by-side.
- Deterministic, sorted output (per-component vertex order, canonical `(min, max)` for bridges) so downstream code never needs to renormalize.

**Non-Goals:**

- Not introducing directed analogues (dominators, directed strong-connectivity beyond the existing SCC API). Out of scope by explicit decision.
- Not unifying the three passes into a single fused traversal. Explicit decision per issue and per user confirmation; the marginal speedup is not worth the coupling.
- Not adding incremental / online maintenance. The APIs operate on an immutable `CsmGraph` snapshot, matching every other algorithm in this module.
- Not changing `DynamicGraph` or the mutable storage path. The decomposition lives on the CSR snapshot.
- Not adding new error variants. Reuse the existing `GraphError` shape.

## Decisions

### D1. Always-undirected semantics via on-the-fly symmetric adjacency

Each of the three algorithms first materializes a **symmetric CSR adjacency** from `self.forward_edges`: for every stored `(u, v)` with `u != v`, contribute both `(u, v)` and `(v, u)` (deduplicated). Self-loops are dropped at this step. The DFS then operates over this symmetric view.

- **Why over alternatives:**
  - *Alternative A — call sites pre-symmetrize:* leaks the requirement to every caller; defeats the "drop-in undirected analysis" goal.
  - *Alternative B — walk `forward_edges` plus `backward_edges` jointly:* `CsmGraph` does maintain reverse adjacency, but using it requires per-step deduplication during the DFS (every directed edge `(u, v)` paired with reverse `(v, u)` would otherwise be seen as two undirected edges and break bridge detection's "skip parent edge by edge-id" trick). Cleaner to symmetrize once.
  - *Alternative C — represent each undirected edge once via canonical ordering:* requires non-trivial bookkeeping per DFS step to find "the other direction" and complicates the bridge low-link comparison. Symmetric CSR is the simplest correct shape.
- **Cost:** one extra O(V + E) build per call. Within the O(V + E) budget of each algorithm — does not change asymptotic complexity. Memory is O(V + E') where E' is the symmetric edge count (≤ 2E).
- The symmetric-adjacency construction is factored into a single private helper in a shared module (`graph_csm_algo_biconnectivity_common.rs` or a free function in the same `graph_csm` module) so all three algorithms call the same code. This keeps the three pass files focused on the Tarjan logic and gives one place to test the symmetrization corner cases (self-loops, parallel reverse edges, isolated vertices).

### D2. Three independent files, three independent passes

Per `AGENTS.md` ("one type, one module") and the issue's preference:

- `graph_csm_algo_articulation_points.rs`
- `graph_csm_algo_bridges.rs`
- `graph_csm_algo_biconnected_components.rs`

Each file holds exactly one trait-method impl. The three files share only the symmetric-adjacency helper (D1).

- **Why over a fused pass:** the fused Tarjan variant that emits all three answers at once is well-known and saves ~2×. At Tarjan's O(V+E) baseline, that is sub-millisecond for the 10K-node target. Three independent files are dramatically easier to review, test in isolation, and reason about; they also let `bridges` and `articulation_points` be specialized (`bridges` does not need the component-stack bookkeeping, `articulation_points` does not need the edge stack). Locality of concern wins.

### D3. Iterative DFS, edge-indexed parent tracking

Each implementation uses an explicit `Vec<(node, slice::Iter)>` stack mirroring the SCC implementation (no recursion — Rust stack overflow risk on deep graphs is the same here as it is for SCC; the existing crate already chose iterative for this reason).

Bridge detection requires distinguishing "the edge we arrived on" from "a true back edge to the parent." The standard fix — tracking the parent **vertex** — fails on multi-edges. We therefore track the parent **edge index** into the symmetric adjacency array, which is unambiguous regardless of multi-edges. This is the textbook fix and costs one extra `usize` per stack frame.

- **Alternative considered:** "parent vertex" tracking. Rejected because the spec contract is silent on multi-edges in the input, but `CsmGraph` can in principle carry them, and the symmetrization step itself can create two parallel undirected edges if both `(u, v)` and `(v, u)` were stored. Even though D1 deduplicates, deduplication is conservative; relying on it would couple two pieces of logic. Parent-edge-index is robust either way.

### D4. Output canonicalization

- `articulation_points`: returned in ascending vertex order (single `Vec::sort` at the end — O(V log V) is well under the V+E budget for practical V).
- `bridges`: each tuple `(u, v)` is normalized to `(min(u, v), max(u, v))` at insertion time; the result vector is sorted lexicographically before return.
- `biconnected_components`: inside each component the vertex set is collected via the edge stack, deduplicated (vertex bit-set on the symmetric view's V), and sorted ascending. Outer ordering is the natural pop order of the algorithm (root component last per DFS tree), which is deterministic for a deterministic CSR.

Determinism is explicit so test scenarios in the spec can compare against literal expected values without bespoke set comparisons.

### D5. Trait surface extension, not a new trait

The three methods are added to the existing `StructuralGraphAlgorithms<N, W>` trait in `ultragraph/src/traits/graph_algo_structural.rs`. They are not given default bodies; the impl on `CsmGraph` provides them in the three new files.

- **Why not a new `BiconnectivityAlgorithms` trait:** the three methods are structural decompositions of the same nature as SCC. Splitting traits would force consumers to import a second name and complicate the `UltraGraph` facade in `ultragraph/src/types/ultra_graph/graph_algo.rs`. The existing trait is already named for this category.
- **Breaking-change consideration:** adding required (no-default) methods to a public trait is technically breaking for any external impl of `StructuralGraphAlgorithms`. Inside this monorepo only `CsmGraph` (and a thin pass-through on `UltraGraph`) implements it. The breakage is bounded and is part of the additive feature.

### D6. UltraGraph facade pass-through

`ultragraph/src/types/ultra_graph/graph_algo.rs` currently forwards `strongly_connected_components` from `UltraGraph` to its underlying `CsmGraph`. The three new methods get analogous one-line forwarders so external callers using `UltraGraph` see the same surface.

### D7. Error model: reuse existing variants only

The new code returns `GraphError::AlgorithmError(&'static str)` for internal invariant violations (matching the SCC implementation pattern). No new variants. The spec already pins this contract ("no new error variants").

## Risks / Trade-offs

- **Risk:** per-call symmetric-adjacency rebuild duplicates work if a caller invokes all three APIs back-to-back → **Mitigation:** acceptable for v1 — three O(V+E) builds at 10K nodes is microseconds; a future optimization can expose a `BiconnectivityReport` that runs all three in one pass and reuses the symmetric adjacency, without removing the three convenience methods.
- **Risk:** trait change is technically breaking for any out-of-tree impl → **Mitigation:** documented in proposal Impact; no known external implementers; alternative (separate trait) was considered and rejected for facade complexity.
- **Risk:** iterative DFS bookkeeping for biconnected components (edge stack popping per articulation discovery) is error-prone → **Mitigation:** the spec defines six concrete biconnected-components scenarios covering cycle, bow-tie, bridge-joined cycles, tree, empty, and disconnected — each becomes a test case. Cross-API consistency requirement (spec) acts as a property check: the test suite asserts agreement between the three APIs on every fixture, catching divergence even on inputs not specifically targeted.
- **Risk:** unbounded recursion if anyone "refactors back" to recursive DFS → **Mitigation:** match the SCC file's iterative pattern exactly; reviewers will catch a deviation. A short top-of-file comment on each new file notes the parent-edge-index requirement so the reason for the slightly more complex stack frame is not forgotten.
- **Trade-off:** sorting outputs costs O(V log V) we do not strictly need. Chosen anyway for deterministic test fixtures and for caller ergonomics; the cost is dwarfed by the DFS itself.
