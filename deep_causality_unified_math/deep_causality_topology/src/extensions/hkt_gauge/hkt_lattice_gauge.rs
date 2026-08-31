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

use crate::{GaugeGroup, LatticeComplex, LatticeGaugeField, LinkVariable, TopologyError};
use deep_causality_algebra::{ComplexField, DivisionAlgebra, Field, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// HKT witness for LatticeGaugeField<G, D, M, R>.
///
/// Enables functional transformations of lattice gauge fields.
/// The witness is parameterized by the gauge group G, dimension D, and matrix type M.
/// The HKT operates over the scalar type R (beta parameter).
///
/// # Type Parameters
///
/// * `G` - Gauge group (U1, SU2, SU3, etc.)
/// * `D` - Spacetime dimension
/// * `M` - Matrix element type (TensorData)
#[derive(Debug, Clone, Copy, Default)]
pub struct LatticeGaugeFieldWitness<G: GaugeGroup, const D: usize, M>(PhantomData<(G, M)>);

impl<G: GaugeGroup, const D: usize, M> LatticeGaugeFieldWitness<G, D, M> {
    /// Create a new witness.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

// ============================================================================
// HKT trait impls are intentionally deferred.
//
// The blocker is a missing capability, not a struct-level bound and not the compiler.
//
// The struct bound `R: RealField` is not what stops a witness here. `R` is not the
// parameter a functor over this type would map; `M`, the matrix element, is, and `M`
// carries no struct-level bound. A witness over `M` with `R` fixed compiles on stable
// today:
//
//     pub struct LgfOverM<G: GaugeGroup, const D: usize, R: RealField>(PhantomData<(G, R)>);
//     impl<G: GaugeGroup, const D: usize, R: RealField> HKT for LgfOverM<G, D, R> {
//         type Type<T> = LatticeGaugeField<G, D, T, R>;
//     }
//
// What stops `Functor` is that `fmap` must rebuild each `LinkVariable<G, B, R>`, which
// needs `B: Field + Copy + Default + PartialOrd + Debug`. The trait gives the body no
// bound on `B` at all, so the body cannot do the arithmetic. An earlier version of this
// note blamed the element marker the trait used to carry; that marker is gone, and its
// removal changed nothing here, because an empty marker never granted a capability
// either. The next-generation trait solver does not change it, measured on
// `rustc 1.100.0-nightly (bff8e12ff 2026-08-26)`.
//
// The inherent methods below are therefore the right design rather than a stopgap: they
// name the real bounds, which the compiler enforces and no downstream crate can forge.
// See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §3 and §6.3.
//
// The cross-algebra composition story is preserved on `Manifold` (see
// `extensions/hkt_manifold/mod.rs`) — that is the central composition surface.
// The lattice gauge field's functional surface remains available via the inherent
// `map_field` / `zip_with` / `scale_field` / `identity_field` methods below.
// ============================================================================

// ============================================================================
// Type-Safe Operations (with proper Clone + Default bounds)
// ============================================================================

impl<G: GaugeGroup, const D: usize, R: RealField + FromPrimitive + ToPrimitive>
    LatticeGaugeFieldWitness<G, D, R>
{
    /// Transform a lattice gauge field by mapping over all scalars (Matrix elements).
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
        field: LatticeGaugeField<G, D, A, R>,
        mut f: F,
    ) -> LatticeGaugeField<G, D, B, R>
    where
        A: Field + Copy + Default + PartialOrd + Debug + Clone,
        B: Field + Copy + Default + PartialOrd + Debug + Clone,
        F: FnMut(A) -> B,
    {
        let lattice = field.lattice_arc().clone();
        // `f` maps the matrix element `A` to `B`. The coupling `beta` has type `R`, which this
        // map leaves alone, so it carries over unchanged.
        let beta = *field.beta();

        let mut new_links = HashMap::with_capacity(field.links().len());
        for (cell, link) in field.links().iter() {
            let new_link = map_link_variable::<G, A, B, R, F>(link, &mut f);
            new_links.insert(cell.clone(), new_link);
        }

        LatticeGaugeField::from_links_unchecked(lattice, new_links, beta, ())
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
        field_a: &LatticeGaugeField<G, D, T, R>,
        field_b: &LatticeGaugeField<G, D, T, R>,
        mut f: F,
    ) -> Result<LatticeGaugeField<G, D, T, R>, TopologyError>
    where
        T: Field + Copy + Default + PartialOrd + Clone + std::fmt::Debug,
        F: FnMut(&T, &T) -> T,
    {
        // Validate lattice shapes match
        if field_a.lattice().shape() != field_b.lattice().shape() {
            return Err(TopologyError::LatticeGaugeError(format!(
                "LatticeComplex shape mismatch: {:?} vs {:?}",
                field_a.lattice().shape(),
                field_b.lattice().shape()
            )));
        }

        let lattice = field_a.lattice_arc().clone();
        // `f` combines matrix elements of type `T`; the coupling has type `R`, which `f` cannot
        // be applied to. The two fields are required to share a lattice shape, so taking
        // `field_a`'s coupling is the only reading that does not invent a value.
        let beta = *field_a.beta();

        // Combine link variables
        let mut new_links = HashMap::with_capacity(field_a.links().len());
        for (cell, link_a) in field_a.links() {
            let link_b = field_b.links().get(cell).ok_or_else(|| {
                TopologyError::LatticeGaugeError(format!(
                    "Missing link for cell {:?} during zip_with",
                    cell
                ))
            })?;
            let new_link = zip_link_variables::<G, T, R, F>(link_a, link_b, &mut f);
            new_links.insert(cell.clone(), new_link);
        }

        Ok(LatticeGaugeField::from_links_unchecked(
            lattice,
            new_links,
            beta,
            (),
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
        field: LatticeGaugeField<G, D, T, R>,
        factor: T,
    ) -> LatticeGaugeField<G, D, T, R>
    where
        T: Field
            + Copy
            + Default
            + PartialOrd
            + Clone
            + std::ops::Mul<Output = T>
            + std::fmt::Debug,
    {
        let factor_clone = factor;
        Self::map_field(field, move |x| x * factor_clone)
    }

    /// Create an identity field on the given lattice.
    ///
    /// Convenience wrapper that enforces proper type constraints.
    pub fn identity_field<T>(
        lattice: Arc<LatticeComplex<D, R>>,
        beta: R,
    ) -> Result<LatticeGaugeField<G, D, T, R>, TopologyError>
    where
        T: Field
            + Copy
            + Default
            + PartialOrd
            + ComplexField<R>
            + DivisionAlgebra<R>
            + std::fmt::Debug,
    {
        LatticeGaugeField::try_identity(lattice, beta)
    }
}

/// Map a function over all elements of a LinkVariable.
fn map_link_variable<G: GaugeGroup, A, B, R, F>(
    link: &LinkVariable<G, A, R>,
    f: &mut F,
) -> LinkVariable<G, B, R>
where
    A: Field + Copy + Default + PartialOrd + Clone,
    B: Field + Copy + Default + PartialOrd + Clone,
    R: RealField,
    F: FnMut(A) -> B,
    A: Debug,
{
    let n = G::matrix_dim();

    let old_data = link.as_slice();
    let new_data: Vec<B> = old_data.iter().map(|x| f(*x)).collect();

    let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
        .unwrap_or_else(|_| panic!("LinkVariable fmap failed for {}x{} matrix", n, n));

    LinkVariable::from_matrix_unchecked(tensor)
}

/// Combine two LinkVariables element-wise using a binary function.
fn zip_link_variables<G: GaugeGroup, T, R, F>(
    link_a: &LinkVariable<G, T, R>,
    link_b: &LinkVariable<G, T, R>,
    f: &mut F,
) -> LinkVariable<G, T, R>
where
    T: Field + Copy + Default + PartialOrd + Clone,
    R: RealField,
    F: FnMut(&T, &T) -> T,
    T: Debug,
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

impl<G: GaugeGroup, const D: usize, M> std::fmt::Display for LatticeGaugeFieldWitness<G, D, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LatticeGaugeFieldWitness<{}, {}D>", G::name(), D)
    }
}
