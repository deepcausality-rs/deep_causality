/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{einstein_tensor_kernel, geodesic_deviation_kernel};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// einstein_tensor_kernel Tests
// =============================================================================

#[test]
fn test_einstein_tensor_kernel_valid() {
    // G_uv = R_uv - 0.5 * R * g_uv, with every input distinct so the two tensors cannot be
    // confused for one another and the 0.5 is pinned by the answer rather than by cancellation.
    //   R_uv = [[1, 2], [3, 4]],  g_uv = [[1, 0], [0, -1]],  R = 6
    //   G = [[1, 2], [3, 4]] - 3 * [[1, 0], [0, -1]] = [[-2, 2], [3, 7]]
    let ricci = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let scalar_r = 6.0;

    let g = einstein_tensor_kernel(&ricci, scalar_r, &metric).unwrap();

    // All four components, not just the first: a wrong factor or a swapped operand moves at
    // least one of them.
    let expected = [-2.0_f64, 2.0, 3.0, 7.0];
    for (i, want) in expected.iter().enumerate() {
        assert!(
            (g.data()[i] - want).abs() < 1e-10,
            "G[{i}] = {}, expected {want}",
            g.data()[i]
        );
    }
}

#[test]
fn test_einstein_tensor_is_linear_in_the_ricci_tensor() {
    // G(R_uv + S_uv, R + S, g) == G(R_uv, R, g) + G(S_uv, S, g). Holds for any g and needs no
    // oracle; a non-linear error in the trace term breaks it.
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![-0.5, 1.5, 0.25, 2.0], vec![2, 2]).unwrap();
    let sum = CausalTensor::new(vec![0.5, 3.5, 3.25, 6.0], vec![2, 2]).unwrap();

    let g_a = einstein_tensor_kernel(&a, 6.0, &metric).unwrap();
    let g_b = einstein_tensor_kernel(&b, 1.5, &metric).unwrap();
    let g_sum = einstein_tensor_kernel(&sum, 7.5, &metric).unwrap();

    for i in 0..4 {
        let residual: f64 = g_sum.data()[i] - (g_a.data()[i] + g_b.data()[i]);
        assert!(
            residual.abs() < 1e-10,
            "linearity broken at component {i}: residual {residual}"
        );
    }
}

#[test]
fn test_einstein_tensor_kernel_dimension_error() {
    // A rank-1 metric against a rank-2 Ricci. The rank guard and the shape guard both reject
    // this, so the message is what says which one fired; without it a guard that tested only the
    // Ricci rank would look identical.
    let ricci = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 2], vec![2]).unwrap(); // Rank 1
    let err = einstein_tensor_kernel(&ricci, 2.0, &metric).unwrap_err();
    let message = format!("{err}");
    assert!(
        message.contains("rank-2"),
        "the rank guard must fire first; got {message}"
    );
}

#[test]
fn test_einstein_tensor_kernel_shape_mismatch() {
    let ricci = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = einstein_tensor_kernel(&ricci, 2.0, &metric);
    assert!(result.is_err());
}

#[test]
fn test_einstein_tensor_kernel_non_square() {
    let ricci = CausalTensor::new(vec![1.0; 4], vec![1, 4]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 4], vec![1, 4]).unwrap();
    let result = einstein_tensor_kernel(&ricci, 2.0, &metric);
    assert!(result.is_err());
}

#[test]
fn test_einstein_tensor_kernel_flat_space() {
    // Flat Minkowski: R_uv = 0, R = 0
    let ricci = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let scalar_r = 0.0;

    let result = einstein_tensor_kernel(&ricci, scalar_r, &metric);
    assert!(result.is_ok());

    let g = result.unwrap();
    // G = 0 - 0 = 0
    for val in g.data() {
        assert!(*val == 0.0);
    }
}

// =============================================================================
// geodesic_deviation_kernel Tests
// =============================================================================

