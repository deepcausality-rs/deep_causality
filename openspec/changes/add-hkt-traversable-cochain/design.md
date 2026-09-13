<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

`Traversable` has two carriers, `OptionWitness` and `ResultWitness`, both in `haft` and both over
containers holding at most one element. No shaped container implements it. `hkt_gaps.md` §5 ranks
that first of six items and calls it the multiplier: `sequence` is what turns a nesting into a
reordering, and without it `CausalTensor<Uncertain<T>>` can only carry uncertainty beside the
computation rather than under it.

Three facts about the current tree set the approach.

- The `Constraint` slot is gone from `HKT` (`hkt_gaps.md` §1; `lax_monoidal/mod.rs:64-66`). With
  it went E0276, the error that once stopped an impl from bounding the anonymous closure type that
  `sequence`'s accumulator fold puts inside `M`.
- `Applicative::apply` is `fn apply<A, B, Func>(F::Type<Func>, F::Type<A>) -> F::Type<B> where A: Clone, Func: FnMut(A) -> B`.
  It bounds `Func` and nothing else, so a closure accumulator is admissible.
- H1 is settled per crate. `DenseVectorWitness` and `CausalTensorWitness` each carry `Monad`, so
  neither is in the state that once made `Traversable` premature.

`Traversable: Functor<F> + Foldable<F>`, and all three target witnesses already have both
supertraits, so no supertrait work is needed.

The prior decision recorded in `haft-vec-traversable` is the one live constraint. Its premise is
the first bullet above, which no longer holds, so Decision 2 records the lapse rather than working
around the requirement.

## Goals / Non-Goals

**Goals:**

- `Traversable` for `DenseVectorWitness`, `CausalTensorWitness` and `VecWitness`.
- `CochainWitness` binding `Cochain<R>` under `Functor` and `Foldable`.
- Lean proofs of the identity and naturality laws for the sequential `sequence`, plus element-count
  preservation, since the existing `Option` proofs do not cover a multi-element carrier.
- A law-based test suite that is shown to reject wrong implementations before the implementations
  are written, per `unified-math-tdd-protocol`.
- The `haft-vec-traversable` prohibition replaced, recording the change of premise that voided it
  rather than silently contradicting a live requirement.

**Non-Goals:**

- Changing `Traversable`'s signature or its inner-`M` bound. The bound stays `Applicative`.
- Editing the `OptionWitness` or `ResultWitness` impls.
- The `Uncertain` witness (gap 2). This change is its precondition, not its delivery.
- The `CausalTensorTrainOperator` witness (gap 3), deferred per Decision 5.
- `Traversable` for `CsrMatrixWitness`, `DenseMatrixWitness`, the `Zip*` witnesses, or any
  `topology` witness. Not requested and not ranked.
- Any `Pure`, `Applicative` or `Monad` claim for `Cochain`.
- The Lean composition law. It is deferred for `Option` already, on hypotheses this change does not
  supply.
- Any edit to `Haft/Traversable.lean`. The new proofs live in their own file.

## Decisions

### Decision 1: `sequence` is the left-to-right accumulator fold through `Applicative::apply`

For each of the three container witnesses:

```rust
let mut acc: M::Type<Vec<A>> = M::pure(Vec::new());
for m_a in <drain the container in index order> {
    acc = M::apply(M::fmap(acc, |v: Vec<A>| move |a: A| { let mut v = v.clone(); v.push(a); v }), m_a);
}
M::fmap(acc, <rebuild the container>)
```

**Why.** It is the standard list traversal, it keeps the inner bound at `Applicative`, and it was
measured to compile and pass for all three witnesses against the live tree before this document
was written.

**Alternative rejected: the `zip_with` fold over `Semigroupal`.** The code note at
`hkt_vec_ext.rs:134-147` prefers it because the combining function never enters `M`. It requires
moving `sequence`'s inner bound to `Semigroupal + Pure`, which is substitutive with `Applicative`
rather than weaker: measured, that takes admissible inner witnesses from 19 to 3 and loses every
effect monad in the workspace. The `apply` fold needs no such move, so the trade is unnecessary.

