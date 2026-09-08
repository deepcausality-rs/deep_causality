/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Discrete interior product (contraction) on **simplicial** cochains, by Whitney forms.
//!
//! # Why not the star–wedge identity used on the lattice
//!
//! [`interior_product`](super::interior_product) on a cubical lattice implements Hirani's
//! `i_X ω = (−1)^{k(D−k)} ⋆(⋆ω ∧ X♭)`. That route needs three things a simplicial complex here does
//! not have: a simplicial wedge, a transport carrying dual cochains back onto primal cells, and —
//! decisively — a **correct diagonal Hodge star at intermediate grades**.
//!
//! The third is the blocker, and it is measurable. `build_lumped_mass_hodge_star` returns the
//! dual/primal volume ratio only at the two endpoint grades: at `k = 0` it returns the barycentric
//! dual volume, at `k = n` the reciprocal of the primal volume, and at **every grade in between it
//! returns the primal volume `|σ|` itself**. On a regular tetrahedron scaled by `h`, `⋆₂` measures
//! as `O(h²)` where a Hodge star on 2-forms in three dimensions must scale as `O(h⁻¹)`. Composing
//! that into `⋆(⋆B ∧ v)` yields a number, and the number is wrong by a factor of `h³`.
//!
//! So this module takes the other standard discretisation, which needs no Hodge star at all.
//!
//! # Whitney forms
//!
//! On each tetrahedron the cochains are interpolated by the lowest-order Whitney basis (Whitney,
//! *Geometric Integration Theory*, 1957; Bossavit, *Computational Electromagnetism*, 1998 §5), and
//! the contraction is performed on the reconstructed vector fields:
//!
//! ```text
//! W¹_{ij}  = λ_i ∇λ_j − λ_j ∇λ_i                                    (edge  i<j)
//! W²_{ijk} = 2 (λ_i ∇λ_j × ∇λ_k + λ_j ∇λ_k × ∇λ_i + λ_k ∇λ_i × ∇λ_j) (face  i<j<k)
//! ```
//!
//! These are dual to integration over the cells they are named for — `∫_{e_ij} W¹_{ij}·dl = 1` and
//! `∫_{f_ijk} W²_{ijk}·dA = 1`, with every other cell of the same grade integrating to zero — so a
//! cochain's coefficients *are* the interpolation weights, and the interpolation **reproduces
//! constant fields exactly**. That last property is the oracle the tests use.
//!
//! # What is computed
//!
//! For `k = 2` in three dimensions, with `V` the vector field of the 1-cochain `x_flat` and `B` the
//! vector field of the 2-cochain `omega`:
//!
//! ```text
//! i_V B = (B × V)♭
//! ```
//!
//! read off the components: `i_V (B_x dy∧dz + B_y dz∧dx + B_z dx∧dy)` expands to
//! `dx (B_y V_z − B_z V_y) + dy (B_z V_x − B_x V_z) + dz (B_x V_y − B_y V_x)`, which is `B × V`.
//! The result is returned as a primal 1-cochain: the circulation of `B × V` along each edge.
//!
//! Both fields are reconstructed at the barycentre of each tetrahedron, contracted there, and the
//! product integrated along each of the tetrahedron's six edges by the midpoint rule. An edge
//! shared by several tetrahedra takes the mean of their contributions. Every step is exact for
//! constant `V` and `B`: the reconstruction reproduces them, their cross product is constant, the
//! midpoint rule is exact for a constant integrand, and the mean of equal values is that value.
//!
//! # Grade restriction
//!
//! Only `k = 2` on a three-dimensional complex is implemented, which is the contraction the ideal
//! MHD induction equation needs. `k = 1` and `k = 3` are each a different projection — onto
//! vertices and onto faces — and are refused rather than approximated, because an operator that
//! has not been checked against a closed form has no business in a solver.

use std::collections::BTreeMap;

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;

use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};
use crate::types::manifold::Manifold;
use crate::types::simplicial_complex::SimplicialComplex;

/// The ambient and intrinsic dimension this module implements.
const DIM: usize = 3;

