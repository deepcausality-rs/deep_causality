/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the march-and-reduce step and the one reduced-row
//! struct. Case configuration lives in `model_config`; tuned values in `constants`.

use crate::FloatType;
use crate::constants::{DIAMETER_M, F_STRUCT_HZ, MARGIN_MIN, NU_AIR_M2_S, ST_BAND};
use crate::model_config::WakeCase;
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{CaseRun, GateSeq, PhysicsError, StudyView, TableRow, strouhal_number};
use std::path::{Path, PathBuf};

/// One swept airspeed's reduced result, all in the working precision.
#[derive(Clone)]
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

impl TableRow for MarginRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("airspeed", "m/s"),
        ("reynolds", "-"),
        ("strouhal", "-"),
        ("shedding_frequency", "Hz"),
        ("margin", "-"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.airspeed,
            self.reynolds,
            self.strouhal,
            self.f_shed_hz,
            self.margin,
        ]
    }
}

/// Reduce one airspeed's wake march to a [`MarginRow`] (the grammar's `reduce` step): the
/// airspeed comes from the case, the sampling `dt` from the [`WakeCase`], the probe series from
/// the report. Extract the Strouhal number from the probe's developed tail, then dimensionalize
/// the shedding frequency and margin. The march itself is the grammar's `.march()` over the
/// example-local `WakeCase: Marchable`.
pub fn margin_row(
    run: &CaseRun<'_, FloatType, WakeCase, FloatType>,
) -> Result<MarginRow, PhysicsError> {
    let airspeed = *run.case();
    let dt = run.config().dt();
    let report = run.report();

    let reynolds = airspeed * ft(DIAMETER_M) / ft(NU_AIR_M2_S);
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

// ── The resonance-margin gating sequence ──────────────────────────────────────────────────────

/// The margin study's gating sequence: the extracted Strouhal numbers sit in the validated band,
/// the resonance margin clears the placard, and every swept wake is a finite oscillation.
pub fn viv_gates() -> GateSeq<MarginRow> {
    GateSeq::new("vortex-shedding resonance margin")
        .gate("strouhal band", gate_strouhal_band)
        .gate("resonance margin", gate_resonance_margin)
        .gate("finite wake", gate_finite_wake)
}

/// Every extracted Strouhal number sits in the validated band for this grid.
pub fn gate_strouhal_band(view: &StudyView<'_, MarginRow>) -> (bool, String) {
    let ok = view
        .rows()
        .iter()
        .all(|r| r.strouhal >= ft(ST_BAND.0) && r.strouhal <= ft(ST_BAND.1));
    let (st_lo, st_hi) = view.rows().iter().fold(
        (FloatType::INFINITY, -FloatType::INFINITY),
        |(lo, hi), r| (lo.min(r.strouhal), hi.max(r.strouhal)),
    );
    (
        ok,
        format!(
            "extracted St in [{:.4}, {:.4}], validated band [{}, {}] for this grid",
            Into::<f64>::into(st_lo),
            Into::<f64>::into(st_hi),
            ST_BAND.0,
            ST_BAND.1
        ),
    )
}

/// The minimum resonance margin clears the placard.
pub fn gate_resonance_margin(view: &StudyView<'_, MarginRow>) -> (bool, String) {
    let min_margin = view
        .rows()
        .iter()
        .fold(FloatType::INFINITY, |m, r| m.min(r.margin));
    (
        min_margin >= ft(MARGIN_MIN),
        format!(
            "min |f_struct - f_shed| / f_struct = {:.3}, placard minimum {MARGIN_MIN}",
            Into::<f64>::into(min_margin)
        ),
    )
}

/// Every swept wake returned a finite, oscillating result.
pub fn gate_finite_wake(view: &StudyView<'_, MarginRow>) -> (bool, String) {
    let ok = view
        .rows()
        .iter()
        .all(|r| r.f_shed_hz.is_finite() && r.strouhal > ft(0.0));
    (
        ok,
        format!(
            "{} sweeps returned a finite, oscillating wake",
            view.rows().len()
        ),
    )
}

/// A file next to this example's sources, resolved from the crate manifest so the example runs
/// from any working directory.
pub fn example_file(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cfd/viv_resonance_margin")
        .join(name)
}
