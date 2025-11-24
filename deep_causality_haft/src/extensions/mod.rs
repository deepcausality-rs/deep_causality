//! This module provides Higher-Kinded Type (HKT) witness implementations
//! for common Rust types such as `Option`, `Result`, and `Vec`.
//!
//! These implementations allow these standard library types to be used with
//! the generic functional programming traits (Functor, Applicative, Monad, Foldable)
//! defined in the `deep_causality_haft` crate.
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub mod func_fold_b_tree_map_ext;
pub mod func_fold_hash_map_ext;
pub mod func_fold_vec_deque_ext;
pub mod hkt_box_ext;
pub mod hkt_linked_list_ext;
pub mod hkt_option_ext;
pub mod hkt_result_ext;
pub mod hkt_tuple_ext;
pub mod hkt_vec_ext;