impl<R> Manifold<SimplicialComplex<R>, R>
where
    R: RealField + FromPrimitive + Default + PartialEq + core::fmt::Debug,
{
    /// The discrete interior product `i_X ω` of a 1-cochain with a 2-cochain on a
    /// three-dimensional simplicial complex, returned as a primal 1-cochain.
    ///
    /// See the module documentation for the construction and for why it is Whitney interpolation
    /// rather than the star–wedge identity the cubical backend uses.
    ///
    /// # Errors
    ///
    /// * [`TopologyError::InvalidGradeOperation`] when `k != 2`, or when the complex is not
    ///   three-dimensional.
    /// * [`TopologyError::DimensionMismatch`] when `x_flat` is not a 1-cochain of length
    ///   `n₁`, or `omega` is not a 2-cochain of length `n₂`.
    /// * [`TopologyError::InvalidInput`] when the complex carries no coordinates, when the
    ///   ambient dimension is not three, or when a tetrahedron is degenerate.
    pub fn interior_product(
        &self,
        x_flat: &CausalTensor<R>,
        omega: &CausalTensor<R>,
        k: usize,
    ) -> Result<CausalTensor<R>, TopologyError> {
        let complex = &self.complex;
        let skeletons = complex.skeletons();

        if k != 2 {
            return Err(TopologyError(TopologyErrorEnum::InvalidGradeOperation(
                format!(
                    "the simplicial interior product is implemented for k = 2 only (the \
                     contraction the induction equation needs); got k = {k}. Grades 1 and 3 \
                     project onto vertices and onto faces respectively, which are different \
                     constructions and are refused rather than approximated"
                ),
            )));
        }
        if skeletons.len() != DIM + 1 {
            return Err(TopologyError(TopologyErrorEnum::InvalidGradeOperation(
                format!(
                    "the simplicial interior product needs a 3-dimensional complex (4 skeletons); \
                     this complex has {} skeleton(s), so it is {}-dimensional",
                    skeletons.len(),
                    skeletons.len().saturating_sub(1)
                ),
            )));
        }

        let n1 = skeletons[1].simplices().len();
        let n2 = skeletons[2].simplices().len();
        if x_flat.len() != n1 {
            return Err(TopologyError(TopologyErrorEnum::DimensionMismatch(
                format!(
                    "contraction field: expected {n1} grade-1 coefficients, got {}",
                    x_flat.len()
                ),
            )));
        }
        if omega.len() != n2 {
            return Err(TopologyError(TopologyErrorEnum::DimensionMismatch(
                format!(
                    "form operand: expected {n2} grade-2 coefficients, got {}",
                    omega.len()
                ),
            )));
        }

        let geometry = complex.geometric_data.as_ref().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "the simplicial interior product interpolates with Whitney forms and so needs \
                 vertex coordinates; this complex was built without geometry"
                    .to_string(),
            ))
        })?;
        if geometry.ambient_dim != DIM {
            return Err(TopologyError(TopologyErrorEnum::InvalidInput(format!(
                "the simplicial interior product needs a 3-dimensional ambient space; the \
                 complex carries coordinates in {} dimensions",
                geometry.ambient_dim
            ))));
        }
        let coords = &geometry.coords;

        // Index lookups for the two cochain grades, keyed by the sorted vertex tuples the
        // skeletons store.
        let mut edge_index: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for (i, e) in skeletons[1].simplices().iter().enumerate() {
            let v = e.vertices();
            if v.len() == 2 {
                edge_index.insert((v[0], v[1]), i);
            }
        }
        let mut face_index: BTreeMap<(usize, usize, usize), usize> = BTreeMap::new();
        for (i, f) in skeletons[2].simplices().iter().enumerate() {
            let v = f.vertices();
            if v.len() == 3 {
                face_index.insert((v[0], v[1], v[2]), i);
            }
        }

        let v_vals = x_flat.as_slice();
        let b_vals = omega.as_slice();

        let quarter = R::from_f64(0.25).ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "0.25 has no image in the scalar type".to_string(),
            ))
        })?;
        let half = R::from_f64(0.5).ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "0.5 has no image in the scalar type".to_string(),
            ))
        })?;

        let mut acc = vec![R::zero(); n1];
        let mut hits = vec![0usize; n1];

        for tet in skeletons[DIM].simplices() {
            let tv = tet.vertices();
            if tv.len() != DIM + 1 {
                continue;
            }
            let mut points = [[R::zero(); DIM]; 4];
            for (slot, &vertex) in points.iter_mut().zip(tv.iter()) {
                let base = vertex * DIM;
                if base + DIM > coords.len() {
                    return Err(TopologyError(TopologyErrorEnum::InvalidInput(format!(
                        "vertex {vertex} has no coordinates: the coordinate slab holds {} \
                         scalars for a {DIM}-dimensional ambient space",
                        coords.len()
                    ))));
                }
                slot.copy_from_slice(&coords[base..base + DIM]);
            }

            let grad = barycentric_gradients(&points).ok_or_else(|| {
                TopologyError(TopologyErrorEnum::InvalidInput(format!(
                    "tetrahedron {tv:?} is degenerate: its four vertices are coplanar, so it has \
                     no barycentric coordinates to interpolate with"
                )))
            })?;

            // V at the barycentre, from the Whitney 1-forms: at λ = ¼ everywhere,
            // W¹_{ij} = (∇λ_j − ∇λ_i) / 4.
            let mut v_field = [R::zero(); DIM];
            for i in 0..4 {
                for j in (i + 1)..4 {
                    let Some(&e) = edge_index.get(&(tv[i], tv[j])) else {
                        return Err(TopologyError(TopologyErrorEnum::DimensionMismatch(
                            format!(
                                "edge ({}, {}) of tetrahedron {tv:?} is absent from the 1-skeleton",
                                tv[i], tv[j]
                            ),
                        )));
                    };
                    let w = v_vals[e] * quarter;
                    for d in 0..DIM {
                        v_field[d] += w * (grad[j][d] - grad[i][d]);
                    }
                }
            }

            // B at the barycentre, from the Whitney 2-forms: at λ = ¼ everywhere,
            // W²_{ijk} = ½ (∇λ_j × ∇λ_k + ∇λ_k × ∇λ_i + ∇λ_i × ∇λ_j).
            let mut b_field = [R::zero(); DIM];
            for i in 0..4 {
                for j in (i + 1)..4 {
                    for l in (j + 1)..4 {
                        let Some(&f) = face_index.get(&(tv[i], tv[j], tv[l])) else {
                            return Err(TopologyError(TopologyErrorEnum::DimensionMismatch(
                                format!(
                                    "face ({}, {}, {}) of tetrahedron {tv:?} is absent from the \
                                     2-skeleton",
                                    tv[i], tv[j], tv[l]
                                ),
                            )));
                        };
                        let jk = cross(&grad[j], &grad[l]);
                        let ki = cross(&grad[l], &grad[i]);
                        let ij = cross(&grad[i], &grad[j]);
                        let w = b_vals[f] * half;
                        for d in 0..DIM {
                            b_field[d] += w * (jk[d] + ki[d] + ij[d]);
                        }
                    }
                }
            }

            // i_V B = (B × V)♭, evaluated once per tetrahedron and integrated along each of its
            // six edges by the midpoint rule.
            let contracted = cross(&b_field, &v_field);
            for i in 0..4 {
                for j in (i + 1)..4 {
                    let e = edge_index[&(tv[i], tv[j])];
                    let mut circulation = R::zero();
                    for d in 0..DIM {
                        circulation += contracted[d] * (points[j][d] - points[i][d]);
                    }
                    acc[e] += circulation;
                    hits[e] += 1;
                }
            }
        }

        // An edge shared by several tetrahedra takes their mean. An edge in no tetrahedron at all
        // keeps its zero: there is no volume on which to interpolate, and the contraction of a
        // field that is not defined there is not a number this can invent.
        for (value, &count) in acc.iter_mut().zip(hits.iter()) {
            if count > 1 {
                let n = R::from_usize(count).ok_or_else(|| {
                    TopologyError(TopologyErrorEnum::InvalidInput(format!(
                        "the incidence count {count} has no image in the scalar type"
                    )))
                })?;
                *value /= n;
            }
        }

        CausalTensor::new(acc, vec![n1]).map_err(|e| {
            TopologyError(TopologyErrorEnum::TensorError(format!(
                "interior product result: {e}"
            )))
        })
    }
}

