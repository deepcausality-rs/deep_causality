/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{ClockData, OrbitData};
use chrono::NaiveDateTime;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_physics::{EARTH_ROTATION_RATE, SpaceTimeCoordinate};

/// Maximum time gap (seconds) for centered finite difference in clock drift rate.
/// Two adjacent points must be within this gap for the derivative to be computed.
const MAX_GAP_CENTERED_DIFF_S: f64 = 7200.0; // 2 hours

/// Maximum time gap (seconds) for one-sided finite difference at boundaries.
const MAX_GAP_BOUNDARY_DIFF_S: f64 = 3600.0; // 1 hour

/// Convert a NaiveDateTime to seconds (Unix epoch, f64).
#[inline]
fn timestamp_to_seconds(ts: NaiveDateTime) -> f64 {
    ts.and_utc().timestamp_millis() as f64 / 1000.0
}

/// Interpolates orbit data to match clock timestamps using a 10th-order Lagrange polynomial.
///
/// # Type Parameter
/// - `R`: Output real field type (`f64`, `DoubleFloat`)
///
/// # Precision
///
/// All arithmetic (interpolation, division, velocity calculations) is performed in type `R`
/// to preserve precision. Raw f64 data from `ClockData`/`OrbitData` is cast to `R` immediately.
///
/// # Two-Pass Interpolation
///
/// This function uses a two-pass approach:
/// 1. **Pass 1**: Interpolate position/velocity for each clock timestamp
/// 2. **Pass 2**: Compute `clock_drift_rate` from `get_total_bias()` of adjacent coordinates
///
/// This is required because `get_total_bias()` includes a periodic relativistic correction
/// that depends on position and velocity, which is critical for accurate GM derivation.
///
/// # Inertial Velocity
///
/// The stored velocity `v_ms` is the **inertial velocity** (ECI frame), not the ECEF velocity.
/// This accounts for Earth's rotation: V_inertial = V_ecef + (Ω × r)
pub fn interpolate_space_time_single_pass<R>(
    clock_data: &[ClockData<R>],
    orbit_data: &[OrbitData<R>],
) -> Vec<SpaceTimeCoordinate<R>>
where
    R: RealField + From<f64> + Into<f64> + Clone,
{
    // =========================================================================
    // Single pass: Interpolate position/velocity AND compute drift rate inline
    // (matching legacy approach that uses ClockData indices directly)
    // =========================================================================
    let mut results = Vec::with_capacity(clock_data.len());
    let mut orbit_idx = 0;

    // Constants cast to R once
    let eps = R::from(0.01);
    let two = R::from(2.0);
    let omega = R::from(EARTH_ROTATION_RATE);
    let n_clocks = clock_data.len();

    for (i, clock) in clock_data.iter().enumerate() {
        let t_clock_f64 = timestamp_to_seconds(clock.timestamp());
        let t_clock = R::from(t_clock_f64);

        // Compute clock_drift_rate INLINE using ClockData indices (matching legacy!)
        let clock_drift_rate = if i > 0 && i < n_clocks - 1 {
            let prev = &clock_data[i - 1];
            let next = &clock_data[i + 1];
            let dt_prev = t_clock_f64 - timestamp_to_seconds(prev.timestamp());
            let dt_next = timestamp_to_seconds(next.timestamp()) - t_clock_f64;

            if dt_prev > 0.0 && dt_next > 0.0 && dt_prev < 3600.0 && dt_next < 3600.0 {
                // Centered difference: (f(x+h) - f(x-h)) / 2h
                let bias_prev: f64 = prev.bias_s().into();
                let bias_next: f64 = next.bias_s().into();
                R::from((bias_next - bias_prev) / (dt_next + dt_prev))
            } else {
                R::from(0.0) // Gap too large
            }
        } else {
            R::from(0.0) // Boundary
        };

        // Find the window of 10 points centered around the clock time
        while orbit_idx + 1 < orbit_data.len()
            && orbit_data[orbit_idx + 1].timestamp() <= clock.timestamp()
        {
            orbit_idx += 1;
        }

        // Window: 10 points from [orbit_idx - 4] to [orbit_idx + 5] (inclusive)
        // Slightly forward-biased to favor interpolation into known data
        let start_idx = orbit_idx as isize - 4;
        let end_idx = orbit_idx as isize + 5;

        if start_idx < 0 || end_idx >= orbit_data.len() as isize {
            // Not enough points for interpolation
            continue;
        }

        let start_idx = start_idx as usize;
        let end_idx = end_idx as usize;
        let window = &orbit_data[start_idx..=end_idx];

        // Prepare data for interpolation - cast to R immediately
        let times: Vec<R> = window
            .iter()
            .map(|p| R::from(timestamp_to_seconds(p.timestamp())))
            .collect();
        let xs: Vec<R> = window.iter().map(|p| p.x_m()).collect();
        let ys: Vec<R> = window.iter().map(|p| p.y_m()).collect();
        let zs: Vec<R> = window.iter().map(|p| p.z_m()).collect();

        // 1. Calculate Position P(t) - all in R precision
        let x = lagrange_interpolate(t_clock, &times, &xs);
        let y = lagrange_interpolate(t_clock, &times, &ys);
        let z = lagrange_interpolate(t_clock, &times, &zs);

        // 2. Calculate ECEF Velocity V(t) using numerical derivative - all in R precision
        let t_plus = t_clock + eps;
        let t_minus = t_clock - eps;

        let x_p = lagrange_interpolate(t_plus, &times, &xs);
        let y_p = lagrange_interpolate(t_plus, &times, &ys);
        let z_p = lagrange_interpolate(t_plus, &times, &zs);

        let x_m = lagrange_interpolate(t_minus, &times, &xs);
        let y_m = lagrange_interpolate(t_minus, &times, &ys);
        let z_m = lagrange_interpolate(t_minus, &times, &zs);

        let two_eps = two * eps;
        let vx_ecef = (x_p - x_m) / two_eps;
        let vy_ecef = (y_p - y_m) / two_eps;
        let vz_ecef = (z_p - z_m) / two_eps;

        // 3. Convert ECEF velocity to Inertial (ECI) velocity
        // V_inertial = V_ecef + (Ω × r), where Ω = [0, 0, ω_earth]
        // Cross product: Ω × r = [-ω*y, ω*x, 0]
        let vx_eci = vx_ecef + (-omega * y);
        let vy_eci = vy_ecef + (omega * x);
        let vz_eci = vz_ecef; // z-component unchanged

        // Calculate Magnitudes - all in R precision
        let r_m = (x * x + y * y + z * z).sqrt();
        let v_ms = (vx_eci * vx_eci + vy_eci * vy_eci + vz_eci * vz_eci).sqrt();

        results.push(SpaceTimeCoordinate {
            timestamp: clock.timestamp().and_utc().timestamp() as u64,
            sat_id: clock.sat_id().as_num(),
            r_m,
            v_ms,
            clock_bias_s: clock.bias_s(),
            position: [x, y, z],
            velocity: [vx_ecef, vy_ecef, vz_ecef],
            clock_drift_rate,
        });
    }

    results
}

