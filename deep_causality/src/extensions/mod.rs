/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod assumable;
pub mod causable;
pub mod inferable;
pub mod observable;
// //! # Type Extensions for Standard Collections
// //!
// //! This module provides local trait implementations—referred to as *type extensions*—for external types,
// //! specifically Rust standard library collections. These implementations enable advanced reasoning capabilities
// //! on common data structures.
// //!
// //! The following reasoning traits are implemented:
// //!
// //! - [`AssumableReasoning`]
// //! - [`CausableReasoning`]
// //! - [`InferableReasoning`]
// //! - [`ObservableReasoning`]
// //!
// //! Each trait offers a significant default implementation. When the trait is imported, the compiler
// //! automatically inserts its default implementation into the associated type extension.
// //! All traits and their default behaviors are defined in the `protocols` module.
// //!
// //! ## Implemented Collections
// //!
// //! Since Rust lacks a unified collection abstraction, each collection type requires its own extension.
// //! Type extensions are currently implemented for the following standard collections:
// //!
// //! - [`[T; N]`](https://doc.rust-lang.org/std/primitive.array.html) — Fixed-size arrays
// //! - [`Vec<T>`](https://doc.rust-lang.org/std/vec/struct.Vec.html) — Growable vectors
// //! - [`VecDeque<T>`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) — Double-ended queues
// //! - [`HashMap<K, V>`](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html) — Hash maps
// //! - [`BTreeMap<K, V>`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) — Sorted maps
// //!
// //! ## Unimplemented Collections
// //!
// //! The following collections are *not* implemented due to the need for additional trait bounds such as `Eq`, `Hash`, etc.,
// //! and lack of a clear use case in the current context:
// //!
// //! - `HashSet`
// //! - `BTreeSet`
// //! - `LinkedList`
// //!
// //! `Vec<T>` and `VecDeque<T>` already cover most practical use cases effectively.
// //!
// //! ## Macros and Implementation Details
// //!
// //! Rust's default trait implementations can only rely on methods defined within the trait itself.
// //! As such, several helper methods—such as `len`, `is_empty`, and `get_all_items`—are implemented explicitly
// //! within each extension. These are mostly trivial and generated via compiler macros defined in the
// //! [`deep_causality_macros`](https://docs.rs/deep_causality_macros) crate.
// //!
// //! ## Further Reading
// //!
// //! - [Extension Traits in Rust](http://xion.io/post/code/rust-extension-traits.html)
