# num-generic-monoid Specification

## Purpose
TBD - created by archiving change num-generic-monoid-tower. Update Purpose after archive.
## Requirements
### Requirement: A carrier-and-operation-generic Monoid decoupled from Zero/One

`deep_causality_num` SHALL provide a `Monoid` trait with an identity constructor and an associative binary combine — `fn empty() -> Self` and `fn combine(self, other: Self) -> Self` (receiver shape settled per design D1) — that does NOT require `Add`, `Mul`, `Zero`, or `One`. The existing `AddMonoid`/`MulMonoid` numeric monoids SHALL remain unchanged as the numeric specializations. Every implementor SHALL satisfy left identity, right identity, and associativity.

#### Scenario: The aggregation carrier that has no arithmetic can be a monoid

- **WHEN** `bool` (which implements neither `Add` nor `Zero`) is given a combine operation
- **THEN** it implements `Monoid` without any arithmetic bound, and its identity/associativity laws hold

#### Scenario: Numeric monoids are untouched

- **WHEN** the change lands
- **THEN** `AddMonoid`/`MulMonoid` and the rest of the numeric tower compile and pass their existing tests unchanged

### Requirement: Monoid laws are tested and proved in Lean

Each `Monoid` law SHALL be exercised by a Rust law-test registered in `deep_causality_num/tests/BUILD.bazel`, and SHALL be proved in Lean under `DeepCausalityFormal/Num/`, bare-`lean` typecheck, bound by a `THEOREM_MAP.md` id (`num.monoid.left_id`, `num.monoid.right_id`, `num.monoid.assoc`) with a matching Rust witness.

#### Scenario: Every monoid law has both sides of the bridge

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** each `num.monoid.*` id has a `proved` Lean location and a passing Rust witness, and the Lean file typechecks standalone with bare `lean`

