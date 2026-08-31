# haft-monoid-markers Specification

## Purpose
The two marker traits that record which monoid object an endofunctor claims to be: `Compositional` for composition and `Convolutional` for Day convolution. Neither is ever granted by inference, so the absence of a marker reads as a deliberate withholding, and a witness carrying both owes the applicative-monad coherence law as a checkable obligation rather than a documented recommendation.
## Requirements
### Requirement: Two markers record which monoid an endofunctor claims
`deep_causality_haft` SHALL provide `Compositional<F: HKT>: Monad<F>` and `Convolutional<F: HKT>: Semigroupal<F>` as empty marker traits. `Compositional` SHALL record the promise that μ associates under composition, a monoid object in (End(𝒞), ∘, Id). `Convolutional` SHALL record the promise that φ associates under Day convolution, a monoid object in (End(𝒞), ⊛, Id). Each supertrait SHALL be the structure the promise is about, so a promise cannot be made where the structure is absent.

#### Scenario: The promise cannot outrun the structure

- **WHEN** a witness that implements neither `Monad` nor `Semigroupal` attempts the corresponding marker
- **THEN** the impl is rejected for the missing supertrait

#### Scenario: Which monoid is recoverable from the type

- **WHEN** generic code needs an endofunctor whose φ associates
- **THEN** it can bound on `Convolutional` and reject a witness that carries φ without the promise

### Requirement: Markers are never handed out by inference
Neither marker SHALL be blanket-implemented, derived, or implied by any other trait. Each impl SHALL be written out as one line naming one witness, following `deep_causality_algebra::Associative<O: Operator>`, whose documentation records that a marker recording an unverifiable promise cannot be granted by inference. The absence of a marker on a witness SHALL be readable as a deliberate withholding rather than an oversight, and the documentation SHALL say so.

#### Scenario: No downstream type acquires a promise silently

- **WHEN** a new witness implements `Semigroupal` or `Monad`
- **THEN** it does not thereby acquire `Convolutional` or `Compositional`, and must state the promise itself

#### Scenario: A withheld promise is explicable

- **WHEN** a witness carries the structure but not the marker
- **THEN** its documentation states which law it declines to promise and why

### Requirement: Holding both markers incurs the applicative-monad coherence obligation
A witness implementing both `Compositional<F>` and `Convolutional<F>` SHALL satisfy `apply(f_ab, f_a) == bind(f_ab, |f| fmap(f_a, f))`, the law already proved as `haft.monad.applicative_coherence`. That obligation SHALL be discharged by a law test naming the witness. The documentation on both markers SHALL state that the conjunction carries this obligation, replacing the standing recommendation in `Monad`'s docstring that the law be stated per witness in prose.

#### Scenario: The conjunction is a checkable law, not a comment

- **WHEN** a witness carries both markers
- **THEN** a law test exercises the coherence equation on it, and the test fails if the two induced applicatives disagree

#### Scenario: Carrying one marker incurs nothing about the other

- **WHEN** a witness carries only `Convolutional`
- **THEN** no coherence obligation against `bind` arises, since it need not be a monad at all

