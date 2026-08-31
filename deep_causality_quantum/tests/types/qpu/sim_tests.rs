/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qpu")]

use deep_causality_quantum::{GateOp, QpuSampler, QuantumCircuit, ShotHistogram, SimQpu};

fn bell() -> QuantumCircuit {
    QuantumCircuit::new(
        2,
        vec![
            GateOp::H(0),
            GateOp::Cnot {
                control: 0,
                target: 1,
            },
        ],
        vec![0, 1],
    )
    .unwrap()
}

#[test]
fn test_bell_counts_sum_to_shots_and_are_correlated() {
    let sim = SimQpu::new(0xC0FFEE);
    let hist = sim.sample(&bell(), 2000).unwrap();
    assert_eq!(hist.total(), 2000);
    assert_eq!(hist.num_bits(), 2);
    // A Bell state only produces the correlated outcomes 00 and 11.
    assert_eq!(hist.count(0b01), 0);
    assert_eq!(hist.count(0b10), 0);
    assert!(hist.count(0b00) > 0);
    assert!(hist.count(0b11) > 0);
    assert_eq!(hist.count(0b00) + hist.count(0b11), 2000);
    // Roughly balanced (well within tolerance for 2000 shots).
    let p00 = hist.count(0b00) as f64 / 2000.0;
    assert!((0.4..0.6).contains(&p00), "unbalanced: {}", p00);
}

