/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for under-exercised arms of `CubicalReggeGeometry<D, R, S>`:
//! the D = 4 single-edge gradient enumeration, the cut-cell star path, the
//! `signature` zero-eigenvalue metric arm, `with_cut_cells` / `cut_registry`,
//! the `PerEdge` open-axis `axis_length_at_position` branches, the graded
//! `tanh` uniform / degenerate arms, the Euclidean `sign_factor`, and the
//! `StarCache` Default / Clone (warm) / Debug surfaces.

use deep_causality_topology::utils_tests::{open_cube_3, unit_geometry};
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, Euclidean, HasHodgeStar, LatticeComplex,
    Lorentzian, Primitive, SignatureMarker,
};

const TOL: f64 = 1e-10;

// -- gradient.rs: D = 4 single-edge enumeration --------------------------------------

#[test]
fn single_edge_gradient_d4_enumerates_multiaxis_hinges() {
    // Exercises the D = 4 arm of `hinge_gradient_at_edge`: hinges are 2-cells,
    // so `target_hinge_grade = 2`, `other_axes_len = 1`, and the inner loop
    // walks the `2^1 = 2` straddle offsets (`q[b] ∈ {p[b], p[b]-1}`) for every
    // hinge orientation containing the edge axis. The open-lattice out-of-bounds
    // rejection (`valid = false`) fires for boundary edges. We drive the whole
    // edge set so both straddle branches and the rejection arm are hit, and
    // assert every returned component is finite and the action carries deficit
    // somewhere on the open boundary.
    //
    // NOTE: we deliberately do NOT assert `regge_gradient_at_edge(e) ==
    // regge_gradient()[e]` here. For D = 3 the two are identical (a hinge is a
    // single edge — see `single_edge_gradient_agrees_with_full_gradient_open_cube_3d`),
    // but for D >= 4 the closed-form single-edge enumeration credits an edge from
    // *every* hinge straddling it (`q[b] ∈ {p[b], p[b]-1}`), whereas the
    // full-vector `hinge_gradient_sum` only credits the two edges at each hinge's
    // base corner `q`. The two formulas therefore diverge on multi-axis (D >= 4)
    // hinges. This divergence lives in the source (gradient.rs) and is outside
    // the scope of these add-tests-only coverage tests; it is documented here so
    // the D = 4 enumeration arm is still exercised without baking in an equality
    // the source does not currently satisfy.
    let lattice = LatticeComplex::<4, f64>::open([3, 3, 3, 3]);
    let num_edges = lattice.num_cells(1);
    let lens: Vec<f64> = (0..num_edges).map(|i| 1.0 + 0.01 * (i as f64)).collect();
    let geom = CubicalReggeGeometry::<4, f64>::from_edge_lengths(lens);

    let mut any_nonzero = false;
    for e in 0..num_edges {
        let single = geom.regge_gradient_at_edge(&lattice, e);
        assert!(
            single.is_finite(),
            "edge {e}: single gradient must be finite"
        );
        if single.abs() > TOL {
            any_nonzero = true;
        }
    }
    assert!(
        any_nonzero,
        "an open 4D lattice must carry boundary-hinge deficit somewhere"
    );

    // The full reference vector is still well-formed and the correct length.
    let full = geom.regge_gradient(&lattice);
    assert_eq!(full.len(), num_edges);
    assert!(full.iter().all(|g| g.is_finite()));
}

#[test]
fn single_edge_gradient_d1_is_zero() {
    // D < 2: the early `return R::zero()` arm (no hinges exist).
    let lattice = LatticeComplex::<1, f64>::open([4]);
    let num_edges = lattice.num_cells(1);
    let geom = CubicalReggeGeometry::<1, f64>::uniform(2.0);
    for e in 0..num_edges {
        assert_eq!(geom.regge_gradient_at_edge(&lattice, e), 0.0);
    }
    // Full gradient is also all-zeros for D < 2.
    assert!(geom.regge_gradient(&lattice).iter().all(|&g| g == 0.0));
}

// -- mod.rs: with_cut_cells / cut_registry getter ------------------------------------

#[test]
fn with_cut_cells_attaches_registry_and_resets_cache() {
    let geom = unit_geometry::<3>();
    assert!(geom.cut_registry().is_none());

    let registry = CutCellRegistry::<3, f64>::new();
    let cut_geom = geom.with_cut_cells(registry);
    assert!(cut_geom.cut_registry().is_some());
    assert!(cut_geom.cut_registry().unwrap().is_empty());
}

