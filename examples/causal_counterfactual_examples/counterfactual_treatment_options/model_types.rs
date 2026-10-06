/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types, patient context and constants for the aneurysm counterfactual chain.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalityError, PropagatingProcess};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
pub type FloatType = f64;

/// Number of cardiac cycles each run simulates.
pub const N_CYCLES: u32 = 30;

/// The patient and the aneurysm wall the steps read, one `Data` contextoid per quantity. The
/// context holds no position, clock or event, so its spatial, temporal and spacetime slots are
/// empty.
pub type PatientContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Process alias for the chain: no state, the patient as context.
pub type PatientProcess<T> = PropagatingProcess<T, (), PatientContext>;

/// Contextoid id: baseline systolic blood pressure in mmHg.
pub const BASELINE_BP: ContextoidId = 1;
/// Contextoid id: wall shear stress gained per mmHg above `WSS_ONSET_BP`, in Pa/mmHg.
pub const WSS_GAIN: ContextoidId = 2;
/// Contextoid id: systolic blood pressure below which the dome sees no wall shear stress, in
/// mmHg.
pub const WSS_ONSET_BP: ContextoidId = 3;
/// Contextoid id: wall shear stress above which the wall accumulates fatigue, in Pa.
pub const CRITICAL_WSS: ContextoidId = 4;
/// Contextoid id: fatigue added per cycle at `WSS = CRITICAL_WSS`; scales with
/// `WSS / CRITICAL_WSS`.
pub const DAMAGE_RATE: ContextoidId = 5;
/// Contextoid id: fatigue healed per cycle while the wall shear stress is at or below
/// `CRITICAL_WSS`.
pub const HEALING_RATE: ContextoidId = 6;
/// Contextoid id: fatigue at which the wall ruptures; fatigue is clamped to [0, 1].
pub const RUPTURE_THRESHOLD: ContextoidId = 7;

/// The hypertensive patient: one `Data` contextoid per quantity, keyed by its contextoid id.
///
/// The wall shear stress surrogate is tuned so that a hypertensive patient
/// sits clearly above the critical threshold and a controlled patient sits
/// clearly below it.
pub fn patient_context() -> Result<PatientContext, ContextIndexError> {
    let facts = [
        (BASELINE_BP, 175.0), // hypertensive patient
        (WSS_GAIN, 0.22),
        (WSS_ONSET_BP, 80.0),
        (CRITICAL_WSS, 15.0),
        (DAMAGE_RATE, 0.04),
        (HEALING_RATE, 0.005),
        (RUPTURE_THRESHOLD, 0.75),
    ];
    let mut context = Context::with_capacity(1, "patient", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read the payload of the `Data` contextoid `id` out of the patient context.
pub fn read(context: &PatientContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the patient context holds no Datoid with contextoid id {id}"
        ))
    })
}

#[derive(Debug, Clone, Default)]
pub struct CycleSummary {
    pub cycles_run: u32,
    pub final_fatigue: FloatType,
    pub ruptured: bool,
    pub peak_wss: FloatType,
}
