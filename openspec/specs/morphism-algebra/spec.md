# morphism-algebra Specification

## Purpose
TBD - created by archiving change causal-arrow-foundations. Update Purpose after archive.
## Requirements
### Requirement: Typed-arrow base with identity and application

`deep_causality_haft` SHALL provide a `Morphism` trait parameterized by a two-argument HKT witness (`P: HKT2Unbound`), giving a family of arrows `P::Type<A, B>` an identity arrow and the ability to apply an arrow to an input. The trait SHALL expose `identity<A>() -> P::Type<A, A>` and `apply<A, B>(arrow: &P::Type<A, B>, input: A) -> B`, with type parameters constrained by the witness's `Satisfies<P::Constraint>` bound, consistent with the crate's existing witness-based traits (`Profunctor`, `Bifunctor`, `Promonad`). The crate SHALL ship at least one concrete witness that implements `Morphism` under static dispatch with no `dyn`/trait objects (a function-pointer carrier whose `Type<A, B>` is `fn(A) -> B`).

General arrow composition (`P::Type<A,B>` with `P::Type<B,C>` into `P::Type<A,C>`) over capturing closures SHALL NOT be a method of this trait in this capability, because no single concrete carrier exists for it under the no-`dyn` policy; composition is deferred to a later capability whose carrier is a free/defunctionalized category.

#### Scenario: Identity arrow returns its input unchanged

- **WHEN** the identity arrow is applied to a value `x`
- **THEN** `apply(identity(), x)` returns a value equal to `x`

#### Scenario: An arrow applied to an input runs it

- **WHEN** a function-pointer arrow `f` and an input `x` are given
- **THEN** `apply(&f, x)` returns the same result as calling `f(x)` directly

### Requirement: Type-preserving endomorphism fragment with bounded iteration and fixpoint combinators

`deep_causality_haft` SHALL provide an `Endomorphism` trait, supertraited on `Morphism` (`Endomorphism<P>: Morphism<P>`), marking the type-preserving fragment (arrows `P::Type<T, T>`). It SHALL be implementable for any `Morphism` witness via a blanket implementation. The trait SHALL host iteration combinators that repeatedly apply a `T → T` arrow:

- `iterate_n(arrow, x, n)` SHALL apply the arrow exactly `n` times.
- `iterate_to_fixpoint(arrow, x, max_steps)` SHALL apply the arrow until a fixpoint is reached (the next iterate equals the current value) or `max_steps` applications have occurred, requiring `T: Clone + PartialEq`.
- `iterate_until(arrow, x, predicate, max_steps)` SHALL apply the arrow until `predicate(&x)` holds or `max_steps` applications have occurred.

Every combinator that can fail to converge SHALL be bounded by an explicit `max_steps` and SHALL report whether convergence (a fixpoint, or the predicate holding) was actually reached, rather than looping without bound. The combinators SHALL be expressed by repeated application via `Morphism::apply` and SHALL use static dispatch with no `dyn`/trait objects.

#### Scenario: Iterating exactly n times

- **WHEN** `iterate_n(arrow, x, n)` is called with the increment arrow `t ↦ t + 1` and `x = 0`
- **THEN** the result equals `n`

#### Scenario: Reaching a fixpoint reports convergence

- **WHEN** `iterate_to_fixpoint(arrow, x, max_steps)` is called with an arrow that has a fixpoint reachable within `max_steps`
- **THEN** it returns that fixpoint value together with a flag indicating convergence was reached, and applying the arrow once more to the returned value leaves it unchanged

#### Scenario: Hitting the step bound reports non-convergence

- **WHEN** `iterate_to_fixpoint(arrow, x, max_steps)` is called with an arrow that never reaches a fixpoint (for example `t ↦ t + 1`)
- **THEN** it stops after `max_steps` applications and returns a flag indicating convergence was not reached, rather than iterating without bound

#### Scenario: Iterating until a predicate holds

- **WHEN** `iterate_until(arrow, x, predicate, max_steps)` is called with a predicate that becomes true within the bound
- **THEN** it returns the first value satisfying the predicate together with a flag indicating the predicate was met

