/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The context layer of DeepCausality.
//!
//! A causal model reasons about something, and that something is the context: an explicit, typed
//! environment that a causaloid or a causal-monad chain reads while it runs. This crate owns that
//! environment — the [`Context`] hypergraph, the [`Contextoid`] nodes it holds, the node types for
//! data, space, time, spacetime and symbols, and the traits that describe them.
//!
//! Context is a separate crate so that it is an opt-in dependency. Reasoning without context is the
//! default case and costs nothing; a model that needs context declares this crate and imports from
//! it, which keeps a dependency search honest about where context is used.
//!
//! Because this crate sits beside `deep_causality` rather than above it, a consumer of the causal
//! monad can carry a typed context without depending on the causaloid stack:
//!
//! ```
//! use deep_causality_context::BaseContext;
//! use deep_causality_core::{PropagatingEffect, PropagatingProcess};
//!
//! let ctx = BaseContext::with_capacity(1, "demo", 10);
//! let process: PropagatingProcess<f64, (), BaseContext> =
//!     PropagatingProcess::with_state(PropagatingEffect::pure(1.0), (), Some(ctx));
//! ```
//!
//! See <https://docs.deepcausality.com/> for the documentation.

mod alias;
mod errors;
mod traits;
mod types;
pub mod utils_test;

//
// Aliases
//
pub use crate::alias::*;
//
// Error types
//
pub use crate::errors::*;
//
// Traits
//
// Adjustable Traits
pub use crate::traits::adjustable::{Adjustable, UncertainAdjustable};
// Contextuable Traits
pub use crate::traits::contextuable::Contextuable;
pub use crate::traits::contextuable::coordinate::Coordinate;
pub use crate::traits::contextuable::datable::Datable;
pub use crate::traits::contextuable::datable_uncertain::UncertainDatable;
pub use crate::traits::contextuable::distance::Distance;
pub use crate::traits::contextuable::metric_signature::MetricSignature;
pub use crate::traits::contextuable::metric_tensor::MetricTensor4D;
pub use crate::traits::contextuable::space_temporal::SpaceTemporal;
pub use crate::traits::contextuable::space_temporal::SpaceTemporalInterval;
pub use crate::traits::contextuable::spatial::Spatial;
pub use crate::traits::contextuable::temporal::Temporal;
// Contextuable Graph Traits
pub use crate::traits::contextuable_graph::ContextuableGraph;
pub use crate::traits::contextuable_graph::ExtendableContextuableGraph;
// Indexable Traits
pub use crate::traits::indexable::data_index_current::CurrentDataIndex;
pub use crate::traits::indexable::data_index_previous::PreviousDataIndex;
pub use crate::traits::indexable::data_indexable::DataIndexable;
pub use crate::traits::indexable::time_index_current::CurrentTimeIndex;
pub use crate::traits::indexable::time_index_previous::PreviousTimeIndex;
pub use crate::traits::indexable::time_indexable::TimeIndexable;
// Scalar traits
pub use crate::traits::scalar::scalar_projector::ScalarProjector;
pub use crate::traits::scalar::scalar_value::ScalarValue;
//
// Types
//
// Default context node types.
pub use crate::traits::context_frame::ContextFrame;
pub use crate::types::context_frames::{BaseFrame, ClockFrame, UniformFrame};
pub use crate::types::context_node_types::data::Data;
pub use crate::types::context_node_types::data_uncertain::uncertain_bool_data::UncertainBoolData;
pub use crate::types::context_node_types::data_uncertain::uncertain_data::UncertainData;
pub use crate::types::context_node_types::no_space_time::NoSpaceTime;
pub use crate::types::context_node_types::root::Root;
// Space context node types.
pub use crate::types::context_node_types::space::ecef_space::EcefSpace;
pub use crate::types::context_node_types::space::euclidean_space::EuclideanSpace;
pub use crate::types::context_node_types::space::geo_space::GeoSpace;
pub use crate::types::context_node_types::space::ned_space::NedSpace;
pub use crate::types::context_node_types::space::space_kind::SpaceKind;
// Space time context node types.
pub use crate::types::context_node_types::space_time::euclidean_spacetime::EuclideanSpacetime;
pub use crate::types::context_node_types::space_time::lorentzian_spacetime::LorentzianSpacetime;
pub use crate::types::context_node_types::space_time::space_time_kind::SpaceTimeKind;
pub use crate::types::context_node_types::space_time::tangent_spacetime::TangentSpacetime;
// Symbolic spacetime context node types.
pub use crate::types::context_node_types::symbol_spacetime::causal_set_spacetime::CausalSetSpacetime;
pub use crate::types::context_node_types::symbol_spacetime::conformal_spacetime::ConformalSpacetime;
// Time context node types.
pub use crate::types::context_node_types::time::discrete_time::DiscreteTime;
pub use crate::types::context_node_types::time::entropic_time::EntropicTime;
pub use crate::types::context_node_types::time::euclidean_time::EuclideanTime;
pub use crate::types::context_node_types::time::lorentzian_time::LorentzianTime;
pub use crate::types::context_node_types::time::symbolic_time::{SymbolicTime, SymbolicTimeUnit};
pub use crate::types::context_node_types::time::time_kind::TimeKind;
// Context types
pub use crate::types::context_types::context_graph;
pub use crate::types::context_types::context_graph::Context;
pub use crate::types::context_types::contextoid::contextoid_type::*;
pub use crate::types::context_types::contextoid::*;
// Other context types
pub use crate::types::context_types::relation_kind::*;
pub use crate::types::context_types::time_scale::TimeScale;
pub use crate::types::context_types::vertical_datum::VerticalDatum;

// Re-exported so an implementor of a context trait does not need a second import for the identity
// trait every context node carries.
pub use deep_causality_core::Identifiable;
