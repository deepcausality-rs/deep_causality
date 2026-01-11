/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lattice Gauge Field type for lattice gauge theory computations.
//!
//! A lattice gauge field assigns group-valued link variables to each edge
//! of a discrete lattice, enabling Wilson-formulation gauge theory.
//!
//!
//! # Mathematical Structure
//!
//! - **Lattice:** Discrete spacetime Γ ⊂ ℤᴰ with spacing a
//! - **Link variables:** U_μ(x) ∈ G on each edge
//! - **Plaquettes:** Ordered product around elementary squares
//! - **Wilson action:** S = β Σ_p (1 - Re[Tr(U_p)]/N)

use crate::errors::topology_error::TopologyError;
use crate::traits::cw_complex::CWComplex;
use crate::types::gauge::link_variable::LinkVariable;
use crate::{GaugeGroup, Lattice, LatticeCell};
use std::collections::HashMap;
use std::sync::Arc;
mod display;
mod getters;
mod ops_actions;
mod ops_continuum;
mod ops_gauge;
mod ops_gauge_transform;
pub mod ops_gradient_flow;
mod ops_metropolis;
mod ops_monte_carlo;
mod ops_plague;
pub mod ops_smearing;

mod ops_wilson;
mod utils;

/// A gauge field on a D-dimensional lattice.
///
/// Link variables U_μ(n) are stored on each edge of the lattice.
/// For a hypercubic lattice: D × num_vertices links total (periodic).
///
/// # Type Parameters
///
/// * `G` - Gauge group (U1, SU2, SU3, etc.)
/// * `D` - Spacetime dimension
/// * `T` - Scalar type for matrix elements
#[derive(Debug, Clone)]
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, T> {
    /// The underlying lattice structure.
    lattice: Arc<Lattice<D>>,

    /// Link variables indexed by LatticeCell (1-cells only).
    /// Key: edge cell, Value: group element
    links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,

    /// Coupling parameter β = 2N/g².
    beta: T,
}

// ============================================================================
// Constructors
// ============================================================================

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Create with all links set to identity.
    ///
    /// This is a "cold start" configuration representing the trivial vacuum.
    ///
    /// # Errors
    ///
    /// Returns `TopologyError` if link creation fails.
    pub fn try_identity(lattice: Arc<Lattice<D>>, beta: T) -> Result<Self, TopologyError>
    where
        T: From<f64>,
    {
        let mut links = HashMap::new();

        // Iterate over all 1-cells (edges)
        for cell in lattice.cells(1) {
            let link = LinkVariable::try_identity().map_err(TopologyError::from)?;
            links.insert(cell, link);
        }

        Ok(Self {
            lattice,
            links,
            beta,
        })
    }

    /// Create with all links set to identity (convenience method).
    ///
    /// # Panics
    ///
    /// Panics if link creation fails (should not happen for valid lattice).
    pub fn identity(lattice: Arc<Lattice<D>>, beta: T) -> Self
    where
        T: From<f64>,
    {
        Self::try_identity(lattice, beta)
            .unwrap_or_else(|e| panic!("Identity field creation failed: {}", e))
    }

    /// Create from explicit link data.
    ///
    /// # Arguments
    ///
    /// * `lattice` - The underlying lattice structure
    /// * `links` - Map of edge cells to link variables
    /// * `beta` - Coupling parameter
    ///
    /// # Returns
    ///
    /// A new `LatticeGaugeField` or error if validation fails.
    ///
    /// # Errors
    ///
    /// Returns error if links are missing for some edges.
    pub fn try_from_links(
        lattice: Arc<Lattice<D>>,
        links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,
        beta: T,
    ) -> Result<Self, TopologyError> {
        // Validate that we have links for all edges
        let expected_count = lattice.num_cells(1);
        if links.len() != expected_count {
            return Err(TopologyError::LatticeGaugeError(format!(
                "Link count mismatch: expected {}, got {}",
                expected_count,
                links.len()
            )));
        }

        Ok(Self {
            lattice,
            links,
            beta,
        })
    }

    /// Create with random links ("hot start" for Monte Carlo).
    ///
    /// Initializes all link variables to random SU(N) elements.
    /// This represents a "hot start" configuration far from equilibrium.
    ///
    /// # Physics
    ///
    /// Hot start configurations:
    /// - Begin far from the classical vacuum (all identity)
    /// - Require thermalization before measurements
    /// - Are used to verify thermalization independence
    ///
    /// In contrast, "cold start" (`identity()`) begins at the trivial vacuum.
    ///
    /// # Arguments
    ///
    /// * `lattice` - The underlying lattice structure
    /// * `beta` - Coupling parameter β = 2N/g²
    /// * `rng` - Random number generator
    ///
    /// # Errors
    ///
    /// Returns `TopologyError` if link creation fails.
    pub fn try_random<R>(
        lattice: Arc<Lattice<D>>,
        beta: T,
        rng: &mut R,
    ) -> Result<Self, TopologyError>
    where
        R: deep_causality_rand::Rng,
        T: From<f64>
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + PartialOrd,
    {
        let mut links = HashMap::new();

        for cell in lattice.cells(1) {
            let link = LinkVariable::try_random(rng).map_err(TopologyError::from)?;
            links.insert(cell, link);
        }

        Ok(Self {
            lattice,
            links,
            beta,
        })
    }

    /// Create with random links (convenience method).
    ///
    /// See [`try_random`](Self::try_random) for details.
    ///
    /// # Panics
    ///
    /// Panics if link creation fails.
    pub fn random<R>(lattice: Arc<Lattice<D>>, beta: T, rng: &mut R) -> Self
    where
        R: deep_causality_rand::Rng,
        T: From<f64>
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + PartialOrd,
    {
        Self::try_random(lattice, beta, rng)
            .unwrap_or_else(|e| panic!("Random field creation failed: {}", e))
    }
}

// Separate impl block without Clone + Default bounds for HKT compatibility
impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Create from explicit link data without validation.
    ///
    /// This constructor has minimal bounds for HKT compatibility.
    ///
    /// # Arguments
    ///
    /// * `lattice` - The underlying lattice structure
    /// * `links` - Map of edge cells to link variables
    /// * `beta` - Coupling parameter
    ///
    /// # Returns
    ///
    /// A new `LatticeGaugeField`.
    pub fn from_links_unchecked(
        lattice: Arc<Lattice<D>>,
        links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,
        beta: T,
    ) -> Self {
        Self {
            lattice,
            links,
            beta,
        }
    }
}
