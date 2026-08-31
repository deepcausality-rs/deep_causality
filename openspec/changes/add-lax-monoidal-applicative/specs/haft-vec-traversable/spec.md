## ADDED Requirements

### Requirement: `VecWitness` implements `Traversable` through the inner witness's `zip_with`
`deep_causality_haft` SHALL implement `Traversable<VecWitness>` for `VecWitness`, with `sequence` folding an accumulator through the inner witness using `zip_with` rather than `apply`. The fold SHALL be `acc = M::zip_with(acc, m_a, |mut v, a| { v.push(a); v })` seeded with `M::pure(Vec::new())`. The `apply`-based fold SHALL NOT be used, because it places an anonymous closure inside `M` and so requires that closure type to satisfy `M::Constraint`, which `sequence` cannot declare and an impl may not add.

#### Scenario: A vector of options flips

- **WHEN** `sequence::<i32, OptionWitness>` is applied to `vec![Some(1), Some(2), Some(3)]`
- **THEN** the result is `Some(vec![1, 2, 3])`

#### Scenario: Failure propagates and the empty case succeeds

- **WHEN** `sequence` is applied to a vector containing `None`, and separately to an empty vector
- **THEN** the results are `None` and `Some(vec![])` respectively

#### Scenario: The result carrier is not restricted to `Option`

- **WHEN** `sequence::<i32, ResultWitness<String>>` is applied to `vec![Ok(1), Ok(2)]`
- **THEN** the result is `Ok(vec![1, 2])`

### Requirement: The inner witness needs only the semigroupal structure
`Traversable` for `VecWitness` SHALL bound its inner witness on `Semigroupal` and `Pure`, not on `MonoidalApplicative`. `VecWitness` itself SHALL NOT be required to implement `Semigroupal`, `Convolutional` or `MonoidalApplicative` in order to be `Traversable`, since the structure is demanded of the inner carrier alone.

#### Scenario: `Vec` traverses without adopting the monoidal stack

- **WHEN** `VecWitness::sequence` is used
- **THEN** it compiles while `VecWitness` still carries only `Applicative`, keeping its cartesian semantics untouched

### Requirement: The `Traversable` documentation stops referring to code that does not exist
The `Traversable::sequence` doctest SHALL compile and run rather than being fenced `rust,ignore`, and SHALL only name witnesses that implement the trait. The note at the foot of `hkt_vec_ext.rs` recording that `VecWitness` lacks `Traversable` SHALL be removed once the impl lands.

#### Scenario: Every doctest on the trait executes

- **WHEN** the crate's doctests are run
- **THEN** no `Traversable` example is skipped, and each names a witness that implements the trait
