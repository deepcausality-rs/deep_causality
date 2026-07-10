# causaloid-graph-algebra Specification

## Purpose
TBD - created by archiving change causaloid-formalization-stages-2-5. Update Purpose after archive.
## Requirements
### Requirement: A characterization corpus gates the fold rewrite

Before any engine change, a characterization-test corpus (assumption #10) SHALL capture the current
graph-reasoning behavior — chains, trees, fan-out, and the reconvergent diamond — recording exact
outputs (value, error, state, context, log). After the rewrite, chains and trees SHALL reproduce
their captured outputs bit-identically; the reconvergent diamond — which currently fails loudly —
becomes the defined-merge case, and that behavior change SHALL be the only one, documented in the
corpus and the changelog as **BREAKING** on the diamond case.

#### Scenario: The corpus exists before the rewrite

- **WHEN** the fold-rewrite tasks begin
- **THEN** the characterization corpus is committed and green against the pre-change engine

#### Scenario: Chains and trees are bit-identical

- **WHEN** the corpus runs against the rewritten fold
- **THEN** every chain and tree case reproduces its captured output exactly, and only the diamond
  cases carry an updated (defined-merge) expectation with a documented rationale

### Requirement: The graph fold is invariant under every consistent schedule

The rewritten engine SHALL evaluate the frozen graph as a topological fold that applies
`∇ ∘ (Λ₁ ⊗ Λ₂)` at reconvergent joins — each incoming edge's Λ decoration applied first
(identity-keyed), then the commutative merge `∇` — and the formalization SHALL prove
`core.causaloid.graph_fold_order_invariant`: the fold result is invariant under every evaluation
schedule consistent with the derived causal order (assumption #2 Q1). The proof SHALL build on the
`CommutativeMonoid` bound of `∇` and the symmetric-monoidal PROP (`haft.sym_monoidal.*`).

#### Scenario: Two consistent schedules agree

- **WHEN** the same frozen graph is folded under two different topological schedules
- **THEN** the resulting value, error, and state channels are equal, and the logs are equal as
  multisets

### Requirement: Joins follow the per-channel policy

At a reconvergent join the channels SHALL merge per the standing rulings: the **value** channel by
`∇` (a `CommutativeMonoid`, optionally pre-composed with per-edge Λ's); the **log** channel as a
multiset at the join (assumption #1's "up to log permutation", applied — within a branch the log
stays ordered); the **state** channel under a **single-writer invariant** — at most one incoming
branch writes state, checked at freeze time, violation = freeze error, so no state merge is ever
defined or needed.

#### Scenario: Value merges commutatively

- **WHEN** branch results `a` and `b` arrive at a join in either order
- **THEN** the merged value is `∇(Λ₁(a), Λ₂(b))` and equals the merge of the swapped arrival

#### Scenario: A two-writer diamond is rejected at freeze

- **WHEN** a graph is frozen in which two branches of a diamond both write state
- **THEN** freezing fails with a single-writer violation naming the join, and evaluation is never
  reached

### Requirement: Freeze-time checks guard the fold's preconditions

The freeze step SHALL check the structural preconditions the fold's theorems assume: acyclicity
(the existing opt-in `freeze_dag`), the single-writer invariant at joins, and an extension point
for level-specific hooks (the enforcement point of the QCM-on-EPP freeze model). A graph that fails
a check SHALL not produce a frozen evaluable graph.

#### Scenario: Checks run at freeze, not at evaluation

- **WHEN** a graph violating a freeze check is frozen
- **THEN** the freeze returns the specific check error, and no partial evaluation occurs

### Requirement: The graph algebra is tested and proved in Lean

`core.causaloid.graph_fold_order_invariant` SHALL be proved in Lean under
`DeepCausalityFormal/Core/GraphAlgebra.lean` (bare-`lean`, zero `sorry`), citing the
`CommutativeMonoid` and PROP theorems as the base with deviation notes, bound by a `THEOREM_MAP.md`
row with a Rust witness under `deep_causality/tests/formalization_lean/` (Bazel-registered) that
exercises schedule permutation on a real reconvergent graph.

#### Scenario: Both bridge sides exist for the graph algebra

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** `core.causaloid.graph_fold_order_invariant` has a proved Lean location and a passing
  Rust witness, and `Core/GraphAlgebra.lean` typechecks standalone with bare `lean`
