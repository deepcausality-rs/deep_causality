## ADDED Requirements

### Requirement: A semigroupal structure whose primitive is `zip_with`
`deep_causality_haft` SHALL provide `Semigroupal<F: HKT>: Functor<F>` carrying the lax monoidal structure map φ, with `zip_with` as the required method and `zip` as a provided method derived from it. `zip_with` SHALL take `fa: F::Type<A>`, `fb: F::Type<B>` and `f: Func` where `Func: FnMut(A, B) -> C`, bounding `A`, `B` and `C` by `Satisfies<F::Constraint>` and nothing else. It SHALL NOT require `Clone`, `Copy` or `Default` on any payload type. `zip` SHALL return `F::Type<(A, B)>` and SHALL additionally bound `(A, B): Satisfies<F::Constraint>`, since it is the only operation that constructs the tuple.

#### Scenario: The primitive admits a move-only payload

- **WHEN** `zip_with` is called over payload types that implement neither `Clone` nor `Copy`
- **THEN** it compiles and each component of `fa` and `fb` is moved exactly once

#### Scenario: `zip` is derived and needs no separate implementation

- **WHEN** a witness implements `zip_with` only
- **THEN** `zip` is available on it without further code, and equals `zip_with(fa, fb, |a, b| (a, b))`

#### Scenario: The tuple bound does not reach `apply` or its callers

- **WHEN** a function generic over the witness calls the `apply` derived from `zip_with`
- **THEN** it compiles without declaring `(Func, A): Satisfies<F::Constraint>`, which the `zip`-based derivation would have required at both the trait and every call site

### Requirement: A lax monoidal structure that adds the unit separately
`deep_causality_haft` SHALL provide `LaxMonoidal<F: HKT>: Semigroupal<F>` carrying `unit() -> F::Type<()>`, the map η : I → F I. The unit SHALL live on this trait and not on `Semigroupal`, so that a witness with a lawful φ and no lawful η can implement the former alone. `unit` SHALL NOT be implementable by fabricating a context; a witness that would have to invent a complex, a grade, a lattice, a shape or an adjacency map in order to return a value SHALL implement `Semigroupal` only.

#### Scenario: A context-carrying witness takes the structure without the unit

- **WHEN** a witness whose carrier requires a complex, lattice or shape implements `Semigroupal`
- **THEN** it compiles and is usable without supplying `unit`, and the unit coherence laws are not stated against it

#### Scenario: A shapeless witness takes both

- **WHEN** a fixed-arity product or single-slot container implements `LaxMonoidal`
- **THEN** both `unit` and `zip_with` are available and the unit laws hold up to the unitors `((), A) ≅ A` and `(A, ()) ≅ A`

### Requirement: The new traits are named and placed apart from the cartesian PROP
The traits SHALL live in a new `deep_causality_haft/src/lax_monoidal/` module and SHALL NOT be added to `src/monoidal/`, which holds the value-level cartesian `SymMonoidal` PROP. The functor-level trait SHALL NOT be named `Monoidal`. Each module's documentation SHALL carry a doc-link to the other stating that the two structures sit at different levels, since `SymMonoidal::unit` and `LaxMonoidal::unit` are different maps reachable from one crate root.

#### Scenario: Both units are reachable and distinguishable

- **WHEN** a reader encounters `unit` in either module
- **THEN** the surrounding documentation names its level, values or endofunctors, and links to the sibling structure

#### Scenario: The existing PROP is unchanged

- **WHEN** `src/monoidal/mod.rs` is compared against its state before this change
- **THEN** `SymMonoidal` keeps every generator, law and citation it had, and the `haft-symmetric-monoidal-prop` requirements still hold

### Requirement: The `MonoidalMerge` docstring states the structure it actually carries
`MonoidalMerge`'s documentation SHALL describe the trait as a semigroupal structure carrying φ alone, and SHALL NOT describe it as a lax monoidal functor, which is the triple (F, φ, η). It SHALL record that no unit exists in the trait and none is derivable from `merge`, that unitality is therefore not statable against it, and that the trait name predates the distinction. The header of `lean/DeepCausalityFormal/Haft/MonoidalMerge.lean` SHALL carry the same correction.

#### Scenario: The claim matches the trait surface

- **WHEN** the docstring's categorical claim is checked against the trait's methods
- **THEN** every structure map it names is present in the trait, and no law is claimed that the trait cannot state
