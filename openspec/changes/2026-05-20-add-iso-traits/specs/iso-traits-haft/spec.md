## ADDED Requirements

### Requirement: `NaturalIso<F, G>` provides HKT-witness natural isomorphism

The crate `deep_causality_haft` SHALL expose a trait `NaturalIso<F, G>` in a new module `src/iso/`. The trait SHALL declare two methods: `fn to_target<T>(fa: F::Type<T>) -> G::Type<T>;` and `fn to_source<T>(ga: G::Type<T>) -> F::Type<T>;`. The trait SHALL bound `F: HKT, G: HKT` in its `where`-clause. Method names SHALL be `to_target` and `to_source` for consistency with the Tier 2 `Iso<S, T>` trait in `deep_causality_num`.

#### Scenario: NaturalIso declares to_target and to_source over HKT witnesses

- **WHEN** a downstream user inspects `deep_causality_haft::iso::NaturalIso<F, G>`
- **THEN** the trait declaration SHALL be `pub trait NaturalIso<F, G> where F: HKT, G: HKT { fn to_target<T>(fa: F::Type<T>) -> G::Type<T>; fn to_source<T>(ga: G::Type<T>) -> F::Type<T>; }`
- **AND** both methods are generic over the value type `T`
- **AND** the trait does NOT take `&self` (both methods are associated functions on the implementer)

#### Scenario: NaturalIso can be implemented on an existing HKT witness

- **WHEN** a downstream crate writes `impl NaturalIso<F, G> for F` where `F` is an existing HKT witness type
- **THEN** the impl type-checks
- **AND** the orphan rule is the only structural constraint on which `Self` is permitted in a given crate
- **AND** a dedicated separate witness type is NOT required for the single-convention case

### Requirement: `NaturalIso5<F, G>` extends natural isomorphism to the 5-arity HKT witnesses

The crate `deep_causality_haft` SHALL expose a trait `NaturalIso5<F, G>` in `src/iso/natural_iso_5.rs`. The trait SHALL bound `F` and `G` against the existing `HKT5` machinery in `deep_causality_haft` (whose exact signature is determined by the existing `HKT5` trait declaration). The trait SHALL declare two methods: `fn to_target<V, S, C, E, L>(fa: F::Type<V, S, C, E, L>) -> G::Type<V, S, C, E, L>;` and `fn to_source<V, S, C, E, L>(ga: G::Type<V, S, C, E, L>) -> F::Type<V, S, C, E, L>;`. `NaturalIso5` is the 5-arity counterpart of `NaturalIso` and is intended for use against the propagating-effect carrier `CausalEffectPropagationProcess<V, S, C, E, L>` in `deep_causality_core`.

#### Scenario: NaturalIso5 surface declares the 5-arity natural-iso methods

- **WHEN** a downstream user inspects `deep_causality_haft::iso::NaturalIso5<F, G>`
- **THEN** the trait SHALL declare `fn to_target<V, S, C, E, L>(...) -> G::Type<V, S, C, E, L>` and `fn to_source<V, S, C, E, L>(...) -> F::Type<V, S, C, E, L>`
- **AND** both methods are generic over all five type parameters of the HKT5

#### Scenario: NaturalIso5 bounds align with the existing HKT5 trait

- **WHEN** a reviewer inspects the `where`-clause of `NaturalIso5<F, G>`
- **THEN** the bounds SHALL match the existing `HKT5<...>` trait declared in `deep_causality_haft`
- **AND** the trait can be used against any HKT5 witness, including the `CausalEffectPropagationProcess` witness in `deep_causality_core`

### Requirement: NaturalIso round-trip and naturality laws are documented and property-tested

The doc comment for `NaturalIso<F, G>` SHALL document two laws:

- **Round-trip:** `<Self as NaturalIso<F, G>>::to_source(<Self as NaturalIso<F, G>>::to_target(fa)) == fa` and the symmetric case, for every `T` and every `fa: F::Type<T>`.
- **Naturality:** for any function `h: T -> U`, `<Self as NaturalIso<F, G>>::to_target(F::fmap(fa, h)) == G::fmap(<Self as NaturalIso<F, G>>::to_target(fa), h)`.

Property-test helpers in `deep_causality_haft/src/iso/test_support.rs` SHALL provide functions that exercise both laws and SHALL be `#[cfg(test)]`-gated.

#### Scenario: Round-trip helper exists for NaturalIso

- **WHEN** a reviewer inspects `deep_causality_haft/src/iso/test_support.rs`
- **THEN** the module SHALL export a function (or family of monomorphized functions) that exercises the round-trip law for a given `NaturalIso<F, G>` impl, asserting `to_source(to_target(fa)) == fa` for a representative set of `fa: F::Type<T>` values

#### Scenario: Naturality helper exists for NaturalIso

- **WHEN** a reviewer inspects `deep_causality_haft/src/iso/test_support.rs`
- **THEN** the module SHALL export a helper that exercises naturality against a caller-supplied function `h: T -> U`
- **AND** a fixed bank of test functions (negation, doubling, identity, constant, string-conversion) SHALL be available as a `#[cfg(test)]`-only helper module that test files can import

### Requirement: NaturalIso trait surface compiles under `no_std`

The traits `NaturalIso<F, G>` and `NaturalIso5<F, G>` SHALL compile under `cargo build -p deep_causality_haft --no-default-features --features alloc`. The trait declarations SHALL use `core::*` paths where applicable and SHALL NOT pull in `std`-only dependencies. Test support that requires `std` SHALL be gated behind `#[cfg(test)]`.

#### Scenario: NaturalIso trait declarations build under no_std

- **WHEN** the project is built with `cargo build -p deep_causality_haft --no-default-features --features alloc`
- **THEN** the build SHALL succeed
- **AND** the trait declarations SHALL be available for use in downstream `no_std` crates

### Requirement: Public re-exports surface `NaturalIso` and `NaturalIso5` at the crate root

The `deep_causality_haft` crate SHALL re-export `NaturalIso` and `NaturalIso5` from its `src/lib.rs` so that downstream consumers can write `use deep_causality_haft::{NaturalIso, NaturalIso5};` without referencing the inner module path.

#### Scenario: NaturalIso is importable at the crate root

- **WHEN** a downstream user writes `use deep_causality_haft::NaturalIso;`
- **THEN** the import resolves to the trait declared in `src/iso/natural_iso.rs`

#### Scenario: NaturalIso5 is importable at the crate root

- **WHEN** a downstream user writes `use deep_causality_haft::NaturalIso5;`
- **THEN** the import resolves to the trait declared in `src/iso/natural_iso_5.rs`
