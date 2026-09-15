/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The composition law `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` in Frobenius norm on Choi operators.
//!
//! Provenance of the literals. The Frobenius-induced norm of a unitary is one, of a state
//! preparation one, of the completely depolarising channel one (attained on `ρ ∝ I`), and of a
//! partial trace over a `d`-dimensional factor `√d`, since `Tr_B(X ⊗ I_B) = d · X` while
//! `‖X ⊗ I_B‖_F = √d ‖X‖_F`. For two rotations `‖J(R_y(a)) − J(R_y(b))‖_F = 2√2 sin((a − b)/2)`
//! (`abstraction_tests`), and tensoring a Choi operator with the identity on a `d`-dimensional
//! input factor multiplies its norm by `√d`. In the tightness chain both traced wires are fully
//! depolarised at the level below, so post-composition by the trace attains its constant `√2`;
//! pre-composition by a trace always does. With `α = 0`, `α' = 0.3`, `β = 1.4`:
//! `ε₁ = 4 sin 0.15`, `ε₂ = 4 sin 0.55`, measured `4√2 sin 0.7 ≈ 3.64`, bound
//! `4√2 (sin 0.15 + sin 0.55) ≈ 3.80`; the bound with `pre = 1` is `4√2 sin 0.15 + 4 sin 0.55 ≈
//! 2.94` and with `post = 1` `4 sin 0.15 + 4√2 sin 0.55 ≈ 3.55`, both below the measured value.
//!
//! Corner-case rows covered by the tests below.
//!
//! `test_exact_links_compose_exactly_on_the_concatenated_code` is the Rust witness of the Lean
//! proofs in `lean/DeepCausalityFormal/Quantum/Abstraction.lean`; the traceability CI check
//! (`.github/workflows/formalization.yml`, job `theorem-map`) requires every proved Lean theorem
//! to have a matching `// THEOREM_MAP: <id>` tag in a Rust file. See the module docstring in
//! `tests/formalization_lean/partial_trace_tests.rs`.
use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    Abstraction, AlignmentSide, Axis, COMPOSITION_NORM, CheckVerdict, CircuitBox, CircuitModel,
    GateOp, LogicalGate, NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, Query,
    TypeAlignment, WireType, code_switching, concatenated_code, depolarizing_kraus,
    distillation_round, stochastic_morphism,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::LatticeComplex;

type C = Complex<f64>;
type W = u64;
type Link = Abstraction<f64, CircuitModel<f64>, CircuitModel<f64>>;

fn caps() -> NumericCaps {
    NumericCaps::default()
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

fn ry(wire: usize, theta: f64) -> CircuitBox<f64> {
    CircuitBox::Kraus {
        wires: vec![wire],
        kraus: vec![
            QubitOperator::<f64>::rotation(Axis::Y, theta)
                .unwrap()
                .matrix()
                .clone(),
        ],
    }
}

fn depolarize(wire: usize) -> CircuitBox<f64> {
    CircuitBox::Kraus {
        wires: vec![wire],
        kraus: depolarizing_kraus::<f64>(0.75).unwrap(),
    }
}

#[test]
fn test_frobenius_induced_norm_of_known_channels() {
    let c = caps();
    let hadamard =
        QcMorphism::from_kraus(&[QubitOperator::<f64>::hadamard().matrix().clone()]).unwrap();
    assert!((hadamard.frobenius_induced_norm(&c).unwrap() - 1.0).abs() < 1e-12);
    assert!((trace_b().frobenius_induced_norm(&c).unwrap() - 2f64.sqrt()).abs() < 1e-12);
    assert!((prepare_b().frobenius_induced_norm(&c).unwrap() - 1.0).abs() < 1e-12);
    let depolarizing = QcMorphism::from_kraus(&depolarizing_kraus::<f64>(0.75).unwrap()).unwrap();
    assert!((depolarizing.frobenius_induced_norm(&c).unwrap() - 1.0).abs() < 1e-12);
    // A scaled unitary has norm |scale|²: the natural representation is K ⊗ conj(K).
    let half = QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[
            C::new(0.5, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.5, 0.0),
        ],
        &[2, 2],
    )])
    .unwrap();
    assert!((half.frobenius_induced_norm(&c).unwrap() - 0.25).abs() < 1e-12);
    // Two operators with a relative phase `i` on one matrix element: the phases cancel in
    // `K ⊗ conj(K)`, so the natural representation is `2 |00⟩⟨00|` and the norm is 2; with `K ⊗ K`
    // they would cancel each other and the norm would be zero.
    let p0 = CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
        ],
        &[2, 2],
    );
    let ip0 = CausalTensor::from_slice(
        &[
            C::new(0.0, 1.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
        ],
        &[2, 2],
    );
    let phased = QcMorphism::from_kraus(&[p0, ip0]).unwrap();
    assert!((phased.frobenius_induced_norm(&c).unwrap() - 2.0).abs() < 1e-12);
    // Classical blocks on a diagonal: the natural representation is block-diagonal and the norm
    // is the largest block norm, here the scalar 2 with norm 4.
    let one = |v: f64| CausalTensor::from_slice(&[C::new(v, 0.0)], &[1, 1]);
    let mut blocks = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    blocks.push(vec![0], vec![0], vec![one(1.0)]).unwrap();
    blocks.push(vec![1], vec![1], vec![one(2.0)]).unwrap();
    assert!((blocks.frobenius_induced_norm(&c).unwrap() - 4.0).abs() < 1e-12);
    // Above the entry cap the norm is refused before the natural representation is formed.
    let tiny = NumericCaps {
        max_entries: 15,
        max_operators: 1 << 12,
    };
    assert!(matches!(
        hadamard.frobenius_induced_norm(&tiny).unwrap_err().0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            entries: 16,
            cap: 15,
            ..
        }
    ));
}

