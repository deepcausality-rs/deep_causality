/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{
    CausalMultiField, CausalMultiVector, Metric, matrix_dim, num_blades,
};
use deep_causality_num::{lift, lift_usize, lower};

// -----------------------------------------------------------------------------------------
// `CausalMultiField` is one multivector per grid cell: a geometric-algebra field.
//
// The storage is the part worth knowing first. A `CausalMultiVector` in Cl(n) has 2^n blade
// coefficients, but a `CausalMultiField` does not store blades. It stores the *matrix
// isomorphism* of each multivector, shaped `[Nx, Ny, Nz, D, D]` where `D = matrix_dim(n)`.
// That is what makes a field product a batched matrix multiply instead of a blade-by-blade
// sum. Blades come back out through `to_coefficients`.
// -----------------------------------------------------------------------------------------

/// Blade indices in Cl(3, 0), confirmed by the `e1 * e2` product printed below.
const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E12: usize = 3;
const E23: usize = 6;

/// Cl(3, 0): three positive basis vectors.
const N: usize = 3;

/// The working scalar. Every coefficient in the field carries this type.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metric = Metric::Euclidean(N);
    let shape = [2, 2, 1];
    let dx = [lift::<FloatType>(0.5); 3];

    // Construction, and the storage that construction commits to.
    let ones = CausalMultiField::<FloatType>::ones(shape, metric, dx);
    print_storage(&ones);

    // A field built from individual multivectors: cell k holds k*e1 + e2.
    let cells: Vec<_> = (0..4)
        .map(|k| multivector(&[(E1, lift_usize(k)), (E2, lift(1.0))], metric))
        .collect::<Result<_, _>>()?;
    let field = CausalMultiField::from_coefficients(&cells, shape, dx);
    let f = field.to_coefficients();
    print_cells(&f);

    // The blade indices used above, checked rather than assumed.
    let e1 = multivector(&[(E1, lift(1.0))], metric)?;
    let e2 = multivector(&[(E2, lift(1.0))], metric)?;
    print_blade_check(&(e1 * e2));

    // <F>_k keeps the blades of grade k and zeroes the rest.
    print_grades(
        &f,
        &field.vector_part().to_coefficients(),
        &field.scalar_part().to_coefficients(),
        &field.bivector_part().to_coefficients(),
        &field.grade_project(1).to_coefficients(),
    );

    // Every product is cell-wise, so the grid shape is unchanged.
    let inner = field.inner_product(&field);
    print_products(
        &inner.to_coefficients(),
        &field.outer_product(&field).to_coefficients(),
        &field.cross(&field).to_coefficients(),
        &field.hodge_dual().to_coefficients(),
        inner.shape(),
    );

    print_algebra(
        &field.scale(lift::<FloatType>(2.0)).to_coefficients(),
        &field.reversion().to_coefficients(),
        &f,
        field.squared_magnitude(),
    );

    print_round_trip(&f);
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

