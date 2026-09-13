/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The exact semantics against the numeric one, on registers both can reach.
//!
//! The anchor is Haruna, arXiv:2511.15224, Eq. (3.56): `T̄(γ) = exp(iπ/4 · (I − Z̄(γ))/2)`, whose
//! physical program is Eq. (3.59). Conjugating `X_q`, `q ∈ γ`, flips the parity of `γ`, so the
//! remainder is `exp(iπ/4 · (1 − 2p)) = e^{iπ/4} exp(−iπ/2 · p)`: phase `1/8` turn on the even
//! pattern and `7/8` on the odd. Its Pauli terms are `I` and `Z̄(γ)` with coefficients
//! `(e^{iπ/4} + e^{−iπ/4})/2 = 1/√2` and `(e^{iπ/4} − e^{−iπ/4})/2 = i/√2`. The numeric side forms
//! the program's unitary `U` and `U X_q U†` directly, and the test asserts the two are one matrix.
//! Corner cases: (B) a one-gate program, (C) an even-overlap fault, (E) a Z-type fault, (I) a
//! Pauli over another register, (K) the refusal of a program outside the normal form.

use deep_causality_algebra::Normed;
use deep_causality_homology::Gf2Chain;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CircuitBox, CircuitModel, ExactLayer, ExactProgram, GateOp, GaugeFieldGate, LogicalPauli,
    NumericCaps, QuantumErrorEnum, WireType, is_clifford_gate, is_diagonal_gate, logical_hadamard,
    logical_s, logical_t,
};
use deep_causality_tensor::{CausalTensor, Tensor};

type C = Complex<f64>;
type W = u64;

fn chain(n: usize, support: &[usize]) -> Gf2Chain<W> {
    Gf2Chain::from_support(n, 1, support).unwrap()
}

fn pauli(n: usize, x: &[usize], z: &[usize]) -> LogicalPauli<W> {
    LogicalPauli::new(chain(n, x), chain(n, z)).unwrap()
}

/// The unitary of a gate program on `n` qubits, from the numeric semantics.
fn unitary_of(n: usize, program: Vec<GateOp>) -> CausalTensor<C> {
    let all: Vec<usize> = (0..n).collect();
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(); n],
        vec![CircuitBox::Unitary {
            wires: all.clone(),
            program,
        }],
        all.clone(),
        all,
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    m.blocks().get(&(vec![], vec![])).unwrap()[0].clone()
}

/// The matrix of a Pauli `(x, z)` as `X^x Z^z` on `n` qubits, qubit 0 most significant.
fn pauli_matrix(p: &LogicalPauli<W>) -> CausalTensor<C> {
    let n = p.len();
    let d = 1usize << n;
    let xs: Vec<usize> = p.x().support().collect();
    let zs: Vec<usize> = p.z().support().collect();
    let mut data = vec![C::new(0.0, 0.0); d * d];
    for col in 0..d {
        // Z first: sign from the bits set in col on z's support; then X flips x's support.
        let mut sign = 1.0;
        for &q in &zs {
            if col & (1 << (n - 1 - q)) != 0 {
                sign = -sign;
            }
        }
        let mut row = col;
        for &q in &xs {
            row ^= 1 << (n - 1 - q);
        }
        data[row * d + col] = C::new(sign, 0.0);
    }
    CausalTensor::from_slice(&data, &[d, d])
}

/// The matrix of a gauge-field gate from its Pauli coefficients: `Σ_S c_S Z̄(γ_S)`.
fn remainder_matrix(g: &GaugeFieldGate<W>) -> CausalTensor<C> {
    let n = g.len();
    let d = 1usize << n;
    let mut acc = CausalTensor::from_slice(&vec![C::new(0.0, 0.0); d * d], &[d, d]);
    for (s, c) in g.pauli_coefficients::<f64>().unwrap() {
        let mut zsupp = chain(n, &[]);
        for (i, b) in g.blocks().iter().enumerate() {
            if s & (1 << i) != 0 {
                zsupp = zsupp.add(b).unwrap();
            }
        }
        let zm = pauli_matrix(&LogicalPauli::new(chain(n, &[]), zsupp).unwrap());
        let scaled: Vec<C> = zm.as_slice().iter().map(|v| *v * c).collect();
        acc = acc + CausalTensor::from_slice(&scaled, &[d, d]);
    }
    acc
}

fn close(a: &CausalTensor<C>, b: &CausalTensor<C>) -> bool {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .all(|(x, y)| (x.re - y.re).abs() < 1e-10 && (x.im - y.im).abs() < 1e-10)
}

fn conj_by(u: &CausalTensor<C>, p: &CausalTensor<C>) -> CausalTensor<C> {
    u.matmul(p).unwrap().matmul(&u.dagger().unwrap()).unwrap()
}

