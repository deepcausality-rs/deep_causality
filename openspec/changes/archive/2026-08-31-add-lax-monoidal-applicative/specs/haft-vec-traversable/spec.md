## ADDED Requirements

### Requirement: `Traversable` keeps its `Applicative` inner bound, and `VecWitness` stays without it
`Traversable::sequence` SHALL keep `M: Applicative<M> + HKT` as its inner-witness bound, and `VecWitness` SHALL NOT implement `Traversable` in this change. A `sequence` for `VecWitness` is writable through `Semigroupal::zip_with` and was measured to compile and pass, but only if the trait's bound moves to `M: Semigroupal<M> + Pure<M> + HKT`, and that move SHALL NOT be made here.

`Applicative` and `Semigroupal + Pure` are substitutive rather than comparable: neither implies the other, so the move is a swap of admissible populations, not a loosening. Measured, it takes the witnesses admissible as the inner applicative from **19 to 3**, losing `BoxWitness`, `CausalMultiFieldWitness`, `CausalMultiVectorWitness`, `CausalTensorWitness`, `CdlEffectWitness`, `CsrMatrixWitness`, `DenseMatrixWitness`, `DenseVectorWitness`, `GraphGeneratableEffectWitness`, `LinkedListWitness`, `ManifoldWitness`, `MyEffectHktWitness`, `MyEffectHktWitness4`, `MyEffectHktWitness5`, `StudyEffectWitness` and `VecWitness` itself. Four of those are the workspace's effect monads, which are the witnesses a downstream caller would most plausibly sequence over. One carrier gained does not pay for sixteen lost.

#### Scenario: The trait's contract is unchanged

- **WHEN** `Traversable::sequence`'s bound is compared against its state before this change
- **THEN** it still reads `M: Applicative<M> + HKT`, and both existing impls are byte-identical

#### Scenario: The effect monads remain admissible as inner applicatives

- **WHEN** the set of witnesses satisfying `sequence`'s inner bound is enumerated
- **THEN** it still contains all four effect witnesses, `BoxWitness`, `LinkedListWitness` and `VecWitness`

### Requirement: The absence is documented with its measurement
The note at the foot of `hkt_vec_ext.rs` SHALL record that a `zip_with`-based `sequence` compiles and passes, that it is withheld because the required bound change costs sixteen inner witnesses to gain one, and that the change should be revisited only after `Semigroupal` is adopted across those witnesses so the bound can move without narrowing the contract. It SHALL cite the E0277 that blocks the `apply`-based fold, so the underlying constraint is not rediscovered.

#### Scenario: A reader learns why rather than only that

- **WHEN** a reader asks why `VecWitness` has no `Traversable`
- **THEN** the note gives the closure-constraint error, the measured cost of the alternative, and the precondition for revisiting

### Requirement: The `Traversable` doctest names only witnesses that implement the trait
The `Traversable::sequence` doctest SHALL compile and run rather than being fenced `rust,ignore`, and SHALL demonstrate the flip over `OptionWitness` and `ResultWitness`. It SHALL NOT reference `VecWitness::sequence`, which does not exist.

#### Scenario: Every doctest on the trait executes

- **WHEN** the crate's doctests are run
- **THEN** no `Traversable` example is skipped, and each names a witness that implements the trait
