<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Each of groups 2 to 5 runs the five-phase cycle of `unified-math-tdd-protocol` in order: API with
`unimplemented!()` bodies, suite written against it and observed failing, defect audit against a
throwaway implementation, implementation, mutation testing. Group 1 supplies the shared fixtures
those phases need. A group's commit message is prepared at its boundary, once its tests and clippy
are green; the user commits.

## 1. Shared fixtures and corner-case enumeration

- [x] 1.1 Enumerate the corner cases in advance and commit the list: empty container, single element, all-success, first-element failure, interior failure, last-element failure, a shaped inner applicative, and for the tensor the shapes `[]`, `[0, 3]`, `[1]`, `[2, 3]`. For `Cochain`: no values, degree 0, and a degree that differs from the value count.
- [x] 1.2 Reuse the established identity-applicative fixture pattern. `haft` ships no public identity witness; the precedent is the private `Ident<T>` / `IdentWitness` (`HKT` + `Functor` + `Pure` + `Applicative`) defined inside `tests/formalization_lean/traversable_tests.rs`, with `tests/alias/*` carrying a second `Identity<T>` variant. Copy that pattern into each crate's test file rather than promoting it to public surface — test trees are not shared across crates, so it is duplicated per crate by the existing convention.
- [x] 1.3 Establish the applicative-morphism fixture for naturality, following the same file's `phi: Ident -> Option` precedent. For the container witnesses use a morphism between two carriers the container can actually hold.
- [x] 1.4 Note that no fixture crosses a crate boundary: each of the four crates defines its own, private to its test module.

## 2. `deep_causality_haft` — `Traversable` for `VecWitness`

- [x] 2.1 Phase 1: declare `impl Traversable<VecWitness> for VecWitness` with an `unimplemented!()` body; confirm `cargo build -p deep_causality_haft` and `bazel build` both succeed.
- [x] 2.2 Phase 2: write the law suite in `tests/extensions/hkt_vec_ext_tests.rs` — naturality, identity at the identity applicative, composition, order preservation over three distinct elements, first-error-wins, empty, single element, and `BoxWitness` and `VecWitness` as inner applicatives. Run it, observe every test fail with the unimplemented panic, and record the failing run and test count.
- [x] 2.2a Reinstate the disabled coverage in `tests/algebra/traversable_tests.rs`. Its header records that `VecWitness::sequence` tests were "temporarily disabled" because the impl "was removed due to constraint system complexity with closures" — the lapsed premise, stated in the test tree. Restore the `VecWitness` cases and delete that note.
- [x] 2.3 Phase 3: audit the suite against deliberate defects in a throwaway impl — reversed accumulator, a dropped element, last-error-wins instead of first, empty returning a failure. Confirm each turns the suite red; widen the suite for any that does not; discard the throwaway.
- [x] 2.4 Phase 4: implement the accumulator fold through `Applicative::apply` per design Decision 1. Document the O(n²) clone cost on the impl docstring.
- [x] 2.5 Rewrite the note at the foot of `hkt_vec_ext.rs`: the impl now exists, the `zip_with` route was rejected because the bound move costs sixteen inner witnesses for nothing the `apply` route lacks, and E0276 from the removed element marker was the original obstruction. Remove the claim that `OptionWitness` and `ResultWitness` are the only two carriers.
- [x] 2.6 Widen the `Traversable::sequence` doctest to name `VecWitness`, and confirm it runs rather than being fenced `rust,ignore`.
- [x] 2.7 Register the test file in its `mod.rs` with `#[cfg(test)]`. No `BUILD.bazel` edit is needed: all four crates glob `tests/<dir>/*_tests.rs`, so a file matching that suffix is picked up automatically — confirm the glob covers the new path rather than assuming it. Run `cargo test -p deep_causality_haft` and prepare the commit message.

## 3. `deep_causality_linear` — `Traversable` for `DenseVectorWitness`

