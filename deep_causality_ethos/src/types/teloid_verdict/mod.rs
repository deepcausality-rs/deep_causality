/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TeloidID, TeloidModal};

mod display;
mod getters;

/// Represents the final, justified outcome of a deontic evaluation.
///
/// For more details, see section 5 (Causality as EPP) and 8 (Teleology) in The EPP reference paper:
/// <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The final deontic modality (e.g., Obligatory, Impermissible) determined
    /// by the consensus of active norms.
    outcome: TeloidModal,
    /// A vector of TeloidIDs that form the justification for the verdict.
    /// This provides an audit trail for the decision.
    justification: Vec<TeloidID>,
}

impl Verdict {
    /// Creates a new `Verdict` instance.
    ///
    /// # Arguments
    ///
    /// * `outcome` - The final deontic modality of the verdict.
    /// * `justification` - A vector of `TeloidID`s providing the justification for the verdict.
    pub fn new(outcome: TeloidModal, justification: Vec<TeloidID>) -> Self {
        Self {
            outcome,
            justification,
        }
    }
}
