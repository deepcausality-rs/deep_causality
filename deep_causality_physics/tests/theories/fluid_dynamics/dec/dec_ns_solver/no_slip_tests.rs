/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the no-slip wall constraint stage (the no-slip-viscous
//! capability of add-walls-and-dec-stencils): wall-tangential edges are
//! exactly zero at seeding and at every step boundary, the constraint
//! touches *only* the tangential set, and the fully periodic path is
//! bit-identical to the unconstrained pipeline.

use deep_causality_physics::DecNsSolver;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn manifold_2d(shape: [usize; 2], periodic: [bool; 2]) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// Replicates the no-slip edge enumeration through the public lattice API:
/// an edge along `axis` is wall-tangential iff it lies on a boundary
/// perpendicular to some other non-periodic axis.
fn wall_tangential_edges<const D: usize>(complex: &LatticeComplex<D, f64>) -> Vec<usize> {
    let periodic = complex.periodic();
    let shape = complex.shape();
    complex
        .iter_cells(1)
        .enumerate()
        .filter_map(|(i, c)| {
            let axis = c.orientation().trailing_zeros() as usize;
            let pos = c.position();
            (0..D)
                .any(|w| w != axis && !periodic[w] && (pos[w] == 0 || pos[w] + 1 == shape[w]))
                .then_some(i)
        })
        .collect()
}

/// A smooth seeding field with nonzero tangential velocity at the walls
/// (uniform x-flow plus a gentle shear), so the constraint has real work.
fn channel_vertex_tensor(manifold: &Manifold<LatticeComplex<2, f64>, f64>) -> CausalTensor<f64> {
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let y = v.position()[1] as f64;
        vertex[2 * vi] = 1.0 + 0.1 * y;
        vertex[2 * vi + 1] = 0.0;
    }
    CausalTensor::new(vertex, vec![2 * n0]).unwrap()
}

fn channel_solver(manifold: &Manifold<LatticeComplex<2, f64>, f64>) -> DecNsSolver<'_, 2, f64> {
    DecNsSolver::new(manifold, 0.01, 0.05, None).unwrap()
}

#[test]
fn seeding_pins_wall_tangential_edges_to_zero() {
    // Periodic-x, wall-y channel: x-edges on the two y-walls are
    // tangential and must come out of seeding exactly zero.
    let m = manifold_2d([8, 6], [true, false]);
    let solver = channel_solver(&m);
    let state = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();

    let tangential = wall_tangential_edges(m.complex());
    assert!(!tangential.is_empty(), "fixture must have wall edges");
    let u = state.as_one_form().as_slice();
    for &e in &tangential {
        assert_eq!(u[e], 0.0, "tangential edge {e} not pinned at seeding");
    }
    // The constraint had real work: the unconstrained projection of this
    // field is nonzero on the tangential set (uniform x-flow at the wall).
    let raw = m.de_rham(&channel_vertex_tensor(&m)).unwrap();
    let unconstrained = m.leray_project(&raw).unwrap();
    let p = unconstrained.projected().as_slice();
    assert!(
        tangential.iter().any(|&e| p[e].abs() > 1e-3),
        "fixture's unconstrained projection should be nonzero at walls"
    );
}

#[test]
fn tangential_edges_are_exactly_zero_at_every_step_boundary() {
    // The spec scenario: march several steps; every step boundary holds
    // the constraint exactly, and divergence stays at solve exactness.
    let m = manifold_2d([8, 6], [true, false]);
    let solver = channel_solver(&m);
    let mut state = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();
    let tangential = wall_tangential_edges(m.complex());

    for step in 0..3 {
        let out = solver.step(&state).unwrap();
        state = out.into_state();
        let u = state.as_one_form().as_slice();
        for &e in &tangential {
            assert_eq!(u[e], 0.0, "tangential edge {e} nonzero after step {step}");
        }
    }
}