#[test]
fn test_geodesic_deviation_vanishes_for_a_flat_riemann_tensor() {
    // The flat limit. It pins nothing on its own — any kernel returning zero satisfies it — so it
    // sits beside the non-zero contraction below rather than standing in for it.
    let riemann = CausalTensor::new(vec![0.0f64; 256], vec![4, 4, 4, 4]).unwrap();
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let a = geodesic_deviation_kernel(&riemann, &velocity, &separation).unwrap();
    for val in a.iter() {
        assert!(*val == 0.0);
    }
}

#[test]
fn test_geodesic_deviation_contracts_the_riemann_tensor_in_the_documented_index_order() {
    // A^mu = -R^mu_{v r s} u^v n^r u^s, with a single non-zero Riemann component so the whole
    // contraction is one product that can be written out by hand.
    //
    // The kernel is 4D: index [mu][v][r][s] is mu*64 + v*16 + r*4 + s.
    // Set R^0_{1 0 1} = 2, everything else zero. With u = (1, 3, 0, 0) and n = (5, 7, 0, 0):
    //   A^0 = -R^0_{1 0 1} * u^1 * n^0 * u^1 = -(2 * 3 * 5 * 3) = -90
    //   A^1 = A^2 = A^3 = 0
    // Every slot is distinguishable: u and n carry different values and the two u slots carry
    // different indices, so pairing n with the s slot instead of r gives 2*3*1*7 = 42, not 90.
    let mut r_data = vec![0.0f64; 256];
    r_data[17] = 2.0; // mu = 0, v = 1, r = 0, s = 1 -> 0*64 + 1*16 + 0*4 + 1
    let riemann = CausalTensor::new(r_data, vec![4, 4, 4, 4]).unwrap();
    let u: [f64; 4] = [1.0, 3.0, 0.0, 0.0];
    let n: [f64; 4] = [5.0, 7.0, 0.0, 0.0];

    let a = geodesic_deviation_kernel(&riemann, &u, &n).unwrap();
    assert!((a[0] - (-90.0)).abs() < 1e-12, "A^0 = {}", a[0]);
    for (mu, component) in a.iter().enumerate().skip(1) {
        assert!(component.abs() < 1e-12, "A^{mu} = {component}");
    }
}

#[test]
fn test_geodesic_deviation_is_linear_in_the_separation() {
    // n -> k n scales the answer by k, for any Riemann tensor. No oracle needed.
    let mut r_data = vec![0.0f64; 256];
    r_data[17] = 2.0; // R^0_{1 0 1}
    r_data[64 + 4] = -0.75; // R^1_{0 1 0}
    let riemann = CausalTensor::new(r_data, vec![4, 4, 4, 4]).unwrap();
    let u: [f64; 4] = [1.0, 3.0, 0.0, 0.0];

    let base = geodesic_deviation_kernel(&riemann, &u, &[5.0, 7.0, 0.0, 0.0]).unwrap();
    for k in [0.5_f64, 2.0, -3.0] {
        let scaled =
            geodesic_deviation_kernel(&riemann, &u, &[5.0 * k, 7.0 * k, 0.0, 0.0]).unwrap();
        for mu in 0..4 {
            assert!(
                (scaled[mu] - k * base[mu]).abs() < 1e-12,
                "k = {k}, component {mu}"
            );
        }
    }
}

#[test]
fn test_geodesic_deviation_kernel_dimension_error() {
    // Wrong ranks should error
    let riemann = CausalTensor::new(vec![1.0f64; 4], vec![2, 2]).unwrap(); // Rank 2, not 4
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let result = geodesic_deviation_kernel(&riemann, &velocity, &separation);
    assert!(result.is_err(), "Should error on wrong Riemann rank");
}

