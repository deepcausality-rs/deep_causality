/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    embed_on_legs, frobenius_norm, hermiticity_defect, identity_matrix, matrix_commutator,
    matrix_trace, partial_trace, supports_intersect,
};
use deep_causality_tensor::{CausalTensor, Tensor};
use std::collections::{BTreeMap, BTreeSet};

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0., 0.), c(1., 0.), c(1., 0.), c(0., 0.)], 2)
}

fn sigma_y() -> CausalTensor<C> {
    mat(vec![c(0., 0.), c(0., -1.), c(0., 1.), c(0., 0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(-1., 0.)], 2)
}

fn proj0() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2)
}

fn proj1() -> CausalTensor<C> {
    mat(vec![c(0., 0.), c(0., 0.), c(0., 0.), c(1., 0.)], 2)
}

fn max_abs_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

fn scale(a: &CausalTensor<C>, s: C) -> CausalTensor<C> {
    let data: Vec<C> = a
        .as_slice()
        .iter()
        .map(|x| c(x.re * s.re - x.im * s.im, x.re * s.im + x.im * s.re))
        .collect();
    CausalTensor::new(data, a.shape().to_vec()).unwrap()
}

// =============================================================================
// Partial trace: defining identities (the Q-PTP properties, task 2.3)
// =============================================================================

// THEOREM_MAP: quantum.partial_trace.kronecker
#[test]
fn test_partial_trace_product_identity() {
    // Tr_B(X ⊗ Y) = X · Tr(Y)
    let x = sigma_x();
    let y = mat(vec![c(2., 0.), c(0., 1.), c(0., -1.), c(3., 0.)], 2); // Tr = 5
    let xy = x.kronecker(&y).unwrap();
    let tr_b = partial_trace(&xy, &[2, 2], &[1]).unwrap();
    let expected = scale(&x, c(5., 0.));
    assert!(max_abs_diff(&tr_b, &expected) < 1e-12);
}

// THEOREM_MAP: quantum.partial_trace.add
// THEOREM_MAP: quantum.partial_trace.smul
#[test]
fn test_partial_trace_linearity() {
    // Tr_B(αM + N) = α·Tr_B(M) + Tr_B(N)
    let m = sigma_x().kronecker(&sigma_z()).unwrap();
    let n = sigma_z().kronecker(&sigma_x()).unwrap();
    let alpha = c(0.5, -1.5);

    let lhs = partial_trace(&(scale(&m, alpha) + n.clone()), &[2, 2], &[1]).unwrap();
    let rhs = scale(&partial_trace(&m, &[2, 2], &[1]).unwrap(), alpha)
        + partial_trace(&n, &[2, 2], &[1]).unwrap();
    assert!(max_abs_diff(&lhs, &rhs) < 1e-12);
}

// THEOREM_MAP: quantum.partial_trace.bimodule
// THEOREM_MAP: quantum.partial_trace.bimodule_right
#[test]
fn test_partial_trace_bimodule_law() {
    // Tr_B((1_B ⊗ Z)·M) = Z·Tr_B(M) — the Q-PTP boundary identity, stated on
    // H_A ⊗ H_B with A the kept (first) leg: Tr_B((Z ⊗ 1_B)·M) = Z·Tr_B(M).
    let z = mat(vec![c(1., 0.), c(2., 1.), c(0., -1.), c(-1., 0.)], 2);
    let m = sigma_x().kronecker(&sigma_y()).unwrap() + sigma_z().kronecker(&proj0()).unwrap();

    let z_full = z.kronecker(&identity_matrix::<f64>(2)).unwrap();
    let lhs = partial_trace(&z_full.matmul(&m).unwrap(), &[2, 2], &[1]).unwrap();
    let rhs = z
        .matmul(&partial_trace(&m, &[2, 2], &[1]).unwrap())
        .unwrap();
    assert!(max_abs_diff(&lhs, &rhs) < 1e-12);
}

