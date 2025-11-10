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
///
/// # Hybrid Dispatch and Extensibility
///
/// The causal engine is designed to be both fast for common types and extensible
/// for user-defined types. This is achieved through a hybrid dispatch system
/// centered on the `EffectValue` enum.
///
/// ## Static Dispatch for Built-in Types
///
/// For core data types like `bool`, `f64`, `CausalTensor`, etc., `EffectValue`
/// provides specific enum variants (e.g., `EffectValue::Deterministic(bool)`).
/// When the engine processes these variants, it benefits from fast, compile-time
/// static dispatch. There is no runtime overhead, as the compiler knows the
/// exact type and memory layout.
///
/// ## Dynamic Dispatch for External Types
///
/// To allow for user-defined types, `EffectValue` has an `External(Box<dyn PropagatingValue>)`
/// variant. Any custom type that implements the `PropagatingValue` trait can be
/// wrapped in this variant.
///
/// When the engine encounters an `External` variant, it uses dynamic dispatch
/// via trait objects. This provides great flexibility but comes with a minor
/// performance cost compared to static dispatch:
///
/// *   **Heap Allocation**: Wrapping the type in a `Box` requires heap allocation.
/// *   **VTable Lookup**: Calling methods on the `dyn PropagatingValue` trait object
///     involves an indirect function call through a virtual table.
/// *   **Runtime Casting**: Retrieving the concrete type requires a safe, but
///     runtime-checked, downcast.
///
/// This trade-off allows the system to maintain high performance for its core
/// operations while providing a powerful mechanism for extension.
///
/// # Implementing `IntoEffectValue` for Custom Types
///
/// To make your own type compatible with the causal engine, you must implement
/// the `IntoEffectValue` trait for it. Your type must also derive `Debug`, `Clone`,
/// and `PartialEq`. The `PropagatingValue` trait will be implemented automatically
/// for any type that meets these bounds.
///
/// Here is a template for implementing `IntoEffectValue`:
///
/// ```rust,ignore
/// use deep_causality::{CausalityError, EffectValue, IntoEffectValue, PropagatingValue};
///
/// // Your custom struct.
/// #[derive(Debug, Clone, PartialEq)]
/// pub struct MyCustomType {
///     pub value: i32,
///     pub description: String,
/// }
///
/// // Implementation to allow it to be used in the causal engine.
/// impl IntoEffectValue for MyCustomType {
///     fn into_effect_value(self) -> EffectValue {
///         // Wrap the custom type in the External variant.
///         EffectValue::External(Box::new(self))
///     }
///
///     fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError> {
///         // Use the helper function to safely downcast from an EffectValue.
///         EffectValue::try_from_effect_value::<Self>(&ev)
///             .map(|v| v.clone())
///             .ok_or_else(|| {
///                 CausalityError(format!(
///                     "Failed to convert EffectValue to MyCustomType. Found wrong type: {:?}",
///                     ev
///                 ))
///             })
///     }
/// }
/// ```
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
