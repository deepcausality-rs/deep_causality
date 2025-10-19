# Deep Causality HAFT

**HAFT: Higher-Order Abstract Functional Traits**

`deep_causality_haft` is a sub-crate of the `deep_causality` project, providing traits for Higher-Kinded Types (HKTs) in
Rust. This enables writing generic, abstract code that can operate over different container types like `Option<T>` and
`Result<T, E>`.

## What are Higher-Kinded Types?

In Rust, types like `Option<T>` and `Vec<T>` are generic over a type `T`. We can think of `Option` and `Vec` as "type
constructors": they take a type and produce a new type.

A Higher-Kinded Type is an abstraction over these type constructors. It allows us to write functions that are generic
not just over a type, but over the *shape* or *kind* of a type constructor. For example, we can write a function that
works with any type constructor that can be mapped over (a `Functor`), without caring if it's an `Option`, a `Result`,
or something else.

This crate provides the fundamental traits (`HKT`, `HKT2`, `HKT3`, `HKT4`, `HKT5`) and functional traits (`Functor`,
`Applicative`, `Monad`, `Foldable`) to enable this pattern.

## Usage

This crate uses a "witness" pattern to represent HKTs. For each type constructor (like `Option`), we define a
zero-sized "witness" type (like `OptionWitness`) that implements the `HKT` trait. These witness types are zero-sized 
and incur no runtime overhead, making them zero-cost abstractions. This crates also comes with default 
witness pattern implementations for commonly used Rust types such as:

* Option -> OptionWitness
* Result -> ResultWitness
* Vec<T> -> VecWitness

Each of those withness types implements the following traits and methods:

* Applicative: `pure<T>(value: T)` and `apply<A, B, Func>(f_ab:HKT, f_a:HKT)`
* Functor: `fmap<A, B, Func>(m_a: HKT, f: Func)`
* Foldable: `fold<A, B, Func>(fa: HKT, init: B, f: Func)`
* Monad: `bind<A, B, Func>(m_a:HKT, f: Func)`

### Example: Using `Functor` with `Option`

Here's how you can use the `Functor` trait with `Option` via its witness type, `OptionWitness`.

```rust
use deep_causality_haft::{Functor, HKT, OptionWitness};

fn main() {
    let opt_a = Some(5);
    let f = |x| x * 2;

    // Use the fmap function from the Functor implementation in OptionWitness
    let opt_b = OptionWitness::fmap(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_mapped = OptionWitness::fmap(opt_none, f);
    assert_eq!(opt_none_mapped, None);
}
```

### Example: Using `Functor` with `Result`

Here's how you can use the `Functor` trait with `Result<T, E>` via its witness type, `ResultWitness<E>`. `HKT2` is used
here because `Result` has two generic parameters, and we are fixing the error type `E`.

```rust
use deep_causality_haft::{Functor, HKT2, ResultWitness};

fn main() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| x * 2;

    // Use the fmap function from the Functor implementation in ResultWitness
    let res_b = ResultWitness::fmap(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_mapped = ResultWitness::fmap(res_err, f);
    assert_eq!(res_err_mapped, Err("Error".to_string()));
}
```

### Example: Using `Foldable` with `Vec`
I 
Here's how you can use the `Foldable` trait with `Vec` via its witness type, `VecWitness`.

```rust
use deep_causality_haft::{Foldable, VecWitness};

fn main() {
    let vec_a = vec![1, 2, 3, 4, 5];

    // Use the fold function from the Foldable implementation in VecWitness to sum elements
    let sum = VecWitness::fold(vec_a, 0, |acc, x| acc + x);
    assert_eq!(sum, 15);

    let vec_empty: Vec<i32> = Vec::new();
    let sum_empty = VecWitness::fold(vec_empty, 0, |acc, x| acc + x);
    assert_eq!(sum_empty, 0);

    let words = vec!["hello".to_string(), "world".to_string()];
    let concatenated = VecWitness::fold(words, String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(&x);
        acc
    });
    assert_eq!(concatenated, "hello world");
}
```

## Type-Encoded Effect System

