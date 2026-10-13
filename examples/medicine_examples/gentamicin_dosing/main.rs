/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Gentamicin dosing from two serum levels
//!
//! Gentamicin kills bacteria in proportion to its peak concentration, and its toxicity to the
//! kidneys and the inner ear follows total exposure. Under conventional multiple-daily dosing,
//! therapeutic drug monitoring therefore checks both ends of a dose: a peak of 6–10 mg/L and a
//! trough of at most 2 mg/L (Begg et al. 1999). The ward rarely samples at either end. It draws
//! two levels in between, and the Sawchuk–Zaske method reconstructs the rest with one
//! first-order elimination law. Once-daily dosing has since replaced this regimen in many
//! hospitals; the review covers both.
//!
//! The four steps form one causal chain, and each needs the result of the one before it:
//!
//! 1. the patient's elimination rate, from the two levels;
//! 2. the peak at the end of the infusion, **backward** in time from the first level;
//! 3. the trough before the next dose, **forward** in time from that peak;
//! 4. the verdict and the dosing interval that brings the trough to 1 mg/L;
//! 5. a check: the law run forward from the peak must return the first measured level.
//!
//! Steps 2 and 3 run the same law, `concentration_after`, in opposite directions of time. The
//! causal order stays one-directional: step 3 runs only after step 2 has produced the peak.
//!
//! References:
//!
//! * Sawchuk RJ, Zaske DE, Cipolle RJ, Wargin WA, Strate RG. Kinetic model for gentamicin dosing
//!   with the use of individual patient parameters. Clin Pharmacol Ther. 1977;21(3):362–369.
//!   <https://pubmed.ncbi.nlm.nih.gov/837654/>
//! * Begg EJ, Barclay ML, Kirkpatrick CJM. The therapeutic monitoring of antimicrobial agents.
//!   Br J Clin Pharmacol. 1999;47(1):23–30. <https://pmc.ncbi.nlm.nih.gov/articles/PMC2014209/>

mod model;
mod utils_print;

use deep_causality_core::{CausalFlow, CausalityError};
use deep_causality_num::const_scalar_from_float;
use model::{
    Level, Measurement, dosing_interval, elimination_rate, peak_at_end_of_infusion, round_trip,
    trough_before_next_dose,
};

/// Change this one line to `f32` or `Float106` to run the program at another precision.
pub type FloatType = f64;

/// 120 mg every 8 hours, infused over the first 30 minutes. Hours count from the 08:00 dose.
const MEASURED: Measurement = Measurement {
    // 09:30
    first: Level {
        hour: const_scalar_from_float!(FloatType, 1.5),
        mg_l: const_scalar_from_float!(FloatType, 7.0),
    },
    // 15:30
    second: Level {
        hour: const_scalar_from_float!(FloatType, 7.5),
        mg_l: const_scalar_from_float!(FloatType, 3.5),
    },
    infusion_end_h: const_scalar_from_float!(FloatType, 0.5),
    interval_h: const_scalar_from_float!(FloatType, 8.0),
};

fn main() -> Result<(), CausalityError> {
    let report = CausalFlow::value(MEASURED)
        .try_step(elimination_rate) // the patient's elimination rate; fails unless the levels decay
        .map(peak_at_end_of_infusion) // backward in time, to 08:30
        .map(trough_before_next_dose) // forward in time, to 16:00
        .map(dosing_interval) // the verdict and the interval
        .map(round_trip) // forward again from the peak: must return the 09:30 level
        .finish()?;

    utils_print::print_report(&report);
    Ok(())
}
