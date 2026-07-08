## ADDED Requirements

### Requirement: Singleton causaloid formalized as a Kleisli arrow

The formalization SHALL model a singleton causaloid as a context-parameterized Kleisli arrow `I → CausalEffect<O>` in the Kleisli category of the causal monad, over the current `CausalEffect` carrier (not the removed `EffectValue`). `Core/Causaloid.lean` SHALL define the arrow and prove that `evaluate` is its extension restricted to the value fragment, reducing the arrow laws to the already-proven `core.causal_arrow.*` theorems. Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness under `tests/formalization_lean/`, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Singleton denotes a lawful Kleisli arrow

- **WHEN** `Core/Causaloid.lean` is typechecked with bare `lean`
- **THEN** the singleton arrow's identity, composition, and error-left-zero laws are proved by reduction to `Core/CausalArrow.lean`, and the bound Rust witness test passes under `bazel test //...`

#### Scenario: Command input on the value channel is total, not a dropped signal

- **WHEN** the model applies `evaluate` to a command (`RelayTo`) on the input channel
- **THEN** it yields a clear error (the F-3 resolution) rather than silently manufacturing `None`, matching the Rust `Causable::evaluate` / `evaluate_stateful` behaviour

### Requirement: Well-formedness caveat closed by the carrier

The formalization SHALL record that the historical F-1 caveat (`error ⇒ value = None` unenforced) is now closed by the `outcome: Result<CausalEffect<V>, E>` carrier, so right identity holds unconditionally, and SHALL NOT reintroduce a conditional well-formedness hypothesis for the singleton.

#### Scenario: Unconditional right identity is inherited

- **WHEN** the singleton laws are stated
- **THEN** they cite `core.causal_monad.right_id` (unconditional) with no `error ⇒ value=None` side-condition

### Requirement: Collection causaloid formalized as a commutative-monoid fold

The formalization SHALL model the Collection causaloid as a fold over a verdict carrier using a commutative-associative monoid, so that the aggregate value is invariant under permutation of the collection. `Core/Collection.lean` SHALL cover `AggregateLogic {All, Any, None, Some(k)}` and prove order-invariance (e.g. via a `Multiset`/`CommMonoid` fold). Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness.

#### Scenario: Aggregate is permutation-invariant

- **WHEN** the same multiset of member verdicts is folded in two different orders under a given `AggregateLogic`
- **THEN** the model proves the two results equal, and the Rust witness confirms it on the real collection evaluation

#### Scenario: Non-commutative aggregation is out of scope by construction

- **WHEN** an aggregation is not a commutative-associative monoid
- **THEN** the order-invariance theorem is not claimed for it (the requirement is scoped to the monoidal aggregates)
