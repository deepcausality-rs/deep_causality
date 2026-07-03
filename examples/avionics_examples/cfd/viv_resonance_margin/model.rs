/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the march-and-reduce step and the one reduced-row
//! struct. Case configuration lives in `model_config`; tuned values in `constants`.

use crate::FloatType;
use crate::constants::{DIAMETER_M, F_STRUCT_HZ, NU_AIR_M2_S};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{CfdFlow, strouhal_number};
use deep_causality_physics::PhysicsError;
use std::path::{Path, PathBuf};

/// One swept airspeed's reduced result, all in the working precision.
pub struct MarginRow {
    /// Airspeed, m/s (the swept input).
    pub airspeed: FloatType,
    /// Reynolds number `V * D / nu_air` the wake was computed at.
    pub reynolds: FloatType,
    /// Extracted Strouhal number `f * D / U` from the wake probe's developed tail.
    pub strouhal: FloatType,
    /// Shedding frequency `St * V / D`, Hz.
    pub f_shed_hz: FloatType,
    /// Resonance margin `|f_struct - f_shed| / f_struct`.
    pub margin: FloatType,
}

/// One airspeed's wake march and reduction (execution): march the case described by
/// [`model_config::wake_case`](crate::model_config::wake_case), extract the Strouhal number
/// from the probe tail, and dimensionalize the shedding frequency and margin.
pub fn margin_row(airspeed: FloatType) -> Result<MarginRow, PhysicsError> {
    let reynolds = airspeed * ft(DIAMETER_M) / ft(NU_AIR_M2_S);
    let (case, dt) = crate::model_config::wake_case(reynolds)?;

    // One-shot geometry: each swept case owns a fresh grid, so `run_owned` materializes
    // internally and drops it with the run.
    let report = CfdFlow::march(&case).run_owned()?;
    let probe = report.series("probe").ok_or_else(|| {
        PhysicsError::PhysicalInvariantBroken("viv-wake: no probe series in the report".into())
    })?;

    // The developed tail: the first half of the record is the street growing out of the
    // geometric asymmetry; the Strouhal number is read from the second half only, the same
    // tail convention the verification harness uses.
    let tail = &probe[probe.len() / 2..];
    let strouhal = strouhal_number(tail, dt, ft(1.0), ft(1.0));

    // Dimensionalize: the nondimensional St carries over, f_shed = St * V / D.
    let f_shed_hz = strouhal * airspeed / ft(DIAMETER_M);
    let margin = (ft(F_STRUCT_HZ) - f_shed_hz).abs() / ft(F_STRUCT_HZ);

    Ok(MarginRow {
        airspeed,
        reynolds,
        strouhal,
        f_shed_hz,
        margin,
    })
}

/// A file next to this example's sources, resolved from the crate manifest so the example runs
/// from any working directory.
pub fn example_file(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cfd/viv_resonance_margin")
        .join(name)
}
