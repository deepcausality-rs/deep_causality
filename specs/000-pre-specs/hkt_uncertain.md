# HKT Witness Types for Uncertain and MaybeUncertain

## 1. Introduction

This report investigates the feasibility and implications of introducing Higher-Kinded Type (HKT) witness types for `Uncertain<T>` and `MaybeUncertain<T>` within the `deep_causality_uncertain` crate. The goal is to enhance the composability and abstract programming capabilities of these types by integrating them with the functional programming traits (Functor, Applicative, Monad) provided by the `deep_causality_haft` crate. We will also discuss the sensible exclusion of the `Foldable` trait for these specific types.

## 2. `Uncertain<T>` as an HKT

The `Uncertain<T>` type represents a single value `T` with inherent uncertainty, modeled as a probability distribution. It is generic over a single type parameter `T`, making it a natural candidate for an HKT of kind `* -> *`.

### 2.1. `UncertainWitness` Definition

To integrate `Uncertain<T>` into the HKT system, a zero-sized witness type `UncertainWitness` would be defined:

```rust
pub struct UncertainWitness;
```

### 2.2. `HKT` Trait Implementation

`UncertainWitness` would implement the core `HKT` trait as follows:

```rust
impl HKT for UncertainWitness {
    type Type<T> = Uncertain<T>;
}
```

This implementation declares that `UncertainWitness` represents the `Uncertain<T>` type constructor, allowing generic functions to abstract over its "shape."

### 2.3. Feasibility of Functional Traits (`Functor`, `Applicative`, `Monad`)

Implementing `Functor`, `Applicative`, and `Monad` for `UncertainWitness` (and thus for `Uncertain<T>`) is highly feasible and conceptually aligned with the type's probabilistic nature.

*   **`Functor` (fmap):**
    *   **Feasibility:** High. The `Uncertain<f64>::map` and `Uncertain<f64>::map_to_bool` methods already exist, demonstrating the ability to apply a function to the inner value of `Uncertain<T>` while preserving its uncertain context.
    *   **Implementation:** `fmap` would involve creating a new `Uncertain<B>` whose computation node applies the given function `f: Fn(A) -> B` to the result of sampling the original `Uncertain<A>`.
    *   **Benefit:** Allows for generic transformations of the uncertain value without altering its underlying probabilistic structure.

*   **`Applicative` (pure, apply):**
    *   **Feasibility:** High.
    *   **`pure`:** The `Uncertain::<T>::point(value)` constructor already serves the purpose of lifting a pure, certain value into the `Uncertain` context.
    *   **`apply`:** This operation would involve applying an `Uncertain<Func>` (where `Func` is a function type `Fn(A) -> B`) to an `Uncertain<A>`. The implementation would sample both the uncertain function and the uncertain argument, then apply the sampled function to the sampled argument.
    *   **Benefit:** Enables combining independent uncertain computations in a structured way, particularly useful for operations with multiple uncertain inputs.

*   **`Monad` (bind):**
    *   **Feasibility:** High.
    *   **Implementation:** `bind` would take an `Uncertain<A>` and a function `f: Fn(A) -> Uncertain<B>`. It would sample `Uncertain<A>`, apply `f` to the sampled value `A` to get an `Uncertain<B>`, and then sample from this resulting `Uncertain<B>`. This effectively chains dependent uncertain computations.
    *   **Benefit:** Provides a powerful mechanism for sequencing uncertain operations where the outcome of one uncertain step influences the definition of the next, crucial for complex probabilistic workflows.

## 3. `MaybeUncertain<T>` as an HKT

The `MaybeUncertain<T>` type represents a value that is probabilistically present or absent. If present, its value is itself `Uncertain<T>`. It is also generic over a single type parameter `T`, making it an HKT of kind `* -> *`.

### 3.1. `MaybeUncertainWitness` Definition

A zero-sized witness type `MaybeUncertainWitness` would be defined:

```rust
pub struct MaybeUncertainWitness;
```

### 3.2. `HKT` Trait Implementation

`MaybeUncertainWitness` would implement the core `HKT` trait as follows:

```rust
impl HKT for MaybeUncertainWitness {
    type Type<T> = MaybeUncertain<T>;
}
```

### 3.3. Feasibility of Functional Traits (`Functor`, `Applicative`, `Monad`)

Implementing functional traits for `MaybeUncertainWitness` is feasible, but requires careful consideration of the probabilistic presence (`is_present: Uncertain<bool>`).

