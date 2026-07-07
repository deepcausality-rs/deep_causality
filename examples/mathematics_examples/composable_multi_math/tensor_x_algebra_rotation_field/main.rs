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
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// `f64` is the right precision here: the rotation result is a permutation of
/// integer-valued components (`e1 -> e2`), so Float106 yields no observable
/// gain.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tensor x Algebra: Discrete Vector Field Rotation ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    let metric = Metric::Euclidean(2);
    let theta = FloatType::pi() / FloatType::from(2.0);
    let (rotor, rotor_rev) = build_rotor_pair(theta, metric);

    // 3x3 grid; every cell holds the unit vector e1.
    let grid_shape = [3usize, 3];
    let field: CausalTensor<CausalMultiVector<FloatType>> =
        CausalTensor::from_shape_fn(&grid_shape, |_idx| unit_x(metric));

    println!("Before rotation (cell [0,0]):");
    print_vector_cell(field.get(&[0, 0]).unwrap());

    // The outer Functor walks the tensor; the inner closure applies R v R~.
    let rotated: CausalTensor<CausalMultiVector<FloatType>> =
        CausalTensorWitness::fmap(field, |v| {
            rotor.geometric_product(&v).geometric_product(&rotor_rev)
        });

    println!("\nAfter 90-degree rotation in e1^e2 (cell [0,0]):");
    print_vector_cell(rotated.get(&[0, 0]).unwrap());

    println!("\nAfter rotation (cell [2,2]):");
    print_vector_cell(rotated.get(&[2, 2]).unwrap());

    println!(
        "\nEvery one of the {} cells went through the same rotor pair",
        9
    );
    println!("via a single `CausalTensorWitness::fmap` call.");

    Ok(())
}

/// 90-degree rotor in the `e1^e2` plane of `Cl(2,0)`.
/// `R = cos(theta/2) - sin(theta/2) e12`, reverse `R~ = cos(theta/2) + sin(theta/2) e12`.
fn build_rotor_pair(
    theta: FloatType,
    metric: Metric,
) -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
    let half = theta / FloatType::from(2.0);
    let c = half.cos();
    let s = half.sin();
    let zero = FloatType::from(0.0);
    // Cl(2,0) coefficient order: [1, e1, e2, e12]
    let r = CausalMultiVector::new(vec![c, zero, zero, -s], metric).unwrap();
    let r_rev = CausalMultiVector::new(vec![c, zero, zero, s], metric).unwrap();
    (r, r_rev)
}

fn unit_x(metric: Metric) -> CausalMultiVector<FloatType> {
    let zero = FloatType::from(0.0);
    let one = FloatType::from(1.0);
    CausalMultiVector::new(vec![zero, one, zero, zero], metric).unwrap()
}

fn print_vector_cell(v: &CausalMultiVector<FloatType>) {
    let d = v.data();
    println!(
        "  scalar = {:.4}, e1 = {:.4}, e2 = {:.4}, e12 = {:.4}",
        d[0], d[1], d[2], d[3]
    );
}
