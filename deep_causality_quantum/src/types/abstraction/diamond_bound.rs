/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The two-sided bound a Frobenius residual on Choi operators places on the diamond distance.
///
/// For a linear map `Φ` with the crate's unnormalised Choi operator `J(Φ)`:
///
/// 1. `‖Φ‖_⋄ ≤ ‖J(Φ)‖_1`: every unit vector on `X ⊗ X` is `(I ⊗ A)|Ω̃⟩` with `‖A‖_F = 1`, so
///    `(Φ ⊗ id)(|ψ⟩⟨ψ|) = (I ⊗ A) J(Φ) (I ⊗ A)†` has trace norm at most `‖A‖_∞² ‖J‖_1 ≤ ‖J‖_1`.
/// 2. `‖J‖_1 ≤ √(rank J) · ‖J‖_F ≤ √(d_in d_out) · ‖J‖_F`, Cauchy–Schwarz on the singular values.
/// 3. `J(Φ)/d_in = (Φ ⊗ id)(ω)` for the maximally entangled state `ω`, so `‖J‖_F ≤ ‖J‖_1 ≤ d_in ‖Φ‖_⋄`.
///
/// Hence `r / d_in ≤ ‖E − F‖_⋄ ≤ √(d_in d_out) · r` for a Frobenius residual `r` between two
/// channels, and a residual of zero certifies a diamond distance of zero. On `id` against `R_z(θ)`
/// on one qubit, whose diamond distance is `2 sin(θ/2)`, the residual is `2√2 sin(θ/2)`, so the
/// upper bound exceeds the distance by the constant `2√2` and the lower bound sits at `√2 sin(θ/2)`.
///
/// A morphism with classical wires is a direct sum of blocks `Δ_xy`, one per classical input value
/// `x` and output value `y`, and its residual is the root sum of squares `r² = Σ_xy r_xy²` over the
/// block Choi operators. An input state concentrates on one `x`, and the trace norm of a direct
/// sum over `y` is the sum of the blocks' trace norms, so
/// `‖Δ‖_⋄ = max_x sup_ρ Σ_y ‖(Δ_xy ⊗ id)(ρ)‖_1`. Two consequences:
///
/// 4. `‖Δ‖_⋄ ≤ max_x Σ_y ‖Δ_xy‖_⋄ ≤ √(d_in d_out) · max_x Σ_y r_xy ≤ √(d_in d_out n_out) · r`,
///    with `n_out` the number of classical output values, by Cauchy–Schwarz over `y`.
/// 5. `‖Δ‖_⋄ ≥ max_xy ‖Δ_xy‖_⋄ ≥ max_xy r_xy / d_in ≥ r / (d_in √(n_in n_out))`, with `n_in`
///    the number of classical input values, since `r² ≤ n_in n_out · max_xy r_xy²`.
///
/// Both factors are needed: two distributions `(1, 0)` and `(0, 1)` have diamond distance `2` and
/// residual `√2`, above the single-block upper bound, and three classical input values each
/// carrying that pair have residual `√6` and diamond distance `2`, below the single-block lower
/// bound. [`from_frobenius`](Self::from_frobenius) is the single-block bound and
/// [`from_frobenius_blocks`](Self::from_frobenius_blocks) the direct-sum bound; with no classical
/// wires the two agree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiamondBound<R> {
    /// `r / (d_in √(n_in n_out))`; `r / d_in` for a single block.
    pub lower: R,
    /// `√(d_in d_out n_out) · r`; `√(d_in d_out) · r` for a single block.
    pub upper: R,
    /// The factor `√(d_in d_out n_out)` the upper bound used.
    pub amplification: R,
}

impl<R> DiamondBound<R>
where
    R: RealField + FromPrimitive,
{
    /// The bound for a Frobenius residual `r` between two channels from `d_in` to `d_out` with no
    /// classical wires: one block. A dimension of zero names no system and is read as the trivial
    /// system of dimension one, which is also what a residual over no quantum wires carries.
    pub fn from_frobenius(r: R, d_in: usize, d_out: usize) -> Self {
        Self::from_frobenius_blocks(r, d_in, d_out, &[], &[])
    }

    /// The bound for a Frobenius residual `r` over the direct sum of the blocks of two morphisms
    /// with quantum dimensions `d_in → d_out` and the classical outcome counts `classical_in` and
    /// `classical_out` of their wires, as [`QcMorphism::classical_in`] and
    /// [`QcMorphism::classical_out`] carry them. A zero dimension or outcome count is read as one.
    ///
    /// [`QcMorphism::classical_in`]: crate::types::circuit_model::QcMorphism::classical_in
    /// [`QcMorphism::classical_out`]: crate::types::circuit_model::QcMorphism::classical_out
    pub fn from_frobenius_blocks(
        r: R,
        d_in: usize,
        d_out: usize,
        classical_in: &[usize],
        classical_out: &[usize],
    ) -> Self {
        let real = |n: usize| R::from_usize(n.max(1)).unwrap_or_else(R::one);
        let count = |counts: &[usize]| counts.iter().map(|&c| real(c)).fold(R::one(), |a, c| a * c);
        let din = real(d_in);
        let dout = real(d_out);
        let n_in = count(classical_in);
        let n_out = count(classical_out);
        let amplification = (din * dout * n_out).sqrt();
        Self {
            lower: r / (din * (n_in * n_out).sqrt()),
            upper: amplification * r,
            amplification,
        }
    }
}
