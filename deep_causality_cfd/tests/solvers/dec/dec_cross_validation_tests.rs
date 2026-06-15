/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MMS cross-validation of the two independent Navier–Stokes RHS
//! formulations (`add-dec-solver-foundations` note §6, task 5.1):
//!
//! * **Pointwise oracle**: `incompressible_ns_rhs_kernel` fed with
//!   tangent-functor (dual-number) first derivatives of the analytic
//!   Taylor–Green field, plus the closed-form TG Laplacian `∇²u = −2k²u`.
//! * **DEC operator form**: `−i_u du♭ − ν Δ_dR u♭` assembled from the de Rham
//!   transfer, the wedge-derived interior product, and the Hodge–de Rham
//!   Laplacian on the sampled cochain.
//!
//! The two sides share no discretization code. In Lamb form
//! `−i_u du = −(u·∇)u + ∇(|u|²/2)`, so the DEC side must equal the kernel
//! RHS (at `∇p = 0`, `g = 0`, `ρ = 1`) **plus** the tangent-functor gradient
//! of the kinetic-energy 0-form — an analytic identity that the discrete
//! chain must reproduce at second order under grid refinement. Disagreement
//! localizes defects: conventions (G4), operators (G1), or transfer (G2).

use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_cfd::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
    incompressible_ns_rhs_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.01;

// ---------------------------------------------------------------------------
// Analytic Taylor–Green field as differentiable fields (2D embedded in 3D)
// ---------------------------------------------------------------------------

/// One TG velocity component as a differentiable field of `(x, y)`.
struct TgComponent {
    comp: usize,
    k: f64,
}

impl DifferentiableField<2> for TgComponent {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let k = S::from_f64(self.k).expect("k lifts");
        match self.comp {
            0 => (k * p[0]).sin() * (k * p[1]).cos(),
            _ => S::from_f64(0.0).expect("zero lifts") - (k * p[0]).cos() * (k * p[1]).sin(),
        }
    }
}

/// The kinetic-energy 0-form `|u|²/2` as a differentiable field.
struct TgKineticEnergy {
    k: f64,
}

impl DifferentiableField<2> for TgKineticEnergy {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let u = TgComponent { comp: 0, k: self.k }.run(p);
        let v = TgComponent { comp: 1, k: self.k }.run(p);
        let half = S::from_f64(0.5).expect("half lifts");
        (u * u + v * v) * half
    }
}

fn tg_velocity(k: f64, x: f64, y: f64) -> [f64; 2] {
    [
        (k * x).sin() * (k * y).cos(),
        -(k * x).cos() * (k * y).sin(),
    ]
}

/// Pointwise-oracle RHS component along `axis` at `(x, y)`: the kernel value
/// (with `∇p = 0`, `g = 0`, `ρ = 1`) plus the kinetic-energy gradient (the
/// Lamb-form identity).
fn oracle_rhs_component(k: f64, x: f64, y: f64, axis: usize) -> f64 {
    let u2 = tg_velocity(k, x, y);
    let u = Velocity3::new_unchecked([u2[0], u2[1], 0.0]);

    // ∇u via the tangent functor: row i = gradient of component i.
    let g0 = TgComponent { comp: 0, k }.gradient(&[x, y]);
    let g1 = TgComponent { comp: 1, k }.gradient(&[x, y]);
    let grad_u = VelocityGradient::new_unchecked([
        [g0[0], g0[1], 0.0],
        [g1[0], g1[1], 0.0],
        [0.0, 0.0, 0.0],
    ]);

    // Closed-form TG Laplacian: ∇²u = −2k²·u (each component is a product of
    // single-mode sines/cosines in x and y).
    let lap = [-2.0 * k * k * u2[0], -2.0 * k * k * u2[1], 0.0];

    let rhs = incompressible_ns_rhs_kernel(
        &u,
        &grad_u,
        &lap,
        &[0.0, 0.0, 0.0],
        &Density::new_unchecked(1.0),
        &KinematicViscosity::new_unchecked(NU),
        &AccelerationVector::new_unchecked([0.0, 0.0, 0.0]),
    )
    .expect("kernel evaluation with ρ = 1 cannot fail");

    // + ∇(|u|²/2) via the tangent functor.
    let grad_ke = TgKineticEnergy { k }.gradient(&[x, y]);
    rhs.value()[axis] + grad_ke[axis]
}

// ---------------------------------------------------------------------------
// DEC side
// ---------------------------------------------------------------------------

fn unit_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|g| lattice.num_cells(g)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn manifold_with_one_form(n: usize, form: &[f64]) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|g| lattice.num_cells(g)).sum();
    let n0 = lattice.num_cells(0);
    let mut data = vec![0.0; total];
    data[n0..n0 + form.len()].copy_from_slice(form);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, tensor, metric, 0)
}

// ---------------------------------------------------------------------------
// The cross-validation
// ---------------------------------------------------------------------------

#[test]
fn dec_rhs_cross_validates_against_pointwise_kernel_at_second_order() {
    let mut rel_errors = Vec::new();

    for n in [8usize, 16, 32] {
        let k = 2.0 * std::f64::consts::PI / (n as f64);
        let manifold = unit_manifold(n);
        let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
        let n0 = lattice.num_cells(0);

        // Vertex-sampled TG velocity → edge 1-form via the de Rham map.
        let mut vertex_field = vec![0.0; 2 * n0];
        for (vi, v) in lattice.iter_cells(0).enumerate() {
            let u2 = tg_velocity(k, v.position()[0] as f64, v.position()[1] as f64);
            vertex_field[2 * vi] = u2[0];
            vertex_field[2 * vi + 1] = u2[1];
        }
        let vertex_tensor = CausalTensor::new(vertex_field, vec![2 * n0]).unwrap();
        let u_flat = manifold.de_rham(&vertex_tensor).unwrap();

        // DEC RHS = −i_u du♭ − ν Δ_dR u♭.
        let m_u = manifold_with_one_form(n, u_flat.as_slice());
        let du = m_u.exterior_derivative(1);
        let conv = manifold.interior_product(&u_flat, &du, 2).unwrap();
        let lap = m_u.laplacian(1);

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

            let dec = -conv.as_slice()[i] - NU * lap.as_slice()[i];
            let oracle = oracle_rhs_component(k, mx, my, axis);

            max_err = max_err.max((dec - oracle).abs());
            max_ref = max_ref.max(oracle.abs());
        }
        rel_errors.push(max_err / max_ref);
    }

    assert!(
        rel_errors[1] < rel_errors[0] / 3.0,
        "DEC vs pointwise oracle not second order (first refinement): {rel_errors:?}"
    );
    assert!(
        rel_errors[2] < rel_errors[1] / 3.0,
        "DEC vs pointwise oracle not second order (second refinement): {rel_errors:?}"
    );
}
