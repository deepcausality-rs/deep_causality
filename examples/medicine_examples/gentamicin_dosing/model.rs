/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! One-compartment, first-order elimination kinetics and the Sawchuk–Zaske dosing steps.
//!
//! Method: Sawchuk RJ, Zaske DE, Cipolle RJ, Wargin WA, Strate RG. Kinetic model for gentamicin
//! dosing with the use of individual patient parameters. Clin Pharmacol Ther. 1977;21(3):362–369.
//! doi:10.1002/cpt1977213362, <https://pubmed.ncbi.nlm.nih.gov/837654/>
//!
//! Targets: Begg EJ, Barclay ML, Kirkpatrick CJM. The therapeutic monitoring of antimicrobial
//! agents. Br J Clin Pharmacol. 1999;47(1):23–30. doi:10.1046/j.1365-2125.1999.00850.x,
//! <https://pmc.ncbi.nlm.nih.gov/articles/PMC2014209/>

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_core::CausalityError;
use deep_causality_num::const_scalar_from_float;

/// Conventional dosing target (Begg et al. 1999): lowest acceptable peak, mg/L.
pub const PEAK_MIN_MG_L: FloatType = const_scalar_from_float!(FloatType, 6.0);
/// Conventional dosing target (Begg et al. 1999): highest acceptable peak, mg/L.
pub const PEAK_MAX_MG_L: FloatType = const_scalar_from_float!(FloatType, 10.0);
/// Conventional dosing target (Begg et al. 1999): highest acceptable trough, mg/L.
pub const TROUGH_MAX_MG_L: FloatType = const_scalar_from_float!(FloatType, 2.0);
/// The trough a redesigned interval aims for, mg/L.
pub const TROUGH_GOAL_MG_L: FloatType = const_scalar_from_float!(FloatType, 1.0);
/// The dosing intervals a ward schedules, hours.
pub const STANDARD_INTERVALS_H: [FloatType; 5] = [
    const_scalar_from_float!(FloatType, 8.0),
    const_scalar_from_float!(FloatType, 12.0),
    const_scalar_from_float!(FloatType, 24.0),
    const_scalar_from_float!(FloatType, 36.0),
    const_scalar_from_float!(FloatType, 48.0),
];

/// A serum level: hours after the dose started, and the concentration in mg/L.
#[derive(Debug, Clone, Copy)]
pub struct Level {
    pub hour: FloatType,
    pub mg_l: FloatType,
}

/// The two levels drawn after one dose, and the dosing schedule they were drawn under.
#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    pub first: Level,
    pub second: Level,
    pub infusion_end_h: FloatType,
    pub interval_h: FloatType,
}

/// The measurement with the patient's elimination rate, per hour.
#[derive(Debug, Clone, Copy)]
pub struct Elimination {
    pub measured: Measurement,
    pub k_per_h: FloatType,
}

/// The elimination with the peak at the end of the infusion.
#[derive(Debug, Clone, Copy)]
pub struct Peak {
    pub elimination: Elimination,
    pub peak: Level,
}

/// The peak with the trough just before the next dose.
#[derive(Debug, Clone, Copy)]
pub struct Course {
    pub peak: Peak,
    pub trough: Level,
}

/// The course, whether it meets the targets, and the interval that brings the trough to goal.
#[derive(Debug, Clone, Copy)]
pub struct Recommendation {
    pub course: Course,
    pub peak_in_range: bool,
    pub trough_in_range: bool,
    /// Hours for the peak to fall to the trough goal, plus the infusion.
    pub interval_to_goal_h: FloatType,
    /// The shortest standard interval at least as long as `interval_to_goal_h`.
    pub interval_h: Option<FloatType>,
}

/// The recommendation with the first level as the law returns it from the peak.
#[derive(Debug, Clone, Copy)]
pub struct Report {
    pub recommendation: Recommendation,
    pub returned: Level,
}

/// First-order elimination: the concentration `hours` after `mg_l`. The law is exact in either
/// direction of time, so a negative `hours` runs it backward.
pub fn concentration_after(mg_l: FloatType, k_per_h: FloatType, hours: FloatType) -> FloatType {
    mg_l * Real::exp(-(k_per_h * hours))
}

/// Step 1. The elimination rate from two levels on the same decay: `k = ln(c1 / c2) / (t2 - t1)`.
pub fn elimination_rate(measured: Measurement) -> Result<Elimination, CausalityError> {
    let (first, second) = (measured.first, measured.second);
    let zero = FloatType::default();
    if second.hour <= first.hour || second.mg_l <= zero || first.mg_l <= second.mg_l {
        return Err(CausalityError::ModelError(format!(
            "the levels {:?} and {:?} do not describe one elimination phase",
            first, second
        )));
    }
    let k_per_h = Real::ln(first.mg_l / second.mg_l) / (second.hour - first.hour);
    Ok(Elimination { measured, k_per_h })
}

/// Step 2, backward in time. The peak at the end of the infusion, from the first level.
pub fn peak_at_end_of_infusion(elimination: Elimination) -> Peak {
    let Elimination { measured, k_per_h } = elimination;
    let hours = measured.infusion_end_h - measured.first.hour; // negative: earlier than the level
    let mg_l = concentration_after(measured.first.mg_l, k_per_h, hours);
    Peak {
        elimination,
        peak: Level {
            hour: measured.infusion_end_h,
            mg_l,
        },
    }
}

/// Step 3, forward in time. The trough just before the next dose, from the peak.
pub fn trough_before_next_dose(peak: Peak) -> Course {
    let Elimination { measured, k_per_h } = peak.elimination;
    let hours = measured.interval_h - peak.peak.hour;
    let mg_l = concentration_after(peak.peak.mg_l, k_per_h, hours);
    Course {
        peak,
        trough: Level {
            hour: measured.interval_h,
            mg_l,
        },
    }
}

/// Step 4. Check the course against the targets, and find the interval at which the peak falls
/// to the trough goal before the next dose: `tau = t_infusion + ln(peak / goal) / k`.
pub fn dosing_interval(course: Course) -> Recommendation {
    let peak = course.peak.peak;
    let Elimination { measured, k_per_h } = course.peak.elimination;
    let interval_to_goal_h =
        measured.infusion_end_h + Real::ln(peak.mg_l / TROUGH_GOAL_MG_L) / k_per_h;
    Recommendation {
        course,
        peak_in_range: (PEAK_MIN_MG_L..=PEAK_MAX_MG_L).contains(&peak.mg_l),
        trough_in_range: course.trough.mg_l <= TROUGH_MAX_MG_L,
        interval_to_goal_h,
        interval_h: STANDARD_INTERVALS_H
            .into_iter()
            .find(|tau| *tau >= interval_to_goal_h),
    }
}

/// Step 5. The check: run the law forward from the reconstructed peak to the hour of the first
/// level. The result must return that level.
pub fn round_trip(recommendation: Recommendation) -> Report {
    let peak = recommendation.course.peak;
    let first = peak.elimination.measured.first;
    let returned_mg_l = concentration_after(
        peak.peak.mg_l,
        peak.elimination.k_per_h,
        first.hour - peak.peak.hour,
    );
    Report {
        recommendation,
        returned: Level {
            hour: first.hour,
            mg_l: returned_mg_l,
        },
    }
}
