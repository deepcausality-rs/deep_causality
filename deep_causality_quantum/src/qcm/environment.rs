/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The immutable environmental-preparation handle (R3 / spec
//! "The environmental preparation is immutable context").
//!
//! Residual environmental quantum data — the Bell-preparation `ρ_A` of a
//! simulated-CJ QCM — is carried as a read-only handle. It is constructed once
//! from a validated [`DensityMatrix`] and thereafter exposes **only** read
//! accessors: there are, by construction, no methods that mutate the wrapped
//! state, so a model that threads `ρ_A` through evaluation cannot alter the
//! preparation mid-pass and its result is reproducible.

use crate::types::density_matrix::DensityMatrix;
use deep_causality_algebra::RealField;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// A read-only handle to the environmental preparation `ρ_A`. Cloneable and
/// comparable, but never mutable: the only way to obtain one is [`Self::new`],
/// and it exposes no write methods, keeping the simulated-CJ model in the
/// verifiable region.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentalPrep<R: RealField> {
    prep: DensityMatrix<R>,
}

impl<R: RealField> EnvironmentalPrep<R> {
    /// Seals a validated density matrix as an immutable preparation.
    pub fn new(prep: DensityMatrix<R>) -> Self {
        Self { prep }
    }

    /// Borrows the preparation as a validated density matrix (read-only).
    pub fn state(&self) -> &DensityMatrix<R> {
        &self.prep
    }

    /// Borrows the underlying complex matrix (read-only).
    pub fn matrix(&self) -> &CausalTensor<Complex<R>> {
        self.prep.matrix()
    }

    /// The Hilbert dimension of the preparation.
    pub fn dim(&self) -> usize {
        self.prep.dim()
    }
}
