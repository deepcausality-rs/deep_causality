/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Advanced Drone MagNav (Magnetic Navigation)
//!
//! **Scenario**: A tactical UAV loses GPS in a contested environment.
//! The Inertial Navigation System (INS) begins to drift. The Mission Computer activates
//! the "MagNav" mode to correlate magnetometer readings with an onboard crustal magnetic map.
//!
//! **Architecture**:
//! *   **Truth Sim**: Generates the "Real World" trajectory and sensor data.
//! *   **Estimator**: A Causal Particle Filter that predicts state via INS and corrects via Magnetometer.

mod model;

use crate::model::{
    AnomalyField, CELL_SIZE, MAP_SIZE, MagneticMap, NavState, ParticleFilter, Pos2,
};
use deep_causality_calculus::DifferentiateFieldExt;
use deep_causality_core::CausalFlow;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Flight Computer: MagNav Initialization ===");

    // 1. Load Magnetic Anomaly Database
    // In a real system, this would load a GeoTIFF or HDF5 file.
    let map = MagneticMap::new(MAP_SIZE, CELL_SIZE)?;
    println!(
        "[SYS] Magnetic Map Loaded. Resolution: {:.1}m, Area: {:.1}km^2",
        CELL_SIZE,
        (MAP_SIZE as f64 * CELL_SIZE / 1000.0).powi(2)
    );

    // The spatial gradient ∇B of the crustal anomaly field, the navigation observable a
    // gradient-aided filter steers by, via the tangent functor over the same closed-form field
    // the map is sampled from.
    let [db_dx, db_dy] = AnomalyField.gradient(&[1.0_f64, 1.0]);
    println!(
        "[NAV] Field gradient ∇B at start: [∂B/∂x = {db_dx:.3}, ∂B/∂y = {db_dy:.3}] nT/grid-unit"
    );

    // 2. Mission Setup
    // Start at [2000, 2000] meters
    let true_pos = Pos2(2000.0, 2000.0);

    // Flying Northeast at 25 m/s (~50 kts)
    let vel_x: f64 = 15.0;
    let vel_y: f64 = 20.0;
    let ground_speed = (vel_x * vel_x + vel_y * vel_y).sqrt();

    println!("[SYS] Mission Profile: SPEED={:.1} m/s", ground_speed);

    // 3. Estimator Initialization
    // We assume we had a rough GPS fix before jamming started, so we init with Gaussian uncertainty.
    // Init Uncertainty: 200m (sigma)
    // If we were completely lost, we'd use init_uniform()
    println!("[NAV] Initializing Particle Filter (N=1000)...");
    let filter = ParticleFilter::init_gaussian(true_pos.0, true_pos.1, 200.0, 1000);

    println!("\n=== Starting Navigation Loop (100 Hz / Display 1 Hz) ===");
    println!("Time [s] |  True Pos [m]   |  Est Pos [m]    | Err [m] |  Mag [nT]  | Status");
    println!("-----------------------------------------------------------------------------");

    let dt = 1.0; // Simulation step [s]
    let duration: usize = 30; // Seconds

    // The navigation loop as one CausalFlow: each tick is the pipeline
    // dynamics -> sensors -> filter -> output, run for `duration` ticks. The fallible causal
    // measurement update rides the flow's error channel.
    CausalFlow::value(NavState::new(true_pos, filter, map, vel_x, vel_y, dt))
        .iterate_n(duration, |tick| {
            tick.next(model::dynamics)
                .next(model::sensors)
                .next(model::filter_update)
                .next(model::output)
        })
        .finish()
        .map(|_| ())
        .map_err(|e| -> Box<dyn Error> { Box::from(format!("{e:?}")) })?;

    println!("\n[SYS] Mission Complete. Final Position Accuracy: High Integrity.");
    Ok(())
}