/// One blade of one cell. The display boundary: `f64` appears here and nowhere else.
fn blade(cells: &[CausalMultiVector<FloatType>], cell: usize, index: usize) -> f64 {
    lower(*cells[cell].get(index).expect("blade index within 2^n"))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_storage(field: &CausalMultiField<FloatType>) {
    println!("=== CausalMultiField Basics ===\n");
    println!("--- Storage ---");
    println!("  metric          : {}", field.metric());
    println!("  grid shape      : {:?}", field.shape());
    println!("  cells           : {}", field.num_cells());
    println!(
        "  dx              : {:?}",
        field.dx().iter().map(|&x| lower(x)).collect::<Vec<_>>()
    );
    println!("  blades per cell : {}  (2^{N})", num_blades(N));
    println!("  matrix_dim      : {}  (D x D per cell)", matrix_dim(N));
    println!(
        "  tensor length   : {}  = {} cells x {}^2 matrix entries",
        field.data().len(),
        field.num_cells(),
        matrix_dim(N)
    );
    println!(
        "  `ones` cell 0 scalar blade = {}",
        blade(&field.to_coefficients(), 0, SCALAR)
    );
}

fn print_cells(f: &[CausalMultiVector<FloatType>]) {
    println!("\n--- from_coefficients: cell k holds k*e1 + e2 ---");
    for k in 0..f.len() {
        println!(
            "  cell {k}: e1 = {:>4}, e2 = {:>4}",
            blade(f, k, E1),
            blade(f, k, E2)
        );
    }
}

fn print_blade_check(e1e2: &CausalMultiVector<FloatType>) {
    let at = |i: usize| lower(*e1e2.get(i).expect("blade index within 2^n"));
    println!(
        "\n  e1 * e2 puts {} at index {E12}, and {} at index {SCALAR}",
        at(E12),
        at(SCALAR)
    );
}

fn print_grades(
    full: &[CausalMultiVector<FloatType>],
    vectors: &[CausalMultiVector<FloatType>],
    scalars: &[CausalMultiVector<FloatType>],
    bivectors: &[CausalMultiVector<FloatType>],
    projected: &[CausalMultiVector<FloatType>],
) {
    println!("\n--- Grade projection on cell 3 (3*e1 + e2, a pure grade-1 vector) ---");
    println!("  full    e1 = {}", blade(full, 3, E1));
    println!("  <F>_1   e1 = {}", blade(vectors, 3, E1));
    println!("  <F>_0   e1 = {}", blade(scalars, 3, E1));
    println!("  <F>_2   e1 = {}", blade(bivectors, 3, E1));
    println!(
        "  grade_project(1) agrees with vector_part: {}",
        blade(projected, 3, E1) == blade(vectors, 3, E1)
    );
}

fn print_products(
    inner: &[CausalMultiVector<FloatType>],
    outer: &[CausalMultiVector<FloatType>],
    cross: &[CausalMultiVector<FloatType>],
    dual: &[CausalMultiVector<FloatType>],
    shape: &[usize],
) {
    println!("\n--- Products on cell 3 ---");
    println!(
        "  F . F  scalar = {}   (|3e1 + e2|^2 = 10)",
        blade(inner, 3, SCALAR)
    );
    println!(
        "  F ^ F  e12    = {}   (a vector wedged with itself)",
        blade(outer, 3, E12)
    );
    println!("  F x F  e12    = {}", blade(cross, 3, E12));
    // The dual of a grade-1 vector in Cl(3) is a grade-2 bivector, so read it there.
    println!(
        "  hodge  e23    = {}   (dual of 3e1 sits on e23)",
        blade(dual, 3, E23)
    );
    println!("  grid shape after every product: {shape:?}");
}

fn print_algebra(
    doubled: &[CausalMultiVector<FloatType>],
    reversed: &[CausalMultiVector<FloatType>],
    f: &[CausalMultiVector<FloatType>],
    squared_magnitude: FloatType,
) {
    println!("\n--- Algebra on cell 3 ---");
    println!("  2F        e1  = {}", blade(doubled, 3, E1));
    println!("  reversion e1  = {}", blade(reversed, 3, E1));

    // `squared_magnitude` reduces the whole field to one scalar rather than one per cell,
    // and it sums over the stored matrix representation. Each blade occupies `matrix_dim`
    // entries of the D x D matrix, so the result is that multiple of the blade-space sum.
    let blade_sum: f64 = (0..f.len())
        .map(|k| {
            (0..num_blades(N))
                .map(|i| blade(f, k, i).powi(2))
                .sum::<f64>()
        })
        .sum();
    println!("  sum of |cell|^2 in blade space   = {blade_sum}");
    println!(
        "  field.squared_magnitude()        = {}  ({}x, one per matrix row)",
        lower(squared_magnitude),
        matrix_dim(N)
    );
}

fn print_round_trip(f: &[CausalMultiVector<FloatType>]) {
    println!("\n--- to_coefficients ---");
    println!("  recovered {} multivectors, one per cell", f.len());
    println!(
        "  cell 3 round trip: e1 = {}, e2 = {}",
        blade(f, 3, E1),
        blade(f, 3, E2)
    );
}
