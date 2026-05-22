# iso-traits-num Specification

## Purpose
TBD - created by archiving change 2026-05-20-add-iso-traits. Update Purpose after archive.
## Requirements
### Requirement: Tier 1 marker subtraits expose structure-preserving isomorphism over `From`/`Into`

The crate `deep_causality_num` SHALL expose a family of marker subtraits in a new module `src/iso/`: `GroupIso<T>`, `RingIso<T>`, `FieldIso<T>`, `AlgebraIso<T, R>`, and `DivisionAlgebraIso<T, R>`. Each subtrait SHALL have an empty body. The subtraits SHALL bound on bidirectional `From` in their `where`-clauses (i.e. both `Self: From<T>` and `T: From<Self>`) together with the corresponding algebraic-structure trait on both `Self` and `T`. Implementing a subtrait is a marker promise that the bidirectional `From` impls form the corresponding structure-preserving homomorphism. The promise is verified by property tests; the type system does not enforce it.

#### Scenario: GroupIso bounds on bidirectional From plus Group

- **WHEN** a downstream user inspects `deep_causality_num::iso::GroupIso<T>`
- **THEN** its `where` clause SHALL be `Self: Group + From<T>, T: Group + From<Self>`
- **AND** the trait body is empty (no methods)
- **AND** the doc comment SHALL describe the group-homomorphism law `T::from(a · b) == T::from(a) · T::from(b)` for all `a, b: Self`

#### Scenario: RingIso extends GroupIso

- **WHEN** a downstream user inspects `deep_causality_num::iso::RingIso<T>`
- **THEN** it SHALL extend `GroupIso<T>` and add `where Self: Ring, T: Ring` bounds
- **AND** the trait body is empty
- **AND** implementing `RingIso<T>` automatically satisfies `GroupIso<T>` via Rust's trait-inheritance mechanism

#### Scenario: FieldIso extends RingIso

- **WHEN** a downstream user inspects `deep_causality_num::iso::FieldIso<T>`
- **THEN** it SHALL extend `RingIso<T>` and add `where Self: Field, T: Field` bounds
- **AND** the trait body is empty

#### Scenario: AlgebraIso introduces the scalar ring parameter

- **WHEN** a downstream user inspects `deep_causality_num::iso::AlgebraIso<T, R>`
- **THEN** its `where` clause SHALL be `Self: Algebra<R> + From<T>, T: Algebra<R> + From<Self>, R: Ring`
- **AND** the trait body is empty
- **AND** the second type parameter `R` represents the scalar ring (per D5)

#### Scenario: DivisionAlgebraIso requires a Field scalar

- **WHEN** a downstream user inspects `deep_causality_num::iso::DivisionAlgebraIso<T, R>`
- **THEN** it SHALL extend `AlgebraIso<T, R>` and add `where Self: DivisionAlgebra<R>, T: DivisionAlgebra<R>, R: Field` bounds
- **AND** the trait body is empty

### Requirement: Tier 2 `Iso<S, T>` provides witness-typed isomorphism with `to_target` / `to_source` methods

The crate `deep_causality_num` SHALL expose a trait `Iso<S, T>` in a new module `src/iso/witness/` with exactly two methods: `fn to_target(s: S) -> T;` and `fn to_source(t: T) -> S;`. Method names SHALL be `to_target` and `to_source` (not `forward`/`backward`, not `from`/`into`) per the naming-collision avoidance documented in D3. The implementer type (`Self`) MAY be the source `S`, the target `T`, or a dedicated witness type; the trait does not constrain `Self`.

#### Scenario: Iso surface declares to_target and to_source

- **WHEN** a downstream user inspects `deep_causality_num::iso::witness::Iso<S, T>`
- **THEN** the trait declares exactly two methods: `fn to_target(s: S) -> T;` and `fn to_source(t: T) -> S;`
- **AND** neither method takes `&self` or `self` (both are associated functions, not instance methods)
- **AND** the trait has no `where`-clause on `Self`

#### Scenario: Iso can be implemented on the source, target, or a separate type

- **WHEN** a downstream crate implements `Iso<A, B>` for `A` (source as implementer), or for `B` (target as implementer), or for `MyIso` (dedicated witness)
- **THEN** all three impl forms type-check
- **AND** the orphan rule is the only structural constraint on which `Self` is permitted in a given crate

### Requirement: Tier 2 marker subtraits parallel the Tier 1 hierarchy with type-pair constraints

The crate `deep_causality_num` SHALL expose marker subtraits in `src/iso/witness/`: `GroupIso<S, T>`, `RingIso<S, T>`, `FieldIso<S, T>`, `AlgebraIso<S, T, R>`, `DivisionAlgebraIso<S, T, R>`. Each subtrait SHALL extend `Iso<S, T>` (directly or transitively) and SHALL have empty body. The `where`-clauses SHALL constrain the type *pair* `(S, T)` rather than the implementer `Self`. Implementing the marker is a promise that the iso preserves the relevant algebraic structure on the type pair.

