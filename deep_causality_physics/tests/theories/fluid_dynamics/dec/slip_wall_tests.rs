/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group S — the free-slip / far-field boundary zone (`add-slip-boundaries-and-surface-forces`).
//!
//! Free-slip = no penetration + zero tangential shear, realized by un-pinning the face's
//! tangential edges from the auto no-slip set. The discriminating gate: a uniform plug flow
//! tangential to the walls is **preserved** under free-slip (no shear, no boundary layer) but
//! **not** under no-slip (the wall edges are pinned to zero). Gated in 2D and 3D.

use deep_causality_physics::{DecNsSolver, SlipWall};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.1;
const U: f64 = 1.0;

fn channel_2d(nx: usize, ny: usize, h: f64) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new([nx, ny], [true, false]); // periodic-x, wall-y.
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// Uniform streamwise (axis 0) vertex vectors `[U, 0]` for seeding a plug flow.
fn uniform_seed_2d(m: &Manifold<LatticeComplex<2, f64>, f64>) -> CausalTensor<f64> {
    let n0 = m.complex().num_cells(0);
    let mut v = vec![0.0; 2 * n0];
    for k in 0..n0 {
        v[2 * k] = U; // x-component.
    }
    CausalTensor::new(v, vec![2 * n0]).unwrap()
}

#[test]
fn free_slip_preserves_a_uniform_plug_flow_2d() {
    let (nx, ny) = (4, 5);
    let h = 1.0 / (ny - 1) as f64;
    let m = channel_2d(nx, ny, h);
    let dt = 0.2 * h * h / (4.0 * NU);

    // Free-slip on both y-walls.
    let slip = DecNsSolver::with_zones(
        &m,
        NU,
        dt,
        (
            SlipWall::<2>::new(1, false).unwrap(),
            SlipWall::<2>::new(1, true).unwrap(),
        ),
    )
    .unwrap();
    let seed = slip.seed_from_vertex_vectors(&uniform_seed_2d(&m)).unwrap();

    // The seed of a uniform plug flow is already uniform under free-slip (no wall is pinned).
    for (i, c) in m.complex().iter_cells(1).enumerate() {
        if c.orientation().trailing_zeros() == 0 {
            assert!(
                (seed.as_one_form().as_slice()[i] - U * h).abs() < 1e-9,
                "free-slip seed x-edge {i} not uniform"
            );
        }
    }

    // No forcing: a uniform plug flow is a steady state under free-slip (zero shear ⇒ zero
    // viscous tendency), so it is preserved.
    let mut state = seed;
    for _ in 0..50 {
        state = slip.step(&state).unwrap().into_state();
    }
    let u = state.as_one_form().as_slice();
    for (i, c) in m.complex().iter_cells(1).enumerate() {
        if c.orientation().trailing_zeros() == 0 {
            assert!(
                (u[i] - U * h).abs() < 1e-8,
                "free-slip did not preserve the plug flow at x-edge {i}: {} vs {}",
                u[i],
                U * h
            );
        } else {
            assert!(u[i].abs() < 1e-8, "transverse leak at y-edge {i}: {}", u[i]);
        }
    }
}

#[test]
fn no_slip_does_not_preserve_the_plug_flow_2d() {
    // The contrast: with the auto no-slip walls (no SlipWall zone), the wall x-edges are pinned to
    // zero, so the uniform plug flow is not preserved — the field differs from free-slip.
    let (nx, ny) = (4, 5);
    let h = 1.0 / (ny - 1) as f64;
    let m = channel_2d(nx, ny, h);
    let dt = 0.2 * h * h / (4.0 * NU);

    let no_slip = DecNsSolver::new(&m, NU, dt, None).unwrap();
    let seed = no_slip
        .seed_from_vertex_vectors(&uniform_seed_2d(&m))
        .unwrap();
    let u = seed.as_one_form().as_slice();

    // No-slip pins the wall-tangential x-edges (y = 0 and y = ny-1) to zero — not a plug flow.
    let shape = m.complex().shape();
    let mut pinned_wall = false;
    for (i, c) in m.complex().iter_cells(1).enumerate() {
        let y = c.position()[1];
        if c.orientation().trailing_zeros() == 0 && (y == 0 || y + 1 == shape[1]) {
            assert_eq!(u[i], 0.0, "no-slip must pin wall x-edge {i} to zero");
            pinned_wall = true;
        }
    }
    assert!(pinned_wall, "the channel must have wall-tangential x-edges");
}

#[test]
fn free_slip_preserves_a_uniform_plug_flow_3d() {
    // periodic-x, periodic-z, free-slip y-walls; a uniform streamwise plug flow is preserved.
    let n = 4;
    let h = 1.0 / (n - 1) as f64;
    let lattice = LatticeComplex::<3, f64>::new([n, n, n], [true, false, true]);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<3, f64>::uniform(h);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let dt = 0.2 * h * h / (6.0 * NU);

    let slip = DecNsSolver::with_zones(
        &m,
        NU,
        dt,
        (
            SlipWall::<3>::new(1, false).unwrap(),
            SlipWall::<3>::new(1, true).unwrap(),
        ),
    )
    .unwrap();
    let n0 = m.complex().num_cells(0);
    let mut v = vec![0.0; 3 * n0];
    for k in 0..n0 {
        v[3 * k] = U; // x-component.
    }
    let seed = slip
        .seed_from_vertex_vectors(&CausalTensor::new(v, vec![3 * n0]).unwrap())
        .unwrap();

    let mut state = seed;
    for _ in 0..30 {
        state = slip.step(&state).unwrap().into_state();
    }
    let u = state.as_one_form().as_slice();
    for (i, c) in m.complex().iter_cells(1).enumerate() {
        if c.orientation().trailing_zeros() == 0 {
            assert!(
                (u[i] - U * h).abs() < 1e-7,
                "3D free-slip did not preserve the plug flow at x-edge {i}: {}",
                u[i]
            );
        }
    }
}
