/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction-validation and stage-hook tests for the boundary zones (`MovingWall`, `Inflow`,
//! `Outflow`, `SlipWall`, `BodyForceZone`) and their static composition. The marching cases drive
//! the happy path through the solver; the `new` rejections, the periodic / metric-free / min-side
//! branches of the `collect_*` hooks, and the cons-tuple composition are covered here.

use deep_causality_cfd::{BodyForceZone, BoundaryZone, Inflow, MovingWall, Outflow, SlipWall};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const N: usize = 5;

fn wall_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn periodic_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn metric_free_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical(lattice, data, 0)
}

// ---------------------------------------------------------------------------
// Constructor validation
// ---------------------------------------------------------------------------

#[test]
fn moving_wall_new_validates_axis_finiteness_and_normal_component() {
    assert!(MovingWall::<2, f64>::new(1, true, [0.5, 0.0]).is_ok());

    let err = MovingWall::<2, f64>::new(2, true, [0.0, 0.0]).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));

    let err = MovingWall::<2, f64>::new(1, true, [f64::NAN, 0.0]).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::NumericalInstability(_)));

    // A non-zero wall-normal component (axis 1 here) is rejected.
    let err = MovingWall::<2, f64>::new(1, true, [0.0, 0.3]).unwrap_err();
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn inflow_new_validates_axis_and_finiteness() {
    assert!(Inflow::<2, f64>::new(0, true, 1.0).is_ok());

    let err = Inflow::<2, f64>::new(2, true, 1.0).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));

    let err = Inflow::<2, f64>::new(0, true, f64::INFINITY).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::NumericalInstability(_)));
}

#[test]
fn outflow_and_slip_wall_new_validate_axis() {
    assert!(Outflow::<2>::new(0, true).is_ok());
    let err = Outflow::<2>::new(2, true).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));

    assert!(SlipWall::<2>::new(1, false).is_ok());
    let err = SlipWall::<2>::new(9, false).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn body_force_zone_exposes_its_cochain() {
    let force = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let zone = BodyForceZone::new(force);
    assert_eq!(zone.force().as_slice(), &[1.0, 2.0, 3.0]);
}

// ---------------------------------------------------------------------------
// Stage hooks: periodic / metric-free / min-side branches
// ---------------------------------------------------------------------------

#[test]
fn moving_wall_lift_handles_min_side_periodic_and_metric_free() {
    // Min side (the zero face): the lift is non-empty on a walled, metric-bearing domain.
    let wall = MovingWall::<2, f64>::new(1, false, [0.7, 0.0]).unwrap();
    let m = wall_manifold();
    let mut out = Vec::new();
    wall.collect_lift(&m, 0, &mut out);
    assert!(
        !out.is_empty(),
        "a moving wall contributes a tangential lift"
    );

    // A periodic wall axis has no wall to move.
    let pm = periodic_manifold();
    let mut out_periodic = Vec::new();
    wall.collect_lift(&pm, 0, &mut out_periodic);
    assert!(out_periodic.is_empty());

    // Without a metric there are no edge lengths to integrate.
    let mf = metric_free_manifold();
    let mut out_no_metric = Vec::new();
    wall.collect_lift(&mf, 0, &mut out_no_metric);
    assert!(out_no_metric.is_empty());
}

#[test]
fn inflow_lift_and_prescribed_edges_handle_periodic_and_metric_free() {
    let inflow = Inflow::<2, f64>::new(0, false, 1.0).unwrap();
    let m = wall_manifold();
    let mut lift = Vec::new();
    inflow.collect_lift(&m, 0, &mut lift);
    assert!(!lift.is_empty());
    let mut prescribed = Vec::new();
    inflow.collect_prescribed_edges(&m, &mut prescribed);
    assert!(!prescribed.is_empty());

    // Periodic axis: both hooks early-return empty.
    let pm = periodic_manifold();
    let mut lift_p = Vec::new();
    let mut presc_p = Vec::new();
    inflow.collect_lift(&pm, 0, &mut lift_p);
    inflow.collect_prescribed_edges(&pm, &mut presc_p);
    assert!(lift_p.is_empty() && presc_p.is_empty());

    // Metric-free: the lift has no edge lengths.
    let mf = metric_free_manifold();
    let mut lift_mf = Vec::new();
    inflow.collect_lift(&mf, 0, &mut lift_mf);
    assert!(lift_mf.is_empty());
}

#[test]
fn inflow_max_side_collects_the_far_face_column() {
    // A max-side inflow anchors on the far normal-edge column (shape[axis] − 2), driving the
    // `max_side` branch of the private edge_column selector.
    let inflow = Inflow::<2, f64>::new(0, true, 1.0).unwrap();
    let m = wall_manifold();

    let mut lift = Vec::new();
    inflow.collect_lift(&m, 0, &mut lift);
    assert!(!lift.is_empty(), "the max-side face contributes a lift");

    let mut prescribed = Vec::new();
    inflow.collect_prescribed_edges(&m, &mut prescribed);
    assert!(!prescribed.is_empty());

    // The far-face normal edges sit at position column N − 2 on the wall axis.
    let far_col = N - 2;
    for &idx in &prescribed {
        let cell = m.complex().iter_cells(1).nth(idx).unwrap();
        assert_eq!(
            cell.position()[0],
            far_col,
            "max-side prescribed edge {idx} is not on the far column"
        );
    }
}

