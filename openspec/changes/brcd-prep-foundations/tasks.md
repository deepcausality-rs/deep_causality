## 1. Tier A — numeric primitives (`linalg-numeric-primitives`)

- [x] 1.1 Moved `cg_solve` + `CgFailure` to `deep_causality_sparse::solver::cg`, made them public, re-exported from the crate root, with doc-comments; `[lints] workspace = true` was already present. CG unit tests relocated to `tests/solver/cg_tests.rs` and registered in `tests/mod.rs` + `tests/BUILD.bazel`.
- [x] 1.2 `deep_causality_topology` now consumes `deep_causality_sparse::{cg_solve, CgFailure}`; the crate-local copy is removed (the gauge helper `subtract_mean_in_place` stays in `utils/cg_solver.rs`). All 1040+ topology tests still pass.
- [ ] 1.3 Add sample mean and sample covariance (`ddof = 1`) over a 2-D `CausalTensor` in `deep_causality_tensor`; unit-test against the closed form; guard the single-observation case.
- [ ] 1.4 Have topology's `Manifold` covariance delegate to the shared covariance (remove the duplicate); confirm topology tests pass with identical values.
- [ ] 1.5 Add a numerically stable `logsumexp` to `deep_causality_num`, generic over `T: RealField`; test small-input agreement with the naive form and large-input no-overflow.
- [ ] 1.6 Add a Gaussian log-density to `deep_causality_num`, generic over `T: RealField`; test closed-form agreement and the non-positive-variance floor.
- [ ] 1.7 Add a conditional-variance helper (covariance Schur complement with ridge) in `deep_causality_num`, generic over `T: RealField`; test empty-parent marginal case, a known multivariate-normal block, and the singular-block ridge path.
- [ ] 1.8 Confirm every Tier A primitive is generic over `T: RealField` (no hardwired `f64`); add a precision-sweep test running each at `f32`, `f64`, and `Float106`.

## 2. Tier B — causal-graph layer (`causal-graph`, in `deep_causality_algorithms::causal_discovery`)

- [ ] 2.1 Confirm or add a parents/predecessors accessor on `ultragraph`'s directed graph; unit-test it.
- [ ] 2.2 Scaffold `causal_discovery::{graph, mec}` beside `surd`; implement the PDAG/CPDAG type (directed arcs + undirected edges) with constructors and getters (parents, neighbors, arcs, undirected edges); test construction and arc-vs-edge adjacency.
- [ ] 2.3 Implement topological order and acyclicity over the arc projection by reusing `ultragraph`; test the acyclic and cyclic cases.
- [ ] 2.4 Implement the Meek orientation rules (PDAG→CPDAG completion, DAG→CPDAG); test a known completion, the arcs-only unchanged case, and idempotence.
- [ ] 2.5 Implement the unshielded-collider validity check against a baseline parent set; test new-collider flagging and pre-existing-collider non-flagging.
- [ ] 2.6 Implement MEC size + representative DAG with the trivial arcs-only case (size 1 = input); document the extension point for a future uniform sampler; test the arcs-only case.
- [ ] 2.7 Confirm the causal-graph layer carries no floating-point scalar and exposes no `RealField` parameter (structural only); a downstream consumer at any precision reuses it unchanged.

## 3. CDL pipeline generalization (`discovery-pipeline`, in `deep_causality_discovery`)

- [ ] 3.0 Confirm `real-field-discovery` has landed (the discovery crate is generic over `T: RealField`, `SurdResult<T>`); this section builds on it.
- [ ] 3.1 Introduce a `DiscoveryOutcome<T>` enum with a `Surd(SurdResult<T>)` variant; change the `CausalDiscovery` trait return type from `SurdResult<T>` to `DiscoveryOutcome<T>` (no `dyn`).
- [ ] 3.2 Generalize the `WithCausalResults` state to carry `DiscoveryOutcome<T>`; update the `causal_discovery` typestate method accordingly.
- [ ] 3.3 Extend the discovery-stage input to carry a primary dataset and an optional second aligned dataset; SURD reads only the primary.
- [ ] 3.4 Add an optional user-supplied domain-graph input to the discovery stage.
- [ ] 3.5 Update the analyzer and the formatter to match `DiscoveryOutcome<T>` exhaustively; the `Surd` arm reproduces the current report verbatim.
- [ ] 3.6 Add a SURD regression test: rankings, decomposition, and rendered report are identical before and after the change on the same input.

## 4. Verification and hygiene

- [ ] 4.1 `cargo build -p` and `cargo test -p` for each touched crate (`deep_causality_num`, `deep_causality_tensor`, `deep_causality_sparse`, `deep_causality_topology`, `ultragraph`, `deep_causality_algorithms`, `deep_causality_discovery`); aim for full coverage of new code.
- [ ] 4.2 Register every new test file in its module tree and in the crate's `tests/BUILD.bazel`, per repo test conventions.
- [ ] 4.3 Confirm no external numeric crate was added, `unsafe_code = "forbid"` is intact in every touched crate, and no `dyn`/trait-object was introduced.
- [ ] 4.4 Run `make format && make fix`, then `make build` and `make test` (more than three crates changed).
- [ ] 4.5 Run `openspec validate brcd-prep-foundations` and confirm the change is apply-complete; prepare a commit message and request the owner commit.
