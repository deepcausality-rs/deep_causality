/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Lean partial-trace proofs.
//!
//! Each `#[test]` here is the executable counterpart of a `THEOREM_MAP`-tagged
//! theorem in the Lean formalization; the traceability CI check
//! (`.github/workflows/formalization.yml`, job `theorem-map`) requires every
//! proved Lean theorem to have a matching `// THEOREM_MAP: <id>` witness in a
//! Rust file. Consolidating them under `tests/formalization_lean/` keeps the
//! Lean↔Rust bridge in one place, mirroring the Lean module layout:
//!
//!   * `lean/DeepCausalityFormal/Quantum/PartialTrace.lean`
//!   * `lean/DeepCausalityFormal/Quantum/PartialTraceCounterexample.lean`

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    frobenius_norm, hermiticity_defect, identity_matrix, matrix_commutator, partial_trace,
    partial_trace_preservation_boundary,
};
use deep_causality_tensor::{CausalTensor, Tensor};

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
    assert_eq!(
        a.shape(),
        b.shape(),
        "max_abs_diff shape mismatch: {:?} vs {:?}",
        a.shape(),
        b.shape()
    );
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
// PartialTrace.lean — the defining identities of Tr_B (the Q-PTP properties).
// =============================================================================

// THEOREM_MAP: quantum.partial_trace.kronecker
#[test]
fn test_partial_trace_product_identity() {
    // Lean: partialTraceRight_kron — Tr_B(X ⊗ Y) = X · Tr(Y).
    let x = sigma_x();
    let y = mat(vec![c(2., 0.), c(0., 1.), c(0., -1.), c(3., 0.)], 2); // Tr = 5
    let xy = x.kronecker(&y).unwrap();
    let tr_b = partial_trace(&xy, &[2, 2], &[1]).unwrap();
    let expected = scale(&x, c(5., 0.));
    assert!(max_abs_diff(&tr_b, &expected) < 1e-12);
}

// THEOREM_MAP: quantum.partial_trace.add
// THEOREM_MAP: quantum.partial_trace.smul
// THEOREM_MAP: quantum.partial_trace.sub
#[test]
fn test_partial_trace_linearity() {
    // Lean: partialTraceRight_add, partialTraceRight_smul —
    // Tr_B(αM + N) = α·Tr_B(M) + Tr_B(N).
    let m = sigma_x().kronecker(&sigma_z()).unwrap();
    let n = sigma_z().kronecker(&sigma_x()).unwrap();
    let alpha = c(0.5, -1.5);

    let lhs = partial_trace(&(scale(&m, alpha) + n.clone()), &[2, 2], &[1]).unwrap();
    let rhs = scale(&partial_trace(&m, &[2, 2], &[1]).unwrap(), alpha)
        + partial_trace(&n, &[2, 2], &[1]).unwrap();
    assert!(max_abs_diff(&lhs, &rhs) < 1e-12);

    // Lean: partialTraceRight_sub — Tr_B(M − N) = Tr_B(M) − Tr_B(N). The subtractive form is
    // the step that carries the trace through a commutator, so it is witnessed on its own
    // rather than left to follow from additivity.
    let sub_lhs = partial_trace(&(&m - &n), &[2, 2], &[1]).unwrap();
    let sub_rhs =
        &partial_trace(&m, &[2, 2], &[1]).unwrap() - &partial_trace(&n, &[2, 2], &[1]).unwrap();
    assert!(max_abs_diff(&sub_lhs, &sub_rhs) < 1e-12);
}

