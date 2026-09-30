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
  /** What backs the behaviour today. */
  today: string;
}

export const deferred: Deferred[] = [
  {
    id: 'CJ reconstruction isomorphism',
    statement: 'Applying the Choi operator of a channel E reconstructs E: applyChoi (choiOf E) = E.',
    today: 'The channel tests run the round trip.',
  },
  {
    id: 'quantum.markov_commutativity',
    statement: 'A process operator is Markov for a graph when it factorizes into pairwise commuting Choi operators.',
    today: 'The freeze check tests it, and the tests name the offending pair (markov_freeze_tests).',
  },
  {
    id: 'quantum.no_influence',
    statement: 'A does not influence D exactly when the partial trace of the process operator over B factors as the marginal on D given C, tensored with the identity on A.',
    today: 'Numerical and property-test witnesses (lean/THEOREM_MAP.md).',
  },
  {
    id: 'quantum.unitary_factorization',
    statement: 'For unitary channels the commuting factorization holds automatically.',
    today: 'LEAN_QUANTUM.md marks it research-grade: the proof needs direct-sum and C*-algebra theory that Mathlib lacks.',
  },
  {
    id: 'quantum.classical_embedding',
    statement: 'Classical causal models are the special case of a diagonal process operator.',
    today: 'Numerical and property-test witnesses. lean/THEOREM_MAP.md names the target and gives no statement.',
  },
  {
    id: 'quantum.cyclic_support',
    statement: 'lean/THEOREM_MAP.md names this target and states no theorem for it.',
    today: 'The builder refuses cyclic structures by decision, and the tests cover the refusal.',
  },
  {
    id: 'quantum.verdict.orthomodular',
    statement: 'The projection lattice is orthomodular.',
    today: 'The Rust verdict carrier and its law tests are complete (projection_tests). The Lean statement would extend core.verdict.carriers.',
  },
];
