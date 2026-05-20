# iso-traits-haft Specification

## Purpose
TBD - created by archiving change 2026-05-20-add-iso-traits. Update Purpose after archive.
## Requirements
### Requirement: `NaturalIso<F, G>` provides HKT-witness natural isomorphism

The crate `deep_causality_haft` SHALL expose a trait `NaturalIso<F, G>` in a new module `src/iso/`. The trait SHALL declare two methods: `fn to_target<T>(fa: F::Type<T>) -> G::Type<T>;` and `fn to_source<T>(ga: G::Type<T>) -> F::Type<T>;`. The trait SHALL bound `F: HKT, G: HKT` in its `where`-clause. Each method SHALL carry the bound `T: Satisfies<F::Constraint> + Satisfies<G::Constraint>` (the existing HKT machinery in `deep_causality_haft` uses `Satisfies` rather than bare generics). Method names SHALL be `to_target` and `to_source` for consistency with the Tier 2 `Iso<S, T>` trait in `deep_causality_num`.

#### Scenario: NaturalIso declares to_target and to_source over HKT witnesses

- **WHEN** a downstream user inspects `deep_causality_haft::iso::NaturalIso<F, G>`
- **THEN** the trait SHALL declare `fn to_target<T>(fa: F::Type<T>) -> G::Type<T>` and `fn to_source<T>(ga: G::Type<T>) -> F::Type<T>`
- **AND** both methods carry `where T: Satisfies<F::Constraint> + Satisfies<G::Constraint>`
- **AND** the trait does NOT take `&self` (both methods are associated functions on the implementer)

#### Scenario: NaturalIso can be implemented on an existing HKT witness

- **WHEN** a downstream crate writes `impl NaturalIso<F, G> for F` where `F` is an existing HKT witness type
- **THEN** the impl type-checks
- **AND** the orphan rule is the only structural constraint on which `Self` is permitted in a given crate
- **AND** a dedicated separate witness type is NOT required for the single-convention case

### Requirement: Arity-N variants `NaturalIso2` through `NaturalIso5` cover unbound multi-parameter HKT witnesses

The crate `deep_causality_haft` SHALL expose four additional natural-iso traits: `NaturalIso2<F, G>`, `NaturalIso3<F, G>`, `NaturalIso4<F, G>`, and `NaturalIso5<F, G>`. Each trait SHALL bound `F` and `G` against the matching unbound HKT trait (`HKT2Unbound`, `HKT3Unbound`, `HKT4Unbound`, `HKT5Unbound` respectively), not the fixed-parameter `HKT2..HKT5` family. Each trait SHALL declare `to_target` and `to_source` methods generic over the matching number of free type parameters. Every free parameter SHALL carry the bound `Satisfies<F::Constraint> + Satisfies<G::Constraint>`. `NaturalIso5` is intended for use against the propagating-effect carrier `<V, S, C, E, L>` in `deep_causality_core`.

#### Scenario: NaturalIso5 surface declares the 5-arity natural-iso methods over HKT5Unbound

- **WHEN** a downstream user inspects `deep_causality_haft::iso::NaturalIso5<F, G>`
- **THEN** the trait bounds SHALL be `F: HKT5Unbound, G: HKT5Unbound`
- **AND** the trait SHALL declare `fn to_target<V, S, C, E, L>(fa: F::Type<V, S, C, E, L>) -> G::Type<V, S, C, E, L>` and the symmetric `to_source`
- **AND** every free parameter `V, S, C, E, L` carries `Satisfies<F::Constraint> + Satisfies<G::Constraint>`

#### Scenario: NaturalIso2 / NaturalIso3 / NaturalIso4 mirror the 5-arity shape

- **WHEN** a downstream user inspects `NaturalIso2`, `NaturalIso3`, or `NaturalIso4`
- **THEN** the trait bounds SHALL use `HKT2Unbound`, `HKT3Unbound`, `HKT4Unbound` respectively
- **AND** the method signatures SHALL declare the matching number of free type parameters (`<A, B>`, `<A, B, C>`, `<A, B, C, D>`)
- **AND** every free parameter carries the `Satisfies` bound on both witnesses' constraints

### Requirement: NaturalIso round-trip and naturality laws are documented and property-tested

The doc comment for `NaturalIso<F, G>` SHALL document two laws:

- **Round-trip:** `to_source(to_target(fa)) == fa` and the symmetric case, for every `T` and every `fa: F::Type<T>`.
- **Naturality:** for any function `h: T -> U`, `to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)`.

Property-test helpers SHALL live in `deep_causality_haft/src/iso/test_support.rs` and SHALL be exposed as `pub mod` (reachable from integration tests under `deep_causality_haft/tests/`), matching the convention used by the Tier 1 / Tier 2 helpers in `deep_causality_num`.

#### Scenario: Round-trip helper takes independent F and G inputs

- **WHEN** a reviewer inspects `deep_causality_haft/src/iso/test_support.rs`
- **THEN** the module SHALL export `assert_natural_iso_round_trip<W, F, G, T>(fa: F::Type<T>, ga: G::Type<T>)`
- **AND** the helper asserts `to_source(to_target(fa)) == fa` and `to_target(to_source(ga)) == ga`
- **AND** the `(fa, ga)` pair is independent (not derived from each other), matching the Tier 1 / Tier 2 round-trip discipline

#### Scenario: Naturality helper exists for NaturalIso

- **WHEN** a reviewer inspects `deep_causality_haft/src/iso/test_support.rs`
- **THEN** the module SHALL export `assert_natural_iso_naturality<W, F, G, A, B, Func>(fa: F::Type<A>, h: Func)`
- **AND** the helper asserts `to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)` against the caller-supplied `h`
- **AND** no fixed function bank is shipped; callers supply their own representative functions inline

### Requirement: NaturalIso trait surface compiles under `no_std`

The traits `NaturalIso<F, G>`, `NaturalIso2<F, G>`, `NaturalIso3<F, G>`, `NaturalIso4<F, G>`, and `NaturalIso5<F, G>` SHALL compile under `cargo build -p deep_causality_haft --no-default-features --features alloc`. The trait declarations SHALL use `core::*` paths where applicable and SHALL NOT pull in `std`-only dependencies. The `test_support` module SHALL remain `pub mod` and use `core::fmt::Debug` so that the trait declarations and helpers all compile under `no_std`.

#### Scenario: NaturalIso trait declarations build under no_std

- **WHEN** the project is built with `cargo build -p deep_causality_haft --no-default-features --features alloc`
- **THEN** the build SHALL succeed
- **AND** the trait declarations SHALL be available for use in downstream `no_std` crates

### Requirement: Public re-exports surface every `NaturalIso*` trait at the crate root

The `deep_causality_haft` crate SHALL re-export `NaturalIso`, `NaturalIso2`, `NaturalIso3`, `NaturalIso4`, and `NaturalIso5` from its `src/lib.rs` so that downstream consumers can write `use deep_causality_haft::{NaturalIso, NaturalIso2, NaturalIso3, NaturalIso4, NaturalIso5};` without referencing the inner module path.

#### Scenario: Every arity is importable at the crate root

- **WHEN** a downstream user writes `use deep_causality_haft::{NaturalIso, NaturalIso2, NaturalIso3, NaturalIso4, NaturalIso5};`
- **THEN** every import resolves to the trait declared in its respective `src/iso/natural_iso{,_2,_3,_4,_5}.rs` file

