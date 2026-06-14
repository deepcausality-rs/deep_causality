/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sign-pinning, law, convergence, and error-path tests for the discrete
//! interior product `Manifold::interior_product` (`i_X ω = ±⋆(⋆ω ∧ X♭)`).
//!
//! The constant-field tests pin every orientation sign exactly (averaging of
//! constants is exact, so any sign error shows as an O(1) violation). The
//! Cartan test checks `i_X dω + d i_X ω` against the continuum Lie derivative
//! at second order under refinement.
//!
//! Unreachable inner-error branches: the `wedge` call inside
//! `interior_product` cannot fail once the outer validation has passed
//! (grade overflow requires `k = 0`, which is rejected first; operand lengths
//! are correct by construction), and `hodge_star_matrix` failures are
//! prevented by metric validation at `from_cubical_with_metric` construction.
//! Both propagation paths exist defensively and are documented here per the
//! AGENTS.md unreachable-code exemption.

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
        + deep_causality_par::MaybeParallel
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
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    // Const-generic mismatch guard: this helper is only used with D = 2.
    let _ = &metric;
    let metric_d: CubicalReggeGeometry<D, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, tensor, metric_d, 0)
}

/// Per-axis constant 1-form on the lattice: edges along axis `a` get `vals[a]`.
fn constant_one_form<const D: usize, R>(lattice: &LatticeComplex<D, R>, vals: [R; D]) -> Vec<R>
where
    R: RealField + deep_causality_par::MaybeParallel,
{
    lattice
        .iter_cells(1)
        .map(|c| vals[c.orientation().trailing_zeros() as usize])
        .collect()
}

// ---------------------------------------------------------------------------
// Sign pins: constant fields are exact through the averaged transport
// ---------------------------------------------------------------------------

/// 2D, k = 2 (even k(D−k) branch): i_X (c dx∧dy) = c·X¹ dy − c·X² dx.
fn assert_2d_two_form_contraction<R>(tol: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(4);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();

    let c = R::from_f64(3.0).expect("lifts");
    let x1 = R::from_f64(0.5).expect("lifts");
    let x2 = R::from_f64(-2.0).expect("lifts");

    let n2 = complex.num_cells(2);
    let omega = CausalTensor::new(vec![c; n2], vec![n2]).unwrap();
    let x_vals = constant_one_form(complex, [x1, x2]);
    let n1 = complex.num_cells(1);
    let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

    let result = manifold.interior_product(&x_flat, &omega, 2).unwrap();

    for (i, cell) in complex.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        let expected = if axis == 0 {
            R::zero() - x2 * c // x-edges: −X²·c
        } else {
            x1 * c // y-edges: +X¹·c
        };
        let got = result.as_slice()[i];
        assert!(
            (got - expected).abs() < tol,
            "edge {i} (axis {axis}): got {got}, expected {expected}"
        );
    }
}

#[test]
fn two_form_contraction_signs_2d_f64() {
    assert_2d_two_form_contraction::<f64>(1e-13);
}

#[test]
fn two_form_contraction_signs_2d_f32() {
    assert_2d_two_form_contraction::<f32>(1e-5_f32);
}

#[test]
fn two_form_contraction_signs_2d_float106() {
    assert_2d_two_form_contraction::<Float106>(Float106::from_f64(1e-28));
}

/// 2D, k = 1 (odd k(D−k) branch): i_X ω = X¹ω₁ + X²ω₂ (a 0-form).
#[test]
fn one_form_contraction_is_inner_product_2d() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(4);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();

    let (w1, w2, x1, x2) = (1.5, -0.75, 2.0, 4.0);
    let n1 = complex.num_cells(1);
    let omega = CausalTensor::new(constant_one_form(complex, [w1, w2]), vec![n1]).unwrap();
    let x_flat = CausalTensor::new(constant_one_form(complex, [x1, x2]), vec![n1]).unwrap();

    let result = manifold.interior_product(&x_flat, &omega, 1).unwrap();
    let expected = x1 * w1 + x2 * w2;
    for (i, got) in result.as_slice().iter().enumerate() {
        assert!(
            (got - expected).abs() < 1e-13,
            "vertex {i}: got {got}, expected {expected}"
        );
    }
}

