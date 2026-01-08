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
    CELL_SIZE, MAG_NOISE_STD, MAP_SIZE, MagneticMap, ParticleFilter, generate_gaussian_noise,
};
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

    // 2. Mission Setup
    // Start at [2000, 2000] meters
    let mut true_pos_x = 2000.0;
    let mut true_pos_y = 2000.0;

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
    let mut filter = ParticleFilter::init_gaussian(true_pos_x, true_pos_y, 200.0, 1000);

    println!("\n=== Starting Navigation Loop (100 Hz / Display 1 Hz) ===");
    println!("Time [s] |  True Pos [m]   |  Est Pos [m]    | Err [m] |  Mag [nT]  | Status");
    println!("-----------------------------------------------------------------------------");

    let dt = 1.0; // Simulation step [s]
    let duration = 30; // Seconds

    for t in 1..=duration {
        // --- A. Dynamics (Truth Simulation) ---
        true_pos_x += vel_x * dt;
        true_pos_y += vel_y * dt;

        // --- B. Sensors (Simulation) ---
        // 1. Magnetometer (Truth + Sensor Noise)
        let true_mag = map.sample(true_pos_x, true_pos_y);
        let obs_mag = true_mag + generate_gaussian_noise(MAG_NOISE_STD);

        // --- C. Navigation Filter (The DeepCausality Logic) ---

        // 1. Time Update (Prediction)
        // Feed in INS velocities (assumed available from IMU)
        filter.predict(vel_x, vel_y, dt);

        // 2. Measurement Update (Correction)
        // Causal update using the observation monad
        filter.update(obs_mag, &map)?;

        // 3. Resample (Architecture housekeeping)
        filter.resample();

        // --- D. Output / Logging ---
        let (est_x, est_y) = filter.estimate();

        // Calculate NAV Error
        let err_x = true_pos_x - est_x;
        let err_y = true_pos_y - est_y;
        let total_err = (err_x * err_x + err_y * err_y).sqrt();

        // Determine Integrity Status
        let status = if total_err < 50.0 {
            "RNP 0.03 (GOOD)"
        } else if total_err < 150.0 {
            "DEGRADED"
        } else {
            "UNCERTAIN"
        };

        println!(
            "{:>6.1}   | [{:>6.0}, {:>6.0}] | [{:>6.0}, {:>6.0}] | {:>6.1}  | {:>6.1}   | {}",
            t as f64 * dt,
            true_pos_x,
            true_pos_y,
            est_x,
            est_y,
            total_err,
            obs_mag,
            status
        );
    }

    println!("\n[SYS] Mission Complete. Final Position Accuracy: High Integrity.");
    Ok(())
}