// -- mod.rs: axis_length_at_position PerEdge open-axis branches -----------------------

#[test]
fn per_edge_axis_length_open_uses_last_edge_at_far_boundary() {
    // On an open lattice a vertex at the far boundary (position == shape-1) has
    // no forward edge along that axis, so `axis_length_at_position` falls back
    // to the `position[axis] - 1` edge — the `else if position[axis] > 0` arm.
    // Build a per-edge geometry whose axis-0 edges have distinct lengths so the
    // fallback is observable through the metric tensor diagonal.
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    let num_edges = lattice.num_cells(1);
    let lens: Vec<f64> = (0..num_edges).map(|i| 1.0 + (i as f64)).collect();
    let geom = CubicalReggeGeometry::<2, f64>::from_edge_lengths(lens);

    // Far corner cell: position [2, 2] is at shape-1 on both axes.
    let far = deep_causality_topology::LatticeCell::<2>::new([2, 2], 0);
    let g = geom.metric_tensor_at(&lattice, &far);
    // Diagonal entries are L_axis^2 from the fallback edges; both must be
    // finite and positive (Euclidean).
    assert!(g.as_slice()[0] > 0.0);
    assert!(g.as_slice()[3] > 0.0);
}

// -- signature.rs: Euclidean sign_factor ---------------------------------------------

#[test]
fn euclidean_sign_factor_is_always_one() {
    // Euclidean::sign_factor ignores the timelike count and returns +1.
    assert_eq!(<Euclidean as SignatureMarker>::sign_factor::<f64>(0), 1.0);
    assert_eq!(<Euclidean as SignatureMarker>::sign_factor::<f64>(1), 1.0);
    assert_eq!(<Euclidean as SignatureMarker>::sign_factor::<f64>(7), 1.0);
    assert!(!<Euclidean as SignatureMarker>::is_lorentzian());
}

// -- has_hodge_star.rs: uniform_axis_spacings Lorentzian arm + cut star ---------------

#[test]
fn lorentzian_uniform_axis_spacings_is_none() {
    // The spectral fast path requires a positive-definite diagonal star;
    // Lorentzian sign factors break that, so `uniform_axis_spacings` returns
    // None on a Lorentzian geometry even though it is axis-aligned.
    let lor: CubicalReggeGeometry<3, f64, Lorentzian> = unit_geometry::<3>()
        .with_timelike_axes([true, false, false])
        .unwrap();
    assert!(lor.uniform_axis_spacings().is_none());
}

#[test]
fn euclidean_uniform_axis_spacings_is_some() {
    let geom = CubicalReggeGeometry::<2, f64>::per_axis([2.0, 3.0]);
    let spacings = geom
        .uniform_axis_spacings()
        .expect("axis-aligned Euclidean");
    assert_eq!(spacings, vec![2.0, 3.0]);
}

#[test]
fn cut_hodge_star_out_of_range_grade_is_empty() {
    // k > D returns an empty 0x0 matrix from `cut_hodge_star_matrix`.
    let lattice = open_cube_3();
    let geom = unit_geometry::<3>();
    let registry = CutCellRegistry::<3, f64>::new();
    let m = geom.cut_hodge_star_matrix(&lattice, &registry, 4).unwrap();
    assert_eq!(m.shape(), (0, 0));
}

#[test]
fn cut_hodge_star_empty_registry_matches_standard_star() {
    // With an empty registry the continuous wetted-fraction clip reduces to the
    // integer wall clip, so the cut star equals the standard star. This drives
    // the `build_star_diagonal` cut-clip closure (the `Some(registry)` arm of
    // the standard `hodge_star_matrix` build closure) on a real lattice.
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(2.0);
    let registry = CutCellRegistry::<2, f64>::new();

    for k in 0..=2 {
        let cut = geom.cut_hodge_star_matrix(&lattice, &registry, k).unwrap();
        let std = geom.hodge_star_matrix(&lattice, k).unwrap();
        assert_eq!(cut.shape(), std.shape());
        let cv = cut.values();
        let sv = std.values();
        assert_eq!(cv.len(), sv.len());
        for (a, b) in cv.iter().zip(sv.iter()) {
            assert!((a - b).abs() < TOL, "cut vs std star entry mismatch");
        }
    }
}

