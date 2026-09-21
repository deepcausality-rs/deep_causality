/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianMetric, PhysicsError};
use core::fmt::Debug;
use core::iter::Sum;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use deep_causality_topology::SimplicialManifold;

/// Calculates relativistic current density J^μ via covariant divergence.
///
/// # Physical Model
///
/// Computes the source current from Maxwell's equations:
/// $$ J^\mu = \nabla_\nu F^{\mu\nu} $$
///
/// which on a simplicial complex is the codifferential of the electromagnetic 2-form:
/// $$ J = \delta F $$
///
/// # The codifferential is the crate's, not this kernel's
///
/// δ is [`deep_causality_topology::SimplicialManifold::codifferential_of`], the adjoint of `d`
/// under the mass-matrix inner product:
///
/// ```text
/// δ_k = M_{k-1}^{-1} B_k M_k
/// ```
///
/// with `B_k` the **boundary** operator taking k-cells to (k−1)-cells. This kernel previously
/// open-coded `⋆d⋆` against the raw `hodge_star_operators()`, applying the **coboundary**
/// `d: Λ² → Λ³` where δ requires the boundary `Λ² → Λ¹`. The result therefore landed on
/// 3-simplices, and a current density that is documented as a 1-form was returned indexed by
/// the wrong skeleton entirely. Nothing caught it: the only test asserted `is_ok()` and that
/// shape, so it pinned the wrong skeleton as correct.
///
/// Routing through the crate's operator also makes `δ² = 0` hold exactly, because
/// `δ_{k-1} δ_k = M^{-1} B_{k-1} B_k M` and `B_{k-1} B_k = 0`. That identity is charge
/// conservation, `∂_μ J^μ = 0`, and it holds whatever the mass matrices contain — which is what
/// makes it usable as an oracle here.
///
/// # Signature
///
/// `spacetime_metric` is read for its `dimension()` alone. `codifferential_of` builds a
/// Riemannian adjoint from the complex's mass matrices; the sign a Lorentzian signature
/// contributes to δ is applied nowhere in the crate, so a west coast metric and an east coast
/// metric yield the same answer. The current returned is the Riemannian codifferential of F.
///
/// # Arguments
/// * `em_manifold` - Manifold with electromagnetic 2-form F data on 2-simplices. It **must**
///   carry a Regge geometry: build it with `Manifold::with_metric`, since the codifferential
///   reads the mass matrices that geometry vends.
/// * `spacetime_metric` - Spacetime signature implementing `LorentzianMetric`
///
/// # Returns
/// Current density 1-form J as a `CausalTensor<R>`, indexed by the 1-simplices.
pub fn relativistic_current_kernel<R, M>(
    em_manifold: &SimplicialManifold<R, R>,
    spacetime_metric: &M,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
    M: LorentzianMetric,
{
    let complex = em_manifold.complex();
    let skeletons = complex.skeletons();

    // 1. Validate dimensions
    if skeletons.len() < 3 {
        return Err(PhysicsError::DimensionMismatch(
            "Requires at least 2-simplices for EM 2-form".into(),
        ));
    }

    if spacetime_metric.dimension() < 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Spacetime needs 4D, got {}D",
            spacetime_metric.dimension()
        )));
    }

    // 2. `codifferential_of` panics on a manifold with no metric, so refuse one here instead.
    // `Manifold::new` builds exactly that, and it is the constructor callers reach for first.
    if em_manifold.metric().is_none() {
        return Err(PhysicsError::DimensionMismatch(
            "relativistic current reads the mass matrices a Regge geometry vends; this manifold \
             carries no metric. Construct it with Manifold::with_metric"
                .into(),
        ));
    }

    // 3. The operator counts stand in for the complex's own dimension: a 2-form on spacetime
    // needs a complex that carries 3-simplices, which a 4D metric alone does not establish.
    //
    // Reading `hodge_star_operators()` here also forces the lazy build while its error is still
    // recoverable. `codifferential_of` reaches the same accessor through `hodge_star_matrix` and
    // `.expect()`s it, so a complex with degenerate geometry would panic inside the crate rather
    // than return; this call converts that into the kernel's own `Result`.
    let hodge_ops = complex
        .hodge_star_operators()
        .map_err(|e| PhysicsError::CalculationError(format!("Hodge ⋆ unavailable: {}", e)))?;

    if hodge_ops.len() < 4 {
        return Err(PhysicsError::CalculationError(format!(
            "Missing Hodge star operators: need 4, have {}",
            hodge_ops.len()
        )));
    }

    if complex.coboundary_operators().len() < 3 {
        return Err(PhysicsError::CalculationError(format!(
            "Missing coboundary operators: need 3, have {}",
            complex.coboundary_operators().len()
        )));
    }

    // 3. Extract F as 2-form data from manifold
    // Data layout: [0-simplices | 1-simplices | 2-simplices | ...]
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();

    let data = em_manifold.data().as_slice();
    if data.len() < n0 + n1 + n2 {
        return Err(PhysicsError::CalculationError(
            "Manifold data too short for 2-form extraction".into(),
        ));
    }

    let f_2form = &data[n0 + n1..n0 + n1 + n2];

    // 4. J = δF, a 1-form on the 1-simplices.
    Ok(em_manifold.codifferential_of(f_2form, 2))
}

