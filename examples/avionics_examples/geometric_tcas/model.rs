/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![allow(dead_code)]
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::Metric;
use deep_causality_multivector::MultiVector;

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

pub struct GeometricTCAS {
    horizontal_prot_radius: f64, // [m] e.g. 1000m
    vertical_prot_radius: f64,   // [m] e.g. 100m
    tau_threshold: f64,          // [s] e.g. 35s
}

impl GeometricTCAS {
    pub fn new() -> Self {
        Self {
            horizontal_prot_radius: 500.0, // Aggressive for demo
            vertical_prot_radius: 100.0,
            tau_threshold: 45.0,
        }
    }

    /// Primary Causal Logic: Detects threat and generates advisory.
    pub fn assess_threat(&self, own: &AircraftState, intruder: &AircraftState) -> ConflictReport {
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
        let assessment = PropagatingEffect::pure((d_cpa, t_cpa)).bind(|params_ref, _, _| {
            let (d, t) = match params_ref {
                EffectValue::Value(v) => v,
                _ => (f64::INFINITY, 0.0),
            };

            let level = if t < 0.0 {
                AdvisoryLevel::None // Passed
            } else if d < self.horizontal_prot_radius && t < self.tau_threshold {
                if t < 20.0 {
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
        let level = match assessment.value() {
            EffectValue::Value(l) => *l,
            _ => AdvisoryLevel::None,
        };

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

        ConflictReport {
            intruder_id: intruder.callsign.clone(),
            advisory: level,
            resolution,
            cpa_dist: d_cpa,
            t_cpa,
        }
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
