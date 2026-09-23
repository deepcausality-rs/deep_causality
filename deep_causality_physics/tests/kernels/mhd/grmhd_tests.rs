/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_physics::{
    PhysicsErrorEnum, energy_momentum_tensor_em_kernel, relativistic_current_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialComplex};

#[test]
fn test_relativistic_current_kernel_4d() {
    // 1. Create 5 points in 4D (Pentatope vertices) working with Euclidean metric for distance
    // P0: (0,0,0,0)
    // P1: (1,0,0,0)
    // P2: (0,1,0,0)
    // P3: (0,0,1,0)
    // P4: (0,0,0,1)
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();

    // 2. Create PointCloud and Triangulate
    // Radius > sqrt(2) ~ 1.414 ensures all unit points connect to each other.
    let cloud = PointCloud::new(point_tensor.clone(), CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();

    // Check we have enough structure
    let skeletons = complex.skeletons();
    assert!(
        skeletons.len() >= 4,
        "Need at least 3-simplices (skeletons 0..3)"
    );

    // 3. Create Manifold with Fake EM Data
    // We need data for 0, 1, and 2-simplices.
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();
    let n3 = skeletons[3].simplices().len();
    let total_simplices = complex.total_simplices();

    let mut data = vec![0.0; total_simplices];

    // inject some "field" into 2-simplices (indices n0 + n1 .. n0 + n1 + n2)
    for i in 0..n2 {
        data[n0 + n1 + i] = (i as f64) * 0.1;
    }

    // The codifferential reads the mass matrices a Regge geometry vends, so the manifold must
    // carry one. This was `Manifold::new` while the kernel open-coded its own operator chain.
    let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap());
    let manifold = Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![total_simplices]).unwrap(),
        Some(regge),
        0,
    )
    .unwrap();

    // 4. Metric
    let metric = EastCoastMetric::minkowski_4d();

    // 5. Run Kernel
    let result = relativistic_current_kernel(&manifold, &metric);
    assert!(
        result.is_ok(),
        "Kernel execution failed: {:?}",
        result.err()
    );

    let j = result.unwrap();
    // J is a current-density 1-form, so it is indexed by the 1-simplices.
    //
    // This assertion read `&[n3]` while the kernel open-coded the codifferential as a chain
    // ending in the *coboundary* d: Lambda^2 -> Lambda^3. That put the answer on 3-simplices,
    // and the test pinned it there. delta takes a 2-form to a 1-form; n1 is the only shape a
    // current density can have.
    assert_eq!(
        j.shape(),
        &[n1],
        "delta F is a 1-form: n0={n0}, n1={n1}, n2={n2}, n3={n3}"
    );
}

#[test]
fn test_relativistic_current_is_divergence_free() {
    // Charge conservation, d_mu J^mu = 0, which on the complex is delta J = delta^2 F = 0.
    //
    // This is exact rather than approximate, and it owes nothing to this implementation:
    //     delta_k = M_{k-1}^-1 B_k M_k
    // so delta_{k-1} delta_k = M^-1 B_{k-1} B_k M, and B_{k-1} B_k = 0 because the boundary of
    // a boundary is empty. The identity therefore holds whatever the mass matrices contain --
    // which is what makes it usable while the intermediate-grade Hodge star is still wrong.
    //
    // It is also the test that would have caught the defect this kernel carried: a chain built
    // on the coboundary instead of the boundary lands on the wrong skeleton, and delta cannot
    // even be applied to the result.
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();
    let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();

    let skeletons = complex.skeletons();
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();
    let total = complex.total_simplices();
    let num_edges = n1;

    // A 2-form with distinct entries on every face. A uniform or all-zero F would make
    // delta F vanish for any operator at all, so the divergence check would pin nothing.
    let mut data = vec![0.0_f64; total];
    for i in 0..n2 {
        data[n0 + n1 + i] = 1.0 + (i as f64) * 0.37;
    }

    let regge =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    let manifold = Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![total]).unwrap(),
        Some(regge),
        0,
    )
    .unwrap();

    let j = relativistic_current_kernel(&manifold, &EastCoastMetric::minkowski_4d()).unwrap();
    assert_eq!(j.shape(), &[n1]);

    // The current must not be trivially zero, or the divergence below would hold vacuously.
    let magnitude: f64 = j.as_slice().iter().map(|x: &f64| x.abs()).sum();
    assert!(
        magnitude > 1e-9,
        "J vanishes identically; the divergence check would pin nothing. |J| = {magnitude}"
    );

    // delta J, a 0-form on the vertices, must vanish.
    let div = manifold.codifferential_of(j.as_slice(), 1);
    assert_eq!(div.shape(), &[n0]);
    for (i, d) in div.as_slice().iter().enumerate() {
        assert!(
            d.abs() < 1e-9,
            "vertex {i}: div J = {d}, charge conservation requires 0"
        );
    }
}

