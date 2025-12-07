/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the core behavior of `Causaloid` instances within the DeepCausality framework,
//! specifically how they implement the `Causable` and `MonadicCausable` traits.
//!
//! It details the evaluation logic for different types of `Causaloid`s (Singleton, Collection, Graph),
//! ensuring proper error propagation and comprehensive log provenance through monadic operations.

use crate::types::causal_types::causaloid::causable_utils;
use crate::{
    Causable, CausalityError, Causaloid, CausaloidType, MonadicCausable, PropagatingEffect,
};
use std::fmt::Debug;

/// Implements the `Causable` trait for `Causaloid`.
///
/// This trait provides fundamental properties and methods for any entity that can
/// participate in a causal relationship. For `Causaloid`, it primarily defines
/// how to determine if a causaloid represents a single, atomic causal unit.
impl<I, O, PS, C> Causable for Causaloid<I, O, PS, C>
where
    I: Default,
    O: Default + Debug,
    PS: Default + Clone,
    C: Clone,
{
    /// Checks if the `Causaloid` is of type `Singleton`.
    ///
    /// A singleton causaloid represents an atomic causal relationship that
    /// can be evaluated independently.
    ///
    /// # Returns
    /// `true` if the `CausaloidType` is `Singleton`, `false` otherwise.
    fn is_singleton(&self) -> bool {
        matches!(self.causal_type, CausaloidType::Singleton)
    }
}

/// Implements the `MonadicCausable` trait for `Causaloid`.
///
/// This implementation provides the core evaluation logic for `Causaloid`s,
/// leveraging monadic principles to handle the flow of effects, errors, and logs.
///
/// **Note**: This base implementation only supports `CausaloidType::Singleton`.
/// For `Collection` and `Graph` evaluation with aggregation support, use the
/// specialized constructors that ensure proper trait bounds are met.
#[allow(clippy::type_complexity)]
impl<I, O, PS, C> MonadicCausable<I, O> for Causaloid<I, O, PS, C>
where
    I: Default + Clone + Send + Sync + 'static,
    O: Default + Debug + Clone + Send + Sync + 'static,
    PS: Default + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Evaluates the causal effect of this `Causaloid` given an `incoming_effect`.
    ///
    /// The evaluation process is monadic, ensuring that errors are propagated
    /// and a comprehensive log of operations is maintained.
    ///
    /// **Important**: This base implementation only supports `Singleton` causaloids.
    /// For `Collection` and `Graph` types, specialized evaluation methods with
    /// proper trait bounds should be used.
    ///
    /// # Arguments
    /// * `incoming_effect` - The `PropagatingEffect` representing the input to this causaloid.
    ///
    /// # Returns
    /// A `PropagatingEffect` containing the result of the causal evaluation,
    /// any errors encountered, and a complete log of the operations performed.
    fn evaluate(&self, incoming_effect: &PropagatingEffect<I>) -> PropagatingEffect<O> {
        match self.causal_type {
            CausaloidType::Singleton => {
                // For a Singleton, the evaluation is a monadic chain of operations:
                // 1. Execute the causal logic with the input.
                // The `bind` operations ensure that logs are aggregated and errors short-circuit.
                incoming_effect
                    .clone()
                    .bind(|effect_value, _, _| match effect_value.into_value() {
                        Some(input) => causable_utils::execute_causal_logic(input, self),
                        None => PropagatingEffect::from_error(CausalityError(
                            deep_causality_core::CausalityErrorEnum::Custom(
                                "Cannot evaluate: input value is None".into(),
                            ),
                        )),
                    })
            }

            CausaloidType::Collection => {
                // Collection evaluation requires O: Aggregatable + Send + Sync + Debug + 'static.
                // This base implementation returns an error; use specialized methods for collections.
                PropagatingEffect::from_error(CausalityError(
                    deep_causality_core::CausalityErrorEnum::Custom(
                        "Collection evaluation requires specialized trait bounds (Aggregatable). \
                         Use evaluate_collection method on the causal_coll directly."
                            .into(),
                    ),
                ))
            }
            CausaloidType::Graph => {
                // Graph evaluation requires specialized trait bounds.
                // This base implementation returns an error; use specialized methods for graphs.
                PropagatingEffect::from_error(CausalityError(
                    deep_causality_core::CausalityErrorEnum::Custom(
                        "Graph evaluation requires specialized trait bounds. \
                         Use evaluate_subgraph_from_cause on the causal_graph directly."
                            .into(),
                    ),
                ))
            }
        }
    }
}
