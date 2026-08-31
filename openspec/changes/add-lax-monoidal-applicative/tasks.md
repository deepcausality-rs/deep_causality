## 1. The haft trait layer

- [x] 1.1 Create `deep_causality_haft/src/lax_monoidal/mod.rs` with module docs stating the endofunctor level and doc-linking `src/monoidal/` as the value-level cartesian sibling
- [x] 1.2 Add `Semigroupal<F: HKT>: Functor<F>` with required `zip_with` and provided `zip`, no `Clone` bound anywhere
- [x] 1.3 Add `LaxMonoidal<F: HKT>: Semigroupal<F>` with `unit() -> F::Type<()>`, documenting that a witness which would have to fabricate a context must implement `Semigroupal` alone
- [x] 1.4 Add `Compositional<F: HKT>: Monad<F>` and `Convolutional<F: HKT>: Semigroupal<F>` as empty markers, documenting the no-inference discipline and the coherence obligation the conjunction carries
- [x] 1.5 Add `MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F>` with `apply` as a provided method derived from `zip_with`
- [x] 1.6 Declare and re-export the module from `src/lib.rs`, beside the existing `SymMonoidal` re-export
- [x] 1.7 Add a reciprocal doc-link in `src/monoidal/mod.rs` pointing at the new module
- [x] 1.8 Add a `compile_fail` doctest proving a witness with `Semigroupal` but without `Convolutional` cannot reach `MonoidalApplicative`
- [x] 1.9 Confirm the crate compiles with no witness implementing the new traits yet

## 2. The Lean formalization

- [x] 2.1 Write `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean`, self-contained with no imports, transcribing the `Option` carrier
- [x] 2.2 Prove `haft.lax_monoidal.naturality`, stated first and against `Semigroupal`
- [x] 2.3 Prove `haft.lax_monoidal.assoc` against `Semigroupal`, modulo the associator
- [x] 2.4 Prove `haft.lax_monoidal.unit_laws` against `LaxMonoidal`, modulo the unitors
- [x] 2.5 Prove `haft.lax_monoidal.apply_agreement` in `LaxMonoidal.lean`, transcribing both sides locally since the file is self-contained; leave `Applicative.lean`'s theorems byte-identical
- [x] 2.6 Add the four ids to the `### Haft layer` table of `lean/THEOREM_MAP.md`
- [x] 2.7 Add Rust witnesses in `deep_causality_haft/tests/formalization_lean/`, one test per id carrying a `THEOREM_MAP:` annotation
- [x] 2.8 Remove the stale DEVIATION NOTE from `Applicative.lean`'s header; the Composition law is already in the Rust docstring and deviations note D1 is marked RESOLVED, so no docstring edit is needed
- [x] 2.9 Verify `bazel test //lean:Haft` passes with no edit to `lean/BUILD.bazel`, and that each `haft.monoidal.*` id still greps to exactly one row

## 3. Cayley-Dickson adoption, closing E1

- [x] 3.1 Implement `Semigroupal` for `ComplexWitness`, `QuaternionWitness` and `OctonionWitness` with componentwise `zip_with`
- [x] 3.2 Implement `LaxMonoidal` for all three, with `unit` returning the all-`()` value
- [x] 3.3 Implement `Convolutional` and `MonoidalApplicative` for all three, the latter with empty bodies
- [x] 3.4 Rewrite the `extensions/mod.rs` module doc: replace the paragraph asserting `Applicative` is unreachable with the Yoneda argument for why componentwise φ is forced, and why `pure` still is not available
- [x] 3.5 Add law tests for naturality, associativity and both unit laws on all three arities, using varied generated inputs and at least one non-float payload
- [x] 3.6 Add a mutation test asserting the index-crossing pairing fails the unit law

## 4. `Dual`, closing E2

- [x] 4.1 Change `src/dual/dual_number/mod.rs:46` from `pub struct Dual<T: Real>` to `pub struct Dual<T>`, and document the storage-versus-computation split in the rustdoc
- [x] 4.2 Add the `deep_causality_haft` dependency to `Cargo.toml`, including the `std` and `no-std` feature lists
- [x] 4.3 Add `//deep_causality_unified_math/deep_causality_haft` to `deps` in `BUILD.bazel` and in `tests/BUILD.bazel`
- [x] 4.4 Add `src/extensions/{mod.rs, hkt_dual.rs}` with `DualWitness` implementing `HKT` over `NoConstraint`, `Functor` and `Foldable`
- [x] 4.5 Do NOT implement `CoMonad`; document in the module docs that it is deferred, that two comultiplications are lawful with nothing selecting between them, and that no caller wants either
- [x] 4.6 Implement `Semigroupal`, `LaxMonoidal`, `Convolutional` and `MonoidalApplicative` for `DualWitness`
- [x] 4.7 Document that `fmap` maps `re` and `du` independently, is the pair functor, carries no chain rule, and is not forward-mode AD
- [x] 4.8 Declare and re-export the module from `src/lib.rs`
- [x] 4.9 Add law tests under `tests/extensions/`, including the swap-variant mutation test and a non-`Real` payload
- [x] 4.10 Verify `Dual<Dual<f64>>` and `Dual<Dual<Dual<f64>>>` still give correct second and third derivatives
- [x] 4.11 Verify `cargo check --workspace --all-targets` reports zero errors after the bound comes off

## 5. `Traversable` for `VecWitness`

- [ ] 5.1 Implement `Traversable<VecWitness>` for `VecWitness`, folding through the inner witness with `zip_with` seeded by `M::pure(Vec::new())`
- [ ] 5.2 Bound the inner witness on `Semigroupal` and `Pure`, not on `MonoidalApplicative`
- [ ] 5.3 Remove the note at the foot of `hkt_vec_ext.rs` recording the absence
- [ ] 5.4 Restore the `Vec<Option<A>> -> Option<Vec<A>>` example to the `Traversable::sequence` doctest now that it compiles
- [ ] 5.5 Add tests for the all-present, contains-`None`, empty, and `ResultWitness` cases

## 6. Verification and documentation

- [ ] 6.1 Confirm `VecWitness::apply` still accepts a `Func` that is `FnMut` and not `Clone`, and still returns the six-element cartesian product for two functions and three arguments
- [ ] 6.2 Confirm `Applicative`, `Pure`, `Monad`, `Traversable`, `Arrow` and the effect system have unchanged signatures, and that all 22 existing `Applicative` impls are untouched
- [ ] 6.3 Update `website/docs/src/content/docs/formalization/haft.md` to 53 rows and the spelled-out count to fifty-three
- [ ] 6.4 Run `bazel test //...` and record the pass count
- [ ] 6.5 Run clippy on `deep_causality_haft`, `deep_causality_num_complex` and `deep_causality_num_dual`, fixing rather than suppressing any lint
- [ ] 6.6 Update `openspec/notes/unified_math/unified_math_gaps.md` to mark E1 and E2 closed
- [ ] 6.7 Update `openspec/notes/hkt_gat/monoidal-applicative.md` status from proposed to implemented, retaining the survey and the measurements
