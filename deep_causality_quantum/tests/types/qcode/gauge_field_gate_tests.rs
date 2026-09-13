/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The gauge-field carrier against Haruna, arXiv:2511.15224, Table 1 column 3 and Eq. (3.63).
//!
//! Provenance: `Z̄ = e^{iπ a}` is half a turn on an odd overlap; `S̄ = e^{iπ/2 a²}` a quarter;
//! `T̄` an eighth (Eq. 3.61 read on integers: `(2n³ − 3n² + 2n)/8 ≡ (n mod 2)/8`, checked by hand at
//! `n = 0..5` in the change's notes); `CZ̄ = e^{iπ a(γ₁) a(γ₂)}` half a turn when both are odd.
//! `S² = Z` and `T² = S` are the Clifford-hierarchy identities. Corner cases: (A) the identity with
//! no blocks, (B) one block, (C) the same block on both sides of a product, (D) mismatched
//! registers, (F) the block limit, (G) a polynomial that is not a parity function, (H) a phase
//! reduced from a negative turn.

use deep_causality_homology::Gf2Chain;
use deep_causality_num_rational::Rational;
use deep_causality_quantum::{
    DiagonalPhase, GateOp, GaugeFieldGate, MAX_GAUGE_BLOCKS, QuantumErrorEnum, reduce_turns,
};

type W = u64;

fn chain(n: usize, support: &[usize]) -> Gf2Chain<W> {
    Gf2Chain::from_support(n, 1, support).unwrap()
}

fn turns(n: i64, d: i64) -> Rational<i64> {
    Rational::new(n, d)
}

#[test]
fn test_table_one_phases() {
    let g = chain(4, &[0, 1, 2]);
    let z = GaugeFieldGate::z(g.clone()).unwrap();
    assert_eq!(z.phases(), &[turns(0, 1), turns(1, 2)]);
    let s = GaugeFieldGate::s(g.clone()).unwrap();
    assert_eq!(s.phases(), &[turns(0, 1), turns(1, 4)]);
    let t = GaugeFieldGate::t(g.clone()).unwrap();
    assert_eq!(t.phases(), &[turns(0, 1), turns(1, 8)]);
    let cz = GaugeFieldGate::cz(g.clone(), chain(4, &[3])).unwrap();
    assert_eq!(
        cz.phases(),
        &[turns(0, 1), turns(0, 1), turns(0, 1), turns(1, 2)]
    );
    assert_eq!(cz.num_blocks(), 2);
    assert_eq!(cz.phase_at(3), turns(1, 2));
    assert_eq!(cz.phase_at(1), turns(0, 1));
    let ccz = GaugeFieldGate::multi_cz(vec![g.clone(), chain(4, &[3]), chain(4, &[1, 3])]).unwrap();
    assert_eq!(ccz.phases().len(), 8);
    assert_eq!(ccz.phase_at(7), turns(1, 2));
    assert_eq!(z.len(), 4);
    assert_eq!(z.degree(), 1);
    assert!(!z.is_empty());
}

#[test]
fn test_hierarchy_identities_through_product() {
    let g = chain(3, &[0, 1, 2]);
    let s = GaugeFieldGate::s(g.clone()).unwrap();
    let z = GaugeFieldGate::z(g.clone()).unwrap();
    assert_eq!(s.product(&s).unwrap(), z, "S̄² = Z̄");
    let t = GaugeFieldGate::t(g.clone()).unwrap();
    assert_eq!(t.product(&t).unwrap(), s, "T̄² = S̄");
    assert_eq!(
        z.product(&z).unwrap().phases(),
        &[turns(0, 1), turns(0, 1)],
        "Z̄² = I"
    );
    // Products over different blocks take the union.
    let cz = GaugeFieldGate::cz(g.clone(), chain(3, &[0])).unwrap();
    let p = z.product(&cz).unwrap();
    assert_eq!(p.num_blocks(), 2);
    assert_eq!(p.phase_at(0b01), turns(1, 2), "γ odd alone: Z̄");
    assert_eq!(p.phase_at(0b11), turns(0, 1), "both odd: Z̄ and CZ̄ cancel");
    let id = GaugeFieldGate::<W>::identity(3, 1);
    assert_eq!(id.product(&t).unwrap(), t);
    assert_eq!(t.product(&id).unwrap(), t);
    assert!(id.is_constant());
}

