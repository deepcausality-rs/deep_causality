/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Curvature, and the manifold that carries it
//!
//! Curvature is what a vector notices when it is carried around a closed loop and comes back
//! pointing somewhere else. The Riemann tensor is the bookkeeping for that: `R(v, w)u` is the
//! change in `u` after a trip around the loop spanned by `v` and `w`.
//!
//! Two witnesses meet here:
//!
//! ```text
//! CurvatureTensorWitness<T>    RiemannMap                 curvature(R, u, v, w)
//! GenericManifoldWitness<K>    Manifold<K, T>, any K      HKT, Functor
//! ```
//!
//! `CurvatureTensorWitness` carries `RiemannMap`, whose vector space is an associated type on the
//! witness. Handing it the wrong container or the wrong scalar is a compile error.
//!
//! `GenericManifoldWitness<K>` is the functor over a manifold on **any** cellular complex.
//! `ManifoldWitness` pins the complex to `SimplicialComplex` and gains `Monad` and `CoMonad` for
//! doing so; this one takes a cubical lattice, a cell complex or a simplicial complex alike, and
//! offers the operations that need nothing from the cells.
//!
//! The space is a 3-sphere of radius `a`, which has constant sectional curvature `K = 1/a²`. Every
//! number the example prints has a closed form beside it:
//!
//! ```text
//! R^d_abc      = K (δ^d_b g_ac − δ^d_c g_ab)
//! R(v, w)u     = K (⟨u, w⟩ v − ⟨u, v⟩ w)
//! Ricci scalar = n (n − 1) K
//! ```

use deep_causality_algebra::Real;
use deep_causality_haft::{Functor, RiemannMap};
use deep_causality_metric::Metric;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lift_count, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorWitness, GenericManifoldWitness,
    LatticeComplex, Manifold, TensorVector,
};

/// The 3-sphere the example works on: three dimensions, radius `a`.
const DIM: usize = 3;
const SPHERE_RADIUS: FloatType = const_scalar_from_int!(FloatType, 2);

/// The region is discretized as a cubical lattice of this shape.
const LATTICE_SHAPE: [usize; DIM] = [2, 2, 2];

/// A test mass this far apart, in metres, feels the tidal acceleration section 4 reports.
const SEPARATION: FloatType = const_scalar_from_int!(FloatType, 1000);

/// What counts as zero when a floating-point identity is checked.
const TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// The working scalar. The curvature components, the vectors and the field all carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let radius = SPHERE_RADIUS;
    let sectional = ONE / (radius * radius);
    print_header(radius, sectional);

    // ---------------------------------------------------------------------
    // 1. The curvature tensor of a space of constant curvature.
    // ---------------------------------------------------------------------
    let riemann = constant_curvature(sectional);
    let flat = CurvatureTensor::<FloatType>::flat(DIM);
    print_tensors(riemann.dim(), riemann.is_flat(), flat.is_flat());

    // ---------------------------------------------------------------------
    // 2. RiemannMap: carry a vector around a loop.
    // ---------------------------------------------------------------------
    // The first argument is the vector being carried; the second and third span the loop. On a
    // sphere, carrying e0 around the (e0, e1) loop turns it toward −e1 by the sectional curvature.
    let e0 = TensorVector::<FloatType>::basis(DIM, 0);
    let e1 = TensorVector::<FloatType>::basis(DIM, 1);

    let carried = CurvatureTensorWitness::<FloatType>::curvature(&riemann, &e0, &e0, &e1);
    let reversed = CurvatureTensorWitness::<FloatType>::curvature(&riemann, &e0, &e1, &e0);
    let closed_form = closed_form_curvature(sectional, e0.as_slice(), e0.as_slice(), e1.as_slice());
    print_transport(carried.as_slice(), reversed.as_slice(), &closed_form);

    // The closed form and the tensor contraction agree component by component.
    for (got, want) in carried.as_slice().iter().zip(closed_form.iter()) {
        assert!(Real::abs(*got - *want) < TOLERANCE);
    }
    // Reversing the loop reverses the result: R is antisymmetric in the two slots spanning it.
    for (forward, back) in carried.as_slice().iter().zip(reversed.as_slice().iter()) {
        assert!(Real::abs(*forward + *back) < TOLERANCE);
    }

    // A flat space returns the vector unchanged, so the curvature operator gives zero.
    let in_flat = CurvatureTensorWitness::<FloatType>::curvature(&flat, &e0, &e0, &e1);
    assert!(in_flat.as_slice().iter().all(|x| *x == ZERO));

    // ---------------------------------------------------------------------
    // 3. The invariants the tensor contracts to.
    // ---------------------------------------------------------------------
    let ricci = riemann.ricci_scalar();
    let expected = lift_count::<FloatType>((DIM * (DIM - 1)) as u64) * sectional;
    let bianchi = riemann.check_bianchi_identity();
    print_invariants(ricci, expected, bianchi);

    assert!(Real::abs(ricci - expected) < TOLERANCE);

    // ---------------------------------------------------------------------
    // 4. GenericManifoldWitness: the field over a cubical lattice.
    // ---------------------------------------------------------------------
    // The region is a cubical complex, so `ManifoldWitness` does not reach it: that witness pins
    // the complex to `SimplicialComplex`. `GenericManifoldWitness<K>` takes any cellular complex,
    // and `fmap` moves the cell data while the lattice travels across untouched.
    let cells: usize = LATTICE_SHAPE.iter().product();
    let lattice = LatticeComplex::<DIM, FloatType>::new(LATTICE_SHAPE, [false; DIM]);
    let field = Manifold::from_cubical(
        lattice,
        CausalTensor::new(vec![ricci; cells], vec![cells])?,
        0,
    );

    // Geodesic deviation: two test masses separated by `d` in a space of sectional curvature K
    // drift apart at `K·d`, which is the tidal acceleration the curvature produces.
    let separation = SEPARATION;
    let tidal = GenericManifoldWitness::<LatticeComplex<DIM, FloatType>>::fmap(field, move |r| {
        r / lift_count::<FloatType>((DIM * (DIM - 1)) as u64) * separation
    });

    print_field(cells, tidal.data().as_slice(), separation);

    // Every cell holds K·d, because the curvature is constant across the region.
    let expected_tidal = sectional * separation;
    for &value in tidal.data().as_slice() {
        assert!(Real::abs(value - expected_tidal) < TOLERANCE);
    }

    print_footer();
    Ok(())
}