**Consequence to state, not hide.** `A: Clone` is on the trait, and the accumulator clones the
`Vec` once per element, so `sequence` is O(n²) in moves for an n-element container. Correct and
lawful, and the honest characterisation is "not the fold to use in a hot loop". Recorded on each
impl. Optimising it means a bound the trait does not carry, and does not belong in this change.

### Decision 2: the `haft-vec-traversable` prohibition lapsed with its premise

An existing spec says `VecWitness` SHALL NOT implement `Traversable`. It is not overridden on
preference: it was correct reasoning from a premise — the `Satisfies`/`Constraint` machinery on
`HKT` closing the `apply` route — that has since been deleted from the code. Both of its supports
were checked against the tree, and one of them no longer exists:

| Spec's support | Measured state |
|---|---|
| "cite the E0277 that blocks the `apply`-based fold" | No such blocker. The code note says **E0276**, caused by the element marker, and that the marker "is gone, so that particular obstruction is gone with it". The `apply` fold compiles today. |
| 19→3 admissible inner witnesses | Real, but it is the cost of the `Semigroupal` bound move. This change does not make that move, so it pays nothing. |

The strongest evidence that this is a lapsed premise rather than a reversed preference is in the
test tree. `tests/algebra/traversable_tests.rs` opens with a note that `VecWitness::sequence` tests
are "temporarily disabled" because the impl "was removed due to constraint system complexity with
closures". The impl existed, the constraint system removed it, and the constraint system is now
gone. Task 2.2a reinstates that coverage and deletes the note.

The spec's two other requirements survive and are re-asserted in the delta: `sequence` keeps its
`Applicative` bound, and the effect monads stay admissible. Only the prohibition changes.

### Decision 3: the tensor's `sequence` preserves shape; the shape is captured before the drain

`CausalTensor` carries a shape independent of its length, and the H1 corners in `bind` all come
from a shaped container having to choose one. `sequence` has no such choice: it is one-in-one-out
by construction, so the input's shape is the only defensible answer and `[2, 3]` comes back
`[2, 3]` rather than `[6]`.

The shape is read before `into_vec()` consumes the tensor and moved into the rebuilding closure.
Measured on the live tree: `[2, 3]`, `[0, 3]` and the rank-0 `[]` all survive.

This is why `sequence` avoids the corner that cost `bind` its associativity: `bind`'s continuation
may return any number of elements, `sequence`'s cannot.

### Decision 4: `CochainWitness` claims `Functor` and `Foldable`, and declines `Pure`

`hkt_gaps.md` §3.3 says the `Pure` question "follows whatever `ChainWitness` claims".
`ChainWitness<R>` implements `Functor` and `Foldable` and no more, so `CochainWitness` matches.

`Pure` is independently wrong here: a `Cochain` carries a degree, `pure` gets one value and no
degree, and any choice — degree 0, or the degree of nothing — is invented rather than derived.

`Cochain<R>` has no struct bound, so the witness needs no bound drop. `fmap` maps `values` and
carries `degree` through unchanged; `fold` folds `values` in index order. Degree preservation is
the functor's structural obligation and is the property the tests pin.

**Not claimed: `Traversable` for `CochainWitness`.** It would follow the same fold, but it is not
in the requested scope and `ChainWitness` does not have it either.

### Decision 5: gap 3 is deferred, and `hkt_gaps.md` gets an errata rather than a silent omission

`hkt_gaps.md` §3.2 costs the operator witness at "an afternoon", blocked only by a struct bound to
be dropped "as `Dual` did". Measured: dropping `T: ConjugateScalar` from
`CausalTensorTrainOperator<T>` yields 27 errors, and every one resolves to
`round_policy: Truncation<<T as ConjugateScalar>::Real>` — the associated type does not exist
without the bound. `Dual` carried no field mentioning an associated type of its own parameter, so
the precedent does not transfer.

Resolving it is a design decision on a live type with an `Arrow` realization: either change how
the policy is stored, or add a bound-free core carrier and convert. Both are larger than this
change and neither should be picked in passing. The note is corrected so the next reader does not
re-derive the same wrong estimate.

### Decision 6: laws are the independent oracle, and each impl owes the same three

`unified-math-tdd-protocol` requires an expectation obtained independently of the code under test.
Category theory supplies one directly: the laws are stated in the literature (McBride & Paterson,
JFP 18(1) 2008 §3, cited on the trait) and are properties of any correct implementation, so a law
test cannot be satisfied by retyping the implementation's formula.

