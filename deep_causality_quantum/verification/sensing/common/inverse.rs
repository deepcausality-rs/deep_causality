/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reading a phase back off the read-out of `fringe_qubit`.

/// The phase behind read-out `p` at contrast `contrast`: the inverse of `½(1 + C sin φ)`.
pub fn phase_of(p: f64, contrast: f64) -> f64 {
    ((2.0 * p - 1.0) / contrast).asin()
}
