/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Naturality on abstractions whose residuals have closed forms.
//!
//! Low level: two qubits, `R_y(0.7) ⊗ I` as the first node and a gate as the second. High level:
//! one qubit, `R_y(0.7)`. `τ = Tr_B`, `E` prepares `|0⟩_B`. With `X` on the second qubit the square
//! commutes exactly. With `CNOT(0 → 1)` after the rotation the residual is `2√2` for every rotation:
//! pre-composition with the unitary `R ⊗ I` conjugates both Choi operators and drops out, leaving
//! `‖J(Tr_B ∘ CNOT) − J(Tr_B)‖_F`. With Kraus operators `K_b = I ⊗ ⟨b|`, each Choi has squared
//! norm `Σ_{b,b'} |Tr K_b† K_b'|² = 2² · 2 = 8`, and the cross term is
//! `Σ_{b,b'} |Tr(CNOT (I ⊗ |b⟩⟨b'|))|² = 4 · 1²`, since `CNOT = P₀ ⊗ I + P₁ ⊗ X` contributes `1`
//! to every `(b, b')`. So the squared residual is `8 + 8 − 2·4 = 8`.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Abstraction, AlignmentSide, Axis, Channel, CheckVerdict, CircuitBox, CircuitModel,
    FROBENIUS_ON_CHOI, Fault, GateOp, NumericCaps, PauliKind, QcMorphism, QuantumErrorEnum,
    QubitOperator, Query, QuerySignature, SemanticsPath, SquareParts, StructureScope,
    TypeAlignment, WireType, stochastic_morphism,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ry(wire: usize, theta: f64) -> CircuitBox<f64> {
    CircuitBox::Channel {
        wires: vec![wire],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

/// `R_y(θ) ⊗ I` as one box on both low-level wires, so the node covers the aligned block.
fn ry_block(theta: f64) -> CircuitBox<f64> {
    let r = QubitOperator::rotation(Axis::Y, theta).unwrap();
    let one = C::new(1.0, 0.0);
    let zero = C::new(0.0, 0.0);
    let identity = CausalTensor::from_slice(&[one, zero, zero, one], &[2, 2]);
    CircuitBox::Channel {
        wires: vec![0, 1],
        channel: Channel::from_kraus(&[r.matrix().kronecker(&identity).unwrap()]).unwrap(),
    }
}

fn trace_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    let k0 = CausalTensor::from_slice(&[one, zero, zero, zero, zero, zero, one, zero], &[2, 4]);
    let k1 = CausalTensor::from_slice(&[zero, one, zero, zero, zero, zero, zero, one], &[2, 4]);
    QcMorphism::from_kraus(&[k0, k1]).unwrap()
}

fn prepare_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[one, zero, zero, zero, zero, one, zero, zero],
        &[4, 2],
    )])
    .unwrap()
}

fn low(second: GateOp) -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.7),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![second],
            },
        ],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap()
}

fn high() -> CircuitModel<f64> {
    CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, 0.7)], vec![0], vec![0]).unwrap()
}

fn alignment() -> TypeAlignment<f64> {
    TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap()
}

#[test]
fn test_a_commuting_square_has_zero_residual_on_io_and_open() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![
            (Query::Io, Query::Io),
            (Query::Open(vec![0]), Query::Open(vec![0])),
        ],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Accepted);
    assert_eq!(r.report.examined(), 2);
    assert_eq!(r.path, SemanticsPath::Numeric);
    assert_eq!(r.norm, FROBENIUS_ON_CHOI);
    assert!(r.worst_residual() < 1e-12);
    assert!(r.bound.upper < 1e-11);
    assert!(r.entries > 0);
    assert_eq!(a.image(&Query::Io), Some(&Query::Io));
    assert_eq!(a.signature().len(), 2);
    assert_eq!(a.query_map().len(), 2);
    assert_eq!(a.low().boxes().len(), 2);
    assert_eq!(a.high().boxes().len(), 1);
    assert_eq!(a.alignment().entries().len(), 1);
}

