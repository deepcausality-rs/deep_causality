/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// Weak Isospin Structure (Matter Field Helper)
// =============================================================================

use crate::{PhysicsError, SIN2_THETA_W};

/// Weak isospin representation for a fermion doublet.
#[derive(Debug, Clone, Copy)]
pub struct WeakIsospin {
    pub isospin: f64,
    pub i3: f64,
    pub hypercharge: f64,
}

impl WeakIsospin {
    pub fn new(isospin: f64, i3: f64, charge: f64) -> Result<Self, PhysicsError> {
        if i3.abs() > isospin + 1e-10 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "I₃ = {} must satisfy |I₃| ≤ I = {}",
                i3, isospin
            )));
        }
        let hypercharge = 2.0 * (charge - i3);
        Ok(Self {
            isospin,
            i3,
            hypercharge,
        })
    }

    pub fn lepton_doublet() -> Self {
        Self {
            isospin: 0.5,
            i3: -0.5,
            hypercharge: -1.0,
        }
    }

    pub fn neutrino() -> Self {
        Self {
            isospin: 0.5,
            i3: 0.5,
            hypercharge: -1.0,
        }
    }

    pub fn up_quark() -> Self {
        Self {
            isospin: 0.5,
            i3: 0.5,
            hypercharge: 1.0 / 3.0,
        }
    }

    pub fn down_quark() -> Self {
        Self {
            isospin: 0.5,
            i3: -0.5,
            hypercharge: 1.0 / 3.0,
        }
    }

    pub fn right_handed(charge: f64) -> Self {
        Self {
            isospin: 0.0,
            i3: 0.0,
            hypercharge: 2.0 * charge,
        }
    }

    pub fn electric_charge(&self) -> f64 {
        self.i3 + self.hypercharge / 2.0
    }

    pub fn vector_coupling(&self) -> f64 {
        let q = self.electric_charge();
        self.i3 - 2.0 * q * SIN2_THETA_W
    }

    pub fn axial_coupling(&self) -> f64 {
        self.i3
    }
    pub fn left_coupling(&self) -> f64 {
        self.vector_coupling() + self.axial_coupling()
    }
    pub fn right_coupling(&self) -> f64 {
        self.vector_coupling() - self.axial_coupling()
    }
}

impl Default for WeakIsospin {
    fn default() -> Self {
        Self::lepton_doublet()
    }
}
