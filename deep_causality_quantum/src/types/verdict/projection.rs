/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The orthomodular projection-lattice `Verdict` carrier (B7 / spec
//! quantum-verdict-orthomodular). A [`Projection`] is a Hermitian idempotent on
//! a fixed `D`-dimensional Hilbert space; the [`Verdict`] operations are the
//! subspace-lattice operations `∧`/`∨`/`⊥` — the Birkhoff–von Neumann quantum
//! logic. It is an **orthomodular** lattice: it satisfies the bounded-lattice,
//! orthocomplement, and orthomodular laws but **fails distributivity** (the way
//! `Prob` fails excluded middle), witnessed by three projections in general
//! position (e.g. `|0⟩`, `|1⟩`, `|+⟩`).
//!
//! No blanket `Verdict` impl is provided for a general operator/process-matrix
//! type: general effects `0 ≤ E ≤ I` form only an effect algebra with *partial*
//! meet/join. Verdicts are extracted from operators at the measurement boundary
//! (see [`crate::verdict::born`]).

use crate::QuantumError;
use crate::types::qgates::operator_linalg::{hermiticity_defect, identity_matrix, square_dim};
use deep_causality_algebra::{RealField, Verdict};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// A projection (Hermitian idempotent) on a fixed `D`-dimensional Hilbert
/// space. The type-level dimension is what lets [`Verdict::bottom`]/[`top`]
/// (which take no arguments) know the space they live in.
///
/// [`top`]: Verdict::top
#[derive(Debug, Clone, PartialEq)]
pub struct Projection<R: RealField, const D: usize> {
    p: CausalTensor<Complex<R>>,
}

fn c_zero<R: RealField>() -> Complex<R> {
    Complex::new(R::zero(), R::zero())
}

impl<R, const D: usize> Projection<R, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The default validation tolerance, `√ε`, scaled by the operator norm.
    pub fn default_tolerance() -> R {
        R::epsilon().sqrt()
    }

    /// Wraps a `D×D` matrix after checking it is Hermitian and idempotent
    /// (`P² = P`) within the default tolerance.
    pub fn new(p: CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        let dim = square_dim(&p)?;
        if dim != D {
            return Err(QuantumError::DimensionMismatch(format!(
                "projection dimension {} ≠ type parameter D = {}",
                dim, D
            )));
        }
        if p.as_slice()
            .iter()
            .any(|c| !c.re.is_finite() || !c.im.is_finite())
        {
            return Err(QuantumError::NonFiniteValue(
                "projection contains a non-finite entry".into(),
            ));
        }
        let tol = Self::default_tolerance();
        if hermiticity_defect(&p)? > tol {
            return Err(QuantumError::NonPositiveOperator(
                "projection is not Hermitian".into(),
            ));
        }
        let p2 = p
            .matmul(&p)
            .map_err(|e| QuantumError::CalculationError(format!("matmul: {:?}", e)))?;
        let defect = p2
            .as_slice()
            .iter()
            .zip(p.as_slice())
            .map(|(a, b)| ((a.re - b.re) * (a.re - b.re) + (a.im - b.im) * (a.im - b.im)).sqrt())
            .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
        if defect > tol {
            return Err(QuantumError::NonPositiveOperator(format!(
                "operator is not idempotent (‖P²−P‖ = {:?})",
                defect
            )));
        }
        Ok(Self { p })
    }

    /// The zero projection `0_D` (lattice bottom).
    pub fn zero() -> Self {
        let data = vec![c_zero::<R>(); D * D];
        Self {
            p: CausalTensor::from_slice(&data, &[D, D]),
        }
    }

    /// The identity projection `I_D` (lattice top).
    pub fn one() -> Self {
        Self {
            p: identity_matrix::<R>(D),
        }
    }

    /// The rank-1 projection `|ψ⟩⟨ψ|` of a (non-zero) `D`-vector column.
    pub fn from_ket(ket: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        let shape = ket.shape();
        let d = match shape {
            [d] | [d, 1] => *d,
            _ => {
                return Err(QuantumError::DimensionMismatch(format!(
                    "expected a ket column, got shape {:?}",
                    shape
                )));
            }
        };
        if d != D {
            return Err(QuantumError::DimensionMismatch(format!(
                "ket dimension {} ≠ D = {}",
                d, D
            )));
        }
        let ks = ket.as_slice();
        let norm_sq = ks
            .iter()
            .fold(R::zero(), |acc, c| acc + c.re * c.re + c.im * c.im);
        if norm_sq <= R::epsilon() {
            return Err(QuantumError::NormalizationError(
                "cannot project onto a (near-)zero ket".into(),
            ));
        }
        let inv = R::one() / norm_sq;
        let mut data = vec![c_zero::<R>(); D * D];
        for i in 0..D {
            for j in 0..D {
                let a = ks[i];
                let b = ks[j];
                data[i * D + j] = Complex::new(
                    (a.re * b.re + a.im * b.im) * inv,
                    (a.im * b.re - a.re * b.im) * inv,
                );
            }
        }
        Self::new(CausalTensor::from_slice(&data, &[D, D]))
    }

    /// The projection matrix.
    pub fn matrix(&self) -> &CausalTensor<Complex<R>> {
        &self.p
    }

    /// The rank (Hilbert-space dimension of the range), i.e. `Tr(P)` rounded.
    pub fn rank(&self) -> usize {
        let tr = (0..D).fold(R::zero(), |acc, i| acc + self.p.as_slice()[i * D + i].re);
        // Tr of a projection is its integer rank.
        let half = R::from_f64(0.5).unwrap_or_else(R::zero);
        let mut r = R::zero();
        let mut n = 0usize;
        while r + half < tr && n <= D {
            r += R::one();
            n += 1;
        }
        n
    }

    /// The subspace order `P ≤ Q` (range(P) ⊆ range(Q)), i.e. `P·Q = P` within
    /// tolerance.
    pub fn leq(&self, other: &Self) -> bool {
        match self.p.matmul(&other.p) {
            Ok(pq) => {
                let tol = Self::default_tolerance();
                let defect = pq
                    .as_slice()
                    .iter()
                    .zip(self.p.as_slice())
                    .map(|(a, b)| {
                        ((a.re - b.re) * (a.re - b.re) + (a.im - b.im) * (a.im - b.im)).sqrt()
                    })
                    .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
                defect <= tol
            }
            Err(_) => false,
        }
    }

    /// Whether two projections commute (`P·Q = Q·P`) — the family within which
    /// the lattice is distributive.
    pub fn commutes_with(&self, other: &Self) -> bool {
        let pq = self.p.matmul(&other.p);
        let qp = other.p.matmul(&self.p);
        match (pq, qp) {
            (Ok(a), Ok(b)) => {
                let tol = Self::default_tolerance();
                let defect = a
                    .as_slice()
                    .iter()
                    .zip(b.as_slice())
                    .map(|(x, y)| {
                        ((x.re - y.re) * (x.re - y.re) + (x.im - y.im) * (x.im - y.im)).sqrt()
                    })
                    .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
                defect <= tol
            }
            _ => false,
        }
    }

    /// The orthogonal projector onto the range of a Hermitian PSD `D×D`
    /// operator, via its eigendecomposition (eigenvectors with eigenvalue above
    /// tolerance). Used to realize the lattice join `range(P) + range(Q) =
    /// range(P + Q)`.
    fn range_projector(psd: &CausalTensor<Complex<R>>) -> Self {
        let (vals, vecs) = match psd.eigen_hermitian() {
            Ok(pair) => pair,
            // A D×D (D ≥ 1) Hermitian operator always decomposes; on the
            // impossible error path, fall back to the top projector.
            Err(_) => return Self::one(),
        };
        let vs = vecs.as_slice();
        // A relative tolerance off the largest eigenvalue.
        let max_ev = vals
            .iter()
            .map(|v| v.re)
            .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
        // A numerical-rank (support) cutoff, NOT the √ε validation tolerance:
        // keep genuine range directions down to ~D·ε·‖·‖. Using √ε here would
        // drop the second eigenvector of P+Q whenever two distinct subspaces are
        // closer than ~1.7e-4 rad, so the join would silently fail to be an upper
        // bound of its arguments.
        let scale = if max_ev > R::one() { max_ev } else { R::one() };
        let d_real = R::from_usize(D).unwrap_or_else(R::one);
        let tol = R::epsilon() * d_real * scale;

        let mut proj = vec![c_zero::<R>(); D * D];
        for (idx, lam) in vals.iter().enumerate() {
            if lam.re <= tol {
                continue;
            }
            // Column `idx` of V is the eigenvector; add v vᴴ.
            for i in 0..D {
                let vi = vs[i * D + idx];
                for j in 0..D {
                    let vj = vs[j * D + idx];
                    // vi * conj(vj)
                    let re = vi.re * vj.re + vi.im * vj.im;
                    let im = vi.im * vj.re - vi.re * vj.im;
                    let cur = proj[i * D + j];
                    proj[i * D + j] = Complex::new(cur.re + re, cur.im + im);
                }
            }
        }
        Self {
            p: CausalTensor::from_slice(&proj, &[D, D]),
        }
    }
}

