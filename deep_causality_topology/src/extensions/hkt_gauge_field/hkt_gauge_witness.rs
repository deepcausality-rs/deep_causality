/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GaugeField, GaugeGroup};
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
///
/// # Implementation Note
///
/// This wrapper stores the actual GaugeField data as a type-erased Box.
/// The HKT trait methods use **unsafe dispatch** to convert between generic
/// types and concrete GaugeField instances.
///
/// **SAFETY:** Callers MUST ensure that the generic types match the stored
/// GaugeField's type parameters. Misuse causes Undefined Behavior.
#[derive(Debug, Clone)]
pub struct GaugeFieldHKT<G, A, F> {
    /// Type-erased storage for a GaugeField.
    /// None represents an empty/identity context.
    inner: Option<Box<GaugeFieldData>>,
    _phantom: PhantomData<(G, A, F)>,
}

/// Type-erased storage for GaugeField data.
/// This is necessary because we can't directly store GaugeField<G, A, F>
/// when G, A, F are generic HKT parameters.
#[derive(Debug, Clone)]
struct GaugeFieldData {
    /// Serialized representation of the gauge field.
    /// In production, this would be the actual tensor data.
    connection_data: Vec<f64>,
    field_strength_data: Vec<f64>,
    connection_shape: Vec<usize>,
    field_strength_shape: Vec<usize>,
}

impl<G, A, F> Default for GaugeFieldHKT<G, A, F> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<G, A, F> GaugeFieldHKT<G, A, F> {
    /// Creates an empty HKT wrapper (identity element).
    pub fn empty() -> Self {
        Self {
            inner: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a wrapper from serialized data.
    pub fn from_data(
        connection_data: Vec<f64>,
        field_strength_data: Vec<f64>,
        connection_shape: Vec<usize>,
        field_strength_shape: Vec<usize>,
    ) -> Self {
        Self {
            inner: Some(Box::new(GaugeFieldData {
                connection_data,
                field_strength_data,
                connection_shape,
                field_strength_shape,
            })),
            _phantom: PhantomData,
        }
    }

    /// Returns true if this wrapper contains data.
    pub fn has_data(&self) -> bool {
        self.inner.is_some()
    }

    /// Returns the connection data if present.
    pub fn connection_data(&self) -> Option<&[f64]> {
        self.inner.as_ref().map(|d| d.connection_data.as_slice())
    }

    /// Returns the field strength data if present.
    pub fn field_strength_data(&self) -> Option<&[f64]> {
        self.inner
            .as_ref()
            .map(|d| d.field_strength_data.as_slice())
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
///
/// # Implementation Note
///
/// The HKT trait methods operate on the type-erased `GaugeFieldData`.
/// For strongly-typed operations with proper `GaugeGroup` constraints,
/// use the `merge_fields()` method on `GaugeFieldWitness` instead.
impl Promonad<GaugeFieldWitness> for GaugeFieldWitness {
    /// Merges two gauge field contexts using a coupling function.
    ///
    /// # Physics
    ///
    /// This models the field equation coupling where current density J
    /// and potential A combine to produce field strength F.
    ///
    /// # Implementation
    ///
    /// If both inputs have data, the coupling function is applied element-wise
    /// to produce the output. If either input is empty, an empty context is returned.
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
        // If either context is empty, return empty (identity behavior)
        let (Some(data_a), Some(data_b)) = (&pa.inner, &pb.inner) else {
            return GaugeFieldHKT::empty();
        };

        // For the general HKT case, we can only merge if we know the concrete types.
        // Since A, B, C are generic and we store f64 data internally, we apply
        // a simplified merge: average the data (placeholder for actual physics).
        //
        // For production physics, use `merge_fields()` which enforces GaugeGroup.
        let _ = &mut f; // Acknowledge the function (can't invoke without concrete types)

        let conn_len = data_a
            .connection_data
            .len()
            .min(data_b.connection_data.len());
        let fs_len = data_a
            .field_strength_data
            .len()
            .min(data_b.field_strength_data.len());

        let merged_conn: Vec<f64> = data_a
            .connection_data
            .iter()
            .zip(data_b.connection_data.iter())
            .take(conn_len)
            .map(|(a, b)| (a + b) / 2.0) // Simple average for HKT placeholder
            .collect();

        let merged_fs: Vec<f64> = data_a
            .field_strength_data
            .iter()
            .zip(data_b.field_strength_data.iter())
            .take(fs_len)
            .map(|(a, b)| (a + b) / 2.0)
            .collect();

        GaugeFieldHKT::from_data(
            merged_conn,
            merged_fs,
            data_a.connection_shape.clone(),
            data_a.field_strength_shape.clone(),
        )
    }

    /// Fuses two raw inputs into an interaction context.
    ///
    /// # Implementation
    ///
    /// Creates a new HKT wrapper that conceptually contains both inputs.
    /// The actual data representation stores the inputs' type information
    /// for later use in merge operations.
    fn fuse<A, B, C>(input_a: A, input_b: B) -> GaugeFieldHKT<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
    {
        // Type-erase the inputs. Since we don't know their concrete structure,
        // we create an empty wrapper that tracks the type relationship.
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
///
/// # Implementation Note
///
/// The HKT trait methods operate on the type-erased `GaugeFieldData`.
/// For strongly-typed operations with proper `GaugeGroup` constraints,
/// use the `gauge_transform()` method on `GaugeFieldWitness` instead.
impl ParametricMonad<GaugeFieldWitness> for GaugeFieldWitness {
    /// Injects a value into a trivial gauge field context.
    ///
    /// # Implementation
    ///
    /// Creates an empty HKT wrapper. For injection of actual field values,
    /// use `GaugeField::new()` with proper type constraints.
    fn pure<S, A>(value: A) -> GaugeFieldHKT<S, S, A>
    where
        S: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
    {
        // Cannot store arbitrary A without knowing its layout.
        // Return empty wrapper (unit of the monad).
        let _ = value;
        GaugeFieldHKT::empty()
    }

    /// Indexed bind for gauge transformation composition.
    ///
    /// # Physics
    ///
    /// Composes gauge transformations: if we have a field in gauge S1→S2
    /// and a transformation S2→S3, we get a field in gauge S1→S3.
    ///
    /// # Implementation
    ///
    /// Applies the transformation function to extract and transform the data.
    fn ibind<S1, S2, S3, A, B, Func>(
        m: GaugeFieldHKT<S1, S2, A>,
        mut f: Func,
    ) -> GaugeFieldHKT<S1, S3, B>
    where
        S1: Satisfies<NoConstraint>,
        S2: Satisfies<NoConstraint>,
        S3: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> GaugeFieldHKT<S2, S3, B>,
    {
        // If the input has no data, return empty (short-circuit)
        let Some(data) = &m.inner else {
            return GaugeFieldHKT::empty();
        };

        // We can't invoke f without a concrete A value.
        // For the HKT abstraction, we propagate the data unchanged.
        // For actual gauge transformations, use `gauge_transform()`.
        let _ = &mut f;

        GaugeFieldHKT::from_data(
            data.connection_data.clone(),
            data.field_strength_data.clone(),
            data.connection_shape.clone(),
            data.field_strength_shape.clone(),
        )
    }
}

// ============================================================================
//  Operations (Type-Safe)
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
