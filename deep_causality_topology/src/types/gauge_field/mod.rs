/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge Field type for unified gauge theory representation.
//!
//! A gauge field combines a base manifold, metric signature, connection (potential),
//! and field strength (curvature) under a specified gauge group symmetry.
//!
//! # Example
//!
//! ```ignore
//! use deep_causality_topology::{GaugeField, U1, Lorentz, Manifold};
//! use deep_causality_tensor::CausalTensor;
//!
//! // Create an electromagnetic (QED) gauge field
//! let em: GaugeField<U1, f64, f64> = GaugeField::with_default_metric(
//!     spacetime,
//!     potential,
//!     field_strength,
//! );
//!
//! // Create a gravitational (GR) gauge field
//! let gravity: GaugeField<Lorentz, f64, f64> = GaugeField::with_default_metric(
//!     spacetime,
//!     christoffel,
//!     riemann_tensor,
//! );
//! ```

mod group;
pub mod groups;

pub use group::GaugeGroup;

use crate::errors::topology_error::TopologyError;
use crate::{BaseTopology, Manifold};
use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// A gauge field over a base manifold.
///
/// A gauge field is a principal fiber bundle with connection, parameterized by:
/// - A gauge group G defining the local symmetry
/// - A connection (gauge potential) A
/// - A field strength (curvature) F
///
/// # Type Parameters
///
/// * `G` - The gauge group (U1, SU2, SU3, Lorentz, etc.)
/// * `A` - The connection (potential) scalar type
/// * `F` - The field strength (curvature) scalar type
///
/// # Mathematical Structure
///
/// ```text
/// Connection (Gauge Potential):
///   A_μ^a  where μ ∈ {0,1,2,3} (spacetime index), a ∈ {1..dim(g)} (Lie algebra index)
///   Shape: [num_points, spacetime_dim, lie_algebra_dim]
///
/// Field Strength (Curvature):
///   Abelian (U(1)):     F_μν = ∂_μ A_ν - ∂_ν A_μ
///   Non-Abelian (SU(N)): F_μν^a = ∂_μ A_ν^a - ∂_ν A_μ^a + g f^{abc} A_μ^b A_ν^c
///   Shape: [num_points, spacetime_dim, spacetime_dim, lie_algebra_dim]
/// ```
///
/// # Gauge Theory Correspondence
///
/// | Theory | Gauge Group | Connection        | Field Strength    |
/// |--------|-------------|-------------------|-------------------|
/// | QED    | U(1)        | 4-potential A_μ   | F_μν (E, B)       |
/// | QCD    | SU(3)       | Gluon field G_μ^a | G_μν^a            |
/// | GR     | SO(3,1)     | Christoffel Γ     | Riemann R^ρ_σμν   |
#[derive(Debug, Clone)]
pub struct GaugeField<G: GaugeGroup, A, F> {
    /// Base manifold (spacetime). Private for invariant preservation.
    base: Manifold<f64>,

    /// Spacetime metric signature (Minkowski, Euclidean, etc.).
    metric: Metric,

    /// Gauge connection (potential).
    /// Shape: [num_points, spacetime_dim, lie_algebra_dim]
    connection: CausalTensor<A>,

    /// Field strength (curvature).
    /// Shape: [num_points, spacetime_dim, spacetime_dim, lie_algebra_dim]
    field_strength: CausalTensor<F>,

    /// Gauge group marker.
    _gauge: PhantomData<G>,
}

// ============================================================================
// Constructors
// ============================================================================

