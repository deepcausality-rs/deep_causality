/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DeepCausality is a hyper-geometric computational causality library
//! that enables fast and deterministic context-aware causal reasoning over complex causality models.
//!
//! Why DeepCausality?
//! * DeepCausality is written in Rust with production-grade safety, reliability, and performance in mind.
//! * DeepCausality provides recursive causal data structures that concisely express arbitrary complex causal structures.
//! * DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored within (multiple) context-hyper-graphs.
//! * DeepCausality simplifies modeling of complex tempo-spatial patterns.
//! * DeepCausality comes with Causal State Machine (CSM)
//!
pub mod errors;
pub mod extensions;
pub(crate) mod macros;
pub mod traits;
pub mod types;
pub mod utils;
pub mod utils_test;

//
// Error types
//
pub use crate::errors::*;
//
// Traits
//
// Adjustable Traits
pub use crate::traits::adjustable::Adjustable;
// Assumeable Traits
pub use crate::traits::assumable::Assumable;
pub use crate::traits::assumable::AssumableReasoning;
// Causable Traits
pub use crate::traits::causable::Causable;
pub use traits::causable::causable_reasoning::CausableReasoning;
// Causable Graph Traits
pub use crate::traits::causable_graph::graph::CausableGraph;
pub use crate::traits::causable_graph::graph_explaining::CausableGraphExplaining;
pub use crate::traits::causable_graph::graph_reasoning::CausableGraphReasoning;
pub use crate::traits::causable_graph::*;
// contextuable Traits
pub use crate::traits::contextuable::Contextuable;
pub use crate::traits::contextuable::coordinate::Coordinate;
pub use crate::traits::contextuable::datable::Datable;
pub use crate::traits::contextuable::metric::Metric;
pub use crate::traits::contextuable::metric_coordinate::MetricCoordinate;
pub use crate::traits::contextuable::metric_tensor::MetricTensor4D;
pub use crate::traits::contextuable::space_temporal::SpaceTemporal;
pub use crate::traits::contextuable::space_temporal::SpaceTemporalInterval;
pub use crate::traits::contextuable::spatial::Spatial;
pub use crate::traits::contextuable::symbolic::Symbolic;
pub use crate::traits::contextuable::temporal::Temporal;
// Contextuable Graph Traits
pub use crate::traits::contextuable_graph::ContextuableGraph;
pub use crate::traits::contextuable_graph::ExtendableContextuableGraph;
// Generatable traits
pub use crate::traits::generatable::Generatable;
pub use crate::traits::generatable::generative_processor::GenerativeProcessor;
// Identifiable Traits
pub use crate::traits::identifiable::Identifiable;
// Indexable Traits
pub use crate::traits::indexable::time_index_current::CurrentTimeIndex;
pub use crate::traits::indexable::time_index_previous::PreviousTimeIndex;
pub use crate::traits::indexable::time_indexable::TimeIndexable;
// Inferable Traits
pub use crate::traits::inferable::Inferable;
pub use crate::traits::inferable::InferableReasoning;
// Observable Traits
pub use crate::traits::observable::Observable;
pub use crate::traits::observable::ObservableReasoning;
// Scalar Traits
pub use crate::traits::scalar::scalar_projector::ScalarProjector;
pub use crate::traits::scalar::scalar_value::ScalarValue;
//
// Types
//
// Alias types
pub use crate::types::alias_types::alias_base::*;
pub use crate::types::alias_types::alias_function::*;
pub use crate::types::alias_types::alias_lock::*;
pub use crate::types::alias_types::alias_primitives::*;
pub use crate::types::alias_types::alias_uniform::*;
pub use crate::types::alias_types::*;
// Causal types
pub use crate::types::causal_types::causal_type::CausaloidType;
pub use crate::types::causal_types::causaloid::Causaloid;
pub use crate::types::causal_types::causaloid_graph::CausaloidGraph;
pub use crate::types::causal_types::*;
// Context types
pub use crate::types::context_types::context_graph;
pub use crate::types::context_types::context_graph::Context;
pub use crate::types::context_types::contextoid::contextoid_type::*;
pub use crate::types::context_types::contextoid::*;
// Default context node types.
pub use crate::types::context_node_types::data::Data;
pub use crate::types::context_node_types::root::Root;
// Space context node types.
pub use crate::types::context_node_types::space::ecef_space::EcefSpace;
pub use crate::types::context_node_types::space::euclidean_space::EuclideanSpace;
pub use crate::types::context_node_types::space::geo_space::GeoSpace;
pub use crate::types::context_node_types::space::ned_space::NedSpace;
pub use crate::types::context_node_types::space::quaternion_space::QuaternionSpace;
pub use crate::types::context_node_types::space::space_kind::SpaceKind;
// Space time context node types.
pub use crate::types::context_node_types::space_time::euclidean_spacetime::EuclideanSpacetime;
pub use crate::types::context_node_types::space_time::lorentzian_spacetime::LorentzianSpacetime;
pub use crate::types::context_node_types::space_time::minkowski_spacetime::MinkowskiSpacetime;
pub use crate::types::context_node_types::space_time::space_time_kind::SpaceTimeKind;
pub use crate::types::context_node_types::space_time::tangent_spacetime::TangentSpacetime;
// Symbolic context node types.
pub use crate::types::context_node_types::symbol::base_symbol::BaseSymbol;
// pub use crate::types::context_types::node_types::symbol::symbol_kind
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
// Other context types
pub use crate::types::context_types::relation_kind::*;
pub use crate::types::context_types::time_scale::TimeScale;
// CSM types
pub use crate::types::csm_types::CSM;
pub use crate::types::csm_types::csm_action::CausalAction;
pub use crate::types::csm_types::csm_state::CausalState;
// Generative types
pub use crate::types::generative_types::generative_output::GenerativeOutput;
pub use crate::types::generative_types::generative_trigger::GenerativeTrigger;
// Model types
pub use crate::types::model_types::assumption::Assumption;
pub use crate::types::model_types::inference::Inference;
pub use crate::types::model_types::model::Model;
pub use crate::types::model_types::observation::Observation;
// Reasoning types
pub use crate::types::reasoning_types::aggregate_logic::AggregateLogic;
pub use crate::types::reasoning_types::propagating_effect::PropagatingEffect;
//
//Symbolic types
pub use crate::types::symbolic_types::symbolic_representation::SymbolicRepresentation;
pub use crate::types::symbolic_types::symbolic_result::SymbolicResult;
// Utils
//
pub use crate::utils::time_utils::*;