#[test]
fn all_walls_box_marches_with_pinned_boundary_edges() {
    // Both axes walled: every boundary-resident edge parallel to a wall is
    // constrained; the march completes and the output divergence stays
    // near the solve's exactness.
    let m = manifold_2d([7, 6], [false, false]);
    let solver = channel_solver(&m);
    let state = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();
    let tangential = wall_tangential_edges(m.complex());
    assert!(!tangential.is_empty());

    let out = solver.step(&state).unwrap();
    let u = out.state().as_one_form().as_slice();
    for &e in &tangential {
        assert_eq!(u[e], 0.0, "tangential edge {e} not pinned");
    }
    assert!(
        out.divergence_residual() < 1e-10,
        "divergence residual {} above solve exactness",
        out.divergence_residual()
    );
}

#[test]
fn divergence_free_seed_passes_through_off_the_walls() {
    // The x-uniform shear fixture is discretely divergence-free, so the
    // constrained projection's potential is zero in exact arithmetic: the
    // free edges pass through untouched and the tangential set is zeroed.
    // (For a general field the constrained projector legitimately adjusts
    // free edges too — that property is pinned by the topology-level
    // constrained-projection tests.)
    let m = manifold_2d([8, 6], [true, false]);
    let solver = channel_solver(&m);
    let seeded = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();

    let raw = m.de_rham(&channel_vertex_tensor(&m)).unwrap();
    let unconstrained = m.leray_project(&raw).unwrap();
    let p = unconstrained.projected().as_slice();
    let s = seeded.as_one_form().as_slice();
    let tangential = wall_tangential_edges(m.complex());

    for (e, (a, b)) in s.iter().zip(p.iter()).enumerate() {
        if tangential.contains(&e) {
            assert_eq!(*a, 0.0);
        } else {
            assert_eq!(*a, *b, "off-set edge {e} modified by the constraint");
        }
    }
}

#[test]
fn periodic_seed_is_bit_identical_to_the_unconstrained_projection() {
    // Fully periodic: the constrained-edge set is empty and seeding is
    // bit-for-bit the plain de Rham + Leray pipeline.
    let m = manifold_2d([8, 8], [true, true]);
    assert!(wall_tangential_edges(m.complex()).is_empty());

    let solver = channel_solver(&m);
    let seeded = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();
    let raw = m.de_rham(&channel_vertex_tensor(&m)).unwrap();
    let unconstrained = m.leray_project(&raw).unwrap();

    assert_eq!(
        seeded.as_one_form().as_slice(),
        unconstrained.projected().as_slice(),
        "periodic seeding must be bit-identical to the unconstrained path"
    );
}

/// The constrained viscous operator `P_S Δ₁ P_S` must be symmetric in the
/// corrected M₁ inner product on walled lattices (the no-slip-viscous
/// spec's boundary-row scenario): assemble `M₁ · P_S Δ₁ P_S` column by
/// column and compare transposed entries.
fn assert_constrained_viscous_m_symmetric<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    tol: f64,
) {
    use deep_causality_topology::HasHodgeStar;
    let complex = manifold.complex();
    let n1 = complex.num_cells(1);
    let tangential = wall_tangential_edges(complex);
    assert!(!tangential.is_empty(), "fixture must have wall edges");

    let metric_binding = manifold.metric();
    let metric = metric_binding.as_ref().unwrap();
    let star = metric.hodge_star_matrix(complex, 1).unwrap();
    let mut mass = vec![0.0; n1];
    for (i, m) in mass.iter_mut().enumerate() {
        for e in star.row_indices()[i]..star.row_indices()[i + 1] {
            if star.col_indices()[e] == i {
                *m = star.values()[e];
            }
        }
    }

    let mut columns: Vec<Vec<f64>> = Vec::with_capacity(n1);
    for j in 0..n1 {
        let mut unit = vec![0.0; n1];
        // P_S on the input: a constrained column is identically zero.
        if tangential.contains(&j) {
            columns.push(unit);
            continue;
        }
        unit[j] = 1.0;
        let mut col = manifold.laplacian_of(&unit, 1).into_vec();
        col.resize(n1, 0.0);
        // P_S on the output, then the M₁ row weights.
        for &t in &tangential {
            col[t] = 0.0;
        }
        for (c, m) in col.iter_mut().zip(mass.iter()) {
            *c *= *m;
        }
        columns.push(col);
    }
    for (i, col_i) in columns.iter().enumerate() {
        for (j, col_j) in columns.iter().enumerate().take(i) {
            let a = col_j[i];
            let b = col_i[j];
            assert!(
                (a - b).abs() < tol,
                "M·P_S·Δ₁·P_S asymmetric at ({i},{j}): {a} vs {b}"
            );
        }
    }
}