- [ ] 3.1 Phase 1: declare the impl in `src/extensions/hkt/dense_vector_witness.rs` with an `unimplemented!()` body; confirm it builds.
- [ ] 3.2 Phase 2: write the law suite in `tests/extensions/hkt/witness_tests.rs` covering the same laws and corner rows as 2.2, plus `DenseVectorWitness` itself as the cartesian inner applicative. Observe it fail; record the run.
- [ ] 3.3 Phase 3: run the defect audit as in 2.3; discard the throwaway.
- [ ] 3.4 Phase 4: implement the fold, draining through the crate-internal `into_data()`. Document the clone cost.
- [ ] 3.5 Run `cargo test -p deep_causality_linear`; prepare the commit message.

## 4. `deep_causality_tensor` — `Traversable` for `CausalTensorWitness`

- [ ] 4.1 Phase 1: declare the impl in `src/extensions/ext_hkt.rs` with an `unimplemented!()` body; confirm it builds.
- [ ] 4.2 Phase 2: write the law suite in `tests/extensions/causal_tensor_ext_hkt_tests.rs` covering the laws, the corner rows, and shape preservation at `[2, 3]`, `[0, 3]`, `[1]` and `[]`. Observe it fail; record the run.
- [ ] 4.3 Phase 3: defect audit, including the shape-specific defects — the result shape replaced by `[len]`, and the shape read after the drain rather than before. Confirm each turns the suite red; discard the throwaway.
- [ ] 4.4 Phase 4: implement the fold, capturing the shape before `into_vec()` and moving it into the rebuilding closure.
- [ ] 4.5 Document on the impl why `sequence` preserves shape where `bind` must choose, so the two docstrings in the file do not read as inconsistent.
- [ ] 4.6 Run `cargo test -p deep_causality_tensor`; prepare the commit message.

## 5. `deep_causality_topology` — `CochainWitness`

- [ ] 5.1 Phase 1: create `src/extensions/hkt_cochain/mod.rs` (the sibling modules are alphabetical, so it sorts first, before `hkt_cell_complex`) declaring `CochainWitness`, its `HKT` binding, and `Functor` and `Foldable` impls with `unimplemented!()` bodies. Add `pub mod hkt_cochain;` to `src/extensions/mod.rs`, matching the ten existing `hkt_*` siblings, and export `CochainWitness` from `lib.rs`. `Cochain` itself is already exported at `lib.rs:63`. Confirm it builds.
- [ ] 5.2 Phase 2: write the suite in `tests/extensions/hkt_cochain_tests.rs` — functor identity and composition, fold/fmap consistency, degree preservation including degree 0, index-order mapping and folding over three distinct values, the empty cochain folding to its initial accumulator, and an input whose degree differs from its value count. Observe it fail; record the run.
- [ ] 5.3 Phase 3: defect audit — degree reset to 0, degree taken from the value count, values reversed, fold skipping the first value. Confirm each turns the suite red; discard the throwaway.
- [ ] 5.4 Phase 4: implement `fmap` mapping `values` and carrying `degree` through, and `fold` folding `values` in index order.
- [ ] 5.5 Record on the witness why `Pure` is declined: a cochain carries a degree, `pure` receives one value and no degree, and `ChainWitness` claims no `Pure` either.
- [ ] 5.6 Register the test file in `tests/extensions/mod.rs` with `#[cfg(test)]`. `BUILD.bazel` already globs `tests/extensions/*_tests.rs`, so no edit is needed there. Run `cargo test -p deep_causality_topology`; prepare the commit message.

## 6. Phase 5 — mutation testing

- [ ] 6.1 Run `scripts/mutants.sh <crate> <file>` for each of the four new impl files. The script wraps `cargo mutants -p <crate> --file <dir>/<file> -j 8` and already handles the `deep_causality_unified_math/` path prefix. Do not run it crate-wide.
- [ ] 6.2 For each surviving mutant, add the test that kills it, or record it in `.cargo/mutants.toml` with the measurement that settles it — never a bare assertion that it is fine.
- [ ] 6.3 Verify each new `.cargo/mutants.toml` entry with the `comm` check the file carries, confirming the pattern matches neither more nor less than it argues for.

