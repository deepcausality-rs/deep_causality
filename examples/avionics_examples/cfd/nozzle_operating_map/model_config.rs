/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): one duct case description per swept back
//! pressure. Execution stays in `model`; tuned values stay in `constants`.

use crate::FloatType;
use crate::constants::{
    CELLS, EXIT_AREA_M2, GAMMA, INLET_AREA_M2, LENGTH_M, MAX_STEPS, P0_PA, RESIDUAL_TOL, T0_K,
    THROAT_AREA_M2,
};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{DuctAreaProfile, DuctConfig, DuctInlet, DuctStop, PhysicsError};

/// The 2:1:2 parabolic demonstration nozzle at one back-pressure ratio. Takes the ratio by
/// reference so it plugs directly into the grammar's `.case(model_config::duct_case)`.
pub fn duct_case(p_ratio: &FloatType) -> Result<DuctConfig<FloatType>, PhysicsError> {
    let p_ratio = *p_ratio;
    DuctConfig::new(
        DuctAreaProfile::ConvergingDiverging {
            inlet_area: ft(INLET_AREA_M2),
            throat_area: ft(THROAT_AREA_M2),
            exit_area: ft(EXIT_AREA_M2),
            length: ft(LENGTH_M),
        },
        DuctInlet {
            p0: ft(P0_PA),
            t0: ft(T0_K),
        },
        ft(GAMMA),
        ft(P0_PA) * p_ratio,
        CELLS,
        DuctStop {
            max_steps: MAX_STEPS,
            residual_tol: ft(RESIDUAL_TOL),
        },
    )
}
