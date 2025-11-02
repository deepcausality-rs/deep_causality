# Quickstart: HKT Integration for Uncertain and MaybeUncertain Types

This quickstart guide outlines the key acceptance scenarios for the HKT integration with `Uncertain<T>` and `MaybeUncertain<T>`.

## Acceptance Scenarios

1.  **Functor Transformation:**
    *   **Given** `Uncertain<T>` and `MaybeUncertain<T>` types.
    *   **When** a developer applies a transformation function using `fmap` (Functor).
    *   **Then** the inner value is transformed while preserving its uncertain context, and the result is a new `Uncertain<T>` or `MaybeUncertain<T>`.

2.  **Applicative Combination:**
    *   **Given** multiple `Uncertain<T>` or `MaybeUncertain<T>` values.
    *   **When** a developer combines them using `apply` (Applicative).
    *   **Then** the computations are combined in a structured way, correctly propagating uncertainty and potential absence.

3.  **Monadic Chaining:**
    *   **Given** a sequence of dependent uncertain computations.
    *   **When** a developer chains them using `bind` (Monad).
    *   **Then** the outcome of one uncertain step correctly influences the definition of the next.

4.  **Foldable Restriction:**
    *   **Given** `Uncertain<T>` or `MaybeUncertain<T>` types.
    *   **When** a developer attempts to use a `Foldable` operation.
    *   **Then** the operation is not available, preventing semantically inappropriate usage.