#[test]
fn test_geodesic_deviation_kernel_vector_length_error() {
    // Correct rank-4 Riemann but velocity/separation are not length 4 → the
    // `u.len() != dim || n.len() != dim` guard must error.
    let riemann = CausalTensor::new(vec![0.0f64; 256], vec![4, 4, 4, 4]).unwrap();

    // Velocity too short
    let bad_u: [f64; 3] = [1.0, 0.0, 0.0];
    let good_n: [f64; 4] = [0.0, 1.0, 0.0, 0.0];
    assert!(
        geodesic_deviation_kernel(&riemann, &bad_u, &good_n).is_err(),
        "Velocity of length 3 must error"
    );

    // Separation too long
    let good_u: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let bad_n: [f64; 5] = [0.0, 1.0, 0.0, 0.0, 0.0];
    assert!(
        geodesic_deviation_kernel(&riemann, &good_u, &bad_n).is_err(),
        "Separation of length 5 must error"
    );
}

// =============================================================================
// geodesic_integrator_kernel Tests
// =============================================================================

use deep_causality_physics::geodesic_integrator_kernel;

#[test]
fn test_geodesic_integrator_kernel_flat_space() {
    // In flat space (Γ = 0), geodesics are straight lines
    let initial_position: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.1, 0.0, 0.0]; // Moving mostly in t

    // Zero Christoffel symbols (flat space)
    let christoffel = CausalTensor::new(vec![0.0f64; 64], vec![4, 4, 4]).unwrap();

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_ok());

    let trajectory = result.unwrap();
    assert_eq!(trajectory.len(), 11); // Initial + 10 steps

    // In flat space, velocity should remain constant
    let (_, final_vel) = &trajectory[10];
    assert!((final_vel[0] - initial_velocity[0]).abs() < 1e-10);
    assert!((final_vel[1] - initial_velocity[1]).abs() < 1e-10);
}

#[test]
fn test_geodesic_integrator_kernel_straight_line() {
    // Verify position changes linearly in flat space
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 2.0];

    let christoffel = CausalTensor::new(vec![0.0f64; 8], vec![2, 2, 2]).unwrap();
    let dt = 0.1;
    let num_steps = 5;

    let result = geodesic_integrator_kernel(
        &initial_position,
        &initial_velocity,
        &christoffel,
        dt,
        num_steps,
    );
    assert!(result.is_ok());

    let trajectory = result.unwrap();
    let (final_pos, _) = &trajectory[num_steps];

    // Expected: x = x0 + v * t = 0 + [1, 2] * 0.5 = [0.5, 1.0]
    let expected_x = initial_velocity[0] * dt * num_steps as f64;
    let expected_y = initial_velocity[1] * dt * num_steps as f64;

    assert!(
        (final_pos[0] - expected_x).abs() < 1e-9,
        "Expected x={}, got {}",
        expected_x,
        final_pos[0]
    );
    assert!(
        (final_pos[1] - expected_y).abs() < 1e-9,
        "Expected y={}, got {}",
        expected_y,
        final_pos[1]
    );
}

// The five tests above all set the connection to zero, where the whole term
// `-Gamma^mu_{nu rho} u^nu u^rho` vanishes and the integrator reduces to straight-line motion.
// The two below give it a non-zero connection, which is the only way the contraction, its sign
// and the integrator's order are pinned by anything.

/// Closed-form solution of the geodesic equation for a constant connection with a single
/// non-zero component.
///
/// With `Gamma^0_{00} = c` and every other component zero, `a^0 = -c (u^0)^2` and the system
/// decouples:
///
/// ```text
///   du/dtau = -c u^2   =>   u(tau) = u0 / (1 + c u0 tau)
///   x(tau) = x0 + (1/c) ln(1 + c u0 tau)
/// ```
///
/// This is an oracle in the strict sense: it is the analytic solution of the equation the kernel
/// claims to integrate, derived independently of the integrator.
fn constant_connection_exact(c: f64, u0: f64, tau: f64) -> (f64, f64) {
    let u = u0 / (1.0 + c * u0 * tau);
    let x = (1.0 + c * u0 * tau).ln() / c;
    (x, u)
}