#[test]
fn test_relativistic_current_is_linear_in_the_field() {
    // delta is linear, so scaling F scales J by the same factor. A kernel that squared the
    // field, or that added a constant anywhere, fails this while still returning plausible
    // numbers of the right shape.
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let build = |scale: f64| {
        let point_tensor = CausalTensor::new(points_data.clone(), vec![5, 4]).unwrap();
        let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
        let complex = cloud.triangulate(1.5).unwrap();
        let sk = complex.skeletons();
        let (n0, n1, n2) = (
            sk[0].simplices().len(),
            sk[1].simplices().len(),
            sk[2].simplices().len(),
        );
        let total = complex.total_simplices();
        let mut data = vec![0.0_f64; total];
        for i in 0..n2 {
            data[n0 + n1 + i] = scale * (1.0 + (i as f64) * 0.37);
        }
        let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap());
        let m = Manifold::with_metric(
            complex,
            CausalTensor::new(data, vec![total]).unwrap(),
            Some(regge),
            0,
        )
        .unwrap();
        relativistic_current_kernel(&m, &EastCoastMetric::minkowski_4d()).unwrap()
    };

    let single = build(1.0);
    let triple = build(3.0);
    for (i, (a, b)) in single.as_slice().iter().zip(triple.as_slice()).enumerate() {
        assert!(
            (3.0 * a - b).abs() < 1e-9,
            "edge {i}: 3*J(F) = {}, J(3F) = {b}",
            3.0 * a
        );
    }
}

#[test]
fn test_relativistic_current_refuses_a_manifold_without_a_metric() {
    // `codifferential_of` panics on a metric-less manifold, and `Manifold::new` builds exactly
    // that. The kernel must refuse rather than let the panic out.
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();
    let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();
    let total = complex.total_simplices();
    let manifold = Manifold::new(
        complex,
        CausalTensor::new(vec![1.0; total], vec![total]).unwrap(),
        0,
    )
    .unwrap();

    let err = relativistic_current_kernel(&manifold, &EastCoastMetric::minkowski_4d()).unwrap_err();
    // A missing metric means the operator cannot be formed, which the crate's topology-error
    // mapping reports as a CalculationError; the dimensions themselves are fine.
    assert!(
        matches!(err.0, PhysicsErrorEnum::CalculationError { .. }),
        "expected a CalculationError refusal, got {err:?}"
    );
    assert!(
        format!("{err}").contains("with_metric"),
        "the refusal must name the constructor that fixes it: {err}"
    );
}

#[test]
fn test_energy_momentum_tensor() {
    // Flat space 2D. F = [[0, E], [-E, 0]].
    let e = 1.0_f64;
    let f_data = vec![0.0, e, -e, 0.0];
    let em: CausalTensor<f64> = CausalTensor::new(f_data, vec![2, 2]).unwrap();

    // Metric diag(-1, 1) (Spacelike convention to get positive energy with standard formula)
    let g_data = vec![-1.0_f64, 0.0, 0.0, 1.0];
    let metric: CausalTensor<f64> = CausalTensor::new(g_data, vec![2, 2]).unwrap();

    let res = energy_momentum_tensor_em_kernel(&em, &metric);
    assert!(res.is_ok());

    let t = res.unwrap();
    // T00 = 0.5 * E^2
    let t00 = t.data()[0];
    assert!((t00 - 0.5).abs() < 1e-10);
    // This does discriminate the 1/4 coefficient (1/2 would give 0, 1/8 would give 0.75), but
    // it is a two-dimensional fixture, and the strongest invariant the tensor has --
    // tracelessness -- holds only in four. The test below supplies that case.
}

