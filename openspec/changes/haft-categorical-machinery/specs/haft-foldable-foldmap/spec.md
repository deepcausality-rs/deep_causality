## ADDED Requirements

### Requirement: Foldable gains a monoidal fold_map

`deep_causality_haft` `Foldable` SHALL provide `fn fold_map<A, M: Monoid>(fa: F::Type<A>, f: impl Fn(A) -> M) -> M`, a default implemented in terms of the existing seeded `fold` and the `Monoid` (`empty`/`combine`) from `num-generic-monoid-tower`. This expresses a `Collection` as a fold-map into its aggregation monoid.

#### Scenario: fold_map folds into a monoid

- **WHEN** `fold_map` is applied to a container with `f` mapping each element to a monoid `M`
- **THEN** the result is the monoid combine of all mapped elements, with `M::empty()` as the seed, equal to `fold` threading `combine`

#### Scenario: order-independence under a commutative monoid

- **WHEN** `M: CommutativeMonoid` and the container is treated as a multiset
- **THEN** `fold_map` is independent of iteration order (this is the coherence the Collection order-independence proof reuses)

### Requirement: fold_map coherence is tested and proved in Lean

The `fold_map` coherence laws (`fold_map(pure a, f) = f a`; homomorphism respect for `empty`/`combine`) SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Haft/Foldable.lean` (bare-`lean`), bound by `THEOREM_MAP.md` ids (`haft.foldable.fold_map_pure`, `haft.foldable.fold_map_monoid_coherence`) with Rust witnesses.

#### Scenario: Both bridge sides exist

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** each `haft.foldable.fold_map_*` id has a `proved` Lean location and a passing Rust witness, and the Lean file typechecks standalone with bare `lean`
