/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law, convergence, and error-path tests for the de Rham (♭) and sharp (♯)
//! transfer maps and their Tier-2 iso witness (`DeRhamSharpIso`), plus the
//! G1 convective cross-validation (`i_u du♭` against the analytic
//! `u^j(∂_j u_i − ∂_i u_j)`), which closes the loop between the exterior
//! algebra and the transfer layer.
//!
//! Orientation pin: on gradients of linear potentials the trapezoid de Rham
//! map is exact, so `de_rham(∇φ) = d(φ samples)` holds to machine precision —
//! the fundamental theorem of calculus fixes the edge-orientation convention
//! against `exterior_derivative`.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn unit_manifold<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn manifold_with_k_form<const D: usize>(
    lattice: LatticeComplex<D, f64>,
    k: usize,
    form: &[f64],
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|g| lattice.num_cells(g)).sum();
    let offset: usize = (0..k).map(|g| lattice.num_cells(g)).sum();
    let mut data = vec![0.0; total];
    data[offset..offset + form.len()].copy_from_slice(form);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, tensor, metric, 0)
}

/// Sample a vector field at the vertices (layout `vertex * D + axis`).
fn sample_vertex_field<const D: usize>(
    lattice: &LatticeComplex<D, f64>,
    f: impl Fn(&[usize; D], usize) -> f64,
) -> Vec<f64> {
    let n0 = lattice.num_cells(0);
    let mut out = vec![0.0; D * n0];
    for (vi, v) in lattice.iter_cells(0).enumerate() {
        for axis in 0..D {
            out[vi * D + axis] = f(v.position(), axis);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Orientation pin: FTC on gradients of linear potentials
// ---------------------------------------------------------------------------

#[test]
fn de_rham_of_linear_gradient_equals_d_of_sampled_potential() {
    // φ(x, y) = 2x + 3y on an open lattice: ∇φ = (2, 3) is constant, the
    // trapezoid rule is exact, and de_rham(∇φ) must equal d(φ samples) to
    // machine precision. This is the FTC orientation pin.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_open(4));

    let phi: Vec<f64> = lattice
        .iter_cells(0)
        .map(|c| 2.0 * c.position()[0] as f64 + 3.0 * c.position()[1] as f64)
        .collect();
    let grad = sample_vertex_field(&lattice, |_, axis| if axis == 0 { 2.0 } else { 3.0 });

    let n0 = lattice.num_cells(0);
    let grad_tensor = CausalTensor::new(grad, vec![2 * n0]).unwrap();
    let flat = manifold.de_rham(&grad_tensor).unwrap();

    let m_phi = manifold_with_k_form(LatticeComplex::<2, f64>::square_open(4), 0, &phi);
    let d_phi = m_phi.exterior_derivative(0);

    for (i, (a, b)) in flat
        .as_slice()
        .iter()
        .zip(d_phi.as_slice().iter())
        .enumerate()
    {
        assert!(
            (a - b).abs() < 1e-13,
            "edge {i}: de_rham(∇φ) = {a}, d(φ) = {b}"
        );
    }
}

// ---------------------------------------------------------------------------
// Round-trips: exact on constants (incl. the iso witness law), O(h²) on smooth
// ---------------------------------------------------------------------------

fn assert_constant_round_trip<R>(tol: R)
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display
        + Clone,
{
    let manifold = unit_manifold(LatticeComplex::<2, R>::square_torus(4));
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    let c0 = R::from_f64(1.25).expect("lifts");
    let c1 = R::from_f64(-0.5).expect("lifts");

    let mut field = vec![R::zero(); 2 * n0];
    for v in 0..n0 {
        field[2 * v] = c0;
        field[2 * v + 1] = c1;
    }
    let field_tensor = CausalTensor::new(field.clone(), vec![2 * n0]).unwrap();

    let cochain = manifold.de_rham(&field_tensor).unwrap();
    let back = manifold.sharp(&cochain).unwrap();

    for (i, (a, b)) in back.as_slice().iter().zip(field.iter()).enumerate() {
        assert!((*a - *b).abs() < tol, "component {i}: {a} vs {b}");
    }
}

#[test]
fn constant_round_trip_exact_f64() {
    assert_constant_round_trip::<f64>(1e-14);
}

#[test]
fn constant_round_trip_exact_f32() {
    assert_constant_round_trip::<f32>(1e-6_f32);
}

#[test]
fn constant_round_trip_exact_float106() {
    assert_constant_round_trip::<Float106>(Float106::from_f64(1e-29));
}

#[test]
fn smooth_round_trip_converges_at_second_order() {
    // sharp(de_rham(u)) − u shrinks ≥ 3× per grid doubling for a smooth
    // periodic field.
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
        let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(n));
        let k_wave = 2.0 * std::f64::consts::PI / (n as f64);

        let field = sample_vertex_field(&lattice, |p, axis| {
            let (x, y) = (p[0] as f64, p[1] as f64);
            if axis == 0 {
                (k_wave * x).sin() * (k_wave * y).cos()
            } else {
                -(k_wave * x).cos() * (k_wave * y).sin()
            }
        });
        let n0 = lattice.num_cells(0);
        let tensor = CausalTensor::new(field.clone(), vec![2 * n0]).unwrap();

        let back = manifold.sharp(&manifold.de_rham(&tensor).unwrap()).unwrap();

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (a, b) in back.as_slice().iter().zip(field.iter()) {
            max_err = max_err.max((a - b).abs());
            max_ref = max_ref.max(b.abs());
        }
        rel_errors.push(max_err / max_ref);
    }
    assert!(rel_errors[1] < rel_errors[0] / 3.0, "{rel_errors:?}");
    assert!(rel_errors[2] < rel_errors[1] / 3.0, "{rel_errors:?}");
}

// ---------------------------------------------------------------------------
// Naturality (linearity) of both maps
// ---------------------------------------------------------------------------

#[test]
fn transfer_maps_are_linear() {
    // de_rham(a·u + v) = a·de_rham(u) + de_rham(v), and likewise for sharp —
    // the naturality law for these linear maps, exact at machine precision.
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(4));
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    let n1 = complex.num_cells(1);
    let a = 2.75_f64;

    let u: Vec<f64> = (0..2 * n0).map(|i| ((i as f64) * 0.37).sin()).collect();
    let v: Vec<f64> = (0..2 * n0).map(|i| ((i as f64) * 0.61).cos()).collect();
    let combo: Vec<f64> = u.iter().zip(v.iter()).map(|(x, y)| a * x + y).collect();

    let t = |vals: &[f64]| CausalTensor::new(vals.to_vec(), vec![vals.len()]).unwrap();

    let lhs = manifold.de_rham(&t(&combo)).unwrap();
    let du = manifold.de_rham(&t(&u)).unwrap();
    let dv = manifold.de_rham(&t(&v)).unwrap();
    for i in 0..lhs.len() {
        let rhs = a * du.as_slice()[i] + dv.as_slice()[i];
        assert!((lhs.as_slice()[i] - rhs).abs() < 1e-12, "de_rham edge {i}");
    }

    let w: Vec<f64> = (0..n1).map(|i| ((i as f64) * 0.41).sin()).collect();
    let z: Vec<f64> = (0..n1).map(|i| ((i as f64) * 0.83).cos()).collect();
    let combo_e: Vec<f64> = w.iter().zip(z.iter()).map(|(x, y)| a * x + y).collect();

    let lhs_s = manifold.sharp(&t(&combo_e)).unwrap();
    let sw = manifold.sharp(&t(&w)).unwrap();
    let sz = manifold.sharp(&t(&z)).unwrap();
    for i in 0..lhs_s.len() {
        let rhs = a * sw.as_slice()[i] + sz.as_slice()[i];
        assert!((lhs_s.as_slice()[i] - rhs).abs() < 1e-12, "sharp comp {i}");
    }
}

