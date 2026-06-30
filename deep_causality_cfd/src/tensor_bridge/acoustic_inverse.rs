/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The closed-form constant-coefficient acoustic-core inverse (design D10, Resolution 6).
//!
//! The stiff acoustic core is the constant-coefficient Helmholtz operator
//! `A₀ = I − β·∂² = (1+2s)·I − s·(S₊ + S₋)` on a periodic `2^l` grid, with `s = β/Δx²` the dimensionless
//! acoustic stiffness. The Tier-B plan made the fast pressure mode implicit through an AMEn solve of
//! *unproven convergence*; this replaces that gamble with an **exact, low-rank operator inverse applied in
//! closed form** — the spatial-acoustic analogue of how LER replaced a stiff source *solve* with a
//! closed-form *exponential*.
//!
//! `A₀` factors exactly through the cyclic shift,
//! ```text
//! A₀ = (s/ρ)·(I − ρ·S₊)·(I − ρ·S₋),   ρ = (1 + 2s − √(1+4s)) / (2s) ∈ (0, 1),
//! ```
//! (the quadratic `s·ρ² − (1+2s)·ρ + s = 0` makes the `I`-coefficient `s(1+ρ²)/ρ = 1+2s`). Hence
//! ```text
//! A₀⁻¹ = (ρ/s)·(I − ρ·S₋)⁻¹·(I − ρ·S₊)⁻¹,   (I − ρ·S₊)⁻¹ = Σ_{k≥0} ρ^k S₊^k.
//! ```
//! The geometric resolvent is evaluated by **binary doubling** — a finite, exact product, not an
//! iteration:
//! ```text
//! Σ_{k=0}^{2^l−1} ρ^k S₊^k = Π_{j=0}^{l−1} (I + ρ^{2^j}·S₊^{2^j}),
//! ```
//! so each resolvent costs `l` shift-applies. The cyclic tail dropped by the finite sum is `ρ^{2^l}`
//! (`< 10⁻³⁰` for the working `l`), i.e. exact to roundoff: `(I − ρ·S₊)·Σ_{k<N} ρ^k S₊^k = (1 − ρ^N)·I`.
//! Crucially `S₊^{2^j}` needs no new operator — incrementing the high `l−j` bits by one *is* adding `2^j`,
//! so `S₊^{2^j} = lift_leading(shift_plus(l−j), j)`.
//!
//! **Free-stream-exact.** `S±·const = const`, so each resolvent maps a uniform field by the gain
//! `Σ_{k<N} ρ^k = (1−ρ^N)/(1−ρ)`; the prefactor `ρ/s` cancels both gains exactly because `s·(1−ρ)² = ρ`
//! (again the quadratic), giving `A₀⁻¹·const = const` to roundoff. An AMEn-per-step solve loses this to
//! its residual tolerance — which is why it was rank-fragile on a captured curved field. The whole
//! construction is verified end-to-end by the `A₀·A₀⁻¹ = I` round-off gate (Resolution 6, gate 1).

use crate::tensor_bridge::operators::{lift_block, lift_leading, lift_trailing};
use crate::tensor_bridge::{shift_minus, shift_plus};
use crate::types::CfdScalar;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator, Truncation,
};

/// Closed-form inverse of the constant-coefficient acoustic core `A₀ = I − β·∂²` on a periodic grid,
/// applied to a right-hand side without any iterative solve. See the module docs for the construction.
pub struct AcousticCoreInverse<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// `ρ^{2^j}` for `j = 0..l` (the doubling weights).
    rho_pow: Vec<R>,
    /// `ρ/s`, the prefactor that makes the inverse free-stream-exact.
    pre_scale: R,
    /// `S₊^{2^j}` for `j = 0..l`, lifted to the field's mode layout.
    sp_pow: Vec<CausalTensorTrainOperator<R>>,
    /// `S₋^{2^j}` for `j = 0..l`.
    sm_pow: Vec<CausalTensorTrainOperator<R>>,
    trunc: Truncation<R>,
}

