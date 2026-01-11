/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge Group trait and marker types.
//!
//! A gauge group defines the local symmetry of a gauge field theory.
//! The Lie algebra dimension determines the number of gauge bosons.

use deep_causality_metric::Metric;
use std::fmt::Debug;

/// Marker trait for gauge groups.
///
/// A gauge group G defines the local symmetry of a gauge field theory.
/// Each gauge group has an associated Lie algebra, whose dimension
/// determines the number of gauge bosons (force carriers).
///
/// # Associated Constants
///
/// - `LIE_ALGEBRA_DIM`: Number of generators (gauge bosons)
/// - `IS_ABELIAN`: Whether the group is commutative
/// - `SPACETIME_DIM`: Dimension of spacetime (default 4)
///
/// # Examples
///
/// ```ignore
/// use deep_causality_topology::{GaugeGroup, U1, Lorentz};
///
/// // U(1) - one generator (photon)
/// assert_eq!(U1::LIE_ALGEBRA_DIM, 1);
/// assert!(U1::IS_ABELIAN);
///
/// // SO(3,1) - six generators (3 rotations + 3 boosts)
/// assert_eq!(Lorentz::LIE_ALGEBRA_DIM, 6);
/// assert!(!Lorentz::IS_ABELIAN);
/// ```
pub trait GaugeGroup: Clone + Debug + Send + Sync + 'static {
    /// Dimension of the Lie algebra (number of generators).
    ///
    /// This equals the number of gauge bosons in the theory:
    /// - U(1): 1 (photon)
    /// - SU(2): 3 (W+, W-, Z)
    /// - SU(3): 8 (8 gluons)
    /// - SO(3,1): 6 (3 rotations + 3 boosts)
    const LIE_ALGEBRA_DIM: usize;

    /// Whether the group is abelian (commutative).
    ///
    /// For abelian groups: F = dA (field strength is exterior derivative of potential)
    /// For non-abelian groups: F = dA + A∧A (includes self-interaction)
    const IS_ABELIAN: bool;

    /// Number of spacetime dimensions (default 4).
    const SPACETIME_DIM: usize = 4;

    /// Human-readable name of the gauge group.
    fn name() -> &'static str;

    /// Default metric for this gauge group.
    ///
    /// Override for specific physics conventions:
    /// - Particle physics typically uses West Coast (+---)
    /// - GR typically uses East Coast (-+++)
    fn default_metric() -> Metric {
        Metric::Minkowski(Self::SPACETIME_DIM)
    }

    /// Returns the matrix dimension N for representation as N×N matrices.
    ///
    /// For SU(N), this is N (the fundamental representation dimension).
    /// For U(1), this is 1.
    /// For SO(3,1), this is 4.
    ///
    /// Default implementation computes N from LIE_ALGEBRA_DIM assuming SU(N):
    /// LIE_ALGEBRA_DIM = N² - 1, so N = √(LIE_ALGEBRA_DIM + 1)
    fn matrix_dim() -> usize {
        // Default: assumes SU(N) where dim = N² - 1
        // N = sqrt(dim + 1), rounded
        let n_sq = Self::LIE_ALGEBRA_DIM + 1;
        // Integer sqrt with exact validation (avoids floating-point rounding issues).
        let n = (n_sq as f64).sqrt() as usize;
        let n = if (n + 1) * (n + 1) <= n_sq { n + 1 } else { n };

        if n > 0 && n * n == n_sq { n } else { 0 }
    }

    /// Returns the structure constant f^{abc} for the Lie algebra.
    ///
    /// Defined by the commutator relation: [T^a, T^b] = i f^{abc} T^c
    ///
    /// # Default
    /// Returns 0.0 (valid for Abelian groups like U(1)).
    /// Overridden by non-Abelian groups (SU(2), SU(3), etc.).
    fn structure_constant(_a: usize, _b: usize, _c: usize) -> f64 {
        0.0
    }
}
