/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Haruna logical gates as physical-gate programs.
//!
//! Every expected program here is read off Junichi Haruna, *Note on Logical
//! Gates by Gauge Field Formalism of Quantum Error Correction*, arXiv:2511.15224
//! (the PDF is in `deep_causality_quantum/papers/`), Table 1 and the equations
//! it summarises:
//!
//! - `Z̄(γ) = ∏_k Z_{i_k}` and `X̄(γ̃) = ∏_k X_{ĩ_k}`, Table 1 rows 1 and 2.
//! - `S̄(γ) = ∏_k S_{j_k} · ∏_{k₁<k₂} CZ_{j_{k₁} j_{k₂}}`, Eq. (3.17).
//! - `T̄(γ) = ∏_k T_{i_k} · ∏_{k₁<k₂} CS†_{k₁k₂} · ∏_{k₁<k₂<k₃} CCZ_{k₁k₂k₃}`,
//!   Eq. (3.59).
//! - `CZ̄(γ₁, γ₂) = ∏_{k,l} CZ_{i_k, j_l}` over the Cartesian product, with
//!   `CZ_{i,i} = Z_i` (§3.3).
//! - `H̄(γ) = e^{-iπ/4} S̄(γ) ∏_k H_{ĩ_k} S̄(γ̃) ∏_k H_{ĩ_k} S̄(γ)`, Eq. (3.27).
//! - The `C^{m-1}Z` reduction rule from Table 1's caption:
//!   `C³Z_{i,i,j,k} = C²Z_{i,j,k}` and `C²Z_{i,i,i} = CZ_{i,i} = Z_i`.

use deep_causality_homology::Gf2Chain;
use deep_causality_quantum::{
    GateOp, QuantumCircuit, QuantumErrorEnum, TUPLE_ENUMERATION_CAP, logical_cz, logical_hadamard,
    logical_multi_cz, logical_multi_cz_with_cap, logical_s, logical_t, logical_t_with_cap,
    logical_x, logical_z, multi_cz_tuple_count, t_tuple_count,
};

type C = Gf2Chain<u64>;

const N: usize = 8;

/// A 1-chain over an 8-qubit register with `support` set.
fn chain(support: &[usize]) -> C {
    C::from_support(N, 1, support).unwrap()
}

// ---------------------------------------------------------------------------
// Table 1 rows 1 and 2: the transversal gates.
// ---------------------------------------------------------------------------

#[test]
fn test_logical_z_is_transversal_z_on_the_support() {
    assert_eq!(
        logical_z(&chain(&[1, 4, 6])),
        vec![GateOp::Z(1), GateOp::Z(4), GateOp::Z(6)]
    );
}

#[test]
fn test_logical_x_is_transversal_x_on_the_support() {
    assert_eq!(logical_x(&chain(&[0, 7])), vec![GateOp::X(0), GateOp::X(7)]);
}

