Each stage ends green: builds, new tests pass, `cargo clippy` clean (rewrite, don't `#[allow]`), `cargo fmt --check` clean, no `dyn`, no `unsafe`, no external numeric crate. Draft a commit message at each stage gate. **Prerequisite: `brcd-estimator` is landed and verified** (provides `BrcdResult<T>`, `BrcdConfig`, and the BRCD entry point). All edits are in `deep_causality_discovery` unless noted.

## 1. `DiscoveryOutcome<T>` result enum

- [ ] 1.0 Confirm `brcd-estimator` has landed (`BrcdResult<T>`, `BrcdConfig`, the BRCD entry point are public in `deep_causality_algorithms`).
- [ ] 1.1 Introduce `DiscoveryOutcome<T>` = `{ Surd(SurdResult<T>), Brcd(BrcdResult<T>) }`; no `dyn`.
- [ ] 1.2 Change the `CausalDiscovery` trait return type from `SurdResult<T>` to `DiscoveryOutcome<T>`.
- [ ] 1.3 Generalize the `WithCausalResults` state to carry `DiscoveryOutcome<T>`; update the `causal_discovery` typestate method.
- [ ] 1.4 Update the analyzer and formatter to match `DiscoveryOutcome<T>` exhaustively; the `Surd` arm reproduces the current report verbatim.
- [ ] 1.5 Tests: compile-time exhaustiveness (a stub variant fails to compile until handled — documented, not committed); SURD arm unchanged.

## 2. Two-dataset carriage

- [ ] 2.1 Extend the discovery-stage input to carry a primary dataset and an optional second aligned dataset (a small struct on the stage input; SURD's single-dataset call stays ergonomic).
- [ ] 2.2 SURD reads only the primary; assert it is unaffected by the presence/absence of the second.
- [ ] 2.3 Tests: SURD with/without a second dataset is identical; a two-dataset path makes both available.

## 3. Domain-graph input

- [ ] 3.1 Add an optional user-supplied domain graph (`MixedGraph` / CPDAG) to the discovery stage input.
- [ ] 3.2 SURD ignores it; the input threads through to the algorithm that needs it.
- [ ] 3.3 Tests: supplied graph reaches the algorithm; absent graph leaves SURD unchanged.

## 4. Wire BRCD end-to-end

- [ ] 4.1 Add a `CausalDiscoveryConfig::Brcd { config: BrcdConfig }` variant driving the `brcd-estimator` entry point with the two datasets + the supplied CPDAG.
- [ ] 4.2 Surface `BrcdResult<T>` as `DiscoveryOutcome::Brcd`; render it in the analyzer and formatter.
- [ ] 4.3 Tests: BRCD runs through the discovery language on a fixture (inputs reach BRCD; result renders); missing-CPDAG surfaces BRCD's required-CPDAG error.

## 5. SURD regression + hygiene

- [ ] 5.1 SURD regression test: rankings, decomposition, and rendered report identical before and after on the same input.
- [ ] 5.2 `cargo build -p deep_causality_discovery` and `cargo test -p deep_causality_discovery`; full coverage of new code; register new test files in the module tree and `tests/BUILD.bazel`.
- [ ] 5.3 Confirm no external numeric crate added, `unsafe_code = "forbid"` intact, no `dyn` introduced.
- [ ] 5.4 `make format && make fix`, then `make build` and `make test` (BREAKING API — rebuild dependents).
- [ ] 5.5 `openspec validate cdl-discovery-pipeline`; prepare a commit message and request the owner commit.
