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
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    EuclideanSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::CausalityError;
use deep_causality_multivector::{CausalMultiVector, Metric};

/// The radar world the tracker reads: the initial radar fix, the initial velocity estimate, and the
/// radar update period. The fix is a point of the example's 3-D Cartesian world, whose speed and
/// G-load use the Euclidean norm, so it is a `EuclideanSpace` contextoid in metres; the velocity
/// components and the period are `Data` contextoids. The context holds no clock or event, so its
/// temporal and spacetime slots are empty.
pub type RadarContext = Context<Data<f64>, EuclideanSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Node index: initial radar fix `(x, y, z)`, m (target detected at ~100 km range).
pub const INITIAL_FIX: usize = 0;
/// Node index: initial velocity estimate along x, m/s (drift).
pub const INITIAL_VEL_X_MS: usize = 1;
/// Node index: initial velocity estimate along y, m/s (closing fast).
pub const INITIAL_VEL_Y_MS: usize = 2;
/// Node index: radar update period, s (100 Hz -> 10 ms).
pub const UPDATE_PERIOD_S: usize = 3;

/// Build the radar world, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn build_radar_world() -> Result<RadarContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "radar acquisition", 4);
    context.add_node(Contextoid::new(
        1,
        ContextoidType::Spaceoid(EuclideanSpace::new(1, 0.0, 100_000.0, 20_000.0)),
    ))?;
    for (id, value) in [
        (2, 500.0),   // INITIAL_VEL_X_MS
        (3, -3400.0), // INITIAL_VEL_Y_MS
        (4, 0.01),    // UPDATE_PERIOD_S
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the radar world. A node that is absent or not a
/// Datoid is an error.
pub fn read(context: &RadarContext, index: usize) -> Result<f64, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "radar world node {index} is absent or not a Datoid"
            ))
        })
}

/// Read the initial radar fix `[x, y, z]` out of the radar world. A fix that is absent or not a
/// Spaceoid is an error.
pub fn initial_fix(context: &RadarContext) -> Result<[f64; 3], CausalityError> {
    context
        .get_node(INITIAL_FIX)
        .and_then(|node| node.vertex_type().spaceoid())
        .map(|fix| [fix.x(), fix.y(), fix.z()])
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "radar world node {INITIAL_FIX} is absent or not a Spaceoid"
            ))
        })
}

/// The radar update period a stage reads, or the error a stage returns when the flow carries no
/// radar world.
fn update_period(ctx: Option<&RadarContext>) -> Result<f64, CausalityError> {
    read(
        ctx.ok_or_else(CausalityError::MissingContext)?,
        UPDATE_PERIOD_S,
    )
}

/// 2T Physics Metric: (4, 2)
/// e1..e4 (Space), e_t1, e_t2 (Time)
pub fn metric_2t() -> Metric {
    Metric::Generic { p: 4, q: 2, r: 0 }
}

/// A tracker that operates in Conformal Phase Space (6D).
pub struct ConformalTracker {
    pub state_6d: CausalMultiVector<f64>, // Current belief state in 6D
    pub generator: CausalMultiVector<f64>, // The "Hamiltonian" / Motion Generator
    pub metric: Metric,
}

impl ConformalTracker {
    /// Initialize with a starting 3D position and estimated velocity/dynamics.
    pub fn new(x: f64, y: f64, z: f64, vx_init: f64, vy_init: f64) -> Self {
        let metric = metric_2t();

        // 1. Lift Initial Position to 6D Null Cone
        // Standard conformal embedding:
        // X = x + 0.5 x^2 n + \bar{n} (simplified for demo)
        // We map to indices directly.
        let mut data = vec![0.0; 64];
        data[1] = x;
        data[2] = y;
        data[4] = z;
        // Constraint X^2 = 0 requires balancing components.
        // For this demo, we assume the "Time" components balance the "Space" components
        data[16] = (x * x + y * y + z * z).sqrt(); // e_t1

        let state_6d = CausalMultiVector::unchecked(data, metric);

        // 2. Define Generator (Dynamics)
        // In 6D, constant velocity, acceleration, and conformal motion are all
        // subgroups of the spin group.
        // We set a generator that creates "Boost-Glide" like motion.
        let mut gen_data = vec![0.0; 64];
        gen_data[1] = vx_init;
        gen_data[2] = vy_init;
        gen_data[16] = vx_init; // Relativistic coupling

        let generator = CausalMultiVector::unchecked(gen_data, metric);

        Self {
            state_6d,
            generator,
            metric,
        }
    }

