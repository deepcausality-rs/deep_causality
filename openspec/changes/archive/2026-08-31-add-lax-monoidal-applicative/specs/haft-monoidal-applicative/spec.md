## ADDED Requirements

### Requirement: A diagonal-free applicative derived from the monoid structure
`deep_causality_haft` SHALL provide `MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F>` whose `apply` is a provided method defined as `Self::zip_with(ff, fa, |mut f, a| f(a))`. Its `apply` SHALL bound `A`, `B` and `Func` by `Satisfies<F::Constraint>`, with `Func: FnMut(A) -> B`, and SHALL NOT require `Clone` on any of them. A witness SHALL be able to adopt the trait with an empty impl body once it has `zip_with` and the marker.

#### Scenario: Adoption costs no method body

- **WHEN** a witness with `Semigroupal` and `Convolutional` writes `impl MonoidalApplicative<W> for W {}`
- **THEN** it compiles and `apply` is available

#### Scenario: `apply` runs over a move-only payload

- **WHEN** `apply` is called where the payload and the function type implement neither `Clone` nor `Copy`
- **THEN** it compiles and consumes each function and each argument exactly once

### Requirement: The promise gates the derived `apply`
`MonoidalApplicative` SHALL name `Convolutional` as a supertrait, so a witness carrying `Functor` and `Semigroupal` but withholding the promise cannot reach the derived `apply`. The gate SHALL be a compile error rather than a documented convention.

#### Scenario: Structure without promise is rejected

- **WHEN** a witness implements `Functor` and `Semigroupal`, omits `Convolutional`, and attempts `MonoidalApplicative`
- **THEN** the impl is rejected with an unsatisfied `Convolutional` bound

### Requirement: `Applicative` is unchanged and the two traits coexist
This change SHALL NOT alter `Applicative<F>: Functor<F> + Pure<F>`, its `apply` signature including the `A: Clone` bound, its four McBride-Paterson laws, or any of its existing impls. `Pure`, `Monad`, `Traversable`, `Arrow` and the effect system SHALL keep their current bounds. Witnesses whose applicative needs the diagonal, specifically `Vec`, `LinkedList` and `VecDeque`, SHALL keep `Applicative` and SHALL NOT be required to adopt `MonoidalApplicative`. Adoption SHALL be per-witness and incremental, with no coordinated cutover.

#### Scenario: Existing callers keep compiling

- **WHEN** `VecWitness::apply` is called with a `Func` that is `FnMut` and not `Clone`, such as a closure capturing by `&mut`
- **THEN** it compiles and behaves exactly as before, since `Applicative` is untouched

#### Scenario: Cartesian semantics are preserved

- **WHEN** `VecWitness::apply` is called with two functions and three arguments
- **THEN** it returns the six-element cartesian product, unchanged, and still agrees with the applicative its own `Monad` induces

#### Scenario: A witness holding both keeps them in step

- **WHEN** a witness implements `Applicative` and `MonoidalApplicative`
- **THEN** a law test asserts the two `apply` results are equal for the same inputs

### Requirement: Broadcast semantics are not silently replaced
A witness whose current `apply` broadcasts one function across many arguments SHALL NOT adopt `MonoidalApplicative` without an explicit decision recorded in its documentation, because `zip_with` cannot broadcast: n pairings need n owned functions, which is the diagonal. `ManifoldWitness`, `CausalTensorWitness` and `CsrMatrixWitness` broadcast today and SHALL be out of scope for adoption in this change.

#### Scenario: Two disagreeing definitions never coexist on one witness

- **WHEN** a broadcasting witness is considered for adoption
- **THEN** it is left on `Applicative` alone, so no caller can reach a derived `apply` that differs from the hand-written one