#[test]
fn test_energy_momentum_tensor_is_traceless_in_four_dimensions() {
    // The Maxwell stress-energy tensor
    //     T^uv = F^ua F^v_a - (1/4) g^uv F_ab F^ab
    // is traceless in four dimensions and only there: its trace is F^2 (1 - D/4). That makes
    // g_uv T^uv == 0 an exact oracle for the 1/4, and one that owes nothing to this
    // implementation -- it is pure index algebra. Measured on this fixture, a coefficient of
    // 1/8 leaves a trace of +7 and 1/2 leaves -14, so the assertion has teeth.
    //
    // Fixture: east-coast metric diag(-1,1,1,1), E along x and B along z, with
    //     F^01 = E = 3,  F^12 = -B = -4
    // following F^0i = E^i and F^ij = -eps^ijk B_k. E and B are given different magnitudes so
    // that no test below can pass by their coinciding.
    //
    // Provenance of the two pinned numbers, both standard results quoted in any classical
    // electrodynamics text (e.g. Jackson 3rd ed. Ch. 12), not derived from this code:
    //     F_ab F^ab = 2 (B^2 - E^2) = 14
    //     T^00      = (E^2 + B^2)/2 = 12.5      (the electromagnetic energy density)
    const E: f64 = 3.0;
    const B: f64 = 4.0;
    const T00: f64 = 12.5;

    let mut f_data = vec![0.0_f64; 16];
    f_data[1] = E; // F^01
    f_data[4] = -E; // F^10
    f_data[6] = -B; // F^12
    f_data[9] = B; // F^21
    let em: CausalTensor<f64> = CausalTensor::new(f_data, vec![4, 4]).unwrap();

    let mut g_data = vec![0.0_f64; 16];
    g_data[0] = -1.0;
    g_data[5] = 1.0;
    g_data[10] = 1.0;
    g_data[15] = 1.0;
    let metric: CausalTensor<f64> = CausalTensor::new(g_data.clone(), vec![4, 4]).unwrap();

    let t = energy_momentum_tensor_em_kernel(&em, &metric).unwrap();
    assert_eq!(t.shape(), &[4, 4]);

    // The energy density.
    assert!(
        (t.data()[0] - T00).abs() < 1e-10,
        "T^00 = {}, expected {T00}",
        t.data()[0]
    );

    // The trace. g is diagonal here, so g_uv T^uv reduces to the signed sum of the diagonal.
    let trace: f64 = (0..4)
        .map(|m| g_data[m * 4 + m] * t.data()[m * 4 + m])
        .sum();
    assert!(
        trace.abs() < 1e-10,
        "g_uv T^uv = {trace}, must vanish in four dimensions"
    );

    // The energy flux carries the Poynting magnitude |E x B| = 12. Only the magnitude is
    // asserted: the sign of T^0i depends on index placement and on the signature convention,
    // and this test does not undertake to pin which of those the crate uses.
    assert!(
        (t.data()[2].abs() - E * B).abs() < 1e-10,
        "|T^02| = {}, expected |E x B| = {}",
        t.data()[2].abs(),
        E * B
    );
}

#[test]
fn test_relativistic_current_kernel_low_dim_metric_error() {
    // Build a valid 4D manifold but pass a metric with dimension < 4
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();
    let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();
    let total = complex.total_simplices();
    let manifold = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; total], vec![total]).unwrap(),
        0,
    )
    .unwrap();

    let metric_3d = EastCoastMetric::new_nd(3).unwrap();
    let r = relativistic_current_kernel(&manifold, &metric_3d);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_relativistic_current_kernel_low_skeleton_error() {
    // 1D point cloud → triangulation produces only 0- and 1-skeletons, no 2-simplices.
    let points = CausalTensor::new(vec![0.0, 1.0, 2.0], vec![3, 1]).unwrap();
    let cloud = PointCloud::new(points, CausalTensor::<f64>::zeros(&[3]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();
    let total = complex.total_simplices();
    let manifold = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; total], vec![total]).unwrap(),
        0,
    )
    .unwrap();

    let metric = EastCoastMetric::minkowski_4d();
    let r = relativistic_current_kernel(&manifold, &metric);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_relativistic_current_kernel_insufficient_hodge_ops_error() {
    // A 2D triangular complex has skeletons {0,1,2} (len 3, passes the >=3
    // check) and a 4D metric (passes the dimension>=4 check), but only 3 Hodge
    // star operators (dims 0..=2) — fewer than the 4 required — so the kernel
    // hits the "Missing Hodge star operators" guard in `relativistic_current_kernel`.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let cloud = PointCloud::new(points, CausalTensor::<f64>::zeros(&[3]), 0).unwrap();
    let complex = cloud.triangulate(1.1).unwrap();
    let total = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let metric_regge =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    let manifold = Manifold::with_metric(
        complex,
        CausalTensor::new(vec![1.0; total], vec![total]).unwrap(),
        Some(metric_regge),
        0,
    )
    .unwrap();

    // 4D spacetime metric so the dimension check passes; failure must come from
    // the Hodge-operator count, not the metric dimension.
    let spacetime = EastCoastMetric::minkowski_4d();
    let r = relativistic_current_kernel(&manifold, &spacetime);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::CalculationError { .. }
        ),
        "expected a CalculationError refusal"
    );
}