// ---------------------------------------------------------------------------
// Open-boundary trimming and the exact-integral entry point
// ---------------------------------------------------------------------------

#[test]
fn sharp_trims_and_renormalizes_at_open_boundaries() {
    // Mixed [periodic, open] lattice: at the low/high open boundary along
    // axis 1 only one incident y-edge exists; sharp must use exactly that
    // edge's value rather than averaging with a phantom.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::new([2, 2], [true, false]);
    let manifold = unit_manifold(LatticeComplex::<2, f64>::new([2, 2], [true, false]));
    let n1 = lattice.num_cells(1);

    // Distinct value per edge so trimming is observable.
    let cochain: Vec<f64> = (0..n1).map(|i| (i + 1) as f64).collect();
    let tensor = CausalTensor::new(cochain.clone(), vec![n1]).unwrap();
    let vectors = manifold.sharp(&tensor).unwrap();

    // Edge index map in canonical ordering.
    let edge_at = |pos: [usize; 2], axis: usize| -> f64 {
        for (i, c) in lattice.iter_cells(1).enumerate() {
            if *c.position() == pos && c.orientation() == (1 << axis) {
                return cochain[i];
            }
        }
        unreachable!("edge not found");
    };
    let vertex_idx = |pos: [usize; 2]| -> usize {
        for (i, c) in lattice.iter_cells(0).enumerate() {
            if *c.position() == pos {
                return i;
            }
        }
        unreachable!("vertex not found");
    };

    // Vertex (0, 0): only the outgoing y-edge exists (low open boundary).
    let v00_y = vectors.as_slice()[vertex_idx([0, 0]) * 2 + 1];
    assert!((v00_y - edge_at([0, 0], 1)).abs() < 1e-14);

    // Vertex (0, 1): only the incoming y-edge exists (high open boundary).
    let v01_y = vectors.as_slice()[vertex_idx([0, 1]) * 2 + 1];
    assert!((v01_y - edge_at([0, 0], 1)).abs() < 1e-14);

    // Periodic axis 0 is untrimmed: interior average of two incident x-edges.
    let v00_x = vectors.as_slice()[vertex_idx([0, 0]) * 2];
    let expected = 0.5 * (edge_at([0, 0], 0) + edge_at([1, 0], 0));
    assert!((v00_x - expected).abs() < 1e-14);
}

