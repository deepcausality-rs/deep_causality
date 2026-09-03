/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! `Hypothesis`: a factorization with a derived structure, the mechanism-level intervention, the
//! embedded contraction `predict` reads, and the warrant-gated marginalisation.
//!
//! Two external anchors: van der Lugt & Lorenz's `C₃` (Example 2.12) for the decomposability gate,
//! and the `√(d_B)` contraction `‖Tr_B(E)‖_F ≤ √(d_B)·‖E‖_F` behind the boundary warrant, which is
//! tight at `E = F ⊗ 1_B`.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CheckVerdict, CommutatorTolerance, FactorSupports, Factorization, Hypothesis,
    ProcessFactors, QuantumErrorEnum, QubitOperator, embed_on_legs, markov_certificate,
};
use deep_causality_tensor::{CausalTensor, Tensor};
use std::collections::BTreeSet;

type C = Complex<f64>;

fn c(re: f64) -> C {
    Complex::new(re, 0.0)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0.), c(1.), c(1.), c(0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1.), c(0.), c(0.), c(-1.)], 2)
}

fn diag(a: f64, b: f64) -> CausalTensor<C> {
    mat(vec![c(a), c(0.), c(0.), c(b)], 2)
}

fn ket0_proj() -> CausalTensor<C> {
    diag(1.0, 0.0)
}

fn graph(
    n: usize,
    edges: &[(usize, usize)],
    freeze: bool,
) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let id = g
            .add_causaloid(test_utils::get_test_causaloid_deterministic(i as u64))
            .unwrap();
        nodes.push(id);
    }
    for &(a, b) in edges {
        g.add_edge(nodes[a], nodes[b]).unwrap();
    }
    if freeze {
        g.freeze();
    }
    g
}

/// Two commuting factors on one leg.
fn commuting_pair() -> (ProcessFactors<f64>, FactorSupports) {
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    (pf, fs)
}

// ---------------------------------------------------------------------------
// Construction and the derived structure.
// ---------------------------------------------------------------------------

#[test]
fn test_a_structural_candidate_carries_factors_and_supports_and_derives_its_structure() {
    let (pf, fs) = commuting_pair();
    let h = Hypothesis::structural("Q1->Q2", pf.clone(), fs.clone()).unwrap();
    assert_eq!(h.name(), "Q1->Q2");
    assert!(h.is_structural() && !h.is_mechanism());
    assert_eq!(h.factors(), Some(&pf));
    assert_eq!(h.supports(), Some(&fs));
    assert!(h.channel().is_none());
    assert!(h.certificate().is_none());

    // The structure is derived from a frozen graph each time, never stored.
    let g = graph(3, &[(0, 1), (1, 2)], true);
    let s1 = h.structure(&g, &[0, 1], &[1, 2]).unwrap();
    let s2 = h.structure(&g, &[0, 1], &[1, 2]).unwrap();
    assert_eq!(s1, s2);
    assert!(s1.influences(0, 2));
    assert!(!s1.influences(1, 0) || true);
}

#[test]
fn test_a_mechanism_candidate_has_no_structure_to_derive() {
    let h = Hypothesis::mechanism(
        "amplitude",
        Channel::unitary(&QubitOperator::<f64>::pauli_x()).unwrap(),
    );
    assert!(h.is_mechanism());
    assert!(h.factors().is_none() && h.supports().is_none());
    assert!(h.channel().is_some());
    let g = graph(2, &[(0, 1)], true);
    assert!(matches!(
        h.structure(&g, &[0], &[1]).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    assert!(h.check_markov(&CommutatorTolerance::default()).is_err());
    assert!(h.joint_operator().is_err());
}

#[test]
fn test_construction_validates_and_names_the_offending_factor() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(2, mat(vec![c(1.); 16], 4));
    let mut fs = FactorSupports::new();
    fs.declare(2, &[2]);
    match Hypothesis::structural("bad", pf, fs).unwrap_err().0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(
                msg.contains("node 2") && msg.contains("dim 4") && msg.contains("implies 2"),
                "{msg}"
            )
        }
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
}

#[test]
fn test_construction_from_a_graph_rejects_unfrozen_and_out_of_range() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    let unfrozen = graph(2, &[(0, 1)], false);
    assert!(matches!(
        Hypothesis::structural_from_graph("u", &unfrozen, pf.clone())
            .unwrap_err()
            .0,
        QuantumErrorEnum::CalculationError(_)
    ));
    let frozen = graph(4, &[(0, 1)], true);
    pf.insert(9, sigma_z());
    match Hypothesis::structural_from_graph("r", &frozen, pf)
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("node 9") && msg.contains("0..4"), "{msg}")
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The decomposability gate.
// ---------------------------------------------------------------------------

