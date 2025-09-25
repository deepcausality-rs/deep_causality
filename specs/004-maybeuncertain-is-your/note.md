# Notes for "Related Work" Section

This document contains raw material and discussion points regarding concepts similar to `MaybeUncertain<T>` that exist
in other programming paradigms and fields.

---

## Initial Brainstorm on Related Concepts

The concept behind `MaybeUncertain<T>`—unifying uncertainty about a value's existence with uncertainty about the value
itself—is a powerful one, and it has parallels in several fields outside of DeepCausality.

Here are a few of the most prominent examples:

### 1. Probabilistic Programming Languages (PPLs)

This is the most direct and powerful parallel. Languages like **Pyro**, **Stan**, and **PyMC** are designed specifically
to build models where any parameter, or even the structure of the model itself, can be a random variable.

`MaybeUncertain<T>` is essentially a small, self-contained PPL for a single variable.

- **How it's similar:** In a PPL, you could explicitly model the scenario like this (pseudocode):

  ```
  // Define a random boolean for presence, drawn from a Bernoulli distribution
  is_present = sample("is_present", Bernoulli(probability=0.7))

  if is_present:
      // If present, define the value as a random variable from a Normal distribution
      value = sample("value", Normal(mean=10.0, std_dev=2.0))
      return value
  else:
      // If not present, return a 'None' or 'Missing' type
      return None
  ```

  When you run inference on this model, you are sampling from a distribution of `Option<f64>`, which is exactly what
  `MaybeUncertain<T>` encapsulates.

- **The difference:** PPLs do this for entire programs, whereas `MaybeUncertain<T>` provides this capability in a
  focused, ergonomic way for individual types within a general-purpose language like Rust.

### 2. Bayesian Statistics and Missing Data Models

In statistics, handling missing data is a classic and complex problem. Simply ignoring missing data or replacing it with
the mean can lead to significant bias.

- **How it's similar:** Advanced Bayesian models treat data missingness not as a certainty, but as a variable to be
  inferred. A model might include a parameter for the probability of a data point being missing, and this probability
  can itself be learned from the data. This is precisely the problem `MaybeUncertain` is designed to solve at the type
  level, by making the probability of presence a first-class citizen.

### 3. Functional Programming and Monads

In functional languages like Haskell, the `Maybe` type (or `Option` in Rust/Scala) is a standard way to handle
computations that might not return a value. It's a container that is either `Some(value)` or `Nothing`.

- **How it's similar:** `MaybeUncertain<T>` can be thought of as a **"Probabilistic Maybe"**.
    - A standard `Option<T>` is a binary choice: the value is either there or it's not.
    - A `MaybeUncertain<T>` represents a *distribution over* `Some(T)` and `None`. Instead of being definitely one or
      the other, it holds the probability of being in either state.
    - The propagation of `None` in arithmetic (`Some(a) + None = None`) is also a classic monadic behavior.

In short, `MaybeUncertain` takes a sophisticated concept from the world of probabilistic modeling and makes it an
ergonomic, type-safe, and composable tool for everyday programming in Rust.

## Discussion

While full-fledged Probabilistic Programming Languages are indeed specialized tools for statisticians and researchers,
the *problems* they solve—reasoning under uncertainty and handling incomplete information—are becoming increasingly
mainstream. This is precisely why integrating a concept like `MaybeUncertain` directly into a general-purpose language
like Rust is so powerful. It extracts one of the most useful patterns from the "niche" PPL world and makes it available
in a practical, ergonomic way, without forcing developers to switch to a completely different paradigm.