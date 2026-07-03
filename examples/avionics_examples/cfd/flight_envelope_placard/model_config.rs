/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): where the test matrix lives and the fitted shock
//! model the study computes with. Execution stays in `model`; tuned values in `constants`.

use crate::FloatType;
use crate::constants;
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::FittedNormalShock;
use deep_causality_physics::PhysicsError;
use std::path::PathBuf;

/// The Mach-altitude matrix: the recorded corridor by default, or a caller-supplied path (the
/// out-of-envelope demonstration passes its own file).
pub fn matrix_path() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("cfd/flight_envelope_placard/mach_alt_matrix.csv")
        })
}

/// Where the placard table is written.
pub fn table_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cfd/flight_envelope_placard/placard_table.csv")
}

/// The exact-Rankine-Hugoniot shock model at the study's effective gamma.
pub fn shock_model() -> Result<FittedNormalShock<FloatType>, PhysicsError> {
    FittedNormalShock::<FloatType>::new(ft(constants::GAMMA))
}