#[test]
fn test_t_bar_remainder_matches_the_numeric_conjugation_at_three_weights() {
    for w in 3..=5 {
        let gamma = chain(w, &(0..w).collect::<Vec<_>>());
        let program = ExactProgram::diagonal(GaugeFieldGate::t(gamma.clone()).unwrap());
        let out = program.conjugate(&pauli(w, &[0], &[])).unwrap();
        assert_eq!(out.pauli, pauli(w, &[0], &[]));
        assert!(!out.remainder.is_constant());
        // Phase table: 1/8 on the even pattern, 7/8 on the odd.
        let ph = out.remainder.phases();
        assert_eq!(*ph[0].numer(), 1);
        assert_eq!(*ph[0].denom(), 8);
        assert_eq!(*ph[1].numer(), 7);
        assert_eq!(*ph[1].denom(), 8);
        // Two Pauli terms of modulus 1/√2, at every weight.
        let coefficients = out.remainder.pauli_coefficients::<f64>().unwrap();
        assert_eq!(coefficients.len(), 2);
        for (_, c) in &coefficients {
            assert!((c.modulus() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
        }
        // Numeric: the physical program's unitary conjugating X_0 equals X_0 · remainder.
        let u = unitary_of(w, logical_t(&gamma).unwrap());
        let numeric = conj_by(&u, &pauli_matrix(&pauli(w, &[0], &[])));
        let exact = pauli_matrix(&out.pauli)
            .matmul(&remainder_matrix(&out.remainder))
            .unwrap();
        assert!(close(&numeric, &exact), "weight {w}");
    }
}

#[test]
fn test_even_overlap_and_z_type_faults_do_not_spread() {
    let gamma = chain(4, &[0, 1, 2]);
    let program = ExactProgram::diagonal(GaugeFieldGate::t(gamma.clone()).unwrap());
    let two = program.conjugate(&pauli(4, &[0, 1], &[])).unwrap();
    assert!(two.remainder.is_constant());
    assert_eq!(two.pauli, pauli(4, &[0, 1], &[]));
    let z = program.conjugate(&pauli(4, &[], &[1])).unwrap();
    assert!(z.remainder.is_constant());
    assert_eq!(z.pauli, pauli(4, &[], &[1]));
    let off = program.conjugate(&pauli(4, &[3], &[])).unwrap();
    assert!(
        off.remainder.is_constant(),
        "a fault off the support flips no parity"
    );
}

#[test]
fn test_s_bar_leaves_a_pauli_remainder_that_the_tableau_agrees_with() {
    // S̄(γ) is Clifford, so its program can be carried either way; both give X_0 · Z̄(γ) up to phase.
    let gamma = chain(3, &[0, 1, 2]);
    let diagonal = ExactProgram::diagonal(GaugeFieldGate::s(gamma.clone()).unwrap());
    let out = diagonal.conjugate(&pauli(3, &[0], &[])).unwrap();
    // Remainder exp(iπ/2 (1 − 2p)) = i · Z̄(γ): phases 1/4 (even) and 3/4 (odd).
    assert_eq!(
        (
            *out.remainder.phases()[0].numer(),
            *out.remainder.phases()[0].denom()
        ),
        (1, 4)
    );
    assert_eq!(
        (
            *out.remainder.phases()[1].numer(),
            *out.remainder.phases()[1].denom()
        ),
        (3, 4)
    );
    let coefficients = out.remainder.pauli_coefficients::<f64>().unwrap();
    assert!(coefficients[0].1.modulus() < 1e-12, "no identity component");
    assert!((coefficients[1].1.im - 1.0).abs() < 1e-12, "i · Z̄(γ)");
    let clifford = ExactProgram::clifford(3, logical_s(&gamma)).unwrap();
    let tableau = clifford.conjugate(&pauli(3, &[0], &[])).unwrap();
    assert!(tableau.remainder.is_constant());
    assert_eq!(tableau.pauli, pauli(3, &[0], &[0, 1, 2]));
    // And numerically, up to the phase i the tableau does not carry.
    let u = unitary_of(3, logical_s(&gamma));
    let numeric = conj_by(&u, &pauli_matrix(&pauli(3, &[0], &[])));
    let scaled: Vec<C> = pauli_matrix(&tableau.pauli)
        .as_slice()
        .iter()
        .map(|v| *v * C::new(0.0, 1.0))
        .collect();
    assert!(close(&numeric, &CausalTensor::from_slice(&scaled, &[8, 8])));
}

#[test]
fn test_hadamard_program_is_clifford_and_numeric_agrees_up_to_phase() {
    let gamma = chain(3, &[0, 1, 2]);
    let dual = chain(3, &[0]);
    let (ops, _phase) = logical_hadamard::<W, f64>(&gamma, &dual).unwrap();
    let program = ExactProgram::clifford(3, ops.clone()).unwrap();
    let out = program.conjugate(&pauli(3, &[], &[0, 1, 2])).unwrap();
    assert!(out.remainder.is_constant());
    let u = unitary_of(3, ops);
    let numeric = conj_by(&u, &pauli_matrix(&pauli(3, &[], &[0, 1, 2])));
    let predicted = pauli_matrix(&out.pauli);
    // Find the phase on the first non-zero entry and compare the rest.
    let (i, p) = predicted
        .as_slice()
        .iter()
        .enumerate()
        .find(|(_, v)| v.modulus() > 0.5)
        .unwrap();
    let phase = numeric.as_slice()[i] * C::new(p.re, -p.im);
    assert!((phase.modulus() - 1.0).abs() < 1e-10);
    let scaled: Vec<C> = predicted.as_slice().iter().map(|v| *v * phase).collect();
    assert!(close(&numeric, &CausalTensor::from_slice(&scaled, &[8, 8])));
}

#[test]
fn test_program_outside_the_normal_form_is_refused_by_name() {
    let gamma = chain(3, &[0, 1, 2]);
    let t = GaugeFieldGate::t(gamma.clone()).unwrap();
    let program = ExactProgram::new(
        3,
        vec![
            ExactLayer::Diagonal(t.clone()),
            ExactLayer::Clifford(vec![GateOp::H(0), GateOp::H(1), GateOp::H(2)]),
            ExactLayer::Diagonal(t),
        ],
    )
    .unwrap();
    let err = program.conjugate(&pauli(3, &[0], &[])).unwrap_err();
    match err.0.clone() {
        QuantumErrorEnum::NoPropagationNormalForm { layer, after } => {
            assert_eq!((layer, after), (1, 0));
            let shown = format!("{err}");
            assert!(
                shown.contains("layer 1") && shown.contains("layer 0"),
                "{shown}"
            );
        }
        other => panic!("{other:?}"),
    }
    // A diagonal Clifford layer after the remainder is fine.
    let program = ExactProgram::new(
        3,
        vec![
            ExactLayer::Diagonal(GaugeFieldGate::t(gamma.clone()).unwrap()),
            ExactLayer::Clifford(vec![
                GateOp::S(0),
                GateOp::Cz {
                    control: 0,
                    target: 1,
                },
            ]),
        ],
    )
    .unwrap();
    assert!(program.conjugate(&pauli(3, &[0], &[])).is_ok());
    assert!(program.is_diagonal());
    assert!(
        !ExactProgram::<W>::clifford(3, vec![GateOp::H(0)])
            .unwrap()
            .is_diagonal()
    );
    let g = program.as_gauge_field_gate().unwrap();
    assert_eq!(g.len(), 3);
}

#[test]
fn test_construction_errors_and_gate_classification() {
    let non_clifford = ExactProgram::<W>::clifford(2, vec![GateOp::T(0)]).unwrap_err();
    assert!(matches!(
        non_clifford.0,
        QuantumErrorEnum::NonCliffordGate(_)
    ));
    let out_of_range = ExactProgram::<W>::clifford(2, vec![GateOp::H(2)]).unwrap_err();
    assert!(matches!(
        out_of_range.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let wrong_register = ExactProgram::<W>::new(
        3,
        vec![ExactLayer::Diagonal(
            GaugeFieldGate::z(chain(2, &[0])).unwrap(),
        )],
    )
    .unwrap_err();
    assert!(matches!(
        wrong_register.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let program = ExactProgram::<W>::clifford(2, vec![GateOp::H(0)]).unwrap();
    let err = program.conjugate(&pauli(3, &[0], &[])).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    assert_eq!(program.num_qubits(), 2);
    assert_eq!(program.layers().len(), 1);
    assert!(program.as_gauge_field_gate().is_err());
    assert!(is_clifford_gate(&GateOp::Cmz { qubits: vec![0, 1] }));
    assert!(!is_clifford_gate(&GateOp::Cmz {
        qubits: vec![0, 1, 2]
    }));
    assert!(!is_clifford_gate(&GateOp::Ccz {
        q0: 0,
        q1: 1,
        q2: 2
    }));
    assert!(is_diagonal_gate(&GateOp::Tdg(0)));
    assert!(!is_diagonal_gate(&GateOp::Cnot {
        control: 0,
        target: 1
    }));
}

#[test]
fn test_clifford_layer_refuses_coincident_and_empty_gates() {
    // `CZ(0, 0)` is not a gate; the tableau rule would pass it as the identity.
    for op in [
        GateOp::Cz {
            control: 0,
            target: 0,
        },
        GateOp::Cnot {
            control: 1,
            target: 1,
        },
        GateOp::Cmz { qubits: vec![] },
        GateOp::Cmz { qubits: vec![1, 1] },
    ] {
        let err = ExactProgram::<W>::clifford(2, vec![op.clone()]).unwrap_err();
        assert!(
            matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
            "{op:?}: {err}"
        );
    }
    // The same gates on distinct qubits are accepted, and the coincident one is caught in any
    // layer position, with the layer named.
    assert!(
        ExactProgram::<W>::clifford(
            2,
            vec![
                GateOp::Cz {
                    control: 0,
                    target: 1
                },
                GateOp::Cmz { qubits: vec![1, 0] }
            ]
        )
        .is_ok()
    );
    let err = ExactProgram::<W>::new(
        2,
        vec![
            ExactLayer::Clifford(vec![GateOp::H(0)]),
            ExactLayer::Clifford(vec![GateOp::Cz {
                control: 1,
                target: 1,
            }]),
        ],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("layer 1")),
        "{err}"
    );
}
