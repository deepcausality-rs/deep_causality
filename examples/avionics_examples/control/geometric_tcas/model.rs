/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![allow(dead_code)]
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{EndoArrow, Euler};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::Metric;
use deep_causality_multivector::MultiVector;

// --- Context ---

/// The encounter world the safety loop reads through the flow's `Context` channel: the TCAS
/// protection radius and alert thresholds and the surveillance tick period, one `Data` contextoid
/// per quantity. The aircraft tracks are the evolving value, not context, so the spatial, temporal
/// and spacetime slots are empty.
pub type TcasContext = Context<Data<f64>, NoSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Contextoid id: protection radius on the 3-D CPA miss distance, m (aggressive for the demo).
pub const CPA_PROT_RADIUS_M: ContextoidId = 1;
/// Contextoid id: time-to-CPA below which a traffic advisory issues, s.
pub const TA_TAU_S: ContextoidId = 2;
/// Contextoid id: time-to-CPA below which a traffic advisory becomes a resolution advisory, s.
pub const RA_TAU_S: ContextoidId = 3;
/// Contextoid id: surveillance tick period, s.
pub const TICK_S: ContextoidId = 4;

/// Build the encounter world, each quantity keyed by its contextoid id.
pub fn build_tcas_world() -> Result<TcasContext, ContextIndexError> {
    let facts = [
        (CPA_PROT_RADIUS_M, 500.0),
        (TA_TAU_S, 45.0),
        (RA_TAU_S, 20.0),
        (TICK_S, 0.5),
    ];
    let mut context = Context::with_capacity(1, "tcas encounter", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read the `Data` contextoid carrying `id` out of the encounter world. An id the world does not
/// hold as a Datoid is an error.
pub fn read(context: &TcasContext, id: ContextoidId) -> Result<f64, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "encounter world holds no Datoid with contextoid id {id}"
        ))
    })
}

/// The encounter world a stage reads, or the error a stage returns when the flow carries none.
fn world(ctx: Option<&TcasContext>) -> Result<&TcasContext, CausalityError> {
    ctx.ok_or_else(CausalityError::MissingContext)
}

// --- Types ---

#[derive(Debug, Clone, PartialEq)]
pub struct AircraftState {
    pub callsign: String,
    pub pos: CausalMultiVector<f64>, // [x, y, z] in meters (Local ENU)
    pub vel: CausalMultiVector<f64>, // [vx, vy, vz] in m/s
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AdvisoryLevel {
    #[default]
    None,
    TA, // Traffic Advisory
    RA, // Resolution Advisory
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolution {
    Maintain,
    Climb,
    Descend,
    TurnLeft,
    TurnRight,
}

#[derive(Debug)]
pub struct ConflictReport {
    pub intruder_id: String,
    pub advisory: AdvisoryLevel,
    pub resolution: Resolution,
    pub cpa_dist: f64, // [m]
    pub t_cpa: f64,    // [s]
}

// --- Logic ---

/// The TCAS monitor over an encounter world: its protection radius and alert thresholds are read
/// from the world on every assessment.
pub struct GeometricTCAS<'w> {
    world: &'w TcasContext,
}

impl<'w> GeometricTCAS<'w> {
    pub fn new(world: &'w TcasContext) -> Self {
        Self { world }
    }

