/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor x Algebra: Rotating a Discrete Vector Field
//!
//! A `CausalTensor` whose cells are `CausalMultiVector` values forms a discrete
//! vector field. The `Functor` impl on `CausalTensorWitness` walks every cell;
//! the cell-level operation is a geometric product `R v R~` from `Cl(2,0)`.
//!
//! Two HKT layers stacked, one uniform API.
//!
//! ## APIs Demonstrated
//! - `CausalTensor::from_shape_fn`
//! - `CausalTensorWitness::fmap` (Functor over the outer container)
//! - `CausalMultiVector::new`, `geometric_product` (Clifford algebra)

use deep_causality_algebra::Real;
use deep_causality_haft::Functor;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{const_scalar_from_int, lower};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// Cells per axis of the field.
const L: usize = 3;

/// `f64` is the right precision here: the rotation result is a permutation of
/// integer-valued components (`e1 -> e2`), so Float106 yields no observable
/// gain.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let metric = Metric::Euclidean(2);
    let theta = FloatType::pi() / TWO;
    let (rotor, rotor_rev) = build_rotor_pair(theta, metric);

    // An L x L grid; every cell holds the unit vector e1.
    let field: CausalTensor<CausalMultiVector<FloatType>> =
        CausalTensor::from_shape_fn(&[L, L], |_idx| unit_x(metric));
    print_before(field.get(&[0, 0]).expect("cell [0,0] is in an LxL grid"));

    // The outer Functor walks the tensor; the inner closure applies R v R~.
    let rotated: CausalTensor<CausalMultiVector<FloatType>> =
        CausalTensorWitness::fmap(field, |v| {
            rotor.geometric_product(&v).geometric_product(&rotor_rev)
        });

    print_after(
        rotated.get(&[0, 0]).expect("cell [0,0] is in an LxL grid"),
        rotated
            .get(&[L - 1, L - 1])
            .expect("the far corner is in an LxL grid"),
    );

    Ok(())
}

/// 90-degree rotor in the `e1^e2` plane of `Cl(2,0)`.
/// `R = cos(theta/2) - sin(theta/2) e12`, reverse `R~ = cos(theta/2) + sin(theta/2) e12`.
fn build_rotor_pair(
    theta: FloatType,
    metric: Metric,
) -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
    let half = theta / TWO;
    let c = half.cos();
    let s = half.sin();
    let zero = ZERO;
    // Cl(2,0) coefficient order: [1, e1, e2, e12]
    let r = CausalMultiVector::new(vec![c, zero, zero, -s], metric)
        .expect("four coefficients is exactly 2^2");
    let r_rev = CausalMultiVector::new(vec![c, zero, zero, s], metric)
        .expect("four coefficients is exactly 2^2");
    (r, r_rev)
}

fn unit_x(metric: Metric) -> CausalMultiVector<FloatType> {
    let zero = ZERO;
    let one = ONE;
    CausalMultiVector::new(vec![zero, one, zero, zero], metric)
        .expect("four coefficients is exactly 2^2")
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Tensor x Algebra: Discrete Vector Field Rotation ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_before(cell: &CausalMultiVector<FloatType>) {
    println!("Before rotation (cell [0,0]):");
    print_vector_cell(cell);
}

fn print_after(origin: &CausalMultiVector<FloatType>, corner: &CausalMultiVector<FloatType>) {
    println!("\nAfter 90-degree rotation in e1^e2 (cell [0,0]):");
    print_vector_cell(origin);
    println!("\nAfter rotation (cell [{},{}]):", L - 1, L - 1);
    print_vector_cell(corner);
    println!(
        "\nEvery one of the {} cells went through the same rotor pair",
        L * L
    );
    println!("via a single `CausalTensorWitness::fmap` call.");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_vector_cell(v: &CausalMultiVector<FloatType>) {
    let d = v.data();
    println!(
        "  scalar = {:.4}, e1 = {:.4}, e2 = {:.4}, e12 = {:.4}",
        lower(d[0]),
        lower(d[1]),
        lower(d[2]),
        lower(d[3])
    );
}
