## Why

The full formalization of the causaloid needs the **aggregation carrier** — booleans, counts, probabilities — expressed as generic algebraic structures with machine-checked laws. `deep_causality_num` cannot express them today: its entire tower is anchored to the arithmetic operators (`AddSemigroup`, `AddMonoid`, `MulMonoid`, `Ring`, `Field`, …) plus `Zero`/`One`, so `bool` under `∧` — which implements neither `Add` nor `Zero` — has no monoid, and the `Associative`/`Commutative`/`Distributive` markers are bare (no operation parameter), tied to `Add`/`Mul`. This is the A1/A2/A3/#5 gap set recorded in `openspec/notes/causal-algebra/algebraic-causaloid.md` and `algebraic-causaloid-assumptions.md`.

The consequence is concrete: the `Collection` causaloid's four `AggregateLogic` cases (`All`/`Any`/`None`/`Some(k)`) are exactly bounded semilattices and a count-monoid-plus-threshold, but that fact is currently implicit inside `monadic_collection_utils.rs` and cannot be *stated* — so assumption #1 (Collection order-independence) is only property-tested, not proved. The reconvergence merge (∇) that the graph reasoning will eventually need also folds its value channel through a commutative monoid on the same tower.

This change closes the `num`-side gaps: a carrier-and-operation-generic monoid tower, decoupled from `Zero`/`One`, with the commutativity/idempotence laws and a verdict carrier — each tested and formalized in Lean.

## What Changes

- **Generic `Monoid` (N1).** A carrier-and-operation-generic `Monoid { fn empty() -> Self; fn combine(self, Self) -> Self; }` with associativity + identity laws, independent of `Zero`/`One`/`Add`/`Mul`. `AddMonoid`/`MulMonoid` stay as the numeric specializations they already are.
- **`CommutativeMonoid`, `Idempotent`, `BoundedSemilattice` (N2/N3).** `CommutativeMonoid: Monoid` (commutativity law); an `Idempotent` marker; `BoundedSemilattice: CommutativeMonoid + Idempotent`. The four `AggregateLogic` cases map onto these: `All` = bounded ∧-semilattice, `Any` = bounded ∨-semilattice, `None` = `Any` post-composed with complement, `Some(k)` = a count `CommutativeMonoid` plus a `≥ k` threshold. The laws attach to the generic `combine` operation, not to the bare `Associative`/`Commutative` markers.
- **Verdict / bounded-lattice carrier with complement (N4).** A `Verdict` (bounded lattice / Boolean-algebra) trait supplying meet, join, bottom, top, and **complement** (`¬`, which `None` needs), so the aggregation output type is a stated bound rather than an ad-hoc bool/prob coercion.
- **Instances.** `bool` (∧/∨ semilattices, complement), a `Count` newtype (count monoid), and a probability carrier (`[0,1]` product / inclusion–exclusion) implement the new traits — the exact carriers `aggregate_effects` reduces today.
- **Tests + Lean.** Each law is exercised by a Rust law-test (house style, registered in the crate's Bazel `tests/BUILD.bazel`) and **proved in Lean** under `DeepCausalityFormal/Num/`, bound by a `THEOREM_MAP.md` id and a Rust witness, bare-`lean` typecheck.

## Capabilities

### New Capabilities
- `num-generic-monoid`: a carrier-and-operation-generic `Monoid` (empty + combine, associativity + identity), decoupled from the numeric `Zero`/`One` tower, with the law markers carried by the operation.
- `num-commutative-semilattice`: `CommutativeMonoid`, `Idempotent`, and `BoundedSemilattice` over the generic monoid, with commutativity and idempotence laws — the algebra of the `AggregateLogic` reducers.
- `num-verdict-algebra`: a bounded-lattice / Boolean-algebra `Verdict` carrier (meet/join/bottom/top/complement) for the aggregation output type, with the lattice + complement laws.

### Modified Capabilities
<!-- No existing spec covers the num algebra tower at requirement level; these are additive new traits alongside the existing numeric tower, which is unchanged. -->

## Impact

- **New `num` traits:** `src/algebra/{monoid_generic,commutative_monoid,idempotent,bounded_semilattice,verdict}.rs` (names per one-type-one-module); exported from `lib.rs`. The existing `AddMonoid`/`MulMonoid`/`Ring`/`Field` tower is untouched; the new tower is orthogonal and additive.
- **New instances:** `bool`, a `Count` newtype, and the probability carrier implement the new traits.
- **New Lean:** `DeepCausalityFormal/Num/{Monoid,CommutativeMonoid,BoundedSemilattice,Verdict}.lean` (or extend the existing `Num/Monoid.lean`), registered in `DeepCausalityFormal.lean`; new `THEOREM_MAP.md` rows.
- **New Rust witnesses/tests:** `deep_causality_num/tests/algebra/*` law-tests, registered in `tests/BUILD.bazel`.
- **No external dependencies;** `unsafe_code = "forbid"`, macro-free `/src`, no-std compatible — consistent with the crate's existing constraints. The generic tower must not weaken any existing numeric-tower bound.
- **Unblocks:** `deep_causality_haft` `Foldable::fold_map` (gap H1) and the `Collection` order-independence proof (assumption #1), which depend on this tower.
