/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DeonticExplainable;
use crate::{DeonticError, EffectEthos, TeloidModal, Verdict};
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> DeonticExplainable<D, S, T, ST, SYM, VS, VT>
    for EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn explain_verdict(&self, verdict: &Verdict) -> Result<String, DeonticError> {
        let mut explanation = format!("The final verdict is {}.\n\n", verdict.outcome());

        explanation.push_str(&format!(
            "This was based on {} active and undefeated norm(s):\n",
            verdict.justification().len()
        ));

        for id in verdict.justification() {
            let teloid = self
                .get_norm(*id)
                .ok_or(DeonticError::TeloidNotFound { id: *id })?;
            explanation.push_str(&format!(
                "- Norm {}: '{}' ({}, Specificity: {}, Timestamp: {}",
                teloid.id(),
                teloid.action_identifier(),
                teloid.modality(),
                teloid.specificity(),
                teloid.timestamp()
            ));
        }

        // Note: A more detailed explanation of *why* these norms won (i.e., which potential
        // defeaters were themselves defeated or inactive) would require re-running or caching
        // the conflict resolution logic. This implementation explains the result, not the process.

        explanation.push_str("\nReasoning Summary:\n");
        let summary = match verdict.outcome() {
            TeloidModal::Impermissible => {
                "The outcome is Impermissible because at least one impermissible norm was active and undefeated, which has the highest precedence."
            }
            TeloidModal::Obligatory => {
                "The outcome is Obligatory because at least one obligatory norm was active and no impermissible norms were found."
            }
            TeloidModal::Optional(_) => {
                "The outcome is Optional because only optional norms were active and undefeated."
            }
        };
        explanation.push_str(summary);

        Ok(explanation)
    }
}
