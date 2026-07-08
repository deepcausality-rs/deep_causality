# num-verdict-algebra Specification

## Purpose
TBD - created by archiving change num-generic-monoid-tower. Update Purpose after archive.
## Requirements
### Requirement: A verdict carrier with meet, join, and complement

`deep_causality_num` SHALL provide a `Verdict` trait — a bounded lattice with complement — supplying `bottom`, `top`, `meet`, `join`, and `complement`, so that the `Collection` aggregation output type is a stated bound rather than an ad-hoc bool/probability coercion. The complement SHALL support the `None` aggregation (`None` = `Any` post-composed with complement). The exact class (Boolean algebra vs probability MV-algebra) SHALL be pinned per design D4, with the probability carrier's `complement = 1 − p` recorded as an MV-algebra (not Boolean) complement.

#### Scenario: None is expressible via complement

- **WHEN** the `None` aggregation is evaluated as "no child fires"
- **THEN** it is the join-fold (`Any`) of the children post-composed with `complement`, using the `Verdict` trait's complement

#### Scenario: bool is a Boolean-algebra verdict

- **WHEN** `bool` implements `Verdict`
- **THEN** `meet = ∧`, `join = ∨`, `complement = !`, `bottom = false`, `top = true`, and the Boolean-algebra laws hold

#### Scenario: The probability carrier is an MV-algebra verdict

- **WHEN** the probability carrier implements `Verdict`
- **THEN** `complement = 1 − p` (an MV-algebra complement), and the caveat that it is not a Boolean algebra is documented (assumption #5 Q2)

### Requirement: Verdict laws are tested and proved in Lean

The bounded-lattice laws (meet/join associativity, commutativity, absorption; bottom/top identities) and the complement laws (involution / De Morgan for the pinned class) SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Num/Verdict.lean` (bare-`lean`), bound by `THEOREM_MAP.md` ids (`num.verdict.lattice_laws`, `num.verdict.complement`) with Rust witnesses.

#### Scenario: Both bridge sides exist for the verdict laws

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** each `num.verdict.*` id has a `proved` Lean location and a passing Rust witness, and `Num/Verdict.lean` typechecks standalone with bare `lean`