impl<R> AcousticCoreInverse<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build `A₀⁻¹` for the stiff core `A₀ = (1+2s)·I − s·(S₊+S₋)` on a periodic `2^l` grid, where the
    /// dimensionless stiffness is `s = β/Δx²` (for `A₀ = I − β·∂²`).
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if `l == 0`; [`PhysicsError::NumericalInstability`] if `s` is
    /// not finite and positive; propagates shift-operator and lift errors.
    pub fn new_1d(l: usize, s: R, trunc: Truncation<R>) -> Result<Self, PhysicsError> {
        if l == 0 {
            return Err(PhysicsError::DimensionMismatch(
                "acoustic-core inverse requires l >= 1".into(),
            ));
        }
        let mut sp_pow = Vec::with_capacity(l);
        let mut sm_pow = Vec::with_capacity(l);
        for j in 0..l {
            // S₊^{2^j} = increment the high `l−j` bits by one (= add 2^j), identity on the low `j` bits.
            sp_pow.push(lift_leading(&shift_plus::<R>(l - j)?, j)?);
            sm_pow.push(lift_leading(&shift_minus::<R>(l - j)?, j)?);
        }
        Self::from_shift_pows(s, sp_pow, sm_pow, trunc)
    }

    /// Build the inverse from caller-supplied shift powers `S₊^{2^j}` / `S₋^{2^j}` (`j = 0..l`), already
    /// lifted to the field's mode layout. Used for per-axis (ADI) inverses on a multi-dimensional field.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] on empty or mismatched shift-power lists;
    /// [`PhysicsError::NumericalInstability`] if `s` is not finite and positive.
    pub fn from_shift_pows(
        s: R,
        sp_pow: Vec<CausalTensorTrainOperator<R>>,
        sm_pow: Vec<CausalTensorTrainOperator<R>>,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let l = sp_pow.len();
        if l == 0 || sm_pow.len() != l {
            return Err(PhysicsError::DimensionMismatch(format!(
                "acoustic-core inverse needs matching non-empty shift powers (got {}, {})",
                sp_pow.len(),
                sm_pow.len()
            )));
        }
        if !s.is_finite() || s <= R::zero() {
            return Err(PhysicsError::NumericalInstability(
                "acoustic stiffness s must be finite and positive".into(),
            ));
        }
        let one = R::one();
        let two = one + one;
        let four = two + two;
        // ρ = (1 + 2s − √(1+4s)) / (2s) ∈ (0,1), the contracting root of s·ρ² − (1+2s)·ρ + s = 0.
        let rho = (one + two * s - (one + four * s).sqrt()) / (two * s);
        let mut rho_pow = Vec::with_capacity(l);
        let mut p = rho;
        for _ in 0..l {
            rho_pow.push(p);
            p = p * p; // ρ^{2^{j+1}} = (ρ^{2^j})²
        }
        // After the loop `p = ρ^{2^l} = ρ^N`. The binary-doubling product is the **finite** geometric sum
        // `Σ_{k<N} ρ^k Sᵏ = (1−ρ^N)·(I−ρS)⁻¹`, so each of the two resolvents carries a `(1−ρ^N)` factor.
        // Folding `1/(1−ρ^N)²` into the prefactor makes `A₀⁻¹` exact (and free-stream-exact) at **all** N,
        // not just in the large-N limit where `ρ^N → 0`. (`s·(1−ρ)² = ρ` gives `(ρ/s)/(1−ρ)² = 1`.)
        let rho_n = p;
        let gain = (one - rho_n) * (one - rho_n);
        let pre_scale = (rho / s) / gain;
        Ok(Self {
            rho_pow,
            pre_scale,
            sp_pow,
            sm_pow,
            trunc,
        })
    }

    /// Apply `A₀⁻¹` to a right-hand side `b`: `(ρ/s)·R₋·R₊·b`, each resolvent a binary-doubling product.
    ///
    /// # Errors
    /// Propagates apply / rounding errors.
    pub fn apply(&self, b: &CausalTensorTrain<R>) -> Result<CausalTensorTrain<R>, PhysicsError> {
        let mut y = b.clone();
        // R₊·b = Π_j (I + ρ^{2^j}·S₊^{2^j}) b.
        for (op, &w) in self.sp_pow.iter().zip(self.rho_pow.iter()) {
            let shifted = op.apply(&y, &self.trunc)?.scale(w);
            y = y.add(&shifted)?.round(&self.trunc)?;
        }
        // R₋·(R₊·b).
        for (op, &w) in self.sm_pow.iter().zip(self.rho_pow.iter()) {
            let shifted = op.apply(&y, &self.trunc)?.scale(w);
            y = y.add(&shifted)?.round(&self.trunc)?;
        }
        Ok(y.scale(self.pre_scale))
    }

    /// The contracting root `ρ ∈ (0,1)` of the factorization (the geometric decay rate of the inverse).
    pub fn rho(&self) -> R {
        self.rho_pow[0]
    }
}

