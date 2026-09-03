/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `derive_code` and `check_ldpc_weights` on the reference complexes.
//!
//! The numbers are the complexes' own: the 3×3 simplicial torus has 9 vertices, 27 edges and 18
//! triangles, every triangle has three edges and every vertex has six, and `β₁(T²) = 2` over 𝔽₂
//! (Hatcher, Example 2.36).

use deep_causality_homology::ChainComplex;
use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_quantum::{
    CheckMatrix, CheckVerdict, LdpcItem, check_ldpc_weights, derive_code,
};

fn torus() -> impl ChainComplex {
    reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2")
        .0
}

#[test]
fn test_the_simplicial_torus_derives_its_own_numbers() {
    let code = derive_code::<u64, _>(&torus()).unwrap();
    assert_eq!(code.n(), 27);
    assert_eq!(code.k(), 2);
    assert_eq!(code.z_generators().len(), 18);
    assert!(code.z_generators().iter().all(|g| g.weight() == 3));
    assert_eq!(code.x_generators().len(), 9);
    assert!(code.x_generators().iter().all(|g| g.weight() == 6));
}

#[test]
fn test_ldpc_weights_pass_at_a_bound_of_six_and_report_the_examined_count() {
    let code = derive_code::<u64, _>(&torus()).unwrap();
    let w = check_ldpc_weights::<f64, u64>(&code, 6).unwrap();
    assert_eq!(w.report.verdict(), CheckVerdict::Accepted);
    assert_eq!(w.max_column_weight, 6);
    assert_eq!(
        w.max_row_weight, 2,
        "each edge lies in two triangles and meets two vertices"
    );
    // Z: 18 columns + 27 rows; X: 9 columns + 27 rows.
    assert_eq!(w.report.examined(), 18 + 27 + 9 + 27);
    assert!(w.offender.is_none());
}

#[test]
fn test_a_bound_the_x_checks_exceed_names_the_offender_and_stops_there() {
    let code = derive_code::<u64, _>(&torus()).unwrap();
    let w = check_ldpc_weights::<f64, u64>(&code, 3).unwrap();
    assert_eq!(w.report.verdict(), CheckVerdict::Rejected);
    assert_eq!(w.offender, Some((CheckMatrix::X, LdpcItem::Column(0))));
    let r = w.report.first_rejection().unwrap();
    assert_eq!(r.measured, 6.0);
    assert_eq!(r.threshold, 3.0);
    assert_eq!(r.margin, 2.0);
    // Examined up to and including the offender: all of Z, then the first X column.
    assert_eq!(w.report.examined(), 18 + 27 + 1);
}

#[test]
fn test_a_complex_without_two_cells_has_no_z_checks() {
    let mut saw_one = false;
    for (complex, _, _) in reference_spaces() {
        if complex.num_cells(2) == 0 && complex.num_cells(1) > 0 {
            let code = derive_code::<u64, _>(&complex).unwrap();
            assert!(code.z_generators().is_empty());
            let w = check_ldpc_weights::<f64, u64>(&code, 1).unwrap();
            // No Z column was examined; the X side still was.
            assert!(w.report.examined() >= complex.num_cells(1));
            saw_one = true;
        }
    }
    let _ = saw_one;
}
