/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT witness and trait implementations for LatticeGaugeField.
//!
//! This module provides Functor, Pure, and Monad implementations for LatticeGaugeField,
//! enabling functional transformations of lattice gauge fields.
//!
//! # Architectural Note
//!
//! LatticeGaugeField<G, D, T> has constraints: G: GaugeGroup, D: const usize.
//! The HKT implementation maps over the scalar type T while preserving the
//! gauge group G and dimension D.
//!
//! Due to Rust trait system limitations, the Functor implementation operates
//! on the beta parameter only. For full field transformations, use the
//! type-safe `map_field()` method which enforces proper Clone + Default bounds.
//!
//! # Physics Interpretation
//!
//! - **Functor::fmap**: Transform coupling parameter β
//! - **Pure::pure**: Lift a value into a minimal field context
//! - **Monad::bind**: Chain field transformations

use crate::{GaugeGroup, Lattice, LatticeGaugeField, LinkVariable, TopologyError};
use deep_causality_haft::{Applicative, Functor, HKT, Monad, NoConstraint, Pure, Satisfies};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// HKT witness for LatticeGaugeField<G, D, T>.
///
/// Enables functional transformations of lattice gauge fields.
/// The witness is parameterized by the gauge group G and dimension D,
/// while the HKT operates over the scalar type T.
///
/// # Type Parameters
///
/// * `G` - Gauge group (U1, SU2, SU3, etc.)
/// * `D` - Spacetime dimension
///
/// # Implementation Note
///
/// Due to HKT trait constraints, the Functor/Monad implementations only
/// transform the beta parameter. For full link variable transformations,
/// use the type-safe helper methods like `map_field()`, `zip_with()`, etc.
#[derive(Debug, Clone, Copy, Default)]
pub struct LatticeGaugeFieldWitness<G: GaugeGroup, const D: usize>(PhantomData<G>);