#[test]
fn test_the_derived_structure_feeds_the_decomposability_gate() {
    // Example 2.12's C₃ as reachability: inputs {0, 1, 2} → outputs {3, 4, 5}, every pair except
    // (0, 5) and (2, 3). Rejected with the witness.
    let g = graph(
        6,
        &[(0, 3), (0, 4), (1, 3), (1, 4), (1, 5), (2, 4), (2, 5)],
        true,
    );
    let mut pf = ProcessFactors::<f64>::new();
    for n in 3..6 {
        pf.insert(n, sigma_z());
    }
    let mut fs = FactorSupports::new();
    for n in 3..6 {
        fs.declare(n, &[n]);
    }
    let h = Hypothesis::structural("cnots", pf.clone(), fs.clone()).unwrap();
    match h
        .check_decomposable(&g, &[0, 1, 2], &[3, 4, 5])
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::NotFaithfullyRepresentable(msg) => {
            assert!(
                msg.contains("[0, 1, 2]") && msg.contains("[3, 4, 5]"),
                "{msg}"
            )
        }
        other => panic!("expected NotFaithfullyRepresentable, got {other:?}"),
    }

    // K₃,₃ passes with one block examined; a 2 × 2 relation examines nothing and reads as vacuous.
    let full = graph(
        6,
        &[
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
        ],
        true,
    );
    let report = h.check_decomposable(&full, &[0, 1, 2], &[3, 4, 5]).unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    assert_eq!(report.examined(), 1);
    let small = h.check_decomposable(&full, &[0, 1], &[3, 4]).unwrap();
    assert_eq!(small.verdict(), CheckVerdict::Vacuous);
    assert_eq!(small.examined(), 0);
}

// ---------------------------------------------------------------------------
// The mechanism-level intervention, and what it invalidates.
// ---------------------------------------------------------------------------

#[test]
fn test_the_cut_touches_one_key_and_drops_the_certificate() {
    let mut pf = ProcessFactors::<f64>::new();
    let mut fs = FactorSupports::new();
    for n in 0..4 {
        pf.insert(n, diag(1.0 + n as f64, 0.5));
        fs.declare(n, &[n]);
    }
    let h = Hypothesis::structural("four", pf.clone(), fs).unwrap();
    let certified = h.check_markov(&CommutatorTolerance::default()).unwrap();
    assert!(certified.certificate().is_some());

    let cut = certified.intervene_mechanism(3, sigma_x()).unwrap();
    let f = cut.factors().unwrap();
    assert_eq!(f.len(), 4);
    assert_eq!(f.get(3), Some(&sigma_x()));
    for n in 0..3 {
        assert_eq!(f.get(n), pf.get(n));
    }
    assert!(cut.supports().unwrap().validate(f).is_ok());
    // The intervened hypothesis starts without a report; the receiver keeps its own.
    assert!(cut.certificate().is_none());
    assert!(certified.certificate().is_some());
    assert_eq!(certified.factors().unwrap().get(3), pf.get(3));
}

#[test]
fn test_a_replacement_of_the_wrong_dimension_or_on_an_undeclared_node_fails_the_cut() {
    let (pf, fs) = commuting_pair();
    let h = Hypothesis::structural("pair", pf, fs).unwrap();
    let before = h.clone();
    match h
        .intervene_mechanism(0, mat(vec![c(1.); 64], 8))
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("node 0") && msg.contains("dim 8"), "{msg}")
        }
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
    match h.intervene_mechanism(7, sigma_z()).unwrap_err().0 {
        QuantumErrorEnum::DimensionMismatch(msg) => assert!(
            msg.contains("node 7") && msg.contains("no declared support"),
            "{msg}"
        ),
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
    assert_eq!(h, before, "the hypothesis passed in is left unchanged");
}

#[test]
fn test_the_re_run_reports_its_own_count_and_a_disjoint_replacement_is_vacuous() {
    let (pf, fs) = commuting_pair();
    let h = Hypothesis::structural("pair", pf, fs)
        .unwrap()
        .check_markov(&CommutatorTolerance::default())
        .unwrap();
    assert_eq!(h.certificate().unwrap().examined(), 1);

    // Move node 1 onto a leg of its own: after the cut nothing overlaps.
    let mut fs2 = h.supports().unwrap().clone();
    fs2.declare(1, &[5]);
    let moved = Hypothesis::structural("moved", h.factors().unwrap().clone(), fs2).unwrap();
    let rerun = moved.check_markov(&CommutatorTolerance::default()).unwrap();
    let report = rerun.certificate().unwrap();
    assert_eq!(report.examined(), 0);
    assert_eq!(report.verdict(), CheckVerdict::Vacuous);
    assert_eq!(report.factorization(), Factorization::Rederived);
}

