/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed operations on [`LatticeGaugeField`].
//!
//! [`LatticeGaugeFieldOps`] is a zero-sized namespace carrying the field's functional surface as
//! inherent methods: `map_field` over every matrix element, `zip_with` to combine two fields on
//! one lattice, `scale_field`, and `identity_field`.
//!
//! It implements no `deep_causality_haft` trait and is not a witness. The note below the type
//! records why, and the short form is that `fmap` would have to rebuild each
//! `LinkVariable<G, B, R>`, which needs `B: Field + Copy + Default + PartialOrd + Debug`, while
//! `Functor` gives the body no bound on `B` at all. The inherent methods name the bounds each
//! operation actually needs, which the compiler enforces and no downstream crate can forge.
//!
//! Cross-algebra composition is not lost by that: it lives on `Manifold`, which is the crate's
//! central composition surface. See `extensions/hkt_manifold`.

use crate::{GaugeGroup, LatticeComplex, LatticeGaugeField, LinkVariable, TopologyError};
use deep_causality_algebra::{ComplexField, DivisionAlgebra, Field, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// Typed operations on `LatticeGaugeField<G, D, M, R>`, as a zero-sized namespace.
///
/// Parameterised by the gauge group `G`, the dimension `D` and the matrix element type `M`; the
/// methods act on the matrix elements. This is not an HKT witness — see the module note.
///
/// # Type Parameters
///
/// * `G` - Gauge group (U1, SU2, SU3, etc.)
/// * `D` - Spacetime dimension
/// * `M` - Matrix element type (TensorData)
#[derive(Debug, Clone, Copy, Default)]
pub struct LatticeGaugeFieldOps<G: GaugeGroup, const D: usize, M>(PhantomData<(G, M)>);

impl<G: GaugeGroup, const D: usize, M> LatticeGaugeFieldOps<G, D, M> {
    /// Create a new operations handle. It carries no data.
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
// bound on `B` at all, so the body cannot do the arithmetic. An empty element marker on the
// trait would not help either, since a marker grants no capability. The next-generation trait
// solver does not change it, measured on `rustc 1.100.0-nightly (bff8e12ff 2026-08-26)`.
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
    LatticeGaugeFieldOps<G, D, R>
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

        let mut new_links = HashMap::with_capacity(field.num_links());
        for (cell, link) in field.iter_links() {
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
        let mut new_links = HashMap::with_capacity(field_a.num_links());
        for (cell, link_a) in field_a.iter_links() {
            let link_b = field_b.link(&cell).ok_or_else(|| {
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
    let old_data = link.as_slice();
    let new_data: Vec<B> = old_data.iter().map(|x| f(*x)).collect();

    LinkVariable::from_matrix_unchecked(new_data)
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
    let data_a = link_a.as_slice();
    let data_b = link_b.as_slice();

    let new_data: Vec<T> = data_a
        .iter()
        .zip(data_b.iter())
        .map(|(a, b)| f(a, b))
        .collect();

    LinkVariable::from_matrix_unchecked(new_data)
}

// ============================================================================
// Display
// ============================================================================

impl<G: GaugeGroup, const D: usize, M> std::fmt::Display for LatticeGaugeFieldOps<G, D, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LatticeGaugeFieldOps<{}, {}D>", G::name(), D)
    }
}