/// The Riemann tensor of a space of constant sectional curvature `K` in a Euclidean frame:
/// `R^d_abc = K (δ^d_b δ_ac − δ^d_c δ_ab)`.
fn constant_curvature(sectional: FloatType) -> CurvatureTensor<FloatType> {
    let delta = |i: usize, j: usize| if i == j { 1.0 } else { 0.0 };

    CurvatureTensor::from_generator(
        DIM,
        Metric::Euclidean(DIM),
        CurvatureSymmetry::Riemann,
        |d, a, b, c| {
            sectional * lift::<FloatType>(delta(d, b) * delta(a, c) - delta(d, c) * delta(a, b))
        },
    )
}

/// `R(v, w)u = K (⟨u, w⟩ v − ⟨u, v⟩ w)`, the closed form the contraction is checked against.
fn closed_form_curvature(
    sectional: FloatType,
    u: &[FloatType],
    v: &[FloatType],
    w: &[FloatType],
) -> Vec<FloatType> {
    let uw = dot(u, w);
    let uv = dot(u, v);

    (0..DIM)
        .map(|d| sectional * (uw * v[d] - uv * w[d]))
        .collect()
}

/// The Euclidean inner product of two vectors.
fn dot(a: &[FloatType], b: &[FloatType]) -> FloatType {
    a.iter()
        .zip(b.iter())
        .fold(ZERO, |acc, (&x, &y)| acc + x * y)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header(radius: FloatType, sectional: FloatType) {
    println!("=== Curvature, and the manifold that carries it ===\n");
    println!(
        "  A {DIM}-sphere of radius a = {:.1}, so the sectional curvature is K = 1/a^2 = {:.4}.\n",
        lower(radius),
        lower(sectional)
    );
}

fn print_tensors(dim: usize, curved_is_flat: bool, flat_is_flat: bool) {
    println!("--- 1. Two curvature tensors ---");
    println!("  dimension                {dim}");
    println!("  constant-curvature R     is_flat = {curved_is_flat}");
    println!("  CurvatureTensor::flat    is_flat = {flat_is_flat}");
}

fn print_transport(carried: &[FloatType], reversed: &[FloatType], closed_form: &[FloatType]) {
    println!("\n--- 2. RiemannMap: carry e0 around the (e0, e1) loop ---");
    println!("  R(e0, e1) e0    contraction  {:?}", to_four(carried));
    println!("                  closed form  {:?}", to_four(closed_form));
    println!("  R(e1, e0) e0    contraction  {:?}", to_four(reversed));
    println!();
    println!("  Reversing the loop reverses the result, which is the antisymmetry of R in");
    println!("  the two slots that span it. A flat tensor returns zero for every loop.");
}

fn print_invariants(ricci: FloatType, expected: FloatType, bianchi: FloatType) {
    println!("\n--- 3. The invariants ---");
    println!("  Ricci scalar             {:.6}", lower(ricci));
    println!("  n(n-1)K, the closed form {:.6}", lower(expected));
    println!("  first Bianchi identity   {:.2e}", lower(bianchi));
}

fn print_field(cells: usize, tidal: &[FloatType], separation: FloatType) {
    println!("\n--- 4. GenericManifoldWitness over a cubical lattice ---");
    println!(
        "  Manifold<LatticeComplex<{DIM}, f64>, f64>   {cells} cells, shape {LATTICE_SHAPE:?}"
    );
    println!("  fmap carried the cell data and left the lattice in place.");
    println!(
        "\n  tidal acceleration at {:.0} m separation: {:?} (K*d)",
        lower(separation),
        to_four(tidal)
    );
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  `RiemannMap` fixes its vector space as an associated type on the witness, so");
    println!("  the curvature operator takes the one container it is defined over and the");
    println!("  compiler rejects the rest. `GenericManifoldWitness` goes the other way: it");
    println!("  asks nothing of the complex, so it serves cubical, cellular and simplicial");
    println!("  manifolds with the operations that read only the cell data.");
}

fn to_four(values: &[FloatType]) -> Vec<f64> {
    values
        .iter()
        .map(|&v| (lower(v) * 10_000.0).round() / 10_000.0)
        .collect()
}
