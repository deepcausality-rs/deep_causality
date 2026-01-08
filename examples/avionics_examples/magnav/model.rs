/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use std::error::Error;
use std::f64::consts::PI;

// --- Constants & Configuration ---
pub const MAP_SIZE: usize = 120; // [km] Covers 120x120km area
pub const CELL_SIZE: f64 = 100.0; // [m] Grid resolution
pub const MAG_NOISE_STD: f64 = 5.0; // [nT] Magnetometer sensor noise
pub const INS_NOISE_STD: f64 = 0.5; // [m/s] Velocity drift/noise sigma

/// Represents the global Crustal Magnetic Field Map.
pub struct MagneticMap {
    pub grid: CausalTensor<f64>,
    pub scale: f64, // Meters per grid unit
}

impl MagneticMap {
    pub fn new(size: usize, scale: f64) -> Result<Self, Box<dyn Error>> {
        let mut data = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let fx = x as f64 * 0.05;
                let fy = y as f64 * 0.05;
                // Synthetic Anomaly: Base Trend + High Freq spatial variation
                // Simulates a realistic crustal anomaly field
                let val = (fx.sin() * fy.cos() * 50.0)
                    + ((fx * 0.3).sin() * 20.0)
                    + ((fy * 0.7).cos() * 15.0);
                data.push(val);
            }
        }
        let grid = CausalTensor::new(data, vec![size, size])?;
        Ok(Self { grid, scale })
    }

    /// Samples the magnetic field at a given position [x, y] in meters.
    /// Returns 0.0 if out of bounds.
    pub fn sample(&self, px: f64, py: f64) -> f64 {
        // Convert physical position (m) to grid coordinates
        let gx = px / self.scale;
        let gy = py / self.scale;

        let size = self.grid.shape()[0];
        if gx < 0.0 || gy < 0.0 || gx >= (size - 1) as f64 || gy >= (size - 1) as f64 {
            return 0.0;
        }

        // Bilinear Interpolation for sub-grid accuracy
        let x0 = gx.floor() as usize;
        let y0 = gy.floor() as usize;
        let dx = gx - x0 as f64;
        let dy = gy - y0 as f64;

        let idx = |cx, cy| cy * size + cx;

        let v00 = self.grid.data().get(idx(x0, y0)).copied().unwrap_or(0.0);
        let v10 = self
            .grid
            .data()
            .get(idx(x0 + 1, y0))
            .copied()
            .unwrap_or(0.0);
        let v01 = self
            .grid
            .data()
            .get(idx(x0, y0 + 1))
            .copied()
            .unwrap_or(0.0);
        let v11 = self
            .grid
            .data()
            .get(idx(x0 + 1, y0 + 1))
            .copied()
            .unwrap_or(0.0);

        let top = v00 * (1.0 - dx) + v10 * dx;
        let bottom = v01 * (1.0 - dx) + v11 * dx;

        top * (1.0 - dy) + bottom * dy
    }
}

/// A single hypothesis in the Particle Filter.
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f64, // Easting [m]
    pub y: f64, // Northing [m]
                // In a full filter, we'd simulate heading/bias too, but pos is enough for demo.
}

/// Causal Particle Filter for solving the "Kidnapped Robot" / Global Localization problem.
pub struct ParticleFilter {
    pub particles: Vec<Particle>,
    pub weights: Vec<f64>,
}

impl ParticleFilter {
    /// Initialize with a Gaussian distribution around a priori fix (e.g., last known GPS).
    pub fn init_gaussian(center_x: f64, center_y: f64, std_dev: f64, count: usize) -> Self {
        let mut particles = Vec::with_capacity(count);
        for _ in 0..count {
            particles.push(Particle {
                x: center_x + generate_gaussian_noise(std_dev),
                y: center_y + generate_gaussian_noise(std_dev),
            });
        }
        Self {
            particles,
            weights: vec![1.0 / count as f64; count],
        }
    }

