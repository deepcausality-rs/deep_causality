# DeepCausality HAFT: Higher Abstract Functional Types

`deep_causality_haft` is the foundational utility crate that provides **Higher-Kinded Types (HKT)** and **functional programming abstractions** for the DeepCausality ecosystem. It serves as the bedrock upon which the core monadic effect system is built.

---

## 🏗️ The Problem: Abstraction in Rust

Rust's type system is powerful, but it lacks native support for **Higher-Kinded Types (HKTs)**. 

In languages like Haskell, you can write a function that takes a generic type constructor `F<_>` (like `Option`, `Vec`, or `Result`) and operates on it. In standard Rust, you can't easily write a trait that says: *"This works for any wrapper type `W<T>` that has a `map` function."*

**DeepCausality needs this abstraction** to define generic causal logic that works across:
- Simple Values (`Option<T>`)
- Error-prone computations (`Result<T, E>`)
- Complex Causal Processes (`CausalEffectPropagationProcess<T, S, C, E, L>`)
- Quantum States (`HilbertState<T>`)

---

## 🧩 The Solution: The "Witness" Pattern

`deep_causality_haft` implements HKTs using the **Witness Pattern** (also known as the "Family" pattern) with Generic Associated Types (GATs).

### 1. The HKT Trait
The core concept is a trait that maps a specific type `T` to a type constructor `Type<T>`:

```rust
pub trait HKT {
    type Type<T>;
}
```

### 2. The Witness
Since we can't implement traits directly on generic type constructors (e.g., `Option`), we define zero-sized "Witness" structs that represent them:

```rust
pub struct OptionWitness;

impl HKT for OptionWitness {
    type Type<T> = Option<T>;
}
```

Now `OptionWitness` allows us to talk about "Option-ness" generally.

---

## 📚 Core Modules

### 1. Algebraic Traits (Functional Programming)
Once we have HKTs, we can define standard functional programming traits that work for *any* container:

| Trait | Concept | Description |
|-------|---------|-------------|
| `Functor` | Map | `fmap: (A -> B) -> F<A> -> F<B>` |
| `Applicative` | Apply | `pure: A -> F<A>` AND `apply: F<A -> B> -> F<A> -> F<B>` |
| `Monad` | Chain | `bind: F<A> -> (A -> F<B>) -> F<B>` |
| `Foldable` | Reduce | `fold: F<A> -> Acc -> Acc` |
| `Traversable` | Flip | `sequence: F<G<A>> -> G<F<A>>` |

### 2. Arity & Fixed Parameters (`HKT2` - `HKT5`)
Real-world types often have more than one generic parameter (e.g., `Result<T, E>`). DeepCausality provides **Arity Traits** to "fix" secondary parameters so they can behave like single-parameter Monads.

*   `HKT2<F>`: Fixes 1 type (e.g., `Result<T, FixedError>`)
*   ...
*   `HKT5<F1, F2, F3, F4>`: Fixes 4 types (used by the core Causal Monad)

**Example:**
A `ResultWitness<String>` implements `HKT2<String>`. Its `Type<T>` becomes `Result<T, String>`.

### 3. The Effect System Bridge
This is the specialized layer that `deep_causality_core` uses. It defines traits like `Effect5` to abstract over complex systems with fixed State, Context, Error, and Logs.

```rust
pub trait Effect5 {
    type Fixed1; // e.g. Error
    type Fixed2; // e.g. Log
    type Fixed3; // e.g. State
    type Fixed4; // e.g. Context
    
    // The simplified HKT that operates on the remaining open 'Value' type
    type HktWitness: HKT5<..., ...>; 
}
```

This allows `CausalMonad` to be written generically for *any* system that provides these 4 fixed types.

---

## 🚀 Why is this important?

Because `deep_causality_haft` exists, the rest of the ecosystem can be **polymorphic over effects**.

1.  **Uniform API**: `map`, `bind`, and `pure` work exactly the same for a simple `Option` as they do for a complex `CausalEffectPropagationProcess`.
2.  **Testability**: We can test causal logic using simple `Vector` or `Result` types without spinning up the full causal engine.
3.  **Extensibility**: If you need a new type of causal container (e.g., a GPU-backed tensor stream), you just implement the `HKT` witness, and all existing algorithms work automatically.

---

## 📦 Extended Features

The crate also includes:
*   **Adjunctions**: For modeling relationships between different categories (used in advanced causal dualities).
*   **Cybernetic Loops**: Traits for feedback systems.
*   **Riemann Maps**: Geometric mapping traits.
*   **Standard Extensions**: Pre-built witnesses for `Vec`, `HashMap`, `BTreeMap`, `LinkedList`, `Box`, `Result`, and `Option`.

---

## Summary

`deep_causality_haft` is the **abstract engine room**. It doesn't contain causal logic itself; it contains the *language* (HKTs, Monads) used to express that logic elegantly and safely.
