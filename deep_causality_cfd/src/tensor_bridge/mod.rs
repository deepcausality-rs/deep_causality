/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The CFD ↔ tensor-network (QTT) bridge: encode a lattice field as a quantized tensor train and
//! assemble finite-difference operators as MPOs.
//!
//! This is the foundation that lets a flowfield live in, and evolve as, a tensor train (the
//! compressed-flowfield lever of the Plasma Blackout Corridor flagship). It provides a quantized
//! field codec ([`quantize`] / [`dequantize`]) and periodic finite-difference MPO assembly
//! ([`shift_plus`] / [`shift_minus`] / [`gradient`] / [`laplacian`]), following the Peddinti
//! (MPS Navier–Stokes) and Kazeev–Khoromskij (QTT operators) constructions.

mod acoustic_inverse;
mod codec;
mod mask;
mod operators;
mod projection;

pub use acoustic_inverse::{AcousticCoreInverse, AcousticCoreInverse2d, AcousticCoreInverse3d};
pub use codec::{dequantize, dequantize_2d, dequantize_3d, quantize, quantize_2d, quantize_3d};
pub use mask::{body_mask_2d, mask_from_fn, plume_mask_2d};
pub use operators::{
    divergence_3d, gradient, gradient_x, gradient_x_3d, gradient_y, gradient_y_3d, gradient_z_3d,
    laplacian, laplacian_2d, laplacian_3d, shift_minus, shift_plus,
};
pub use projection::QttProjector2d;