/// The gradients of the four barycentric coordinates of a tetrahedron.
///
/// `λ_i` is the affine function that is one at vertex `i` and zero at the other three, so the four
/// of them are the columns of the inverse of `[[1, x_j, y_j, z_j]]_j`. Returns `None` when that
/// matrix is singular, which is exactly a degenerate (coplanar) tetrahedron.
fn barycentric_gradients<R>(points: &[[R; DIM]; 4]) -> Option<[[R; DIM]; 4]>
where
    R: RealField + FromPrimitive + Default + PartialEq + core::fmt::Debug,
{
    // λ_i is affine and takes the value δ_ij at vertex j, so writing λ_i(x) = c_i · [1, x, y, z]
    // and stacking the vertices as M with row j equal to [1, x_j, y_j, z_j] gives C Mᵀ = I. Hence
    // C = (Mᵀ)⁻¹ = (M⁻¹)ᵀ, and c_i is the i-th **column** of M⁻¹: its entries 1..3 are ∇λ_i.
    let mut m = [[R::zero(); 4]; 4];
    for (row, p) in m.iter_mut().zip(points.iter()) {
        row[0] = R::one();
        row[1..4].copy_from_slice(p);
    }
    let inv = deep_causality_linear::inverse_4x4(&m).ok()?;
    let mut grad = [[R::zero(); DIM]; 4];
    for (i, g) in grad.iter_mut().enumerate() {
        for (d, slot) in g.iter_mut().enumerate() {
            *slot = inv[d + 1][i];
        }
    }
    Some(grad)
}

/// `a × b`.
fn cross<R: RealField>(a: &[R; DIM], b: &[R; DIM]) -> [R; DIM] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