#### Scenario: Witness GroupIso constrains the type pair, not Self

- **WHEN** a downstream user inspects `deep_causality_num::iso::witness::GroupIso<S, T>`
- **THEN** the trait declaration SHALL be `pub trait GroupIso<S, T>: Iso<S, T> where S: Group, T: Group {}`
- **AND** there is NO bound `Self: Group`
- **AND** the implementer can be any type — the constraint applies to the type *pair*

#### Scenario: Witness FieldIso refuses to instantiate when S or T is not a Field

- **WHEN** a downstream user writes `impl FieldIso<MyType, Quaternion<f64>> for MyImpl` where `Quaternion<f64>` is not a `Field` (because quaternions are non-commutative)
- **THEN** the compiler SHALL reject the impl
- **AND** the rejection cites the unsatisfied `T: Field` bound

#### Scenario: Witness AlgebraIso introduces the scalar ring parameter

- **WHEN** a downstream user inspects `deep_causality_num::iso::witness::AlgebraIso<S, T, R>`
- **THEN** the trait declaration SHALL be `pub trait AlgebraIso<S, T, R>: Iso<S, T> where S: Algebra<R>, T: Algebra<R>, R: Ring {}`
- **AND** the trait body is empty

### Requirement: `StandardIso<S, T>` generic witness with blanket impls auto-derives marker subtraits

The crate `deep_causality_num` SHALL expose a generic zero-sized witness type `StandardIso<S, T>` in `src/iso/witness/standard.rs`. The type SHALL be a named-field struct with two separate `PhantomData<fn() -> X>` fields (one for `S`, one for `T`) rather than a single `PhantomData<(S, T)>` tuple. The split-field form avoids `clippy::type_complexity` on the tuple shape and avoids spurious auto-trait bounds on `S` and `T`. `StandardIso<S, T>` SHALL implement `Iso<S, T>` whenever bidirectional `From` is available. `StandardIso<S, T>` SHALL additionally implement every applicable Tier 2 marker subtrait (`GroupIso<S, T>`, `RingIso<S, T>`, `FieldIso<S, T>`, `AlgebraIso<S, T, R>`, `DivisionAlgebraIso<S, T, R>`) via blanket impls that fire when `S` and `T` satisfy the corresponding algebraic-structure bounds plus bidirectional `From`. No manual marker impls are required for the common case.

#### Scenario: StandardIso implements Iso when bidirectional From exists

- **WHEN** types `A` and `B` satisfy `A: From<B>` and `B: From<A>`
- **THEN** `StandardIso<A, B>: Iso<A, B>` SHALL be satisfied by the blanket impl
- **AND** `StandardIso::<A, B>::to_target(a)` evaluates to `B::from(a)` for any `a: A`
- **AND** `StandardIso::<A, B>::to_source(b)` evaluates to `A::from(b)` for any `b: B`

#### Scenario: StandardIso auto-derives GroupIso when both sides are Groups

- **WHEN** types `A` and `B` satisfy `A: Group + From<B>` and `B: Group + From<A>`
- **THEN** `StandardIso<A, B>: GroupIso<A, B>` SHALL be satisfied by the blanket impl
- **AND** the derivation requires no manual `impl GroupIso<A, B> for StandardIso<A, B>` block
- **AND** generic code that bounds on `StandardIso<A, B>: GroupIso<A, B>` accepts the type pair

#### Scenario: StandardIso auto-derives the full algebraic hierarchy

- **WHEN** types `A` and `B` satisfy `A: DivisionAlgebra<F> + From<B>` and `B: DivisionAlgebra<F> + From<A>` for some `F: Field`
- **THEN** `StandardIso<A, B>: DivisionAlgebraIso<A, B, F>` SHALL be satisfied by the blanket impl
- **AND** the full inheritance chain (`Iso` → `AlgebraIso` → `DivisionAlgebraIso`) is satisfied automatically

#### Scenario: StandardIso does NOT auto-derive markers when an algebraic-structure bound fails

- **WHEN** types `A` and `B` satisfy bidirectional `From` but `A` is not a `Field` (e.g. `A` is `Quaternion<f64>`, non-commutative)
- **THEN** `StandardIso<A, B>: FieldIso<A, B>` SHALL NOT be satisfied
- **AND** code that bounds on this marker SHALL fail to compile, citing the unsatisfied `A: Field` bound

#### Scenario: Named witnesses can coexist with StandardIso

- **WHEN** a downstream crate defines a separate witness type `MyIso` and writes `impl Iso<A, B> for MyIso`
- **THEN** both `MyIso` and `StandardIso<A, B>` implement `Iso<A, B>` without compile-time ambiguity
- **AND** Rust's coherence rules accept the two impls because they are on distinct `Self` types

### Requirement: Tier 1 and Tier 2 marker subtraits live in separate module paths

