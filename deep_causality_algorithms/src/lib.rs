/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod causal_discovery;
#[cfg(feature = "topology")]
pub mod dag_sampling;
pub mod feature_selection;

#[cfg(feature = "topology")]
pub use causal_discovery::brcd;
pub use causal_discovery::surd;
pub use feature_selection::mrmr;
