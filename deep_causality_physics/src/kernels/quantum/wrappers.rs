/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::quantum::mechanics;
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Causal wrapper for [`mechanics::klein_gordon_kernel`].
pub fn klein_gordon<R>(
    psi_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    match mechanics::klein_gordon_kernel(psi_manifold, mass) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