    /// Propagate state forward by dt.
    /// Uses linear evolution: X(t) = X(0) + G * t (First order approx).
    ///
    /// The constant-generator update `X += G·dt` is exactly one `Euler` step of `dX/dt = G`, so the
    /// hand-rolled component loop becomes a single integration-operator arrow. `CausalMultiVector`
    /// already supplies the vector `Add` and scalar `Mul` the endo-arrow needs.
    pub fn predict(&mut self, dt: f64) {
        let generator = self.generator.clone();
        let stepper = Euler::new(dt, move |_: &CausalMultiVector<f64>| generator.clone());
        self.state_6d = stepper.iterate_n(self.state_6d.clone(), 1);
    }

    /// Measurement Update (Mock).
    /// In a real filter, we would take a radar plot (r, az, el) and correct `state_6d`.
    /// Here we just simulate "Perfect Physics" propagation.
    pub fn correct(&mut self, _radar_plot: [f64; 3]) {
        // Placeholder for Kalman Gain application in 6D
    }

    /// Project current 6D state back to 3D world coordinates.
    pub fn get_3d_state(&self) -> [f64; 3] {
        let d = self.state_6d.data();
        // Extract shadow (e1, e2, e3)
        [d[1], d[2], d[4]]
    }
}

/// The per-tick tracking state threaded through the `CausalFlow` pipeline in `main`
/// (`predict -> observe -> derive`). The radar world rides in the flow's `Context` channel.
pub struct Track {
    pub tracker: ConformalTracker,
    pub prev_pos: [f64; 3],
    pub prev_vel_vec: [f64; 3],
    pub pos: [f64; 3],
    pub ms: f64,
}

/// Acquire the initial track from the radar world: target detected at ~100 km range, closing at
/// Mach 10.
pub fn build_initial_track(world: &RadarContext) -> Result<Track, CausalityError> {
    let [init_x, init_y, init_z] = initial_fix(world)?;
    let (vel_x, vel_y) = (
        read(world, INITIAL_VEL_X_MS)?,
        read(world, INITIAL_VEL_Y_MS)?,
    );
    Ok(Track {
        tracker: ConformalTracker::new(init_x, init_y, init_z, vel_x, vel_y),
        prev_pos: [init_x, init_y, init_z],
        prev_vel_vec: [vel_x, vel_y, 0.0],
        pos: [init_x, init_y, init_z],
        ms: 0.0,
    })
}

/// A. Prediction — one Euler step of the linear 6D conformal dynamics over one radar update
/// period, advancing the clock.
pub fn predict(mut t: Track, ctx: Option<&RadarContext>) -> Result<Track, CausalityError> {
    let dt = update_period(ctx)?;
    t.tracker.predict(dt);
    t.ms += dt * 1000.0;
    Ok(t)
}

/// B. Observation — project the 6D belief state back to 3D world coordinates.
pub fn observe(mut t: Track) -> Track {
    t.pos = t.tracker.get_3d_state();
    t
}

/// C. Derived metrics — finite-difference the full velocity vector for speed and G-load, then log
/// the track and roll the history. Differencing the vector (not just its magnitude) keeps the
/// lateral acceleration of a turn in the G-load, not only the change in speed.
pub fn derive(mut t: Track, ctx: Option<&RadarContext>) -> Result<Track, CausalityError> {
    let dt = update_period(ctx)?;
    // Velocity vector from the position delta; speed is its magnitude.
    let vel_vec = [
        (t.pos[0] - t.prev_pos[0]) / dt,
        (t.pos[1] - t.prev_pos[1]) / dt,
        (t.pos[2] - t.prev_pos[2]) / dt,
    ];
    let vel = (vel_vec[0].powi(2) + vel_vec[1].powi(2) + vel_vec[2].powi(2)).sqrt();

    // Acceleration is the change in the velocity vector, so a turn contributes lateral
    // acceleration even at constant speed. G-load is its magnitude over g.
    let accel = [
        (vel_vec[0] - t.prev_vel_vec[0]) / dt,
        (vel_vec[1] - t.prev_vel_vec[1]) / dt,
        (vel_vec[2] - t.prev_vel_vec[2]) / dt,
    ];
    let g_load = (accel[0].powi(2) + accel[1].powi(2) + accel[2].powi(2)).sqrt() / 9.81;

    println!(
        "{:>6.0}   | {:>9.1} | {:>10.1} | {:>9.1} | {:>9.1} | {:>5.1}G",
        t.ms, t.pos[0], t.pos[1], t.pos[2], vel, g_load
    );

    t.prev_pos = t.pos;
    t.prev_vel_vec = vel_vec;
    Ok(t)
}