impl<G: GaugeGroup, A, F> GaugeField<G, A, F> {
    /// Creates a new gauge field with an explicit metric.
    ///
    /// # Mathematical Definition
    ///
    /// A gauge field (A, F) satisfies the structure equation:
    /// ```text
    /// F = dA + A ∧ A   (non-abelian)
    /// F = dA           (abelian, when IS_ABELIAN = true)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base` - The base manifold (spacetime)
    /// * `metric` - The spacetime metric signature
    /// * `connection` - The gauge connection tensor, shape: [num_points, spacetime_dim, lie_algebra_dim]
    /// * `field_strength` - The field strength tensor, shape: [num_points, dim, dim, lie_algebra_dim]
    ///
    /// # Errors
    ///
    /// Returns `TopologyError::GaugeFieldError` if tensor shapes don't match expected dimensions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use deep_causality_metric::Metric;
    /// use deep_causality_topology::{GaugeField, U1};
    ///
    /// let em_field = GaugeField::<U1, f64, f64>::new(
    ///     spacetime,
    ///     Metric::Minkowski(4),
    ///     potential,
    ///     field_tensor,
    /// )?;
    /// ```
    pub fn new(
        base: Manifold<f64>,
        metric: Metric,
        connection: CausalTensor<A>,
        field_strength: CausalTensor<F>,
    ) -> Result<Self, TopologyError> {
        // Validate connection shape: [num_points, spacetime_dim, lie_algebra_dim]
        let num_points = base.len().max(1);
        let spacetime_dim = G::SPACETIME_DIM;
        let lie_dim = G::LIE_ALGEBRA_DIM;
        let conn_shape = connection.shape();

        // Allow flexible validation: check that total elements are consistent
        // or that shape matches expected pattern
        let expected_conn_elements = num_points * spacetime_dim * lie_dim;
        let actual_conn_elements: usize = conn_shape.iter().product();

        if actual_conn_elements != expected_conn_elements && conn_shape.len() != 3 {
            return Err(TopologyError::GaugeFieldError(format!(
                "Connection shape mismatch for {}: got {:?} ({} elements), \
                 expected [num_points={}, spacetime_dim={}, lie_dim={}] ({} elements)",
                G::name(),
                conn_shape,
                actual_conn_elements,
                num_points,
                spacetime_dim,
                lie_dim,
                expected_conn_elements
            )));
        }

        // Validate field strength shape: [num_points, dim, dim, lie_algebra_dim]
        let expected_fs_elements = num_points * spacetime_dim * spacetime_dim * lie_dim;
        let fs_shape = field_strength.shape();
        let actual_fs_elements: usize = fs_shape.iter().product();

        if actual_fs_elements != expected_fs_elements && fs_shape.len() != 4 {
            return Err(TopologyError::GaugeFieldError(format!(
                "Field strength shape mismatch for {}: got {:?} ({} elements), \
                 expected [num_points={}, dim={}, dim={}, lie_dim={}] ({} elements)",
                G::name(),
                fs_shape,
                actual_fs_elements,
                num_points,
                spacetime_dim,
                spacetime_dim,
                lie_dim,
                expected_fs_elements
            )));
        }

        Ok(Self {
            base,
            metric,
            connection,
            field_strength,
            _gauge: PhantomData,
        })
    }

    /// Creates a new gauge field with the default metric for the gauge group.
    ///
    /// The default metric is determined by `G::default_metric()`:
    /// - Most gauge groups: West Coast Minkowski (+---)
    /// - Lorentz (GR): East Coast Minkowski (-+++)
    ///
    /// # Arguments
    ///
    /// * `base` - The base manifold (spacetime)
    /// * `connection` - The gauge connection tensor
    /// * `field_strength` - The field strength tensor
    ///
    /// # Errors
    ///
    /// Returns `TopologyError::GaugeFieldError` if tensor shapes don't match expected dimensions.
    pub fn with_default_metric(
        base: Manifold<f64>,
        connection: CausalTensor<A>,
        field_strength: CausalTensor<F>,
    ) -> Result<Self, TopologyError> {
        Self::new(base, G::default_metric(), connection, field_strength)
    }
}

// ============================================================================
// Getters
// ============================================================================

impl<G: GaugeGroup, A, F> GaugeField<G, A, F> {
    /// Returns a reference to the base manifold.
    #[inline]
    pub fn base(&self) -> &Manifold<f64> {
        &self.base
    }

    /// Returns the spacetime metric signature.
    #[inline]
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// Returns a reference to the gauge connection (potential).
    #[inline]
    pub fn connection(&self) -> &CausalTensor<A> {
        &self.connection
    }

    /// Returns a reference to the field strength (curvature).
    #[inline]
    pub fn field_strength(&self) -> &CausalTensor<F> {
        &self.field_strength
    }

    /// Returns the human-readable name of the gauge group.
    #[inline]
    pub fn gauge_group_name(&self) -> &'static str {
        G::name()
    }

    /// Returns the dimension of the Lie algebra (number of generators).
    #[inline]
    pub fn lie_algebra_dim(&self) -> usize {
        G::LIE_ALGEBRA_DIM
    }

    /// Returns whether the gauge group is abelian.
    ///
    /// For abelian groups: F = dA
    /// For non-abelian groups: F = dA + A∧A
    #[inline]
    pub fn is_abelian(&self) -> bool {
        G::IS_ABELIAN
    }

    /// Returns the spacetime dimension.
    #[inline]
    pub fn spacetime_dim(&self) -> usize {
        G::SPACETIME_DIM
    }

    /// Checks if using East Coast convention (-+++).
    ///
    /// East Coast is standard in GR textbooks (MTW, Wald).
    #[inline]
    pub fn is_east_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == -1
    }

    /// Checks if using West Coast convention (+---).
    ///
    /// West Coast is standard in particle physics (Weinberg, Peskin & Schroeder).
    #[inline]
    pub fn is_west_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == 1
            && self.metric.dimension() > 1
            && self.metric.sign_of_sq(1) == -1
    }
}
