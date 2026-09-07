/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AlfvenSpeed, MagneticPressure};
use crate::{Density, PhysicalField, PhysicsError};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_multivector::MultiVector;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{SimplicialComplex, SimplicialManifold};
use std::collections::HashMap;

/// Calculates the characteristic speed of Alfven waves.
/// $$ v_A = \frac{B}{\sqrt{\mu_0 \rho}} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$ (Uses magnitude $|B|$).
/// *   `density` - Plasma density $\rho$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<AlfvenSpeed<R>, PhysicsError>` - Alfven speed $v_A$.
pub fn alfven_speed_kernel<R>(
    b_field: &PhysicalField<R>,
    density: &Density<R>,
    permeability: R,
) -> Result<AlfvenSpeed<R>, PhysicsError>
where
    R: RealField,
{
    let b_mag = b_field.inner().squared_magnitude().sqrt();
    let rho = density.value();

    if permeability <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permeability must be positive".into(),
        ));
    }

    if rho < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Density cannot be negative".into(),
        ));
    }

    if rho == R::zero() {
        return Err(PhysicsError::Singularity(
            "Zero density in Alfven speed".into(),
        ));
    }

    let denom = (permeability * rho).sqrt();
    let va = b_mag / denom;

    AlfvenSpeed::new(va)
}

/// Calculates magnetic pressure.
/// $$ P_B = \frac{B^2}{2\mu_0} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<MagneticPressure<R>, PhysicsError>` - Magnetic pressure $P_B$.
pub fn magnetic_pressure_kernel<R>(
    b_field: &PhysicalField<R>,
    permeability: R,
) -> Result<MagneticPressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let b_sq = b_field.inner().squared_magnitude();

    if permeability <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permeability must be positive".into(),
        ));
    }

    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let pb = b_sq / (two * permeability);
    MagneticPressure::new(pb)
}

