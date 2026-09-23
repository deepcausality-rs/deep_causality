# Deep Causality HAFT

**HAFT: Higher-Order Abstract Functional Traits**

`deep_causality_haft` provides traits for Higher-Kinded Types (HKTs) in Rust, so generic code can operate over
different container types like `Option<T>` and `Result<T, E>`. It is a sub-crate of the `deep_causality` project.

## Runnable examples

The snippets in this document teach one trait at a time and print their own output.
The runnable examples solve a domain problem instead, so their output differs from the snippets:

```bash
cargo run -p mathematics_examples --example haft_functor_examples        # data anonymization
cargo run -p mathematics_examples --example haft_effect_system_examples  # audited transactions
```

`examples/mathematics_examples/1_foundation/haft/README.md` lists them.

## What are Higher-Kinded Types?

In Rust, types like `Option<T>` and `Vec<T>` are generic over a type `T`. `Option` and `Vec` are "type
constructors": they take a type and produce a new type.

A Higher-Kinded Type abstracts over type constructors. Functions can then be generic over the *shape* or *kind* of a
type constructor, not only over a type. For example, one function can work with any type constructor that can be
mapped over (a `Functor`), whether it is an `Option`, a `Result`, or something else.

This crate provides the fundamental traits (`HKT`, `HKT2`, `HKT3`, `HKT4`, `HKT5`) and functional traits (`Functor`,
`Applicative`, `Monad`, `Foldable`, `Traversable`) for this pattern.

## Usage

This crate represents HKTs with a "witness" pattern. Each type constructor (like `Option`) gets a zero-sized
"witness" type (like `OptionWitness`) that implements the `HKT` trait. Witnesses are zero-sized and carry no runtime
cost. The crate ships witnesses for common Rust types, such as:

* Option -> OptionWitness
* Result -> ResultWitness
* Box -> BoxWitness
* Vec -> VecWitness

Each of these witness types implements the following traits and methods:

* Pure: `pure<T>(value: T)`
* Applicative: `apply<A, B, Func>(f_ab:HKT, f_a:HKT)`
* Functor: `fmap<A, B, Func>(m_a: HKT, f: Func)`
* Foldable: `fold<A, B, Func>(fa: HKT, init: B, f: Func)`
* Monad: `bind<A, B, Func>(m_a:HKT, f: Func)`

Witness types that implement only Functor and Foldable:
* BTreeMap -> BTreeMapWitness
* HashMap -> HashMapWitness
* VecDeque -> VecDequeWitness

### Example: Using `Functor` and `HKT` in a generic function

```rust
use deep_causality_haft::*;

fn double_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT 
    {
    F::fmap(m_a, |x| x * 2)
}

fn main() {
    // Using double_value with Option
    let opt = Some(5);
    println!("Original Option: {:?}", opt);
    let doubled_opt = double_value::<OptionWitness>(opt);
    println!("Doubled Option: {:?}", doubled_opt);

    // Using double_value with Result
    let res = Ok(5);
    println!("Original Result: {:?}", res);
    let doubled_res = double_value::<ResultWitness<i32>>(res);
    println!("Doubled Result: {:?}", doubled_res);

    // Using double_value with Box
    let b = Box::new(7);
    println!("Original Box: {:?}", b);
    let doubled_box = double_value::<BoxWitness>(b);
    println!("Doubled Box: {:?}", doubled_box);

    // Using double_value with Vec
    let vec = vec![1, 2, 3];
    println!("Original Vec: {:?}", vec);
    let doubled_vec = double_value::<VecWitness>(vec);
    println!("Doubled Vec: {:?}", doubled_vec);
    
    // Using double_value with VecDeque
    let vec_dec = VecDeque::<i32>::from(vec![2, 4, 6]);
    println!("Original VecDec: {:?}", vec_dec);
    let doubled_vec_dec = double_value::<VecDequeWitness>(vec_dec);
    println!("Doubled VecDec: {:?}", doubled_vec_dec);
    assert_eq!(doubled_vec_dec, vec![4, 8, 12]);
}
```

Running that snippet prints:

```text 
Original Option: Some(5)
Doubled Option: Some(10)
Original Result: Ok(5)
Doubled Result: Ok(10)
Original Box: 7
Doubled Box: 14
Original Vec: [1, 2, 3]
Doubled Vec: [2, 4, 6]
Original VecDec: [2, 4, 6]
Doubled VecDec: [4, 8, 12]
```

Combined with the deep_causality_num crate, the abstraction extends to the numeric type:

```rust
use deep_causality_haft::{Functor, HKT};
use deep_causality_num::{Float, One}; 
   
fn double_float<F, T>(m_a: F::Type<T>) -> F::Type<T>
   where
     F: Functor<F> + HKT,
     T: Float + One, // T must be a float and support multiplication
     {
     F::fmap(m_a, |x| x * (T::one() + T::one())) // Robust way to multiply by 2.0 for any Float
}
```

This helps in numerical computing, where many data structures hold different numeric types.

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

