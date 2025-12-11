# Data Model: HKT Integration for Uncertain and MaybeUncertain Types

## Entities

- **Uncertain<T>**: Represents a single value `T` with inherent uncertainty, modeled as a probability distribution.
- **MaybeUncertain<T>**: Represents a value that is probabilistically present or absent; if present, its value is `Uncertain<T>`.
- **UncertainWitness**: A zero-sized type that acts as a Higher-Kinded Type witness for `Uncertain<T>`.
- **MaybeUncertainWitness**: A zero-sized type that acts as a Higher-Kinded Type witness for `MaybeUncertain<T>`.
- **HKT Trait**: A trait from `deep_causality_haft` that allows types to be abstracted over their "shape."
- **Functor Trait**: A trait from `deep_causality_haft` for types that can be mapped over.
- **Applicative Trait**: A trait from `deep_causality_haft` for types that can apply functions within their context.
- **Monad Trait**: A trait from `deep_causality_haft` for types that can sequence dependent computations.