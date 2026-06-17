/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Opt-in spectral evaluation of the viscous term `Δ₁u♭` on fully
//! periodic uniform lattices.
//!
//! On a flat torus the grade-1 Hodge–de Rham Laplacian block-diagonalizes
//! per edge family: each of the `D` families is a shifted torus
//! sub-lattice (a contiguous block in the orientation-major edge
//! ordering, axis-0-fastest within the block), and `Δ₁` acts on it as the
//! scalar lattice Laplacian with eigenvalues
//! `λ_k = Σ_d (2 − 2·cos(2π·k_d/N_d)) / h_d²`. The evaluation is
//! therefore `D` real-FFT round trips with a pointwise eigenvalue
//! multiply — a drop-in replacement for the per-stage operator
//! application; the time integration is unchanged.
//!
//! Rounding-level equivalence to `laplacian_of(·, 1)` is pinned by
//! `spectral_diffusion_tests.rs`; the option stays opt-in
//! (`DecNsSolver::with_spectral_diffusion`) unless the validation ladder
//! reproduces the generic path's observed convergence orders.

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use deep_causality_fft::RfftPlanNd;
use deep_causality_num::{Complex, FromPrimitive, RealField};
use deep_causality_par::MaybeParallel;
use deep_causality_topology::{ChainComplex, HasHodgeStar, LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

/// Compiled spectral state for the grade-1 viscous term on one fully
/// periodic uniform manifold.
#[derive(Debug)]
pub(crate) struct SpectralDiffusion<R: RealField + FromPrimitive + MaybeParallel> {
    /// Edges per family (= vertex count on a fully periodic lattice).
    block: usize,
    /// Number of edge families (= D).
    families: usize,
    plan: RfftPlanNd<R>,
    /// Per-FFT-axis eigenvalue weight tables over the spectrum shape.
    weights: Vec<Vec<R>>,
    spec_shape: Vec<usize>,
    scratch: RefCell<SpectralScratch<R>>,
}

#[derive(Debug)]
struct SpectralScratch<R: RealField> {
    spectrum: Vec<Complex<R>>,
    work: Vec<Complex<R>>,
}

impl<R: DecNsScalar> SpectralDiffusion<R> {
    /// Build from a fully periodic uniform Euclidean manifold.
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` when the lattice is not fully
    /// periodic, the metric has no per-axis spacings (per-edge or
    /// Lorentzian), or the FFT plan cannot be built.
    pub(crate) fn new<const D: usize>(
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<Self, PhysicsError> {
        let complex = manifold.complex();
        let (shape, periodic) = complex.uniform_lattice_layout().ok_or_else(|| {
            PhysicsError::TopologyError(
                "spectral diffusion requires a uniform lattice complex".into(),
            )
        })?;
        if !periodic.iter().all(|&p| p) {
            return Err(PhysicsError::TopologyError(
                "spectral diffusion is periodic-only: every lattice axis must be periodic".into(),
            ));
        }
        let spacings = manifold
            .metric()
            .as_ref()
            .and_then(|m| m.uniform_axis_spacings())
            .ok_or_else(|| {
                PhysicsError::TopologyError(
                    "spectral diffusion requires per-axis Euclidean spacings \
                     (unit, uniform, or per-axis metric)"
                        .into(),
                )
            })?;

        // Lattice axis 0 varies fastest within a family block, so the FFT
        // sees the reversed shape (same convention as the spectral
        // Poisson solve in topology).
        let fft_shape: Vec<usize> = shape.iter().rev().copied().collect();
        let plan = RfftPlanNd::<R>::new(&fft_shape)
            .map_err(|e| PhysicsError::TopologyError(format!("spectral diffusion plan: {e}")))?;

        let spec_shape = plan.spectrum_shape().to_vec();
        let d = spec_shape.len();
        let two_pi = R::pi() + R::pi();
        let two = R::one() + R::one();
        let mut weights: Vec<Vec<R>> = Vec::with_capacity(d);
        for (a, &bins) in spec_shape.iter().enumerate() {
            let lattice_axis = d - 1 - a;
            let n = fft_shape[a];
            let h = spacings[lattice_axis];
            let h2 = h * h;
            let nr = <R as FromPrimitive>::from_usize(n).expect("axis length is representable");
            let table: Vec<R> = (0..bins)
                .map(|j| {
                    let jr =
                        <R as FromPrimitive>::from_usize(j).expect("bin index is representable");
                    let theta = two_pi * jr / nr;
                    (two - two * theta.cos()) / h2
                })
                .collect();
            weights.push(table);
        }

        let block: usize = shape.iter().product();
        let czero = Complex::new(R::zero(), R::zero());
        let scratch = SpectralScratch {
            spectrum: vec![czero; plan.spectrum_len()],
            work: vec![czero; plan.scratch_len()],
        };

        Ok(Self {
            block,
            families: D,
            plan,
            weights,
            spec_shape,
            scratch: RefCell::new(scratch),
        })
    }

    /// `out = Δ₁ u♭` per edge family: rFFT → multiply by `λ_k` → irFFT.
    /// Lengths are the rate's construction invariant.
    pub(crate) fn apply_laplacian_1(&self, u: &[R], out: &mut [R]) {
        debug_assert_eq!(u.len(), self.block * self.families);
        debug_assert_eq!(out.len(), self.block * self.families);

        let mut scratch = self.scratch.borrow_mut();
        let scratch = &mut *scratch;
        let d = self.spec_shape.len();
        let mut idx = vec![0usize; d];
        for f in 0..self.families {
            let range = f * self.block..(f + 1) * self.block;
            self.plan
                .execute(&u[range.clone()], &mut scratch.spectrum, &mut scratch.work)
                .expect("buffer lengths fixed at construction");

            // Pointwise eigenvalue multiply with the odometer walk (last
            // axis fastest), matching the spectral Poisson convention.
            idx.iter_mut().for_each(|i| *i = 0);
            for z in scratch.spectrum.iter_mut() {
                let mut lambda = R::zero();
                for (a, &i) in idx.iter().enumerate() {
                    lambda += self.weights[a][i];
                }
                *z = Complex::new(z.re * lambda, z.im * lambda);
                for a in (0..d).rev() {
                    idx[a] += 1;
                    if idx[a] < self.spec_shape[a] {
                        break;
                    }
                    idx[a] = 0;
                }
            }

            self.plan
                .execute_inverse(&mut scratch.spectrum, &mut out[range], &mut scratch.work)
                .expect("buffer lengths fixed at construction");
        }
    }
}