#[test]
fn test_same_seed_reproduces_same_histogram() {
    let a = SimQpu::new(42).sample(&bell(), 500).unwrap();
    let b = SimQpu::new(42).sample(&bell(), 500).unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_different_seeds_yield_valid_bell_histograms() {
    let a = SimQpu::new(1).sample(&bell(), 500).unwrap();
    let b = SimQpu::new(999_999).sample(&bell(), 500).unwrap();
    // Assert the Bell invariant, not an accidental inequality: two independent
    // Binomial(500, 0.5) draws of count(0b00) coincide ~2.5% of the time, so
    // `assert_ne!` would flake. Per-seed determinism is covered by
    // test_same_seed_reproduces_same_histogram.
    for h in [&a, &b] {
        assert_eq!(h.count(0b01), 0);
        assert_eq!(h.count(0b10), 0);
        assert_eq!(h.count(0b00) + h.count(0b11), 500);
    }
}

#[test]
fn test_x_gate_is_deterministic_one() {
    // X|0> = |1>: measuring qubit 0 always yields 1.
    let circuit = QuantumCircuit::new(1, vec![GateOp::X(0)], vec![0]).unwrap();
    let hist = SimQpu::new(7).sample(&circuit, 100).unwrap();
    assert_eq!(hist.count(1), 100);
    assert_eq!(hist.count(0), 0);
}

#[test]
fn test_zero_shots_yields_empty_histogram() {
    let hist = SimQpu::new(7).sample(&bell(), 0).unwrap();
    assert_eq!(hist.total(), 0);
    assert!(hist.entries().is_empty());
}

#[test]
fn test_no_amplitudes_are_exposed() {
    // The ShotHistogram surface is classical counts only — this test documents
    // that the public API offers no amplitude accessor (it would not compile).
    let hist = SimQpu::new(7).sample(&bell(), 10).unwrap();
    let _entries: Vec<(usize, u64)> = hist.entries();
    // (No `hist.amplitude(..)` exists.)
}

#[test]
fn test_gh_z_three_qubit_correlation() {
    // GHZ: H(0), CNOT(0,1), CNOT(1,2) → only 000 and 111.
    let circuit = QuantumCircuit::new(
        3,
        vec![
            GateOp::H(0),
            GateOp::Cnot {
                control: 0,
                target: 1,
            },
            GateOp::Cnot {
                control: 1,
                target: 2,
            },
        ],
        vec![0, 1, 2],
    )
    .unwrap();
    let hist = SimQpu::new(0xABCD).sample(&circuit, 1000).unwrap();
    assert_eq!(hist.count(0b000) + hist.count(0b111), 1000);
}

// ---------------------------------------------------------------------------
// The diagonal gate family: Z, S, S†, T, T†, CZ, CS†, CCZ, C^(m-1)Z.
//
// A diagonal gate multiplies a phase and moves no amplitude, so it is invisible
// on a computational basis state. Each test below makes one observable by
// conjugating with H, and each expected outcome is a deterministic basis state
// fixed by a published gate identity rather than by anything this simulator
// produced.
//
// Reference: Nielsen & Chuang, *Quantum Computation and Quantum Information*,
// 10th anniversary edition. §4.2 gives the single-qubit relations HZH = X,
// S = T^2 and Z = S^2; §4.3 and Fig. 4.4 give the conjugation identity
// CNOT = (I ⊗ H) · CZ · (I ⊗ H), whose m-qubit form turns C^(m-1)Z into
// C^(m-1)X.
// ---------------------------------------------------------------------------

/// Runs `ops` on `n` qubits, measuring all of them LSB-first, and asserts that
/// every shot lands on `expected`. Only used where the outcome is deterministic,
/// so a wrong phase shows up as a different basis state rather than as a shifted
/// distribution.
fn assert_deterministic(n: usize, ops: Vec<GateOp>, expected: usize) {
    let measure: Vec<usize> = (0..n).collect();
    let circuit = QuantumCircuit::new(n, ops, measure).unwrap();
    let hist = SimQpu::new(0xABCD).sample(&circuit, 64).unwrap();
    assert_eq!(
        hist.count(expected),
        64,
        "expected every shot on {:#b}, got {:?}",
        expected,
        hist.entries()
    );
}

#[test]
fn test_h_z_h_is_x() {
    // HZH = X, so the sandwich turns |0> into |1> with certainty.
    assert_deterministic(1, vec![GateOp::H(0), GateOp::Z(0), GateOp::H(0)], 1);
}

#[test]
fn test_s_squared_is_z() {
    // S^2 = Z, so H S S H |0> = |1>. A wrong S phase breaks the interference and
    // the outcome stops being deterministic at all.
    assert_deterministic(
        1,
        vec![GateOp::H(0), GateOp::S(0), GateOp::S(0), GateOp::H(0)],
        1,
    );
}

#[test]
fn test_t_squared_is_s() {
    // T^2 = S, so T T S† = I and the sandwich returns |0>.
    assert_deterministic(
        1,
        vec![
            GateOp::H(0),
            GateOp::T(0),
            GateOp::T(0),
            GateOp::Sdg(0),
            GateOp::H(0),
        ],
        0,
    );
}

#[test]
fn test_adjoints_invert_their_gates() {
    // S S† = I and T T† = I. These pin the sign of the adjoint phases: a phase
    // conjugated the wrong way gives S S = Z here, and the outcome flips to |1>.
    assert_deterministic(
        1,
        vec![GateOp::H(0), GateOp::S(0), GateOp::Sdg(0), GateOp::H(0)],
        0,
    );
    assert_deterministic(
        1,
        vec![GateOp::H(0), GateOp::T(0), GateOp::Tdg(0), GateOp::H(0)],
        0,
    );
}

#[test]
fn test_eight_t_gates_are_the_identity() {
    // T has order 8: (e^{iπ/4})^8 = e^{2πi} = 1. This catches a T phase that is
    // right to a few digits but wrong in the exponent.
    let mut ops = vec![GateOp::H(0)];
    ops.extend(core::iter::repeat_n(GateOp::T(0), 8));
    ops.push(GateOp::H(0));
    assert_deterministic(1, ops, 0);
}

#[test]
fn test_diagonal_gates_move_no_amplitude() {
    // Without interference a diagonal gate is unobservable: it must leave a
    // computational basis state exactly where it was.
    assert_deterministic(1, vec![GateOp::X(0), GateOp::Z(0)], 1);
    assert_deterministic(1, vec![GateOp::X(0), GateOp::S(0), GateOp::T(0)], 1);
    assert_deterministic(
        3,
        vec![
            GateOp::X(0),
            GateOp::X(1),
            GateOp::X(2),
            GateOp::Ccz {
                q0: 0,
                q1: 1,
                q2: 2,
            },
        ],
        0b111,
    );
}

#[test]
fn test_cz_conjugated_by_h_is_cnot() {
    // (I ⊗ H) CZ (I ⊗ H) = CNOT. With the control set the target flips; with the
    // control clear it does not. Both halves are needed: a CZ that phased
    // unconditionally would pass the first and fail the second.
    assert_deterministic(
        2,
        vec![
            GateOp::X(0),
            GateOp::H(1),
            GateOp::Cz {
                control: 0,
                target: 1,
            },
            GateOp::H(1),
        ],
        0b11,
    );
    assert_deterministic(
        2,
        vec![
            GateOp::H(1),
            GateOp::Cz {
                control: 0,
                target: 1,
            },
            GateOp::H(1),
        ],
        0b00,
    );
}

#[test]
fn test_csdg_applied_twice_is_cz() {
    // (CS†)^2 = CZ, so conjugating two CS† by H reproduces the CNOT action.
    assert_deterministic(
        2,
        vec![
            GateOp::X(0),
            GateOp::H(1),
            GateOp::Csdg {
                control: 0,
                target: 1,
            },
            GateOp::Csdg {
                control: 0,
                target: 1,
            },
            GateOp::H(1),
        ],
        0b11,
    );
    // One CS† is not CZ: the target does not fully flip, so the outcome is not
    // deterministic and 0b11 cannot take every shot.
    let circuit = QuantumCircuit::new(
        2,
        vec![
            GateOp::X(0),
            GateOp::H(1),
            GateOp::Csdg {
                control: 0,
                target: 1,
            },
            GateOp::H(1),
        ],
        vec![0, 1],
    )
    .unwrap();
    let hist = SimQpu::new(0xABCD).sample(&circuit, 64).unwrap();
    assert!(hist.count(0b11) < 64);
}

#[test]
fn test_ccz_conjugated_by_h_is_toffoli() {
    // The three-qubit form of the same identity: CCZ conjugated by H on one leg
    // is the Toffoli gate. The target flips only when both other qubits are set.
    assert_deterministic(
        3,
        vec![
            GateOp::X(0),
            GateOp::X(1),
            GateOp::H(2),
            GateOp::Ccz {
                q0: 0,
                q1: 1,
                q2: 2,
            },
            GateOp::H(2),
        ],
        0b111,
    );
    // One control clear: no flip. This is the case a gate that ignored its
    // controls would fail.
    assert_deterministic(
        3,
        vec![
            GateOp::X(0),
            GateOp::H(2),
            GateOp::Ccz {
                q0: 0,
                q1: 1,
                q2: 2,
            },
            GateOp::H(2),
        ],
        0b001,
    );
}

#[test]
fn test_cmz_reproduces_cz_and_ccz_at_their_arities() {
    // C^(m-1)Z is the general form; CZ and CCZ are its two- and three-qubit
    // cases. Same circuits as above with the fixed-arity gates swapped out.
    assert_deterministic(
        2,
        vec![
            GateOp::X(0),
            GateOp::H(1),
            GateOp::Cmz { qubits: vec![0, 1] },
            GateOp::H(1),
        ],
        0b11,
    );
    assert_deterministic(
        3,
        vec![
            GateOp::X(0),
            GateOp::X(1),
            GateOp::H(2),
            GateOp::Cmz {
                qubits: vec![0, 1, 2],
            },
            GateOp::H(2),
        ],
        0b111,
    );
}

#[test]
fn test_cmz_at_four_qubits_is_c3x_under_conjugation() {
    // The arity the fixed variants cannot express. C^3Z on four qubits,
    // conjugated by H on the last, flips it only when the other three are set.
    let controls = vec![GateOp::X(0), GateOp::X(1), GateOp::X(2)];
    let mut all_set = controls.clone();
    all_set.extend(vec![
        GateOp::H(3),
        GateOp::Cmz {
            qubits: vec![0, 1, 2, 3],
        },
        GateOp::H(3),
    ]);
    assert_deterministic(4, all_set, 0b1111);

    // Drop one control: the target must stay clear.
    assert_deterministic(
        4,
        vec![
            GateOp::X(0),
            GateOp::X(1),
            GateOp::H(3),
            GateOp::Cmz {
                qubits: vec![0, 1, 2, 3],
            },
            GateOp::H(3),
        ],
        0b0011,
    );
}

#[test]
fn test_cmz_at_one_qubit_is_z() {
    // The degenerate arity: C^0Z is Z itself.
    assert_deterministic(
        1,
        vec![GateOp::H(0), GateOp::Cmz { qubits: vec![0] }, GateOp::H(0)],
        1,
    );
}
