## Context

`deep_causality_num` has a faithful numeric algebra tower (`Magma → Semigroup → Monoid → Group → Ring → Field`, `Module`, `Algebra`, `DivisionAlgebra`), but every "monoid" in it is `AddMonoid`/`MulMonoid` — the operation is `Add` or `Mul` and the identity is `Zero`/`One`. The markers `Associative`/`Commutative`/`Distributive` are empty and un-parameterised (adequate for a numeric type with one canonical `+` and `×`). This makes the *aggregation* carrier — `bool` under `∧`/`∨`, counts under `+`, probabilities under product/inclusion–exclusion — unreachable as an algebra: `bool` has no `Add`/`Zero`, and "operation `combine` is idempotent and commutative" cannot be stated independently of `Add`/`Mul`.

The formalization target (`algebraic-causaloid.md` Part 2, gaps A1–A3, #5) is a generic commutative-monoid/semilattice tower plus a verdict carrier, so that `Collection = fold_map into the aggregation monoid` and the order-independence theorem is a consequence of stated laws.

## Goals / Non-Goals

**Goals:**
- Add a generic `Monoid` (empty/combine) tower decoupled from `Zero`/`One`, with `CommutativeMonoid`, `Idempotent`, `BoundedSemilattice`, and a `Verdict` bounded-lattice/Boolean-algebra carrier.
- Implement the tower for the exact aggregation carriers (`bool`, `Count`, probability), the ones `aggregate_effects` uses.
- Test every law (Rust law-tests, Bazel-registered) and **prove every law in Lean** (bare-`lean`, THEOREM_MAP-bound, Rust witness) — full formalization, not just tests.

**Non-Goals:**
- No change to the existing numeric tower (`AddMonoid`/`MulMonoid`/`Ring`/`Field`); it stays as the numeric specialization.
- No free monoid / free algebra construction (A4 — explicitly not required of `num`; the free structure is `haft`'s job).
- No `Foldable::fold_map` here (that is the `haft` change H1, which depends on this tower).

## Decisions

**D1 — Generic `Monoid` decoupled from `Zero`/`One`.** `pub trait Monoid: Sized { fn empty() -> Self; fn combine(self, other: Self) -> Self; }` (or `&self` receiver if the coverage/ergonomics of the carriers prefer it — settle in task 1 against the aggregation call sites). Laws: `combine(empty(), x) = x`, `combine(x, empty()) = x`, `combine(combine(x, y), z) = combine(x, combine(y, z))`. It carries its own associativity + identity; it does NOT require `Add`/`Mul`/`Zero`/`One`. Rationale: the aggregation carrier (`bool`) satisfies none of those.

**D2 — The law markers ride the operation, not the global markers (A3).** Commutativity and idempotence attach to `combine` via `CommutativeMonoid` and `Idempotent`, not by retrofitting an operation parameter onto the empty `Associative`/`Commutative` markers (which stay as the numeric-tower markers they are). Smallest surface: `Idempotent` is a marker with a documented law (`combine(x, x) = x`) consumed by `BoundedSemilattice`.

**D3 — Semilattice tower.** `CommutativeMonoid: Monoid` (law `combine(x, y) = combine(y, x)`); `Idempotent` (law `combine(x, x) = x`); `BoundedSemilattice: CommutativeMonoid + Idempotent`. `Some(k)` decomposes as a `CommutativeMonoid` on counts (no idempotence) plus a `≥ k` threshold predicate — so `Count` is a `CommutativeMonoid` but NOT a `BoundedSemilattice`, which the design must keep distinct.

**D4 — `Verdict` carrier (N4 / #5).** `pub trait Verdict: BoundedSemilattice { fn bottom() -> Self; fn top() -> Self; fn meet(self, Self) -> Self; fn join(self, Self) -> Self; fn complement(self) -> Self; }` — a bounded lattice with complement (Boolean algebra for `bool`; the probability MV-algebra is the caveat carrier). `None` = `Any` (join-fold) post-composed with `complement`. Whether the probability carrier is a Boolean algebra or an MV-algebra behind one enum is settled in task 1 (assumption #5 Q2). The exact class is pinned before the `Collection` bound is stated.

**D5 — Instances match `aggregate_effects` exactly.** `bool` (∧ = `BoundedSemilattice` id `true`; ∨ = `BoundedSemilattice` id `false`; via a meet/join wrapper or two marker impls — decide in task 1 to avoid a coherence clash on `bool`), a `Count(u*)` newtype (`CommutativeMonoid`, id `0`), and a probability carrier (`[0,1]` with product/`1−∏(1−pᵢ)`). These are the carriers `utils/monadic_collection_utils.rs` reduces, so the trait-ification is behaviour-preserving for the reducer.

**D6 — Lean formalization, one law one theorem.** Each law is proved in `DeepCausalityFormal/Num/*.lean` (bare-`lean`, self-contained, house style), bound to a `THEOREM_MAP.md` id and a Rust witness in `deep_causality_num/tests/algebra/`. The abstract laws are proved over a modelled generic monoid/semilattice (as `Num/Monoid.lean` already does for the additive monoid); the instance laws (`bool` ∧/∨ idempotent-commutative, `Count` commutative, probability) are proved at the concrete carrier and witnessed by the Rust law-tests.

## Risks / Trade-offs

- **[Coherence clash for two monoids on one type]** `bool` has two monoids (∧, ∨). Rust cannot have two `Monoid for bool`. → Use meet/join *methods* on a single `Verdict for bool` (the lattice carries both operations), or newtype wrappers (`All(bool)`/`Any(bool)`) for the `Monoid` instances. Decide in task 1; the `Verdict` route (both ops as methods) is cleaner and matches D4.
- **[Receiver shape `self` vs `&self`]** affects ergonomics and the `fold_map` default in `haft`. → Settle against the `aggregate_effects` call sites and the `haft` `fold_map` signature in task 1, before the trait is stated.
- **[Weakening a numeric-tower bound]** the new tower must be orthogonal. → It introduces no blanket impl that overlaps the numeric tower; `bool`/`Count`/probability are the only instances, none of which are `Ring`/`Field`.
- **[Probability carrier class]** Boolean algebra vs MV-algebra (assumption #5). → Pin the class in task 1; `complement` for probability is `1 − p`, which is an MV-algebra complement, not Boolean — record the caveat.

## Migration Plan

Additive: new traits + instances + Lean files + tests. Steps: (1) settle the receiver shape, the `bool` two-monoid resolution, and the probability carrier class against `monadic_collection_utils.rs` and the `haft` `fold_map` signature; (2) `Monoid` + laws + Lean + witness; (3) `CommutativeMonoid`/`Idempotent`/`BoundedSemilattice` + laws + Lean + witnesses; (4) `Verdict` + lattice/complement laws + Lean + witnesses; (5) instances (`bool`, `Count`, probability); (6) `bazel test //deep_causality_num/...` green, bare-`lean` on the new `Num/*.lean` exit 0. Rollback = remove the new files; the numeric tower is unaffected.

## Open Questions (settled during implementation)

- `Monoid::combine` receiver — **DECIDED: by-value `fn combine(self, Self) -> Self`** (the standard functional monoid; folds via `acc.combine(f(x))`, matches the intended `fold_map`).
- `bool`'s two monoids — **DECIDED: both.** `Verdict for bool` carries `meet`/`join`/`complement` as methods; the `Monoid`/`BoundedSemilattice` *instances* are the `Conjunction(bool)` (∧) and `Disjunction(bool)` (∨) newtypes, avoiding the coherence clash of two `Monoid for bool`.
- Probability carrier — **DECIDED: MV-algebra.** `Prob(f64)` is a product `CommutativeMonoid` (the `All` reducer) and a `Verdict` with `meet=min`, `join=max`, `complement = 1 − p` (an MV-algebra complement, not Boolean — recorded per assumption #5 Q2; the involution holds up to floating-point rounding, so `Prob` witnesses use a tolerance).
- Lean environment note — the `num` layer uses Mathlib (`lake env lean`), like the existing `Num/Monoid.lean`. The generic-monoid + commutativity laws are stated over Mathlib's `Monoid`/`CommMonoid`; the bounded-semilattice and verdict laws are proved concretely on `Bool` (`by cases … <;> rfl`) because the `Mathlib.Order.*` olean cache is unavailable in this environment (the `lake exe cache` binary is broken on this macOS), and the concrete `Bool` proofs mirror the Rust `Conjunction`/`Disjunction`/`bool`-`Verdict` instances exactly.
