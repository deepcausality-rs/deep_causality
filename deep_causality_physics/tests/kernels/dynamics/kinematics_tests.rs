/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Frequency, Mass, MomentOfInertia, PhysicalVector, PhysicsErrorEnum, angular_momentum_kernel,
    kinetic_energy_kernel, rotational_kinetic_energy_kernel, torque_kernel,
};

// =============================================================================
// kinetic_energy_kernel Tests
// =============================================================================

#[test]
fn test_kinetic_energy_kernel_valid() {
    // KE = 0.5 * m * v^2
    let mass = Mass::<f64>::new(2.0).unwrap();
    // Create a 3D velocity vector [3, 4, 0] with magnitude 5
    let velocity = CausalMultiVector::new(
        vec![0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    // v^2 = 3^2 + 4^2 = 25
    // KE = 0.5 * 2 * 25 = 25
    assert!((ke - 25.0).abs() < 1e-10, "Expected KE = 25, got {}", ke);
}

#[test]
fn test_kinetic_energy_kernel_zero_velocity() {
    let mass = Mass::<f64>::new(10.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!(
        (ke - 0.0).abs() < 1e-10,
        "Zero velocity should give zero KE"
    );
}

#[test]
fn test_kinetic_energy_kernel_zero_mass() {
    let mass = Mass::<f64>::new(0.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!((ke - 0.0).abs() < 1e-10, "Zero mass should give zero KE");
}

// =============================================================================
// rotational_kinetic_energy_kernel Tests
// =============================================================================

#[test]
fn test_rotational_kinetic_energy_kernel_valid() {
    // KE_rot = 0.5 * I * omega^2
    let inertia = MomentOfInertia::<f64>::new(4.0).unwrap();
    let omega = Frequency::<f64>::new(3.0).unwrap();

    let result = rotational_kinetic_energy_kernel(inertia, omega);
    assert!(result.is_ok());

    let ke = result.unwrap();
    // KE = 0.5 * 4 * 9 = 18
    assert!((ke - 18.0).abs() < 1e-10, "Expected KE = 18, got {}", ke);
}

#[test]
fn test_rotational_kinetic_energy_kernel_zero_omega() {
    let inertia = MomentOfInertia::<f64>::new(10.0).unwrap();
    let omega = Frequency::<f64>::new(0.0).unwrap();

    let result = rotational_kinetic_energy_kernel(inertia, omega);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!((ke - 0.0).abs() < 1e-10);
}

// =============================================================================
// torque_kernel Tests
// =============================================================================

#[test]
fn test_torque_kernel_valid() {
    // Torque = r × F (outer product in GA)
    // r = [0, 1, 0, 0] (x-direction)
    // F = [0, 0, 1, 0] (y-direction)
    // r ^ F should give bivector in xy-plane
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let torque: PhysicalVector<f64> = torque_kernel(&radius, &force).unwrap();

    // tau = r ^ F = e1 ^ e2 = e12. Blades are bitmask-indexed, so e12 sits at 3.
    // `!data().is_empty()` cannot see this: a Cl(3) multivector always has eight components.
    let d: &[f64] = torque.inner().data();
    assert!((d[3] - 1.0).abs() < 1e-12, "e12 component = {}", d[3]);
    for (i, v) in d.iter().enumerate() {
        if i != 3 {
            assert!(v.abs() < 1e-12, "blade {i} should vanish, got {v}");
        }
    }
}

#[test]
fn test_torque_is_antisymmetric_in_radius_and_force() {
    // r ^ F = -(F ^ r), and r ^ r = 0. Both identities hold for any input and pin the outer
    // product against the geometric product, whose scalar part would not vanish.
    let r = CausalMultiVector::new(
        vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let f = CausalMultiVector::new(
        vec![0.0, -0.5, 1.5, 0.0, 0.25, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let forward: PhysicalVector<f64> = torque_kernel(&r, &f).unwrap();
    let reversed: PhysicalVector<f64> = torque_kernel(&f, &r).unwrap();
    let a: &[f64] = forward.inner().data();
    let b: &[f64] = reversed.inner().data();
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        assert!((x + y).abs() < 1e-12, "blade {i}: {x} and {y}");
    }

    let self_wedge: PhysicalVector<f64> = torque_kernel(&r, &r).unwrap();
    let z: &[f64] = self_wedge.inner().data();
    for (i, v) in z.iter().enumerate() {
        assert!(v.abs() < 1e-12, "r ^ r blade {i} = {v}");
    }
}

#[test]
fn test_torque_kernel_parallel_vectors() {
    // Parallel vectors have zero cross product
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = torque_kernel(&radius, &force);
    assert!(result.is_ok());
    // Result should be zero bivector
}

// =============================================================================
// angular_momentum_kernel Tests
// =============================================================================

#[test]
fn test_angular_momentum_kernel_valid() {
    // L = r × p (outer product)
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let momentum = CausalMultiVector::new(
        vec![0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let l: PhysicalVector<f64> = angular_momentum_kernel(&radius, &momentum).unwrap();

    // L = r ^ p = e1 ^ (5 e2) = 5 e12, so the e12 blade at index 3 carries 5 and the rest vanish.
    let d: &[f64] = l.inner().data();
    assert!((d[3] - 5.0).abs() < 1e-12, "e12 component = {}", d[3]);
    for (i, v) in d.iter().enumerate() {
        if i != 3 {
            assert!(v.abs() < 1e-12, "blade {i} should vanish, got {v}");
        }
    }
}

#[test]
fn test_angular_momentum_is_linear_in_the_momentum() {
    // L(r, k p) = k L(r, p) for any k, and needs no oracle.
    let r = CausalMultiVector::new(
        vec![0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let p_base = [0.0, 0.5, -1.5, 0.0, 2.0, 0.0, 0.0, 0.0];
    let momentum = CausalMultiVector::new(p_base.to_vec(), Metric::Euclidean(3)).unwrap();
    let base: PhysicalVector<f64> = angular_momentum_kernel(&r, &momentum).unwrap();

    for k in [0.5_f64, 2.0, -3.0] {
        let scaled_p: Vec<f64> = p_base.iter().map(|x| x * k).collect();
        let scaled_m = CausalMultiVector::new(scaled_p, Metric::Euclidean(3)).unwrap();
        let scaled: PhysicalVector<f64> = angular_momentum_kernel(&r, &scaled_m).unwrap();
        let a: &[f64] = base.inner().data();
        let b: &[f64] = scaled.inner().data();
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            assert!((y - k * x).abs() < 1e-12, "k = {k}, blade {i}");
        }
    }
}

// =============================================================================
// PhysicalVector Tests
// =============================================================================

#[test]
fn test_physical_vector_default() {
    let pv = PhysicalVector::<f64>::default();
    // Default should be a scalar 0 multivector
    assert!((pv.inner().data()[0] - 0.0).abs() < 1e-10);
}

#[test]
fn test_physical_vector_new_and_accessors() {
    let mv = CausalMultiVector::new(
        vec![1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let pv = PhysicalVector::new(mv.clone());

    assert_eq!(pv.inner().data(), mv.data());

    let inner = pv.into_inner();
    assert_eq!(inner.data(), mv.data());
}

// =============================================================================
// kinetic_energy_kernel error branches (kinematics.rs:55-57, 62-64)
// =============================================================================

#[test]
fn test_kinetic_energy_kernel_non_finite_velocity() {
    // A velocity component of +∞ makes the squared magnitude non-finite,
    // hitting the `!v_sq.is_finite()` branch (kinematics.rs:55-57).
    let mass = Mass::<f64>::new(2.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
    match result.unwrap_err().0 {
        deep_causality_physics::PhysicsErrorEnum::NumericalInstability(_) => {}
        e => panic!("Expected NumericalInstability, got {e:?}"),
    }
}

#[test]
fn test_kinetic_energy_kernel_negative_squared_speed() {
    // Under a Minkowski (+ - - -) metric a purely spacelike vector has a
    // strictly negative squared magnitude, hitting the negative-squared-speed
    // branch (kinematics.rs:62-64).
    let mass = Mass::<f64>::new(2.0).unwrap();
    // Cl(1,3): 16 basis blades; place a unit value on a spacelike grade-1 axis.
    let mut data = vec![0.0_f64; 16];
    data[2] = 1.0; // spacelike basis vector e1
    let velocity = CausalMultiVector::new(data, Metric::Minkowski(4)).unwrap();

    // Only proceed if this metric does yield a negative squared magnitude.
    let v_sq = {
        use deep_causality_multivector::MultiVector;
        velocity.squared_magnitude()
    };
    assert!(
        v_sq < 0.0,
        "expected negative squared magnitude, got {v_sq}"
    );

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    match result.unwrap_err().0 {
        deep_causality_physics::PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        e => panic!("Expected PhysicalInvariantBroken, got {e:?}"),
    }
}