## 7. Formalization — Lean proofs for the sequential `sequence`

- [x] 7.0a Write `lean/DeepCausalityFormal/Haft/TraversableList.lean` modelling the accumulator fold over an abstract `ApplicativeOps` record, with `seq_identity`, `seq_naturality` and `seq_length` (each via an accumulator-generalised `_aux` lemma). Self-contained, no imports. **Done and typechecked.**
- [x] 7.0b Add the three rows to `lean/THEOREM_MAP.md`: `haft.traversable.list.{identity, naturality, length_preserved}`. **Done.**
- [x] 7.0c Confirm `//lean:Haft` globs the new file and passes. **Done — `bazel test //lean:Haft --nocache_test_results` PASSED (1/1, genuine run), and `bazel test //lean:proofs --nocache_test_results` passes 11/11 namespaces.**
- [x] 7.0c-audit Vacuity audit of the three theorems, to the same standard phase 3 sets for tests. Two defective folds (prepend-instead-of-append, drop-the-element) were machine-checked with `by decide`: the correct fold satisfies identity on a concrete input, both defects violate it, and the dropping fold also violates length preservation. `#print axioms` reports `propext` / `Quot.sound` only, `seq_naturality` constructive, no `sorry`. **Done.**
- [ ] 7.0d Once the Rust impls land, add `tests/formalization_lean/traversable_list_tests.rs` in `deep_causality_haft` as the Rust witness the new file's header names, following the `THEOREM_MAP:` annotation convention of the existing `traversable_tests.rs`. Update the three map rows' witness column to point at it.
- [ ] 7.0e Confirm no `MODULE.bazel` `cache_roots` edit is needed — the file adds no Mathlib import — and that the theorem-map CI accepts the three new ids.

## 8. Documentation and the deferred gap

- [ ] 8.1 Update the trait table in `deep_causality_unified_math/README.md:135`. That row bundles `Traversable` with `NaturalTransformation`, `Category`, `Kleisli`, `Bifunctor` and `Profunctor` under "none", so split it: `Traversable` gets its own row naming `linear` and `tensor` (the table lists implementers *outside* `haft`, so `VecWitness` does not appear there), and the remaining five keep "none". Also add `Functor`/`Foldable` for `topology`'s new `CochainWitness` if that row does not already cover it.
- [ ] 8.2 Update `openspec/notes/unified_math/hkt_gaps.md` §5 and §6: item 1 closed, item 4 closed, with the witnesses named.
- [ ] 8.3 Add the gap-3 errata to `hkt_gaps.md` §3.2: the "afternoon" estimate is wrong, `round_policy: Truncation<<T as ConjugateScalar>::Real>` is the obstruction, dropping the struct bound yields 27 errors all resolving to that field, and the `Dual` precedent does not transfer because `Dual` had no field naming an associated type of its own parameter.
- [ ] 8.4 Correct the `Constraint`-slot documentation drift recorded in `hkt_gaps.md` §7 only if it touches the `Traversable` text this change edits; otherwise leave it to its own change and say so.
- [ ] 8.5 Decide and record whether the new witnesses' law tests carry `THEOREM_MAP:` annotations. `lean/THEOREM_MAP.md:210-211` binds `haft.traversable.identity` and `haft.traversable.naturality` to `Haft/Traversable.lean`, and those Lean theorems are stated over the trait, not per witness — so the new tests are additional Rust witnesses to existing theorems, not new theorem ids. Confirm with the theorem-map CI (`build/scripts/crates.sh` derives the crate list) before adding or omitting annotations.

## 9. Workspace verification

- [ ] 9.1 Run `make format && make fix`; fix clippy findings by rewriting rather than suppressing.
- [ ] 9.2 Run `bazel test //...` and confirm the workspace is green.
- [ ] 9.3 Confirm full line coverage of every added file, per the standing repository requirement.
- [ ] 9.4 Verify each scenario in the two new spec files and the delta spec is exercised by at least one test, and record where.
