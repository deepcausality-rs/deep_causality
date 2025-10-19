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
pub(crate) mod hkt_option_ext;
pub(crate) mod hkt_result_ext;
pub(crate) mod hkt_vec_ext;