```rust
use deep_causality_haft::utils_tests::*;
use deep_causality_haft::{Effect5, MonadEffect5, HKT5};

  // 1. Start with a pure value, lifting it into the effect context
    let initial_effect: MyEffectType<i32> = MyMonadEffect5::pure(10);

    // 2. Define a collection of step functions
    // Each function takes an i32 and returns an effectful i32
    let step_functions: Vec<Box<dyn Fn(i32) -> MyEffectType<i32>>> = vec![
        Box::new(|x: i32| {
            MyCustomEffectType5 {
                value: x * 2,
                f1: None,
                f2: vec!["Operation A: Multiplied by 2".to_string()],
                f3: vec![1],
                f4: vec!["Trace: Executing step 1".to_string()],
            }
        }),
        Box::new(|x: i32| {
            MyCustomEffectType5 {
                value: x + 5,
                f1: None,
                f2: vec!["Operation B: Added 5".to_string()],
                f3: vec![1],
                f4: vec!["Trace: Executing step 2".to_string()],
            }
        }),
        Box::new(|x: i32| {
            MyCustomEffectType5 {
                value: x * 3,
                f1: None,
                f2: vec!["Operation C: Multiplied by 3".to_string()],
                f3: vec![1],
                f4: vec!["Trace: Executing step 3".to_string()],
            }
        }),
    ];

    // 3. Execute all step functions in sequence 
    println!("Process Steps: ");
    let mut current_effect = initial_effect;
    for (i, f) in step_functions.into_iter().enumerate() {
        let prev_logs_len = current_effect.f2.len();
        current_effect = MyMonadEffect5::bind(current_effect, f);
        for log_msg in current_effect.f2.iter().skip(prev_logs_len) {
            println!("  Log (Step {}): {}", i + 1, log_msg);
        }
    }

    println!("Sequenced outcome: {:?}", current_effect.value);
```

When you run [the example ](/deep_causality_haft/examples/effect_system_example.rs)via:

`cargo run  --example haft_effect_system_example`

You will see:

```text 
--- Type-Encoded Effect System Example (Arity 5) ---

Initial effect (pure 10): MyCustomEffectType5 { value: 10, f1: None, f2: [], f3: [], f4: [] }

Process Steps: 
  Log (Step 1): Operation A: Multiplied by 2
  Log (Step 2): Operation B: Added 5
  Log (Step 3): Operation C: Multiplied by 3

Sequenced outcome: 75

... (Truncated)
```

The `Effect3`, `Effect4`, `Effect5` and `MonadEffect3`, `MonadEffect4`, `MonadEffect5` traits provide a powerful
mechanism for building a **type-encoded effect system**. This allows you to manage side-effects (like errors and
logging) in a structured, safe, and composable way, which is particularly useful for building complex data processing
pipelines. It leverages Rust's powerful type system to ensure that these effects are explicitly handled 
and tracked throughout your program.

Here's a breakdown of how it works:

1. **Effects as Types**: Instead of side-effects occurring implicitly, this system represents them explicitly as generic
   type parameters on a container type. For instance, you might have a custom effect type like
   `MyCustomEffectType<T, E, W>`, where:
    * `T` is the primary value of the computation.
    * `E` represents an error type.
    * `W` represents a warning or log type.
      By making these effects part of the type signature, the presence of potential side-effects becomes explicit and
      verifiable by the compiler.

2. **Higher-Kinded Type (HKT) Witnesses**: To make these effect types generic over their primary value `T` while keeping
   the effect types (`E`, `W`, etc.) fixed, the system utilizes Higher-Kinded Types (HKTs). Traits like `Effect3`,
   `Effect4`, and `Effect5` are used to "fix" a certain number of generic parameters of an underlying HKT type (e.g.,
   `HKT3`, `HKT4`, `HKT5`). This allows you to define a "witness" type (e.g., `MyEffectHktWitness<E, W>`) that
   represents the *shape* of your effect container with specific, fixed effect types, leaving one parameter (`T`) open
   for the actual value.

3. **Monadic Logic for Effects (`MonadEffect` traits)**: The core logic for how these effects are handled and combined
   is defined through `MonadEffect` traits (e.g., `MonadEffect3`, `MonadEffect4`, `MonadEffect5`). These traits provide:
    * **`pure`**: A method to lift a "pure" value (a value without any side-effects) into the effectful context.
    * **`bind`**: The central sequencing operation. It allows you to chain computations where each step might produce
      new effects. The implementation of `bind` dictates how effects from different steps are combined. For example, in
      the provided `MyCustomEffectType`, the `bind` implementation ensures that if an error occurs at any point, it
      propagates, and warnings from all steps are accumulated.

4. **Specialized Effect Handling (`LoggableEffect` traits)**: The system can be extended with specialized traits for
   specific types of effects. For example, `LoggableEffect3`, `LoggableEffect4`, and `LoggableEffect5` provide a `log`
   function. This function allows you to add a log message (of a specific fixed type, like `E::Fixed2` for
   `LoggableEffect3`) to the effect container without altering the primary value or causing an error.

5. **Compiler-Enforced Safety**: A significant advantage of this system is that because effects are part of the type
   signature, the Rust compiler statically verifies that all effects are handled correctly. This means that if a
   function is declared to produce a certain type of effect, the compiler ensures that the effect is either explicitly
   handled or propagated. This prevents common bugs related to unhandled errors or forgotten logging, leading to more
   robust and predictable code.