/// Interpolates orbit data to match clock timestamps using a 10th-order Lagrange polynomial.
///
/// # Type Parameter
/// - `R`: Output real field type (`f64`, `DoubleFloat`)
///
/// # Precision
///
/// All arithmetic (interpolation, division, velocity calculations) is performed in type `R`
/// to preserve precision. Raw f64 data from `ClockData`/`OrbitData` is cast to `R` immediately.
///
/// # Two-Pass Interpolation
///
/// This function uses a two-pass approach:
/// 1. **Pass 1**: Interpolate position/velocity for each clock timestamp
/// 2. **Pass 2**: Compute `clock_drift_rate` from `get_total_bias()` of adjacent coordinates
///
/// This is required because `get_total_bias()` includes a periodic relativistic correction
/// that depends on position and velocity, which is critical for accurate GM derivation.
///
/// # Inertial Velocity
///
/// The stored velocity `v_ms` is the **inertial velocity** (ECI frame), not the ECEF velocity.
/// This accounts for Earth's rotation: V_inertial = V_ecef + (Ω × r)
pub fn interpolate_space_time<R>(
    clock_data: &[ClockData<R>],
    orbit_data: &[OrbitData<R>],
) -> Vec<SpaceTimeCoordinate<R>>
where
    R: RealField + From<f64> + Into<f64> + Clone + FromPrimitive,
{
    // =========================================================================
    // PASS 1: Interpolate position/velocity, store raw coordinates
    // =========================================================================
    let mut results = Vec::with_capacity(clock_data.len());
    let mut orbit_idx = 0;

    // Constants cast to R once
    let eps = R::from(0.01);
    let two = R::from(2.0);
    let omega = R::from(EARTH_ROTATION_RATE);

    for clock in clock_data.iter() {
        let t_clock = R::from(timestamp_to_seconds(clock.timestamp()));

        // Find the window of 10 points centered around the clock time
        while orbit_idx + 1 < orbit_data.len()
            && orbit_data[orbit_idx + 1].timestamp() <= clock.timestamp()
        {
            orbit_idx += 1;
        }

        // Window: 10 points from [orbit_idx - 4] to [orbit_idx + 5] (inclusive)
        // Slightly forward-biased to favor interpolation into known data
        let start_idx = orbit_idx as isize - 4;
        let end_idx = orbit_idx as isize + 5;

        if start_idx < 0 || end_idx >= orbit_data.len() as isize {
            // Not enough points for interpolation
            continue;
        }

        let start_idx = start_idx as usize;
        let end_idx = end_idx as usize;
        let window = &orbit_data[start_idx..=end_idx];

        // Prepare data for interpolation - cast to R immediately
        let times: Vec<R> = window
            .iter()
            .map(|p| R::from(timestamp_to_seconds(p.timestamp())))
            .collect();
        let xs: Vec<R> = window.iter().map(|p| p.x_m()).collect();
        let ys: Vec<R> = window.iter().map(|p| p.y_m()).collect();
        let zs: Vec<R> = window.iter().map(|p| p.z_m()).collect();

        // 1. Calculate Position P(t) - all in R precision
        let x = lagrange_interpolate(t_clock, &times, &xs);
        let y = lagrange_interpolate(t_clock, &times, &ys);
        let z = lagrange_interpolate(t_clock, &times, &zs);

        // 2. Calculate ECEF Velocity V(t) using numerical derivative - all in R precision
        let t_plus = t_clock + eps;
        let t_minus = t_clock - eps;

        let x_p = lagrange_interpolate(t_plus, &times, &xs);
        let y_p = lagrange_interpolate(t_plus, &times, &ys);
        let z_p = lagrange_interpolate(t_plus, &times, &zs);

        let x_m = lagrange_interpolate(t_minus, &times, &xs);
        let y_m = lagrange_interpolate(t_minus, &times, &ys);
        let z_m = lagrange_interpolate(t_minus, &times, &zs);

        let two_eps = two * eps;
        let vx_ecef = (x_p - x_m) / two_eps;
        let vy_ecef = (y_p - y_m) / two_eps;
        let vz_ecef = (z_p - z_m) / two_eps;

        // 3. Convert ECEF velocity to Inertial (ECI) velocity
        // V_inertial = V_ecef + (Ω × r), where Ω = [0, 0, ω_earth]
        // Cross product: Ω × r = [-ω*y, ω*x, 0]
        let vx_eci = vx_ecef + (-omega * y);
        let vy_eci = vy_ecef + (omega * x);
        let vz_eci = vz_ecef; // z-component unchanged

        // Calculate Magnitudes - all in R precision
        let r_m = (x * x + y * y + z * z).sqrt();
        let v_ms = (vx_eci * vx_eci + vy_eci * vy_eci + vz_eci * vz_eci).sqrt();

        // Store with placeholder clock_drift_rate (will be computed in pass 2)
        results.push(SpaceTimeCoordinate {
            timestamp: clock.timestamp().and_utc().timestamp() as u64,
            sat_id: clock.sat_id().as_num(),
            r_m,
            v_ms,
            clock_bias_s: clock.bias_s(),
            position: [x, y, z],
            velocity: [vx_ecef, vy_ecef, vz_ecef], // Store ECEF for get_total_bias()
            clock_drift_rate: R::from(0.0),        // Placeholder
        });
    }

    // =========================================================================
    // PASS 2: Compute clock_drift_rate from get_total_bias() of adjacent coordinates
    // =========================================================================
    //
    // This is critical: get_total_bias() adds the periodic relativistic correction
    // Δt_periodic = -2(r·v)/c² which was removed by IGS from published biases.
    //
    // The numerical derivative uses centered finite difference:
    // rate[i] = (bias[i+1] - bias[i-1]) / (t[i+1] - t[i-1])
    //
    // All operations are performed in R to avoid precision loss.
    let n = results.len();
    if n < 3 {
        return results;
    }

    // Compute rates for interior points - all in R precision
    for i in 1..n - 1 {
        let prev = &results[i - 1];
        let next = &results[i + 1];

        let dt = (next.timestamp as f64) - (prev.timestamp as f64);

        if dt > 0.0 && dt < MAX_GAP_CENTERED_DIFF_S {
            let bias_prev = prev.get_total_bias();
            let bias_next = next.get_total_bias();
            let dt_r = R::from(dt);
            let rate = (bias_next - bias_prev) / dt_r;
            results[i].clock_drift_rate = rate;
        }
    }

    // Handle boundaries using one-sided differences
    if n >= 2 {
        // First point: forward difference
        let dt0 = (results[1].timestamp as f64) - (results[0].timestamp as f64);
        if dt0 > 0.0 && dt0 < MAX_GAP_BOUNDARY_DIFF_S {
            let bias0 = results[0].get_total_bias();
            let bias1 = results[1].get_total_bias();
            let dt0_r = R::from(dt0);
            results[0].clock_drift_rate = (bias1 - bias0) / dt0_r;
        }

        // Last point: backward difference
        let dtn = (results[n - 1].timestamp as f64) - (results[n - 2].timestamp as f64);
        if dtn > 0.0 && dtn < MAX_GAP_BOUNDARY_DIFF_S {
            let bias_n1 = results[n - 2].get_total_bias();
            let bias_n = results[n - 1].get_total_bias();
            let dtn_r = R::from(dtn);
            results[n - 1].clock_drift_rate = (bias_n - bias_n1) / dtn_r;
        }
    }

    results
}

/// Generic Lagrange Polynomial Interpolation.
///
/// Computes the value of the polynomial passing through all (x_values[i], y_values[i])
/// points at the target point `x`.
///
/// # Precision
///
/// All arithmetic is performed in type `R` to preserve precision for high-order
/// polynomials (10th order requires ~100 division operations).
///
/// # Numerical Stability
///
/// Uses the standard Lagrange form. For improved stability with very high-order
/// polynomials, consider the barycentric form in future iterations.
fn lagrange_interpolate<R>(x: R, x_values: &[R], y_values: &[R]) -> R
where
    R: RealField,
{
    let n = x_values.len();
    let mut result = R::zero();

    for i in 0..n {
        let mut term = y_values[i];
        for j in 0..n {
            if i != j {
                let denom = x_values[i] - x_values[j];
                assert!(
                    denom != R::zero(),
                    "lagrange_interpolate: duplicate x-values at indices {i} and {j} \
                     produce a singular basis polynomial — caller must supply distinct nodes"
                );
                term = term * (x - x_values[j]) / denom;
            }
        }
        result += term;
    }

    result
}
