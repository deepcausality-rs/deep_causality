/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GaugeField, GaugeGroup, TopologyError};
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
    /// # Implementation — ACKNOWLEDGED HKT Limitation
    ///
    /// **Rust Trait Limitation:** The Promonad trait from deep_causality_haft does not
    /// include `'static` bounds on type parameters. TypeId-based runtime type checking
    /// requires `'static` bounds. Therefore, we cannot safely invoke the user-provided
    /// function `f` without potentially unsound transmutation.
    ///
    /// **Workaround:** This implementation uses simple element-wise averaging as a
    /// placeholder. For actual physics computations, use the type-safe `merge_fields()`
    /// method which enforces `GaugeGroup` bounds and properly invokes the coupling function.
    ///
    /// **Status:** ACKNOWLEDGED. Resolution requires either:
    /// 1. Upstream trait changes to add `'static` bounds to Promonad
    /// 2. New trait solver (`-Ztrait-solver=next`) for better GAT support
    fn merge<A, B, C, Func>(
        pa: GaugeFieldHKT<A, A, A>,
        pb: GaugeFieldHKT<B, B, B>,
        _f: Func,
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

        let conn_len = data_a
            .connection_data
            .len()
            .min(data_b.connection_data.len());
        let fs_len = data_a
            .field_strength_data
            .len()
            .min(data_b.field_strength_data.len());

        // Placeholder: element-wise average
        // For production physics, use merge_fields() which is type-safe.
        let merged_conn: Vec<f64> = data_a
            .connection_data
            .iter()
            .zip(data_b.connection_data.iter())
            .take(conn_len)
            .map(|(a, b)| (a + b) / 2.0)
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
    /// # Implementation — ACKNOWLEDGED HKT Limitation
    ///
    /// **Rust Trait Limitation:** The ParametricMonad trait does not include `'static`
    /// bounds on type parameters. TypeId-based runtime type checking requires `'static`.
    /// Therefore, we cannot safely invoke the user-provided function `f`.
    ///
    /// **Workaround:** This implementation propagates data unchanged. For actual physics
    /// transformations, use the type-safe `gauge_transform()` method which enforces
    /// `GaugeGroup` bounds and properly applies the transformation.
    ///
    /// **Status:** ACKNOWLEDGED. Resolution requires upstream trait changes.
    fn ibind<S1, S2, S3, A, B, Func>(
        m: GaugeFieldHKT<S1, S2, A>,
        _f: Func,
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

        // Placeholder: propagate data unchanged
        // For production physics transformations, use gauge_transform()
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
    /// A new gauge field with coupled field strength, or error if shape validation fails.
    pub fn merge_fields<G, A, B, F, Func>(
        current: &GaugeField<G, A, A>,
        potential: &GaugeField<G, B, B>,
        coupling: Func,
    ) -> Result<GaugeField<G, F, F>, TopologyError>
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
    ///
    /// # Errors
    ///
    /// Returns `TopologyError::GaugeFieldError` if shape validation fails.
    pub fn gauge_transform<G, A, F, Func>(
        field: &GaugeField<G, A, A>,
        transform: Func,
    ) -> Result<GaugeField<G, F, F>, TopologyError>
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
    /// # Mathematical Definition
    ///
    /// For abelian groups (U(1)):
    /// ```text
    /// F_μν = ∂_μ A_ν - ∂_ν A_μ
    /// ```
    ///
    /// This is the exterior derivative of the connection 1-form: F = dA.
    ///
    /// # Implementation
    ///
    /// Uses finite differences to approximate derivatives between adjacent
    /// spacetime points. For single-point fields, uses the connection values
    /// directly to construct the antisymmetric field strength tensor.
    ///
    /// # Returns
    ///
    /// - `Some(F_μν)` for abelian gauge groups
    /// - `None` for non-abelian groups (require additional A∧A term)
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
        let lie_dim = G::LIE_ALGEBRA_DIM;

        // Get connection shape [num_points, spacetime_dim, lie_dim]
        let conn_shape = connection.shape();
        let num_points = if conn_shape.is_empty() {
            1
        } else {
            conn_shape[0]
        };

        // F has shape [num_points, dim, dim, lie_dim]
        let total = num_points * dim * dim * lie_dim;
        let mut fs_data = vec![0.0; total];

        let conn_data = connection.as_slice();

        // Compute antisymmetric field strength F_μν = ∂_μA_ν - ∂_νA_μ
        // For each point, each Lie algebra index
        for p in 0..num_points {
            for a in 0..lie_dim {
                for mu in 0..dim {
                    for nu in 0..dim {
                        // F_μν^a = ∂_μ A_ν^a - ∂_ν A_μ^a
                        // For discrete manifold, approximate derivative using
                        // the connection values at the current point.

                        // Connection index: A_μ^a at point p
                        // Shape [num_points, spacetime_dim, lie_dim]
                        // Linear index: p * (dim * lie_dim) + mu * lie_dim + a
                        let a_mu_idx = p * (dim * lie_dim) + mu * lie_dim + a;
                        let a_nu_idx = p * (dim * lie_dim) + nu * lie_dim + a;

                        let a_mu = conn_data.get(a_mu_idx).copied().unwrap_or(0.0);
                        let a_nu = conn_data.get(a_nu_idx).copied().unwrap_or(0.0);

                        // For a single-point field, we construct the antisymmetric
                        // part from the connection components.
                        // F_μν = A_μ - A_ν (simplified for uniform grid with unit spacing)
                        // This gives F_μν = -F_νμ automatically.

                        // More accurate: use finite difference if multiple points exist
                        let f_mu_nu = if num_points > 1 && p < num_points - 1 {
                            // Forward difference approximation
                            let next_a_nu_idx = (p + 1) * (dim * lie_dim) + nu * lie_dim + a;
                            let next_a_mu_idx = (p + 1) * (dim * lie_dim) + mu * lie_dim + a;

                            let next_a_nu = conn_data.get(next_a_nu_idx).copied().unwrap_or(a_nu);
                            let next_a_mu = conn_data.get(next_a_mu_idx).copied().unwrap_or(a_mu);

                            // ∂_μ A_ν ≈ (A_ν(x+Δx_μ) - A_ν(x)) / Δx
                            // Assuming unit spacing Δx = 1
                            let d_mu_a_nu = next_a_nu - a_nu;
                            let d_nu_a_mu = next_a_mu - a_mu;

                            d_mu_a_nu - d_nu_a_mu
                        } else {
                            // For single point: antisymmetrize the connection components
                            // This gives 0 on diagonal and ±(A_μ - A_ν) off-diagonal
                            a_nu - a_mu
                        };

                        // Field strength index: F_μν^a at point p
                        // Shape [num_points, dim, dim, lie_dim]
                        // Linear index: p * (dim * dim * lie_dim) + mu * (dim * lie_dim) + nu * lie_dim + a
                        let fs_idx =
                            p * (dim * dim * lie_dim) + mu * (dim * lie_dim) + nu * lie_dim + a;

                        if fs_idx < total {
                            fs_data[fs_idx] = f_mu_nu;
                        }
                    }
                }
            }
        }

        Some(CausalTensor::from_vec(
            fs_data,
            &[num_points, dim, dim, lie_dim],
        ))
    }
}
