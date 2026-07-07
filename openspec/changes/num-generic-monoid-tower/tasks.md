## 1. Settle the shape (decide before coding)

- [ ] 1.1 Decide `Monoid::combine` receiver (`self` vs `&self`) against the `aggregate_effects` call sites (`utils/monadic_collection_utils.rs`) and the intended `haft::Foldable::fold_map` signature. Record in design.md.
- [ ] 1.2 Decide the `bool`-two-monoids resolution (`Verdict`-methods carrying meet/join vs `All`/`Any` newtypes). Record.
- [ ] 1.3 Pin the probability carrier's class (Boolean algebra vs MV-algebra; complement `1 − p`). Record the caveat against assumption #5 Q2.

## 2. Generic `Monoid` (N1)

- [ ] 2.1 `src/algebra/monoid_generic.rs`: `pub trait Monoid { fn empty() -> Self; fn combine(...) -> Self; }` (no `Zero`/`One`/`Add`/`Mul` bound). Export from `lib.rs`. Keep `AddMonoid`/`MulMonoid` untouched.
- [ ] 2.2 Rust law-tests (`tests/algebra/monoid_generic_tests.rs`): left identity, right identity, associativity — at representative carriers.
- [ ] 2.3 Lean: `DeepCausalityFormal/Num/Monoid.lean` (extend or add) proving `num.monoid.{left_id, right_id, assoc}` over the modelled generic monoid; `THEOREM_MAP.md` rows; bare-`lean` typecheck; bind the Rust witnesses.

## 3. Commutative / idempotent / semilattice (N2/N3)

- [ ] 3.1 `src/algebra/{commutative_monoid,idempotent,bounded_semilattice}.rs`: `CommutativeMonoid: Monoid` (comm law), `Idempotent` marker (idempotence law), `BoundedSemilattice: CommutativeMonoid + Idempotent`. Export.
- [ ] 3.2 Rust law-tests: commutativity, idempotence, and the derived semilattice absorption where applicable.
- [ ] 3.3 Lean: `Num/CommutativeMonoid.lean` + `Num/BoundedSemilattice.lean` proving `num.commutative_monoid.comm`, `num.semilattice.{idempotent, comm, assoc}`; THEOREM_MAP rows; witnesses; bare-`lean`.

## 4. Verdict carrier (N4 / #5)

- [ ] 4.1 `src/algebra/verdict.rs`: `Verdict: BoundedSemilattice` with `bottom`/`top`/`meet`/`join`/`complement`. Export.
- [ ] 4.2 Rust law-tests: bounded-lattice laws (meet/join assoc/comm/absorption, bottom/top identities) + complement laws (De Morgan / involution for the pinned class).
- [ ] 4.3 Lean: `Num/Verdict.lean` proving `num.verdict.{lattice_laws, complement}` for the Boolean/MV class pinned in 1.3; THEOREM_MAP rows; witnesses; bare-`lean`.

## 5. Instances (the `AggregateLogic` carriers)

- [ ] 5.1 `bool`: `Verdict` (∧ = meet, ∨ = join, complement = `!`), covering `All`/`Any`/`None`.
- [ ] 5.2 `Count` newtype: `CommutativeMonoid` (id `0`), NOT `BoundedSemilattice` — the `Some(k)` carrier (fold then `≥ k` threshold predicate).
- [ ] 5.3 Probability carrier: product / inclusion–exclusion `CommutativeMonoid`(s) + `complement = 1 − p`, per 1.3.
- [ ] 5.4 Confirm the instances reproduce `aggregate_effects`'s results (behaviour-preserving trait-ification); property-test against the existing reducer.

## 6. Verify & hand off

- [ ] 6.1 `bazel test //deep_causality_num/...` green; `make format && make fix` clean (fix clippy, do not suppress); bare-`lean` on every new `Num/*.lean` exit 0; `unsafe_code = "forbid"` intact; `/src` macro-free.
- [ ] 6.2 Confirm the new tower does not weaken any existing numeric-tower bound (the numeric tests are unchanged and green).
- [ ] 6.3 Note the unblock: `haft` `Foldable::fold_map` (change `haft-categorical-machinery` H1) and the `Collection` order-independence proof (assumption #1) can now proceed. Prepare a commit message per task group; do not commit (await user).