#[test]
fn test_partial_trace_preserves_trace_and_hermiticity() {
    let m = sigma_x().kronecker(&sigma_y()).unwrap() + proj0().kronecker(&sigma_z()).unwrap();
    let t_full = matrix_trace(&m).unwrap();
    let tr_b = partial_trace(&m, &[2, 2], &[1]).unwrap();
    let t_after = matrix_trace(&tr_b).unwrap();
    assert!((t_full.re - t_after.re).abs() < 1e-12 && (t_full.im - t_after.im).abs() < 1e-12);
    assert!(hermiticity_defect(&tr_b).unwrap() < 1e-12);
}

#[test]
fn test_partial_trace_named_subset_three_legs() {
    // On H_0 ⊗ H_1 ⊗ H_2, tracing legs {0, 2} of X⊗Y⊗Z gives Y·Tr(X)·Tr(Z).
    let x = sigma_z(); // Tr = 0
    let y = sigma_x();
    let z = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(3., 0.)], 2); // Tr = 4
    let xyz = x.kronecker(&y).unwrap().kronecker(&z).unwrap();

    let kept = partial_trace(&xyz, &[2, 2, 2], &[0, 2]).unwrap();
    // Tr(X) = 0 → result is the zero matrix.
    assert!(frobenius_norm(&kept) < 1e-12);

    // And tracing legs {1, 2} keeps X·(Tr Y)(Tr Z) = 0 as well; a non-zero check:
    let w = mat(vec![c(2., 0.), c(0., 0.), c(0., 0.), c(2., 0.)], 2); // Tr = 4
    let xwz = x.kronecker(&w).unwrap().kronecker(&z).unwrap();
    let kept_x = partial_trace(&xwz, &[2, 2, 2], &[1, 2]).unwrap();
    let expected = scale(&x, c(16., 0.)); // Tr(W)·Tr(Z) = 4·4
    assert!(max_abs_diff(&kept_x, &expected) < 1e-12);
}

#[test]
fn test_partial_trace_full_trace_degenerates_to_trace() {
    let m = sigma_x().kronecker(&sigma_z()).unwrap() + proj1().kronecker(&proj0()).unwrap();
    let all = partial_trace(&m, &[2, 2], &[0, 1]).unwrap();
    assert_eq!(all.shape(), &[1, 1]);
    let t = matrix_trace(&m).unwrap();
    assert!((all.as_slice()[0].re - t.re).abs() < 1e-12);
}

#[test]
fn test_partial_trace_rejects_bad_shapes() {
    let m = sigma_x();
    assert!(partial_trace(&m, &[2, 2], &[1]).is_err()); // 2x2 is not 4x4
    let m4 = sigma_x().kronecker(&sigma_z()).unwrap();
    assert!(partial_trace(&m4, &[2, 2], &[2]).is_err()); // leg out of range
    assert!(partial_trace(&m4, &[2, 2], &[0, 0]).is_err()); // duplicate
    assert!(partial_trace(&m4, &[3, 2], &[0]).is_err()); // wrong factorization
}

// =============================================================================
// The B1 counterexample — partial trace does not preserve commutation.
// Rust witness for `quantum.partial_trace_nonpreservation` (task 5.4):
// X = σx⊗|0><0| + σz⊗|1><1|, Y = σx⊗|0><0| − σz⊗|1><1|:
// [X, Y] = 0 but [Tr₂X, Tr₂Y] = +4i·σy ≠ 0.
// =============================================================================

// THEOREM_MAP: quantum.partial_trace_nonpreservation
// THEOREM_MAP: quantum.partial_trace_nonpreservation.value
#[test]
fn test_partial_trace_nonpreservation_counterexample() {
    let x = sigma_x().kronecker(&proj0()).unwrap() + sigma_z().kronecker(&proj1()).unwrap();
    let y = sigma_x().kronecker(&proj0()).unwrap() - sigma_z().kronecker(&proj1()).unwrap();

    // Both Hermitian, and they commute.
    assert!(hermiticity_defect(&x).unwrap() < 1e-12);
    assert!(hermiticity_defect(&y).unwrap() < 1e-12);
    let comm_full = matrix_commutator(&x, &y).unwrap();
    assert!(frobenius_norm(&comm_full) < 1e-12, "[X,Y] must vanish");

    // Their partial traces do NOT commute: [Tr₂X, Tr₂Y] = +4i·σy.
    let tx = partial_trace(&x, &[2, 2], &[1]).unwrap();
    let ty = partial_trace(&y, &[2, 2], &[1]).unwrap();
    let comm = matrix_commutator(&tx, &ty).unwrap();
    let expected = scale(&sigma_y(), c(0., 4.)); // +4i·σy
    assert!(
        max_abs_diff(&comm, &expected) < 1e-12,
        "[Tr₂X, Tr₂Y] must equal +4i·σy"
    );
    assert!((frobenius_norm(&comm) - 32.0_f64.sqrt()).abs() < 1e-12);
}

