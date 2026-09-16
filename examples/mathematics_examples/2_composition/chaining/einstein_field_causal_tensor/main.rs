/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Einstein Field Equations through a Kleisli chain
//!
//! `G_uv + Λ g_uv = κ T_uv`, solved for `T_uv`, with every step taken through the witness:
//! `pure` lifts a scalar into the tensor context, `fmap` scales, `apply` broadcasts a lifted
//! function, `extend` reads each cell's neighbourhood, and `bind` lets the result's shape
//! depend on the values it carries.

use deep_causality_algebra::Real;
use deep_causality_haft::{Applicative, CoMonad, Functor, Monad, Pure};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::CausalTensorWitness;

/// The working scalar. Every tensor component below carries it.
pub type FloatType = f64;

const NEG_HALF: FloatType = const_scalar_from_float!(FloatType, -0.5);

/// Small numbers and tolerances, at the working type.
const TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-5);

/// Small numbers, declared once at the working type rather than lifted at each use.
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const EIGHT: FloatType = const_scalar_from_int!(FloatType, 8);

fn main() {
    print_header();

    // Normalized units, G = c = 1, so kappa = 8 pi.
    let kappa = EIGHT * FloatType::pi();
    // Cosmological constant: a small positive value, for accelerating expansion.
    let lambda = TOLERANCE;
    print_constants(kappa, lambda);

    // 1. The metric tensor g_uv: a Minkowski signature (- + + +), slightly perturbed so the
    //    space is not simply flat.
    #[rustfmt::skip]
    let g_uv = tensor(&[
        -1.0, 0.01, 0.0, 0.0, // t
         0.01, 1.0, 0.0, 0.0, // x
         0.0,  0.0, 1.0, 0.0, // y
         0.0,  0.0, 0.0, 1.0, // z
    ]);
    print_tensor("Metric Tensor (g_uv)", &g_uv);

    // 2. The Ricci tensor R_uv, taken as given: deriving it from the metric goes through
    //    the Christoffel symbols, which is a different example.
    #[rustfmt::skip]
    let r_uv = tensor(&[
        0.1,   0.005, 0.0,  0.0,  // t
        0.005, 0.05,  0.0,  0.0,  // x
        0.0,   0.0,   0.05, 0.0,  // y
        0.0,   0.0,   0.0,  0.05, // z
    ]);
    print_tensor("Ricci Tensor (R_uv)", &r_uv);

    // 3. Scalar curvature R, lifted into the tensor context by `pure`.
    let scalar_curvature_val = lift::<FloatType>(0.05);
    let r_scalar = CausalTensorWitness::pure(scalar_curvature_val);
    print_curvature(scalar_curvature_val);

    // 4. G_uv = R_uv - 0.5 R g_uv, element-wise, with no manual loop.
    print_einstein_intro();

    // `fmap` transforms the value inside the context.
    let neg_half_r =
        <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(r_scalar, |r| NEG_HALF * r);
    let scalar_val = neg_half_r.data()[0];

    // `apply` broadcasts a lifted function across the data tensor.
    let mul_by_scalar_fn = move |x: FloatType| x * scalar_val;
    let fn_tensor = CausalTensorWitness::pure(mul_by_scalar_fn);
    let term_2 = CausalTensorWitness::apply(fn_tensor, g_uv.clone());

    let g_tensor = &r_uv + &term_2;
    print_tensor("Einstein Tensor (G_uv)", &g_tensor);

    // 5. The cosmological term, scaled the same way.
    print_cosmological_intro();
    let lambda_fn = move |x: FloatType| x * lambda;
    let lambda_fn_tensor = CausalTensorWitness::pure(lambda_fn);
    let lambda_term = <CausalTensorWitness as Applicative<CausalTensorWitness>>::apply(
        lambda_fn_tensor,
        g_uv.clone(),
    );
    let lhs = &g_tensor + &lambda_term;
    print_tensor("LHS (G_uv + Λ * g_uv)", &lhs);

    // 6. T_uv = LHS / kappa, one `fmap` over the whole structure.
    print_stress_intro();
    let t_uv = <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(lhs, move |x| x / kappa);
    print_tensor("Stress-Energy Tensor (T_uv) Result", &t_uv);
    print_verification(t_uv.shape());
    assert_eq!(t_uv.shape(), &[4, 4]);

    // 7. CoMonad: each cell's new value depends on its neighbourhood, which is what a
    //    gradient, a smoothness measure or an anomaly detector needs.
    print_comonad_intro();
    let anomaly_map = CausalTensorWitness::extend(&t_uv, |view| {
        let data = view.data();
        let center = data[0];
        let neighbor = if data.len() > 1 { data[1] } else { center };
        Real::abs(center - neighbor)
    });
    print_tensor("Anomaly Map (Local Gradients)", &anomaly_map);

    // 8. Monad: `bind` lets the *shape* of the result depend on the values. Quantizing the
    //    energy splits every high component in two, so the tensor grows.
    print_monad_intro();
    let threshold = lift::<FloatType>(0.001);
    let half = TWO;
    let quantized_energy =
        <CausalTensorWitness as Monad<CausalTensorWitness>>::bind(t_uv.clone(), move |val| {
            if val > threshold {
                CausalTensor::new(vec![val / half, val / half], vec![2])
                    .expect("two elements in a rank-1 shape of 2")
            } else {
                CausalTensor::new(vec![val], vec![1]).expect("one element in a rank-1 shape of 1")
            }
        });
    print_quantized(t_uv.len(), &quantized_energy);
}

