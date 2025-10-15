# Deep Causality HAFT

**HAFT: Higher-Order Abstract Functional Traits**

`deep_causality_haft` is a sub-crate of the `deep_causality` project, providing traits for Higher-Kinded Types (HKTs) in Rust. This enables writing generic, abstract code that can operate over different container types like `Option<T>` and `Result<T, E>`.

## What are Higher-Kinded Types?

In Rust, types like `Option<T>` and `Vec<T>` are generic over a type `T`. We can think of `Option` and `Vec` as "type constructors": they take a type and produce a new type.

A Higher-Kinded Type is an abstraction over these type constructors. It allows us to write functions that are generic not just over a type, but over the *shape* or *kind* of a type constructor. For example, we can write a function that works with any type constructor that can be mapped over (a `Functor`), without caring if it's an `Option`, a `Result`, or something else.

This crate provides the fundamental traits (`HKT`, `HKT2`, `HKT3`) and functional traits (`Functor`, `Monad`) to enable this pattern.

## Usage

This crate uses a "witness" pattern to represent HKTs. For each type constructor (like `Option`), we define a zero-sized "witness" type (like `OptionWitness`) that implements the `HKT` trait.

### Example: Using `Functor` with `Option`

Here's how you can use the `Functor` trait with `Option` via its witness type, `OptionWitness`.

```rust
use deep_causality_haft::{Functor, HKT, OptionWitness};

// Manual implementation of Functor for OptionWitness
impl Functor<OptionWitness> for OptionWitness {
    fn fmap<A, B, Func>(m_a: <OptionWitness as HKT>::Type<A>, f: Func) -> <OptionWitness as HKT>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

fn main() {
    let opt_a = Some(5);
    let f = |x| x * 2;

    // Use the fmap function from our Functor implementation
    let opt_b = OptionWitness::fmap(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_mapped = OptionWitness::fmap(opt_none, f);
    assert_eq!(opt_none_mapped, None);
}
```

### Example: Using `Functor` with `Result`

Here's how you can use the `Functor` trait with `Result<T, E>` via its witness type, `ResultWitness<E>`. `HKT2` is used here because `Result` has two generic parameters, and we are fixing the error type `E`.

```rust
use deep_causality_haft::{Functor, HKT2, ResultWitness};

// Manual implementation of Functor for ResultWitness
impl<E> Functor<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    fn fmap<A, B, Func>(m_a: <ResultWitness<E> as HKT2<E>>::Type<A>, f: Func) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

fn main() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| x * 2;

    // Use the fmap function from our Functor implementation
    let res_b = ResultWitness::fmap(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_mapped = ResultWitness::fmap(res_err, f);
    assert_eq!(res_err_mapped, Err("Error".to_string()));
}
```


## Type-Encoded Effect System

The `Effect3` and `MonadEffect3` traits provide a powerful mechanism for building a **type-encoded effect system**. This allows you to manage side-effects (like errors and logging) in a structured, safe, and composable way, which is particularly useful for building complex data processing pipelines.

### How it Works

1.  **Effects as Types**: Side-effects are represented by generic type parameters on a container (e.g., `E` for Error, `W` for Warning on a custom `MyEffect<T, E, W>` type).
2.  **Rules as Traits**: The logic for how to handle and combine these effects is defined by implementing the `MonadEffect3` trait. For example, the `bind` function can specify that the pipeline should halt on an error while accumulating warnings.
3.  **Compiler-Enforced Safety**: Because the effects are part of the type signature, the Rust compiler can statically verify that all effects are handled correctly. This prevents bugs and ensures that your pipeline code remains pure and focused on its core logic.
4.  **Extensibility**: This pattern is extensible. If you need to manage more side-effects, you can introduce `HKT4` and `Effect4` traits to handle them, without having to rewrite your core pipeline logic.

In essence, this crate provides the tools to build a small, powerful, and compile-time-checked effects library tailored perfectly for your application's needs, forming the foundation for building powerful, abstract, and reusable causal models in the `deep_causality` ecosystem.
```

