/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge transformations for lattice gauge fields.
//!
//! Gauge transformations are local group rotations that preserve physics.

use crate::traits::cellular_complex::CellularComplex;
use crate::{GaugeGroup, LatticeGaugeField, LinkVariable, TopologyError};
use deep_causality_algebra::{ComplexField, DivisionAlgebra, Field, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};

use std::fmt::Debug;

// ============================================================================
// Gauge Transformations
// ============================================================================

impl<
    G: GaugeGroup,
    const D: usize,
    M: Field + Copy + Default + PartialOrd + Debug + ComplexField<R> + DivisionAlgebra<R>,
    R: RealField + FromPrimitive + ToPrimitive,
    S,
> LatticeGaugeField<G, D, M, R, S>
{
    /// Apply a random gauge transformation (for testing gauge invariance).
    ///
    /// Useful for verifying that observables are gauge-invariant.
    pub fn try_random_gauge_transform<RngType>(
        &mut self,
        rng: &mut RngType,
    ) -> Result<(), TopologyError>
    where
        RngType: deep_causality_stats::Rng,
        M: crate::types::gauge::link_variable::random::RandomField + DivisionAlgebra<R> + Field,
        R: RealField,
    {
        use std::collections::HashMap;

        // Pre-generate gauge elements for all sites
        let mut gauge_elements: HashMap<[usize; D], LinkVariable<G, M, R>> = HashMap::new();

        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            let omega = LinkVariable::<G, M, R>::try_random(rng).map_err(TopologyError::from)?;
            gauge_elements.insert(site, omega);
        }

        // Apply transformation
        self.gauge_transform(|site| {
            gauge_elements
                .get(site)
                .cloned()
                .unwrap_or_else(|| LinkVariable::identity())
        });
        Ok(())
    }
}