impl<G: GaugeGroup, const D: usize> LatticeGaugeFieldWitness<G, D> {
    /// Create a new witness.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

// ============================================================================
// HKT Implementation
// ============================================================================

impl<G: GaugeGroup, const D: usize> HKT for LatticeGaugeFieldWitness<G, D> {
    type Constraint = NoConstraint;
    type Type<T>
        = LatticeGaugeField<G, D, T>
    where
        T: Satisfies<NoConstraint>;
}

// ============================================================================
// Functor Implementation
// ============================================================================

/// Functor implementation: map over the beta parameter.
///
/// # Limitation
///
/// Due to trait bound constraints, this only transforms beta.
/// For full field transformations, use `LatticeGaugeFieldWitness::map_field()`.
impl<G: GaugeGroup, const D: usize> Functor<LatticeGaugeFieldWitness<G, D>>
    for LatticeGaugeFieldWitness<G, D>
{
    /// Map a function over the beta parameter of the lattice gauge field.
    ///
    /// Note: This creates an empty field with transformed beta.
    /// For full link variable transformation, use `map_field()`.
    fn fmap<A, B, F>(fa: LatticeGaugeField<G, D, A>, mut f: F) -> LatticeGaugeField<G, D, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> B,
    {
        // Consume the field to get components
        let (lattice, _links, beta_a) = fa.into_parts();
        let beta = f(beta_a);

        // Create empty field with new beta (links cannot be transformed without Clone)
        LatticeGaugeField::from_links_unchecked(lattice, HashMap::new(), beta)
    }
}

// ============================================================================
// Applicative Implementation
// ============================================================================

/// Applicative implementation: apply a wrapped function to a wrapped value.
impl<G: GaugeGroup, const D: usize> Applicative<LatticeGaugeFieldWitness<G, D>>
    for LatticeGaugeFieldWitness<G, D>
{
    /// Apply a function wrapped in a field to a value wrapped in a field.
    ///
    /// Combines the beta parameter (function) from `fab` with the beta parameter
    /// (value) from `fa`. Links are discarded/reset as per HKT limitation.
    fn apply<A, B, F>(
        fab: LatticeGaugeField<G, D, F>,
        fa: LatticeGaugeField<G, D, A>,
    ) -> LatticeGaugeField<G, D, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnOnce(A) -> B + Satisfies<NoConstraint>,
    {
        let (_lattice_f, _links_f, f_beta) = fab.into_parts();
        let (lattice_a, _links_a, a_beta) = fa.into_parts();

        // Apply function to value
        let b_beta = f_beta(a_beta);

        // Create new field
        LatticeGaugeField::from_links_unchecked(lattice_a, HashMap::new(), b_beta)
    }
}

// ============================================================================
// Pure Implementation
// ============================================================================

/// Pure implementation: lift a value into a minimal field context.
impl<G: GaugeGroup, const D: usize> Pure<LatticeGaugeFieldWitness<G, D>>
    for LatticeGaugeFieldWitness<G, D>
{
    /// Lift a value into a lattice gauge field context.
    ///
    /// Creates a minimal gauge field with no links and the given value as beta.
    fn pure<T>(value: T) -> LatticeGaugeField<G, D, T>
    where
        T: Satisfies<NoConstraint>,
    {
        let shape = [1usize; D];
        let lattice = Arc::new(Lattice::new(shape, [true; D]));
        let links = HashMap::new();

        LatticeGaugeField::from_links_unchecked(lattice, links, value)
    }
}

// ============================================================================
// Monad Implementation
// ============================================================================

/// Monad implementation: chain field transformations.
///
/// # Physics Interpretation
///
/// This enables chaining gauge field operations where each step
/// produces a new field configuration.
impl<G: GaugeGroup, const D: usize> Monad<LatticeGaugeFieldWitness<G, D>>
    for LatticeGaugeFieldWitness<G, D>
{
    /// Monadic bind for chaining lattice gauge field transformations.
    fn bind<A, B, F>(ma: LatticeGaugeField<G, D, A>, f: F) -> LatticeGaugeField<G, D, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> LatticeGaugeField<G, D, B>,
    {
        let (_lattice, _links, beta_a) = ma.into_parts();
        let mut func = f;
        func(beta_a)
    }
}

// ============================================================================
// Type-Safe Operations (with proper Clone + Default bounds)
// ============================================================================

impl<G: GaugeGroup, const D: usize> LatticeGaugeFieldWitness<G, D> {
    /// Transform a lattice gauge field by mapping over all scalars.
    ///
    /// This is the production-ready method for full field transformation.
    /// Unlike the HKT Functor, this requires Clone + Default bounds.
    ///
    /// # Arguments
    ///
    /// * `field` - The input lattice gauge field
    /// * `f` - Function to apply to each scalar value
    ///
    /// # Returns
    ///
    /// A new lattice gauge field with all values transformed.
    pub fn map_field<A, B, F>(
        field: LatticeGaugeField<G, D, A>,
        mut f: F,
    ) -> LatticeGaugeField<G, D, B>
    where
        A: Clone + Default,
        B: Clone + Default,
        F: FnMut(A) -> B,
    {
        let lattice = field.lattice_arc().clone();
        let beta = f(field.beta().clone());

        // Transform all link variables
        let mut new_links = HashMap::with_capacity(field.links().len());
        for (cell, link) in field.links().iter() {
            let new_link = map_link_variable::<G, A, B, F>(link, &mut f);
            new_links.insert(cell.clone(), new_link);
        }

        LatticeGaugeField::from_links_unchecked(lattice, new_links, beta)
    }