#[test]
fn test_an_entangling_gate_breaks_the_square_by_root_two() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::Cnot {
            control: 0,
            target: 1,
        }),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Rejected);
    let rejected = r.report.first_rejection().unwrap();
    assert_eq!(rejected.item, deep_causality_quantum::CheckItem::Index(0));
    assert!(
        (rejected.measured - 8f64.sqrt()).abs() < 1e-10,
        "{}",
        rejected.measured
    );
    // The bound brackets the residual's diamond distance: √2/4 ≤ ⋄ ≤ √2·√8.
    assert!((r.bound.lower - 8f64.sqrt() / 4.0).abs() < 1e-10);
    assert!((r.bound.upper - 8.0).abs() < 1e-10);
    // The same low-level model with the CNOT on the other side of the rotation? Same residual:
    // the dephasing kills the same off-diagonals of any unitary's Choi operator.
    let a2 = Abstraction::new(
        CircuitModel::ungrouped(
            vec![WireType::qubit(), WireType::qubit()],
            vec![
                ry_block(1.3),
                CircuitBox::Unitary {
                    wires: vec![0, 1],
                    program: vec![GateOp::Cnot {
                        control: 0,
                        target: 1,
                    }],
                },
            ],
            vec![0, 1],
            vec![0, 1],
        )
        .unwrap(),
        CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, 1.3)], vec![0], vec![0])
            .unwrap(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!((a2.check_naturality(&caps).unwrap().worst_residual() - 8f64.sqrt()).abs() < 1e-10);
}

#[test]
fn test_empty_signature_is_vacuous_and_a_double_mapping_is_refused() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(low(GateOp::X(1)), high(), alignment(), vec![]).unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Vacuous);
    assert_eq!(r.report.examined(), 0);
    assert_eq!(r.worst_residual(), 0.0);
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io), (Query::Io, Query::Io)],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("mapped twice"))
    );
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Open(vec![9]))],
    )
    .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!(matches!(
        a.square(&Query::Observe(vec![0]), &caps).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
}

#[test]
fn test_observe_and_a_failing_swapped_program() {
    let caps = NumericCaps::default();
    // A spectator wire the alignment does not cover: low wire 1 is fresh and never an output, so
    // the aligned type of the high qubit is low wire 0 alone and observing it on both sides gives
    // one classical bit each.
    let identity = QcMorphism::<f64>::identity(2).unwrap();
    let spectator = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry(0, 0.7),
            CircuitBox::Unitary {
                wires: vec![1],
                program: vec![GateOp::X(0)],
            },
        ],
        vec![0],
        vec![0],
    )
    .unwrap();
    let a = Abstraction::new(
        spectator,
        high(),
        TypeAlignment::new(vec![(vec![0], vec![0], identity.clone(), identity)]).unwrap(),
        vec![
            (Query::Observe(vec![0]), Query::Observe(vec![0])),
            (Query::Io, Query::Io),
        ],
    )
    .unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(
        r.report.verdict(),
        CheckVerdict::Accepted,
        "{:?}",
        r.report.worst()
    );
    // Mapping the high-level rotation to a low-level model with a different angle names the query.
    let wrong = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.2),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![GateOp::X(1)],
            },
        ],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    // With the block alignment, observing low wire 0 alone leaves low wire 1 as a quantum output
    // that π of the high-level type does not have: the square is ill-typed and construction says
    // so.
    let ill = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Observe(vec![0]), Query::Observe(vec![0]))],
    )
    .unwrap_err();
    assert!(matches!(
        ill.0,
        QuantumErrorEnum::CalculationError(ref m) if m.contains("ill-typed")
    ));
    let a = Abstraction::new(wrong, high(), alignment(), vec![(Query::Io, Query::Io)]).unwrap();
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Rejected);
    assert_eq!(
        r.report.first_rejection().unwrap().item,
        deep_causality_quantum::CheckItem::Index(0)
    );
}

