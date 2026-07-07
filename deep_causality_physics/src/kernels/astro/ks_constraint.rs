/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Projection onto the Kustaanheimo–Stiefel constraint surface — the B2 "evolve freely, then project
//! onto the constraint manifold" step of the perturbed-conformal trajectory split (Gap-3 Resolution 1),
//! the Leray/Hodge analogue for the trajectory axis.
//!
//! A KS phase state `(u, w)` (`u` the regularised position 4-vector, `w = u'` its fictitious-time
//! velocity) corresponds to a physical 3-D `(r, v)` **iff** the **bilinear relation** holds:
//! `b(u, w) = u₄w₁ − u₃w₂ + u₂w₃ − u₁w₄ = 0` (the 4th component of `L(u)·w`; it is the KS realisation of
//! the Sp(2,R) gauge condition — the heavier Bars `(4,2)` packaging is optional, FS-1). A measurement
//! update on `w` can leave the surface; this module measures the residual and projects `w` back to the
//! nearest constraint-satisfying velocity under the **fixed gauge that holds `u`** (the position 4-vector
//! is unchanged; only the constraint-violating component of the velocity is removed).
//!
//! # References
//! * Stiefel, E. L. & Scheifele, G., *Linear and Regular Celestial Mechanics*, Springer (1971), §19.

use crate::PhysicsError;
use deep_causality_algebra::RealField;

/// The KS bilinear constraint residual `b(u, w) = u₄w₁ − u₃w₂ + u₂w₃ − u₁w₄`. Zero iff `(u, w)` maps to
/// a physical 3-D `(r, v)`.
pub fn ks_bilinear_residual<R: RealField>(u: [R; 4], w: [R; 4]) -> R {
    u[3] * w[0] - u[2] * w[1] + u[1] * w[2] - u[0] * w[3]
}

/// Project the KS velocity `w` onto the constraint surface `b(u, ·) = 0`, returning the **nearest**
/// constraint-satisfying velocity under the fixed gauge that keeps `u`. The constraint is linear in `w`
/// with gradient `g = (u₄, −u₃, u₂, −u₁)` and `|g|² = |u|²`, so the orthogonal projection removes the
/// violating component: `w' = w − (b(u,w)/|u|²)·g`. Idempotent.
///
/// # Errors
/// [`PhysicsError::Singularity`] if `|u|² = 0` (no position to gauge against).
pub fn ks_project_velocity<R: RealField>(u: [R; 4], w: [R; 4]) -> Result<[R; 4], PhysicsError> {
    let u2 = u[0] * u[0] + u[1] * u[1] + u[2] * u[2] + u[3] * u[3];
    if u2 <= R::zero() {
        return Err(PhysicsError::Singularity(
            "KS projection needs a non-zero position 4-vector".into(),
        ));
    }
    let b = ks_bilinear_residual(u, w);
    let scale = b / u2;
    // g = (u₄, −u₃, u₂, −u₁).
    Ok([
        w[0] - scale * u[3],
        w[1] - scale * (-u[2]),
        w[2] - scale * u[1],
        w[3] - scale * (-u[0]),
    ])
}
