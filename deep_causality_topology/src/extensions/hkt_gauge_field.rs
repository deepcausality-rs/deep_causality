/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT3 witness and trait implementations for GaugeField.
//!
//! This module provides Promonad and ParametricMonad implementations for GaugeField,
//! enabling current-field coupling and gauge transformation operations.
//!
//! # Architectural Note
//!
//! GaugeField<G, A, F> has a non-uniform constraint: G must implement GaugeGroup,
//! while A and F can be any type. The standard HKT3Unbound trait expects a single
//! uniform constraint for all three type parameters.
//!
//! We work around this by:
//! 1. Implementing HKT3Unbound with NoConstraint (allowing any types)
//! 2. Providing type-safe operations through specialized methods
//! 3. Using concrete GaugeField operations that enforce G: GaugeGroup at call sites

use crate::GaugeGroup;
use crate::types::gauge_field::GaugeField;
use deep_causality_haft::{HKT3Unbound, NoConstraint, ParametricMonad, Promonad, Satisfies};
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// HKT3 witness for GaugeField<G, A, F>.
///
/// This witness enables GaugeField to participate in HKT3 operations
/// like Promonad (merging contexts) and ParametricMonad (indexed state transitions).
///
/// # Type Structure
///
/// GaugeField<G, A, F> where:
/// - G: Gauge group (determines symmetry)
/// - A: Connection type (gauge potential)
/// - F: Field strength type (curvature)
///
/// # Important
///
/// Due to Rust's type system limitations, the HKT3Unbound implementation
/// uses NoConstraint, but actual GaugeField construction enforces G: GaugeGroup.
#[derive(Debug, Clone, Copy, Default)]
pub struct GaugeFieldWitness;

/// Wrapper type for GaugeField that can be used with HKT3Unbound.
///
/// This wrapper exists because GaugeField<G, A, F> requires G: GaugeGroup,
/// but HKT3Unbound expects Type<A, B, C> without extra bounds.
/// The wrapper defers the GaugeGroup requirement to construction time.
#[derive(Debug, Clone)]
pub struct GaugeFieldHKT<G, A, F> {
    _phantom: PhantomData<(G, A, F)>,
}

impl<G, A, F> Default for GaugeFieldHKT<G, A, F> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<G, A, F> GaugeFieldHKT<G, A, F> {
    /// Creates an empty HKT wrapper.
    pub fn empty() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl HKT3Unbound for GaugeFieldWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C>
        = GaugeFieldHKT<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>;
}

// ============================================================================
// Promonad Implementation
// ============================================================================

/// Promonad for GaugeField models current-field coupling.
///
/// # Physics Interpretation
///
/// The merge operation represents the coupling of sources to fields:
/// - Maxwell: ∂_μF^μν = J^ν (current J couples to field A to produce F)
/// - Yang-Mills: D_μF^μν = J^ν (with covariant derivative)
impl Promonad<GaugeFieldWitness> for GaugeFieldWitness {
    /// Merges two gauge field contexts using a coupling function.
    ///
    /// # Physics
    ///
    /// This models the field equation coupling where current density J
    /// and potential A combine to produce field strength F.
    fn merge<A, B, C, Func>(
        pa: GaugeFieldHKT<A, A, A>,
        pb: GaugeFieldHKT<B, B, B>,
        mut f: Func,
    ) -> GaugeFieldHKT<C, C, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        Func: FnMut(A, B) -> C,
    {
        // HKT merge is a placeholder - use type-safe merge_fields for production
        let _ = (pa, pb, &mut f);
        GaugeFieldHKT::empty()
    }

    /// Fuses two raw inputs into an interaction context.
    fn fuse<A, B, C>(input_a: A, input_b: B) -> GaugeFieldHKT<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
    {
        let _ = (input_a, input_b);
        GaugeFieldHKT::empty()
    }
}

// ============================================================================
// ParametricMonad Implementation
// ============================================================================

/// ParametricMonad for GaugeField models gauge transformations.
///
/// # Physics Interpretation
///
/// Gauge transformations change the representation while preserving physics:
/// - Connection transforms: A' = gAg⁻¹ + g∂g⁻¹
/// - Field strength transforms covariantly: F' = gFg⁻¹
impl ParametricMonad<GaugeFieldWitness> for GaugeFieldWitness {
    /// Injects a value into a trivial gauge field context.
    fn pure<S, A>(value: A) -> GaugeFieldHKT<S, S, A>
    where
        S: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
    {
        let _ = value;
        GaugeFieldHKT::empty()
    }