/// The pentatope complex with a 2-form of distinct entries on every face, rebuilt through
/// `SimplicialComplex::new` from the given boundary and coboundary operators. The skeletons and
/// Hodge ⋆ operators are those of the triangulation.
fn pentatope_manifold(
    keep_boundary: bool,
    keep_coboundary: bool,
) -> Manifold<SimplicialComplex<f64>, f64> {
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();
    let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let full = cloud.triangulate(1.5).unwrap();

    let skeletons = full.skeletons().clone();
    let (n0, n1, n2) = (
        skeletons[0].simplices().len(),
        skeletons[1].simplices().len(),
        skeletons[2].simplices().len(),
    );
    let total = full.total_simplices();
    let boundary = if keep_boundary {
        full.boundary_operators().clone()
    } else {
        vec![]
    };
    let coboundary = if keep_coboundary {
        full.coboundary_operators().clone()
    } else {
        vec![]
    };
    let hodge = full.hodge_star_operators().unwrap().clone();
    let complex = SimplicialComplex::new(skeletons, boundary, coboundary, hodge);

    let mut data = vec![0.0_f64; total];
    for i in 0..n2 {
        data[n0 + n1 + i] = 1.0 + (i as f64) * 0.37;
    }
    let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![total]).unwrap(),
        Some(regge),
        0,
    )
    .unwrap()
}

#[test]
fn test_relativistic_current_does_not_need_coboundary_operators() {
    // delta_2 = M_1^-1 B_2 M_2 reads the boundary operator, never the coboundary. A complex that
    // carries its boundary operators and no coboundary operators therefore yields the same J as
    // the full complex.
    let full = pentatope_manifold(true, true);
    let no_coboundary = pentatope_manifold(true, false);
    let metric = EastCoastMetric::minkowski_4d();

    let expected = relativistic_current_kernel(&full, &metric).unwrap();
    let got = relativistic_current_kernel(&no_coboundary, &metric).unwrap();
    assert!(
        expected.as_slice().iter().any(|x: &f64| x.abs() > 1e-9),
        "J vanishes on the full complex, so the comparison would pin nothing"
    );
    assert_eq!(got.as_slice(), expected.as_slice());
}

#[test]
fn test_relativistic_current_refuses_a_complex_without_boundary_operators() {
    // `codifferential_of` reads an absent boundary operator as zero, so without this refusal a
    // complex carrying only coboundary operators would return J = 0 for any F.
    let manifold = pentatope_manifold(false, true);

    let err = relativistic_current_kernel(&manifold, &EastCoastMetric::minkowski_4d()).unwrap_err();
    match err.0 {
        PhysicsErrorEnum::CalculationError(msg) => assert!(
            msg.contains("Missing boundary operator") && msg.contains("have 0"),
            "unexpected message: {msg}"
        ),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

// NOTE on defensively-unreachable GRMHD branches:
//   * `relativistic_current_kernel`, "Manifold data too short for 2-form extraction".
//     `Manifold::new` and `Manifold::with_metric` reject any data tensor whose length differs
//     from the complex's total simplex count, and that total is at least n0 + n1 + n2, so the
//     data slab is never shorter than the 2-form domain.
//   * `energy_momentum_tensor_em_kernel`, the `|| (len == 1 && [0] == 1)` operand and the
//     "Scalar contraction failed" else-arm: the kernel admits only rank-2
//     `em_tensor` and `metric`, and contracting two rank-2 tensors over both
//     axes always yields a scalar whose shape `is_empty()` is true. That
//     short-circuits the `||` and never takes the else. The scalar path is
//     exercised by `test_energy_momentum_tensor`.

#[test]
fn test_energy_momentum_tensor_dimension_error() {
    let em = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();
    assert!(
        matches!(
            energy_momentum_tensor_em_kernel(&em, &metric)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}