Per container witness:

1. **Naturality** — for an applicative morphism `phi`, `phi(sequence(t)) == sequence(fmap(t, phi))`.
   `Option -> Result` is the morphism used.
2. **Identity** — `sequence` at the identity applicative returns the structure unchanged. The
   trait's docstring warns that the weaker phrasing is vacuous; the identity applicative is used.
3. **Composition** — sequencing a composite equals composing the sequenced results.

Plus per witness: short-circuit on the first failing element, order preservation, and for the
tensor, shape preservation.

`CochainWitness` owes the functor laws (identity, composition), the foldable/functor consistency
law, and degree preservation.

**Corner rows enumerated in advance**, per the protocol: the empty container; the single-element
container; all-success and first-failure and last-failure; a shaped inner applicative reading
cartesian; and for the tensor `[]`, `[0, 3]`, `[1]`, `[2, 3]`. For `Cochain`: empty values,
degree 0, and a degree above 0 with a length that does not equal the degree.

**The identity-applicative fixture already has a precedent, and it is private.** `haft` ships no
public identity witness. `tests/formalization_lean/traversable_tests.rs` defines a private
`Ident<T>` with `IdentWitness` carrying `HKT`, `Functor`, `Pure` and `Applicative`, specifically
because — in that file's own words — "the docstring's own version is vacuous; deviation D5". A
second `Identity<T>` variant sits in `tests/alias/`. The suites here copy that pattern per crate
rather than promoting it to public surface: test trees are not shared across crates, and the
existing convention is duplication.

**The laws are also proved in Lean, because the existing proofs do not cover these carriers.**
`Haft/Traversable.lean` proves identity and naturality for `OptionWitness::sequence`, whose carrier
holds at most one element — both proofs are `cases x <;> rfl`. The three new impls run an
accumulator fold over many elements, and their laws are inductions with a *generalised
accumulator*: the accumulator is not invariant across a step, so the statement has to quantify
over it. Nothing in the `Option` file implies them. Decision 8 records what was added.

**The composition law is Rust-only here, and that is deliberate.** `lean/THEOREM_MAP.md` binds
`haft.traversable.identity` and `haft.traversable.naturality` to `Haft/Traversable.lean` as
proved, and lists `haft.traversable.composition` under "Not yet on the map", blocked on
"lawful-applicative hypotheses for `M`, `N` (scaling)". This change tests composition in Rust at
concrete carriers, which is tractable, and does not attempt the Lean proof. The two existing
theorem ids are stated over the trait rather than per witness, so the new tests are additional
Rust witnesses to existing theorems and introduce no new theorem id — task 7.5 confirms that
against the theorem-map CI rather than assuming it.

### Decision 8: the Lean proofs are a new file, and they strengthen the morphism record

`Haft/TraversableList.lean` models the shared kernel of all three impls — the accumulator fold —
over an abstract applicative given as an operations record, so the theorems quantify over every
applicative the Rust code could see. The three impls differ only in how the result is rebuilt (a
`Vec`, a `DenseVector`, a `CausalTensor` re-wearing the input shape), which is a bijection on the
element list and does not affect the laws. Three theorems, all machine-checked:

| Theorem | What it pins |
|---|---|
| `haft.traversable.list.identity` | `sequence` at the Identity applicative is the identity — also pins element order, so a reversing or dropping fold fails it |
| `haft.traversable.list.naturality` | applicative morphisms commute with `sequence` |
| `haft.traversable.list.length_preserved` | the element count is preserved |

Two points worth stating rather than burying.

**The morphism record needs `preserves_apply`.** `Option`'s `sequence` consumes only `pure` and
`fmap`, so `Traversable.lean`'s `ApplicativeMorphism` carries only those two. The fold also calls
`apply`, so naturality over it genuinely requires the third. This is not a weakening: preserving
`apply` is part of the standard definition of an applicative morphism (McBride–Paterson 2008 §6),
which the existing file omitted only because its carrier never exercised it. The new file declares
its own record rather than editing the existing one, so `Traversable.lean` is untouched.