/// Calculates the time evolution of the magnetic field (Frozen-in flux).
/// $$ \frac{\partial \mathbf{B}}{\partial t} = \nabla \times (\mathbf{v} \times \mathbf{B}) $$
///
/// **Geometric Algebra Implementation**:
/// In the language of differential forms/GA on a Manifold:
/// $$ \partial_t B = -d(i_v B) $$
/// where $B$ is a 2-form (flux), $v$ is a vector field (represented as a 1-form),
/// $i_v$ is interior product (contraction), and $d$ is exterior derivative.
///
/// This implementation relies on the identity:
/// $$ i_v B = \star (v \wedge \star B) $$
/// (valid for 3D manifolds where $v$ and $\star B$ are 1-forms).
///
/// # Arguments
/// *   `v_manifold` - Manifold containing the velocity field $v$ (1-form).
/// *   `b_manifold` - Manifold containing the magnetic flux 2-form $B$.
///
/// # Returns
/// *   `Result<CausalTensor<R>, PhysicsError>` — the rate of change of `B`, a 2-form.
///
/// # Domain, and the Hodge-star convention this needs
///
/// **Three dimensions only.** The chain applies `hodge_star_operators()[2]` twice: once as `⋆` on
/// 2-forms and once as `⋆` on `(n−1)`-forms. Those are the same operator exactly when `2 = n − 1`.
/// In 2D the second application would need `⋆` on 1-forms, so the identity above does not close.
///
/// **The operators must be degree-changing.** This kernel needs `⋆₂ : Λ² → Λ¹`, shape `(n₁, n₂)`,
/// and `d₁ : Λ¹ → Λ²`, shape `(n₂, n₁)`. `deep_causality_topology` ships the *diagonal* lumped-mass
/// star instead — `⋆_k : Cᵏ(primal) → D^(n−k)(dual)`, shape `(n_k, n_k)` — which is the standard DEC
/// operator and the one the sibling `grmhd` kernel uses correctly. The two are related by a
/// transport step, `⋆̃_k = transport_(dual→primal) ∘ ⋆_k`, which this kernel does not perform.
///
/// The shapes are therefore checked up front rather than assumed. Against a complex built from real
/// geometry the check refuses, and that refusal is correct: the operator supplied is not the one
/// this formulation needs. Reformulating it onto the crate's discrete interior product —
/// `Manifold::interior_product`, which implements the same star–wedge identity *with* the transport
/// — is tracked separately.
pub fn ideal_induction_kernel<R>(
    v_manifold: &SimplicialManifold<R, R>,
    b_manifold: &SimplicialManifold<R, R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + FromPrimitive + Default + PartialEq + Debug,
{
    // 1. Validation
    let complex = v_manifold.complex();
    let skeletons = complex.skeletons();

    // Three dimensions, not "at least two". The identity `i_v B = ⋆(v ∧ ⋆B)` closes only when the
    // two stars are the same operator, which needs `2 = n − 1`. The previous bound admitted 2D
    // complexes whose algebra this chain cannot serve.
    if skeletons.len() < 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "ideal induction needs a 3D complex: the identity i_v B = *(v ^ *B) applies * on \
             2-forms and on (n-1)-forms as one operator, which holds only at n = 3. This complex \
             has {} skeletons, so it is {}-dimensional",
            skeletons.len(),
            skeletons.len().saturating_sub(1)
        )));
    }

    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();

    // Verify data lengths (Manifold enforces this on creation, but checks are cheap).
    //
    // Both manifolds are checked, and against the same complex. `n0`, `n1` and `n2` come from
    // `v_manifold`'s complex, and the slices below index BOTH buffers with offsets derived from
    // them. The signature takes two independent manifolds, so a `b_manifold` built over a smaller
    // complex is a legal call; without this check it indexed past the end of `b_manifold`'s buffer
    // and panicked, where every other malformed input here returns `DimensionMismatch`.
    if v_manifold.data().len() < n0 + n1 + n2 {
        return Err(PhysicsError::DimensionMismatch(
            "v_manifold data too small".into(),
        ));
    }
    if b_manifold.data().len() < n0 + n1 + n2 {
        return Err(PhysicsError::DimensionMismatch(
            "b_manifold data too small".into(),
        ));
    }

    // 2. Extract Data Slices
    // v is 1-form: offset = n0, len = n1
    let v_offset = n0;
    let v_slice = &v_manifold.data().as_slice()[v_offset..v_offset + n1];

    // B is 2-form: offset = n0 + n1, len = n2
    let b_offset = n0 + n1;
    let b_slice = &b_manifold.data().as_slice()[b_offset..b_offset + n2];

    // 3. Compute Hodge Star of B (star_b)
    // star_b: 2-form -> 1-form (in 3D)
    // Using hodge_star_operators[2]. The accessor is fallible.
    // degenerate input geometry surfaces throgh the error type.
    let hodge_ops = complex
        .hodge_star_operators()
        .map_err(|e| PhysicsError::CalculationError(format!("Hodge ⋆ unavailable: {}", e)))?;
    if hodge_ops.len() <= 2 {
        return Err(PhysicsError::CalculationError(
            "Hodge star operator for 2-forms not available".into(),
        ));
    }
    let h_star_2 = &hodge_ops[2];
    // `⋆₂ : Λ² → Λ¹` has shape `(n1, n2)`. The crate's diagonal star is `(n2, n2)`; checking here
    // names the mismatch instead of letting it surface three steps later as a length error.
    if h_star_2.shape() != (n1, n2) {
        return Err(PhysicsError::DimensionMismatch(format!(
            "ideal induction needs a degree-changing Hodge star on 2-forms, shape ({n1}, {n2}) \
             mapping 2-forms to 1-forms; hodge_star_operators()[2] has shape {:?}. The diagonal \
             lumped-mass star this crate builds is ({n2}, {n2}) and preserves degree",
            h_star_2.shape()
        )));
    }
    let star_b_data = h_star_2.vec_mult(b_slice)?;

    // 4. Compute Wedge Product: v ^ star_b
    // v: 1-form, star_b: 1-form -> Result: 2-form
    let wedge_data = wedge_product_1form_1form(v_slice, &star_b_data, complex)?;

    // 5. Compute Interior Product proxy: iv_b = star(v ^ star_b)
    // wedge_data is 2-form. star maps to 1-form.
    let iv_b_data = h_star_2.vec_mult(&wedge_data)?;

    // 6. Compute Exterior Derivative: d(iv_b)
    // iv_b is 1-form. d maps to 2-form.
    // Use coboundary_operators[1].
    if complex.coboundary_operators().len() <= 1 {
        return Err(PhysicsError::CalculationError(
            "Coboundary operator for 1-forms not available".into(),
        ));
    }
    let d_1 = &complex.coboundary_operators()[1];
    // `d₁ : Λ¹ → Λ²` has shape `(n2, n1)`.
    if d_1.shape() != (n2, n1) {
        return Err(PhysicsError::DimensionMismatch(format!(
            "ideal induction needs the coboundary on 1-forms with shape ({n2}, {n1}); \
             coboundary_operators()[1] has shape {:?}",
            d_1.shape()
        )));
    }
    let dt_b_neg_data = d_1.vec_mult_real(&iv_b_data)?;

    // 7. Result
    // Returns the 2-form part of the change.
    let result_len = dt_b_neg_data.len();
    CausalTensor::new(dt_b_neg_data, vec![result_len]).map_err(PhysicsError::from)
}