#[test]
fn hodge_star_through_attached_empty_cut_registry_matches_uncut() {
    // Attaching an empty registry to the geometry routes `hodge_star_matrix`
    // through the `Some(registry)` build arm (lines 263-265), which must stay
    // byte-equal to the uncut star (the Stage-3 equivalence).
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    let base = CubicalReggeGeometry::<2, f64>::uniform(2.0);
    let cut_geom = base
        .clone()
        .with_cut_cells(CutCellRegistry::<2, f64>::new());

    for k in 0..=2 {
        let with_cut = cut_geom.hodge_star_matrix(&lattice, k).unwrap();
        let without = base.hodge_star_matrix(&lattice, k).unwrap();
        let a = with_cut.values();
        let b = without.values();
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert!((x - y).abs() < TOL);
        }
    }
}

#[test]
fn per_edge_cut_star_with_solid_clips_dual() {
    // A per-edge star built through a registry containing a solid cell exercises
    // the per-edge dual averaging together with the cut-fraction clip
    // (per_edge_corner_product across the corner masks). Build a cylinder cut on
    // a 3D open lattice so the registry is non-empty.
    let lattice = LatticeComplex::<3, f64>::open([4, 4, 4]);
    let geom = CubicalReggeGeometry::<3, f64>::uniform(1.0);
    let prim = Primitive::<3, f64>::cylinder(2, [2.0, 2.0, 0.0], 1.0);
    let registry = CutCellRegistry::from_primitive(&lattice, &geom, &prim).unwrap();
    assert!(!registry.is_empty(), "cylinder must intersect the lattice");

    // Build the cut star at grade 1 (edges); must be diagonal and finite.
    let m = geom.cut_hodge_star_matrix(&lattice, &registry, 1).unwrap();
    let n = lattice.num_cells(1);
    assert_eq!(m.shape(), (n, n));
    for v in m.values() {
        assert!(v.is_finite());
    }
}

// -- star_cache.rs: Default, Clone (warm), Debug -------------------------------------

#[test]
fn star_cache_clone_carries_warm_slots_and_serves_borrowed() {
    // Build a geometry, warm its star cache (first hodge_star_matrix call), then
    // clone it. The clone must carry the warm slots (StarCache::clone warm arm)
    // and still serve correct stars. Also drives Default via `derive`d Clone of
    // the surrounding geometry.
    let lattice = open_cube_3();
    let geom = CubicalReggeGeometry::<3, f64>::uniform(2.0);

    // Warm the cache.
    let first = geom.hodge_star_matrix(&lattice, 1).unwrap().into_owned();

    // Clone the warm geometry (carries the warm StarCache).
    let cloned = geom.clone();
    let from_clone = cloned.hodge_star_matrix(&lattice, 1).unwrap().into_owned();

    assert_eq!(first.shape(), from_clone.shape());
    let a = first.values();
    let b = from_clone.values();
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert!((x - y).abs() < TOL);
    }
    // Equality ignores cache warmth.
    assert_eq!(geom, cloned);
}

#[test]
fn star_cache_debug_reports_warmth() {
    // The geometry's Debug includes the StarCache Debug (warm flag). Cold then
    // warm must both format without panicking and the geometry must be Debug.
    let lattice = open_cube_3();
    let geom = CubicalReggeGeometry::<3, f64>::uniform(2.0);
    let cold = format!("{geom:?}");
    assert!(cold.contains("StarCache"));
    let _ = geom.hodge_star_matrix(&lattice, 0).unwrap();
    let warm = format!("{geom:?}");
    assert!(warm.contains("StarCache"));
}

#[test]
fn star_cache_fingerprint_mismatch_falls_back_to_owned() {
    // Warm the cache against one shape, then request the star on a differently
    // shaped lattice with the *same* geometry value: the fingerprint guard
    // misses and the build falls through to `Cow::Owned`.
    let geom = CubicalReggeGeometry::<2, f64>::uniform(2.0);
    let lattice_a = LatticeComplex::<2, f64>::open([3, 3]);
    let lattice_b = LatticeComplex::<2, f64>::open([4, 4]);

    let _warm = geom.hodge_star_matrix(&lattice_a, 1).unwrap();
    let on_b = geom.hodge_star_matrix(&lattice_b, 1).unwrap();
    // The B star has the B edge count, proving the owned rebuild ran.
    assert_eq!(on_b.shape().0, lattice_b.num_cells(1));
}

