/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The subject: the square torus as a cubical chain complex. Kitaev's toric code is this complex
//! read as a CSS code, qubits on the edges, Z checks on the faces, X checks on the vertices.

use crate::FloatType;
use crate::constants::LATTICE_SIDE;
use deep_causality_topology::LatticeComplex;

/// The `L × L` square torus.
pub fn square_torus() -> LatticeComplex<2, FloatType> {
    LatticeComplex::<2, FloatType>::square_torus(LATTICE_SIDE)
}
