/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::abstraction::fault_set::{Fault, PauliKind};
use crate::types::circuit_model::{CircuitBox, CircuitModel, NumericCaps, WireType};
#[cfg(feature = "qcm")]
use crate::types::qcm::dem_model::{DemModel, Mechanism};
use crate::types::qpu::circuit::GateOp;
use crate::utils_tests::hand_built_complex::HandBuiltComplex;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The `[[4, 2, 2]]` code as a chain complex: two 0-cells, four 1-cells each running from the first
/// vertex to the second, and one 2-cell whose boundary is the four edges with alternating signs.
///
/// Over ℤ, `∂₁ = [[−1, −1, −1, −1], [1, 1, 1, 1]]` and `∂₂ = [1, −1, 1, −1]ᵀ`, so `∂₁ ∂₂ = 0`.
/// Over 𝔽₂ the one column of `∂₂` is the Z-check `ZZZZ`, the two rows of `∂₁` are both the X-check
/// `XXXX`, `rank ∂₁ = rank ∂₂ = 1`, and `k = 4 − 1 − 1 = 2`. Every non-trivial logical operator has
/// weight at least 2, so the distance is 2. The composite Choi of a channel from its four qubits to
/// its two logical qubits has `2^12` entries.
pub fn four_two_two() -> HandBuiltComplex {
    let d1: [(usize, usize, i8); 8] = [
        (0, 0, -1),
        (0, 1, -1),
        (0, 2, -1),
        (0, 3, -1),
        (1, 0, 1),
        (1, 1, 1),
        (1, 2, 1),
        (1, 3, 1),
    ];
    let d2: [(usize, usize, i8); 4] = [(0, 0, 1), (1, 0, -1), (2, 0, 1), (3, 0, -1)];
    HandBuiltComplex::new(vec![2, 4, 1], &[&d1, &d2])
}

/// A code with three logical qubits and no checks: one vertex and three loop edges on it, so
/// `∂₁ = 0` over ℤ, `H₁ = 𝔽₂³`, `n = k = 3`, and every logical representative is one edge. It is
/// the smallest outer code on which a three-qubit gate stays within one block.
pub fn three_three_one() -> HandBuiltComplex {
    HandBuiltComplex::new(vec![1, 3], &[&[]])
}

/// A complex with no logical qubits: two vertices joined by one edge, so `∂₁ = [−1, 1]ᵀ` has full
/// column rank and `H₁ = 0`. As a code it has `n = 1` and `k = 0`.
pub fn no_logical_qubits() -> HandBuiltComplex {
    HandBuiltComplex::new(vec![2, 1], &[&[(0, 0, -1), (1, 0, 1)]])
}

/// The small memory experiment: three data qubits under independent `X` noise `p`, one correlated
/// `X ⊗ X` channel of probability `p_c` on the first two, two parity checks through ancillas, and
/// every qubit measured. Wires `0..3` are data, `3, 4` the ancillas, `5..10` the classical record
/// `(a0, a1, m0, m1, m2)`. The noise boxes are nodes `0..4`, the correlated channel node 3, so a
/// fault "after node 3" sits between the noise and the syndrome extraction. Only `X` errors and
/// `Z`-basis readout occur, so the circuit's record is exactly a classical stochastic process:
/// `a0 = q0 ⊕ q1`, `a1 = q1 ⊕ q2`, `m_i = q_i`.
pub fn memory_experiment<R>(p: f64, p_c: f64) -> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let c = |v: f64| Complex::new(R::from_f64(v).expect("a literal"), R::zero());
    let zero = c(0.0);
    let flip = |q: f64| -> Vec<CausalTensor<Complex<R>>> {
        let stay = c((1.0 - q).sqrt());
        let go = c(q.sqrt());
        vec![
            CausalTensor::from_slice(&[stay, zero, zero, stay], &[2, 2]),
            CausalTensor::from_slice(&[zero, go, go, zero], &[2, 2]),
        ]
    };
    let two_flip = |q: f64| -> Vec<CausalTensor<Complex<R>>> {
        let stay = c((1.0 - q).sqrt());
        let go = c(q.sqrt());
        let mut identity = vec![zero; 16];
        let mut xx = vec![zero; 16];
        for i in 0..4 {
            identity[i * 4 + i] = stay;
            xx[i * 4 + (i ^ 3)] = go;
        }
        vec![
            CausalTensor::from_slice(&identity, &[4, 4]),
            CausalTensor::from_slice(&xx, &[4, 4]),
        ]
    };
    let mut wires = vec![WireType::qubit(); 5];
    wires.extend(core::iter::repeat_n(WireType::bit(), 5));
    let boxes = vec![
        CircuitBox::Kraus {
            wires: vec![0],
            kraus: flip(p),
        },
        CircuitBox::Kraus {
            wires: vec![1],
            kraus: flip(p),
        },
        CircuitBox::Kraus {
            wires: vec![2],
            kraus: flip(p),
        },
        CircuitBox::Kraus {
            wires: vec![0, 1],
            kraus: two_flip(p_c),
        },
        CircuitBox::Unitary {
            wires: vec![0, 1, 3],
            program: vec![
                GateOp::Cnot {
                    control: 0,
                    target: 2,
                },
                GateOp::Cnot {
                    control: 1,
                    target: 2,
                },
            ],
        },
        CircuitBox::Unitary {
            wires: vec![1, 2, 4],
            program: vec![
                GateOp::Cnot {
                    control: 0,
                    target: 2,
                },
                GateOp::Cnot {
                    control: 1,
                    target: 2,
                },
            ],
        },
        CircuitBox::Measurement {
            wires: vec![3],
            outcome: 5,
        },
        CircuitBox::Measurement {
            wires: vec![4],
            outcome: 6,
        },
        CircuitBox::Measurement {
            wires: vec![0],
            outcome: 7,
        },
        CircuitBox::Measurement {
            wires: vec![1],
            outcome: 8,
        },
        CircuitBox::Measurement {
            wires: vec![2],
            outcome: 9,
        },
    ];
    CircuitModel::ungrouped(wires, boxes, vec![], vec![5, 6, 7, 8, 9]).expect("a valid circuit")
}

