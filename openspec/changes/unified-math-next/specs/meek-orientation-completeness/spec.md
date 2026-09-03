<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: The Meek closure is a MixedGraph operation in the topology crate

The Meek orientation closure SHALL live in `deep_causality_topology` as inherent methods on `MixedGraph<T>`, beside the acyclicity operations, and `deep_causality_algorithms` SHALL consume it rather than carry its own copy.

Meek's rules are graph mathematics over a partially directed graph. `MixedGraph` is the workspace's
carrier for exactly that, and it already owns the neighbouring closure — `topological_sort`,
`has_cycle` and `find_cycle` are inherent methods in `mixed_graph/acyclicity/`. Placing the Meek
closure anywhere else splits one subject across two crates.

The dependency edge already exists: `deep_causality_algorithms` is tier 7 and
`deep_causality_topology` is tier 6, and the current implementation already imports `MixedGraph` and
`EdgeKind` from it. Nothing new is introduced by the move.

#### Scenario: The closure is reachable as a graph operation
- **WHEN** a caller holds a `MixedGraph<T>`
- **THEN** the Meek closure is available as an inherent method without importing `deep_causality_algorithms`

#### Scenario: The algorithms crate keeps no second copy
- **WHEN** `deep_causality_algorithms` is searched for an orientation rule implementation
- **THEN** none is found, and its three former call sites resolve to the topology method

#### Scenario: The move preserves the existing rule behaviour
- **WHEN** the R1–R3 closure runs on any input the previous implementation accepted
- **THEN** it produces the same orientation

### Requirement: The default closure applies rules R1 through R4

The default Meek closure SHALL apply rules R1, R2, R3 and R4 to a fixpoint, so that it is complete for any input whose oriented arcs admit a consistent DAG extension, including arcs that did not arise from a v-structure.

With `a — b` the undirected edge under consideration and `∼` adjacency, the four rules orient
`a → b` when:

- **R1** — there is `c` with `c → a` and `c ≁ b`.
- **R2** — there is `c` with `a → c → b`.
- **R3** — there are `c, d` with `d — a — c` undirected, `d → b`, `c → b`, and `c ≁ d`.
- **R4** — there are `c, d` with `d — a — c` undirected, `d → c → b`, and `b ≁ d`.

R1–R3 are complete only for the pattern of a DAG. The four together are sound and complete with
respect to any set of arcs admitting a consistent DAG extension, which is the hypothesis that covers
background knowledge, and a graph closed under all four is a maximally oriented PDAG.

This matters here because the caller supplies background knowledge. BRCD's Algorithm 1 adds `F → R`
as a directed arc and fixes a cut configuration before invoking the closure; neither orientation
arises from a v-structure. The paper's Corollary 4.2 — the completeness of that enumeration — rests
on the closure being complete under exactly this hypothesis.

#### Scenario: R4 fires where it applies
- **WHEN** the closure runs on a graph containing `d — a — c` undirected, `d → c → b`, `b ≁ d`, and `a — b` undirected
- **THEN** `a — b` is oriented `a → b`

#### Scenario: The closure reaches a fixpoint
- **WHEN** the closure runs on any input
- **THEN** it terminates, and a second invocation orients nothing further

#### Scenario: A pattern is unaffected by the added rule
- **WHEN** the closure runs on the pattern of a DAG, where R1–R3 are already complete
- **THEN** it produces the same orientation the R1–R3 closure produces

#### Scenario: An F-augmented graph is maximally oriented
- **WHEN** the closure runs on a CPDAG augmented with `F → R` arcs and a fixed cut configuration
- **THEN** the result is closed under all four rules

### Requirement: The R1-to-R3 closure remains available as a named entry point

A separate, explicitly named closure applying only R1, R2 and R3 SHALL remain available, and its documentation SHALL state that it is complete only for the pattern of a DAG and is retained for parity with the Python reference.

The reference BRCD implementation calls `graphical_models.PDAG.to_complete_pdag`, whose completion
applies R1–R3. Reproducing a reference's output is a legitimate need — for a differential test
against it, or for a result that must match a published number. What is not legitimate is that
behaviour being the default and its limitation being recorded only in a doc comment whose stated
precondition the caller violates.

Making the restricted closure a named opt-in inverts that: the mathematically complete closure is
what a caller gets by default, and the parity closure is something a caller asks for, with the reason
at the call site.

#### Scenario: The restricted closure is distinct and documented
- **WHEN** the R1–R3 entry point is called
- **THEN** it applies R1, R2 and R3 only
- **AND** its documentation states the pattern-only completeness hypothesis and names the reference it matches

