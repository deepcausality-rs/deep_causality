//! The `deep_causality_haft` crate provides foundational traits and utilities for
//! implementing Higher-Kinded Types (HKTs) and functional programming patterns
//! (Functor, Applicative, Monad, Foldable) in Rust.
//!
//! This crate is a core component of the `deep_causality` project, enabling
//! the construction of flexible and robust type-encoded effect systems.
//!
//! # Core Concepts
//!
//! *   **Higher-Kinded Types (HKTs)**: Abstractions over type constructors (e.g., `Option<T>`, `Vec<T>`).
//!     This allows writing generic code that works across different container types.
//! *   **Functor**: Defines the `fmap` operation for mapping a function over a type constructor.
//! *   **Applicative**: Extends `Functor` with `pure` (to lift values) and `apply` (to apply
//!     functions within a context).
//! *   **Monad**: Provides the `bind` operation for sequencing computations that produce
//!     effectful values.
//! *   **Foldable**: Defines the `fold` operation for reducing a data structure to a single value.
//! *   **Type-Encoded Effect Systems**: A mechanism to explicitly track and manage side-effects
//!     (like errors, logging, counters) using Rust's type system, ensuring compile-time verification.
//!
//! # Modules
//!
//! *   `applicative`: Defines the `Applicative` trait.
//! *   `effect`: Defines traits (`Effect3`, `Effect4`, `Effect5`) for partially applying HKTs
//!     to build type-encoded effect systems.
//! *   `extensions`: Provides concrete HKT witness implementations for standard Rust types
//!     like `Option`, `Result`, and `Vec`.
//! *   `foldable`: Defines the `Foldable` trait.
//! *   `functor`: Defines the `Functor` trait.
//! *   `hkt`: Defines the core `HKT` traits (`HKT`, `HKT2`, `HKT3`, `HKT4`, `HKT5`) and `Placeholder`.
//! *   `monad`: Defines the `Monad` trait.
//! *   `monad_effect`: Defines traits (`MonadEffect3`, `MonadEffect4`, `MonadEffect5`)
//!     for monadic operations within type-encoded effect systems.
//! *   `utils_tests`: Internal utilities and test-specific effect types.
//!
//! # Usage
//!
//! This crate is primarily intended for internal use within the `deep_causality` project
//! to build its core abstractions. However, the traits and concepts can be generally applied
//! to other Rust projects requiring advanced functional programming patterns and effect management.
//!
mod applicative;
mod comonad;
mod effect;
mod extensions;
mod foldable;
mod functor;
mod hkt;
mod monad;
mod monad_effect;
mod traversable;
pub mod utils_tests;

// Functional extensions for std types
/// Re-exports `BTreeMapWitness`, the HKT witness for `BTreeMap<K, V>`.
pub use crate::extensions::func_fold_b_tree_map_ext::BTreeMapWitness;
/// Re-exports `HashMapWitness`, the HKT witness for `HashMap<K, V>`.
pub use crate::extensions::func_fold_hash_map_ext::HashMapWitness;
/// Re-exports `VecDequeWitness`, the HKT witness for `VecDeque<T>`.
pub use crate::extensions::func_fold_vec_deque_ext::VecDequeWitness;
/// Re-exports `BoxWitness`, the HKT witness for `Box<T>`.
pub use crate::extensions::hkt_box_ext::BoxWitness;
/// Re-exports `LinkedListWitness`, the HKT witness for `LinkedList<T>`.
pub use crate::extensions::hkt_linked_list_ext::LinkedListWitness;
/// Re-exports `OptionWitness`, the HKT witness for `Option<T>`.
pub use crate::extensions::hkt_option_ext::OptionWitness;
/// Re-exports `ResultWitness`, the HKT witness for `Result<T, E>`.
pub use crate::extensions::hkt_result_ext::ResultWitness;
/// Re-exports `VecWitness`, the HKT witness for `Vec<T>`.
pub use crate::extensions::hkt_vec_ext::VecWitness;

// Effects for Arity 3 - 5
/// Re-exports the `Effect3` trait for arity-3 type-encoded effect systems.
pub use crate::effect::Effect3;
/// Re-exports the `Effect4` trait for arity-4 type-encoded effect systems.
pub use crate::effect::Effect4;
/// Re-exports the `Effect5` trait for arity-5 type-encoded effect systems.
pub use crate::effect::Effect5;
// Monad Effects
/// Re-exports the `MonadEffect3` trait for monadic operations in arity-3 effect systems.
pub use crate::monad_effect::MonadEffect3;
/// Re-exports the `MonadEffect4` trait for monadic operations in arity-4 effect systems.
pub use crate::monad_effect::MonadEffect4;
/// Re-exports the `MonadEffect5` trait for monadic operations in arity-5 effect systems.
pub use crate::monad_effect::MonadEffect5;

// HKT Trait for Arity 1 - 5
/// Re-exports the core `HKT` trait for arity-1 Higher-Kinded Types.
pub use crate::hkt::HKT;
/// Re-exports the `HKT2` trait for arity-2 Higher-Kinded Types.
pub use crate::hkt::HKT2;
/// Re-exports the `HKT3` trait for arity-3 Higher-Kinded Types.
pub use crate::hkt::HKT3;
/// Re-exports the `HKT4` trait for arity-4 Higher-Kinded Types.
pub use crate::hkt::HKT4;
/// Re-exports the `HKT5` trait for arity-5 Higher-Kinded Types.
pub use crate::hkt::HKT5;
/// Re-exports `Placeholder`, a zero-sized type used in HKT witness implementations.
pub use crate::hkt::Placeholder;

// Functional traits
/// Re-exports the `Applicative` trait for applying functions within a context.
pub use crate::applicative::Applicative;
/// Re-exports the `Comonad` trait
pub use crate::comonad::CoMonad;
/// Re-exports the `Foldable` trait for reducing data structures.
pub use crate::foldable::Foldable;
/// Re-exports the `Functor` trait for mapping over type constructors.
pub use crate::functor::Functor;
/// Re-exports the `Monad` trait for sequencing effectful computations.
pub use crate::monad::Monad;
/// Re-exports the `Traversable` trait to flip generic structures inside out.
pub use crate::traversable::Traversable;
