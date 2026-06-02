## Context

CDL's typestate pipeline (`deep_causality_discovery`) is hardwired to SURD: a single dataset flows through the stages and `SurdResult<T>` is the result type carried by `WithCausalResults` and consumed by the analyzer and formatter. `brcd-estimator` adds a second algorithm with a two-dataset, optional-graph input and a `BrcdResult<T>` output. This change generalizes the seam and wires BRCD in. It depends on `brcd-estimator` (for `BrcdResult<T>` and the BRCD entry point) and lands last, so it is designed against the verified algorithm.

The decisions below were extracted from the archived `brcd-prep-foundations` (decisions D4/D5 there) and preserved in `openspec/notes/cdl-integration.md`. Repo constraints: no external numeric crates, `unsafe_code = "forbid"`, static dispatch (no `dyn`), full test coverage of new code.

## Goals / Non-Goals

**Goals:**
- A `DiscoveryOutcome<T>` closed enum carrying SURD and BRCD results, matched exhaustively, no `dyn`.
- Two-dataset carriage and an optional domain-graph input through the discovery stage, with SURD unaffected.
- BRCD reachable end-to-end through the CDL discovery language, rendered by the analyzer and formatter.
- SURD's numerics and report byte-identical before and after.

**Non-Goals:**
- Any change to the BRCD algorithm or SURD numerics.
- The BOSS bootstrap CPDAG path and Forest-KDE (out of scope in `brcd-estimator`).

## Decisions

**D1. Closed enum, not a second generic parameter or `dyn`.**
Replace `SurdResult<T>` with `DiscoveryOutcome<T>` = `{ Surd(SurdResult<T>), Brcd(BrcdResult<T>) }`. The analyzer and formatter match on it; adding an algorithm is a compile-checked change (exhaustive match fails until handled). A second generic result parameter threaded through the typestate was rejected for the type churn across every state; a boxed trait object was rejected as it violates the static-dispatch rule. The enum mirrors the existing `CausalDiscoveryConfig` enum, so the pattern is already established. Precision `T` comes from the archived `real-field-discovery`; this change adds only the algorithm dimension.

**D2. Two datasets, the second optional, not a regime-labelled single tensor.**
The discovery stage carries a primary dataset and an optional second aligned dataset. SURD reads the first and is unaffected; BRCD reads both. A single tensor with a regime-indicator column was rejected: it bakes BRCD's F-node convention into the data model and complicates SURD's path. Keeping datasets separate lets BRCD build its own F-node indicator (which it already does internally).

**D3. Optional domain graph on the stage input.**
The discovery stage accepts an optional graph over the variables (a `MixedGraph` / CPDAG). Algorithms that do not consume a graph ignore it; BRCD reads it as its required CPDAG. Absence is permitted for SURD; BRCD surfaces its own missing-CPDAG error (from `brcd-estimator`).

**D4. SURD path is behaviour-preserving; the migration is mechanical.**
Generalizing the result type and dataset carriage does not alter SURD output. A regression test asserts the rankings, the SURD decomposition, and the rendered report are identical before and after on the same input.

**D5. Wire `BrcdResult<T>` as the second outcome variant against the verified estimator.**
A `CausalDiscoveryConfig::Brcd { … }` variant drives the `brcd-estimator` entry point with the two datasets + the optional CPDAG, and surfaces `BrcdResult<T>` as `DiscoveryOutcome::Brcd`. Because `brcd-estimator` is already verified, this change's tests assert the *wiring* (the right inputs reach BRCD; its result renders), not the algorithm's correctness.

## Risks / Trade-offs

- **BREAKING source-API change in `deep_causality_discovery`.** → The owner controls the repo; the SURD result path is behaviour-preserving; the migration is mechanical and covered by tasks; no external consumers depend on the old signatures.
- **Exhaustive-match churn across analyzer/formatter.** → That is the intended compile-time guarantee (D1); the two arms are small.
- **Coupling to `brcd-estimator`'s public types.** → Intentional; this change lands after it, against the real `BrcdResult<T>`.

## Open Questions

- Exact carrier for the second dataset and the optional graph (a small struct on the stage input vs added enum fields) — settled in tasks; default a struct on the discovery-stage input to keep SURD's single-dataset call ergonomic.
- Whether the BRCD `CausalDiscoveryConfig` variant carries the seed/transform/prior inline or a nested `BrcdConfig` — default the nested `BrcdConfig` from `brcd-estimator` to avoid duplicating its surface.
