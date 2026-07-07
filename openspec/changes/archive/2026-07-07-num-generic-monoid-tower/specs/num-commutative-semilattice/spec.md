## ADDED Requirements

### Requirement: Commutative monoid, idempotent marker, and bounded semilattice

`deep_causality_num` SHALL provide `CommutativeMonoid: Monoid` (adding the commutativity law `combine(x, y) = combine(y, x)`), an `Idempotent` marker (law `combine(x, x) = x`), and `BoundedSemilattice: CommutativeMonoid + Idempotent`. The commutativity and idempotence laws SHALL attach to the generic `combine` operation, not to the bare `Associative`/`Commutative` markers (which remain the numeric-tower markers). A `CommutativeMonoid` that is NOT idempotent (a count monoid) SHALL be expressible and SHALL NOT be forced to implement `BoundedSemilattice`.

#### Scenario: The AggregateLogic reducers are named algebras

- **WHEN** `All`/`Any` are modelled
- **THEN** each is a `BoundedSemilattice` (∧ with identity `true`; ∨ with identity `false`), and `Some(k)` is a `CommutativeMonoid` on counts (identity `0`) that is deliberately not a `BoundedSemilattice`

#### Scenario: Idempotence is separable from commutativity

- **WHEN** a carrier is commutative but not idempotent (counts under `+`)
- **THEN** it implements `CommutativeMonoid` but not `Idempotent`/`BoundedSemilattice`

### Requirement: Semilattice laws are tested and proved in Lean

Commutativity and idempotence SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Num/` (bare-`lean`), each bound by a `THEOREM_MAP.md` id (`num.commutative_monoid.comm`, `num.semilattice.idempotent`, and the inherited `num.semilattice.{comm, assoc}`) with a Rust witness.

#### Scenario: The order-independence theorem becomes a consequence of stated laws

- **WHEN** a multiset of verdicts is folded under a `BoundedSemilattice`
- **THEN** the result is independent of order, and this follows from the proved commutativity + associativity laws rather than being asserted (this is the algebra assumption #1 needs to upgrade from property-tested to proved)