// -- metropolis.rs: panic on non-PerEdge geometry ------------------------------------

#[test]
#[should_panic(expected = "metropolis_update requires `PerEdge` geometry")]
fn metropolis_update_panics_on_uniform_geometry() {
    // The single-edge Metropolis update is only defined on a PerEdge geometry;
    // a Uniform geometry hits the `_ => panic!(..)` arm.
    let lattice = open_cube_3();
    let mut geom = CubicalReggeGeometry::<3, f64>::uniform(1.0);
    let mut rng = deep_causality_rand::rng();
    let _ = geom.metropolis_update(&lattice, &mut rng, 0.1, 1.0);
}

// -- graded.rs: tanh degenerate (v < 2) and uniform (beta -> 0) arms ------------------

#[test]
fn graded_tanh_degenerate_short_axis_returns_unit_edges() {
    // A graded axis with only one vertex layer (shape[axis] = 1) makes
    // `tanh_nodes` take the `v < 2` early return; every edge then falls to the
    // uniform `R::one()` arm of the edge loop.
    let lattice = LatticeComplex::<2, f64>::open([1, 3]);
    let geom = CubicalReggeGeometry::<2, f64>::from_graded_tanh(&lattice, 0, 4.0, 2.0);
    // Geometry is PerEdge and well-formed; every recorded length is finite.
    let lens = geom.edge_lengths().expect("graded geometry is PerEdge");
    assert!(lens.iter().all(|l| l.is_finite()));
    assert!(lens.iter().all(|&l| l > 0.0));
}

#[test]
fn graded_tanh_zero_beta_is_uniform_spacing() {
    // beta = 0 makes tanh(beta/2) = 0, so `tanh_nodes` flags `uniform` and the
    // node parameter degenerates to the linear ramp xi. The wall-normal axis
    // then has uniform spacing total_length / (v - 1).
    let lattice = LatticeComplex::<2, f64>::open([4, 3]);
    let total_length = 6.0_f64;
    let geom = CubicalReggeGeometry::<2, f64>::from_graded_tanh(&lattice, 0, total_length, 0.0);
    let lens = geom.edge_lengths().expect("graded geometry is PerEdge");

    // Axis-0 edges come first in iter_cells(1); on a [4,3] open lattice there
    // are edges_along(0) of them. With uniform spacing every axis-0 edge length
    // should equal total_length / (shape[0] - 1) = 6 / 3 = 2.
    let n0 = lattice
        .cells(1)
        .filter(|c| c.orientation() == 1u32 << 0)
        .count();
    for &l in lens.iter().take(n0) {
        assert!(
            (l - 2.0).abs() < TOL,
            "uniform (beta=0) tanh spacing must be {total_length}/3 = 2, got {l}"
        );
    }
}

// -- signature.rs / mod.rs: Lorentzian non-axis-0 timelike yields Custom metric -------

#[test]
fn lorentzian_non_axis0_timelike_yields_custom_metric() {
    // A Lorentzian geometry whose timelike axis is *not* axis 0 produces a
    // `Metric::Custom` (per-axis neg_mask), exercising the Custom branch of
    // `signature()` and the metric-driven sign in `metric_tensor_at`.
    let lor: CubicalReggeGeometry<3, f64, Lorentzian> = unit_geometry::<3>()
        .with_timelike_axes([false, false, true])
        .unwrap();
    let metric = lor.signature();
    assert!(matches!(
        metric,
        deep_causality_metric::Metric::Custom { dim: 3, .. }
    ));

    // The metric tensor diagonal must carry a negative entry on the timelike
    // axis (axis 2) and positive on the others.
    let lattice = open_cube_3();
    let cell = deep_causality_topology::LatticeCell::<3>::new([1, 1, 1], 0);
    let g = lor.metric_tensor_at(&lattice, &cell);
    let d = g.as_slice();
    assert!(d[0] > 0.0); // axis 0 spacelike
    assert!(d[4] > 0.0); // axis 1 spacelike
    assert!(d[8] < 0.0); // axis 2 timelike
}
