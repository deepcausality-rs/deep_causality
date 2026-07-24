# haft-clone-functor Specification

## Purpose
TBD - created by archiving change add-haft-clone-functor. Update Purpose after archive.
## Requirements
### Requirement: `CloneFunctor` capability trait

`deep_causality_haft` SHALL provide a witness-capability trait `CloneFunctor` over
`HKT<Constraint = NoConstraint>` with `fn clone_type<T: Clone>(fa: &Self::Type<T>) -> Self::Type<T>`.
A witness that opts in supplies the structural clone of its `Type<T>` when `T: Clone` — the same
"witness supplies the operation" shape as `EqFunctor::eq_type` and `DebugFunctor::fmt_type`. The trait
SHALL be exported from `lib.rs` and SHALL be `core`/`alloc`-free so it compiles in every feature
configuration.

#### Scenario: a witness supplies the clone of its Type

- **WHEN** `CloneFunctor::clone_type` is called on a witness `W`'s `<W as HKT>::Type<T>` with
  `T: Clone`
- **THEN** it returns a structural clone of the container, equal to the input

### Requirement: Opt-in generic `Clone` for `Free` and `Cofree`

`deep_causality_haft` SHALL provide generic `Clone` instances, defined in the crate that owns
`Free`/`Cofree` (required by the orphan rule):

- `impl<F: CloneFunctor, A: Clone> Clone for Free<F, A>`.
- `impl<F: CloneFunctor, A: Clone> Clone for Cofree<F, A>`.

The recursion in these impls SHALL be driven through `F::clone_type`, never through an
`F::Type<..>: Clone` projection bound, so that the instances terminate at every concrete witness (the
projection-bound path overflows the trait solver with `E0275`, the failure `EqFunctor`/`DebugFunctor`
route around). These instances SHALL be strictly additive: `Free`/`Cofree` definitions and their
existing `PartialEq`/`Eq`/`Debug` and `fold`-based stories are unchanged, and the instances appear
only for witnesses that implement `CloneFunctor`.

#### Scenario: cloning a recursive Free/Cofree tree terminates and reproduces it

- **WHEN** `x.clone()` is evaluated for a `Free<W, A>` (or `Cofree<W, A>`) over a witness
  `W: CloneFunctor` with `A: Clone`
- **THEN** it compiles (no `E0275` overflow) and returns a value structurally equal to `x`

#### Scenario: Clone is opt-in per witness

- **WHEN** a `Free<W, A>` (or `Cofree<W, A>`) is cloned over a witness `W` that does not implement
  `CloneFunctor`
- **THEN** the program does not compile (there is no `Clone` instance)

### Requirement: Capability impls for the built-in functor witnesses

`deep_causality_haft` SHALL implement `CloneFunctor` for its built-in single-hole functor witnesses
that can carry `Free`/`Cofree` — `OptionWitness`, `VecWitness`, `BoxWitness`, `LinkedListWitness`, and
`VecDequeWitness`. Each body delegates to the container's own `Clone`. Each `Free`/`Cofree` built over
such a witness SHALL therefore obtain `Clone` for `Clone` payloads without further work by a consumer.

#### Scenario: Free/Cofree over a built-in witness is cloneable

- **WHEN** a `Free<OptionWitness, i32>` (or `Cofree<VecWitness, i32>`) is cloned
- **THEN** the operation succeeds using the built-in witness's `CloneFunctor` impl

### Requirement: `Clone` instances are tested and carry no new categorical law

The generated `Clone` on `Free`/`Cofree` SHALL reproduce the tree (a clone is structurally equal to
its input whenever the witness's `clone_type` is), exercised by Rust tests (Bazel-registered). `Clone`
is a derived instance, not a categorical law, and SHALL NOT add a new Lean theorem or a new
`formalization.yml` crate-allowlist entry.

#### Scenario: clone reproduces the tree on sample carriers

- **WHEN** the tests run over sample `Free`/`Cofree` trees
- **THEN** `x.clone() == x` for each, and `clone_type` agrees with the underlying container's `Clone`

#### Scenario: the free_monad NOTE is reconciled

- **WHEN** `free_monad.rs`'s NOTE about absent derived instances is read
- **THEN** it lists `Clone` alongside `PartialEq`/`Eq`/`Debug` on the opt-in
  `EqFunctor`/`DebugFunctor`/`CloneFunctor` route and no longer states that `Clone` is absent