#[test]
fn test_concrete_do_is_derived_and_agrees_on_both_sides() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Open(vec![0]), Query::Open(vec![0]))],
    )
    .unwrap();
    // The opened high-level type is (input 0, fresh 1): a two-qubit state.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let state = [
        C::new(s, 0.0),
        C::new(0.0, 0.0),
        C::new(0.0, 0.0),
        C::new(s, 0.0),
    ];
    let (left, right) = a.concrete_do(&Query::Open(vec![0]), &state, &caps).unwrap();
    assert_eq!((left.d_in(), left.d_out()), (1, 2));
    assert!(left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12);
    let err = a
        .concrete_do(&Query::Open(vec![0]), &state[..2], &caps)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let a_io = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert!(matches!(
        a_io.concrete_do(&Query::Io, &state, &caps).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
}

/// Theorem 51's scope: `Equivalent` only when both models are classical; a quantum model on either
/// side makes the predicates a necessary condition. The classical model here is a wire-only model
/// with no mechanism, the only all-classical circuit model there is.
#[test]
fn test_scope_is_equivalent_only_when_both_sides_are_classical() {
    let classical =
        || CircuitModel::<f64>::ungrouped(vec![WireType::bit()], vec![], vec![], vec![]).unwrap();
    let empty = || TypeAlignment::<f64>::new(vec![]).unwrap();
    let both = Abstraction::new(
        classical(),
        classical(),
        empty(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert_eq!(
        both.check_alignment_structure(&[]).unwrap().scope,
        StructureScope::Equivalent
    );
    let quantum_wire =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![], vec![]).unwrap();
    let quantum_low = Abstraction::new(
        quantum_wire,
        classical(),
        empty(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    assert_eq!(
        quantum_low.check_alignment_structure(&[]).unwrap().scope,
        StructureScope::Necessary
    );
    // A quantum high-level model has vertices that a classical low-level one cannot supply blocks
    // for, so that side of the conjunction is decided by the quantum-low case above.
    let circuit = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let s = circuit.check_alignment_structure(&[vec![0]]).unwrap();
    assert_eq!(s.scope, StructureScope::Necessary);
    assert!(s.simple && s.extra_simple && s.full);
}

/// `square_with` takes any high-level query that is well formed on `H`, in or out of the
/// signature, and refuses a malformed one with the signature's own error.
#[test]
fn test_square_with_takes_any_well_formed_high_query() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Open(vec![0]), Query::Open(vec![0]))],
    )
    .unwrap();
    assert!(a.image(&Query::Io).is_none());
    let (left, right) = a.square_with(&Query::Io, &Query::Io, &caps).unwrap();
    assert!(left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12);
    let err = a
        .square_with(&Query::Inc(vec![vec![0], vec![0]]), &Query::Io, &caps)
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("two interchange sets")),
        "{err:?}"
    );
    let err = a
        .square_with(&Query::Open(vec![9]), &Query::Io, &caps)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

