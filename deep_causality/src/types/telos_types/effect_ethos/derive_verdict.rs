/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Datable, SpaceTemporal, Spatial, Symbolic, TeloidID, Temporal};
use crate::{DeonticError, EffectEthos, Teloid, TeloidModal, Verdict};

// Private helper methods for EffectEthos
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Derives a `Verdict` based on a collection of `Teloid` norms.
    ///
    /// This method evaluates the provided norms in a specific order of precedence:
    /// 1. Impermissible norms: If any are present, an `Impermissible` verdict is returned.
    /// 2. Obligatory norms: If any are present (and no Impermissible norms were found), an `Obligatory` verdict is returned.
    /// 3. Optional norms: If any are present (and no Impermissible or Obligatory norms were found),
    ///    an `Optional` verdict is returned, with the total cost of all optional norms.
    ///
    /// If the `norms` vector is empty, or if no norms with a clear modality are found, an `InconclusiveVerdict` error is returned.
    ///
    /// # Arguments
    ///
    /// * `norms` - A `Vec` of `Teloid` instances representing the norms to evaluate.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(Verdict)` if a verdict can be derived.
    /// * `Err(DeonticError::InconclusiveVerdict)` if no clear verdict can be derived or the input is empty.
    pub(super) fn derive_verdict(
        &self,
        norms: Vec<Teloid<D, S, T, ST, SYM, VS, VT>>,
    ) -> Result<Verdict, DeonticError> {
        if norms.is_empty() {
            return Err(DeonticError::InconclusiveVerdict);
        }

        // Store all IDs for the final justification, regardless of modality.
        let all_norm_ids: Vec<TeloidID> = norms.iter().map(|n| n.id()).collect();

        // Check for any Impermissible norms first, as they have the highest precedence.
        let has_impermissible = norms
            .iter()
            .any(|n| n.modality() == TeloidModal::Impermissible);

        if has_impermissible {
            return Ok(Verdict::new(TeloidModal::Impermissible, all_norm_ids));
        }

        // Check for Obligatory norms next.
        let has_obligatory = norms
            .iter()
            .any(|n| n.modality() == TeloidModal::Obligatory);

        if has_obligatory {
            return Ok(Verdict::new(TeloidModal::Obligatory, all_norm_ids));
        }

        // Finally, handle Optional norms.
        let total_cost: i64 = norms
            .iter()
            .filter_map(|n| {
                if let TeloidModal::Optional(cost) = n.modality() {
                    Some(cost)
                } else {
                    None
                }
            })
            .sum();

        if !norms.iter().all(|n| matches!(n.modality(), TeloidModal::Optional(_))) {
             // This branch should ideally not be hit if the logic is sound,
             // but as a safeguard, if we have a mix that doesn't include Impermissible or Obligatory,
             // we default to the safest (inconclusive) verdict.
             return Err(DeonticError::InconclusiveVerdict);
        }

        Ok(Verdict::new(TeloidModal::Optional(total_cost), all_norm_ids))
    }
}
