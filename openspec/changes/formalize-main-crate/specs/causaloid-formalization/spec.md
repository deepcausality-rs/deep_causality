<!--
Synced 2026-07-10. The original requirements "Singleton causaloid formalized as a Kleisli arrow"
and "Well-formedness caveat closed by the carrier" are DELIVERED by prior changes and now live in
the main specs: the singleton/Kleisli content via `causaloid-fixpoint` and `causaloid-catamorphism`
(`core.causal_arrow.{category_laws, left_zero}`, the atom case of
`core.causaloid.{catamorphism_unique, arrow_fragment}`), the F-1 closure via `core-formalization`
(#7, `core.causal_monad.right_id` unconditional). The collection closure landed as
`causaloid-verdict-closure` (`core.verdict.closure`). What REMAINS in this change is the residue
below: the F-3 command-input theorem and the collection permutation-invariance theorem.
-->

## ADDED Requirements

### Requirement: Command input on the value channel is a stated theorem (F-3)

The formalization SHALL state and prove that `evaluate` applied to a command (`RelayTo`) on the
input channel yields a specific, named error — never a silent `None` and never a dropped signal —
matching the implemented Rust behaviour in `Causaloid::evaluate` and `evaluate_stateful` (the
singleton's command-specific error path). The theorem SHALL carry a `THEOREM_MAP.md` row and a
Rust witness under `deep_causality/tests/formalization_lean/`, and the Lean model SHALL typecheck
standalone with bare `lean`.

#### Scenario: Command input is total, not a dropped signal

- **WHEN** the model applies `evaluate` to a command on the input channel
- **THEN** it yields the command-specific error (the F-3 resolution), and the bound Rust witness
  confirms the real singleton path produces that error rather than manufacturing `None`

### Requirement: Collection aggregation is permutation-invariant over the Verdict carrier

Extending `Core/VerdictClosure.lean` (the landed closure theorem `core.verdict.closure`), the
formalization SHALL prove the #1 scoped order-invariance theorem on the collection path: for each
`AggregateLogic` mode, the aggregate **value** is invariant under permutation of the member bag —
`All`/`Any` as commutative-monoid folds (the `fuse_perm` device of `Core/GraphAlgebra.lean`
applies), `None` via the `Any` result, `Some(k)` via permutation-invariance of the firing count.
The theorem SHALL carry the #1 scope explicitly (value channel; stateless path; all-success; logs
up to permutation) and SHALL NOT claim more. Each theorem MUST carry a `THEOREM_MAP.md` row and a
Rust witness on the real `evaluate_collection` path.

#### Scenario: Aggregate is permutation-invariant

- **WHEN** the same bag of member verdicts is folded in two different orders under a given
  `AggregateLogic`
- **THEN** the model proves the two results equal, and the Rust witness confirms it on the real
  collection evaluation

#### Scenario: The scope is the #1 ruling, stated honestly

- **WHEN** the theorem is read
- **THEN** it claims invariance for the value channel on the stateless all-success path only,
  citing the #1 scoped ruling; the log channel and the stateful path are excluded by statement
