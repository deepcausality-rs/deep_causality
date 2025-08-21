/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
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

        // Check for any Impermissible norms first, as they have the highest precedence.
        let impermissible_norms: Vec<_> = norms
            .iter()
            .filter(|n| n.modality() == TeloidModal::Impermissible)
            .collect();

        if !impermissible_norms.is_empty() {
            return Ok(Verdict::new(
                TeloidModal::Impermissible,
                impermissible_norms.iter().map(|n| n.id()).collect(),
            ));
        }

        // Check for Obligatory norms next.
        let obligatory_norms: Vec<_> = norms
            .iter()
            .filter(|n| n.modality() == TeloidModal::Obligatory)
            .collect();

        if !obligatory_norms.is_empty() {
            return Ok(Verdict::new(
                TeloidModal::Obligatory,
                obligatory_norms.iter().map(|n| n.id()).collect(),
            ));
        }

        // Finally, handle Optional norms.
        let optional_norms: Vec<_> = norms
            .iter()
            .filter_map(|n| {
                if let TeloidModal::Optional(cost) = n.modality() {
                    Some((n.id(), cost))
                } else {
                    None
                }
            })
            .collect();

        if !optional_norms.is_empty() {
            let total_cost: i64 = optional_norms.iter().map(|(_, cost)| cost).sum();
            // Here, a cost threshold could be applied. For now, we just return the collective optionality.

            return Ok(Verdict::new(
                TeloidModal::Optional(total_cost),
                optional_norms.iter().map(|(id, _)| *id).collect(),
            ));
        }

        // If we reach here, no norms with a clear modality were found.
        Err(DeonticError::InconclusiveVerdict)
    }
}
