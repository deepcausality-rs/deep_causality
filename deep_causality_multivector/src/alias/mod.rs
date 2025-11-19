/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod alias_complex;
mod alias_pga3d;
mod alias_real;

use crate::CausalMultiVector;
use deep_causality_num::Complex;

pub type Complex64 = Complex<f64>;

pub type ComplexMultiVector = CausalMultiVector<Complex64>;

pub type DixonAlgebra = CausalMultiVector<Complex64>;

pub type PGA3DMultiVector = CausalMultiVector<f64>;

pub type RealMultiVector = CausalMultiVector<f64>;