/// 3D, k = 2: i_X ω with ω = ω₁₂ dx∧dy + ω₁₃ dx∧dz + ω₂₃ dy∧dz gives
/// dx: −X²ω₁₂ − X³ω₁₃; dy: X¹ω₁₂ − X³ω₂₃; dz: X¹ω₁₃ + X²ω₂₃.
#[test]
fn two_form_contraction_signs_3d() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(3);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();

    let (w12, w13, w23) = (1.0, 2.0, 4.0);
    let (x1, x2, x3) = (0.5, -1.5, 3.0);

    // Face cochain: orientation bits {0,1} → ω₁₂, {0,2} → ω₁₃, {1,2} → ω₂₃.
    let n2 = complex.num_cells(2);
    let omega_vals: Vec<f64> = complex
        .iter_cells(2)
        .map(|c| match c.orientation() {
            0b011 => w12,
            0b101 => w13,
            0b110 => w23,
            o => unreachable!("unexpected 2-cell orientation {o:#b}"),
        })
        .collect();
    let omega = CausalTensor::new(omega_vals, vec![n2]).unwrap();

    let n1 = complex.num_cells(1);
    let x_flat = CausalTensor::new(constant_one_form(complex, [x1, x2, x3]), vec![n1]).unwrap();

    let result = manifold.interior_product(&x_flat, &omega, 2).unwrap();

    for (i, cell) in complex.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        let expected = match axis {
            0 => -x2 * w12 - x3 * w13,
            1 => x1 * w12 - x3 * w23,
            _ => x1 * w13 + x2 * w23,
        };
        let got = result.as_slice()[i];
        assert!(
            (got - expected).abs() < 1e-13,
            "edge {i} (axis {axis}): got {got}, expected {expected}"
        );
    }
}

