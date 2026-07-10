## ADDED Requirements

### Requirement: Collection aggregation is closed over the Verdict algebra

The formalization SHALL prove `core.verdict.closure`: the aggregation modes `All`, `Any`, `None`,
and `Some(k)` are closed operations in the `Verdict` algebra (bounded lattice + complement +
threshold) — each maps a bag of `Verdict` values to a `Verdict` value — and therefore
`Coll : Causaloid → Causaloid`: a collection of causaloids with a `Verdict` output is itself a
causaloid with a `Verdict` output (assumption #5, "a collection always outputs another causaloid",
made rigorous). The proof SHALL build on the existing `Verdict` laws
(`num.verdict.{lattice_laws,complement}`, `Algebra/Verdict.lean`) and the `CommutativeMonoid`
carrier monoids (`Conjunction`, `Disjunction`, `Count`, `Prob`) rather than re-proving them.

#### Scenario: Every aggregation mode lands in the carrier

- **WHEN** `Core/VerdictClosure.lean` is checked
- **THEN** `core.verdict.closure` is closed: for each of `All`/`Any`/`None`/`Some(k)`, the
  aggregate of any finite bag of `Verdict` values is a `Verdict` value, with `None` expressed as
  `Any` post-composed with `complement`

#### Scenario: The collection is again a causaloid

- **WHEN** the closure theorem is instantiated at the fixpoint (`core.causaloid.fixpoint`)
- **THEN** `Coll` is an operation `Causaloid → Causaloid` in the model — the `Coll` summand's
  output inhabits the same carrier the `Atom` summand requires

### Requirement: Collection aggregation requires a Verdict output type

The main crate's `Collection` aggregation SHALL require `O: Verdict` in place of "any `O`" — the
stated carrier bound the closure theorem needs. This is a **BREAKING** bound tightening: collection
aggregation over an output type with no `Verdict` instance no longer compiles. The two shipped
carriers SHALL remain `bool` (Boolean algebra) and the probability carrier (MV algebra,
`complement = 1 − p`), named by `core.verdict.carriers` behind the one trait.

#### Scenario: Aggregation compiles for Verdict carriers

- **WHEN** a collection causaloid aggregates over `bool` or the probability carrier
- **THEN** it compiles and evaluates as before, with the aggregation going through the `Verdict`
  operations

#### Scenario: A non-Verdict output type is rejected

- **WHEN** collection aggregation is attempted with an output type that has no `Verdict` instance
- **THEN** it fails to compile with the missing-bound error naming `Verdict`

### Requirement: The Verdict closure is tested and proved in Lean

`core.verdict.closure` and `core.verdict.carriers` SHALL be proved in Lean under
`DeepCausalityFormal/Core/VerdictClosure.lean` (bare-`lean`, zero `sorry`), citing the existing
`num.verdict.*` theorems as the base, bound by `THEOREM_MAP.md` rows with Rust witnesses under
`deep_causality/tests/formalization_lean/` (Bazel-registered).

#### Scenario: Both bridge sides exist for the closure

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** each `core.verdict.*` id has a proved Lean location and a passing Rust witness, and
  `Core/VerdictClosure.lean` typechecks standalone with bare `lean`