// --- Helper Functions ---

/// Computes the Wedge Product of two 1-forms on a Simplicial Complex.
/// Result is a 2-form.
///
/// Formula used (Cup Product):
/// $(\alpha \cup \beta)([0,1,2]) = \alpha([0,1]) \cdot \beta([1,2])$
/// Wedge Product $\alpha \wedge \beta = \alpha \cup \beta - \beta \cup \alpha$.
fn wedge_product_1form_1form<R>(
    alpha: &[R],
    beta: &[R],
    complex: &SimplicialComplex<R>,
) -> Result<Vec<R>, PhysicsError>
where
    R: RealField,
{
    let skeletons = complex.skeletons();
    if skeletons.len() < 3 {
        return Err(PhysicsError::DimensionMismatch(
            "Complex must have 2-simplices".into(),
        ));
    }
    let edges = skeletons[1].simplices();
    let faces = skeletons[2].simplices();

    // Both operands are 1-forms, so both are indexed by edge. Checked rather than defaulted: the
    // body below reads `alpha` and `beta` at edge indices, and reading a short slice with
    // `.get(i).unwrap_or(&zero)` would turn a wrong-length operand into a plausible answer with
    // some terms silently dropped. That is how the degree mismatch above stayed invisible.
    if alpha.len() != edges.len() || beta.len() != edges.len() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "wedge of two 1-forms: both operands are indexed by edge, so both must have length \
             {}; got {} and {}",
            edges.len(),
            alpha.len(),
            beta.len()
        )));
    }

    // Build Edge Lookup Map: (min(u,v), max(u,v)) -> edge_index
    let mut edge_map = HashMap::with_capacity(edges.len());
    for (idx, edge_simplex) in edges.iter().enumerate() {
        let verts = edge_simplex.vertices();
        if verts.len() >= 2 {
            edge_map.insert((verts[0], verts[1]), idx);
        }
    }

    let zero = R::zero();
    let mut result = Vec::with_capacity(faces.len());

    for face in faces {
        let verts = face.vertices(); // Sorted [v0, v1, v2]
        if verts.len() != 3 {
            result.push(zero);
            continue;
        }
        let v0 = verts[0];
        let v1 = verts[1];
        let v2 = verts[2];

        let e01_idx = edge_map.get(&(v0, v1));
        let e12_idx = edge_map.get(&(v1, v2));

        if let (Some(&idx01), Some(&idx12)) = (e01_idx, e12_idx) {
            // Direct indexing: `edge_map` only ever holds indices below `edges.len()`, and both
            // operands were checked against that length above.
            let val_alpha_01 = alpha[idx01];
            let val_beta_12 = beta[idx12];
            let val_beta_01 = beta[idx01];
            let val_alpha_12 = alpha[idx12];

            // \alpha \wedge \beta = \alpha \cup \beta - \beta \cup \alpha
            let term1 = val_alpha_01 * val_beta_12;
            let term2 = val_beta_01 * val_alpha_12;

            result.push(term1 - term2);
        } else {
            result.push(zero);
        }
    }

    Ok(result)
}