Running that snippet prints:

```text 
Process Steps: 
  Log (Step 1): Operation A: Multiplied by 2
  Log (Step 2): Operation B: Added 5
  Log (Step 3): Operation C: Multiplied by 3

Sequenced outcome: 75

... (Truncated)
```

The `Effect3`, `Effect4`, `Effect5` and `MonadEffect3`, `MonadEffect4`, `MonadEffect5` traits build a
**type-encoded effect system**. It manages side-effects (like errors and logging) in a structured, composable way, for
example in data processing pipelines, and uses the type system to make every effect explicit and tracked.

How it works:

1. **Effects as Types**: The system represents side-effects explicitly as generic type parameters on a container
   type. For instance, you might have a custom effect type like
   `MyCustomEffectType<T, E, W>`, where:
    * `T` is the primary value of the computation.
    * `E` represents an error type.
    * `W` represents a warning or log type.
      With the effects in the type signature, the compiler sees and verifies every potential side-effect.

2. **Higher-Kinded Type (HKT) Witnesses**: To make these effect types generic over their primary value `T` while keeping
   the effect types (`E`, `W`, etc.) fixed, the system uses Higher-Kinded Types (HKTs). Traits like `Effect3`,
   `Effect4`, and `Effect5` fix a number of generic parameters of an underlying HKT type (e.g., `HKT3`, `HKT4`,
   `HKT5`). You define a "witness" type (e.g., `MyEffectHktWitness<E, W>`) that represents the *shape* of your effect
   container with specific, fixed effect types, leaving one parameter (`T`) open for the actual value.

3. **Monadic Logic for Effects (`MonadEffect` traits)**: The `MonadEffect` traits (e.g., `MonadEffect3`,
   `MonadEffect4`, `MonadEffect5`) define how effects are handled and combined. They provide:
    * **`pure`**: A method to lift a "pure" value (a value without any side-effects) into the effectful context.
    * **`bind`**: The central sequencing operation. It chains computations where each step might produce new effects,
      and its implementation decides how effects from different steps combine. In the provided `MyCustomEffectType`,
      `bind` propagates an error from any step and accumulates the warnings from all steps.

4. **Specialized Effect Handling (`LoggableEffect` traits)**: Specialized traits extend the system for specific kinds
   of effects. The test utilities in `utils_tests` define `LoggableEffect3`, `LoggableEffect4`, and `LoggableEffect5`,
   whose `log` function adds a log message (of a fixed type, like `E::Fixed2` for `LoggableEffect3`) to the effect
   container without altering the primary value or causing an error.

5. **Compiler-Enforced Safety**: Because effects are part of the type signature, the compiler statically verifies
   that they are handled. If a function declares an effect, the compiler ensures the effect is either handled or
   propagated, which prevents unhandled errors and forgotten logging.


## Unbound HKTs & Functional Traits (Arity 2-5)

The crate also supports "Unbound" Higher-Kinded Types, where all generic parameters are free to vary. They carry functional patterns from category theory used in systems modeling.

### Unbound HKT Traits

*   **`HKT2Unbound` - `HKT5Unbound`**: Base traits for multi-arity type constructors (e.g., `Result<A, B>`, `(A, B, C)`).
*   **`Bifunctor`**: Maps over both types of a binary constructor simultaneously.
    *   *Usage*: Evolving a coupled system (e.g., `(Metric, Plasma)`) where both components change type.
*   **`Profunctor`**: Contravariant input, Covariant output.
    *   *Usage*: Adapters, Optics, and State Machines where you pre-process input and post-process output.
*   **`Adjunction`**: Defines a dual relationship between two functors ($L \dashv R$).
    *   *Usage*: Conservation laws, optimization (Primal/Dual), and Galois connections.
*   **`ParametricMonad`**: A Monad where the state type changes (Indexed Monad).
    *   *Usage*: Modeling state transitions (e.g., `Solid -> Liquid -> Gas`) or protocol state machines.
*   **`MonoidalMerge`**: Models interaction or fusion of contexts.
    *   *Usage*: Tensor products, force calculations (merging fields), and quantum entanglement.
*   **`RiemannMap`**: Models curvature and scattering (Arity 4).
    *   *Usage*: General Relativity (Curvature Tensor), Particle Physics (Scattering Matrices).
*   **`CyberneticLoop`**: Models a complete feedback control loop (Arity 5).
    *   *Usage*: Autonomous agents (OODA Loop), Control Theory, and Error Correction.

### Example: Bifunctor

```rust
use deep_causality_haft::{Bifunctor, HKT2Unbound};

struct ResultWitness;
impl HKT2Unbound for ResultWitness {
    type Type<A, B> = Result<A, B>;
}

impl Bifunctor<ResultWitness> for ResultWitness {
    fn bimap<A, B, C, D, F1, F2>(fab: Result<A, B>, mut f1: F1, mut f2: F2) -> Result<C, D>
    where F1: FnMut(A) -> C, F2: FnMut(B) -> D {
        match fab {
            Ok(a) => Ok(f1(a)),
            Err(b) => Err(f2(b)),
        }
    }
}

// Usage
let res: Result<i32, &str> = Ok(10);
let new_res = ResultWitness::bimap(res, |x| x * 2.0, |e| e.len()); // Result<f64, usize>
```