/// Calculates the electromagnetic stress-energy tensor $T^{\mu\nu}_{EM}$.
/// $$ T^{\mu\nu} = F^{\mu\alpha}F^\nu_\alpha - \frac{1}{4} g^{\mu\nu} F_{\alpha\beta}F^{\alpha\beta} $$
///
/// # Arguments
/// *   `em_tensor` - Electromagnetic tensor $F^{\mu\nu}$ (Rank 2, Contravariant).
/// *   `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2, Covariant).
///
/// # Returns
/// *   `Result<CausalTensor<R>, PhysicsError>` - Stress-energy tensor $T^{\mu\nu}$.
pub fn energy_momentum_tensor_em_kernel<R>(
    em_tensor: &CausalTensor<R>,
    metric: &CausalTensor<R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + FromPrimitive + Sum + Default + PartialOrd + Debug,
{
    if em_tensor.num_dim() != 2 || metric.num_dim() != 2 {
        return Err(PhysicsError::DimensionMismatch(
            "Tensors must be Rank 2".into(),
        ));
    }

    // 1. Compute covariant F_αβ = g_αμ * F^μν * g_νβ
    let gf = metric.matmul(em_tensor)?;
    let f_lower = gf.matmul(metric)?;

    // 2. Compute Scalar F^2 = F^ab * F_ab (Contraction)
    let f2_op =
        EinSumOp::<R>::contraction(em_tensor.clone(), f_lower.clone(), vec![0, 1], vec![0, 1]);
    let f2_tensor = CausalTensor::ein_sum(&f2_op)?;
    let f2_val = if f2_tensor.shape().is_empty()
        || (f2_tensor.shape().len() == 1 && f2_tensor.shape()[0] == 1)
    {
        f2_tensor.data()[0]
    } else {
        return Err(PhysicsError::CalculationError(
            "Scalar contraction failed".into(),
        ));
    };

    // 3. Compute Term 1: T1^uv = F^ua * F^v_a
    let f_mixed = em_tensor.matmul(metric)?;
    let f_mixed_t_op = EinSumOp::<R>::transpose(f_mixed.clone(), vec![1, 0]);
    let f_mixed_t = CausalTensor::ein_sum(&f_mixed_t_op)?;
    let term1 = em_tensor.matmul(&f_mixed_t)?;

    // 4. Compute Term 2: 1/4 * g^uv * F^2
    let metric_inv = metric.inverse()?;
    let quarter = R::from_f64(0.25)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.25) failed".into()))?;
    let term2 = metric_inv * (quarter * f2_val);

    // 5. Result T = Term1 - Term2
    let stress_energy = term1 - term2;

    Ok(stress_energy)
}
