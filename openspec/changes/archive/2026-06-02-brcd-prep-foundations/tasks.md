## 1. Tier A — numeric primitives (`linalg-numeric-primitives`)

- [x] 1.1 Moved `cg_solve` + `CgFailure` to `deep_causality_sparse::solver::cg`, made them public, re-exported from the crate root, with doc-comments; `[lints] workspace = true` was already present. CG unit tests relocated to `tests/solver/cg_tests.rs` and registered in `tests/mod.rs` + `tests/BUILD.bazel`.
- [x] 1.2 `deep_causality_topology` now consumes `deep_causality_sparse::{cg_solve, CgFailure}`; the crate-local copy is removed (the gauge helper `subtract_mean_in_place` stays in `utils/cg_solver.rs`). All 1040+ topology tests still pass.
- [x] 1.3 Added `CausalTensorStatsExt` (`sample_mean`, `sample_covariance` with `ddof = 1`) over a 2-D `CausalTensor` in `deep_causality_tensor`, generic over `T: RealField + FromPrimitive`. 12 tests (f64 + f32) cover the closed form, per-column variance, uniform→zero, symmetry, and the rejection paths (non-2D → `DimensionMismatch`, empty/zero-column → `EmptyTensor`, single observation → `InvalidParameter`).
- [x] 1.4 Topology's `Manifold::covariance_matrix_impl` now delegates the variance computation to `sample_covariance` (reshaping the field into an `n × 1` matrix); the duplicate mean/variance loop is removed. Its `n < 2` guard and message are preserved; all 1040 topology tests pass with identical values.
- [x] 1.5 Added `logsumexp` (max-shift) to `CausalTensorStatsExt` in `deep_causality_tensor`, generic over `T: RealField`. Tests cover naive agreement for small inputs, no-overflow for large inputs, single-element, empty (`−∞`), and non-finite-max. *(Placement: owner directed these into `deep_causality_tensor`'s stats extension rather than `deep_causality_num` — `num` is the number-systems/algebra crate; statistics computed over data belong with `sample_mean`/`sample_covariance` in tensor.)*
- [x] 1.6 Added `gaussian_log_density` (element-wise over a tensor) to `CausalTensorStatsExt`, generic over `T: RealField + FromPrimitive`. Tests cover closed-form agreement, element-wise correctness, the zero- and negative-variance floor, and the empty-tensor case.
- [x] 1.7 Added `conditional_variance` (covariance Schur complement with ridge, via in-place Cholesky) to `CausalTensorStatsExt`, taking the covariance tensor as `self`. Tests cover the empty-parent marginal case, a one-parent closed form, a two-parent identity block, the singular-block ridge path, and rejection of non-square/non-2D/out-of-range inputs.
- [x] 1.8 Every Tier A primitive is generic over `T: RealField` with no hardwired `f64`. Added a precision-sweep test (`causal_tensor_ext_stats_sweep_tests.rs`) running mean, covariance, `logsumexp`, `gaussian_log_density`, and `conditional_variance` at `f32`, `f64`, and `Float106` via one generic body; `cg_solve` already covered f32/f64 in sparse.

## 2. Tier B — causal-graph layer (`causal-graph`, in `deep_causality_algorithms::causal_discovery`)

- [x] 2.1 Confirmed: `ultragraph`'s `GraphTraversal::inbound_edges(node)` already returns the direct predecessors (parents) of a node on a frozen graph, and is already unit-tested — `test_inbound_edges_on_static_graph` asserts predecessor semantics (e.g. `inbound_edges(2) == [0, 1]`), plus `_on_dynamic_graph` (`GraphNotFrozen`) and `_invalid_node` error paths. No code change needed; the causal-graph layer will consume `inbound_edges` for parent sets.
- [x] 2.2 Add `deep_causality_topology` as a dependency of `deep_causality_algorithms` (D2 resolved: accept it); scaffold `causal_discovery::brcd::{meek, validity, mec}` — **all BRCD code is rooted at the `brcd` module** (owner directive). The CPDAG/PDAG type is `topology::MixedGraph` — no graph type is defined here. *(The `mixed-graph` change is landed and archived: `MixedGraph` already provides directed arcs + undirected/bidirected/circle edges, constructors, getters — parents, neighbors, per-kind enumeration — and the endpoint invariant.)*
- [x] 2.3 ~~Implement topological order/acyclicity~~ **Provided by `MixedGraph`**: `topological_sort`, `has_cycle`, and `find_cycle` over the directed-arc projection ship with the type and are already tested. Confirm the Meek/validity operations consume them; no new implementation needed.
- [x] 2.4 Implement the Meek orientation rules over `MixedGraph` (PDAG→CPDAG completion, DAG→CPDAG), using its per-endpoint `set_endpoint`/`orient` and `parents`/`undirected_neighbors`; test a known completion, the arcs-only unchanged case, and idempotence.
- [x] 2.5 Implement the unshielded-collider validity check over `MixedGraph` against a baseline parent set; test new-collider flagging and pre-existing-collider non-flagging.
- [x] 2.6 Implement MEC size + representative DAG with the trivial arcs-only case (size 1 = input) over `MixedGraph`; document the extension point for a future uniform sampler; test the arcs-only case.
- [x] 2.7 Confirm the causal-graph operations carry no floating-point scalar and expose no `RealField` parameter (they operate on `MixedGraph`, which is structural); a downstream consumer at any precision reuses them unchanged.

## 3. CDL pipeline generalization — MOVED OUT

The CDL discovery-pipeline generalization (`DiscoveryOutcome<T>`, two-dataset
carriage, user-supplied domain graph, SURD preservation) is **no longer part of
this change.** It is a breaking change to `deep_causality_discovery`'s public API
whose shape is dictated by what BRCD needs at the seam, so it is sequenced
*after* the BRCD estimator is built and verified — not ahead of it.

- It is preserved as a design note at `openspec/notes/cdl-integration.md`
  (rationale + the four requirements + decisions D4/D5).
- It becomes its own change `cdl-discovery-pipeline`, landed last with the real
  `BrcdResult<T>` wired in as the second `DiscoveryOutcome` variant.

This change is therefore the two pure foundation layers only: Tier A
(`linalg-numeric-primitives`) and Tier B (`causal-graph`).

## 4. Verification and hygiene

- [x] 4.1 `cargo build -p` and `cargo test -p` for each touched crate (`deep_causality_tensor`, `deep_causality_sparse`, `deep_causality_topology`, `deep_causality_algorithms`); full coverage of new code (Tier A stats sweep + Tier B brcd suite).
- [x] 4.2 Register every new test file in its module tree and in the crate's `tests/BUILD.bazel`, per repo test conventions (sparse `cg_tests`, tensor `ext_stats`/sweep, algorithms `brcd` suite).
- [x] 4.3 Confirm no external numeric crate was added, `unsafe_code = "forbid"` is intact in every touched crate, and no `dyn`/trait-object was introduced.
- [x] 4.4 Run `make format && make fix`, then `make build` and `make test` (more than three crates changed).
- [x] 4.5 Run `openspec validate brcd-prep-foundations` and confirm the change is apply-complete; prepare a commit message and request the owner commit.