/// Runs the constant-connection case on a chosen axis.
///
/// `axis` selects which component carries both the connection and the motion, so the flat index
/// `mu * dim * dim + nu * dim + rho` is exercised away from zero. On axis 0 every term of that
/// expression vanishes and a mutated stride is indistinguishable from the correct one.
fn constant_connection_run_on(
    axis: usize,
    c: f64,
    u0: f64,
    horizon: f64,
    steps: usize,
) -> (f64, f64) {
    let dim = 2usize;
    let other = 1 - axis;
    let mut gamma = vec![0.0f64; dim * dim * dim]; // index mu*4 + nu*2 + rho
    gamma[axis * dim * dim + axis * dim + axis] = c; // Gamma^axis_{axis axis}
    let christoffel = CausalTensor::new(gamma, vec![dim, dim, dim]).unwrap();

    let x0 = vec![0.0f64; dim];
    let mut v0 = vec![0.0f64; dim];
    v0[axis] = u0;

    let trajectory =
        geodesic_integrator_kernel(&x0, &v0, &christoffel, horizon / steps as f64, steps).unwrap();

    let (x, u) = &trajectory[steps];
    // The connection drives only `axis`, so the other component must not move.
    assert!(x[other].abs() < 1e-14, "x^{other} drifted to {}", x[other]);
    assert!(u[other].abs() < 1e-14, "u^{other} drifted to {}", u[other]);
    (x[axis], u[axis])
}

fn constant_connection_run(c: f64, u0: f64, horizon: f64, steps: usize) -> (f64, f64) {
    constant_connection_run_on(0, c, u0, horizon, steps)
}

#[test]
fn test_geodesic_integrator_matches_the_closed_form_for_a_constant_connection() {
    // c = 1, u0 = 1, tau = 1: u -> 1/2 and x -> ln 2. A sign error on the connection term sends
    // u to 1/(1 - tau), which diverges at tau = 1 rather than halving.
    let (x, u) = constant_connection_run(1.0, 1.0, 1.0, 200);
    let (x_exact, u_exact) = constant_connection_exact(1.0, 1.0, 1.0);

    assert!((u - 0.5).abs() < 1e-9, "u = {u}, expected 1/2");
    assert!(
        (x - std::f64::consts::LN_2).abs() < 1e-9,
        "x = {x}, expected ln 2"
    );
    assert!((u - u_exact).abs() < 1e-9 && (x - x_exact).abs() < 1e-9);
}

#[test]
fn test_geodesic_integrator_is_fourth_order_in_the_step() {
    // RK4: halving the step must cut the error by about 2^4. This pins the integrator's order
    // independently of any single value, and a wrong stage coefficient shows up here even when
    // one evaluation looks plausible.
    let (c, u0, horizon) = (1.0_f64, 1.0_f64, 1.0_f64);
    let (x_exact, _) = constant_connection_exact(c, u0, horizon);

    let err = |steps: usize| (constant_connection_run(c, u0, horizon, steps).0 - x_exact).abs();
    let coarse = err(8);
    let fine = err(16);

    let ratio = coarse / fine;
    assert!(
        (10.0..=22.0).contains(&ratio),
        "observed order ratio {ratio} (coarse {coarse:e}, fine {fine:e}); RK4 must be near 16"
    );
}

#[test]
fn test_geodesic_integrator_closed_form_holds_on_every_axis() {
    // The same analytic solution, driven on axis 1 rather than axis 0. On axis 0 the flat index
    // `mu * dim * dim + nu * dim + rho` is 0 whatever the strides are, so that case alone cannot
    // tell a correct stride from a mutated one.
    for axis in 0..2 {
        let (x, u) = constant_connection_run_on(axis, 1.0, 1.0, 1.0, 200);
        assert!((u - 0.5).abs() < 1e-9, "axis {axis}: u = {u}, expected 1/2");
        assert!(
            (x - std::f64::consts::LN_2).abs() < 1e-9,
            "axis {axis}: x = {x}, expected ln 2"
        );
    }
}