impl<R, const D: usize> Verdict for Projection<R, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn bottom() -> Self {
        Self::zero()
    }

    fn top() -> Self {
        Self::one()
    }

    /// The lattice join `P ∨ Q`: the projector onto `range(P) + range(Q)`,
    /// realized as the range projector of the PSD sum `P + Q`.
    fn join(self, other: Self) -> Self {
        let mut sum = self.p.as_slice().to_vec();
        for (s, o) in sum.iter_mut().zip(other.p.as_slice()) {
            *s = Complex::new(s.re + o.re, s.im + o.im);
        }
        let sum = CausalTensor::from_slice(&sum, &[D, D]);
        Self::range_projector(&sum)
    }

    /// The lattice meet `P ∧ Q`: `¬(¬P ∨ ¬Q)` (De Morgan) — the projector onto
    /// `range(P) ∩ range(Q)`.
    fn meet(self, other: Self) -> Self {
        let not_p = self.complement();
        let not_q = other.complement();
        not_p.join(not_q).complement()
    }

    /// The orthocomplement `P^⊥ = I − P`.
    fn complement(self) -> Self {
        let id = identity_matrix::<R>(D);
        let mut data = id.as_slice().to_vec();
        for (d, p) in data.iter_mut().zip(self.p.as_slice()) {
            *d = Complex::new(d.re - p.re, d.im - p.im);
        }
        Self {
            p: CausalTensor::from_slice(&data, &[D, D]),
        }
    }
}