/// The caps the memory experiment evaluates under: its record opens `2^5` measurement branches on
/// top of the noise branches, and the operator cap counts the traced basis states before the zero
/// operators are dropped.
pub fn memory_experiment_caps() -> NumericCaps {
    NumericCaps {
        max_entries: 1 << 24,
        max_operators: 1 << 20,
    }
}

/// The fault locations of the memory experiment, after the last noise node: `X` on each data qubit
/// and the correlated `X ⊗ X` on qubits 0 and 1.
pub fn memory_locations() -> Vec<Fault> {
    vec![
        Fault::new(Some(3), vec![(0, PauliKind::X)]).expect("a fault"),
        Fault::new(Some(3), vec![(1, PauliKind::X)]).expect("a fault"),
        Fault::new(Some(3), vec![(2, PauliKind::X)]).expect("a fault"),
        Fault::new(Some(3), vec![(0, PauliKind::X), (1, PauliKind::X)]).expect("a fault"),
    ]
}

/// The decoder's reading of the record: detectors `D0 = a0`, `D1 = a1`, observable `L0 = m0`, a
/// deterministic `32 × 8` stochastic matrix over the strings `(a0, a1, m0, m1, m2)` and
/// `(D0, D1, L0)`, first bit most significant.
pub fn memory_tau() -> Vec<Vec<f64>> {
    (0..32)
        .map(|x: usize| {
            let a0 = (x >> 4) & 1;
            let a1 = (x >> 3) & 1;
            let m0 = (x >> 2) & 1;
            let y = (a0 << 2) | (a1 << 1) | m0;
            (0..8).map(|k| if k == y { 1.0 } else { 0.0 }).collect()
        })
        .collect()
}

/// The detector error model of the memory experiment: `X` on `q0` flips `D0, L0`, on `q1` flips
/// `D0, D1`, on `q2` flips `D1`, and the correlated `X ⊗ X` on `q0, q1` flips `D1, L0` (`D0` twice
/// cancels); the last is present only when `complete` is set.
#[cfg(feature = "qcm")]
pub fn memory_dem(p: f64, p_c: f64, complete: bool) -> DemModel {
    let mut mechanisms = vec![
        Mechanism {
            probability: p,
            detectors: vec![0],
            observables: vec![0],
        },
        Mechanism {
            probability: p,
            detectors: vec![0, 1],
            observables: vec![],
        },
        Mechanism {
            probability: p,
            detectors: vec![1],
            observables: vec![],
        },
    ];
    if complete {
        mechanisms.push(Mechanism {
            probability: p_c,
            detectors: vec![1],
            observables: vec![0],
        });
    }
    DemModel::new(mechanisms, 2, 1).expect("a valid model")
}
