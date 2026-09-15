/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `[[4, 2, 2]]` fixture against its stated parameters: `n = 4` from the 1-cells, `k = 2` from
//! `β₁` over 𝔽₂, one Z-check `ZZZZ` and two X-checks both `XXXX`, `∂₁∂₂ = 0` over ℤ; the checkless
//! `[[3, 3, 1]]` fixture and the complex with no logical qubits against theirs.

use deep_causality_homology::{ChainComplex, HomologyField};
use deep_causality_quantum::utils_tests::{four_two_two, no_logical_qubits, three_three_one};
use deep_causality_quantum::{LogicalBasis, derive_code};

#[test]
fn test_four_two_two_parameters() {
    let c = four_two_two();
    assert_eq!((c.num_cells(0), c.num_cells(1), c.num_cells(2)), (2, 4, 1));
    assert_eq!(c.betti_number_over(1, HomologyField::Gf2).unwrap(), 2);
    let code = derive_code::<u64, _>(&c).unwrap();
    assert_eq!((code.n(), code.k()), (4, 2));
    assert_eq!(code.z_generators().len(), 1);
    assert_eq!(code.z_generators()[0].weight(), 4);
    assert_eq!(code.x_generators().len(), 2);
    assert!(code.x_generators().iter().all(|g| g.weight() == 4));
    let basis = LogicalBasis::<u64>::from_complex(&c, 1).unwrap();
    assert_eq!(basis.num_logical_qubits(), 2);
    assert!(
        basis.homology().iter().all(|g| g.weight() >= 2),
        "distance two"
    );
}

#[test]
fn test_boundary_composes_to_zero_over_the_integers() {
    let c = four_two_two();
    let d1 = c.boundary_matrix(1);
    let d2 = c.boundary_matrix(2);
    // ∂₁ ∂₂: one column, two rows.
    let d2_col: Vec<i64> = {
        let mut v = vec![0i64; 4];
        for (idx, &r) in d2.col_indices().iter().enumerate() {
            let _ = r;
            let row = d2.row_indices().iter().position(|&s| s > idx).unwrap() - 1;
            v[row] += d2.values()[idx] as i64;
        }
        v
    };
    for r in 0..2 {
        let mut acc = 0i64;
        for idx in d1.row_indices()[r]..d1.row_indices()[r + 1] {
            acc += d1.values()[idx] as i64 * d2_col[d1.col_indices()[idx]];
        }
        assert_eq!(acc, 0, "row {r} of ∂₁∂₂");
    }
}

#[test]
fn test_three_three_one_has_three_logical_qubits_and_no_checks() {
    let c = three_three_one();
    assert_eq!((c.num_cells(0), c.num_cells(1), c.num_cells(2)), (1, 3, 0));
    assert_eq!(c.betti_number_over(1, HomologyField::Gf2).unwrap(), 3);
    let code = derive_code::<u64, _>(&c).unwrap();
    assert_eq!((code.n(), code.k()), (3, 3));
    assert!(code.z_generators().is_empty(), "no 2-cells, no Z-checks");
    assert!(
        code.x_generators().iter().all(|g| g.weight() == 0),
        "∂₁ = 0, so the X-check of the one vertex is trivial"
    );
    let basis = LogicalBasis::<u64>::from_complex(&c, 1).unwrap();
    assert_eq!(basis.num_logical_qubits(), 3);
    assert!(basis.homology().iter().all(|g| g.weight() == 1));
}

#[test]
fn test_no_logical_qubits_has_first_betti_number_zero() {
    let c = no_logical_qubits();
    assert_eq!((c.num_cells(0), c.num_cells(1), c.num_cells(2)), (2, 1, 0));
    assert_eq!(c.betti_number_over(1, HomologyField::Gf2).unwrap(), 0);
    let code = derive_code::<u64, _>(&c).unwrap();
    assert_eq!((code.n(), code.k()), (1, 0));
    let basis = LogicalBasis::<u64>::from_complex(&c, 1).unwrap();
    assert_eq!(basis.num_logical_qubits(), 0);
    assert_eq!(basis.len(), 1);
}