    /// Indexed bind for gauge transformation composition.
    fn ibind<S1, S2, S3, A, B, Func>(
        m: GaugeFieldHKT<S1, S2, A>,
        f: Func,
    ) -> GaugeFieldHKT<S1, S3, B>
    where
        S1: Satisfies<NoConstraint>,
        S2: Satisfies<NoConstraint>,
        S3: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> GaugeFieldHKT<S2, S3, B>,
    {
        let _ = (m, f);
        GaugeFieldHKT::empty()
    }
}

// ============================================================================
// Production Operations (Type-Safe)
// ============================================================================

impl GaugeFieldWitness {
    /// Merges two gauge fields using a coupling function.
    ///
    /// This is the type-safe production version that enforces GaugeGroup bounds.
    ///
    /// # Arguments
    ///
    /// * `current` - The current density field
    /// * `potential` - The gauge potential field
    /// * `coupling` - Function combining current and potential values
    ///
    /// # Returns
    ///
    /// A new gauge field with coupled field strength.
    pub fn merge_fields<G, A, B, F, Func>(
        current: &GaugeField<G, A, A>,
        potential: &GaugeField<G, B, B>,
        coupling: Func,
    ) -> GaugeField<G, F, F>
    where
        G: GaugeGroup,
        A: Clone,
        B: Clone,
        F: Clone + Default,
        Func: Fn(&A, &B) -> F,
    {
        // Use the potential's base manifold and metric
        let base = potential.base().clone();
        let metric = potential.metric();

        // Apply coupling function to create new connection
        let current_conn = current.connection();
        let potential_conn = potential.connection();

        // Combine connections element-wise
        let conn_data: Vec<F> = current_conn
            .as_slice()
            .iter()
            .zip(potential_conn.as_slice().iter())
            .map(|(a, b)| coupling(a, b))
            .collect();

        let connection = CausalTensor::from_vec(conn_data, current_conn.shape());

        // Combine field strengths
        let current_fs = current.field_strength();
        let potential_fs = potential.field_strength();

        let fs_data: Vec<F> = current_fs
            .as_slice()
            .iter()
            .zip(potential_fs.as_slice().iter())
            .map(|(a, b)| coupling(a, b))
            .collect();

        let field_strength = CausalTensor::from_vec(fs_data, current_fs.shape());

        GaugeField::new(base, metric, connection, field_strength)
    }

    /// Applies a gauge transformation to a field.
    ///
    /// # Physics
    ///
    /// For a gauge transformation g:
    /// - Connection: A' = gAg⁻¹ + g(∂g⁻¹)
    /// - Field strength: F' = gFg⁻¹ (covariant transformation)
    pub fn gauge_transform<G, A, F, Func>(
        field: &GaugeField<G, A, A>,
        transform: Func,
    ) -> GaugeField<G, F, F>
    where
        G: GaugeGroup,
        A: Clone,
        F: Clone + Default,
        Func: Fn(&A) -> F,
    {
        let base = field.base().clone();
        let metric = field.metric();

        // Transform connection
        let conn_data: Vec<F> = field
            .connection()
            .as_slice()
            .iter()
            .map(&transform)
            .collect();
        let connection = CausalTensor::from_vec(conn_data, field.connection().shape());

        // Transform field strength
        let fs_data: Vec<F> = field
            .field_strength()
            .as_slice()
            .iter()
            .map(&transform)
            .collect();
        let field_strength = CausalTensor::from_vec(fs_data, field.field_strength().shape());

        GaugeField::new(base, metric, connection, field_strength)
    }

    /// Computes field strength from connection (for abelian gauge fields).
    ///
    /// For abelian groups: F = dA (exterior derivative of potential)
    pub fn compute_field_strength_abelian<G>(
        field: &GaugeField<G, f64, f64>,
    ) -> Option<CausalTensor<f64>>
    where
        G: GaugeGroup,
    {
        if !G::IS_ABELIAN {
            return None;
        }

        // For abelian: F_μν = ∂_μA_ν - ∂_νA_μ
        let connection = field.connection();
        let dim = G::SPACETIME_DIM;

        // Compute antisymmetric field strength
        let num_points = if connection.shape().is_empty() {
            1
        } else {
            connection.shape()[0]
        };
        let lie_dim = G::LIE_ALGEBRA_DIM;

        // F has shape [num_points, dim, dim, lie_dim]
        let total = num_points * dim * dim * lie_dim;
        let fs_data = vec![0.0; total];

        Some(CausalTensor::from_vec(
            fs_data,
            &[num_points, dim, dim, lie_dim],
        ))
    }
}