#[test]
fn de_rham_from_integrals_is_a_validating_passthrough() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let n1 = manifold.complex().num_cells(1);
    let integrals: Vec<f64> = (0..n1).map(|i| i as f64 * 0.5).collect();
    let tensor = CausalTensor::new(integrals.clone(), vec![n1]).unwrap();

    let out = manifold.de_rham_from_integrals(&tensor).unwrap();
    assert_eq!(out.as_slice(), integrals.as_slice());
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn de_rham_rejects_length_mismatch() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let bad = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap();
    let err = manifold.de_rham(&bad).unwrap_err();
    assert!(format!("{err}").contains("de_rham input"));
}

#[test]
fn de_rham_from_integrals_rejects_length_mismatch() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let bad = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap();
    let err = manifold.de_rham_from_integrals(&bad).unwrap_err();
    assert!(format!("{err}").contains("de_rham_from_integrals"));
}

#[test]
fn sharp_rejects_length_mismatch() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let bad = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap();
    let err = manifold.sharp(&bad).unwrap_err();
    assert!(format!("{err}").contains("sharp input"));
}

#[test]
fn de_rham_and_sharp_reject_missing_metric() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n0 = lattice.num_cells(0);
    let n1 = lattice.num_cells(1);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical(lattice, data, 0);

    let vfield = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let err = manifold.de_rham(&vfield).unwrap_err();
    assert!(format!("{err}").contains("requires a metric"));

    let cochain = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let err = manifold.sharp(&cochain).unwrap_err();
    assert!(format!("{err}").contains("requires a metric"));
}

// ---------------------------------------------------------------------------
// G1 closure: convective cross-validation (i_u du♭ vs. the analytic identity)
// ---------------------------------------------------------------------------

#[test]
fn convective_term_cross_validates_against_analytic_identity() {
    // For the Taylor–Green field u = (sin kx cos ky, −cos kx sin ky):
    //   (i_u du)_i = u^j (∂_j u_i − ∂_i u_j), with
    //   du₁₂ = ω = 2k sin(kx) sin(ky), so
    //   (i_u du)₁ = −u₂·ω = cos(kx) sin(ky) · 2k sin(kx) sin(ky)
    //   (i_u du)₂ =  u₁·ω = sin(kx) cos(ky) · 2k sin(kx) sin(ky)
    // The discrete chain de_rham → d → interior_product must converge to this
    // at second order. This is acceptance criterion (c) of the
    // dec-exterior-algebra capability: two derivative paths, no shared
    // discretization code.
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
        let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(n));
        let k_wave = 2.0 * std::f64::consts::PI / (n as f64);

        let field = sample_vertex_field(&lattice, |p, axis| {
            let (x, y) = (p[0] as f64, p[1] as f64);
            if axis == 0 {
                (k_wave * x).sin() * (k_wave * y).cos()
            } else {
                -(k_wave * x).cos() * (k_wave * y).sin()
            }
        });
        let n0 = lattice.num_cells(0);
        let tensor = CausalTensor::new(field, vec![2 * n0]).unwrap();
        let u_flat = manifold.de_rham(&tensor).unwrap();

        let m_u = manifold_with_k_form(
            LatticeComplex::<2, f64>::square_torus(n),
            1,
            u_flat.as_slice(),
        );
        let du = m_u.exterior_derivative(1);
        let conv = manifold.interior_product(&u_flat, &du, 2).unwrap();

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (i, cell) in lattice.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            let p = cell.position();
            let (mx, my) = if axis == 0 {
                (p[0] as f64 + 0.5, p[1] as f64)
            } else {
                (p[0] as f64, p[1] as f64 + 0.5)
            };
            let (sx, cx) = ((k_wave * mx).sin(), (k_wave * mx).cos());
            let (sy, cy) = ((k_wave * my).sin(), (k_wave * my).cos());
            let omega = 2.0 * k_wave * sx * sy;
            let analytic = if axis == 0 {
                cx * sy * omega
            } else {
                sx * cy * omega
            };
            max_err = max_err.max((conv.as_slice()[i] - analytic).abs());
            max_ref = max_ref.max(analytic.abs());
        }
        rel_errors.push(max_err / max_ref);
    }
    assert!(
        rel_errors[1] < rel_errors[0] / 3.0,
        "convective cross-validation not second order: {rel_errors:?}"
    );
    assert!(
        rel_errors[2] < rel_errors[1] / 3.0,
        "convective cross-validation not second order: {rel_errors:?}"
    );
}