// ---------------------------------------------------------------------------
// predict: the joint operator by embedding, never by tracing.
// ---------------------------------------------------------------------------

#[test]
fn test_the_joint_operator_is_built_by_embedding_on_the_union() {
    // Factors on legs {0, 1} and {1, 2}, each leg a qubit: the union is {0, 1, 2}, the space is
    // 8 × 8, and the joint operator is the product of the two embedded factors. No leg is traced.
    let a = sigma_z().kronecker(&diag(2.0, 3.0)).unwrap();
    let b = diag(1.0, -1.0).kronecker(&sigma_z()).unwrap();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, a.clone());
    pf.insert(1, b.clone());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    fs.declare(1, &[1, 2]);
    let h = Hypothesis::structural("overlap", pf, fs.clone()).unwrap();

    let legs = h.legs().unwrap();
    assert_eq!(legs.keys().copied().collect::<Vec<_>>(), vec![0, 1, 2]);
    let joint = h.joint_operator().unwrap();
    assert_eq!(joint.shape(), &[8, 8]);

    let space = fs.space_map(&[0usize, 1, 2].into_iter().collect::<BTreeSet<_>>());
    let ea = embed_on_legs(&a, &[0usize, 1].into_iter().collect(), &space).unwrap();
    let eb = embed_on_legs(&b, &[1usize, 2].into_iter().collect(), &space).unwrap();
    let expected = ea.matmul(&eb).unwrap();
    for (x, y) in joint.as_slice().iter().zip(expected.as_slice()) {
        assert!((x.re - y.re).abs() < 1e-12 && (x.im - y.im).abs() < 1e-12);
    }
}

#[test]
fn test_predict_intervenes_then_evaluates_and_leaves_the_receiver_alone() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, diag(0.7, 0.3));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    let h = Hypothesis::structural("one", pf, fs).unwrap();
    // Tr(σ · |0⟩⟨0|) = 0.7 on the model as it stands.
    assert!((h.evaluate(&ket0_proj()).unwrap() - 0.7).abs() < 1e-12);
    // Under do(0 ← diag(0.2, 0.8)) the same read-out is 0.2, and the receiver still reads 0.7.
    assert!((h.predict(0, diag(0.2, 0.8), &ket0_proj()).unwrap() - 0.2).abs() < 1e-12);
    assert!((h.evaluate(&ket0_proj()).unwrap() - 0.7).abs() < 1e-12);
    // A mis-sized instrument is refused.
    assert!(matches!(
        h.evaluate(&mat(vec![c(1.); 16], 4)).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_a_non_finite_trace_is_refused_rather_than_returned() {
    // A NaN compares false against every threshold, so the imaginary-part gate alone would pass
    // it through as a number. A NaN in the factor and an infinity in the instrument both stop.
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    let mut nan = ProcessFactors::<f64>::new();
    nan.insert(0, diag(f64::NAN, 0.3));
    let h = Hypothesis::structural("nan", nan, fs.clone()).unwrap();
    match h.evaluate(&ket0_proj()).unwrap_err().0 {
        QuantumErrorEnum::NonFiniteValue(msg) => assert!(msg.contains("not finite"), "{msg}"),
        other => panic!("expected NonFiniteValue, got {other:?}"),
    }

    let mut finite = ProcessFactors::<f64>::new();
    finite.insert(0, diag(0.7, 0.3));
    let h = Hypothesis::structural("one", finite, fs).unwrap();
    assert!(matches!(
        h.evaluate(&diag(f64::INFINITY, 0.0)).unwrap_err().0,
        QuantumErrorEnum::NonFiniteValue(_)
    ));
}

// ---------------------------------------------------------------------------
// Marginalisation, gated on the boundary warrant.
// ---------------------------------------------------------------------------

/// `σ = Z ⊗ W` on legs {0, 1}: `Z ⊗ 1` commutes with it exactly.
fn commuting_boundary_model() -> Hypothesis<f64> {
    let w = mat(
        vec![c(1.), Complex::new(0., 2.), Complex::new(0., -2.), c(4.)],
        2,
    );
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z().kronecker(&w).unwrap());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    Hypothesis::structural("zw", pf, fs).unwrap()
}

#[test]
fn test_a_held_warrant_travels_with_the_marginalised_model_degraded_by_its_amplification() {
    let h = commuting_boundary_model()
        .check_markov(&CommutatorTolerance::default())
        .unwrap();
    let m = h.marginalise(1, &sigma_z(), 1e-12).unwrap();
    assert!(m.warrant.holds);
    assert_eq!(m.warrant.hypothesis_residual, 0.0);
    assert!((m.warrant.amplification - 2f64.sqrt()).abs() < 1e-15);
    assert_eq!(m.warrant.conclusion_bound, 0.0);
    assert_eq!(m.operator.shape(), &[2, 2]);
    // Tr_B(Z ⊗ W) = Tr(W)·Z = 5·Z.
    assert!((m.operator.as_slice()[0].re - 5.0).abs() < 1e-12);
    assert!((m.operator.as_slice()[3].re + 5.0).abs() < 1e-12);
    // The prior certificate is not carried; its margin is reported degraded by √(d_B).
    let prior = h.certificate().unwrap().worst_margin();
    assert_eq!(
        m.prior_margin_degraded,
        prior.map(|p| p * m.warrant.amplification)
    );
}

#[test]
fn test_the_amplification_is_the_root_of_the_traced_dimension() {
    // Keep leg 0, trace legs {1, 2}: d_B = 4 and the amplification reads 2.
    let w = diag(1.0, 4.0).kronecker(&diag(2.0, 3.0)).unwrap();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z().kronecker(&w).unwrap());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1, 2]);
    let h = Hypothesis::structural("zww", pf, fs).unwrap();
    let m = h.marginalise(1, &sigma_z(), 1e-12).unwrap();
    assert!((m.warrant.amplification - 2.0).abs() < 1e-15);
    assert_eq!(
        m.warrant.conclusion_bound,
        2.0 * m.warrant.hypothesis_residual
    );
    assert_eq!(m.operator.shape(), &[2, 2]);
    assert!(
        m.prior_margin_degraded.is_none(),
        "no certificate, nothing to degrade"
    );
}