// THEOREM_MAP: quantum.partial_trace_preservation_boundary
#[test]
fn test_partial_trace_preservation_boundary_case() {
    // The conditional theorem's hypothesis (Q-PTP): if Y = Z ⊗ 1_B acts only
    // on the boundary (kept) factor and [X, Y] = 0, then the partial traces
    // commute. Witness for `quantum.partial_trace_preservation_boundary`.
    let z = sigma_z();
    let y = z.kronecker(&identity_matrix::<f64>(2)).unwrap();
    // X commuting with Z ⊗ 1: X = σz ⊗ W for any W.
    let w = mat(vec![c(1., 0.), c(0., 2.), c(0., -2.), c(4., 0.)], 2);
    let x = sigma_z().kronecker(&w).unwrap();

    assert!(frobenius_norm(&matrix_commutator(&x, &y).unwrap()) < 1e-12);
    let tx = partial_trace(&x, &[2, 2], &[1]).unwrap();
    let ty = partial_trace(&y, &[2, 2], &[1]).unwrap();
    assert!(
        frobenius_norm(&matrix_commutator(&tx, &ty).unwrap()) < 1e-12,
        "boundary-only support must preserve commutation"
    );
}

// =============================================================================
// Commutator, supports, embedding (task 2.5)
// =============================================================================

#[test]
fn test_matrix_commutator_pauli_algebra() {
    // [σx, σy] = 2i·σz
    let comm = matrix_commutator(&sigma_x(), &sigma_y()).unwrap();
    let expected = scale(&sigma_z(), c(0., 2.));
    assert!(max_abs_diff(&comm, &expected) < 1e-12);

    // [A, A] = 0
    let self_comm = matrix_commutator(&sigma_x(), &sigma_x()).unwrap();
    assert!(frobenius_norm(&self_comm) < 1e-12);

    // Dimension mismatch rejected
    let big = identity_matrix::<f64>(4);
    assert!(matrix_commutator(&sigma_x(), &big).is_err());
}

#[test]
fn test_supports_intersect() {
    let a: BTreeSet<usize> = [0, 1].into_iter().collect();
    let b: BTreeSet<usize> = [1, 2].into_iter().collect();
    let d: BTreeSet<usize> = [3, 4].into_iter().collect();
    assert!(supports_intersect(&a, &b));
    assert!(!supports_intersect(&a, &d));
}

#[test]
fn test_embed_on_legs_adjacent_matches_kronecker() {
    // Embedding onto legs {0} of a 2-leg space is A ⊗ I; onto {1} is I ⊗ A.
    let space: BTreeMap<usize, usize> = [(0, 2), (1, 2)].into_iter().collect();
    let a = sigma_y();

    let on0 = embed_on_legs(&a, &[0].into_iter().collect(), &space).unwrap();
    let expected0 = a.kronecker(&identity_matrix::<f64>(2)).unwrap();
    assert!(max_abs_diff(&on0, &expected0) < 1e-12);

    let on1 = embed_on_legs(&a, &[1].into_iter().collect(), &space).unwrap();
    let expected1 = identity_matrix::<f64>(2).kronecker(&a).unwrap();
    assert!(max_abs_diff(&on1, &expected1) < 1e-12);
}