/// Open-lattice contraction: exercises the boundary-trimming transport
/// branches (offset combinations that fall off the open boundary are skipped
/// and the average renormalized). With the boundary-corrected Hodge star
/// (wall-hodge-star, add-walls-and-dec-stencils) the dual volumes clip at
/// walls, so the constant-field expectation is exact only where the
/// transport gathers unclipped interior star entries: the analytic value is
/// asserted on interior edges (margin one from every wall), the orientation
/// sign everywhere.
#[test]
fn two_form_contraction_signs_2d_open_lattice() {
    let n = 4usize;
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(n);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();

    let (c, x1, x2) = (2.0, 1.5, -0.5);
    let n2 = complex.num_cells(2);
    let n1 = complex.num_cells(1);
    let omega = CausalTensor::new(vec![c; n2], vec![n2]).unwrap();
    let x_flat = CausalTensor::new(constant_one_form(complex, [x1, x2]), vec![n1]).unwrap();

    let result = manifold.interior_product(&x_flat, &omega, 2).unwrap();

    for (i, cell) in complex.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        let expected = if axis == 0 { -x2 * c } else { x1 * c };
        let got = result.as_slice()[i];

        let pos = cell.position();
        let active_extent = |a: usize| if a == axis { n - 1 } else { n };
        let interior = (0..2).all(|a| pos[a] >= 1 && pos[a] + 2 <= active_extent(a));
        if interior {
            assert!(
                (got - expected).abs() < 1e-13,
                "interior edge {i} (axis {axis}): got {got}, expected {expected}"
            );
        } else {
            assert!(
                got * expected > 0.0,
                "boundary edge {i} (axis {axis}): got {got}, expected sign of {expected}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Contraction twice with the same 1-form
// ---------------------------------------------------------------------------

#[test]
fn contraction_twice_vanishes_for_constant_direction() {
    // i_X (i_X ω) = 0: exact for a constant contraction direction (averaging
    // of constants is exact, so the continuum antisymmetry cancellation
    // survives discretization unchanged).
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(4);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();

    let n2 = complex.num_cells(2);
    let n1 = complex.num_cells(1);
    // Non-constant ω, constant X.
    let omega_vals: Vec<f64> = (0..n2).map(|i| (i as f64 * 0.7).sin() + 1.3).collect();
    let omega = CausalTensor::new(omega_vals, vec![n2]).unwrap();
    let x_flat = CausalTensor::new(constant_one_form(complex, [1.25, -0.5]), vec![n1]).unwrap();

    let once = manifold.interior_product(&x_flat, &omega, 2).unwrap();
    let twice = manifold.interior_product(&x_flat, &once, 1).unwrap();

    for (i, v) in twice.as_slice().iter().enumerate() {
        assert!(v.abs() < 1e-12, "vertex {i}: i_X i_X ω = {v}, expected 0");
    }
}

#[test]
fn contraction_twice_converges_to_zero_for_varying_direction() {
    // For a spatially varying X the cancellation holds to discretization
    // order: the residual shrinks at least ~4× per refinement (second order)
    // relative to the magnitude of the single contraction. The ladder starts
    // at N = 16: at N = 8 the asymptotic regime has not yet set in (measured
    // ratio 2.1× for the 8→16 step vs. 3.5× for 16→32).
    let mut rel_residuals = Vec::new();
    for n in [16usize, 32, 64] {
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
        let manifold = unit_manifold(lattice);
        let complex = manifold.complex();
        let k_wave = 2.0 * std::f64::consts::PI / (n as f64);

        let n2 = complex.num_cells(2);
        let n1 = complex.num_cells(1);
        let omega_vals: Vec<f64> = complex
            .iter_cells(2)
            .map(|c| {
                let p = c.position();
                1.0 + (k_wave * (p[0] as f64 + 0.5)).sin() * (k_wave * (p[1] as f64 + 0.5)).cos()
            })
            .collect();
        let omega = CausalTensor::new(omega_vals, vec![n2]).unwrap();

        let x_vals: Vec<f64> = complex
            .iter_cells(1)
            .map(|c| {
                let axis = c.orientation().trailing_zeros() as usize;
                let p = c.position();
                let (mx, my) = if axis == 0 {
                    (p[0] as f64 + 0.5, p[1] as f64)
                } else {
                    (p[0] as f64, p[1] as f64 + 0.5)
                };
                if axis == 0 {
                    (k_wave * my).cos()
                } else {
                    (k_wave * mx).sin()
                }
            })
            .collect();
        let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

        let once = manifold.interior_product(&x_flat, &omega, 2).unwrap();
        let twice = manifold.interior_product(&x_flat, &once, 1).unwrap();

        let scale = once
            .as_slice()
            .iter()
            .fold(0.0_f64, |m, v| m.max(v.abs()))
            .max(1e-300);
        let resid = twice.as_slice().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        rel_residuals.push(resid / scale);
    }
    assert!(
        rel_residuals[1] < rel_residuals[0] / 3.0,
        "{rel_residuals:?}"
    );
    assert!(
        rel_residuals[2] < rel_residuals[1] / 3.0,
        "{rel_residuals:?}"
    );
}

// ---------------------------------------------------------------------------
// Cartan's magic formula: i_X dω + d i_X ω → L_X ω at second order
// ---------------------------------------------------------------------------

#[test]
fn cartan_formula_converges_to_lie_derivative_at_second_order() {
    // ω = (sin(ky), sin(kx)), X = (cos(kx), cos(ky)) on an N×N torus with
    // k = 2π/N. Continuum Lie derivative (L_X ω)_i = X^j ∂_j ω_i + ω_j ∂_i X^j:
    //   (L_X ω)₁ = k·cos²(ky) − k·sin(ky)·sin(kx)
    //   (L_X ω)₂ = k·cos²(kx) − k·sin(kx)·sin(ky)
    // evaluated at edge midpoints. The discrete i_X dω + d i_X ω must agree
    // with relative sup-error shrinking ≥ 3× per grid doubling.
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
        let manifold = unit_manifold(lattice);
        let complex = manifold.complex();
        let k_wave = 2.0 * std::f64::consts::PI / (n as f64);

        let n1 = complex.num_cells(1);
        let midpoint = |c: &deep_causality_topology::LatticeCell<2>| {
            let axis = c.orientation().trailing_zeros() as usize;
            let p = c.position();
            if axis == 0 {
                (p[0] as f64 + 0.5, p[1] as f64, axis)
            } else {
                (p[0] as f64, p[1] as f64 + 0.5, axis)
            }
        };

        let omega_vals: Vec<f64> = complex
            .iter_cells(1)
            .map(|c| {
                let (mx, my, axis) = midpoint(&c);
                if axis == 0 {
                    (k_wave * my).sin()
                } else {
                    (k_wave * mx).sin()
                }
            })
            .collect();
        let x_vals: Vec<f64> = complex
            .iter_cells(1)
            .map(|c| {
                let (mx, my, axis) = midpoint(&c);
                if axis == 0 {
                    (k_wave * mx).cos()
                } else {
                    (k_wave * my).cos()
                }
            })
            .collect();
        let omega = CausalTensor::new(omega_vals.clone(), vec![n1]).unwrap();
        let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

        // i_X dω
        let m_omega =
            manifold_with_k_form(LatticeComplex::<2, f64>::square_torus(n), 1, &omega_vals);
        let d_omega = m_omega.exterior_derivative(1);
        let term1 = manifold.interior_product(&x_flat, &d_omega, 2).unwrap();

        // d i_X ω
        let ix_omega = manifold.interior_product(&x_flat, &omega, 1).unwrap();
        let m_ix = manifold_with_k_form(
            LatticeComplex::<2, f64>::square_torus(n),
            0,
            ix_omega.as_slice(),
        );
        let term2 = m_ix.exterior_derivative(0);

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (i, cell) in complex.iter_cells(1).enumerate() {
            let (mx, my, axis) = midpoint(&cell);
            let analytic = if axis == 0 {
                k_wave * (k_wave * my).cos().powi(2)
                    - k_wave * (k_wave * my).sin() * (k_wave * mx).sin()
            } else {
                k_wave * (k_wave * mx).cos().powi(2)
                    - k_wave * (k_wave * mx).sin() * (k_wave * my).sin()
            };
            let discrete = term1.as_slice()[i] + term2.as_slice()[i];
            max_err = max_err.max((discrete - analytic).abs());
            max_ref = max_ref.max(analytic.abs());
        }
        rel_errors.push(max_err / max_ref);
    }

    assert!(
        rel_errors[1] < rel_errors[0] / 3.0,
        "Cartan first refinement not second order: {rel_errors:?}"
    );
    assert!(
        rel_errors[2] < rel_errors[1] / 3.0,
        "Cartan second refinement not second order: {rel_errors:?}"
    );
}

// Cartan's magic formula on a SMOOTHLY GRADED metric (CFD R1, task B1): the
// operator law must survive grading. Axis-1 edge lengths are modulated by
// `1 + a·cos(2π·pos/N)` — smooth, periodic, and summing to N, so the wavenumber is
// unchanged. The manufactured solution is evaluated at *physical* (cumulative-length)
// y-midpoints, and the interior product uses the graded ⋆. With smooth grading the
// discrete `i_X dω + d i_X ω` still converges to the Lie derivative under refinement.
// ---------------------------------------------------------------------------

fn graded_manifold_2d(
    lattice: LatticeComplex<2, f64>,
    metric: CubicalReggeGeometry<2, f64>,
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

#[test]
fn cartan_formula_converges_under_smooth_grading() {
    const AMP: f64 = 0.1;
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);

        // Smooth periodic edge-length modulation on axis 1; axis 0 stays unit.
        let len =
            |pos: usize| 1.0 + AMP * (2.0 * std::f64::consts::PI * pos as f64 / n as f64).cos();
        let edge_lengths: Vec<f64> = lattice
            .iter_cells(1)
            .map(|c| {
                let axis = c.orientation().trailing_zeros() as usize;
                if axis == 1 { len(c.position()[1]) } else { 1.0 }
            })
            .collect();
        let metric = CubicalReggeGeometry::<2, f64>::from_edge_lengths(edge_lengths);
        let manifold = graded_manifold_2d(LatticeComplex::square_torus(n), metric);
        let complex = manifold.complex();

        // Physical y-coordinate of vertex j (cumulative edge length); the modulation
        // sums to N, so the total length is N and the wavenumber is the uniform k.
        let mut y_node = vec![0.0_f64; n + 1];
        for j in 0..n {
            y_node[j + 1] = y_node[j] + len(j);
        }
        let k_wave = 2.0 * std::f64::consts::PI / (n as f64);

        let n1 = complex.num_cells(1);
        // (physical-x, physical-y, axis) of an edge midpoint.
        let midpoint = |c: &deep_causality_topology::LatticeCell<2>| {
            let axis = c.orientation().trailing_zeros() as usize;
            let p = c.position();
            let x = if axis == 0 {
                p[0] as f64 + 0.5
            } else {
                p[0] as f64
            };
            let y = if axis == 1 {
                0.5 * (y_node[p[1]] + y_node[p[1] + 1])
            } else {
                y_node[p[1]]
            };
            (x, y, axis)
        };

        let omega_vals: Vec<f64> = complex
            .iter_cells(1)
            .map(|c| {
                let (mx, my, axis) = midpoint(&c);
                if axis == 0 {
                    (k_wave * my).sin()
                } else {
                    (k_wave * mx).sin()
                }
            })
            .collect();
        // `x_flat` is X♭, the flat 1-form: its edge value is `X^axis · length(edge)`.
        // On the unit metric this equals the vector component; on a graded mesh it must
        // carry the edge-length factor, else the contraction is inconsistent.
        let x_vals: Vec<f64> = complex
            .iter_cells(1)
            .map(|c| {
                let (mx, my, axis) = midpoint(&c);
                let component = if axis == 0 {
                    (k_wave * mx).cos()
                } else {
                    (k_wave * my).cos()
                };
                let length = if axis == 1 { len(c.position()[1]) } else { 1.0 };
                component * length
            })
            .collect();
        let omega = CausalTensor::new(omega_vals.clone(), vec![n1]).unwrap();
        let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

        // i_X dω (d is metric-free, so a unit-metric carrier suffices for it).
        let m_omega =
            manifold_with_k_form(LatticeComplex::<2, f64>::square_torus(n), 1, &omega_vals);
        let d_omega = m_omega.exterior_derivative(1);
        let term1 = manifold.interior_product(&x_flat, &d_omega, 2).unwrap();

        // d i_X ω
        let ix_omega = manifold.interior_product(&x_flat, &omega, 1).unwrap();
        let m_ix = manifold_with_k_form(
            LatticeComplex::<2, f64>::square_torus(n),
            0,
            ix_omega.as_slice(),
        );
        let term2 = m_ix.exterior_derivative(0);

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (i, cell) in complex.iter_cells(1).enumerate() {
            let (mx, my, axis) = midpoint(&cell);
            let analytic = if axis == 0 {
                k_wave * (k_wave * my).cos().powi(2)
                    - k_wave * (k_wave * my).sin() * (k_wave * mx).sin()
            } else {
                k_wave * (k_wave * mx).cos().powi(2)
                    - k_wave * (k_wave * mx).sin() * (k_wave * my).sin()
            };
            let discrete = term1.as_slice()[i] + term2.as_slice()[i];
            max_err = max_err.max((discrete - analytic).abs());
            max_ref = max_ref.max(analytic.abs());
        }
        rel_errors.push(max_err / max_ref);
    }

    // Smooth (mild) grading must not destroy convergence: the finest-grid error is well
    // below the coarsest. A robust trend check — the strict order quantification and the
    // grading-amplitude limit where order collapses live in the graded-MMS example, per
    // the tests-fast / examples-verify split. (At strong grading the interior product's
    // order degrades, as expected; that boundary is the example's job to quantify.)
    assert!(
        rel_errors[2] < 0.6 * rel_errors[0],
        "smooth grading broke convergence: {rel_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn interior_product_rejects_zero_form() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    let n1 = complex.num_cells(1);
    let omega = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let x_flat = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = manifold.interior_product(&x_flat, &omega, 0).unwrap_err();
    assert!(format!("{err}").contains("requires 1 <= k <= D"));
}

#[test]
fn interior_product_rejects_grade_above_dimension() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(vec![0.0; 1], vec![1]).unwrap();
    let x_flat = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = manifold.interior_product(&x_flat, &omega, 3).unwrap_err();
    assert!(format!("{err}").contains("requires 1 <= k <= D"));
}

#[test]
fn interior_product_rejects_contraction_field_length_mismatch() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n2 = manifold.complex().num_cells(2);
    let omega = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();
    let x_flat = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();

    let err = manifold.interior_product(&x_flat, &omega, 2).unwrap_err();
    assert!(format!("{err}").contains("contraction field"));
}

#[test]
fn interior_product_rejects_form_length_mismatch() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let x_flat = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = manifold.interior_product(&x_flat, &omega, 2).unwrap_err();
    assert!(format!("{err}").contains("form operand"));
}

#[test]
fn interior_product_rejects_missing_metric() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n1 = lattice.num_cells(1);
    let n2 = lattice.num_cells(2);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical(lattice, data, 0);

    let omega = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();
    let x_flat = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = manifold.interior_product(&x_flat, &omega, 2).unwrap_err();
    assert!(format!("{err}").contains("requires a metric"));
}
