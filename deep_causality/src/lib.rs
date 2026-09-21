/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DeepCausality is a Rust framework for dynamic causality. It enables fast, deterministic,
//! context-aware causal reasoning over complex multi-stage causal models, and stays correct
//! when both the data and the rules evolve at runtime.
//!
//! Why DeepCausality?
//! * Written in Rust with production-grade safety, reliability, and performance in mind.
//! * Provides recursive causal data structures that concisely express arbitrary complex causal structures.
//! * Threads context awareness across data-like, time-like, space-like, and spacetime-like entities stored within one or more context hypergraphs.
//! * Models complex tempo-spatial patterns through the Causal Monad and the Effect Propagation Process.
//! * Ships with a Causal State Machine (CSM) for pairing inference with action.
//!
//! See <https://docs.deepcausality.com/> for the documentation.
//!
mod alias;
mod errors;
mod extensions;
mod traits;
mod types;
mod utils;
pub mod utils_test;

// Re-Export Core Types
pub use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalFlow, CausalMonad, CausalityError,
    CausalityErrorEnum, EffectLog, Identifiable, PropagatingEffect, PropagatingProcess,
};
// The shared primitive aliases. Core is their single declaration site; `TeloidTag` and `TeloidID`
// are deliberately not re-exported, because nothing here uses them and `deep_causality_ethos`
// declares its own pair.
pub use deep_causality_core::{
    CausaloidId, DescriptionValue, FloatType, IdentificationValue, NumberType, NumericalValue,
};

pub use deep_causality_haft::{LogAddEntry, LogAppend, LogEffect, LogSize};

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
// Assumeable Traits
pub use crate::traits::assumable::Assumable;
pub use crate::traits::assumable::AssumableReasoning;
// Causable Traits
pub use crate::traits::causable::stateful::StatefulMonadicCausable;
pub use crate::traits::causable::{Causable, MonadicCausable};
// Causable Graph Traits
pub use crate::traits::causable_graph::graph::CausableGraph;
pub use crate::traits::causable_graph::graph_reasoning::MonadicCausableGraphReasoning;
pub use crate::traits::causable_graph::graph_reasoning::stateful::StatefulMonadicCausableGraphReasoning;
pub use crate::traits::causable_graph::*;
// CSM traits
pub use crate::traits::csm_evaluable::CsmEvaluable;
// Inferable Traits
pub use crate::traits::inferable::Inferable;
pub use crate::traits::inferable::InferableReasoning;
// Observable Traits
pub use crate::traits::observable::Observable;
pub use crate::traits::observable::ObservableReasoning;
// Transferable Trait
pub use crate::traits::transferable::Transferable;
//
// Types
//
// Causal types
pub use crate::types::causal_types::aggregate_logic::AggregateLogic;
pub use crate::types::causal_types::causal_type::CausaloidType;
pub use crate::types::causal_types::causaloid::Causaloid;
pub use crate::types::causal_types::causaloid_graph::CausaloidGraph;
pub use crate::types::causal_types::causaloid_graph::lambda_edges::{EdgeLambdaFn, LambdaEdges};
pub use crate::types::causal_types::*;
// CSM types
pub use crate::types::csm_types::csm::CSM;
pub use crate::types::csm_types::csm_action::CausalAction;
pub use crate::types::csm_types::csm_parameter::action_parameter_value::ActionParameterValue;
pub use crate::types::csm_types::csm_parameter::proposed_action::ProposedAction;
pub use crate::types::csm_types::csm_parameter::uncertain_parameter::UncertainParameter;
pub use crate::types::csm_types::csm_state::CausalState;
// Generative types
pub use crate::types::generative_types::effect_system::{
    AuditableGraphGenerator, GraphGeneratableEffect, GraphGeneratableEffectSystem,
    GraphGeneratableEffectWitness,
};
pub use crate::types::generative_types::interpreter::{CausalSystemState, Interpreter};
pub use crate::types::generative_types::modification_log::{
    ModificationLog, ModificationLogEntry, OpStatus,
};
pub use crate::types::generative_types::operation::{OpTree, Operation};
// Model types
pub use crate::types::model_types::assumption::Assumption;
pub use crate::types::model_types::inference::Inference;
pub use crate::types::model_types::model::Model;
pub use crate::types::model_types::observation::Observation;

//
//

// Utils
//
pub use crate::utils::math_utils;
pub use crate::utils::monadic_collection_utils;
pub use crate::utils::monadic_collection_utils::Aggregatable;
// The collection-aggregation carrier algebra (the `Aggregatable: Verdict` bound;
// `core.verdict.closure`).
pub use crate::utils::time_utils;
pub use deep_causality_algebra::Verdict;
// Causable Collection Traits
pub use traits::causable_collection::collection_accessor::CausableCollectionAccessor;
pub use traits::causable_collection::collection_reasoning::monadic_collection::MonadicCausableCollection;
pub use traits::causable_collection::collection_reasoning::stateful_monadic_collection::StatefulMonadicCausableCollection;

// Uncertainty types
pub use deep_causality_uncertain::Uncertain;