/// Closed-form inverse of the 2-D constant-coefficient acoustic core `A₀ = I − β·∇²` on a periodic
/// `2^lx × 2^ly` grid, via **ADI dimensional splitting**:
/// `A₀⁻¹ ≈ (I − β·∂ₓ²)⁻¹·(I − β·∂ᵧ²)⁻¹`, each factor the 1-D closed-form inverse acting along one axis.
/// The splitting error is the `O(β²·∂ₓ²∂ᵧ²)` cross term; free-stream exactness is preserved exactly
/// (each 1-D factor maps a uniform field to itself). The per-axis stiffness is `s = β/Δx²`, `β/Δy²`.
pub struct AcousticCoreInverse2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    inv_x: AcousticCoreInverse<R>,
    inv_y: AcousticCoreInverse<R>,
}

impl<R> AcousticCoreInverse2d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the 2-D ADI inverse of `A₀ = I − β·∇²` on a periodic `2^lx × 2^ly` grid (cell sizes
    /// `dx, dy`). The shift powers `S±^{2^j}` are the existing 1-D shifts lifted onto each axis of the
    /// serial `x`-then-`y` mode layout — no new operator.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if `lx == 0` or `ly == 0`;
    /// [`PhysicsError::NumericalInstability`] if `β`, `dx`, or `dy` is not finite and positive.
    pub fn new(
        lx: usize,
        ly: usize,
        dx: R,
        dy: R,
        beta: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        if lx == 0 || ly == 0 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "2-D acoustic inverse requires lx,ly >= 1 (got {lx},{ly})"
            )));
        }
        // x-axis: S±^{2^j} = increment the high `lx−j` x-bits by one, identity on the low `j` x-bits and
        // on all `ly` y-bits → lift_leading(shift(lx−j), j + ly).
        let mut xp = Vec::with_capacity(lx);
        let mut xm = Vec::with_capacity(lx);
        for j in 0..lx {
            xp.push(lift_leading(&shift_plus::<R>(lx - j)?, j + ly)?);
            xm.push(lift_leading(&shift_minus::<R>(lx - j)?, j + ly)?);
        }
        // y-axis: identity on all `lx` x-bits, then the y-shift on the high `ly−j` y-bits →
        // lift_trailing(lift_leading(shift(ly−j), j), lx).
        let mut yp = Vec::with_capacity(ly);
        let mut ym = Vec::with_capacity(ly);
        for j in 0..ly {
            yp.push(lift_trailing(
                &lift_leading(&shift_plus::<R>(ly - j)?, j)?,
                lx,
            )?);
            ym.push(lift_trailing(
                &lift_leading(&shift_minus::<R>(ly - j)?, j)?,
                lx,
            )?);
        }
        let sx = beta / (dx * dx);
        let sy = beta / (dy * dy);
        let inv_x = AcousticCoreInverse::from_shift_pows(sx, xp, xm, trunc)?;
        let inv_y = AcousticCoreInverse::from_shift_pows(sy, yp, ym, trunc)?;
        Ok(Self { inv_x, inv_y })
    }

    /// Apply the 2-D inverse `A₀⁻¹·b = (I−β∂ₓ²)⁻¹·(I−β∂ᵧ²)⁻¹·b`.
    ///
    /// # Errors
    /// Propagates the per-axis apply / rounding errors.
    pub fn apply(&self, b: &CausalTensorTrain<R>) -> Result<CausalTensorTrain<R>, PhysicsError> {
        self.inv_x.apply(&self.inv_y.apply(b)?)
    }
}

