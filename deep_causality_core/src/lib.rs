/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

mod alias;
mod errors;
mod traits;
mod types;

// Re-export alias types
pub use alias::*;

// Re-export error types
pub use errors::*;

// Re-export traits
pub use traits::intervenable::*;
pub use traits::propagating_value::*;

// Re-export types
pub use types::effect_log::log_effect::EffectLog;
pub use types::effect_log::*;
pub use types::effect_value::*;
pub use types::monad_types::causal_effect_system::CausalEffectSystem;
pub use types::monad_types::causal_monad::CausalMonad;
pub use types::monad_types::*;
pub use types::numeric_value::*;
pub use types::propagating_effect::*;