/// A 4x4 tensor from row-major literals, lifted into the working scalar.
fn tensor(values: &[f64]) -> CausalTensor<FloatType> {
    let data: Vec<FloatType> = values.iter().map(|&x| lift(x)).collect();
    CausalTensor::new(data, vec![4, 4]).expect("sixteen components in a 4x4 shape")
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

pub(crate) fn print_header() {
    println!("============================================================");
    println!("   Einstein Field Equations with CausalTensor & HKT");
    println!("============================================================");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
    println!("This example demonstrates solving the Einstein Field Equations (EFE):");
    println!("  G_uv + Λ * g_uv = κ * T_uv");
    println!();
    println!("Where:");
    println!("  G_uv = R_uv - 0.5 * R * g_uv  (Einstein Tensor)");
    println!("  R_uv                          (Ricci Curvature Tensor)");
    println!("  R                             (Scalar Curvature)");
    println!("  g_uv                          (Metric Tensor)");
    println!("  Λ                             (Cosmological Constant)");
    println!("  κ                             (Einstein Constant)");
    println!("  T_uv                          (Stress-Energy Tensor)");
    println!();
    println!("Value of HKT (Higher-Kinded Types) in this context:");
    println!("1. Abstraction: We treat tensors as abstract contexts (Functors/Applicatives).");
    println!("2. Safety: Operations are lifted into the context, handling shapes implicitly.");
    println!("3. Composability: We chain operations (map, apply) without manual loops.");
    println!("4. Clarity: The code mirrors the mathematical equation structure.");
    println!("============================================================");
    println!();
}

fn print_constants(kappa: FloatType, lambda: FloatType) {
    println!("Constants:");
    println!("  Kappa (κ)  : {:.4}", lower(kappa));
    println!("  Lambda (Λ) : {:.4}", lower(lambda));
    println!("--------------------------------------------------");
}

fn print_curvature(r: FloatType) {
    println!("Scalar Curvature (R): {:.4}", lower(r));
    println!("--------------------------------------------------");
}

fn print_einstein_intro() {
    println!("Calculating Einstein Tensor G_uv...");
    println!("Formula: G_uv = R_uv + (-0.5 * R * g_uv)");
}

fn print_cosmological_intro() {
    println!("Adding Cosmological Term...");
    println!("Formula: LHS = G_uv + (Λ * g_uv)");
}

fn print_stress_intro() {
    println!("Solving for Stress-Energy Tensor T_uv...");
    println!("Formula: T_uv = LHS / κ");
    println!("--------------------------------------------------");
}

fn print_verification(shape: &[usize]) {
    println!("--------------------------------------------------");
    println!("Verification:");
    println!("Shape of T_uv: {shape:?}");
    println!("Calculation completed successfully.");
}

fn print_comonad_intro() {
    println!("--------------------------------------------------");
    println!("7. Advanced HKT: CoMonad for Local Field Analysis");
    println!("   CoMonad `extend` allows us to perform operations where each element's");
    println!("   new value depends on its 'neighborhood' (context).");
    println!(
        "   This is ideal for calculating field gradients, smoothness, or detecting anomalies."
    );
}

fn print_monad_intro() {
    println!("--------------------------------------------------");
    println!("8. Advanced HKT: Monad for Dependent Computation");
    println!("   Monad `bind` allows us to chain operations where the structure of the result");
    println!("   depends on the input values. Here we 'quantize' the energy:");
    println!(
        "   If energy > threshold, we split it into multiple 'quanta' (expanding the tensor)."
    );
}

fn print_quantized(original_len: usize, quantized: &CausalTensor<FloatType>) {
    println!("Quantized Energy Tensor (Flattened):");
    println!("Original Size: {original_len}");
    println!("New Size:      {}", quantized.len());
    println!("(Notice the size increase due to splitting high-energy components)");
    // Now a flat rank-1 tensor of quanta, so the 2D layout does not apply.
    println!("{quantized:?}");
}

/// Pretty-prints a 2D tensor. The display boundary: `f64` appears here and nowhere else.
pub(crate) fn print_tensor(name: &str, tensor: &CausalTensor<FloatType>) {
    println!("{name}:");
    let shape = tensor.shape();
    if shape.len() != 2 {
        println!("{tensor:?}");
        return;
    }
    let (rows, cols) = (shape[0], shape[1]);
    let data = tensor.data();

    println!("[");
    for i in 0..rows {
        print!("  [");
        for j in 0..cols {
            let val = lower(data[i * cols + j]);
            if val.abs() < 1e-10 {
                print!("{:>10.4}", 0.0);
            } else {
                print!("{val:>10.4}");
            }
            if j < cols - 1 {
                print!(", ");
            }
        }
        println!("  ],");
    }
    println!("]");
    println!();
}
