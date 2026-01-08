/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
pub use crate::alias::*;

// Re-export error types
pub use crate::errors::causality_error::{CausalityError, CausalityErrorEnum};

// Re-export traits
pub use crate::traits::control_flow_protocol::{ControlFlowProtocol, FromProtocol, ToProtocol};
pub use crate::traits::intervenable::Intervenable;

// Re-export types
// Builder
pub use crate::types::builder::control_flow_builder::ControlFlowBuilder;
pub use crate::types::builder::executable_edge::ExecutableEdge;
pub use crate::types::builder::executable_graph::ExecutableGraph;
pub use crate::types::builder::executable_node::ExecutableNode;
pub use crate::types::builder::executable_node_type::NodeType;
pub use errors::graph_error::GraphError;
//
pub use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;
pub use crate::types::causal_effect_propagation_process::hkt::CausalEffectPropagationProcessWitness;
pub use crate::types::causal_effect_system::CausalEffectSystem;
pub use crate::types::causal_monad::CausalMonad;
pub use crate::types::effect_log::log_effect::EffectLog;
pub use crate::types::effect_value::EffectValue;
pub use crate::types::propagating_effect::PropagatingEffect;
pub use crate::types::propagating_effect::hkt::PropagatingEffectWitness;
pub use crate::types::propagating_process::PropagatingProcess;
pub use crate::types::propagating_process::hkt::PropagatingProcessWitness;
