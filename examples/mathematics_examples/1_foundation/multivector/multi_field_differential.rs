/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{Axis, CausalMultiField, CausalMultiVector, Metric, num_blades};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_usize, lower};

// -----------------------------------------------------------------------------------------
// The geometric derivative on a `CausalMultiField`.
//
// `dx` is the per-axis cell spacing a field is built with, and it is what the four operators
// here integrate against:
//
//   partial_derivative(axis)  central difference, (F[i+1] - F[i-1]) / (2 dx)
//   gradient()                the full geometric derivative, all grades at once
//   divergence()              grade 0 of the gradient
//   curl()                    grade 2 of the gradient
//
// Divergence and curl are therefore two readings of one computation rather than two
// computations. Both fields below have a closed form, so every printed number can be checked
// against it.
//
// A central difference needs a neighbour on each side, so an axis shorter than 3 cells
// differentiates to zero, and the first and last cell of every axis do too.
// -----------------------------------------------------------------------------------------

const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E12: usize = 3;

/// Cl(3, 0).
const N: usize = 3;
/// Cells per axis on the grids below.
const L: usize = 5;

/// The working scalar. Grid spacing and every coefficient carry this type.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const HALF: FloatType = const_scalar_from_float!(FloatType, 0.5);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metric = Metric::Euclidean(N);
    let h = HALF;
    let dx = [h; 3];

    // ---------------------------------------------------------------------
    // A linear field: F(x) = x * e1, on a 5 x 1 x 1 grid.
    // dF/dx = e1 exactly, so div F = 1 and curl F = 0.
    // ---------------------------------------------------------------------
    let cells: Vec<_> = (0..L)
        .map(|i| multivector(&[(E1, lift_usize::<FloatType>(i) * h)], metric))
        .collect::<Result<_, _>>()?;
    let linear = CausalMultiField::from_coefficients(&cells, [L, 1, 1], dx);

    print_linear(
        lower(h),
        &linear.divergence().to_coefficients(),
        &linear.curl().to_coefficients(),
        &linear.gradient().to_coefficients(),
        linear.partial_derivative(Axis::X).shape(),
        linear
            .partial_derivative(Axis::Y)
            .as_slice()
            .iter()
            .all(|&v| lower(v) == 0.0),
    );

    // ---------------------------------------------------------------------
    // A rotational field: F(x, y) = -y*e1 + x*e2, on a 5 x 5 x 1 grid.
    // div F = 0 and curl F = 2*e12, both constant.
    // ---------------------------------------------------------------------
    let mut cells = Vec::with_capacity(L * L);
    for ix in 0..L {
        for iy in 0..L {
            let (x, y) = (
                lift_usize::<FloatType>(ix) * h,
                lift_usize::<FloatType>(iy) * h,
            );
            cells.push(multivector(&[(E1, -y), (E2, x)], metric)?);
        }
    }
    let rot = CausalMultiField::from_coefficients(&cells, [L, L, 1], dx);

    let div = rot.divergence().to_coefficients();
    let curl = rot.curl().to_coefficients();
    print_rotational(&div, &curl);

    // Divergence and curl are one gradient, read at two grades.
    let g = rot.gradient();
    print_one_gradient(
        &div,
        &curl,
        &g.grade_project(0).to_coefficients(),
        &g.grade_project(2).to_coefficients(),
    );

    Ok(())
}

/// Builds a multivector in Cl(N) from `(blade index, coefficient)` pairs.
fn multivector(
    terms: &[(usize, FloatType)],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    let mut coeffs = vec![ZERO; num_blades(N)];
    for &(index, value) in terms {
        coeffs[index] = value;
    }
    Ok(CausalMultiVector::new(coeffs, metric)?)
}

/// One blade of one cell. The display boundary: `f64` appears here and nowhere else.
fn blade(cells: &[CausalMultiVector<FloatType>], cell: usize, index: usize) -> f64 {
    lower(*cells[cell].get(index).expect("blade index within 2^n"))
}

/// Linear index of grid cell `(ix, iy)` on an `L x L x 1` grid, which is row-major in x.
fn at(ix: usize, iy: usize) -> usize {
    ix * L + iy
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_linear(
    h: f64,
    div: &[CausalMultiVector<FloatType>],
    curl: &[CausalMultiVector<FloatType>],
    grad: &[CausalMultiVector<FloatType>],
    d_dx_shape: &[usize],
    d_dy_is_zero: bool,
) {
    println!("=== CausalMultiField: the geometric derivative ===\n");
    println!("--- F(x) = x*e1 on a {L}x1x1 grid, dx = {h} ---");
    println!("  closed form: div F = 1, curl F = 0\n");
    println!("  cell   div F   curl F(e12)   grad F(scalar)");
    for i in 0..L {
        let edge = if i == 0 || i == L - 1 {
            "   <- edge cell"
        } else {
            ""
        };
        println!(
            "   {i}    {:>5}   {:>9}   {:>12}{edge}",
            blade(div, i, SCALAR),
            blade(curl, i, E12),
            blade(grad, i, SCALAR)
        );
    }
    println!(
        "\n  partial_derivative returns the raw matrix-representation tensor: shape {d_dx_shape:?}"
    );
    println!("  d/dy on an axis of length 1 is all zero: {d_dy_is_zero}");
}

fn print_rotational(div: &[CausalMultiVector<FloatType>], curl: &[CausalMultiVector<FloatType>]) {
    println!("\n--- F(x, y) = -y*e1 + x*e2 on a {L}x{L}x1 grid ---");
    println!("  closed form: div F = 0, curl F = 2*e12\n");
    println!("  interior cells only (both axes away from the edge):");
    println!("   (x, y)   div F   curl F(e12)");
    for ix in 1..L - 1 {
        for iy in 1..L - 1 {
            println!(
                "   ({ix}, {iy})   {:>5}   {:>9}",
                blade(div, at(ix, iy), SCALAR),
                blade(curl, at(ix, iy), E12)
            );
        }
    }
}

fn print_one_gradient(
    div: &[CausalMultiVector<FloatType>],
    curl: &[CausalMultiVector<FloatType>],
    grade0: &[CausalMultiVector<FloatType>],
    grade2: &[CausalMultiVector<FloatType>],
) {
    let centre = at(L / 2, L / 2);
    println!("\n--- One gradient, two readings (centre cell) ---");
    println!(
        "  divergence()  == grade_project(0): {}",
        blade(div, centre, SCALAR) == blade(grade0, centre, SCALAR)
    );
    println!(
        "  curl()        == grade_project(2): {}",
        blade(curl, centre, E12) == blade(grade2, centre, E12)
    );
}