#[test]
fn test_conjugation_flips_parities() {
    let g = chain(3, &[0, 1, 2]);
    let t = GaugeFieldGate::t(g.clone()).unwrap();
    assert_eq!(t.parity_flips(&chain(3, &[0])).unwrap(), 1);
    assert_eq!(t.parity_flips(&chain(3, &[0, 1])).unwrap(), 0);
    let r = t.conjugated_by_pauli(&chain(3, &[0])).unwrap();
    assert_eq!(r.phases(), &[turns(1, 8), turns(7, 8)]);
    assert!(!r.is_constant());
    assert!(
        t.conjugated_by_pauli(&chain(3, &[0, 1]))
            .unwrap()
            .is_constant()
    );
    assert!(t.conjugated_by_pauli(&chain(3, &[])).unwrap().is_constant());
    let cz = GaugeFieldGate::cz(g, chain(3, &[0])).unwrap();
    // A fault on qubit 1 flips block 1 only: remainder is Z̄ on block 2 when block 1... phases:
    // φ(idx ⊕ 1) − φ(idx): pattern 0b10 (γ₂ odd) gains 1/2, pattern 0b11 loses 1/2 ≡ 1/2.
    let r = cz.conjugated_by_pauli(&chain(3, &[1])).unwrap();
    assert_eq!(
        r.phases(),
        &[turns(0, 1), turns(0, 1), turns(1, 2), turns(1, 2)]
    );
    assert!(!r.is_constant());
}

#[test]
fn test_pauli_coefficients_of_the_t_remainder() {
    let g = chain(5, &[0, 1, 2, 3, 4]);
    let r = GaugeFieldGate::t(g)
        .unwrap()
        .conjugated_by_pauli(&chain(5, &[2]))
        .unwrap();
    let c = r.pauli_coefficients::<f64>().unwrap();
    assert_eq!(c.len(), 2);
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert!(
        (c[0].1.re - s).abs() < 1e-12 && c[0].1.im.abs() < 1e-12,
        "identity coefficient cos(π/4)"
    );
    assert!(
        (c[1].1.im - s).abs() < 1e-12 && c[1].1.re.abs() < 1e-12,
        "Z̄ coefficient i sin(π/4)"
    );
}

#[test]
fn test_from_diagonal_phase_bridges_the_polynomial_carrier() {
    let g = chain(6, &[0, 1, 2, 3, 4, 5]);
    for (poly, gate) in [
        (
            DiagonalPhase::z(g.clone()),
            GaugeFieldGate::z(g.clone()).unwrap(),
        ),
        (
            DiagonalPhase::s(g.clone()),
            GaugeFieldGate::s(g.clone()).unwrap(),
        ),
        (
            DiagonalPhase::t(g.clone()),
            GaugeFieldGate::t(g.clone()).unwrap(),
        ),
    ] {
        assert_eq!(GaugeFieldGate::from_diagonal_phase(&poly).unwrap(), gate);
    }
    // `n²/8` is not a parity function: n = 2 gives 1/2, n = 0 gives 0.
    let not_parity = DiagonalPhase::new(g, vec![0, 0, 1], 3).unwrap();
    let err = GaugeFieldGate::from_diagonal_phase(&not_parity).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("overlap 2")));
}