**The theorems were audited for vacuity, not just typechecked.** A proof that passes proves nothing
about the code unless it also rejects wrong code — the same standard `unified-math-tdd-protocol`
phase 3 applies to tests. Two defective folds were written and machine-checked against the laws:
one prepending instead of appending (reversing order), one dropping the element. `by decide`
confirms the correct fold satisfies identity on a concrete input while both defects violate it,
and that the dropping fold also violates length preservation. An axiom audit
(`#print axioms`) shows the three theorems rest only on `propext` and `Quot.sound`, with
`seq_naturality` fully constructive; there is no `sorry` and no added axiom.

**`length_preserved` is the formal content of the tensor's shape claim.** `CausalTensorWitness`
re-wears the input's shape on the output, which is sound exactly because the element count cannot
change. That is the property Decision 3 argues informally and this theorem states. `bind` has no
counterpart, which is the asymmetry Decision 3 records.

Composition stays deferred, as it is for `Option`: `THEOREM_MAP.md` lists
`haft.traversable.composition` as blocked on lawful-applicative hypotheses for both `M` and `N`.
This change does not lift that, and does not add a `.list` variant of it.

### Decision 7: file placement follows each crate's existing tree

| Crate | Impl | Tests |
|---|---|---|
| `haft` | `src/extensions/hkt_vec_ext.rs` | `tests/extensions/hkt_vec_ext_tests.rs` |
| `linear` | `src/extensions/hkt/dense_vector_witness.rs` | `tests/extensions/hkt/witness_tests.rs` |
| `tensor` | `src/extensions/ext_hkt.rs` | `tests/extensions/causal_tensor_ext_hkt_tests.rs` |
| `topology` | `src/extensions/hkt_cochain/mod.rs` (new) | `tests/extensions/hkt_cochain_tests.rs` (new) |

Each `Traversable` impl joins the file already holding its witness's other impls, per one-type-one-module.
New test files are registered in their `mod.rs` with `#[cfg(test)]` and in the crate's `BUILD.bazel`.

## Risks / Trade-offs

**[Contradicting a live spec sets a precedent for overriding requirements on convenience]** →
The override is confined to one requirement, both of its supports were re-measured against the
tree and are recorded in the proposal and in Decision 2, and the spec's other two requirements are
re-asserted rather than dropped. The wrong E0277 citation is itself a defect in the existing spec
and is corrected rather than inherited.

**[The O(n²) clone makes `sequence` unusable on a large tensor and someone finds out in production]**
→ Stated on each impl docstring and in the spec, not left to be discovered. No caller in the
workspace uses `sequence` on a shaped container today, so nothing regresses.

**[Law tests pass vacuously — the failure mode `AGENTS.md` documents]** → Phase 3 of the protocol
is mandatory here: deliberate defects (a reversed accumulator, a dropped element, the tensor's
shape replaced by `[len]`, `Cochain`'s degree reset to 0) are introduced into a throwaway
implementation and each must turn the suite red before phase 4 begins. Every law is exercised at
a container with at least two distinct elements, so a permutation defect cannot hide.

**[The tensor's shape decision conflicts with the `bind` corners recorded in `ext_hkt.rs`]** →
It does not, and the reason is stated in Decision 3: `bind` chooses because its continuation may
change the element count, `sequence` cannot. The distinction is recorded on the impl so a reader
comparing the two files does not read it as an inconsistency.

**[`cargo mutants` over a whole crate is prohibitively slow]** → `AGENTS.md` states it is not a
whole-workspace gate. The run is scoped to the four new impl regions with
`--file`, not to the crates.

## Migration Plan

Additive; no rollback beyond reverting the commits. Five groups, each green before the next, and
each following the five-phase cycle:

1. `haft` — `VecWitness`, plus the note rewrite. Lowest tier, and it proves the fold shape.
2. `linear` — `DenseVectorWitness`.
3. `tensor` — `CausalTensorWitness`, including the shape corners.
4. `topology` — `CochainWitness`.
5. Docs — the `README.md` trait table, `hkt_gaps.md` §5/§6 closure and the gap-3 errata.

`bazel test //...` is the workspace gate; `cargo test -p <crate>` for iteration within a group.

## Open Questions

None blocking. Two settled above that a reader might expect to be open: whether `VecWitness` is in
scope (yes, Decision 2) and whether `Cochain` gets `Pure` (no, Decision 4).