The Tier 1 and Tier 2 marker subtraits share short names (`GroupIso`, `RingIso`, etc.). To avoid name collision, Tier 1 SHALL live at `deep_causality_num::iso::*` and Tier 2 SHALL live at `deep_causality_num::iso::witness::*`. The top-level `deep_causality_num::iso` module SHALL NOT re-export Tier 2 subtraits at its root; consumers disambiguate by importing from the appropriate module path.

#### Scenario: Tier 1 and Tier 2 GroupIso are distinct traits

- **WHEN** a downstream user imports `use deep_causality_num::iso::GroupIso;`
- **THEN** the imported trait has the Tier 1 signature `pub trait GroupIso<T> where ... {}`
- **AND** importing `use deep_causality_num::iso::witness::GroupIso;` resolves to the Tier 2 signature `pub trait GroupIso<S, T>: Iso<S, T> where ... {}`
- **AND** both imports in the same scope are a name conflict, requiring `as` aliases or fully-qualified paths

### Requirement: Iso trait surface compiles under `no_std`

The Tier 1 marker subtraits and the Tier 2 `Iso<S, T>`, marker subtraits, and `StandardIso<S, T>` SHALL compile under `cargo build -p deep_causality_num --no-default-features --features libm_math`. The trait declarations and `StandardIso<S, T>` SHALL use `core::marker::PhantomData` (not `std::marker::PhantomData`) where applicable. Property-test helpers in `test_support.rs` modules SHALL be exposed as `pub mod` (NOT `#[cfg(test)]`-gated) because Bazel's per-crate build cannot reach `tests/` from `src/`. The helpers themselves use only `core::fmt::Debug` and `assert_eq!` and remain `no_std`-compatible.

#### Scenario: Tier 1 trait declarations build under no_std

- **WHEN** the project is built with `cargo build -p deep_causality_num --no-default-features --features libm_math`
- **THEN** the build SHALL succeed
- **AND** the Tier 1 marker subtraits in `src/iso/` SHALL be available for use in downstream `no_std` crates

#### Scenario: StandardIso<S, T> uses core::marker::PhantomData

- **WHEN** a reviewer inspects `deep_causality_num/src/iso/witness/standard.rs`
- **THEN** the import for `PhantomData` SHALL be `use core::marker::PhantomData;` (not `use std::marker::PhantomData;`)
- **AND** `StandardIso<S, T>` SHALL build under `--no-default-features`

### Requirement: Property-test infrastructure exists for every iso marker

Property-test helpers SHALL exist in `deep_causality_num/src/iso/test_support.rs` (Tier 1) and `deep_causality_num/src/iso/witness/test_support.rs` (Tier 2). The helpers SHALL be exposed as `pub mod` (reachable from integration tests under `deep_causality_num/tests/`). The helpers SHALL be generic over the impl under test. Each marker subtrait SHALL have a corresponding `assert_*_law` helper that exercises the appropriate round-trip or homomorphism property using `assert_eq!` with descriptive failure messages. Helpers exercise only their own marker's contribution to the homomorphism chain; consumers verifying a deeper marker (e.g. `FieldIso`) compose with the parent helpers.

#### Scenario: Tier 1 round-trip helper takes independent S and T inputs

- **WHEN** a reviewer inspects `deep_causality_num/src/iso/test_support.rs`
- **THEN** the module SHALL export a function `assert_iso_from_round_trip<S, T>(s: S, t: T)` where `S: From<T> + Clone + PartialEq + core::fmt::Debug` and `T: From<S> + Clone + PartialEq + core::fmt::Debug`
- **AND** the function asserts `S::from(T::from(s.clone())) == s` and `T::from(S::from(t.clone())) == t`
- **AND** the `(s, t)` pair is independent (not derived from each other) so that non-bijective conversions where one direction is many-to-one cannot pass undetected
- **AND** the failure message identifies which round-trip direction failed

#### Scenario: Tier 2 witness round-trip helper takes independent S and T inputs

- **WHEN** a reviewer inspects `deep_causality_num/src/iso/witness/test_support.rs`
- **THEN** the module SHALL export a function `assert_witness_iso_round_trip<W, S, T>(s: S, t: T)` where `W: Iso<S, T>`, `S: Clone + PartialEq + core::fmt::Debug`, `T: Clone + PartialEq + core::fmt::Debug`
- **AND** the function asserts `W::to_source(W::to_target(s.clone())) == s` and `W::to_target(W::to_source(t.clone())) == t`
- **AND** the same independent-inputs rationale as the Tier 1 helper applies

#### Scenario: Homomorphism helpers exist for each algebraic level

- **WHEN** a reviewer inspects the test_support modules
- **THEN** each algebraic level (Group, Ring, Field, Algebra, DivisionAlgebra) SHALL have a corresponding `assert_*_law` helper for both Tier 1 and Tier 2
- **AND** each helper asserts ONLY the homomorphism contribution of the marker it names (e.g. `assert_field_iso_from_laws` checks multiplicative-inverse preservation only; ring-level laws are inherited and must be exercised via the ring helper)
- **AND** failure messages describe the specific law that failed