#[test]
fn test_from_diagonal_clifford() {
    let g = GaugeFieldGate::<W>::from_diagonal_clifford(2, &[GateOp::S(0), GateOp::S(0)]).unwrap();
    assert_eq!(g, GaugeFieldGate::z(chain(2, &[0])).unwrap());
    let g =
        GaugeFieldGate::<W>::from_diagonal_clifford(2, &[GateOp::S(1), GateOp::Sdg(1)]).unwrap();
    assert!(g.is_constant());
    let cz = GaugeFieldGate::<W>::from_diagonal_clifford(
        2,
        &[GateOp::Cz {
            control: 0,
            target: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        cz,
        GaugeFieldGate::cz(chain(2, &[0]), chain(2, &[1])).unwrap()
    );
    let cmz = GaugeFieldGate::<W>::from_diagonal_clifford(2, &[GateOp::Cmz { qubits: vec![1, 0] }])
        .unwrap();
    assert_eq!(cmz.phase_at(3), turns(1, 2));
    let z = GaugeFieldGate::<W>::from_diagonal_clifford(1, &[GateOp::Z(0)]).unwrap();
    assert_eq!(z, GaugeFieldGate::z(chain(1, &[0])).unwrap());
    let err = GaugeFieldGate::<W>::from_diagonal_clifford(2, &[GateOp::H(0)]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}

#[test]
fn test_construction_errors_and_reduction() {
    assert_eq!(reduce_turns(turns(-1, 8)), turns(7, 8));
    assert_eq!(reduce_turns(turns(9, 8)), turns(1, 8));
    assert_eq!(reduce_turns(turns(1, -2)), turns(1, 2));
    assert!(GaugeFieldGate::<W>::o_k(vec![], 1).is_err());
    assert!(GaugeFieldGate::<W>::o_k(vec![chain(2, &[0])], 61).is_err());
    let wrong_len =
        GaugeFieldGate::<W>::new(2, 1, vec![chain(2, &[0])], vec![turns(0, 1)]).unwrap_err();
    assert!(matches!(
        wrong_len.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let mixed =
        GaugeFieldGate::<W>::new(2, 1, vec![chain(3, &[0])], vec![turns(0, 1), turns(0, 1)])
            .unwrap_err();
    assert!(matches!(mixed.0, QuantumErrorEnum::DimensionMismatch(_)));
    let too_many: Vec<Gf2Chain<W>> = (0..=MAX_GAUGE_BLOCKS).map(|q| chain(20, &[q])).collect();
    assert!(GaugeFieldGate::<W>::multi_cz(too_many).is_err());
    let a = GaugeFieldGate::z(chain(2, &[0])).unwrap();
    let b = GaugeFieldGate::z(chain(3, &[0])).unwrap();
    assert!(matches!(
        a.product(&b).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert!(matches!(
        a.parity_flips(&chain(3, &[0])).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // Products that would exceed the block limit are refused.
    let many_a: Vec<Gf2Chain<W>> = (0..MAX_GAUGE_BLOCKS).map(|q| chain(40, &[q])).collect();
    let many_b: Vec<Gf2Chain<W>> = (MAX_GAUGE_BLOCKS..2 * MAX_GAUGE_BLOCKS)
        .map(|q| chain(40, &[q]))
        .collect();
    let ga = GaugeFieldGate::multi_cz(many_a).unwrap();
    let gb = GaugeFieldGate::multi_cz(many_b).unwrap();
    assert!(ga.product(&gb).is_err());
}

#[test]
fn test_from_diagonal_program_reads_table_one_back_and_rejects_an_incomplete_one() {
    use deep_causality_quantum::{logical_cz, logical_s, logical_t};
    let g = chain(5, &[0, 1, 2]);
    let s = GaugeFieldGate::<W>::from_diagonal_program(5, &logical_s(&g), vec![g.clone()]).unwrap();
    assert_eq!(s, GaugeFieldGate::s(g.clone()).unwrap());
    let t = GaugeFieldGate::<W>::from_diagonal_program(5, &logical_t(&g).unwrap(), vec![g.clone()])
        .unwrap();
    assert_eq!(t, GaugeFieldGate::t(g.clone()).unwrap());
    let h = chain(5, &[3, 4]);
    let cz = GaugeFieldGate::<W>::from_diagonal_program(
        5,
        &logical_cz(&g, &h).unwrap(),
        vec![g.clone(), h.clone()],
    )
    .unwrap();
    assert_eq!(cz, GaugeFieldGate::cz(g.clone(), h.clone()).unwrap());
    // S on the support without the CZ pairs is n/4: two set qubits give 1/2 where parity gives 0.
    let only_s: Vec<GateOp> = g.support().map(GateOp::S).collect();
    let err = GaugeFieldGate::<W>::from_diagonal_program(5, &only_s, vec![g.clone()]).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not a function of the block parities"))
    );
    // A non-diagonal gate, a gate off the blocks, and a wrong register are refused.
    let err = GaugeFieldGate::<W>::from_diagonal_program(5, &[GateOp::H(0)], vec![g.clone()])
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not diagonal"))
    );
    let err = GaugeFieldGate::<W>::from_diagonal_program(5, &[GateOp::Z(4)], vec![g.clone()])
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("in no block"))
    );
    let err = GaugeFieldGate::<W>::from_diagonal_program(4, &[GateOp::Z(0)], vec![g.clone()])
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    // Two identical blocks leave the odd/even mixed patterns unreachable.
    let err =
        GaugeFieldGate::<W>::from_diagonal_program(5, &[GateOp::Z(0)], vec![g.clone(), g.clone()])
            .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("unreachable"))
    );
    // An empty program on a block is the identity table.
    let id = GaugeFieldGate::<W>::from_diagonal_program(5, &[], vec![g]).unwrap();
    assert!(id.is_constant());
}
