/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{DifferentiableField, Euler, Scalar};
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_haft::Arrow;
use deep_causality_tensor::CausalTensor;
use std::error::Error;
use std::f64::consts::PI;
use std::ops::{Add, Mul};

/// 2-D position `(x, y)` in metres — the integrator state. `Euler` needs a module-valued state
/// (`Add` + scalar `Mul`), so the truth position rides in this newtype.
#[derive(Clone, Copy)]
pub struct Pos2(pub f64, pub f64);

impl Add for Pos2 {
    type Output = Pos2;
    fn add(self, o: Pos2) -> Pos2 {
        Pos2(self.0 + o.0, self.1 + o.1)
    }
}

impl Mul<f64> for Pos2 {
    type Output = Pos2;
    fn mul(self, s: f64) -> Pos2 {
        Pos2(self.0 * s, self.1 * s)
    }
}

/// The closed-form synthetic crustal anomaly field `B(fx, fy)`, written once as a scalar-generic
/// field. The grid is sampled from it (`run` at `f64`), and its spatial gradient `∇B` — the
/// navigation observable a gradient-aided filter uses, previously never computed — falls out of
/// the same definition via the tangent functor (`AnomalyField.gradient(&[fx, fy])`).
pub struct AnomalyField;

impl DifferentiableField<2> for AnomalyField {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let (fx, fy) = (p[0], p[1]);
        let c50 = S::from_f64(50.0).expect("constant lifts into the working scalar");
        let c20 = S::from_f64(20.0).expect("constant lifts into the working scalar");
        let c15 = S::from_f64(15.0).expect("constant lifts into the working scalar");
        let c03 = S::from_f64(0.3).expect("constant lifts into the working scalar");
        let c07 = S::from_f64(0.7).expect("constant lifts into the working scalar");
        // Base trend + high-frequency spatial variation (a realistic crustal anomaly).
        fx.sin() * fy.cos() * c50 + (fx * c03).sin() * c20 + (fy * c07).cos() * c15
    }
}

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
                // Sample the closed-form anomaly field that `AnomalyField` differentiates,
                // so the grid and the gradient `∇B` share one definition.
                data.push(AnomalyField.run(&[fx, fy]));
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
            let z: f64 = obs_ref.into_value().unwrap_or(0.0);

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
        if let Some(new_weights) = effect.value() {
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
    // A simple, deterministic LCG for reproducible tests (atomic, no `unsafe`).
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(123456789);
    let next = (SEED
        .load(Ordering::Relaxed)
        .wrapping_mul(1664525)
        .wrapping_add(1013904223))
        % 4294967296;
    SEED.store(next, Ordering::Relaxed);
    (next as f64) / 4294967296.0
}

/// The navigation state threaded through the `CausalFlow` pipeline in `main`
/// (`dynamics -> sensors -> filter -> output`). It carries the mutable per-tick state and the fixed
/// context (the magnetic map, the INS velocity, the step) so each stage can be a free-standing
/// sub-process `fn(NavState) -> CausalFlow<NavState>`.
pub struct NavState {
    pub true_pos: Pos2,
    pub filter: ParticleFilter,
    pub map: MagneticMap,
    pub vel_x: f64,
    pub vel_y: f64,
    pub dt: f64,
    pub obs_mag: f64,
    pub tick: u32,
}

impl NavState {
    pub fn new(
        true_pos: Pos2,
        filter: ParticleFilter,
        map: MagneticMap,
        vel_x: f64,
        vel_y: f64,
        dt: f64,
    ) -> Self {
        Self {
            true_pos,
            filter,
            map,
            vel_x,
            vel_y,
            dt,
            obs_mag: 0.0,
            tick: 0,
        }
    }
}

/// A. Dynamics (truth simulation): truth kinematics ẋ = v as one `Euler` endo-arrow step.
pub(crate) fn dynamics(mut s: NavState) -> CausalFlow<NavState> {
    let (vel_x, vel_y) = (s.vel_x, s.vel_y);
    let kinematics = Euler::new(s.dt, move |_: &Pos2| Pos2(vel_x, vel_y));
    s.true_pos = kinematics.run(s.true_pos);
    CausalFlow::value(s)
}

/// B. Sensors: magnetometer reading = field truth + sensor noise.
pub(crate) fn sensors(mut s: NavState) -> CausalFlow<NavState> {
    s.obs_mag = s.map.sample(s.true_pos.0, s.true_pos.1) + generate_gaussian_noise(MAG_NOISE_STD);
    CausalFlow::value(s)
}

/// C. Navigation filter: predict (INS) -> causal measurement update -> resample. A failed causal
/// update short-circuits the flow's error channel.
pub(crate) fn filter_update(mut s: NavState) -> CausalFlow<NavState> {
    s.filter.predict(s.vel_x, s.vel_y, s.dt);
    if let Err(e) = s.filter.update(s.obs_mag, &s.map) {
        return CausalFlow::fail(CausalityError::new(CausalityErrorEnum::Custom(
            e.to_string(),
        )));
    }
    s.filter.resample();
    CausalFlow::value(s)
}

/// D. Output / logging: weighted-mean estimate, NAV error, integrity status.
pub(crate) fn output(mut s: NavState) -> CausalFlow<NavState> {
    s.tick += 1;
    let (est_x, est_y) = s.filter.estimate();
    let (true_pos_x, true_pos_y) = (s.true_pos.0, s.true_pos.1);
    let total_err = ((true_pos_x - est_x).powi(2) + (true_pos_y - est_y).powi(2)).sqrt();
    let status = if total_err < 50.0 {
        "RNP 0.03 (GOOD)"
    } else if total_err < 150.0 {
        "DEGRADED"
    } else {
        "UNCERTAIN"
    };
    println!(
        "{:>6.1}   | [{:>6.0}, {:>6.0}] | [{:>6.0}, {:>6.0}] | {:>6.1}  | {:>6.1}   | {}",
        s.tick as f64 * s.dt,
        true_pos_x,
        true_pos_y,
        est_x,
        est_y,
        total_err,
        s.obs_mag,
        status
    );
    CausalFlow::value(s)
}
