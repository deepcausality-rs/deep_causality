/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group Z — the boundary-zone abstraction (`add-boundary-zone-abstraction`).
//!
//! The static zone composition is the canonical surface for the explicit boundary actuators; the
//! numerical-equivalence gate pins that a zone-built solver marches **bit-identically** to the
//! legacy construction it replaces:
//! - a `BodyForceZone` reproduces the `DecNsSolver::new(.., Some(force))` Poiseuille march;
//! - a `MovingWall` zone reproduces the `.with_moving_wall(..)` lid-driven cavity march;
//! - the two compose statically as `(BodyForceZone, MovingWall)`.

use deep_causality_physics::{
    BodyForceOneForm, BodyForceZone, DecNsSolver, MovingWall, SolenoidalField,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.1;

fn channel(ny: usize, periodic: [bool; 2]) -> (Manifold<LatticeComplex<2, f64>, f64>, f64) {
    let h = 1.0 / (ny - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([4, ny], periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    (
        Manifold::from_cubical_with_metric(lattice, data, metric, 0),
        h,
    )
}

/// The Poiseuille streamwise body force as a grade-1 edge cochain (`G·h` on the x-edges).
fn poiseuille_force(m: &Manifold<LatticeComplex<2, f64>, f64>, h: f64) -> CausalTensor<f64> {
    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    CausalTensor::new(force, vec![n1]).unwrap()
}

fn march(solver: &DecNsSolver<'_, 2, f64>, seed: &SolenoidalField<f64>, steps: usize) -> Vec<f64> {
    let mut state = seed.clone();
    for _ in 0..steps {
        state = solver.step(&state).unwrap().into_state();
    }
    state.as_one_form().as_slice().to_vec()
}

#[test]
fn body_force_zone_marches_bit_identically_to_the_legacy_solver() {
    let (m, h) = channel(9, [true, false]);
    let dt = 0.5 * h * h / (4.0 * NU);
    let force_tensor = poiseuille_force(&m, h);

    let force = BodyForceOneForm::new(force_tensor.clone(), &m).unwrap();
    let legacy = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();
    let zoned = DecNsSolver::with_zones(&m, NU, dt, BodyForceZone::new(force_tensor)).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 50);
    let b = march(&zoned, &seed, 50);
    assert_eq!(
        a, b,
        "body-force zone must reproduce the legacy march bit-for-bit"
    );
}

#[test]
fn moving_wall_zone_marches_bit_identically_to_with_moving_wall() {
    let (m, h) = channel(7, [false, false]);
    let dt = 0.5 * h * h / (4.0 * NU); // diffusive-safe at this ν.
    let lid = 1.0;

    let legacy = DecNsSolver::new(&m, NU, dt, None)
        .unwrap()
        .with_moving_wall(1, true, [lid, 0.0])
        .unwrap();
    let zoned =
        DecNsSolver::with_zones(&m, NU, dt, MovingWall::new(1, true, [lid, 0.0]).unwrap()).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    // Seed through the legacy solver (its lift is applied at seed time); the zoned solver carries
    // the identical lift, so seeding it gives the same field — march from the legacy seed for both.
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 40);
    let b = march(&zoned, &seed, 40);
    assert_eq!(
        a, b,
        "moving-wall zone must reproduce with_moving_wall bit-for-bit"
    );
}

#[test]
fn body_force_and_moving_wall_compose_statically() {
    let (m, h) = channel(7, [true, false]);
    let dt = 0.2 * h * h / (4.0 * NU);
    let force_tensor = poiseuille_force(&m, h);
    let lid = 0.5;

    let force = BodyForceOneForm::new(force_tensor.clone(), &m).unwrap();
    let legacy = DecNsSolver::new(&m, NU, dt, Some(&force))
        .unwrap()
        .with_moving_wall(1, true, [lid, 0.0])
        .unwrap();
    let zoned = DecNsSolver::with_zones(
        &m,
        NU,
        dt,
        (
            BodyForceZone::new(force_tensor),
            MovingWall::new(1, true, [lid, 0.0]).unwrap(),
        ),
    )
    .unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 30);
    let b = march(&zoned, &seed, 30);
    assert_eq!(
        a, b,
        "the composed zone set must reproduce the legacy march bit-for-bit"
    );
}