    /// Primary Causal Logic: Detects threat and generates advisory.
    pub fn assess_threat(
        &self,
        own: &AircraftState,
        intruder: &AircraftState,
    ) -> Result<ConflictReport, CausalityError> {
        // Causal Chain:
        // Relative State -> CPA Geometry -> Risk Assessment -> Advisory
        // We implicitly assume linear extrapolation for the "Lookahead"

        // 1. Relative State (P_rel, V_rel)
        // Note: In GA, differences are straightforward.
        // We assume Sub is implemented/available or do discrete ops.
        // For robustness in this example, we extract and rebuild.
        // (Assuming simple Euclidean Metric(3) for both)

        let p_rel = self.sub_vec(&intruder.pos, &own.pos); // Intruder relative to Own
        let v_rel = self.sub_vec(&intruder.vel, &own.vel);

        // 2. Geometric CPA Calc
        let (d_cpa, t_cpa) = self.calculate_cpa(&p_rel, &v_rel);

        // 3. Decision Logic (Monadic Bind simulation)
        // We use PropagatingEffect to represent the "Safety Interlock"
        // If parameters are safe, effect is "Clear". If not, "Advisory".
        let cpa_prot_radius = read(self.world, CPA_PROT_RADIUS_M)?;
        let ta_tau = read(self.world, TA_TAU_S)?;
        let ra_tau = read(self.world, RA_TAU_S)?;
        let assessment = PropagatingEffect::pure((d_cpa, t_cpa)).bind(|params_ref, _, _| {
            let Some((d, t)) = params_ref.into_value() else {
                return PropagatingEffect::from_error(CausalityError::ValueNotAvailable());
            };

            let level = if t < 0.0 {
                AdvisoryLevel::None // Passed
            } else if d < cpa_prot_radius && t < ta_tau {
                if t < ra_tau {
                    AdvisoryLevel::RA
                } else {
                    AdvisoryLevel::TA
                }
            } else {
                AdvisoryLevel::None
            };

            PropagatingEffect::pure(level)
        });

        // 4. Formulate Report
        let (outcome, _, _, _) = assessment.into_parts();
        let level = outcome?
            .into_value()
            .ok_or_else(CausalityError::ValueNotAvailable)?;

        let resolution = if level == AdvisoryLevel::RA {
            // Heuristic Resolution Logic
            // In full GA, we'd rotate V_rel away from the collision cone.
            // Here: Simple Vertical Logic
            let rel_z = p_rel.data()[4]; // e3 component
            if rel_z > 0.0 {
                Resolution::Descend // Intruder is above, go down
            } else {
                Resolution::Climb // Intruder is below, go up
            }
        } else {
            Resolution::Maintain
        };

        Ok(ConflictReport {
            intruder_id: intruder.callsign.clone(),
            advisory: level,
            resolution,
            cpa_dist: d_cpa,
            t_cpa,
        })
    }

    // --- Helpers ---

    fn sub_vec(
        &self,
        a: &CausalMultiVector<f64>,
        b: &CausalMultiVector<f64>,
    ) -> CausalMultiVector<f64> {
        let da = a.data();
        let db = b.data();
        let diff: Vec<f64> = da.iter().zip(db.iter()).map(|(x, y)| x - y).collect();
        CausalMultiVector::unchecked(diff, Metric::Euclidean(3))
    }