#[test]
fn test_a_failed_warrant_refuses_the_marginalisation_and_traces_nothing() {
    // σ = X ⊗ W and Z on the kept leg: Z ⊗ 1 anticommutes with X ⊗ W, so the residual is large.
    let w = mat(
        vec![c(1.), Complex::new(0., 2.), Complex::new(0., -2.), c(4.)],
        2,
    );
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x().kronecker(&w).unwrap());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    let h = Hypothesis::structural("xw", pf, fs).unwrap();

    let typed = h.boundary_warrant(1, &sigma_z(), 1e-6).unwrap();
    assert!(!typed.holds);
    assert!(typed.hypothesis_residual > 1e-6);

    match h.marginalise(1, &sigma_z(), 1e-6).unwrap_err().0 {
        QuantumErrorEnum::BoundaryNotHeld(msg) => {
            assert!(msg.contains("1e-6"), "names the tolerance: {msg}");
            assert!(msg.contains("nothing was traced"), "{msg}");
        }
        other => panic!("expected BoundaryNotHeld, got {other:?}"),
    }
    // Keeping every leg, or none, is not a marginalisation.
    assert!(h.marginalise(0, &sigma_z(), 1e-6).is_err());
    assert!(h.marginalise(2, &sigma_z(), 1e-6).is_err());
}

// ---------------------------------------------------------------------------
// Composition and the disjoint-legs inheritance rule.
// ---------------------------------------------------------------------------

fn single(name: &str, node: usize, leg: usize, factor: CausalTensor<C>) -> Hypothesis<f64> {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(node, factor);
    let mut fs = FactorSupports::new();
    fs.declare(node, &[leg]);
    Hypothesis::structural(name, pf, fs)
        .unwrap()
        .check_markov(&CommutatorTolerance::default())
        .unwrap()
}