*   **`Functor` (fmap):**
    *   **Feasibility:** High.
    *   **Implementation:** `fmap` would apply the function `f` to the inner `Uncertain<A>` only if `is_present` evaluates to `true` during sampling. Otherwise, it would yield `None`. The `is_present` flag would be propagated unchanged.
    *   **Benefit:** Allows for transformations of the uncertain value while respecting its potential absence.

*   **`Applicative` (pure, apply):**
    *   **Feasibility:** High.
    *   **`pure`:** Would create a `MaybeUncertain<T>` that is certainly present (`Uncertain::<bool>::point(true)`) and whose value is `Uncertain::<T>::point(value)`.
    *   **`apply`:** This would involve applying a `MaybeUncertain<Func>` to a `MaybeUncertain<A>`. The `is_present` flags of both would be combined (e.g., using logical AND), and the function would be applied to the inner `Uncertain<A>` only if both are present.
    *   **Benefit:** Combines independent uncertain computations, correctly propagating the possibility of absence.

*   **`Monad` (bind):**
    *   **Feasibility:** High.
    *   **Implementation:** `bind` would take a `MaybeUncertain<A>` and a function `f: Fn(A) -> MaybeUncertain<B>`. It would first sample the `is_present` flag of `MaybeUncertain<A>`. If `false`, the result is `None`. If `true`, it samples the inner `Uncertain<A>`, applies `f` to get a `MaybeUncertain<B>`, and then samples this result.
    *   **Benefit:** Enables sequencing dependent uncertain operations, where the presence or absence of a value at one step affects the next.

## 4. Exclusion of the `Foldable` Trait

While `Functor`, `Applicative`, and `Monad` are highly sensible for `Uncertain<T>` and `MaybeUncertain<T>`, the `Foldable` trait is generally **not a sensible inclusion** for these types.

### 4.1. Conceptual Mismatch

*   **`Foldable`'s Purpose:** The `Foldable` trait is designed for data structures that represent *collections* of values (e.g., `Vec`, `Option` as a collection of 0 or 1 items, `BTreeMap` as a collection of key-value pairs). Its primary operation, `fold`, reduces this collection to a single summary value.
*   **`Uncertain` and `MaybeUncertain`'s Nature:** `Uncertain<T>` and `MaybeUncertain<T>` represent *single probabilistic entities*, not collections. They encapsulate a single value (or the potential absence of one) whose exact state is unknown until sampled.

### 4.2. Loss of Information

*   Applying a traditional `fold` operation to a single `Uncertain<T>` would force it into a single, certain value, thereby destroying the very uncertainty it is designed to model. This would be a lossy and semantically misleading operation.
*   While one could conceive of "folding over samples" (e.g., summing up many samples to get an expected value), this is already covered by specific statistical methods like `expected_value()` or `standard_deviation()`. These are specialized aggregations that respect the probabilistic nature, rather than a generic `fold` that implies a structural reduction.

### 4.3. Clarity and Idiomatic Usage

*   Excluding `Foldable` maintains clarity about the nature of `Uncertain<T>` and `MaybeUncertain<T>` as single, probabilistic values rather than iterable collections.
*   It prevents developers from attempting to use `fold` in ways that might be semantically inappropriate or lead to unexpected results given the probabilistic context.

## 5. Benefits of HKT Integration

Integrating `Uncertain<T>` and `MaybeUncertain<T>` into the HKT system via witness types and functional traits offers several significant advantages:

*   **Enhanced Composability:** Allows these types to be seamlessly combined with other HKT-enabled types and abstractions within the `deep_causality` ecosystem and beyond.
*   **Increased Genericity:** Enables writing more abstract and reusable code that operates uniformly over any type that exhibits Functor, Applicative, or Monadic behavior, regardless of its specific underlying structure.
*   **Improved Code Clarity:** By adhering to well-established functional programming patterns, the code becomes more predictable and easier to reason about, especially when dealing with complex chains of uncertain computations.
*   **Stronger Abstraction:** Promotes a higher level of abstraction, separating the "what" (the computation logic) from the "how" (the uncertainty propagation mechanism).

## 6. Conclusion

The integration of `Uncertain<T>` and `MaybeUncertain<T>` with HKT witness types and the `Functor`, `Applicative`, and `Monad` traits from `deep_causality_haft` is highly feasible and recommended. This approach aligns with modern functional programming principles and will significantly enhance the utility and composability of the `deep_causality_uncertain` crate. The deliberate exclusion of the `Foldable` trait is sensible, as it avoids semantic confusion and respects the fundamental nature of these types as single probabilistic entities rather than collections.
