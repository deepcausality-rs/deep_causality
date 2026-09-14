<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: `CochainWitness` binds `Cochain<R>` as a higher-kinded type
`deep_causality_topology` SHALL provide a `CochainWitness` implementing `HKT` with `type Type<T> = Cochain<T>`, exported from the crate root, so that the three cochain-adjacent siblings are uniformly witnessed.

`ChainWitness<R>` binds `Chain<R, G>` and `ExteriorDerivativeWitness` binds `DifferentialForm<T>`;
`Cochain` is the remaining sibling and is live, constructed by the cup product, the cut-cell
registry and the CFD graded-MMS verification. `Cochain<R>` carries no struct bound, so no bound
needs dropping.

#### Scenario: The witness is reachable from the crate root

- **WHEN** `CochainWitness` is imported directly from `deep_causality_topology`
- **THEN** the import resolves, and `<CochainWitness as HKT>::Type<T>` is `Cochain<T>`

### Requirement: `CochainWitness` implements `Functor` and `Foldable` and claims nothing further
`CochainWitness` SHALL implement `Functor` and `Foldable`, and SHALL NOT implement `Pure`, `Applicative`, `Monad`, `CoMonad` or `Traversable`.

This matches `ChainWitness`, which claims `Functor` and `Foldable` and no more. `Pure` is
independently wrong for this type: a cochain carries a degree, `pure` receives one value and no
degree, and any degree it picked would be invented rather than derived.

#### Scenario: The two claimed traits are usable

- **WHEN** `fmap` and `fold` are called through `CochainWitness`
- **THEN** both compile and return the mapped cochain and the folded accumulator respectively

#### Scenario: The declined traits are absent

- **WHEN** the impls for `CochainWitness` are enumerated
- **THEN** `Pure`, `Applicative`, `Monad`, `CoMonad` and `Traversable` are absent, and the reason for declining `Pure` is recorded on the witness

### Requirement: `fmap` maps the values and carries the degree through unchanged
`CochainWitness::fmap` SHALL apply the function to every value in index order, SHALL preserve the number of values, and SHALL return a cochain whose degree equals the input's degree.

The degree is the invariant that distinguishes this type from a bare `Vec`, and preserving it is
the functor's structural obligation. It is preserved for every degree, including 0.

#### Scenario: The degree survives a map that changes the element type

- **WHEN** a cochain of degree 2 is mapped by a function from one element type to another
- **THEN** the result has degree 2 and the same number of values

#### Scenario: Degree 0 survives

- **WHEN** a cochain of degree 0 is mapped
- **THEN** the result has degree 0

#### Scenario: Values are mapped in index order

- **WHEN** a cochain of at least three distinct values is mapped by a function whose result depends on the value
- **THEN** each output position holds the function applied to the input at that same position

### Requirement: `fold` reduces the values in index order
`CochainWitness::fold` SHALL fold the values in index order from the supplied initial accumulator, and SHALL return that initial accumulator unchanged for a cochain carrying no values.

#### Scenario: A non-commutative fold reveals the order

- **WHEN** a cochain of at least three distinct values is folded by an order-sensitive operation
- **THEN** the result matches the same operation applied left to right over the values

#### Scenario: An empty cochain folds to the initial accumulator

- **WHEN** a cochain carrying no values is folded
- **THEN** the result is the initial accumulator, and the folding function is never called

### Requirement: The witness satisfies the functor laws and functor/foldable consistency
`CochainWitness` SHALL be shown to satisfy the functor identity and composition laws, and its `fold` SHALL agree with a fold over the values obtained through `fmap`, tested against the laws rather than against a retyped copy of the implementation.

#### Scenario: Functor identity holds

- **WHEN** a cochain is mapped by the identity function
- **THEN** the result equals the original cochain, degree included

#### Scenario: Functor composition holds

- **WHEN** a cochain is mapped by `f` then `g`, and separately by their composition
- **THEN** the two results are equal

#### Scenario: `fold` and `fmap` agree

- **WHEN** a cochain is folded directly, and separately mapped then folded by the corresponding operation
- **THEN** the two accumulators are equal

#### Scenario: The laws are exercised where degree and length differ

- **WHEN** the law suite's inputs are reviewed
- **THEN** at least one input has a degree that differs from its value count, so a defect confusing the two cannot pass