#[test]
fn test_embed_on_legs_non_adjacent() {
    // op on legs {0, 2} of a 3-leg space: (X ⊗ Z) embedded must equal
    // X ⊗ 1 ⊗ Z (leg 1 identity in the middle).
    let space: BTreeMap<usize, usize> = [(0, 2), (1, 2), (2, 2)].into_iter().collect();
    let op = sigma_x().kronecker(&sigma_z()).unwrap();
    let embedded = embed_on_legs(&op, &[0, 2].into_iter().collect(), &space).unwrap();

    let expected = sigma_x()
        .kronecker(&identity_matrix::<f64>(2))
        .unwrap()
        .kronecker(&sigma_z())
        .unwrap();
    assert!(max_abs_diff(&embedded, &expected) < 1e-12);
}

#[test]
fn test_embed_on_legs_disjoint_supports_commute() {
    // Operators embedded on disjoint legs always commute — the reason the
    // freeze check may skip non-intersecting supports.
    let space: BTreeMap<usize, usize> = [(0, 2), (1, 2), (2, 2)].into_iter().collect();
    let a = embed_on_legs(&sigma_x(), &[0].into_iter().collect(), &space).unwrap();
    let b = embed_on_legs(&sigma_y(), &[2].into_iter().collect(), &space).unwrap();
    let comm = matrix_commutator(&a, &b).unwrap();
    assert!(frobenius_norm(&comm) < 1e-12);
}

#[test]
fn test_embed_on_legs_rejects_mismatches() {
    let space: BTreeMap<usize, usize> = [(0, 2), (1, 2)].into_iter().collect();
    // op dim disagrees with its legs
    let op4 = identity_matrix::<f64>(4);
    assert!(embed_on_legs(&op4, &[0].into_iter().collect(), &space).is_err());
    // leg not in the space
    assert!(embed_on_legs(&sigma_x(), &[7].into_iter().collect(), &space).is_err());
}

// =============================================================================
// Freeze-critical paths at Complex<Float106> (task 2.6)
// =============================================================================

#[test]
fn test_counterexample_and_eigen_at_float106() {
    type F = Float106;
    type CF = Complex<F>;
    let f = Float106::from_f64;
    let cf = |re: f64, im: f64| Complex::new(f(re), f(im));

    let m2 = |d: [[(f64, f64); 2]; 2]| -> CausalTensor<CF> {
        let data: Vec<CF> = d
            .iter()
            .flat_map(|row| row.iter().map(|(re, im)| cf(*re, *im)))
            .collect();
        CausalTensor::new(data, vec![2, 2]).unwrap()
    };
    let sx = m2([[(0., 0.), (1., 0.)], [(1., 0.), (0., 0.)]]);
    let sz = m2([[(1., 0.), (0., 0.)], [(0., 0.), (-1., 0.)]]);
    let p0 = m2([[(1., 0.), (0., 0.)], [(0., 0.), (0., 0.)]]);
    let p1 = m2([[(0., 0.), (0., 0.)], [(0., 0.), (1., 0.)]]);

    // The B1 counterexample at double-double precision.
    let x = sx.kronecker(&p0).unwrap() + sz.kronecker(&p1).unwrap();
    let y = sx.kronecker(&p0).unwrap() - sz.kronecker(&p1).unwrap();
    let full: f64 = frobenius_norm(&matrix_commutator(&x, &y).unwrap()).into();
    assert!(full < 1e-30, "[X,Y] must vanish at Float106: {}", full);

    let tx = partial_trace(&x, &[2, 2], &[1]).unwrap();
    let ty = partial_trace(&y, &[2, 2], &[1]).unwrap();
    let reduced: f64 = frobenius_norm(&matrix_commutator(&tx, &ty).unwrap()).into();
    assert!(
        (reduced - 32.0_f64.sqrt()).abs() < 1e-12,
        "‖[Tr₂X, Tr₂Y]‖_F must be √32 at Float106: {}",
        reduced
    );

    // The Hermitian eigensolver at Float106: σx spectrum is ±1.
    let (vals, _) = sx.eigen_hermitian().unwrap();
    let mut re: Vec<f64> = vals.iter().map(|c| c.re.into()).collect();
    re.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((re[0] + 1.0).abs() < 1e-28 && (re[1] - 1.0).abs() < 1e-28);
}
