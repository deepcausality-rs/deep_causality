/**
 * The Lean 4 formalization: what is proved, and what is stated and not proved.
 *
 * `theorems` is generated from the quantum section of `lean/THEOREM_MAP.md`, one
 * entry per row marked `proved`, in the map's order. Each witness is the Rust test
 * the map names. `deferred` lists the targets the same section and
 * `deep_causality_quantum/LEAN_QUANTUM.md` state and do not prove. Both files ship
 * in the repository, so every claim here can be read without building anything.
 */

export interface Theorem {
  /** Identifier used in THEOREM_MAP.md. */
  id: string;
  /** What it states, in the map's own words. */
  statement: string;
  /** Lean file and declaration. */
  lean: string;
  /** Rust test file and function. */
  witness: string;
}

export const theorems: Theorem[] = [
  {
    id: 'quantum.partial_trace.add',
    statement: 'Tr_B(M+N) = Tr_B M + Tr_B N',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_add',
    witness: 'partial_trace_tests.rs :: test_partial_trace_linearity',
  },
  {
    id: 'quantum.partial_trace.sub',
    statement: 'Tr_B(M−N) = Tr_B M − Tr_B N',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_sub',
    witness: 'partial_trace_tests.rs :: test_partial_trace_linearity',
  },
  {
    id: 'quantum.partial_trace.smul',
    statement: 'Tr_B(c•M) = c•Tr_B M',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_smul',
    witness: 'partial_trace_tests.rs :: test_partial_trace_linearity',
  },
  {
    id: 'quantum.partial_trace.kronecker',
    statement: 'Tr_B(X⊗Y) = Tr(Y)•X',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_kron',
    witness: 'partial_trace_tests.rs :: test_partial_trace_product_identity',
  },
  {
    id: 'quantum.partial_trace.bimodule',
    statement: 'Tr_B((Z⊗1)·M) = Z·Tr_B M',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_bimodule',
    witness: 'partial_trace_tests.rs :: test_partial_trace_bimodule_law',
  },
  {
    id: 'quantum.partial_trace.bimodule_right',
    statement: 'Tr_B(M·(Z⊗1)) = Tr_B M·Z',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_bimodule_right',
    witness: 'partial_trace_tests.rs :: test_partial_trace_bimodule_law',
  },
  {
    id: 'quantum.partial_trace_preservation_boundary',
    statement: 'boundary op commutes ⇒ its A-part commutes with Tr_B M (Q-PTP)',
    lean: 'Quantum/PartialTrace.lean :: partial_trace_preservation_boundary',
    witness: 'partial_trace_tests.rs :: test_partial_trace_preservation_boundary_case',
  },
  {
    id: 'quantum.partial_trace.commutator_transport',
    statement: 'Tr_B([Z⊗1,M]) = [Z,Tr_B M], unconditional (Q-PTT)',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_commutator',
    witness: 'partial_trace_tests.rs :: test_partial_trace_transports_the_commutator_exactly',
  },
  {
    id: 'quantum.partial_trace_nonpreservation',
    statement: '[X,Y]=0 but [Tr_B X, Tr_B Y] ≠ 0 (B1 counterexample)',
    lean: 'Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation',
    witness: 'partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample',
  },
  {
    id: 'quantum.partial_trace_nonpreservation.value',
    statement: '[Tr_B X, Tr_B Y] = [[0,4],[−4,0]] (= +4i·σy)',
    lean: 'Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation_value',
    witness: 'partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample',
  },
  {
    id: 'quantum.choi.apply_add',
    statement: 'applyChoi J is additive in the state',
    lean: 'Quantum/Choi.lean :: applyChoi_add',
    witness: 'choi_tests.rs :: test_apply_choi_is_linear',
  },
  {
    id: 'quantum.choi.apply_smul',
    statement: 'applyChoi J (c•A) = c•applyChoi J A',
    lean: 'Quantum/Choi.lean :: applyChoi_smul',
    witness: 'choi_tests.rs :: test_apply_choi_is_linear',
  },
  {
    id: 'quantum.choi.reconstruction',
    statement: 'applyChoi (choiOf E) A = E A for every linear channel E (the Choi–Jamiołkowski reconstruction)',
    lean: 'Quantum/Choi.lean :: applyChoi_choiOf',
    witness: 'choi_tests.rs :: test_choi_reconstruction_recovers_the_channel',
  },
  {
    id: 'quantum.verdict.orthomodular',
    statement: 'the subspace lattice behind the Projection verdict is orthomodular and not distributive',
    lean: 'Quantum/Orthomodular.lean :: subspace_lattice_orthomodular',
    witness: 'orthomodular_tests.rs :: test_the_orthomodular_law_holds_for_a_line_inside_a_plane',
  },
  {
    id: 'quantum.markov_commutativity',
    statement: 'factors on disjoint legs commute, and a pairwise-commuting product does not depend on its order (Lorenz 2022, Def. 3.3)',
    lean: 'Quantum/Markov.lean :: kron_one_commute_one_kron',
    witness: 'qcm_theorem_tests.rs :: test_factors_on_disjoint_legs_commute',
  },
  {
    id: 'quantum.classical_embedding',
    statement: 'diagonal factors commute pairwise, so a classical model meets the Markov condition',
    lean: 'Quantum/ClassicalEmbedding.lean :: diagonal_commute',
    witness: 'qcm_theorem_tests.rs :: test_diagonal_factors_commute_and_their_kronecker_product_is_diagonal',
  },
  {
    id: 'quantum.no_influence',
    statement: 'the D marginal factors through Tr_A exactly when it depends on the A input only through its trace (Lorenz & Barrett 2021, Def. 1)',
    lean: 'Quantum/NoInfluence.lean :: noInfluence_iff',
    witness: 'qcm_theorem_tests.rs :: test_a_product_unitary_has_no_influence_from_a_to_d',
  },
  {
    id: 'quantum.abstraction.compose_exact',
    statement: 'two commuting abstraction squares paste: (τ₂τ₁)·L = H·(σ₂σ₁) (Lorenz & Tull Prop. 17, exact case)',
    lean: 'Quantum/Abstraction.lean :: abstraction_compose_exact',
    witness: 'composition_tests.rs :: test_exact_links_compose_exactly_on_the_concatenated_code',
  },
  {
    id: 'quantum.abstraction.compose_exact.defect',
    statement: 'zero link defects give a zero composite defect, the form the Rust residual measures',
    lean: 'Quantum/Abstraction.lean :: abstraction_compose_exact_defect',
    witness: 'composition_tests.rs :: test_exact_links_compose_exactly_on_the_concatenated_code',
  },
];

export interface Deferred {
  id: string;
  /** What the target says, in words. */
  statement: string;
  /** Why it is not proved. */
  reason: string;
}

export const deferred: Deferred[] = [
  {
    id: 'quantum.unitary_factorization',
    statement: 'Lorenz and Barrett’s Theorem 1: for unitary channels, the commuting factorization holds automatically.',
    reason: 'Its proof rests on the commutant and direct-sum decomposition of finite-dimensional C*-algebras. The pinned Mathlib has Wedderburn–Artin and C*-algebra basics but not that decomposition.',
  },
];
