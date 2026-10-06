/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types, patient context and constants for the aneurysm counterfactual chain.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
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

/// Node index: baseline systolic blood pressure in mmHg.
pub const BASELINE_BP: usize = 0;
/// Node index: wall shear stress gained per mmHg above `WSS_ONSET_BP`, in Pa/mmHg.
pub const WSS_GAIN: usize = 1;
/// Node index: systolic blood pressure below which the dome sees no wall shear stress, in mmHg.
pub const WSS_ONSET_BP: usize = 2;
/// Node index: wall shear stress above which the wall accumulates fatigue, in Pa.
pub const CRITICAL_WSS: usize = 3;
/// Node index: fatigue added per cycle at `WSS = CRITICAL_WSS`; scales with `WSS / CRITICAL_WSS`.
pub const DAMAGE_RATE: usize = 4;
/// Node index: fatigue healed per cycle while the wall shear stress is at or below `CRITICAL_WSS`.
pub const HEALING_RATE: usize = 5;
/// Node index: fatigue at which the wall ruptures; fatigue is clamped to [0, 1].
pub const RUPTURE_THRESHOLD: usize = 6;

/// The hypertensive patient, added in node-index order: node `i` holds contextoid id `i + 1`.
///
/// The wall shear stress surrogate is tuned so that a hypertensive patient
/// sits clearly above the critical threshold and a controlled patient sits
/// clearly below it.
pub fn patient_context() -> Result<PatientContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "patient", 7);
    for (id, value) in [
        (1, 175.0), // BASELINE_BP: hypertensive patient
        (2, 0.22),  // WSS_GAIN
        (3, 80.0),  // WSS_ONSET_BP
        (4, 15.0),  // CRITICAL_WSS
        (5, 0.04),  // DAMAGE_RATE
        (6, 0.005), // HEALING_RATE
        (7, 0.75),  // RUPTURE_THRESHOLD
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the patient context.
pub fn read(context: &PatientContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "the patient context holds no Datoid at node index {index}"
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