/// Three wires down to one through two partial traces, the traced wires fully depolarised below
/// their trace, and three rotation angles. Both constants are `√2` and the bound with either
/// constant set to one is exceeded.
fn tightness_chain(alpha: f64, alpha_prime: f64, beta: f64) -> (Link, Link) {
    let low = CircuitModel::ungrouped(
        vec![WireType::qubit(); 3],
        vec![ry(0, alpha), depolarize(2)],
        vec![0, 1, 2],
        vec![0, 1, 2],
    )
    .unwrap();
    let middle = CircuitModel::ungrouped(
        vec![WireType::qubit(); 2],
        vec![ry(0, alpha_prime), depolarize(1)],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    let high =
        CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, beta)], vec![0], vec![0])
            .unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let first = Abstraction::new(
        low,
        middle.clone(),
        TypeAlignment::new(vec![
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
            (vec![1], vec![2], id.clone(), id),
        ])
        .unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let second = Abstraction::new(
        middle,
        high,
        TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    (first, second)
}

#[test]
fn test_the_bound_is_tight_on_a_constructed_case() {
    let (first, second) = tightness_chain(0.0, 0.3, 1.4);
    let composed = first.compose(second, &caps()).unwrap();
    let law = &composed.law;
    assert_eq!(law.rows.len(), 1);
    let row = &law.rows[0];
    let s = |x: f64| x.sin();
    assert!((row.pre - 2f64.sqrt()).abs() < 1e-9, "pre {}", row.pre);
    assert!((row.post - 2f64.sqrt()).abs() < 1e-9, "post {}", row.post);
    assert!(
        (row.epsilon_first - 4.0 * s(0.15)).abs() < 1e-9,
        "ε₁ {}",
        row.epsilon_first
    );
    assert!(
        (row.epsilon_second - 4.0 * s(0.55)).abs() < 1e-9,
        "ε₂ {}",
        row.epsilon_second
    );
    assert!(
        (row.measured - 4.0 * 2f64.sqrt() * s(0.7)).abs() < 1e-9,
        "measured {}",
        row.measured
    );
    assert!((row.bound - 4.0 * 2f64.sqrt() * (s(0.15) + s(0.55))).abs() < 1e-9);
    assert!(row.holds() && law.holds());
    // Either constant set to one loses the bound.
    let with_pre_one = row.post * row.epsilon_first + row.epsilon_second;
    let with_post_one = row.epsilon_first + row.pre * row.epsilon_second;
    assert!(
        row.measured > with_pre_one,
        "{} vs {with_pre_one}",
        row.measured
    );
    assert!(
        row.measured > with_post_one,
        "{} vs {with_post_one}",
        row.measured
    );
    // The composite traces both extra wires: its alignment sends the one high wire to all three.
    let entry = &composed.abstraction.alignment().entries()[0];
    assert_eq!(entry.low(), &[0, 1, 2]);
    assert_eq!(entry.side(), AlignmentSide::Any);
    assert_eq!(law.norm, COMPOSITION_NORM);
    assert_eq!(law.tightest().unwrap().query, Query::Io);
    assert!(
        (law.bound() - row.bound).abs() < 1e-12 && (law.measured() - row.measured).abs() < 1e-12
    );
}

#[test]
fn test_provenance_records_the_law() {
    let (first, second) = tightness_chain(0.0, 0.3, 1.4);
    let composed = first.compose(second, &caps()).unwrap();
    let shown = format!("{}", composed.law);
    for needle in [
        "frobenius-on-choi",
        "ε₁ = ",
        "ε₂ = ",
        "‖τ₁‖_pre = ",
        "‖τ₂‖_post = ",
        "bound = ",
        "measured = ",
        "holds",
        "io:",
    ] {
        assert!(shown.contains(needle), "{needle} missing in {shown}");
    }
    assert_eq!(shown.lines().count(), 2);
}

// THEOREM_MAP: quantum.abstraction.compose_exact
// THEOREM_MAP: quantum.abstraction.compose_exact.defect
#[test]
fn test_exact_links_compose_exactly_on_the_concatenated_code() {
    let complex = four_two_two();
    // CZ̄ of the inner code pairs a qubit of each outer block, and a two-qubit gate across blocks
    // has no transversal gadget in this construction: refused by name.
    let across = concatenated_code::<W, _, _, f64>(&complex, &complex, &LogicalGate::Cz(0, 1))
        .expect_err("refused");
    assert!(
        matches!(across.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("across outer blocks")),
        "{across}"
    );
    for gate in [
        LogicalGate::Z(0),
        LogicalGate::Z(1),
        LogicalGate::X(0),
        LogicalGate::X(1),
    ] {
        let chain = concatenated_code::<W, _, _, f64>(&complex, &complex, &gate).unwrap();
        assert_eq!(chain.first.low().wires().len(), 8);
        assert_eq!(chain.first.low().inputs(), &[0, 1]);
        assert_eq!(chain.first.high().wires().len(), 4);
        let composed = chain.compose(&caps()).unwrap();
        let row = &composed.law.rows[0];
        assert!(
            row.epsilon_first < 1e-9,
            "{}: ε₁ {}",
            gate.name(),
            row.epsilon_first
        );
        assert!(
            row.epsilon_second < 1e-9,
            "{}: ε₂ {}",
            gate.name(),
            row.epsilon_second
        );
        assert!(
            row.measured < 1e-9,
            "{}: measured {}",
            gate.name(),
            row.measured
        );
        assert!(row.bound < 1e-9 && composed.law.holds());
        // π = π₁ ∘ π₂: the two logical wires reach all eight physical wires on the output side and
        // the two low logical wires on the input side.
        let entries = composed.abstraction.alignment().entries();
        let output = entries
            .iter()
            .find(|e| e.side() == AlignmentSide::Output)
            .unwrap();
        assert_eq!(output.high(), &[0, 1]);
        assert_eq!(output.low(), &(0..8).collect::<Vec<_>>());
        assert_eq!((output.tau().d_in(), output.tau().d_out()), (256, 4));
        let input = entries
            .iter()
            .find(|e| e.side() == AlignmentSide::Input)
            .unwrap();
        assert_eq!(input.low(), &[0, 1]);
        let report = composed.abstraction.check_naturality(&caps()).unwrap();
        assert_eq!(report.report.verdict(), CheckVerdict::Accepted);
        assert!(report.worst_residual() < 1e-9);
    }
}

#[test]
fn test_code_switching_composes_exactly_without_noise_and_bounds_the_noisy_gadget() {
    let small = four_two_two();
    let torus = LatticeComplex::<2, f64>::square_torus(2);
    let clean = code_switching::<W, _, _, f64>(&small, &torus, &LogicalGate::Z(0), vec![]);
    let clean = clean.unwrap().compose(&caps()).unwrap();
    let row = &clean.law.rows[0];
    assert!(
        row.epsilon_first < 1e-9 && row.epsilon_second < 1e-9 && row.measured < 1e-9,
        "{}",
        clean.law
    );
    assert!(
        (row.pre - 1.0).abs() < 1e-9,
        "the gadget's input alignment is the identity"
    );

    let noisy = code_switching::<W, _, _, f64>(
        &small,
        &torus,
        &LogicalGate::Z(0),
        depolarizing_kraus::<f64>(0.1).unwrap(),
    )
    .unwrap()
    .compose(&caps())
    .unwrap();
    let row = &noisy.law.rows[0];
    assert!(
        row.epsilon_first > 0.1,
        "the gadget noise is the first link's residual: {}",
        row.epsilon_first
    );
    assert!(row.epsilon_second < 1e-9);
    assert!(row.measured > 0.0 && row.holds(), "{}", noisy.law);
    assert!(row.post >= 1.0, "the recovery's norm {}", row.post);
    // With ε₂ at zero the bound is the post constant times ε₁, not the pre constant.
    assert!(
        (row.bound - row.post * row.epsilon_first).abs() < 1e-9,
        "{}",
        noisy.law
    );
    assert!((row.pre - 1.0).abs() < 1e-9);
}

#[test]
fn test_a_distillation_round_reports_the_noise_the_recovery_leaves() {
    let complex = four_two_two();
    let quiet = distillation_round::<W, _, f64>(&complex, 0.0)
        .unwrap()
        .compose(&caps())
        .unwrap();
    assert!(quiet.law.measured() < 1e-9, "{}", quiet.law);
    let noisy = distillation_round::<W, _, f64>(&complex, 0.05)
        .unwrap()
        .compose(&caps())
        .unwrap();
    let row = &noisy.law.rows[0];
    assert!(row.epsilon_first > 0.0 && row.epsilon_second < 1e-9);
    assert!(row.measured > 0.0 && row.holds(), "{}", noisy.law);
    assert!(distillation_round::<W, _, f64>(&complex, 1.5).is_err());
}

#[test]
fn test_composition_errors_name_the_query_and_the_wire() {
    // A second link whose signature has a query the first link does not map.
    let (first, _) = tightness_chain(0.0, 0.3, 1.4);
    let middle = first.high().clone();
    let high = CircuitModel::ungrouped(
        vec![WireType::qubit(); 2],
        vec![ry(0, 1.4)],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let second = Abstraction::new(
        middle,
        high,
        TypeAlignment::new(vec![
            (vec![0], vec![0], id.clone(), id.clone()),
            (vec![1], vec![1], id.clone(), id),
        ])
        .unwrap(),
        vec![
            (Query::Io, Query::Io),
            (Query::Open(vec![0]), Query::Open(vec![0])),
        ],
    )
    .unwrap();
    let err = first.compose(second, &caps()).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("no image") && m.contains("Open")),
        "{err}"
    );
    // A second link aligning a middle wire the first link does not cover.
    let (first, _) = tightness_chain(0.0, 0.3, 1.4);
    let middle = first.high().clone();
    let high = CircuitModel::ungrouped(
        vec![WireType::qubit(); 2],
        vec![ry(0, 1.4)],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let uncovered = Abstraction::new(
        middle,
        high,
        TypeAlignment::new(vec![
            (vec![0], vec![0], id.clone(), id.clone()),
            (vec![1], vec![1], id.clone(), id),
        ])
        .unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    // The first link covers middle wire 1 through its own entry, so this composes; a middle wire
    // outside the first link's alignment cannot be built at all, since the middle model has two
    // wires. Check the well-typed composite instead: two output entries, low wires [0, 1] and [2].
    let composed = first.compose(uncovered, &caps()).unwrap();
    let lows: Vec<Vec<usize>> = composed
        .abstraction
        .alignment()
        .entries()
        .iter()
        .map(|e| e.low().to_vec())
        .collect();
    assert_eq!(lows, vec![vec![0, 1], vec![2]]);
}

/// A two-sided entry of the second link over a sided first link splits into an input-side and an
/// output-side composite entry, each assembled on its own side.
#[test]
fn test_a_two_sided_entry_over_a_sided_link_splits() {
    let complex = four_two_two();
    let chain = concatenated_code::<W, _, _, f64>(&complex, &complex, &LogicalGate::Z(0)).unwrap();
    // Replace the second link by an unsided identity abstraction from the middle model onto a
    // copy of itself, one `Any` entry per block, over the sided first link.
    let middle = chain.first.high().clone();
    let id2 = QcMorphism::<f64>::identity(4).unwrap();
    let second = Abstraction::new(
        middle.clone(),
        middle,
        TypeAlignment::new(vec![
            (vec![0, 1], vec![0, 1], id2.clone(), id2.clone()),
            (vec![2, 3], vec![2, 3], id2.clone(), id2),
        ])
        .unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap();
    let composed = chain.first.compose(second, &caps()).unwrap();
    let sides: Vec<AlignmentSide> = composed
        .abstraction
        .alignment()
        .entries()
        .iter()
        .map(|e| e.side())
        .collect();
    assert_eq!(
        sides,
        vec![
            AlignmentSide::Input,
            AlignmentSide::Output,
            AlignmentSide::Input,
            AlignmentSide::Output
        ]
    );
    let lows: Vec<Vec<usize>> = composed
        .abstraction
        .alignment()
        .entries()
        .iter()
        .map(|e| e.low().to_vec())
        .collect();
    assert_eq!(
        lows,
        vec![vec![0, 1], vec![0, 1, 2, 3], vec![4, 5], vec![4, 5, 6, 7]]
    );
    assert!(composed.law.holds() && composed.law.measured() < 1e-9);
}

/// One qubit rotated by `R_y(θ)`, then flipped by `X` when asked; observed, it yields one bit.
fn rotated(theta: f64, flipped: bool) -> CircuitModel<f64> {
    let mut boxes = vec![ry(0, theta)];
    if flipped {
        boxes.push(CircuitBox::Unitary {
            wires: vec![0],
            program: vec![GateOp::X(0)],
        });
    }
    CircuitModel::ungrouped(vec![WireType::qubit()], boxes, vec![0], vec![0]).unwrap()
}

/// The bit flip as a classical output map on the trivial quantum system.
fn flip() -> QcMorphism<f64> {
    stochastic_morphism::<f64>(&[vec![0.0, 1.0], vec![1.0, 0.0]], &[2], &[2]).unwrap()
}

/// A link between two one-qubit models aligned by the identity, mapping `Observe(0)` to itself,
/// with a classical output map when given.
fn observed_link(
    low: CircuitModel<f64>,
    high: CircuitModel<f64>,
    map: Option<QcMorphism<f64>>,
) -> Link {
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let alignment = TypeAlignment::new(vec![(vec![0], vec![0], id.clone(), id)]).unwrap();
    let alignment = match map {
        Some(m) => alignment.with_classical_output(m).unwrap(),
        None => alignment,
    };
    Abstraction::new(
        low,
        high,
        alignment,
        vec![(Query::Observe(vec![0]), Query::Observe(vec![0]))],
    )
    .unwrap()
}

/// A classical output map on either link is carried into the composite alignment, and two maps
/// compose. The flipped level measures `X R_y(θ)|0⟩`, so a square through the bit flip commutes
/// exactly and the composite residual is zero only if the composite carries the map.
#[test]
fn test_compose_carries_a_classical_output_map_from_either_link() {
    let c = caps();
    let keys = |m: &QcMorphism<f64>| m.blocks().keys().cloned().collect::<Vec<_>>();
    let flipped = vec![(vec![0], vec![1]), (vec![1], vec![0])];
    let straight = vec![(vec![0], vec![0]), (vec![1], vec![1])];

    // The second link carries the map.
    let first = observed_link(rotated(0.7, false), rotated(0.7, false), None);
    let second = observed_link(rotated(0.7, false), rotated(0.7, true), Some(flip()));
    let composed = first.compose(second, &c).unwrap();
    let map = composed
        .abstraction
        .alignment()
        .classical_output()
        .expect("the second link's map is carried");
    assert_eq!(keys(map), flipped);
    assert!(
        composed.law.measured() < 1e-9 && composed.law.holds(),
        "{}",
        composed.law
    );

    // The first link carries the map.
    let first = observed_link(rotated(0.7, false), rotated(0.7, true), Some(flip()));
    let second = observed_link(rotated(0.7, true), rotated(0.7, true), None);
    let composed = first.compose(second, &c).unwrap();
    let map = composed
        .abstraction
        .alignment()
        .classical_output()
        .expect("the first link's map is carried");
    assert_eq!(keys(map), flipped);
    assert!(
        composed.law.measured() < 1e-9 && composed.law.holds(),
        "{}",
        composed.law
    );

    // Both carry one: flip then flip is the identity on the bit.
    let first = observed_link(rotated(0.7, false), rotated(0.7, true), Some(flip()));
    let second = observed_link(rotated(0.7, true), rotated(0.7, false), Some(flip()));
    let composed = first.compose(second, &c).unwrap();
    let map = composed
        .abstraction
        .alignment()
        .classical_output()
        .expect("the composite of both maps is carried");
    assert_eq!(keys(map), straight);
    assert_eq!(
        (map.classical_in(), map.classical_out()),
        (&[2][..], &[2][..])
    );
    let row = &composed.law.rows[0];
    assert!(row.epsilon_first < 1e-9 && row.epsilon_second < 1e-9);
    assert!(row.measured < 1e-9 && row.holds(), "{}", composed.law);
    assert_eq!(row.query, Query::Observe(vec![0]));
}
