/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The persistence contract for the DeepCausality context.
//!
//! A `Context` in `deep_causality_context` lives in memory. This crate declares what a store has
//! to hold to keep one: the record vocabulary a context projects onto, the storage trait a backend
//! implements, and the vocabulary types both sides name. It depends on nothing, so a backend links
//! this crate alone.
//!
//! See <https://docs.deepcausality.com/> for the documentation.

mod alias;
mod constants;
mod errors;
mod traits;
mod types;
pub mod utils_test;

//
// Vocabulary shared with the context node types
//
pub use crate::types::relation_kind::RelationKind;
pub use crate::types::substrate_ref::SubstrateRef;
pub use crate::types::time_scale::TimeScale;
pub use crate::types::vertical_datum::VerticalDatum;