    /// Time Update: Propagate particles based on INS (Control Input) + Noise.
    pub fn predict(&mut self, vel_x: f64, vel_y: f64, dt: f64) {
        for p in self.particles.iter_mut() {
            // Constant Velocity Motion Model + Process Noise
            p.x += (vel_x * dt) + generate_gaussian_noise(INS_NOISE_STD * dt.sqrt());
            p.y += (vel_y * dt) + generate_gaussian_noise(INS_NOISE_STD * dt.sqrt());
        }
    }

    /// Measurement Update: Causal weighting based on Magnetometer Observation.
    /// Uses PropagatingEffect to bind the observation to the belief state.
    pub fn update(&mut self, measured_mag: f64, map: &MagneticMap) -> Result<(), Box<dyn Error>> {
        // We lift the update into a causal effect
        // The observation is the "cause" of the weight change.
        let effect = PropagatingEffect::pure(measured_mag).bind(|obs_ref, _, _| {
            let z = match obs_ref {
                EffectValue::Value(v) => v,
                _ => 0.0,
            };

            // Compute unnormalized likelihoods
            // P(z|x) ~ exp( - (z - h(x))^2 / 2R )
            let likelihoods: Vec<f64> = self
                .particles
                .iter()
                .map(|p| {
                    let expected = map.sample(p.x, p.y);
                    let diff = z - expected;
                    let exponent = -0.5 * (diff * diff) / (MAG_NOISE_STD * MAG_NOISE_STD);
                    exponent.exp()
                })
                .collect();

            PropagatingEffect::pure(likelihoods)
        });

        // Apply weights
        if let EffectValue::Value(new_weights) = effect.value() {
            // Update and Normalize
            let total_weight: f64 = new_weights.iter().sum();
            if total_weight > 1e-12 {
                self.weights = new_weights.iter().map(|w| w / total_weight).collect();
            } else {
                // If particles are completely lost (all weights ~0), inject random particles (recovery)
                // For this demo, we just reset to uniform weights to avoid crash
                self.weights.fill(1.0 / self.particles.len() as f64);
            }
            Ok(())
        } else {
            Err("Causal update failed".into())
        }
    }

    /// Resampling (Systematic).
    /// Prevents degeneracy where all weight collapses to one particle.
    pub fn resample(&mut self) {
        // Calculate Effective Sample Size (N_eff)
        let n_eff = 1.0 / self.weights.iter().map(|w| w * w).sum::<f64>();

        // Resample only if necessary (e.g. N_eff < N/2)
        if n_eff < (self.particles.len() as f64 / 2.0) {
            let mut new_particles = Vec::with_capacity(self.particles.len());
            let step = 1.0 / self.particles.len() as f64;
            let mut u = rand_f64() * step;
            let mut c = self.weights[0];
            let mut i = 0;

            for _ in 0..self.particles.len() {
                while u > c {
                    i += 1;
                    if i >= self.weights.len() {
                        i = self.weights.len() - 1;
                    }
                    c += self.weights[i];
                }
                new_particles.push(self.particles[i]);
                u += step;
            }
            self.particles = new_particles;
            self.weights.fill(1.0 / self.particles.len() as f64);
        }
    }

    /// Estimate current position (Weighted Mean).
    pub fn estimate(&self) -> (f64, f64) {
        let mut x = 0.0;
        let mut y = 0.0;
        for (i, p) in self.particles.iter().enumerate() {
            x += p.x * self.weights[i];
            y += p.y * self.weights[i];
        }
        (x, y)
    }
}

// --- Utilities ---

pub fn generate_gaussian_noise(std_dev: f64) -> f64 {
    let u1: f64 = rand_f64();
    let u2: f64 = rand_f64();
    ((-2.0 * u1.ln()).sqrt()) * (2.0 * PI * u2).cos() * std_dev
}

fn rand_f64() -> f64 {
    // A simple, deterministic LCG for reproducible tests
    static mut SEED: u64 = 123456789;
    unsafe {
        SEED = (SEED.wrapping_mul(1664525).wrapping_add(1013904223)) % 4294967296;
        (SEED as f64) / 4294967296.0
    }
}