#[test]
fn test_geodesic_integrator_rejects_a_non_cubic_connection() {
    // The shape guard tests all three extents. Each needs its own case, or a guard that checks
    // only one of them passes for the other two.
    let position = [0.0f64, 0.0];
    let velocity = [1.0f64, 0.0];
    for shape in [vec![3, 2, 2], vec![2, 3, 2], vec![2, 2, 3]] {
        let len: usize = shape.iter().product();
        let christoffel = CausalTensor::new(vec![0.0f64; len], shape.clone()).unwrap();
        let result = geodesic_integrator_kernel(&position, &velocity, &christoffel, 0.1, 1);
        assert!(
            result.is_err(),
            "a connection of shape {shape:?} must be refused for a 2D state"
        );
    }
}

#[test]
fn test_geodesic_integrator_reports_divergence_when_only_the_position_overflows() {
    // The instability guard checks the position and the velocity. With a zero connection the
    // velocity stays finite for ever, so a position that overflows on its own is the only way to
    // reach the position half of that check.
    let christoffel = CausalTensor::new(vec![0.0f64; 8], vec![2, 2, 2]).unwrap();
    let result = geodesic_integrator_kernel(&[0.0, 0.0], &[1.0e300, 0.0], &christoffel, 1.0e10, 20);
    assert!(
        result.is_err(),
        "x = u * t overflows while u stays finite, and that must still be reported"
    );
}

#[test]
fn test_geodesic_integrator_kernel_dimension_mismatch() {
    let initial_position: Vec<f64> = vec![0.0, 0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0]; // Wrong dimension

    let christoffel = CausalTensor::new(vec![0.0f64; 27], vec![3, 3, 3]).unwrap();

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_err());
}

#[test]
fn test_geodesic_integrator_kernel_wrong_christoffel_rank() {
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0];

    let christoffel = CausalTensor::new(vec![0.0f64; 4], vec![2, 2]).unwrap(); // Rank 2, not 3

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_err());
}

#[test]
fn test_geodesic_integrator_kernel_invalid_step() {
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0];
    let christoffel = CausalTensor::new(vec![0.0f64; 8], vec![2, 2, 2]).unwrap();

    // Zero step size should error
    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.0, 10);
    assert!(result.is_err());

    // NaN step size should error
    let result = geodesic_integrator_kernel(
        &initial_position,
        &initial_velocity,
        &christoffel,
        f64::NAN,
        10,
    );
    assert!(result.is_err());
}

#[test]
fn test_geodesic_integrator_kernel_divergence_error() {
    // A huge Christoffel coupling with a large velocity and step size makes the
    // RK4 acceleration grow without bound, overflowing to a non-finite value and
    // exercising the "geodesic integration diverged" instability branch.
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1e150, 1e150];

    // Γ[0,0,0] and Γ[1,1,1] enormous → a^mu ∝ -Γ u u explodes.
    let mut gamma = vec![0.0f64; 8]; // [2,2,2]
    gamma[0] = 1e150; // Γ^0_00
    gamma[7] = 1e150; // Γ^1_11
    let christoffel = CausalTensor::new(gamma, vec![2, 2, 2]).unwrap();

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 1e10, 20);
    assert!(
        result.is_err(),
        "Diverging RK4 integration must yield a NumericalInstability error"
    );
}

#[test]
fn test_geodesic_deviation_rejects_a_rank4_tensor_whose_extents_are_not_four() {
    // The kernel fixes `dim = 4` and validates only the Riemann tensor's *rank*, so a rank-4
    // tensor with smaller extents reaches the contraction and is indexed far past its length.
    let riemann = CausalTensor::new(vec![0.0f64; 16], vec![2, 2, 2, 2]).unwrap();
    let u: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let n: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let result = geodesic_deviation_kernel(&riemann, &u, &n);
    assert!(
        result.is_err(),
        "a [2,2,2,2] Riemann tensor must be refused, not indexed as if it were [4,4,4,4]"
    );
}
