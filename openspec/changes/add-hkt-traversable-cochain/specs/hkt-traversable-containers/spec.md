<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Three container witnesses implement `Traversable`
`DenseVectorWitness`, `CausalTensorWitness` and `VecWitness` SHALL each implement `Traversable`, taking the trait's carrier count from two to five, and SHALL do so without altering `Traversable`'s signature, its inner-`M` bound, or either existing impl.

`Traversable: Functor<F> + Foldable<F>`, and all three witnesses already carry both supertraits, so
no supertrait work is required. Each impl lives in the file that already holds its witness's other
categorical impls.

#### Scenario: Each witness is usable as a `Traversable` carrier

- **WHEN** `sequence` is called through each of the three witnesses with `OptionWitness` as the inner applicative
- **THEN** each call compiles and returns the flipped structure

#### Scenario: The trait's contract is unchanged

- **WHEN** `Traversable::sequence`'s declaration is compared against its state before this change
- **THEN** it still reads `M: Applicative<M> + HKT`, and the `OptionWitness` and `ResultWitness` impls are byte-identical

### Requirement: `sequence` is the left-to-right accumulator fold and preserves element order
`sequence` SHALL traverse the container in index order and SHALL preserve that order in the result, so that a successful `sequence` returns the same elements, in the same positions, that the input carried.

The fold accumulates through `Applicative::apply`, keeping the inner bound at `Applicative`. The
accumulator clones once per element, so `sequence` is O(n²) in moves for an n-element container;
this SHALL be recorded on each impl's docstring rather than left for a caller to discover.

#### Scenario: Order survives a multi-element traversal

- **WHEN** a container of at least three distinct successful elements is sequenced
- **THEN** the result carries those elements in their original positions

#### Scenario: The cost is documented

- **WHEN** each new `Traversable` impl's docstring is read
- **THEN** it states that the accumulator clones once per element and that the fold is quadratic in moves

### Requirement: A failing element short-circuits the whole traversal, and the first failure wins
`sequence` SHALL return the inner applicative's failure when any element carries one, SHALL report the **first** such element in index order, and SHALL NOT return a partial structure.

#### Scenario: One `None` collapses the container

- **WHEN** a container whose elements are `Some` except at one interior position is sequenced over `OptionWitness`
- **THEN** the result is `None`

#### Scenario: The first error wins over a later one

- **WHEN** a container carrying `Ok`, then `Err("a")`, then `Err("b")` is sequenced over `ResultWitness`
- **THEN** the result is `Err("a")`

#### Scenario: An all-success traversal carries every element

- **WHEN** a container whose every element is `Some` is sequenced over `OptionWitness`
- **THEN** the result is `Some` and carries every element

### Requirement: An empty container sequences to the inner applicative's `pure` of the empty container
`sequence` SHALL return `M::pure` of the empty container when the outer container holds no elements, rather than failing, panicking, or returning a fabricated element.

This is the identity element of the fold and the case a hand-written loop most often gets wrong.

#### Scenario: The empty container succeeds

- **WHEN** an empty container is sequenced over `OptionWitness`
- **THEN** the result is `Some` and carries an empty container

#### Scenario: A single-element container round-trips

- **WHEN** a one-element container carrying `Some(x)` is sequenced
- **THEN** the result is `Some` and carries exactly one element, `x`

### Requirement: `CausalTensorWitness::sequence` preserves the input tensor's shape
`CausalTensorWitness::sequence` SHALL return a tensor whose shape equals the input tensor's shape, reading that shape before the input is consumed, so that a `[2, 3]` input returns a `[2, 3]` result rather than a flat `[6]`.

`sequence` is one-in-one-out by construction, so unlike `bind` it has no shape choice to make: its
continuation cannot change the element count. The impl SHALL record that distinction, so a reader
comparing it against the `bind` corner cases documented in the same file does not read it as an
inconsistency.

#### Scenario: A rank-2 shape survives

- **WHEN** a `[2, 3]` tensor of `Some` values is sequenced
- **THEN** the resulting tensor reports shape `[2, 3]` and carries the six values in order

#### Scenario: A zero-extent shape survives

- **WHEN** a tensor of shape `[0, 3]` is sequenced
- **THEN** the result succeeds and reports shape `[0, 3]`, not `[0]`

#### Scenario: The rank-0 shape survives

- **WHEN** a one-element tensor of shape `[]` is sequenced
- **THEN** the result succeeds and reports shape `[]`, not `[1]`

#### Scenario: The relationship to `bind` is recorded