/// An opened query with a classical input: the encoder reads bit 1 and prepares wire 0, the
/// rotation on wire 0 is opened, so the query runs from the fresh wire 2 and the classical bit.
/// The concrete form carries the bit as a free classical input beside the prepared state.
#[test]
fn test_concrete_do_carries_the_classical_inputs_of_an_opened_query() {
    let caps = NumericCaps::default();
    let ket = |a: f64, b: f64| CausalTensor::from_slice(&[C::new(a, 0.0), C::new(b, 0.0)], &[2]);
    let model = || {
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit(), WireType::bit()],
            vec![
                CircuitBox::Encoder {
                    input: 1,
                    outputs: vec![0],
                    states: vec![ket(1.0, 0.0), ket(0.0, 1.0)],
                },
                ry(0, 0.7),
            ],
            vec![],
            vec![0],
        )
        .unwrap()
    };
    let identity = QcMorphism::<f64>::identity(2).unwrap();
    let a = Abstraction::new(
        model(),
        model(),
        TypeAlignment::new(vec![(vec![0], vec![0], identity.clone(), identity)]).unwrap(),
        vec![(Query::Open(vec![1]), Query::Open(vec![1]))],
    )
    .unwrap();
    let (left, right) = a.square(&Query::Open(vec![1]), &caps).unwrap();
    assert_eq!(left.classical_in(), &[2]);
    assert!(left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12);
    let state = [C::new(0.0, 0.0), C::new(1.0, 0.0)];
    let (left, right) = a.concrete_do(&Query::Open(vec![1]), &state, &caps).unwrap();
    assert_eq!((left.d_in(), left.d_out()), (1, 2));
    assert_eq!(left.classical_in(), &[2]);
    assert_eq!(left.classical_out(), &[]);
    assert!(left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12);
    // The prepared |1⟩ on the fresh wire is the output for either value of the bit.
    let (blocks, _) = left.choi_blocks(&caps).unwrap();
    assert_eq!(blocks.len(), 2);
    for x in 0..2 {
        let j = blocks.get(&(vec![x], vec![])).unwrap();
        assert!((j.as_slice()[3].re - 1.0).abs() < 1e-12, "bit {x}: {j:?}");
    }
}

/// A sided alignment in the shape of Example 58 under an interchange: the low-level model takes
/// wire 0 as input and prepares wire 1, the input side aligns by the identity and the output side
/// through `Tr_B`. `Inc({0})` copies the input on both levels; the copies align through the
/// copied input entry, and the square commutes since the interchanged output is the copy's
/// rotation on either level.
#[test]
fn test_a_sided_alignment_squares_an_interchange_query() {
    let caps = NumericCaps::default();
    let identity = QcMorphism::<f64>::identity(2).unwrap();
    let low = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.7),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![GateOp::X(1)],
            },
        ],
        vec![0],
        vec![0, 1],
    )
    .unwrap();
    let sided = TypeAlignment::new_sided(vec![
        (
            AlignmentSide::Input,
            (vec![0], vec![0], identity.clone(), identity),
        ),
        (
            AlignmentSide::Output,
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
        ),
    ])
    .unwrap();
    let inc = Query::Inc(vec![vec![0]]);
    let a = Abstraction::new(
        low,
        high(),
        sided,
        vec![(Query::Io, Query::Io), (inc.clone(), inc.clone())],
    )
    .unwrap();
    let (left, right) = a.square(&inc, &caps).unwrap();
    assert_eq!((left.d_in(), left.d_out()), (4, 2));
    assert!(
        left.frobenius_distance(&right, &caps).unwrap().0 < 1e-12,
        "{}",
        left.frobenius_distance(&right, &caps).unwrap().0
    );
    let r = a.check_naturality(&caps).unwrap();
    assert_eq!(r.report.examined(), 2);
    assert_eq!(r.report.verdict(), CheckVerdict::Accepted);
}

/// Construction checks that the alignment covers both types of every mapped query: a missing
/// output entry names the wire, an ill-typed square names its side, and a classical output map
/// that does not fit the square's classical wires is refused.
#[test]
fn test_new_refuses_a_map_whose_types_the_alignment_does_not_cover() {
    let input_only = TypeAlignment::new_sided(vec![(
        AlignmentSide::Input,
        (vec![0], vec![0, 1], trace_b(), prepare_b()),
    )])
    .unwrap();
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        input_only,
        vec![(Query::Io, Query::Io)],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("wire 0") && m.contains("not aligned")),
        "{err}"
    );
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Observe(vec![0]), Query::Observe(vec![0]))],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("ill-typed") && m.contains("Output")),
        "{err}"
    );
    let flip = stochastic_morphism::<f64>(&[vec![0.0, 1.0], vec![1.0, 0.0]], &[2], &[2]).unwrap();
    let err = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment().with_classical_output(flip).unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("classical output map")),
        "{err}"
    );
    // A well-typed map on the same models is admitted.
    assert!(
        Abstraction::new(
            low(GateOp::X(1)),
            high(),
            alignment(),
            vec![(Query::Io, Query::Io)]
        )
        .is_ok()
    );
}