#[test]
fn constrained_viscous_operator_m_symmetric_walled_2d() {
    let m = manifold_2d([4, 4], [false, false]);
    assert_constrained_viscous_m_symmetric(&m, 1e-13);
}

#[test]
fn constrained_viscous_operator_m_symmetric_mixed_3d() {
    let lattice = LatticeComplex::<3, f64>::new([3, 3, 3], [true, false, false]);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(0.5);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    assert_constrained_viscous_m_symmetric(&m, 1e-12);
}

/// March a Couette channel (periodic-x, wall-y, lid at y-max moving with
/// `U = 1`) from rest to steady state; return the sup error of the x-edge
/// profile against the exact linear solution `u_x(y) = U·y/H`.
fn couette_steady_profile_error(ny: usize) -> f64 {
    let m = manifold_2d([4, ny], [true, false]);
    let nu = 0.5;
    let dt = 0.2;
    let solver = DecNsSolver::new(&m, nu, dt, None)
        .unwrap()
        .with_moving_wall(1, true, [1.0, 0.0])
        .unwrap();

    // Start from rest: the seed is the lift alone.
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    // March to steady state (diffusion time H²/ν ≈ 50 ⇒ a few hundred
    // steps), with an early exit on stationarity.
    let mut previous = state.as_one_form().as_slice().to_vec();
    for _ in 0..4000 {
        state = solver.step(&state).unwrap().into_state();
        let now = state.as_one_form().as_slice();
        let delta = now
            .iter()
            .zip(previous.iter())
            .fold(0.0f64, |acc, (a, b)| acc.max((a - b).abs()));
        if delta < 1e-13 {
            break;
        }
        previous = now.to_vec();
    }

    // Compare every x-edge against the linear profile.
    let h = (ny - 1) as f64;
    let u = state.as_one_form().as_slice();
    let mut err = 0.0f64;
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize != 0 {
            continue;
        }
        let y = cell.position()[1] as f64;
        err = err.max((u[idx] - y / h).abs());
    }
    err
}

#[test]
fn couette_diffusion_relaxes_to_the_linear_profile() {
    // The Couette scenario, and stronger than the spec asks: the linear
    // shear lies in the kernel of the discrete viscous operator, and the
    // convective residual of an x-uniform shear is a discrete gradient the
    // constrained projector removes — so the march lands on the exact
    // linear profile at stationarity rounding (measured 2.5e-12 / 1.0e-11),
    // not merely at the discretization's order.
    for ny in [6usize, 11] {
        let err = couette_steady_profile_error(ny);
        assert!(
            err < 1e-9,
            "ny = {ny}: Couette profile error {err:e} above rounding"
        );
    }
}

