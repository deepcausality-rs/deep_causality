/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstrating basic CsrMatrix operations:");

    // 1. Create a CsrMatrix from triplets
    println!("\n--- Matrix A (2x3) from triplets ---");
    let triplets_a = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let a = CsrMatrix::from_triplets(2, 3, &triplets_a)?;
    // A = [[1.0, 0.0, 2.0],
    //      [0.0, 3.0, 0.0]]
    println!("Matrix A:\n{}", a); // Using the Display trait
    println!("A[0,0]: {}", a.get_value_at(0, 0));
    println!("A[0,1]: {}", a.get_value_at(0, 1)); // Should be 0.0

    // 2. Scalar Multiplication
    println!("\n--- Scalar Multiplication (A * 2.0) ---");
    let scalar = 2.0;
    let b = a.scalar_mult(scalar);
    // B = [[2.0, 0.0, 4.0],
    //      [0.0, 6.0, 0.0]]
    println!("Matrix B (A *{});\n{}", scalar, b); // Corrected format string and output

    // 3. Matrix Addition
    println!("\n--- Matrix Addition (A + C) ---");
    let triplets_c = vec![(0, 1, 5.0), (1, 0, 6.0)];
    let c = CsrMatrix::from_triplets(2, 3, &triplets_c)?;
    // C = [[0.0, 5.0, 0.0],
    //      [6.0, 0.0, 0.0]]
    println!("Matrix C:\n{}", c);

    let d = a.add_matrix(&c)?;
    // D = A + C = [[1.0, 5.0, 2.0],
    //              [6.0, 3.0, 0.0]]
    println!("Matrix D (A + C):\n{}", d);

    // 4. Matrix-Vector Multiplication (A * x)
    println!("\n--- Matrix-Vector Multiplication (A * x) ---");
    let x = vec![1.0, 2.0, 3.0];
    let y = a.vec_mult(&x)?;
    // y = Ax = [(1.0*1.0 + 0.0*2.0 + 2.0*3.0), (0.0*1.0 + 3.0*2.0 + 0.0*3.0)] = [7.0, 6.0]
    println!("Vector x: {:?}", x);
    println!("Result y = Ax: {:?}", y);

    // 5. Matrix Multiplication (A * E)
    println!("\n--- Matrix Multiplication (A * E) ---");
    let triplets_e = vec![(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)];
    let e = CsrMatrix::from_triplets(3, 2, &triplets_e)?;
    // E (3x2) = [[4.0, 0.0],
    //            [0.0, 5.0],
    //            [6.0, 0.0]]
    println!("Matrix E:\n{}", e);

    let f = a.mat_mult(&e)?;
    // F = A * E (2x2) = [[(1*4+0*0+2*6), (1*0+0*5+2*0)],
    //                    [(0*4+3*0+0*6), (0*0+3*5+0*0)]]
    //                 = [[16.0, 0.0],
    //                    [0.0, 15.0]]
    println!("Matrix F (A * E):\n{}", f);

    // 6. Transpose (A^T)
    println!("\n--- Transpose (A^T) ---");
    let a_t = a.transpose();
    // A^T (3x2) = [[1.0, 0.0],
    //              [0.0, 3.0],
    //              [2.0, 0.0]]
    println!("Matrix A^T:\n{}", a_t);

    // 7. Error Handling Example (DimensionMismatch)
    println!("\n--- Error Handling (Dimension Mismatch) ---");
    let x_invalid = vec![1.0, 2.0]; // Incorrect length for A (2x3)
    match a.vec_mult(&x_invalid) {
        Ok(_) => println!("Unexpected success with invalid vector length"),
        Err(e) => println!("Caught expected error: {}", e),
    }

    Ok(())
}