## Natural Isomorphisms (Tier 3 Iso Traits)

The crate provides traits for **natural isomorphisms** between HKT witnesses: bijections between two type constructors that commute with `fmap`. They form Tier 3 of the three-tier isomorphism design; Tiers 1 and 2, for concrete-type isos, live in `deep_causality_algebra`.

HKT witnesses are zero-sized markers without values, so `From`/`Into` cannot apply at this level and a witness-typed trait is required. Each `NaturalIso*` is parameterised over the two witnesses `F` and `G` and exposes `to_target` / `to_source`:

| Trait | Base HKT | Free type parameters |
|---|---|---|
| `NaturalIso<F, G>` | `HKT` | `T` |
| `NaturalIso2<F, G>` | `HKT2Unbound` | `A, B` |
| `NaturalIso3<F, G>` | `HKT3Unbound` | `A, B, C` |
| `NaturalIso4<F, G>` | `HKT4Unbound` | `A, B, C, D` |
| `NaturalIso5<F, G>` | `HKT5Unbound` | `V, S, C, E, L` |

### Laws

Every implementer must satisfy, for every type parameter tuple permitted by both witnesses' constraints:

1. **Round-trip identity** (both directions independently):
   * `to_source(to_target(fa)) == fa`
   * `to_target(to_source(ga)) == ga`
2. **Naturality**: `to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)` for any function `h: T -> U` (and the symmetric law through `to_source`). Naturality makes the iso *structure-preserving*: it respects every later `fmap`.

The `deep_causality_haft::iso::test_support` module provides `assert_natural_iso_round_trip` and `assert_natural_iso_naturality` helpers for property-style verification in downstream crates.

### Example: `NaturalIso` between `Option` and a structurally-equivalent twin

```rust,ignore
use deep_causality_haft::{HKT, NaturalIso, OptionWitness};

#[derive(Debug, Clone, PartialEq)]
enum MyOption<T> { MySome(T), MyNone }

struct MyOptionWitness;
impl HKT for MyOptionWitness {
    type Type<T> = MyOption<T>;
}

struct OptionMyOptionIso;
impl NaturalIso<OptionWitness, MyOptionWitness> for OptionMyOptionIso {
    fn to_target<T>(fa: Option<T>) -> MyOption<T> {
        match fa { Some(t) => MyOption::MySome(t), None => MyOption::MyNone }
    }
    fn to_source<T>(ga: MyOption<T>) -> Option<T> {
        match ga { MyOption::MySome(t) => Some(t), MyOption::MyNone => None }
    }
}
```

### Use cases

* **Carrier-shape refactors**: prove two representations of the same conceptual container (e.g. `Option<T>` vs a domain-specific `Maybe<T>` newtype) are interchangeable, without re-deriving every `Functor`/`Monad` impl.
* **Effect-system migration**: `NaturalIso5` is the natural fit for swapping the propagating-effect carrier `<V, S, C, E, L>` with an equivalent shape (e.g. a logging-only specialisation), with naturality guaranteeing that any pipeline written against the old carrier still composes against the new one.
* **Theory-side equivalences**: encode "these two functors are the same" as a checked law rather than a comment.

## non-std support

The crate supports `no-std` environments such as embedded systems. The `std` feature is on by default, so disable it to build for `no-std`. With an allocator, enable the `alloc` feature for dynamic collections like `Vec`, `Box`, `BTreeMap`, etc.

### Cargo Build and Test for `no-std`

**1. Building for `no-std` with Allocator:**

To build for `no-std` with dynamic collections (via `alloc`):

```bash
cargo build --no-default-features --features alloc -p deep_causality_haft
```

**2. Testing for `no-std` with Allocator:**

To test for `no-std` with an allocator:

```bash
cargo test --no-default-features --features alloc -p deep_causality_haft
```

**3. Building for `no-std` without Allocator (Core/Algebra only):**

Without an allocator, build without the `alloc` feature. The crate then offers only the core HKT traits and algebraic structures that need no dynamic memory.

```bash
cargo build --no-default-features -p deep_causality_haft
```

**4. Testing for `no-std` without Allocator:**

```bash
cargo test --no-default-features -p deep_causality_haft
```

### Bazel Build

For regular (std) builds, run:

```bash
   bazel build //deep_causality_haft/...
```

and

```bash
   bazel test //deep_causality_haft/...
```

for tests. To build for non-std, use

```bash
   bazel build --@rules_rust//rust/settings:no_std=alloc //deep_causality_haft/...
```

and

```bash
   bazel test --@rules_rust//rust/settings:no_std=alloc //deep_causality_haft/...
```

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