#### Scenario: The two differ exactly where R4 fires
- **WHEN** both closures run on the same input
- **THEN** their outputs differ only on edges R4 orients, and the complete closure orients a superset

#### Scenario: Parity with the reference is preserved on demand
- **WHEN** a caller needs the reference's output
- **THEN** the R1–R3 entry point supplies it without reverting the default

### Requirement: The difference R4 makes is established by search before the rules are moved

A search over small graphs SHALL establish whether R4 changes the orientation of any graph this workspace constructs, and its result SHALL be recorded, before the closure is moved or the rule is added.

Background knowledge being present is necessary for R4 to matter, not sufficient. The rule may or may
not fire on the particular shapes BRCD builds — a CPDAG augmented with `F → R` arcs under a cut
configuration — and which of those it is determines whether this stage fixes a live defect or closes
a latent one. Both are worth doing; they are not worth conflating, and the answer belongs in the
record either way.

The search enumerates graphs to a stated vertex bound, augments each as Algorithm 1 does, closes
under R1–R3 and under R1–R4, and compares.

#### Scenario: The search is exhaustive to its stated bound
- **WHEN** the search runs
- **THEN** it enumerates every graph up to the stated vertex bound, applies the F-augmentation and cut configurations of Algorithm 1, and compares both closures on each

#### Scenario: A difference is recorded as a defect with its witness
- **WHEN** the search finds a graph the two closures orient differently
- **THEN** that graph is committed as a regression test
- **AND** the finding is recorded as a live defect with the propagation path through the MEC size to the posterior

#### Scenario: No difference is recorded as a result
- **WHEN** the search finds no difference up to its bound
- **THEN** the bound and the negative result are recorded in the stage's notes and in the closure's documentation
- **AND** the rule is still added, because the bound is not a proof for all inputs

### Requirement: Chordality is checked where it is assumed rather than assumed silently

An operation that requires a chordal component SHALL verify chordality and return a typed error when it does not hold, rather than proceeding on an unchecked assumption.

The clique-picking MEC-size computation is defined only on chordal components. Today chordality is
assumed and never checked, in both the `dag_sampling` path and the BRCD MEC path. That assumption is
coupled to the closure: an incomplete closure is one way a component that should be chordal is not,
so fixing R4 without checking chordality leaves the other half of the same failure surface open.

A non-chordal component reaching clique-picking produces a number rather than a complaint, and that
number reaches the posterior.

#### Scenario: A chordal component is accepted
- **WHEN** a chordal component is offered to an operation that requires chordality
- **THEN** the check passes and the operation proceeds

#### Scenario: A non-chordal component is refused
- **WHEN** a component containing a chordless cycle of length four or more is offered
- **THEN** a typed error naming the violation is returned, and no count is produced

#### Scenario: The check is reachable from both MEC paths
- **WHEN** either the `dag_sampling` clique-picking path or the BRCD MEC path runs
- **THEN** the chordality check has run on the components it consumes

### Requirement: The closure does not validate its input, and says so

The closure SHALL document that it performs no extendability check, and SHALL NOT be specified to signal on an input admitting no consistent DAG extension.

The rules are sound only relative to an input some DAG could have produced. On a PDAG that admits no
consistent extension the closure still returns an orientation, and that orientation means nothing.

Making it signal was considered and rejected for this change. Detecting non-extendability is a
separate algorithm, not a by-product of orienting, and adding it would change what BRCD's existing
call sites do. The honest specification is therefore that the closure orients and nothing more, with
the boundary written down rather than implied.

Two things follow, and both are properties of the documentation rather than of a return value. The
sweep tries one direction before the other, so an edge both directions compel is oriented the way
the sweep reached it — deterministic, but a consequence of iteration order and not a decision. And a
doubly-compelled edge is a symptom of non-extendability, not a characterisation of it: a PDAG can
admit no extension with no single edge compelled both ways.

#### Scenario: The boundary is documented
- **WHEN** the closure's documentation is read
- **THEN** it states that no extendability check is performed
- **AND** it states that the tie-break on a doubly-compelled edge follows the iteration order rather than a rule

#### Scenario: A non-extendable input still terminates
- **WHEN** the closure runs on a PDAG that admits no consistent DAG extension
- **THEN** it terminates and returns an orientation, without panicking or looping

#### Scenario: No direction is pinned on a contradictory edge
- **WHEN** a test covers a doubly-compelled edge
- **THEN** it asserts termination and the graph's invariants, and does not assert which direction was chosen