- **WHEN** the `Traversable` impl's docstring is read
- **THEN** it states why `sequence` preserves shape where `bind` must choose one

### Requirement: Each implementation satisfies the naturality, identity and composition laws
Each new `Traversable` impl SHALL be shown to satisfy the three laws stated on the trait, tested against the laws as an implementation-independent oracle rather than against a retyped copy of the implementation.

The laws are McBride & Paterson, JFP 18(1) 2008 §3, already cited on the trait. A law is a property
of every correct implementation, so a law test cannot be satisfied by restating what the code does.
The identity law SHALL be tested at the identity applicative, not by the weaker phrasing the trait's
docstring records as vacuous.

#### Scenario: Naturality holds across an applicative morphism

- **WHEN** an applicative morphism from `OptionWitness` to `ResultWitness` is applied after `sequence` and, separately, to each element before `sequence`
- **THEN** the two results are equal

#### Scenario: Identity holds at the identity applicative

- **WHEN** a container is sequenced at an applicative whose `Type<T>` is `T`
- **THEN** the result is the original container unchanged

#### Scenario: Composition holds

- **WHEN** a container is sequenced through a composite of two applicatives, and separately by composing the two sequenced results
- **THEN** the two results are equal

#### Scenario: Every law is exercised at more than one element

- **WHEN** the law suite's inputs are reviewed
- **THEN** each law is exercised by at least one container holding two or more distinct elements, so that a permutation or dropped-element defect cannot pass

### Requirement: A shaped inner applicative is admissible and reads cartesian
`sequence` SHALL accept a shaped container witness as the inner applicative `M`, and the result SHALL follow that witness's own `apply` semantics rather than a special case.

`DenseVectorWitness` and `CausalTensorWitness` carry the cartesian applicative, pinned by their
`Monad`. Sequencing over one is the strongest available check that the fold delegates to `M`
instead of assuming a failure-shaped carrier.

#### Scenario: Sequencing at a cartesian inner applicative enumerates the product

- **WHEN** a two-element container whose elements are two-element `DenseVector`s is sequenced over `DenseVectorWitness`
- **THEN** the result carries the four cartesian combinations in function-major order

### Requirement: The sequential `sequence` is machine-checked in Lean
The identity and naturality laws SHALL be proved in Lean for the accumulator fold the three impls share, in a file separate from the existing `Haft/Traversable.lean`, and SHALL be registered in `lean/THEOREM_MAP.md`.

The existing proofs cover `OptionWitness::sequence`, whose carrier holds at most one element and
whose laws discharge by case analysis. They do not imply the multi-element case: the fold's laws
are inductions over the element list with the accumulator generalised, because the accumulator is
not invariant across a step. The new file SHALL model the fold over an abstract applicative given
as an operations record, so the theorems quantify over every applicative the Rust code could see,
and SHALL NOT edit `Haft/Traversable.lean`.

The morphism record for the sequential case SHALL require preservation of `apply` in addition to
`pure` and `fmap`, because the fold calls `apply`. This is part of the standard definition of an
applicative morphism and is not a weakening of the naturality statement.

The Lean composition law remains out of scope, as it already is for `Option`.

#### Scenario: The proofs typecheck under the workspace gate

- **WHEN** `bazel test //lean:Haft` runs
- **THEN** it passes with the new file included, and the file introduces no Mathlib import

#### Scenario: The theorems are discoverable from the map

- **WHEN** `lean/THEOREM_MAP.md` is read
- **THEN** it carries a row per new theorem, each naming its Lean file and its Rust witness

#### Scenario: The element count is proved preserved

- **WHEN** the sequential `sequence` is applied to a list of any length
- **THEN** a machine-checked theorem states the result carries the same number of elements, which is what licenses the tensor re-wearing the input's shape

### Requirement: The admissible inner-witness population is unchanged
This change SHALL NOT reduce the set of witnesses admissible as `sequence`'s inner applicative, and the four effect witnesses, `BoxWitness`, `LinkedListWitness` and `VecWitness` SHALL remain admissible.

The measured 19→3 collapse recorded against the `Semigroupal` route is the cost of moving the
inner bound. That move is not made here.

#### Scenario: The effect monads remain admissible

- **WHEN** the set of witnesses satisfying `sequence`'s inner bound is enumerated after this change
- **THEN** it still contains all four effect witnesses, `BoxWitness`, `LinkedListWitness` and `VecWitness`

#### Scenario: A previously named lost carrier still works

- **WHEN** a container is sequenced with `BoxWitness` as the inner applicative
- **THEN** the call compiles and returns the boxed container