    /// Combine two lattice gauge fields using a binary operation on scalars.
    ///
    /// # Arguments
    ///
    /// * `field_a` - First lattice gauge field
    /// * `field_b` - Second lattice gauge field
    /// * `f` - Binary function to combine scalar values
    ///
    /// # Returns
    ///
    /// A new lattice gauge field with combined values, or error if lattices don't match.
    ///
    /// # Errors
    ///
    /// Returns `TopologyError::LatticeGaugeError` if the lattices have different shapes.
    pub fn zip_with<T, F>(
        field_a: &LatticeGaugeField<G, D, T>,
        field_b: &LatticeGaugeField<G, D, T>,
        mut f: F,
    ) -> Result<LatticeGaugeField<G, D, T>, TopologyError>
    where
        T: Clone + Default,
        F: FnMut(&T, &T) -> T,
    {
        // Validate lattice shapes match
        if field_a.lattice().shape() != field_b.lattice().shape() {
            return Err(TopologyError::LatticeGaugeError(format!(
                "Lattice shape mismatch: {:?} vs {:?}",
                field_a.lattice().shape(),
                field_b.lattice().shape()
            )));
        }

        let lattice = field_a.lattice_arc().clone();
        let beta = f(field_a.beta(), field_b.beta());

        // Combine link variables
        let mut new_links = HashMap::with_capacity(field_a.links().len());
        for (cell, link_a) in field_a.links() {
            if let Some(link_b) = field_b.links().get(cell) {
                let new_link = zip_link_variables::<G, T, F>(link_a, link_b, &mut f);
                new_links.insert(cell.clone(), new_link);
            }
        }

        Ok(LatticeGaugeField::from_links_unchecked(
            lattice, new_links, beta,
        ))
    }

    /// Scale all link variable matrices by a scalar factor.
    ///
    /// # Arguments
    ///
    /// * `field` - The input lattice gauge field
    /// * `factor` - The scaling factor
    ///
    /// # Returns
    ///
    /// A new lattice gauge field with scaled link variables.
    pub fn scale_field<T>(
        field: LatticeGaugeField<G, D, T>,
        factor: T,
    ) -> LatticeGaugeField<G, D, T>
    where
        T: Clone + Default + std::ops::Mul<Output = T>,
    {
        let factor_clone = factor.clone();
        Self::map_field(field, move |x| x * factor_clone.clone())
    }

    /// Create an identity field on the given lattice.
    ///
    /// Convenience wrapper that enforces proper type constraints.
    pub fn identity_field<T>(
        lattice: Arc<Lattice<D>>,
        beta: T,
    ) -> Result<LatticeGaugeField<G, D, T>, TopologyError>
    where
        T: Clone + Default + From<f64>,
    {
        LatticeGaugeField::try_identity(lattice, beta)
    }
}

/// Map a function over all elements of a LinkVariable.
fn map_link_variable<G: GaugeGroup, A, B, F>(
    link: &LinkVariable<G, A>,
    f: &mut F,
) -> LinkVariable<G, B>
where
    A: Clone + Default,
    B: Clone + Default,
    F: FnMut(A) -> B,
{
    let n = G::matrix_dim();
    let old_data = link.as_slice();
    let new_data: Vec<B> = old_data.iter().map(|x| f(x.clone())).collect();

    let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
        .unwrap_or_else(|_| panic!("LinkVariable fmap failed for {}x{} matrix", n, n));

    LinkVariable::from_matrix_unchecked(tensor)
}

/// Combine two LinkVariables element-wise using a binary function.
fn zip_link_variables<G: GaugeGroup, T, F>(
    link_a: &LinkVariable<G, T>,
    link_b: &LinkVariable<G, T>,
    f: &mut F,
) -> LinkVariable<G, T>
where
    T: Clone + Default,
    F: FnMut(&T, &T) -> T,
{
    let n = G::matrix_dim();
    let data_a = link_a.as_slice();
    let data_b = link_b.as_slice();

    let new_data: Vec<T> = data_a
        .iter()
        .zip(data_b.iter())
        .map(|(a, b)| f(a, b))
        .collect();

    let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
        .unwrap_or_else(|_| panic!("LinkVariable zip failed for {}x{} matrix", n, n));

    LinkVariable::from_matrix_unchecked(tensor)
}

// ============================================================================
// Display
// ============================================================================

impl<G: GaugeGroup, const D: usize> std::fmt::Display for LatticeGaugeFieldWitness<G, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LatticeGaugeFieldWitness<{}, {}D>", G::name(), D)
    }
}
