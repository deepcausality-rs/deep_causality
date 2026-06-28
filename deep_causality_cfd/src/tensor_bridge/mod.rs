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

mod codec;
mod operators;

pub use codec::{dequantize, quantize};
pub use operators::{gradient, laplacian, shift_minus, shift_plus};
