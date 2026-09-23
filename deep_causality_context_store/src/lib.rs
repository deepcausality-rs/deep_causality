/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The persistence contract for the DeepCausality context.
//!
//! A `Context` in `deep_causality_context` lives in memory. This crate declares what a store
//! holds to keep one: the records a context projects onto ([`ContextoidRecord`],
//! [`RelationRecord`], [`ContextSnapshot`] and the [`NodeRecord`] tree under them), the
//! [`ContextStorage`] trait a backend implements, the [`Recordable`] projection the context crate
//! implements beside its node types, the [`Substrate`] a reference-holding store keeps values in,
//! and the [`ContextStorageStream`] a backend adds when it reports changes. Every scalar in a
//! record is `f64` and every tick `u64`; every identifier is a `u64` the store hands out. The
//! crate depends on nothing, so a backend links it alone. [`utils_test`] holds an in-memory
//! backend for tests on both sides of the contract.
//!
//! See <https://docs.deepcausality.com/> for the documentation.

mod alias;
mod constants;
mod errors;
mod traits;
mod types;
pub mod utils_test;

//
// Aliases and constants
//
pub use crate::alias::*;
pub use crate::constants::*;
//
// Error types
//
pub use crate::errors::*;
//
// Traits
//
pub use crate::traits::context_events::{ContextEventItem, ContextEvents};
pub use crate::traits::context_storage::ContextStorage;
pub use crate::traits::context_storage_stream::ContextStorageStream;
pub use crate::traits::recordable::Recordable;
pub use crate::traits::substrate::Substrate;
//
// Records
//
pub use crate::types::context_event::ContextEvent;
pub use crate::types::id_reserve::IdReserve;
pub use crate::types::records::context_record::ContextRecord;
pub use crate::types::records::context_snapshot::ContextSnapshot;
pub use crate::types::records::contextoid_record::ContextoidRecord;
pub use crate::types::records::data_record::DataRecord;
pub use crate::types::records::extra_context_snapshot::ExtraContextSnapshot;
pub use crate::types::records::node_record::NodeRecord;
pub use crate::types::records::relation_record::RelationRecord;
pub use crate::types::records::space_record::SpaceRecord;
pub use crate::types::records::space_time_record::SpaceTimeRecord;
pub use crate::types::records::time_record::TimeRecord;
//
// Vocabulary shared with the context node types
//
pub use crate::types::relation_kind::RelationKind;
pub use crate::types::substrate_ref::SubstrateRef;
pub use crate::types::time_scale::TimeScale;
pub use crate::types::vertical_datum::VerticalDatum;
