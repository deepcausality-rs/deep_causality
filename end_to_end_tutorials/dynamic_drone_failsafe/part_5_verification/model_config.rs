/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scenarios. Each is drawn from the operational design domain by its number alone, so any
//! scenario of a campaign can be flown again on its own.

use crate::constants::*;
use crate::model_types::Scenario;
use deep_causality_algebra::Real;
use deep_causality_num::{lift_usize, lower};
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainError};
use dynamic_drone_failsafe::{FaultTimeline, FloatType, Mission, Terrain};

/// Scenario `id`. People and a stand of trees lie around the point where the fix drops out, the
/// worst place for a fail-safe that lands where it is.
pub fn scenario(id: usize) -> Result<Scenario, UncertainError> {
    let draw = |slot: u64, range: (FloatType, FloatType)| {
        Uncertain::uniform(range.0, range.1).sample_at(
            &SampleSession::seeded(CAMPAIGN_SEED * 1_000 + slot),
            id as u64,
        )
    };
    let start_hour = draw(0, START_HOUR)?;
    let wind = polar(draw(1, WIND_M_S)?, draw(2, DIRECTION_DEG)?);
    let gust = polar(draw(3, GUST_M)?, draw(4, DIRECTION_DEG)?);
    let line_across = draw(5, LINE_ACROSS_M)?;

    let fix_lost_s = lower(draw(6, FIX_LOST_AT_S)?) as i64;
    let link_lost_s = (fix_lost_s + lower(draw(7, LINK_OFFSET_S)?) as i64).max(1);
    let battery_s = fix_lost_s.max(link_lost_s) + lower(draw(8, BATTERY_OFFSET_S)?) as i64;
    let cell_fails = draw(9, (ZERO, ONE))? < CELL_FAILURE_SHARE;
    let restored =
        |share_slot: u64, after_slot: u64, lost_s: i64| -> Result<Option<usize>, UncertainError> {
            Ok(if draw(share_slot, (ZERO, ONE))? < RECOVERY_SHARE {
                Some((lost_s + lower(draw(after_slot, RECOVERY_AFTER_S)?) as i64) as usize)
            } else {
                None
            })
        };
    let faults = FaultTimeline::new(
        fix_lost_s as usize - DEGRADED_LEAD_S,
        fix_lost_s as usize,
        link_lost_s as usize,
        cell_fails.then_some(battery_s as usize),
    )
    .restoring(
        restored(10, 11, fix_lost_s)?,
        restored(12, 13, link_lost_s)?,
    );

    let fault_along = INSPECTION_SPEED_M_S * lift_usize::<FloatType>(fix_lost_s as usize);
    let terrace_across = draw(14, TERRACE_FROM_ACROSS_M)?;
    let terrace_along = draw(15, TERRACE_FROM_ALONG_M)?;
    let woodland_across = draw(16, WOODLAND_FROM_ACROSS_M)?;
    let woodland_along = fault_along - draw(17, WOODLAND_BEFORE_M)?;
    let woodland_width = draw(18, WOODLAND_WIDTH_M)?;
    let woodland_length = draw(19, WOODLAND_LENGTH_M)?;
    let crew = (lower(Real::floor(draw(
        20,
        (ZERO, lift_usize::<FloatType>(MAX_CREW + 1)),
    )?)) as usize)
        .min(MAX_CREW);
    let across = (
        larger(CREEK_EDGE_M, line_across - CREW_SPREAD_ACROSS_M),
        smaller(ROAD_TO_M, line_across + CREW_SPREAD_ACROSS_M),
    );
    let along = (
        fault_along - CREW_SPREAD_ALONG_M,
        fault_along + CREW_SPREAD_ALONG_M,
    );
    let mut crew_m = Vec::with_capacity(crew);
    for k in 0..crew as u64 {
        crew_m.push((draw(21 + 2 * k, across)?, draw(22 + 2 * k, along)?));
    }

    Ok(Scenario {
        id,
        mission: Mission::new(
            line_across,
            start_hour,
            wind,
            gust,
            faults,
            CAMPAIGN_SEED * 100_000 + 8 * id as u64,
        ),
        terrain: Terrain::with(
            line_across,
            (
                (terrace_across, terrace_across + TERRACE_SIZE_M.0),
                (terrace_along, terrace_along + TERRACE_SIZE_M.1),
            ),
            (
                (woodland_across, woodland_across + woodland_width),
                (woodland_along, woodland_along + woodland_length),
            ),
            crew_m,
        ),
        crew,
    })
}

/// A vector of length `size` pointing `direction_deg` from the across axis toward the along axis.
fn polar(size: FloatType, direction_deg: FloatType) -> (FloatType, FloatType) {
    let angle = direction_deg * <FloatType as Real>::pi() / DEGREES_PER_HALF_TURN;
    (size * Real::cos(angle), size * Real::sin(angle))
}

fn larger(a: FloatType, b: FloatType) -> FloatType {
    if a > b { a } else { b }
}

fn smaller(a: FloatType, b: FloatType) -> FloatType {
    if a < b { a } else { b }
}