/// `square_parts` forms the square and its two alignment channels in one pass and agrees with the
/// three wrappers that each form them on their own.
#[test]
fn test_square_parts_agrees_with_its_wrappers() {
    let caps = NumericCaps::default();
    let a = Abstraction::new(
        low(GateOp::X(1)),
        high(),
        alignment(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let parts: SquareParts<f64> = a.square_parts(&Query::Io, &Query::Io, &caps).unwrap();
    let (left, right) = a.square_with(&Query::Io, &Query::Io, &caps).unwrap();
    assert_eq!(parts.left, left);
    assert_eq!(parts.right, right);
    assert_eq!(
        parts.tau_in,
        a.tau_in(&Query::Io, &Query::Io, &caps).unwrap()
    );
    assert_eq!(
        parts.tau_out,
        a.tau_out(&Query::Io, &Query::Io, &caps).unwrap()
    );
    // `τ_out` is `Tr_B` here and `τ_in` the same channel: the square runs from the two low wires.
    assert_eq!((parts.tau_in.d_in(), parts.tau_in.d_out()), (4, 2));
    assert_eq!((parts.tau_out.d_in(), parts.tau_out.d_out()), (4, 2));
    assert!(
        parts
            .left
            .frobenius_distance(&parts.right, &caps)
            .unwrap()
            .0
            < 1e-12
    );
    let err = a
        .square_parts(&Query::Open(vec![9]), &Query::Io, &caps)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

/// A declared signature must be mapped in full, and nothing outside it may be mapped: five
/// queries with four pairs are refused naming the fifth, an extra pair is refused naming it, and
/// the full map keeps the signature's order.
#[test]
fn test_with_signature_requires_a_total_map_on_the_declared_signature() {
    let identity = QcMorphism::<f64>::identity(2).unwrap();
    let same = || {
        TypeAlignment::new(vec![(vec![0], vec![0], identity.clone(), identity.clone())]).unwrap()
    };
    let fault = Query::Fault(Fault::new(None, vec![(0, PauliKind::X)]).unwrap());
    let five = vec![
        Query::Io,
        Query::Open(vec![0]),
        Query::Observe(vec![0]),
        fault.clone(),
        Query::Inc(vec![vec![0]]),
    ];
    let signature = QuerySignature::new(&high().induced_dag(), five.clone()).unwrap();
    let four: Vec<(Query, Query)> = five[..4].iter().map(|q| (q.clone(), q.clone())).collect();
    let err = Abstraction::with_signature(high(), high(), same(), signature.clone(), four.clone())
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("Inc([[0]])") && m.contains("no image")),
        "{err}"
    );
    let mut extra: Vec<(Query, Query)> = five.iter().map(|q| (q.clone(), q.clone())).collect();
    extra.push((Query::Inc(vec![]), Query::Inc(vec![])));
    let err =
        Abstraction::with_signature(high(), high(), same(), signature.clone(), extra).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("Inc([])") && m.contains("not in the signature")),
        "{err}"
    );
    let full: Vec<(Query, Query)> = five.iter().rev().map(|q| (q.clone(), q.clone())).collect();
    let a = Abstraction::with_signature(high(), high(), same(), signature, full).unwrap();
    assert_eq!(a.signature().queries(), &five[..]);
    assert_eq!(a.query_map().len(), 5);
    // The map-derived form on the same four pairs is admitted, with the map's order.
    let a = Abstraction::new(high(), high(), same(), four).unwrap();
    assert_eq!(a.signature().len(), 4);
    assert_eq!(a.signature().queries()[3], fault);
}