/// Closed-form inverse of the 3-D constant-coefficient acoustic core `A₀ = I − β·∇²` on a periodic
/// `2^lx × 2^ly × 2^lz` grid, via **ADI dimensional splitting**:
/// `A₀⁻¹ ≈ (I−β∂ₓ²)⁻¹·(I−β∂ᵧ²)⁻¹·(I−β∂_z²)⁻¹`. The splitting error is the `O(β²)` cross terms; free-stream
/// exactness is preserved exactly (each 1-D factor maps a uniform field to itself).
pub struct AcousticCoreInverse3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    inv_x: AcousticCoreInverse<R>,
    inv_y: AcousticCoreInverse<R>,
    inv_z: AcousticCoreInverse<R>,
}

impl<R> AcousticCoreInverse3d<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the 3-D ADI inverse of `A₀ = I − β·∇²` on a periodic `2^lx × 2^ly × 2^lz` grid `dims`, with
    /// per-axis cell sizes `cells = (dx, dy, dz)`. The shift powers `S±^{2^j}` are the existing 1-D shifts
    /// lifted onto each axis of the serial `x`-`y`-`z` mode layout via `lift_block` — no new operator.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if any of `lx, ly, lz` is zero;
    /// [`PhysicsError::NumericalInstability`] if `β`, `dx`, `dy`, or `dz` is not finite and positive.
    pub fn new(
        dims: (usize, usize, usize),
        cells: (R, R, R),
        beta: R,
        trunc: Truncation<R>,
    ) -> Result<Self, PhysicsError> {
        let (lx, ly, lz) = dims;
        let (dx, dy, dz) = cells;
        if lx == 0 || ly == 0 || lz == 0 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "3-D acoustic inverse requires lx,ly,lz >= 1 (got {lx},{ly},{lz})"
            )));
        }
        // S±^{2^j} along an axis = the 1-D shift on the high `l−j` bits of that axis's block, identity on
        // the `j` low bits of the block and on every mode of the other two axes.
        let mut xp = Vec::with_capacity(lx);
        let mut xm = Vec::with_capacity(lx);
        for j in 0..lx {
            xp.push(lift_block(&shift_plus::<R>(lx - j)?, 0, j + ly + lz)?);
            xm.push(lift_block(&shift_minus::<R>(lx - j)?, 0, j + ly + lz)?);
        }
        let mut yp = Vec::with_capacity(ly);
        let mut ym = Vec::with_capacity(ly);
        for j in 0..ly {
            yp.push(lift_block(&shift_plus::<R>(ly - j)?, lx, j + lz)?);
            ym.push(lift_block(&shift_minus::<R>(ly - j)?, lx, j + lz)?);
        }
        let mut zp = Vec::with_capacity(lz);
        let mut zm = Vec::with_capacity(lz);
        for j in 0..lz {
            zp.push(lift_block(&shift_plus::<R>(lz - j)?, lx + ly, j)?);
            zm.push(lift_block(&shift_minus::<R>(lz - j)?, lx + ly, j)?);
        }
        let sx = beta / (dx * dx);
        let sy = beta / (dy * dy);
        let sz = beta / (dz * dz);
        let inv_x = AcousticCoreInverse::from_shift_pows(sx, xp, xm, trunc)?;
        let inv_y = AcousticCoreInverse::from_shift_pows(sy, yp, ym, trunc)?;
        let inv_z = AcousticCoreInverse::from_shift_pows(sz, zp, zm, trunc)?;
        Ok(Self {
            inv_x,
            inv_y,
            inv_z,
        })
    }

    /// Apply the 3-D inverse `A₀⁻¹·b = (I−β∂ₓ²)⁻¹·(I−β∂ᵧ²)⁻¹·(I−β∂_z²)⁻¹·b`.
    ///
    /// # Errors
    /// Propagates the per-axis apply / rounding errors.
    pub fn apply(&self, b: &CausalTensorTrain<R>) -> Result<CausalTensorTrain<R>, PhysicsError> {
        self.inv_x.apply(&self.inv_y.apply(&self.inv_z.apply(b)?)?)
    }
}
