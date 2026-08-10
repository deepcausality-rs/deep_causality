/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The CFD ↔ tensor-network (QTT) bridge: encode a lattice field as a quantized tensor train and
//! assemble finite-difference operators as MPOs.
//!
//! This is the foundation that lets a flowfield live in, and evolve as, a tensor train (the
//! compressed-flowfield lever of the plasma-blackout corridor example). It provides a quantized
//! field codec ([`quantize`] / [`dequantize`]) and periodic finite-difference MPO assembly
//! ([`shift_plus`] / [`shift_minus`] / [`gradient`] / [`laplacian`]).
//!
//! The mode layout is serial-per-axis: each axis contributes its own `l` binary modes in sequence,
//! rather than interleaving the axes' bits.
//!
//! References:
//! - Peddinti, R. D., Pisoni, S., Marini, A., Lott, P., Argentieri, H., Tiunov, E. & Aolita, L.
//!   (2024). *A quantum-inspired framework for computational fluid dynamics.* Communications
//!   Physics **7**, 135 — the MPS Navier–Stokes construction this bridge follows.
//! - Kazeev, V. A. & Khoromskij, B. N. *Low-Rank Explicit QTT Representation of the Laplace Operator
//!   and Its Inverse* — the QTT finite-difference operator construction: finite-difference operators
//!   as MPOs built from binary grid-shift operators at small bond dimension.
//!
//! Neither PDF is in `papers/` yet; both are listed there as cited-without-PDF. The Kazeev–Khoromskij
//! entry carries author and title only, which is what this repository can confirm; complete its venue
//! and year when the PDF is added rather than from recall.

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
