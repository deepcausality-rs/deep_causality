/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::reasoning_types::propagating_effect::PropagatingEffect;
use crate::{
    Causable, CausalMonad, CausalityError, Causaloid, CausaloidId, Datable, IntoEffectValue,
    MonadicCausable, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Display;

/// A centralized, type-erased storage for all `Causaloid` instances.
///
/// It uses `TypeId` to store homogeneous collections of `Causaloid<I, O>` and
/// provides a hybrid static/dynamic dispatch mechanism for evaluation.
#[derive(Default)]
pub struct CausaloidRegistry {
    /// Stores `Vec<Box<dyn MonadicCausable<CausalMonad> + Send + Sync>>` for different `Causaloid` types, indexed by `TypeId`.
    storage: HashMap<TypeId, Vec<Box<dyn MonadicCausable<CausalMonad> + Send + Sync>>>,
    /// Maps a stable `CausaloidId` (u64) to its `TypeId` and index within its typed vector.
    lookup: HashMap<CausaloidId, (TypeId, usize)>,
    /// Counter for generating unique IDs.
    next_id: CausaloidId,
}

impl CausaloidRegistry {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            lookup: HashMap::new(),
            next_id: 0,
        }
    }

    /// Registers a `Causaloid` with the registry, assigning it a unique `CausaloidId`.
    ///
    /// The `Causaloid` is stored in a type-specific vector within the registry's storage.
    ///
    /// # Arguments
    ///
    /// * `causaloid` - The `Causaloid` instance to register.
    ///
    /// # Returns
    ///
    /// The unique `CausaloidId` assigned to the registered `Causaloid`.
    #[allow(clippy::type_complexity)]
    pub fn register<I, O, D, S, T, ST, SYM, VS, VT>(
        &mut self,
        causaloid: Causaloid<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> CausaloidId
    where
        I: IntoEffectValue,
        O: IntoEffectValue,
        D: Datable + Clone,
        S: Spatial<VS> + Clone,
        T: Temporal<VT> + Clone,
        ST: SpaceTemporal<VS, VT> + Clone,
        SYM: Symbolic + Clone,
        VS: Clone,
        VT: Clone,
        Causaloid<I, O, D, S, T, ST, SYM, VS, VT>:
            MonadicCausable<CausalMonad> + Causable + Display + Send + Sync + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        let type_id = TypeId::of::<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>();

        let entry = self.storage.entry(type_id).or_default();

        let index = entry.len();
        entry.push(Box::new(causaloid));

        self.lookup.insert(id, (type_id, index));
        id
    }

    /// Evaluates a `Causaloid` identified by its `CausaloidId` with a given `PropagatingEffect`.
    ///
    /// This method performs a `TypeId` lookup and then dispatches the `evaluate` call
    /// to the correct `Causaloid` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The `CausaloidId` of the `Causaloid` to evaluate.
    /// * `effect` - The `PropagatingEffect` to pass as input to the `Causaloid`.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` representing the output of the `Causaloid`'s evaluation.
    /// Returns an error if the `CausaloidId` is not found or if there's a type mismatch.
    pub fn evaluate(&self, id: CausaloidId, effect: &PropagatingEffect) -> PropagatingEffect {
        let (type_id, index) = match self.lookup.get(&id) {
            Some(info) => info,
            None => {
                return PropagatingEffect::from_error(CausalityError(format!(
                    "Causaloid with ID {} not found in registry.",
                    id
                )));
            }
        };

        if let Some(typed_vec) = self.storage.get(type_id) {
            if let Some(causaloid_trait_object) = typed_vec.get(*index) {
                causaloid_trait_object.evaluate(self, effect)
            } else {
                PropagatingEffect::from_error(CausalityError(format!(
                    "Causaloid with ID {} found in lookup, but index {} is out of bounds for its type vector.",
                    id, index
                )))
            }
        } else {
            PropagatingEffect::from_error(CausalityError(format!(
                "Causaloid with TypeId {:?} not found in registry storage.",
                type_id
            )))
        }
    }
}
