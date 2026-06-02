# CDL discovery-pipeline generalization

Design note for the change that generalizes the CDL discovery stage so a second
algorithm (BRCD) can flow through it. Extracted from `brcd-prep-foundations`,
where it was the `discovery-pipeline` capability (section 3). It is held here as
a note until BRCD is implemented and verified, then converted into its own
change and landed last — on top of a working estimator, not ahead of it.

## Why this is separate from the prep foundations

`brcd-prep-foundations` delivers two kinds of reusable base: numeric primitives
(`linalg-numeric-primitives`) and the causal-graph operations
(`causal-graph`). Both are pure foundations — nothing in CDL's public surface
moves. The pipeline generalization is different in kind: it is a **breaking
change to `deep_causality_discovery`'s public API** (the `CausalDiscovery`
trait, the `WithCausalResults` state, the analyzer, and the formatter), and its
shape is dictated by what BRCD actually needs at the seam — two aligned
datasets, an optional domain graph, and a polymorphic result.

Generalizing the seam *before* BRCD exists means designing the carrier against a
reconstructed interface. Doing it *after* BRCD is built and verified means
designing it against the real one. The cost of waiting is low (the change is
mechanical and behaviour-preserving for SURD); the cost of guessing wrong is a
reworked public API. So the order is: prep foundations → BRCD estimator +
reference verification → this pipeline generalization.

## What it changes

CDL (`deep_causality_discovery`) hosts one discovery algorithm, SURD, and
hardcodes its result type and single-dataset flow throughout the typestate
pipeline. This change generalizes three things and preserves SURD exactly.

### Requirement: two-dataset carriage through the discovery stage

The discovery stage carries a primary dataset and an optional second aligned
dataset. A single-dataset algorithm (SURD) reads only the primary and is
unaffected by the presence or absence of the second. A two-dataset algorithm
(BRCD: normal + anomalous regimes) reads both.

### Requirement: algorithm-specific discovery result without dynamic dispatch

Discovery output becomes a closed enum `DiscoveryOutcome<T>` of
algorithm-specific result types, parameterized over the precision `T: RealField`
introduced by `real-field-discovery`, with no `dyn` trait objects. One variant
exists now — `Surd(SurdResult<T>)`; a `Brcd(..)` variant is added by the CDL
change once `BrcdResult<T>` is known. The analyzer and formatter handle each
variant by exhaustive match, so adding an algorithm is a compile-checked change.

### Requirement: user-supplied domain graph input

The discovery stage accepts an optional user-supplied graph over the variables.
Algorithms that do not consume a graph ignore it; algorithms that require one
(BRCD, which can take a domain/service graph) read it from the stage input.

### Requirement: SURD behaviour is preserved

Generalizing the result type and the dataset carriage does not alter SURD
output. The rankings, the SURD decomposition, and the rendered report are
identical before and after, on the same input — guarded by a regression test.

## Decisions carried over

- **D4. Closed enum, not generics or `dyn`.** Replace the precision-generic
  `SurdResult<T>` with a `DiscoveryOutcome<T>` enum. The analyzer and formatter
  match on it. A second generic result parameter threaded through the typestate
  was rejected for the type churn it spreads across every state; a boxed trait
  object was rejected because it violates the repo's static-dispatch rule. The
  enum mirrors the existing `CausalDiscoveryConfig` enum, so the pattern is
  already established in this crate. Precision `T` comes from
  `real-field-discovery`; this change adds only the algorithm dimension.
- **D5. Two datasets, the second optional, not a regime-labelled single
  tensor.** SURD reads the first and is unaffected; a two-dataset algorithm
  reads both. A single tensor with a regime-indicator column was rejected: it
  bakes the F-node convention into the data model before the algorithm that
  needs it exists, and complicates SURD's path. Keeping datasets separate lets
  BRCD construct its own regime indicator.

## Surface touched (BREAKING for `deep_causality_discovery`)

`CausalDiscovery` trait return type, `CausalDiscoveryConfig` enum,
`WithCausalResults` state, the analyzer, and the formatter. Behaviour-preserving
for SURD; the migration is mechanical. No external consumers depend on the old
signatures.

## Status

Held as a note. Becomes change `cdl-discovery-pipeline` after the BRCD estimator
lands and is verified, and is implemented with the real `BrcdResult<T>` wired in
as the second `DiscoveryOutcome` variant. See `openspec/notes/rca/BRCD.md` for
the algorithm and `brcd-prep-foundations` for the foundations it builds on.