// THEOREM_MAP: quantum.partial_trace.bimodule
// THEOREM_MAP: quantum.partial_trace.bimodule_right
#[test]
fn test_partial_trace_bimodule_law() {
    // Lean: partialTraceRight_bimodule(_right) — the Q-PTP boundary identity on
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

// THEOREM_MAP: quantum.partial_trace_preservation_boundary
#[test]
fn test_partial_trace_preservation_boundary_case() {
    // Lean: partial_trace_preservation_boundary — the conditional theorem's
    // hypothesis (Q-PTP): if Y = Z ⊗ 1_B acts only on the boundary (kept) factor
    // and [X, Y] = 0, then the partial traces commute.
    let z = sigma_z();
    let y = z.kronecker(&identity_matrix::<f64>(2)).unwrap();
    // X commuting with Z ⊗ 1: X = σz ⊗ W for any W.
    let w = mat(vec![c(1., 0.), c(0., 2.), c(0., -2.), c(4., 0.)], 2);
    let x = sigma_z().kronecker(&w).unwrap();

    // The operands are small dyadic integers and the commutator vanishes algebraically —
    // [σz⊗W, σz⊗1] = (σz σz)⊗W − (σz σz)⊗W — so kron and matmul are exact in binary64 and this
    // is a bit-exact zero rather than a tolerance standing in for one. Asserted as such, because
    // the theorem's hypothesis is exact equality and a `< 1e-12` here would misrepresent an exact
    // premise as an approximate one. See `test_a_tolerance_is_not_warrant_for_the_exact_conclusion`
    // for the case where the hypothesis genuinely only holds approximately.
    assert_eq!(frobenius_norm(&matrix_commutator(&x, &y).unwrap()), 0.0);
    let tx = partial_trace(&x, &[2, 2], &[1]).unwrap();
    let ty = partial_trace(&y, &[2, 2], &[1]).unwrap();
    assert_eq!(
        frobenius_norm(&matrix_commutator(&tx, &ty).unwrap()),
        0.0,
        "boundary-only support must preserve commutation"
    );
}

// THEOREM_MAP: quantum.partial_trace.commutator_transport
#[test]
fn test_partial_trace_transports_the_commutator_exactly() {
    // Lean: partialTraceRight_commutator — Tr_B([Z⊗1_B, M]) = [Z, Tr_B(M)], with no hypothesis.
    // This is the identity the boundary ruling rests on: it holds for every M, not only for one
    // whose commutator vanishes, which is what lets a caller bound a near-miss instead of having
    // to assert an exact premise.
    let z = sigma_z();
    let boundary = z.kronecker(&identity_matrix::<f64>(2)).unwrap();
    // An M that does NOT commute with Z⊗1, so both sides are non-zero and the identity has
    // content. σx⊗W fails to commute with σz⊗1 because σx and σz anticommute.
    let w = mat(vec![c(1., 0.), c(0., 2.), c(0., -2.), c(4., 0.)], 2);
    let m = sigma_x().kronecker(&w).unwrap();

    let lhs = partial_trace(&matrix_commutator(&boundary, &m).unwrap(), &[2, 2], &[1]).unwrap();
    let rhs = matrix_commutator(&z, &partial_trace(&m, &[2, 2], &[1]).unwrap()).unwrap();

    assert!(
        frobenius_norm(&lhs) > 0.0,
        "the test case must be non-trivial"
    );
    assert_eq!(
        max_abs_diff(&lhs, &rhs),
        0.0,
        "the transport is an identity"
    );
}

#[test]
fn test_the_boundary_check_recovers_the_exact_theorem_at_zero() {
    // Exactly zero in, exactly zero out: the Lean theorem is the vanishing case of the bound.
    let z = sigma_z();
    let w = mat(vec![c(1., 0.), c(0., 2.), c(0., -2.), c(4., 0.)], 2);
    let m = sigma_z().kronecker(&w).unwrap();

    let warrant = partial_trace_preservation_boundary(&z, &m, [2, 2], 0.0).unwrap();
    assert_eq!(warrant.hypothesis_residual, 0.0);
    assert_eq!(warrant.conclusion_bound, 0.0);
    assert!(
        warrant.holds,
        "a zero residual must satisfy a zero tolerance"
    );
}

#[test]
fn test_a_tolerance_is_not_warrant_for_the_exact_conclusion() {
    // The ruling G-16 asks for. A hypothesis satisfied only to ε does not give the exact
    // conclusion; it gives the conclusion at √(d_B)·ε. Here the commutator is small but non-zero,
    // and the traced commutator is correspondingly non-zero — so a caller that read `holds` as
    // "the operators commute after marginalisation" would be wrong.
    let z = sigma_z();
    let eps = 1e-9;
    // M = σz⊗W + ε·(σx⊗1): the first term commutes with σz⊗1, the second does not.
    let w = mat(vec![c(1., 0.), c(0., 2.), c(0., -2.), c(4., 0.)], 2);
    let base = sigma_z().kronecker(&w).unwrap();
    let perturb = sigma_x().kronecker(&identity_matrix::<f64>(2)).unwrap();
    let m = mat(
        base.as_slice()
            .iter()
            .zip(perturb.as_slice())
            .map(|(a, b)| c(a.re + eps * b.re, a.im + eps * b.im))
            .collect(),
        4,
    );

    let warrant = partial_trace_preservation_boundary(&z, &m, [2, 2], 1e-6).unwrap();
    assert!(warrant.holds, "the residual is well inside the tolerance");
    assert!(
        warrant.hypothesis_residual > 0.0,
        "the hypothesis must be genuinely approximate here"
    );

    // The conclusion is not exact, and the certified bound covers what it actually is.
    let traced = matrix_commutator(&z, &partial_trace(&m, &[2, 2], &[1]).unwrap()).unwrap();
    let actual = frobenius_norm(&traced);
    assert!(actual > 0.0, "the traced commutator does not vanish");
    assert!(
        actual <= warrant.conclusion_bound + 1e-15,
        "the certified bound {} must cover the actual {}",
        warrant.conclusion_bound,
        actual
    );
}

#[test]
fn test_the_amplification_is_the_square_root_of_the_traced_dimension() {
    // ‖Tr_B(E)‖_F ≤ √(d_B)·‖E‖_F, tight at E = F ⊗ 1_B. The factor is what the ruling costs, and
    // it grows with the dimension marginalised away.
    let z = sigma_z();
    let m = sigma_z().kronecker(&identity_matrix::<f64>(2)).unwrap();
    let w2 = partial_trace_preservation_boundary(&z, &m, [2, 2], 1.0).unwrap();
    assert!((w2.amplification - 2f64.sqrt()).abs() < 1e-15);

    // A wider traced factor amplifies more.
    let m4 = sigma_z().kronecker(&identity_matrix::<f64>(4)).unwrap();
    let w4 = partial_trace_preservation_boundary(&z, &m4, [2, 4], 1.0).unwrap();
    assert!((w4.amplification - 2.0).abs() < 1e-15);
    assert!(w4.amplification > w2.amplification);
}

#[test]
fn test_the_boundary_check_rejects_shapes_it_cannot_factor() {
    let z = sigma_z();
    let m = sigma_z().kronecker(&identity_matrix::<f64>(2)).unwrap();
    // dims that do not multiply to the operator's side.
    assert!(partial_trace_preservation_boundary(&z, &m, [2, 3], 1.0).is_err());
    // a kept-factor operator of the wrong side.
    let wrong = identity_matrix::<f64>(4);
    assert!(partial_trace_preservation_boundary(&wrong, &m, [2, 2], 1.0).is_err());
}

// =============================================================================
// PartialTraceCounterexample.lean — partial trace does NOT preserve commutation.
// The B1 witness: X = σx⊗|0><0| + σz⊗|1><1|, Y = σx⊗|0><0| − σz⊗|1><1|:
// [X, Y] = 0 but [Tr₂X, Tr₂Y] = +4i·σy ≠ 0.
// =============================================================================

// THEOREM_MAP: quantum.partial_trace_nonpreservation
// THEOREM_MAP: quantum.partial_trace_nonpreservation.value
#[test]
fn test_partial_trace_nonpreservation_counterexample() {
    // Lean: partial_trace_nonpreservation (+ _value, the exact +4i·σy).
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