#[test]
fn outflow_collects_reference_vertices_and_slip_edges() {
    let outflow = Outflow::<2>::new(0, false).unwrap();
    let m = wall_manifold();
    let mut refs = Vec::new();
    BoundaryZone::<2, f64>::collect_reference_vertices(&outflow, &m, &mut refs);
    assert!(!refs.is_empty(), "the outflow face pins reference vertices");
    let mut slip = Vec::new();
    BoundaryZone::<2, f64>::collect_slip_edges(&outflow, &m, &mut slip);
    assert!(!slip.is_empty(), "the outflow frees its tangential edges");
}

#[test]
fn slip_wall_frees_its_tangential_edges() {
    let slip = SlipWall::<2>::new(1, true).unwrap();
    let m = wall_manifold();
    let mut edges = Vec::new();
    BoundaryZone::<2, f64>::collect_slip_edges(&slip, &m, &mut edges);
    assert!(!edges.is_empty());
}

#[test]
fn body_force_zone_adds_to_the_rate_source() {
    let force = CausalTensor::new(vec![0.5, 0.0, 0.25], vec![3]).unwrap();
    let zone = BodyForceZone::new(force);
    let m = wall_manifold();
    let mut acc = vec![1.0, 1.0, 1.0];
    BoundaryZone::<2, f64>::collect_rate_source(&zone, &m, &mut acc);
    assert_eq!(acc, vec![1.5, 1.0, 1.25]);
}

// ---------------------------------------------------------------------------
// Static composition (cons-tuple): exercises the (A, B) folds and the trait defaults
// ---------------------------------------------------------------------------

#[test]
fn composed_zones_fold_every_stage_hook() {
    // A four-zone tuple: each hook folds over all members. Members that do not override a hook
    // use the trait's default no-op, so every `collect_*` stage is exercised.
    let zones = (
        MovingWall::<2, f64>::new(1, true, [0.5, 0.0]).unwrap(),
        (
            Inflow::<2, f64>::new(0, false, 1.0).unwrap(),
            (
                Outflow::<2>::new(0, true).unwrap(),
                SlipWall::<2>::new(1, false).unwrap(),
            ),
        ),
    );
    let m = wall_manifold();

    let mut rate = vec![0.0; m.complex().num_cells(1)];
    zones.collect_rate_source(&m, &mut rate);

    let mut constrained = Vec::new();
    zones.collect_constrained_edges(&m, &mut constrained);

    let mut lift = Vec::new();
    zones.collect_lift(&m, 0, &mut lift);

    let mut prescribed = Vec::new();
    zones.collect_prescribed_edges(&m, &mut prescribed);

    let mut refs = Vec::new();
    zones.collect_reference_vertices(&m, &mut refs);

    let mut slip = Vec::new();
    zones.collect_slip_edges(&m, &mut slip);

    // The composition wires the actuators through: a moving-wall lift, inflow prescribed edges,
    // outflow reference vertices, and freed slip edges are all collected.
    assert!(!lift.is_empty());
    assert!(!prescribed.is_empty());
    assert!(!refs.is_empty());
    assert!(!slip.is_empty());
}

#[test]
fn no_shipped_zone_supplies_constrained_edges() {
    // Why every existing harness is unmoved by wiring `collect_constrained_edges`: the hook is
    // reachable but no shipped zone implements it, so the folded set is empty and
    // `rate_constrained` is byte-identical to before the wiring.
    //
    // This is the cheap decisive form of "no harness result moved" for the DEC path — a marched
    // endpoint from one harness would be weaker evidence and far more expensive. If a zone ever
    // starts supplying constraints, this fails and the harness baselines genuinely need re-running.
    let m = wall_manifold();
    let mut out = Vec::new();

    MovingWall::<2, f64>::new(1, true, [0.5, 0.0])
        .unwrap()
        .collect_constrained_edges(&m, &mut out);
    Inflow::<2, f64>::new(0, false, 1.0)
        .unwrap()
        .collect_constrained_edges(&m, &mut out);
    Outflow::<2>::new(0, true)
        .unwrap()
        .collect_constrained_edges(&m, &mut out);
    SlipWall::<2>::new(1, false)
        .unwrap()
        .collect_constrained_edges(&m, &mut out);
    BoundaryZone::<2, f64>::collect_constrained_edges(
        &BodyForceZone::new(
            CausalTensor::new(
                vec![0.0; m.complex().num_cells(1)],
                vec![m.complex().num_cells(1)],
            )
            .unwrap(),
        ),
        &m,
        &mut out,
    );

    assert!(
        out.is_empty(),
        "a shipped zone now supplies constrained edges ({out:?}); the harness baselines must be \
         re-derived, because the wiring is no longer inert"
    );
}
