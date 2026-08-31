## 1. The haft trait layer

- [ ] 1.1 Create `deep_causality_haft/src/lax_monoidal/mod.rs` with module docs stating the endofunctor level and doc-linking `src/monoidal/` as the value-level cartesian sibling
- [ ] 1.2 Add `Semigroupal<F: HKT>: Functor<F>` with required `zip_with` and provided `zip`, no `Clone` bound anywhere
- [ ] 1.3 Add `LaxMonoidal<F: HKT>: Semigroupal<F>` with `unit() -> F::Type<()>`, documenting that a witness which would have to fabricate a context must implement `Semigroupal` alone
- [ ] 1.4 Add `Compositional<F: HKT>: Monad<F>` and `Convolutional<F: HKT>: Semigroupal<F>` as empty markers, documenting the no-inference discipline and the coherence obligation the conjunction carries
- [ ] 1.5 Add `MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F>` with `apply` as a provided method derived from `zip_with`
- [ ] 1.6 Declare and re-export the module from `src/lib.rs`, beside the existing `SymMonoidal` re-export
- [ ] 1.7 Add a reciprocal doc-link in `src/monoidal/mod.rs` pointing at the new module
- [ ] 1.8 Add a `compile_fail` doctest proving a witness with `Semigroupal` but without `Convolutional` cannot reach `MonoidalApplicative`
- [ ] 1.9 Confirm the crate compiles with no witness implementing the new traits yet

## 2. The Lean formalization

- [ ] 2.1 Write `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean`, self-contained with no imports, transcribing the `Option` carrier
- [ ] 2.2 Prove `haft.lax_monoidal.naturality`, stated first and against `Semigroupal`
- [ ] 2.3 Prove `haft.lax_monoidal.assoc` against `Semigroupal`, modulo the associator
- [ ] 2.4 Prove `haft.lax_monoidal.unit_laws` against `LaxMonoidal`, modulo the unitors
- [ ] 2.5 Add `haft.lax_monoidal.apply_agreement` to `Applicative.lean`, leaving its four existing laws byte-identical
- [ ] 2.6 Add the four ids to the `### Haft layer` table of `lean/THEOREM_MAP.md`
- [ ] 2.7 Add Rust witnesses in `deep_causality_haft/tests/formalization_lean/`, one test per id carrying a `THEOREM_MAP:` annotation
- [ ] 2.8 Clear the deviation `Applicative.lean` reports: add the missing Composition law to the `Applicative` Rust docstring
- [ ] 2.9 Verify `bazel test //lean:Haft` passes with no edit to `lean/BUILD.bazel`, and that each `haft.monoidal.*` id still greps to exactly one row

## 3. Cayley-Dickson adoption, closing E1

- [ ] 3.1 Implement `Semigroupal` for `ComplexWitness`, `QuaternionWitness` and `OctonionWitness` with componentwise `zip_with`
- [ ] 3.2 Implement `LaxMonoidal` for all three, with `unit` returning the all-`()` value
- [ ] 3.3 Implement `Convolutional` and `MonoidalApplicative` for all three, the latter with empty bodies
- [ ] 3.4 Rewrite the `extensions/mod.rs` module doc: replace the paragraph asserting `Applicative` is unreachable with the Yoneda argument for why componentwise φ is forced, and why `pure` still is not available
- [ ] 3.5 Add law tests for naturality, associativity and both unit laws on all three arities, using varied generated inputs and at least one non-float payload
- [ ] 3.6 Add a mutation test asserting the index-crossing pairing fails the unit law

## 4. `Dual`, closing E2

- [ ] 4.1 Change `src/dual/dual_number/mod.rs:46` from `pub struct Dual<T: Real>` to `pub struct Dual<T>`, and document the storage-versus-computation split in the rustdoc
- [ ] 4.2 Add the `deep_causality_haft` dependency to `Cargo.toml`, including the `std` and `no-std` feature lists
- [ ] 4.3 Add `//deep_causality_unified_math/deep_causality_haft` to `deps` in `BUILD.bazel` and in `tests/BUILD.bazel`
- [ ] 4.4 Add `src/extensions/{mod.rs, hkt_dual.rs}` with `DualWitness` implementing `HKT` over `NoConstraint`, `Functor` and `Foldable`
- [ ] 4.5 Do NOT implement `CoMonad`; document in the module docs that it is deferred, that two comultiplications are lawful with nothing selecting between them, and that no caller wants either
- [ ] 4.6 Implement `Semigroupal`, `LaxMonoidal`, `Convolutional` and `MonoidalApplicative` for `DualWitness`
- [ ] 4.7 Document that `fmap` maps `re` and `du` independently, is the pair functor, carries no chain rule, and is not forward-mode AD
- [ ] 4.8 Declare and re-export the module from `src/lib.rs`
- [ ] 4.9 Add law tests under `tests/extensions/`, including the swap-variant mutation test and a non-`Real` payload
- [ ] 4.10 Verify `Dual<Dual<f64>>` and `Dual<Dual<Dual<f64>>>` still give correct second and third derivatives
- [ ] 4.11 Verify `cargo check --workspace --all-targets` reports zero errors after the bound comes off

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
