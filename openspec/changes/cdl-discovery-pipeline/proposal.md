## Why

CDL (`deep_causality_discovery`) hosts one discovery algorithm, SURD, and hardcodes its result type (`SurdResult<T>`) and single-dataset flow throughout the typestate pipeline. The BRCD estimator (change `brcd-estimator`) is a second algorithm with a different shape: it consumes **two** aligned datasets (normal + anomalous), can take a user-supplied domain graph (CPDAG), and returns a `BrcdResult<T>` rather than a `SurdResult<T>`. To expose BRCD through the CDL discovery language, the pipeline seam must be generalized.

This change generalizes that seam and wires BRCD in. It is sequenced **after** `brcd-estimator` is built and verified, so the carrier is designed against the real `BrcdResult<T>` and the integration is the final step over a known-correct algorithm — not a reconstructed one. The rationale of record is `openspec/notes/cdl-integration.md` (extracted from the archived `brcd-prep-foundations`, where this was the deferred `discovery-pipeline` capability).

## What Changes

- **Algorithm-specific result without dynamic dispatch.** Replace the precision-generic `SurdResult<T>` with a closed enum `DiscoveryOutcome<T>` (`Surd(SurdResult<T>)`, `Brcd(BrcdResult<T>)`), parameterized over `T: RealField`, no `dyn`. The `CausalDiscovery` trait return type, the `CausalDiscoveryConfig` enum, the `WithCausalResults` state, the analyzer, and the formatter handle each variant by exhaustive match.
- **Two-dataset carriage.** The discovery stage carries a primary dataset and an optional second aligned dataset. SURD reads only the primary and is unaffected; BRCD reads both (normal + anomalous).
- **User-supplied domain graph input.** The discovery stage accepts an optional graph over the variables. SURD ignores it; BRCD reads it as its CPDAG.
- **BRCD wired end-to-end.** A `CausalDiscoveryConfig` variant for BRCD drives the `brcd-estimator` entry point and surfaces its `BrcdResult<T>` as `DiscoveryOutcome::Brcd`, rendered by the analyzer and formatter.
- **SURD preserved.** SURD's numerical results and rendered report are identical before and after, guarded by a regression test.

## Capabilities

### New Capabilities
- `discovery-pipeline`: a CDL discovery stage that carries two aligned datasets, accepts an optional user-supplied domain graph, and returns an algorithm-specific result enum `DiscoveryOutcome<T>` (`Surd`, `Brcd`) via exhaustive matching with no `dyn`, with SURD behaviour preserved and BRCD exposed end-to-end.

### Modified Capabilities
<!-- The existing generic-precision-discovery capability governs SURD's precision genericity; this change adds the algorithm dimension on top without changing that requirement's text. SURD behaviour preservation is asserted as a new scenario here. -->

## Impact

- **Crates touched.** `deep_causality_discovery` (trait, config enum, state, analyzer, formatter, stage input). Depends on `brcd-estimator` for `BrcdResult<T>` and the BRCD entry point in `deep_causality_algorithms`.
- **Public API.** **BREAKING** at the source-API level of `deep_causality_discovery`: the `CausalDiscovery` trait return type and the `WithCausalResults` state signature change from `SurdResult<T>` to `DiscoveryOutcome<T>`. Behaviour-preserving for SURD. No external consumers depend on the old signatures.
- **Dependencies.** None added beyond the existing `deep_causality_discovery → deep_causality_algorithms` edge. No external numeric crates; `unsafe_code = "forbid"`; static dispatch (no `dyn`).
- **Prerequisite.** `brcd-estimator` must land and be verified first.
- **Out of scope.** Any change to BRCD's algorithm or to SURD's numerics. The BOSS bootstrap path and Forest-KDE remain out of scope (deferred in `brcd-estimator`).