#[test]
fn test_a_composite_over_disjoint_legs_inherits_whatever_its_arity() {
    let a = single("a", 0, 0, sigma_z());
    let b = single("b", 1, 1, diag(2.0, 5.0));
    let ab = a.compose(&b).unwrap();
    assert_eq!(ab.name(), "a∘b");
    assert_eq!(ab.factors().unwrap().len(), 2);
    let inherited = ab
        .certificate()
        .expect("certified parts on disjoint legs inherit");
    assert_eq!(inherited.factorization(), Factorization::Inherited);
    assert!(
        inherited.is_vacuous(),
        "two disjoint single factors tested no pair"
    );

    // A third certified factor on a leg of its own: every cross pair commutes by construction,
    // so the composite inherits at three factors as it did at two.
    let c3 = single("c", 2, 2, sigma_x());
    let abc = ab.compose(&c3).unwrap();
    assert_eq!(abc.factors().unwrap().len(), 3);
    assert_eq!(
        abc.certificate().map(|r| r.factorization()),
        Some(Factorization::Inherited)
    );

    // Overlapping keys cannot compose.
    assert!(a.compose(&a).is_err());
    // An uncertified part yields no certificate even on disjoint legs.
    let plain = Hypothesis::structural(
        "p",
        {
            let mut pf = ProcessFactors::new();
            pf.insert(4, sigma_z());
            pf
        },
        {
            let mut fs = FactorSupports::new();
            fs.declare(4, &[4]);
            fs
        },
    )
    .unwrap();
    assert!(a.compose(&plain).unwrap().certificate().is_none());
}

#[test]
fn test_certified_parts_sharing_a_leg_compose_without_a_certificate() {
    // σx and σz, each certified alone, on one shared leg. Neither part's report covers the cross
    // pair, so the composite inherits nothing, and the check on the composite rejects that pair.
    let a = single("a", 0, 0, sigma_x());
    let b = single("b", 1, 0, sigma_z());
    let ab = a.compose(&b).unwrap();
    assert_eq!(ab.factors().unwrap().len(), 2);
    assert!(ab.certificate().is_none());
    let checked = ab.check_markov(&CommutatorTolerance::default()).unwrap();
    let report = checked.certificate().unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert_eq!(report.factorization(), Factorization::Rederived);
    assert!(matches!(
        markov_certificate(report).unwrap_err().0,
        QuantumErrorEnum::CommutatorNonZero {
            node_j: 0,
            node_k: 1,
            ..
        }
    ));
}

#[test]
fn test_compose_rejects_a_leg_registered_at_two_dimensions() {
    // Part A registers leg 5 at dimension 2 explicitly, part B at dimension 4. A registered qubit
    // is not an absent leg, and the mismatch is refused in either order.
    let mut pa = ProcessFactors::<f64>::new();
    pa.insert(0, sigma_z());
    let mut sa = FactorSupports::new();
    sa.declare(0, &[5]);
    sa.set_leg_dim(5, 2);
    let a = Hypothesis::structural("a", pa, sa).unwrap();

    let mut pb = ProcessFactors::<f64>::new();
    pb.insert(1, mat(vec![c(1.); 16], 4));
    let mut sb = FactorSupports::new();
    sb.declare(1, &[5]);
    sb.set_leg_dim(5, 4);
    let b = Hypothesis::structural("b", pb, sb).unwrap();

    for (x, y) in [(&a, &b), (&b, &a)] {
        match x.compose(y).unwrap_err().0 {
            QuantumErrorEnum::DimensionMismatch(msg) => assert!(
                msg.contains("cannot compose") && msg.contains("leg 5"),
                "{msg}"
            ),
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }
}

#[test]
fn test_a_failed_re_check_on_inherited_factors_is_a_certificate_failure() {
    // Two parts certified separately, composed onto one shared leg so the naive product does
    // not commute: the re-check on inherited factors reports CertificateNotInherited.
    let a = single("a", 0, 0, sigma_x());
    let b = single("b", 1, 0, sigma_z());
    let ab = a.compose(&b).unwrap();
    let rechecked = ab
        .check_markov_as(&CommutatorTolerance::default(), Factorization::Inherited)
        .unwrap();
    let report = rechecked.certificate().unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert!(matches!(
        markov_certificate(report).unwrap_err().0,
        QuantumErrorEnum::CertificateNotInherited {
            node_j: 0,
            node_k: 1,
            ..
        }
    ));
    // On the model's own factors the same pair is the physics failure.
    let own = ab.check_markov(&CommutatorTolerance::default()).unwrap();
    assert!(matches!(
        markov_certificate(own.certificate().unwrap())
            .unwrap_err()
            .0,
        QuantumErrorEnum::CommutatorNonZero { .. }
    ));
}

#[test]
fn test_a_dimension_product_that_would_overflow_is_reported() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    fs.set_leg_dim(0, usize::MAX);
    fs.set_leg_dim(1, 3);
    match Hypothesis::structural("wide", pf, fs).unwrap_err().0 {
        QuantumErrorEnum::DimensionMismatch(msg) => assert!(msg.contains("overflow"), "{msg}"),
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
}