    fn calculate_cpa(&self, p: &CausalMultiVector<f64>, v: &CausalMultiVector<f64>) -> (f64, f64) {
        // CPA Distance = |P ^ V| / |V|
        // Time to CPA = -(P . V) / |V|^2

        let v_sq = v.squared_magnitude();
        if v_sq < 1e-6 {
            return (p.squared_magnitude().sqrt(), 0.0);
        }

        // Wedge product magnitude
        let wedge = p.outer_product(v);
        let area = wedge.squared_magnitude().sqrt();
        let d_cpa = area / v_sq.sqrt();

        // Dot product
        let dot = p.inner_product(v).data()[0];
        let t_cpa = -dot / v_sq;

        (d_cpa, t_cpa)
    }
}

// Factory for Vectors
pub fn vec3(x: f64, y: f64, z: f64) -> CausalMultiVector<f64> {
    let mut d = vec![0.0; 8];
    d[1] = x; // e1
    d[2] = y; // e2
    d[4] = z; // e3
    CausalMultiVector::unchecked(d, Metric::Euclidean(3))
}

pub fn add_vec(a: &CausalMultiVector<f64>, b: &CausalMultiVector<f64>) -> CausalMultiVector<f64> {
    let da = a.data();
    let db = b.data();
    let sum: Vec<f64> = da.iter().zip(db.iter()).map(|(x, y)| x + y).collect();
    CausalMultiVector::unchecked(sum, Metric::Euclidean(3))
}

pub fn scale_vec(v: &CausalMultiVector<f64>, s: f64) -> CausalMultiVector<f64> {
    let d = v.data();
    let scaled: Vec<f64> = d.iter().map(|x| x * s).collect();
    CausalMultiVector::unchecked(scaled, Metric::Euclidean(3))
}

// --- Encounter pipeline ---

/// The per-tick encounter state threaded through the TCAS `CausalFlow` pipeline in `main`
/// (`assess -> intervene? -> output -> integrate`). It carries the mutable state; the monitor's
/// thresholds and the tick period are the [`TcasContext`] in the flow's `Context` channel, which
/// each stage reads.
pub struct Engagement {
    pub ownship: AircraftState,
    pub intruder: AircraftState,
    pub ra_duration: f64,
    pub tick: u32,
    pub report: Option<ConflictReport>,
    pub triggered: bool,
    pub will_intervene: bool,
}

/// Set up the converging head-on encounter (slightly offset in altitude).
pub fn build_initial_engagement() -> Engagement {
    Engagement {
        ownship: AircraftState {
            callsign: "AIRBUS_01".into(),
            pos: vec3(0.0, 0.0, 10000.0),
            vel: vec3(0.0, 200.0, 0.0),
        },
        intruder: AircraftState {
            callsign: "UNK_TRAFFIC".into(),
            pos: vec3(0.0, 8000.0, 10050.0),
            vel: vec3(0.0, -200.0, 0.0),
        },
        ra_duration: 0.0,
        tick: 0,
        report: None,
        triggered: false,
        will_intervene: false,
    }
}

/// A. Assess the threat, accumulate RA persistence, and decide whether to auto-intervene.
pub fn assess(mut e: Engagement, ctx: Option<&TcasContext>) -> Result<Engagement, CausalityError> {
    let world = world(ctx)?;
    let report = GeometricTCAS::new(world).assess_threat(&e.ownship, &e.intruder)?;
    let tick_s = read(world, TICK_S)?;
    e.triggered = if report.advisory == AdvisoryLevel::RA {
        e.ra_duration += tick_s;
        e.ra_duration > 2.5 && report.resolution == Resolution::Descend
    } else {
        e.ra_duration = 0.0;
        false
    };
    e.will_intervene = e.triggered && e.ownship.vel.data()[4] > -20.0;
    e.report = Some(report);
    Ok(e)
}

/// B. Auto-pilot takeover (the conditional branch arm): force a descent on the ownship velocity
/// and record the override.
pub fn intervene(mut e: Engagement) -> Engagement {
    let mut d = e.ownship.vel.data().clone();
    d[4] = (d[4] - 5.0).max(-20.0);
    e.ownship.vel = CausalMultiVector::unchecked(d, Metric::Euclidean(3));
    println!("      > [BLACKBOX AUDIT]: Automatic Intervention Recorded.");
    e
}

/// C. Output the advisory row.
pub fn output(mut e: Engagement, ctx: Option<&TcasContext>) -> Result<Engagement, CausalityError> {
    let tick_s = read(world(ctx)?, TICK_S)?;
    let report = e.report.as_ref().ok_or_else(|| {
        CausalityError::MissingParameter("no conflict report: assess has not run this tick")
    })?;
    let time = e.tick as f64 * tick_s;
    let sys_status = if e.will_intervene {
        " [\x1b[31mAUTO INTERVENE\x1b[0m]"
    } else if e.triggered {
        " [\x1b[32mAVOIDING\x1b[0m]"
    } else {
        ""
    };
    let alert_str = match report.advisory {
        AdvisoryLevel::None => "CLEAR",
        AdvisoryLevel::TA => "\x1b[33mTRAFFIC ADVISORY\x1b[0m",
        AdvisoryLevel::RA => "\x1b[31mRES ADVISORY\x1b[0m",
    };
    let res_str = match report.resolution {
        Resolution::Maintain => "-".to_string(),
        _ => format!("{:?}", report.resolution).to_uppercase(),
    };
    println!(
        "{:>6.1}  | {:>8.0} | {:>7.1}  | {:>7.1}  | {:<16} | {}{}",
        time,
        report.t_cpa * 400.0,
        report.t_cpa,
        report.cpa_dist,
        alert_str,
        res_str,
        sys_status
    );
    e.tick += 1;
    Ok(e)
}

/// D. Update dynamics: one Euler step of each aircraft's constant-velocity kinematics over one
/// tick period.
pub fn integrate(
    mut e: Engagement,
    ctx: Option<&TcasContext>,
) -> Result<Engagement, CausalityError> {
    let dt = read(world(ctx)?, TICK_S)?;
    let own_vel = e.ownship.vel.clone();
    e.ownship.pos = Euler::new(dt, move |_: &CausalMultiVector<f64>| own_vel.clone())
        .iterate_n(e.ownship.pos.clone(), 1);
    let intr_vel = e.intruder.vel.clone();
    e.intruder.pos = Euler::new(dt, move |_: &CausalMultiVector<f64>| intr_vel.clone())
        .iterate_n(e.intruder.pos.clone(), 1);
    Ok(e)
}