#[test]
fn moving_wall_rejects_invalid_configurations() {
    let m = manifold_2d([8, 6], [true, false]);
    // Periodic axis: there is no wall to move.
    let err = DecNsSolver::new(&m, 0.01, 0.05, None)
        .unwrap()
        .with_moving_wall(0, false, [0.0, 1.0])
        .unwrap_err();
    assert!(format!("{err}").contains("periodic"));
    // Wall-normal velocity component.
    let err = DecNsSolver::new(&m, 0.01, 0.05, None)
        .unwrap()
        .with_moving_wall(1, true, [1.0, 0.5])
        .unwrap_err();
    assert!(format!("{err}").contains("wall-normal"));
    // Axis out of range.
    let err = DecNsSolver::new(&m, 0.01, 0.05, None)
        .unwrap()
        .with_moving_wall(7, true, [1.0, 0.0])
        .unwrap_err();
    assert!(format!("{err}").contains("out of range"));
    // Non-finite velocity.
    let err = DecNsSolver::new(&m, 0.01, 0.05, None)
        .unwrap()
        .with_moving_wall(1, true, [f64::NAN, 0.0])
        .unwrap_err();
    assert!(format!("{err}").contains("finite"));
}

#[test]
fn moving_wall_lift_is_held_exactly_at_step_boundaries() {
    let m = manifold_2d([6, 5], [true, false]);
    let solver = DecNsSolver::new(&m, 0.2, 0.1, None)
        .unwrap()
        .with_moving_wall(1, true, [1.0, 0.0])
        .unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    let shape = m.complex().shape();
    let lid_y = shape[1] - 1;
    for _ in 0..3 {
        state = solver.step(&state).unwrap().into_state();
        let u = state.as_one_form().as_slice();
        for (idx, cell) in m.complex().iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            let y = cell.position()[1];
            if axis == 0 && y == lid_y {
                assert_eq!(u[idx], 1.0, "lid edge {idx} lost its lift");
            } else if axis == 0 && y == 0 {
                assert_eq!(u[idx], 0.0, "bottom wall edge {idx} not pinned");
            }
        }
    }
}

/// The compiled stencil assembly (the default) and the generic
/// compositional assembly must agree through the **full constrained
/// projected rate** on a wall-bounded lattice — the wall march is
/// evaluation-strategy-agnostic (C1.4: the tables compile from the
/// corrected star; the constraint lives in the shared projector).
#[test]
fn fused_and_generic_projected_rates_agree_on_walls() {
    use deep_causality_physics::{DecNsRate, VelocityOneForm};
    use deep_causality_topology::HodgeDecomposeOptions;

    let m = manifold_2d([8, 6], [true, false]);
    let solver = channel_solver(&m);
    let state = solver
        .seed_from_vertex_vectors(&channel_vertex_tensor(&m))
        .unwrap();
    let u = VelocityOneForm::new(state.as_one_form().clone(), &m).unwrap();

    let fused = DecNsRate::new(&m, 0.01, None).unwrap();
    let generic = DecNsRate::new(&m, 0.01, None)
        .unwrap()
        .with_generic_assembly();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-13),
        max_iterations: Some(10_000),
    };

    let a = fused.eval_projected(&u, &opts).unwrap();
    let b = generic.eval_projected(&u, &opts).unwrap();
    for (i, (x, y)) in a
        .as_tensor()
        .as_slice()
        .iter()
        .zip(b.as_tensor().as_slice().iter())
        .enumerate()
    {
        assert!((x - y).abs() <= 1e-10, "edge {i}: fused {x} vs generic {y}");
    }

    // Both strategies produce the exact zeros on the tangential set.
    let tangential = wall_tangential_edges(m.complex());
    for &e in &tangential {
        assert_eq!(a.as_tensor().as_slice()[e], 0.0);
        assert_eq!(b.as_tensor().as_slice()[e], 0.0);
    }
}

#[test]
fn integral_seeding_is_constrained_too() {
    // The second seeding entry point routes through the same projection +
    // constraint; tangential edges come out exactly zero.
    let m = manifold_2d([8, 6], [true, false]);
    let solver = channel_solver(&m);
    let n1 = m.complex().num_cells(1);
    let integrals = CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap();
    let state = solver.seed_from_edge_integrals(&integrals).unwrap();
    let u = state.as_one_form().as_slice();
    for &e in &wall_tangential_edges(m.complex()) {
        assert_eq!(u[e], 0.0);
    }
}
