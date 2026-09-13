<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: `Traversable` keeps its `Applicative` inner bound, and `VecWitness` implements it
`Traversable::sequence` SHALL keep `M: Applicative<M> + HKT` as its inner-witness bound, and `VecWitness` SHALL implement `Traversable` through an accumulator fold written against that bound. The bound SHALL NOT move to `M: Semigroupal<M> + Pure<M> + HKT`.

The previous version of this requirement forbade the impl. That prohibition is not overridden on
preference; **its premise has been removed from the code**, and the requirement lapsed with it.

**The premise.** The prohibition rested on `sequence`'s accumulator fold being unwritable against
the `Applicative` bound. That fold puts an anonymous closure inside `M`. While `HKT` carried an
associated `Constraint` and every method carried `T: Satisfies<F::Constraint>`, `Applicative::apply`
required that closure type to satisfy the witness constraint, `sequence` could not declare the
bound, and an impl could not add it — E0276. With the `apply` route closed, the only remaining
route was `Semigroupal::zip_with`, which needs the inner bound moved, and that move costs sixteen
inner witnesses. Forbidding the impl was the correct conclusion **from that premise**.

**The premise is gone.** `Satisfies` and the associated `Constraint` have been removed. `HKT` is
now `type Type<T>;` and nothing else (`hkt/mod.rs`), the removal is recorded at
`lax_monoidal/mod.rs:64-66`, and the code note at `hkt_vec_ext.rs` already states that "the marker
is gone, so that particular obstruction is gone with it". `Applicative::apply` now bounds only
`Func: FnMut(A) -> B`, so the closure accumulator is admissible and the `apply`-based fold compiles
and passes — measured on the tree for `VecWitness` and for the two shaped container witnesses.

**What follows.** The `zip_with` route is no longer the only route, so its cost is no longer a
reason to withhold the impl. The sixteen-witness measurement itself still stands and still governs
the bound: `Applicative` and `Semigroupal + Pure` are substitutive rather than comparable, and
moving the bound would take the admissible inner witnesses from **19 to 3**, losing `BoxWitness`,
`CausalMultiFieldWitness`, `CausalMultiVectorWitness`, `CausalTensorWitness`, `CdlEffectWitness`,
`CsrMatrixWitness`, `DenseMatrixWitness`, `DenseVectorWitness`, `GraphGeneratableEffectWitness`,
`LinkedListWitness`, `ManifoldWitness`, `MyEffectHktWitness`, `MyEffectHktWitness4`,
`MyEffectHktWitness5`, `StudyEffectWitness` and `VecWitness` itself. That is why the bound stays
at `Applicative` — and why the impl, which needs no such move, is admitted at no cost to the
trait's contract.

One consequence for the record: the previous requirement obliged the note to cite "the E0277 that
blocks the `apply`-based fold". That citation was wrong on two counts — the error was E0276, and
it no longer blocks anything.

#### Scenario: The trait's contract is unchanged

- **WHEN** `Traversable::sequence`'s bound is compared against its state before this change
- **THEN** it still reads `M: Applicative<M> + HKT`, and the `OptionWitness` and `ResultWitness` impls are byte-identical

#### Scenario: The lapsed premise is confirmed absent rather than assumed

- **WHEN** `HKT` and `Applicative::apply` are read on the current tree
- **THEN** `HKT` declares `type Type<T>;` with no associated `Constraint`, and `apply` bounds only `Func: FnMut(A) -> B`, so no bound on the closure accumulator is required

#### Scenario: The effect monads remain admissible as inner applicatives

- **WHEN** the set of witnesses satisfying `sequence`'s inner bound is enumerated
- **THEN** it still contains all four effect witnesses, `BoxWitness`, `LinkedListWitness` and `VecWitness`

#### Scenario: `VecWitness` sequences over the carriers the bound move would have lost

- **WHEN** a `Vec` is sequenced with `BoxWitness`, and separately with `VecWitness`, as the inner applicative
- **THEN** both calls compile and return the flipped structure

### Requirement: The decision and its change of premise are documented
The note at the foot of `hkt_vec_ext.rs` SHALL record that `VecWitness` implements `Traversable` through an `apply`-based fold, and SHALL record why the earlier decision to withhold it no longer applies: the E0276 that blocked the fold was raised by the `Satisfies`/`Constraint` machinery on `HKT`, that machinery has been removed, and the fold compiles without it.

The note SHALL also keep the measurement that remains live — that moving the inner bound to
`Semigroupal + Pure` would cost sixteen admissible inner witnesses to gain one — and SHALL state
that this is why the bound is retained rather than why the impl is withheld. Recording both keeps
the reasoning auditable: one input to the earlier decision lapsed, the other did not, and the
conclusion changed only because of the first.

#### Scenario: A reader learns which premise changed

- **WHEN** a reader asks why `VecWitness` now implements `Traversable` when a prior decision forbade it
- **THEN** the note identifies the removed `Constraint` slot and its E0276 as the lapsed premise, and distinguishes it from the sixteen-witness measurement, which still holds and still pins the bound

#### Scenario: The superseded conclusion is not left standing

- **WHEN** the note is read after this change
- **THEN** it does not claim that `OptionWitness` and `ResultWitness` are the only two `Traversable` carriers, and does not attribute the obstruction to E0277

### Requirement: The `Traversable` doctest names only witnesses that implement the trait
The `Traversable::sequence` doctest SHALL compile and run rather than being fenced `rust,ignore`, and SHALL demonstrate the flip over witnesses that implement the trait. It SHALL NOT name a witness that does not implement it.

`VecWitness` becomes admissible in the doctest with this change; the constraint is that every
witness named implements the trait, not which ones are chosen.

#### Scenario: Every doctest on the trait executes

- **WHEN** the crate's doctests are run
- **THEN** no `Traversable` example is skipped, and each names a witness that implements the trait