#[test]
fn test_the_empty_chain_yields_the_empty_program() {
    // A chain with no support is the identity logical operator, and the empty
    // product is the empty circuit rather than an error.
    assert!(logical_z(&chain(&[])).is_empty());
    assert!(logical_x(&chain(&[])).is_empty());
    assert!(logical_s(&chain(&[])).is_empty());
    assert!(logical_t(&chain(&[])).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Eq. (3.17): S̄(γ) = ∏ S · ∏ CZ over pairs.
// ---------------------------------------------------------------------------

#[test]
fn test_logical_s_matches_equation_3_17() {
    // supp(γ) = {1, 3, 5}: three transversal S, then CZ on each of the C(3,2)
    // pairs in increasing order.
    assert_eq!(
        logical_s(&chain(&[1, 3, 5])),
        vec![
            GateOp::S(1),
            GateOp::S(3),
            GateOp::S(5),
            GateOp::Cz {
                control: 1,
                target: 3
            },
            GateOp::Cz {
                control: 1,
                target: 5
            },
            GateOp::Cz {
                control: 3,
                target: 5
            },
        ]
    );
}

#[test]
fn test_logical_s_on_a_weight_one_chain_is_a_single_s() {
    // C(1,2) = 0 pairs, so no CZ. The degenerate case Eq. (3.17) still covers.
    assert_eq!(logical_s(&chain(&[2])), vec![GateOp::S(2)]);
}

#[test]
fn test_logical_s_gate_counts_follow_the_binomials() {
    // |supp| transversal gates and C(|supp|, 2) pairs, at four weights.
    for w in 1..=4usize {
        let support: Vec<usize> = (0..w).collect();
        let ops = logical_s(&chain(&support));
        assert_eq!(ops.len(), w + binomial(w, 2), "weight {}", w);
    }
}

// ---------------------------------------------------------------------------
// Eq. (3.59): T̄(γ) = ∏ T · ∏ CS† over pairs · ∏ CCZ over triples.
// ---------------------------------------------------------------------------

#[test]
fn test_logical_t_matches_equation_3_59() {
    assert_eq!(
        logical_t(&chain(&[0, 2, 3])).unwrap(),
        vec![
            GateOp::T(0),
            GateOp::T(2),
            GateOp::T(3),
            GateOp::Csdg {
                control: 0,
                target: 2
            },
            GateOp::Csdg {
                control: 0,
                target: 3
            },
            GateOp::Csdg {
                control: 2,
                target: 3
            },
            GateOp::Ccz {
                q0: 0,
                q1: 2,
                q2: 3
            },
        ]
    );
}

/// `C(n, k)` for the small `n` these tests use. Written multiplicatively so the
/// weight-one and weight-two cases do not underflow `usize` on `n - 2`.
fn binomial(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let mut acc = 1usize;
    for i in 0..k {
        acc = acc * (n - i) / (i + 1);
    }
    acc
}

#[test]
fn test_logical_t_gate_counts_follow_the_binomials() {
    // |supp| + C(|supp|,2) + C(|supp|,3), which is the shape Eq. (3.59) fixes.
    for w in 1..=5usize {
        let support: Vec<usize> = (0..w).collect();
        let ops = logical_t(&chain(&support)).unwrap();
        let expected = w + binomial(w, 2) + binomial(w, 3);
        assert_eq!(ops.len(), expected, "weight {}", w);
    }
}

#[test]
fn test_logical_t_has_no_ccz_below_weight_three() {
    // C(2,3) = 0, so a weight-two chain gets no triple factor.
    let ops = logical_t(&chain(&[1, 4])).unwrap();
    assert!(!ops.iter().any(|g| matches!(g, GateOp::Ccz { .. })));
}

// ---------------------------------------------------------------------------
// §3.3: CZ̄(γ₁, γ₂) over the Cartesian product, with CZ_{i,i} = Z_i.
// ---------------------------------------------------------------------------

#[test]
fn test_logical_cz_is_the_full_cartesian_product() {
    // Disjoint supports {0, 1} and {5}: 2 × 1 = 2 gates, and no reduction.
    assert_eq!(
        logical_cz(&chain(&[0, 1]), &chain(&[5])).unwrap(),
        vec![
            GateOp::Cz {
                control: 0,
                target: 5
            },
            GateOp::Cz {
                control: 1,
                target: 5
            },
        ]
    );
}

#[test]
fn test_logical_cz_reduces_a_coincident_pair_to_z() {
    // The paper defines CZ_{i,i} = exp(iπ z_i z_i) = exp(iπ z_i) = Z_i. Supports
    // {2, 3} and {3} share qubit 3, so that factor is a single-qubit Z.
    assert_eq!(
        logical_cz(&chain(&[2, 3]), &chain(&[3])).unwrap(),
        vec![
            GateOp::Cz {
                control: 2,
                target: 3
            },
            GateOp::Z(3),
        ]
    );
}

#[test]
fn test_logical_cz_on_identical_chains_is_all_z() {
    // γ₁ = γ₂ makes every diagonal factor coincide, so the product is the
    // transversal Z on the support, which is Z̄(γ) itself.
    let g = chain(&[1, 6]);
    let ops = logical_cz(&g, &g).unwrap();
    let zs: Vec<&GateOp> = ops.iter().filter(|g| matches!(g, GateOp::Z(_))).collect();
    assert_eq!(zs, vec![&GateOp::Z(1), &GateOp::Z(6)]);
    assert_eq!(ops.len(), 4);
}

#[test]
fn test_logical_cz_rejects_chains_over_different_registers() {
    let a = chain(&[0]);
    let b = C::from_support(16, 1, &[0]).unwrap();
    assert!(logical_cz(&a, &b).is_err());
}

// ---------------------------------------------------------------------------
// Table 1 row 6 and its caption: C^{m-1}Z and the reduction rule.
// ---------------------------------------------------------------------------

#[test]
fn test_multi_cz_over_three_disjoint_chains_is_ccz() {
    assert_eq!(
        logical_multi_cz(&[&chain(&[0]), &chain(&[1]), &chain(&[2])]).unwrap(),
        vec![GateOp::Ccz {
            q0: 0,
            q1: 1,
            q2: 2
        }]
    );
}

#[test]
fn test_multi_cz_applies_the_papers_reduction_rule() {
    // Table 1's caption: C³Z_{i,i,j,k} = C²Z_{i,j,k}. Four chains where the
    // first two coincide on qubit 0 must yield a three-index CCZ, not a
    // four-index gate with a repeat.
    assert_eq!(
        logical_multi_cz(&[&chain(&[0]), &chain(&[0]), &chain(&[1]), &chain(&[2])]).unwrap(),
        vec![GateOp::Ccz {
            q0: 0,
            q1: 1,
            q2: 2
        }]
    );
    // And C²Z_{i,i,i} = CZ_{i,i} = Z_i.
    assert_eq!(
        logical_multi_cz(&[&chain(&[3]), &chain(&[3]), &chain(&[3])]).unwrap(),
        vec![GateOp::Z(3)]
    );
}

#[test]
fn test_multi_cz_at_four_distinct_chains_uses_the_general_form() {
    assert_eq!(
        logical_multi_cz(&[&chain(&[0]), &chain(&[1]), &chain(&[2]), &chain(&[3])]).unwrap(),
        vec![GateOp::Cmz {
            qubits: vec![0, 1, 2, 3]
        }]
    );
}

#[test]
fn test_multi_cz_ranges_over_the_whole_product() {
    // Supports of size 2 and 2 give 4 tuples.
    let ops = logical_multi_cz(&[&chain(&[0, 1]), &chain(&[2, 3])]).unwrap();
    assert_eq!(ops.len(), 4);
}

#[test]
fn test_multi_cz_rejects_an_empty_chain_list() {
    let empty: [&C; 0] = [];
    assert!(logical_multi_cz(&empty).is_err());
}

#[test]
fn test_multi_cz_with_an_unsupported_chain_is_the_empty_program() {
    // An empty support makes the Cartesian product empty, so the logical gate
    // is the identity and its program is empty.
    assert!(
        logical_multi_cz(&[&chain(&[0]), &chain(&[])])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_an_empty_support_after_a_saturated_product_still_counts_zero() {
    // Twenty-two weight-eight chains multiply to 2^66, past u64::MAX, so the running product
    // saturates. An empty support after them makes the Cartesian product empty all the same,
    // and the program is the identity rather than a refusal above the cap.
    let full = chain(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let empty = chain(&[]);
    let mut chains: Vec<&C> = vec![&full; 22];
    assert_eq!(multi_cz_tuple_count(&chains), u64::MAX);
    chains.push(&empty);
    assert_eq!(multi_cz_tuple_count(&chains), 0);
    assert!(logical_multi_cz(&chains).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Eq. (3.27): the logical Hadamard and its global phase.
// ---------------------------------------------------------------------------

#[test]
fn test_logical_hadamard_matches_equation_3_27() {
    // γ = {0}, γ̃ = {1}. The program is S(γ) · H over supp(γ̃) · S(γ̃) · H · S(γ),
    // which at weight one on each side is S(0), H(1), S(1), H(1), S(0).
    let (ops, _) = logical_hadamard::<u64, f64>(&chain(&[0]), &chain(&[1])).unwrap();
    assert_eq!(
        ops,
        vec![
            GateOp::S(0),
            GateOp::H(1),
            GateOp::S(1),
            GateOp::H(1),
            GateOp::S(0),
        ]
    );
}

#[test]
fn test_logical_hadamard_conjugates_with_the_magnetic_support() {
    // The transversal Hadamards run over supp(γ̃), not supp(γ). With γ̃ = {2, 5}
    // there are two H per conjugating layer, and they bracket S̄(γ̃).
    let (ops, _) = logical_hadamard::<u64, f64>(&chain(&[0]), &chain(&[2, 5])).unwrap();
    let h_targets: Vec<usize> = ops
        .iter()
        .filter_map(|g| match g {
            GateOp::H(q) => Some(*q),
            _ => None,
        })
        .collect();
    assert_eq!(h_targets, vec![2, 5, 2, 5]);
}

#[test]
fn test_logical_hadamard_returns_the_minus_pi_over_four_phase() {
    // Table 1 carries e^{-iπ/4} = cos(π/4) − i sin(π/4) = (√2/2, −√2/2).
    let (_, phase) = logical_hadamard::<u64, f64>(&chain(&[0]), &chain(&[1])).unwrap();
    let root_half = core::f64::consts::FRAC_1_SQRT_2;
    assert!((phase.re - root_half).abs() < 1e-15, "re = {}", phase.re);
    assert!((phase.im + root_half).abs() < 1e-15, "im = {}", phase.im);
    // A global phase is a unit complex number; a wrong sign or magnitude here
    // would change a controlled-H into a different gate.
    assert!((phase.re * phase.re + phase.im * phase.im - 1.0).abs() < 1e-15);
}

#[test]
fn test_logical_hadamard_rejects_chains_over_different_registers() {
    let a = chain(&[0]);
    let b = C::from_support(16, 1, &[1]).unwrap();
    assert!(logical_hadamard::<u64, f64>(&a, &b).is_err());
}

// ---------------------------------------------------------------------------
// The programs are runnable circuits, which is the point of the retyping.
// ---------------------------------------------------------------------------

#[test]
fn test_every_gate_program_builds_a_valid_circuit() {
    // `QuantumCircuit::new` rejects an out-of-range index and any gate naming
    // one qubit twice, so this asserts the builders emit nothing the paper's
    // reduction rules should already have removed.
    let g = chain(&[0, 2, 4]);
    let gt = chain(&[1, 3]);
    let programs = vec![
        logical_z(&g),
        logical_x(&gt),
        logical_s(&g),
        logical_t(&g).unwrap(),
        logical_cz(&g, &gt).unwrap(),
        // Overlapping supports, where the CZ_{i,i} reduction fires.
        logical_cz(&g, &g).unwrap(),
        logical_multi_cz(&[&g, &g, &gt]).unwrap(),
        logical_hadamard::<u64, f64>(&g, &gt).unwrap().0,
    ];
    for (i, ops) in programs.into_iter().enumerate() {
        assert!(
            QuantumCircuit::new(N, ops, vec![]).is_ok(),
            "program {} produced an invalid circuit",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// The tuple cap: cost is reported before it is paid.
// ---------------------------------------------------------------------------

#[test]
fn test_the_t_tuple_count_is_the_two_binomials() {
    // C(w, 2) + C(w, 3) for the weights the earlier tests use, plus the two degenerate ones.
    for w in 0..=6usize {
        let support: Vec<usize> = (0..w).collect();
        let expected = (binomial(w, 2) + binomial(w, 3)) as u64;
        assert_eq!(t_tuple_count(&chain(&support)), expected, "weight {w}");
    }
}

#[test]
fn test_a_toric_representative_is_under_the_default_cap() {
    // A weight-four chain is 6 + 4 tuples; the default cap is about a million.
    let g = chain(&[0, 1, 2, 3]);
    assert!(t_tuple_count(&g) < TUPLE_ENUMERATION_CAP);
    assert_eq!(logical_t(&g).unwrap().len(), 4 + 6 + 4);
}

#[test]
fn test_a_wide_representative_fails_loudly_before_allocating() {
    // With the cap set below the count, the builder refuses and names both numbers. The
    // register is small so the "wide" chain is only wide relative to the cap, which is the point:
    // the guard is on the count, not on the register.
    let g = chain(&[0, 1, 2, 3, 4]);
    let count = t_tuple_count(&g);
    assert_eq!(count, 10 + 10);
    let err = logical_t_with_cap(&g, count - 1).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains(&count.to_string()), "names the count: {msg}");
            assert!(
                msg.contains(&(count - 1).to_string()),
                "names the cap: {msg}"
            );
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
    // Exactly at the cap is allowed.
    assert!(logical_t_with_cap(&g, count).is_ok());
}

#[test]
fn test_multi_cz_counts_the_cartesian_product_and_caps_it() {
    let a = chain(&[0, 1, 2]);
    let b = chain(&[3, 4]);
    let c = chain(&[5, 6]);
    assert_eq!(multi_cz_tuple_count(&[&a, &b, &c]), 3 * 2 * 2);
    assert_eq!(logical_multi_cz(&[&a, &b, &c]).unwrap().len(), 12);
    let err = logical_multi_cz_with_cap(&[&a, &b, &c], 11).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    assert_eq!(
        logical_multi_cz_with_cap(&[&a, &b, &c], 12).unwrap().len(),
        12
    );
}
