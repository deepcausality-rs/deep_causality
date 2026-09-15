/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric, matrix_dim, num_blades};
use deep_causality_num::{lift, lower};
use deep_causality_tensor::{CausalTensor, Tensor};

// -----------------------------------------------------------------------------------------
// Every Clifford algebra Cl(p, q) is isomorphic to a matrix algebra, and this crate uses that
// isomorphism as its representation.
//
//   to_matrix()             the D x D matrix of a multivector, D = matrix_dim(n)
//   from_matrix(m, metric)  back to blade coefficients, by trace projection
//   get_gamma_matrix(i)     the generator matrix for blade i
//
// The law that makes the representation useful is that it is a homomorphism: the geometric
// product of two multivectors is the ordinary matrix product of their representations. That is
// checked below, and it is why `CausalMultiField` stores matrices rather than blades -- a field
// product becomes one batched matrix multiply.
// -----------------------------------------------------------------------------------------

const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E12: usize = 3;

/// Cl(3, 0).
const N: usize = 3;

/// The working scalar. Blade coefficients and matrix entries carry this type.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metric = Metric::Euclidean(N);

    let e1 = multivector(&[(E1, lift(1.0))], metric)?;
    let e2 = multivector(&[(E2, lift(1.0))], metric)?;

    // A multivector as a matrix, and the generator it coincides with.
    print_single(&e1.to_matrix(), &e1.get_gamma_matrix(E1));

    // The homomorphism: geometric product == matrix product.
    let product = e1.clone() * e2.clone();
    print_homomorphism(
        &product,
        &product.to_matrix(),
        &e1.to_matrix().matmul(&e2.to_matrix())?,
    );

    // from_matrix recovers coefficients by trace projection, a_i = Tr(M * G_i^dagger) / D.
    let mixed = multivector(
        &[(SCALAR, lift(2.0)), (E1, lift(-1.0)), (E12, lift(0.5))],
        metric,
    )?;
    let recovered = CausalMultiVector::from_matrix(mixed.to_matrix(), metric);
    print_round_trip(&mixed, &recovered);

    // Non-commutativity survives the representation.
    let reversed = e2 * e1;
    print_non_commutativity(
        &product,
        &reversed,
        &product.to_matrix(),
        &reversed.to_matrix(),
    );

    Ok(())
}

/// Builds a multivector in Cl(N) from `(blade index, coefficient)` pairs.
fn multivector(
    terms: &[(usize, FloatType)],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    let mut coeffs = vec![lift::<FloatType>(0.0); num_blades(N)];
    for &(index, value) in terms {
        coeffs[index] = value;
    }
    Ok(CausalMultiVector::new(coeffs, metric)?)
}

/// One blade coefficient. The display boundary: `f64` appears here and nowhere else.
fn blade(mv: &CausalMultiVector<FloatType>, index: usize) -> f64 {
    lower(*mv.get(index).expect("blade index within 2^n"))
}

fn same(a: &CausalTensor<FloatType>, b: &CausalTensor<FloatType>) -> bool {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .all(|(&x, &y)| lower(x) == lower(y))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_single(m1: &CausalTensor<FloatType>, gamma1: &CausalTensor<FloatType>) {
    println!("=== Multivector matrix representation ===\n");
    println!("--- Cl(3, 0) ---");
    println!("  blades      : {}  (2^{N})", num_blades(N));
    println!(
        "  matrix_dim  : {}  (so every multivector is a {}x{} matrix)",
        matrix_dim(N),
        matrix_dim(N),
        matrix_dim(N)
    );

    println!("\n--- e1.to_matrix() ---");
    println!("  shape: {:?}", m1.shape());
    print_matrix(m1);
    println!(
        "  get_gamma_matrix({E1}) equals e1.to_matrix(): {}",
        same(m1, gamma1)
    );
}

fn print_homomorphism(
    product: &CausalMultiVector<FloatType>,
    product_matrix: &CausalTensor<FloatType>,
    matrix_product: &CausalTensor<FloatType>,
) {
    println!("\n--- (e1 * e2).to_matrix()  vs  e1.to_matrix() * e2.to_matrix() ---");
    print_matrix(product_matrix);
    println!("  equal: {}", same(product_matrix, matrix_product));
    println!(
        "  (e1*e2) is the bivector e12: coefficient at index {E12} = {}",
        blade(product, E12)
    );
}

fn print_round_trip(
    mixed: &CausalMultiVector<FloatType>,
    recovered: &CausalMultiVector<FloatType>,
) {
    println!("\n--- Round trip of 2 - e1 + 0.5*e12 ---");
    println!("  blade  original  recovered");
    for i in 0..num_blades(N) {
        let (a, b) = (blade(mixed, i), blade(recovered, i));
        if a != 0.0 || b != 0.0 {
            println!("   {i:>4}  {a:>8}  {b:>9}");
        }
    }
    println!(
        "  every blade agrees: {}",
        (0..num_blades(N)).all(|i| blade(mixed, i) == blade(recovered, i))
    );
}

fn print_non_commutativity(
    product: &CausalMultiVector<FloatType>,
    reversed: &CausalMultiVector<FloatType>,
    e1e2: &CausalTensor<FloatType>,
    e2e1: &CausalTensor<FloatType>,
) {
    println!("\n--- e1*e2 and e2*e1 ---");
    println!("  matrices equal: {}", same(e1e2, e2e1));
    println!(
        "  e12 coefficients: e1*e2 = {}, e2*e1 = {}",
        blade(product, E12),
        blade(reversed, E12)
    );
}

fn print_matrix(m: &CausalTensor<FloatType>) {
    let (s, d) = (m.as_slice(), matrix_dim(N));
    for row in 0..d {
        let cells: Vec<String> = (0..d)
            .map(|col| format!("{:>5}", lower(s[row * d + col])))
            .collect();
        println!("    [{}]", cells.join(" "));
    }
}
