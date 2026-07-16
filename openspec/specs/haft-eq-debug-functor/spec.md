# haft-eq-debug-functor Specification

## Purpose
TBD - created by archiving change haft-cofree-and-eq-debug. Update Purpose after archive.
## Requirements
### Requirement: `EqFunctor` and `DebugFunctor` capability traits

`deep_causality_haft` SHALL provide two witness-capability traits over
`HKT<Constraint = NoConstraint>`:

- `EqFunctor` with `fn eq_type<T: PartialEq>(a: &Self::Type<T>, b: &Self::Type<T>) -> bool`.
- `DebugFunctor` with `fn fmt_type<T: core::fmt::Debug>(fa: &Self::Type<T>, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result`.

A witness that opts in supplies structural comparison / `Debug` formatting of its `Type<T>` when
`T: PartialEq` / `T: Debug` — the same "witness supplies the operation" shape as `Functor::fmap` and
`Foldable::fold`. Both traits SHALL be exported from `lib.rs` and SHALL be no-std clean.

#### Scenario: a witness supplies comparison and formatting for its Type

- **WHEN** `EqFunctor::eq_type` / `DebugFunctor::fmt_type` are called on a witness `W`'s
  `<W as HKT>::Type<T>` with `T: PartialEq` / `T: Debug`
- **THEN** the call returns the structural equality of the two containers / writes their `Debug`
  representation

### Requirement: Opt-in generic `PartialEq`/`Eq`/`Debug` for `Free` and `Cofree`

`deep_causality_haft` SHALL provide generic instances, defined in the crate that owns `Free`/`Cofree`
(required by the orphan rule):

- `impl<F: EqFunctor, A: PartialEq> PartialEq for Free<F, A>` and the same for `Cofree<F, A>`.
- `impl<F: EqFunctor, A: Eq> Eq for Free<F, A>` and the same for `Cofree<F, A>`.
- `impl<F: DebugFunctor, A: Debug> Debug for Free<F, A>` and the same for `Cofree<F, A>`.

The recursion in these impls SHALL be driven through `F::eq_type` / `F::fmt_type`, never through an
`F::Type<..>: Trait` projection bound, so that the instances terminate at every concrete witness.
These instances SHALL be strictly additive: `Free`'s definition and its existing "compare by folding
to a canonical value" approach are unchanged, and the instances appear only for witnesses that
implement the capability.

#### Scenario: structural equality on a recursive Free/Cofree tree terminates

- **WHEN** `a == b` is evaluated for two `Free<W, A>` (or `Cofree<W, A>`) values over a witness `W:
  EqFunctor` with `A: PartialEq`
- **THEN** it compiles (no `E0275` overflow) and returns structural equality of the trees

#### Scenario: the projection-bound path is not used

- **WHEN** the generic `PartialEq`/`Debug` impls are reviewed
- **THEN** their bounds are `F: EqFunctor` / `F: DebugFunctor` (plus `A: PartialEq`/`A: Debug`), not
  `F::Type<Box<..>>: PartialEq`/`Debug`

### Requirement: Capability impls for the built-in functor witnesses

`deep_causality_haft` SHALL implement `EqFunctor` and `DebugFunctor` for its built-in single-hole
`Functor` witnesses that can carry `Free`/`Cofree` — at least `OptionWitness`, `VecWitness`, and
`BoxWitness`, and the other single-hole functor witnesses in the crate (e.g. `LinkedList`/`VecDeque`).
Each `Free`/`Cofree` built over such a witness SHALL therefore obtain `PartialEq`/`Eq`/`Debug` for
`PartialEq`/`Eq`/`Debug` payloads without further work by a consumer.

#### Scenario: Free over a built-in witness is comparable and printable

- **WHEN** a `Free<OptionWitness, i32>` (or `Free<VecWitness, i32>`, `Cofree<VecWitness, i32>`) is
  compared or `{:?}`-formatted
- **THEN** the operation succeeds using the built-in witness's `EqFunctor`/`DebugFunctor` impl

### Requirement: `Eq`/`Debug` instances are lawful, tested, and carry no new categorical law

The generated `PartialEq` on `Free`/`Cofree` SHALL be a structural equivalence (reflexive,
symmetric, transitive) whenever the witness's `eq_type` is, and the `Debug` output SHALL reflect the
tree structure. These obligations SHALL be exercised by Rust property tests (Bazel-registered). The
instances are derived instances, not categorical laws, and SHALL NOT add a new Lean theorem or a new
`formalization.yml` crate-allowlist entry.

#### Scenario: equivalence-relation properties hold on sample trees

- **WHEN** the property tests run over sample `Free`/`Cofree` trees
- **THEN** `PartialEq` is reflexive, symmetric, and transitive, and equal trees produce equal
  `Debug` strings

#### Scenario: the free_monad NOTE is reconciled

- **WHEN** `free_monad.rs`'s NOTE about the absence of `PartialEq`/`Debug` is read
- **THEN** it retains the correct statement that `#[derive]` overflows on the generic type and points
  at the opt-in `EqFunctor`/`DebugFunctor` route
