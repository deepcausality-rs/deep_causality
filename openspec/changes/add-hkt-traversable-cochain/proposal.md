<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

`openspec/notes/unified_math/hkt_gaps.md` §5 records `Traversable` as the multiplier for the whole
categorical layer: without `sequence`, nesting composes layers and nothing reorders them, so a
`CausalTensor<Uncertain<T>>` can only carry uncertainty as an inert payload. It ranks
`Traversable` for `DenseVectorWitness` and `CausalTensorWitness` first of six items, unblocked,
and `Cochain` fourth, blocked by nothing. Two of the three witnesses that would gain the trait
have carried `Monad` since H1 was settled per crate, so the obstruction the archived note recorded
is gone.

`Cochain<R>` is the odd sibling of three: `ChainWitness<R>` and `ExteriorDerivativeWitness` are
both witnessed and `Cochain` is not, while the cup product, the cut-cell registry and the CFD
graded-MMS verification all construct it.

## What Changes

- `Traversable` for `DenseVectorWitness` (`deep_causality_linear`), `CausalTensorWitness`
  (`deep_causality_tensor`) and `VecWitness` (`deep_causality_haft`), taking the trait's carrier
  count from two to five.
- A `CochainWitness` in `deep_causality_topology` binding `Cochain<R>` under `Functor` and
  `Foldable`, matching what `ChainWitness` claims.
- Three new Lean theorems in `lean/DeepCausalityFormal/Haft/TraversableList.lean` covering the
  sequential `sequence` the three new impls share. The existing `Traversable.lean` proves the laws
  only for `Option`'s at-most-one-element carrier, where both discharge by `cases x <;> rfl`; the
  accumulator fold needs inductions those proofs do not supply.
- **BREAKING for one spec, not for code**: `haft-vec-traversable` normatively forbids
  `VecWitness: Traversable`. That prohibition rested on a premise the code no longer has, and
  lapsed with it — see below. No public signature changes and no existing impl is edited, so no
  downstream code breaks.
- Gap 3, the `CausalTensorTrainOperator` witness, is **deferred** rather than closed, and
  `hkt_gaps.md` gets the corrected cost as an errata.

### Why the `VecWitness` prohibition lapsed

This is not a decision reversed on preference. The prohibition was sound reasoning from a premise
that has since been deleted from the code, and it lapsed when the premise did.

**The premise.** `sequence`'s accumulator fold puts an anonymous closure inside `M`. While `HKT`
carried an associated `Constraint` and every method carried `T: Satisfies<F::Constraint>`,
`Applicative::apply` required that closure to satisfy the witness constraint, `sequence` could not
declare the bound, and an impl could not add it (E0276). That left only the `Semigroupal::zip_with`
route, which requires moving `sequence`'s inner bound and costs sixteen admissible inner witnesses.
Given a closed `apply` route and an expensive `zip_with` route, withholding the impl was correct.

**The premise is gone.** `Satisfies` and the associated `Constraint` have been removed. `HKT` is
now `type Type<T>;` alone, the removal is recorded at `lax_monoidal/mod.rs:64-66`, and the code
note already says "the marker is gone, so that particular obstruction is gone with it".
`Applicative::apply` bounds only `Func: FnMut(A) -> B`. An `apply`-based `sequence` was therefore
written against the live tree for all three witnesses and it compiles and passes: `Option` and
`Result` carriers, first-error-wins ordering, empty outer structures, a shaped inner applicative
reading cartesian, and `BoxWitness` — one of the sixteen carriers a bound move would lose — still
admissible.

**What survives.** The 19→3 measurement is still true and still binding, but it constrains the
*bound*, not the impl: it is the cost of moving `sequence`'s inner bound to `Semigroupal + Pure`.
**This change does not move that bound.** `sequence` keeps `M: Applicative<M> + HKT`, so the
population is untouched and nothing is lost. The existing spec's other two requirements (the
trait's contract unchanged, the effect monads still admissible) remain satisfied and are
re-asserted rather than dropped.

One correction for the record: the existing spec obliges the note to cite "the E0277 that blocks
the `apply`-based fold". The error was **E0276**, and it no longer blocks anything.

### Why gap 3 is deferred

`hkt_gaps.md` §3.2 costs the operator witness at "an afternoon", blocked only by "the struct bound
`T: ConjugateScalar`. Drop it to the impls, as `Dual` did." That is wrong, and dropping the bound
proves it: 27 compiler errors, all resolving to one line. `CausalTensorTrainOperator<T>` holds
`round_policy: Truncation<<T as ConjugateScalar>::Real>`, and the associated type `T::Real` does
not exist without the bound. `Dual` had no such field, so the precedent does not transfer.
Resolving it means either changing the operator's stored policy representation or introducing a
bound-free core carrier — a design decision on a live type with an `Arrow` realization, not a
mechanical bound move. It is re-filed with the corrected cost.

## Capabilities

### New Capabilities
- `hkt-traversable-containers`: `Traversable` for the two shaped container witnesses and
  `VecWitness` — the `sequence` contract, the order and short-circuit semantics, shape
  preservation for the tensor, and the law obligations each impl owes.
- `topology-cochain-witness`: the `Cochain<R>` HKT witness — which traits it claims, which it
  declines and why, and the degree-preservation obligation.

### Modified Capabilities
- `haft-vec-traversable`: the requirement forbidding `VecWitness: Traversable` is replaced, and
  the change of premise that voided it is recorded in the requirement itself, so the reasoning
  stays auditable. The requirements fixing `sequence`'s inner bound at `Applicative`, preserving
  the admissible-witness population, and keeping the doctest executable are retained, with the
  doctest requirement widened to permit naming `VecWitness` now that it implements the trait.

## Impact

**Code.** Four crates, additive only:

| Crate | Change |
|---|---|
| `deep_causality_haft` | `Traversable` for `VecWitness`; rewrite the `hkt_vec_ext.rs` note to record the decision reversal and its evidence |
| `deep_causality_linear` | `Traversable` for `DenseVectorWitness` |
| `deep_causality_tensor` | `Traversable` for `CausalTensorWitness` |
| `deep_causality_topology` | `CochainWitness` (`Functor`, `Foldable`), exported from `lib.rs` |

`deep_causality_topology` is tier 7 and `deep_causality_tensor` tier 5, so both sit above `haft`
and take the trait without a dependency edit. No `Cargo.toml` dependency changes.

**Not changed.** `Traversable`'s signature, its inner-`M` bound, and the two existing
`OptionWitness` / `ResultWitness` impls. No public type gains or loses a struct bound.

**Docs.** The trait table in `deep_causality_unified_math/README.md` currently lists `Traversable`
under "Implementers outside `haft`: none"; that row moves. `hkt_gaps.md` §5 and §6 get the closure
record and the gap-3 errata.

**Tests.** New law suites per witness under each crate's existing `tests/extensions/` tree, plus a
`cargo mutants` run scoped to the new code.

**Formalization.** `lean/DeepCausalityFormal/Haft/TraversableList.lean` (new, no imports) and three
`THEOREM_MAP.md` rows. `//lean:Haft` globs its directory, so no `BUILD.bazel` edit is needed; the
file adds no Mathlib import, so `cache_roots` in `MODULE.bazel` is untouched.
