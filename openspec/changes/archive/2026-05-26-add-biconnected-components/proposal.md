## Why

Reachability-based fault-impact analysis on `CsmGraph` currently costs O(V·(V+E)) per pass. At ServiceRadar's working scale (~10K nodes once backbone + attachment + virtualization links are included), that is roughly 10⁹ operations per full sweep — affordable as a nightly batch but not on every topology delta, which forces consumers to operate on stale predictions. Tarjan's classical articulation-point / bridge / biconnected-component decomposition collapses the same answer to a single O(V+E) DFS, turning "recompute hourly and hope nothing changed" into "recompute on every topology change."

## What Changes

- Add three new structural graph-analysis APIs on `CsmGraph` (and surface them through the `GraphAlgoStructural` trait):
  - `articulation_points(&self) -> Result<Vec<usize>, GraphError>`
  - `bridges(&self) -> Result<Vec<(usize, usize)>, GraphError>`
  - `biconnected_components(&self) -> Result<Vec<Vec<usize>>, GraphError>`
- All three operate with **undirected semantics**. When invoked on a directed `CsmGraph`, the implementation internally treats each directed edge `(u, v)` as the undirected edge `{u, v}` for the duration of the pass. No `directed: bool` flag is exposed — directed analogues (dominators, SCC) are different algorithms and are not in scope.
- Each API is an independent Tarjan DFS pass, each O(V + E), matching the file/module shape already used by `find_cycle` and `strongly_connected_components`.
- No breaking changes. Pure additive surface on `CsmGraph` and its trait.

## Capabilities

### New Capabilities

- `ultragraph-biconnectivity`: Structural decomposition of an undirected view of a `CsmGraph` into articulation points (cut vertices), bridges (cut edges), and biconnected components (maximal 2-vertex-connected edge sets). Defines the contract for the three new APIs, their undirected-view semantics, complexity guarantees, and error/empty-graph behavior.

### Modified Capabilities

_None._ No existing requirements change; this is additive.

## Impact

- **Crate:** `ultragraph` only.
- **Source files added** (one algorithm per module, per `AGENTS.md`):
  - `ultragraph/src/types/storage/graph_csm/graph_csm_algo_biconnectivity.rs` (or a sibling module of the existing `graph_csm_algo_structural.rs`)
  - Trait extension in `ultragraph/src/traits/graph_algo_structural.rs` (additive method signatures with no default body, or a new sibling trait if symmetry with existing groupings prefers it — decided in `design.md`).
- **Tests:** new files under `ultragraph/tests/` mirroring the src layout, registered in the corresponding `mod.rs` and in `ultragraph/tests/BUILD.bazel`. 100% line coverage of the new code, per project policy.
- **Public API:** new methods on `CsmGraph` re-exported from `ultragraph::lib.rs` via the existing trait export path. No removals, no signature changes to existing items.
- **Dependencies:** none added. Pure-Rust, no `unsafe`, no macros in `src/`, static dispatch only — consistent with crate conventions.
- **Downstream consumers** (`deep_causality`, ServiceRadar integration): unblocks per-delta structural recompute; no required migration.
