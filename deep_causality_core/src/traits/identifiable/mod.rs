/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::IdentificationValue;

/// Trait for types that have a unique identifier.
///
/// Implemented on both sides of the context/causal split: `Contextoid` in
/// `deep_causality_context`, and `Causaloid`, `Model`, `Inference`, `Assumption`, `Observation`
/// and `ProposedAction` in `deep_causality`. It lives here because this is the only crate both
/// can reach, and because [`IdentificationValue`] — the type an id *is* — is defined here.
pub trait Identifiable {
    /// Returns the unique id of this item.
    fn id(&self) -> IdentificationValue;
}
