---
title: Higher-Kinded Types
description: How DeepCausality encodes the type constructors that Rust does not natively support.
section: concepts
order: 7
---

A higher-kinded type is a type that takes another type as a parameter and produces a type. Most languages with a serious type system have first-class HKTs; Rust does not. DeepCausality needs HKTs anyway because the Causal Monad is one. The library encodes them with a small amount of trait machinery, and the result is essentially free at runtime.

This page explains what gets encoded, where the encoding lives, and what you need to know to use it.

## The shape of the problem

The Causal Monad has the operation `pure` whose signature, in a hypothetical Rust with HKT support, would read:

```text
// Hypothetical syntax; not real Rust.
fn pure<T>(t: T) -> M<T>
```

Where `M` is a type constructor: give it a `T`, get back `M<T>`. The same goes for `bind`:

```text
fn bind<T, U>(m: M<T>, f: impl Fn(T) -> M<U>) -> M<U>
```

A monad implementation needs `M` to be a parameter. Rust lacks the syntax. The workaround uses a *witness* type plus an associated type.

## The encoding

The crate `deep_causality_haft` defines the trait hierarchy:

```rust
pub trait HKT {
    type Constraint;
    type Type<T>: Satisfies<Self::Constraint>;
}

pub trait HKT3<Fixed1, Fixed2> {
    type Type<T>;
}

pub trait HKT5<Fixed1, Fixed2, Fixed3, Fixed4> {
    type Type<T>;
}
```

Each variant fixes some parameters and varies the rest. `HKT5` is the one the Causal Monad uses: four slots are fixed (state, context, error, log) and the fifth (the value type) varies through `Type<T>`.

The witness pattern looks like this:

```rust
pub struct CausalEffectPropagationProcessWitness<S, C, E, L>(
    Placeholder,
    PhantomData<S>,
    PhantomData<C>,
    PhantomData<E>,
    PhantomData<L>,
);

impl<S, C, E, L> HKT5<S, C, E, L> for CausalEffectPropagationProcessWitness<S, C, E, L> {
    type Type<Value> = CausalEffectPropagationProcess<Value, S, C, E, L>;
}
```

The witness type is zero-sized at runtime; the body of the impl is the trick. `Type<Value>` is the type-level function that produces the concrete process given a value type.

A second witness, `PropagatingEffectWitness<E, L>`, fixes the state and context to `()` and `()`. It is what the stateless `PropagatingEffect` alias uses.

## What you actually write

Almost nothing. The witnesses live behind aliases and are not part of the day-to-day API surface. Most code looks like:

```rust
use deep_causality::prelude::*;

let m: PropagatingEffect<i32> = CausalMonad::<(), ()>::pure(10);
let n = CausalMonad::<(), ()>::bind(m, |x| CausalMonad::pure(x + 1));
```

There is no `HKT5::Type<…>` in sight. The compiler resolves it. The encoding earns its keep because the *library author* could write the monad's `bind` once, generically over the witness, and have it work for every concrete instantiation.

## Why this matters

The HKT machinery is what lets the Causal Monad satisfy its laws *generically*, then specialize to a hundred concrete shapes (stateless, stateful, with this context, with that error, with that log) without rewriting the laws each time. The runtime cost is the cost of monomorphization: the compiler emits one specialized version per concrete witness, no virtual calls, no boxed type erasure.

A sibling payoff: the type-checker becomes the law-checker. If a `bind` impl produces something whose `Type<U>` does not match the expected output type, the program does not compile. The kind of cross-arity bug that bites you in dynamic monad implementations is impossible here.

## When you would touch this directly

Three cases.

You are writing a new monad on top of the existing machinery. Pick `HKT3` or `HKT5` depending on how many parameters you need to fix; define a witness; implement `Functor`, `Pure`, `Applicative`, and `Monad` against it. The existing impls in `deep_causality_core::types::causal_effect_propagation_process::hkt` are the worked examples.

You are writing a function that should be polymorphic across multiple effects. The function signature carries the witness type as a generic. Most application code does not need this; library code that wants to be reusable across effects does.

You are debugging a confusing type error inside a `bind` chain. The error message will mention `Type<T>` somewhere; knowing what `Type<T>` is helps you read it.

## What it is not

The HKT machinery is not a runtime feature. There is no dispatch at runtime. The witness is zero-sized.

The HKT machinery is not equivalent to GATs. Rust's Generic Associated Types help inside this encoding (the `type Type<T>` is a GAT), but they do not give you HKTs by themselves. The witness pattern is what closes the gap.

The HKT machinery is not pretending to be Haskell. It encodes the parts the library actually uses. It does not try to import the whole typeclass hierarchy from Prelude.

## Where to look next

[Causal Monad](/docs/concepts/causal-monad/) is the user of the HKT encoding. [Effect Propagation Process](/docs/concepts/effect-propagation-process/) is the type the encoding parameterizes. The `deep_causality_haft` crate's docs.rs page has the formal definitions for `HKT`, `HKT3`, and `HKT5`.
