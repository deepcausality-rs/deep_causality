---
title: Higher-Kinded Types
description: How DeepCausality encodes the type constructors that Rust does not natively support.
sidebar:
  order: 8
---

A higher-kinded type is a type that takes another type as a parameter and produces a type. For historical reasons, the Rust team decided against including higher-kinded types into the Rust programming language. However, with the introduction of the causal discovery language, monadic composition becomes a viable alternative, and that also enabled the causal monad and effect propagation process. Therefore, the Deep Causality Project decided to include a higher-kinded type implementation in a dedicated crate that uses the witness pattern and a trait hierarchy to establish arity-five higher-kinded types and the corresponding effect.

## The encoding

The crate [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft) defines the trait hierarchy:

```rust
pub trait HKT {
    type Constraint: ?Sized;
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}

pub trait HKT2<F> {
    type Type<T>;
}


pub trait HKT3<Fixed1, Fixed2> {
    type Type<T>;
}

pub trait HKT4<F1, F2, F3> {
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

The witnesses live behind aliases and are not part of the day-to-day API surface. Most code looks like:

```rust
use deep_causality::PropagatingEffect;

let m: PropagatingEffect<i32> = PropagatingEffect::pure(10);
let n = m.bind(|value, _state, _context| {
    let x = value.into_value().unwrap_or_default();
    PropagatingEffect::pure(x + 1)
});
```

There is no `HKT5::Type<…>` in sight. The compiler resolves it. The *library author* writes the monad's `bind` once, generically over the witness, and have it work for every concrete instantiation.

## Why this matters

The HKT machinery is what lets the Causal Monad satisfy its laws *generically*, then specialize to a hundred concrete shapes (stateless, stateful, with this context, with that error, with that log) without rewriting the laws each time. The runtime cost is the cost of monomorphization: the compiler emits one specialized version per concrete witness, no virtual calls, no boxed type erasure.


## Where to look next

[Causal Monad](/concepts/causal-monad/) is the user of the HKT encoding. [Effect Propagation Process](/concepts/effect-propagation-process/) is the type the encoding parameterizes. The formal definitions for `HKT`, `HKT3`, and `HKT5` are on [docs.rs/deep_causality_haft](https://docs.rs/deep_causality_haft).
