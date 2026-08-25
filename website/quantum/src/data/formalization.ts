/**
 * The LEAN 4 formalization, and what is not formalized.
 *
 * Rows are copied from `lean/THEOREM_MAP.md` (the section "Quantum — the
 * partial-trace / Choi foundation and the B1 witness"), with each Rust witness
 * path confirmed by locating the named `#[test]` function in the crate's
 * `tests/formalization_lean/` directory.
 *
 * The deferred list is copied from `deep_causality_quantum/LEAN_QUANTUM.md`.
 * Both files ship in the repository, so a claim here can be checked without
 * building anything.
 */

export interface Theorem {
  /** Identifier used in THEOREM_MAP.md. */
  id: string;
  /** What it states, in the map's own words. */
  statement: string;
  /** LEAN file and declaration. */
  lean: string;
  /** Rust test file and function. */
  witness: string;
}

export const theorems: Theorem[] = [
  {
    id: 'quantum.partial_trace.add',
    statement: 'Tr_B(M + N) = Tr_B M + Tr_B N',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_add',
    witness: 'partial_trace_tests.rs :: test_partial_trace_linearity',
  },
  {
    id: 'quantum.partial_trace.smul',
    statement: 'Tr_B(c • M) = c • Tr_B M',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_smul',
    witness: 'partial_trace_tests.rs :: test_partial_trace_linearity',
  },
  {
    id: 'quantum.partial_trace.kronecker',
    statement: 'Tr_B(X ⊗ Y) = Tr(Y) • X',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_kron',
    witness: 'partial_trace_tests.rs :: test_partial_trace_product_identity',
  },
  {
    id: 'quantum.partial_trace.bimodule',
    statement: 'Tr_B((Z ⊗ 1) · M) = Z · Tr_B M',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_bimodule',
    witness: 'partial_trace_tests.rs :: test_partial_trace_bimodule_law',
  },
  {
    id: 'quantum.partial_trace.bimodule_right',
    statement: 'Tr_B(M · (Z ⊗ 1)) = Tr_B M · Z',
    lean: 'Quantum/PartialTrace.lean :: partialTraceRight_bimodule_right',
    witness: 'partial_trace_tests.rs :: test_partial_trace_bimodule_law',
  },
  {
    id: 'quantum.partial_trace_preservation_boundary',
    statement:
      'A boundary operator that commutes implies its A-part commutes with Tr_B M.',
    lean: 'Quantum/PartialTrace.lean :: partial_trace_preservation_boundary',
    witness: 'partial_trace_tests.rs :: test_partial_trace_preservation_boundary_case',
  },
  {
    id: 'quantum.partial_trace_nonpreservation',
    statement: '[X, Y] = 0 yet [Tr_B X, Tr_B Y] ≠ 0.',
    lean: 'Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation',
    witness:
      'partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample',
  },
  {
    id: 'quantum.partial_trace_nonpreservation.value',
    statement: '[Tr_B X, Tr_B Y] = [[0, 4], [−4, 0]], which is +4i·σy.',
    lean: 'Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation_value',
    witness:
      'partial_trace_tests.rs :: test_partial_trace_nonpreservation_counterexample',
  },
  {
    id: 'quantum.choi.apply_add',
    statement: 'applyChoi J is additive in the state.',
    lean: 'Quantum/Choi.lean :: applyChoi_add',
    witness: 'choi_tests.rs :: test_apply_choi_is_linear',
  },
  {
    id: 'quantum.choi.apply_smul',
    statement: 'applyChoi J (c • A) = c • applyChoi J A',
    lean: 'Quantum/Choi.lean :: applyChoi_smul',
    witness: 'choi_tests.rs :: test_apply_choi_is_linear',
  },
];

export interface Deferred {
  id: string;
  /** What the crate carries today instead of a LEAN proof. */
  carries: string;
  /** Why it is not proved yet, in LEAN_QUANTUM.md's own terms. */
  reason: string;
}

export const deferred: Deferred[] = [
  {
    id: 'Choi–Jamiołkowski round trip',
    carries: 'applyChoi (choiOf E) = E, witnessed by channel_tests.',
    reason: 'Needs net-new Mathlib machinery.',
  },
  {
    id: 'quantum.no_influence',
    carries: 'Numerical and property-test witnesses in the crate.',
    reason: 'Needs net-new Mathlib machinery.',
  },
  {
    id: 'quantum.markov_commutativity',
    carries: 'The freeze check, witnessed by qcm/markov_freeze_tests.',
    reason: 'Needs net-new Mathlib machinery.',
  },
  {
    id: 'quantum.unitary_factorization',
    carries: 'No LEAN statement.',
    reason:
      'Research-grade; needs the direct-sum and C*-structure theory Mathlib lacks.',
  },
  {
    id: 'quantum.classical_embedding',
    carries: 'No LEAN statement.',
    reason: 'Stated as a deferred target.',
  },
  {
    id: 'quantum.cyclic_support',
    carries: 'No LEAN statement.',
    reason: 'Stated as a deferred target.',
  },
  {
    id: 'quantum.verdict.orthomodular',
    carries:
      'The Rust projection-lattice carrier and its law tests are complete (verdict/projection_tests).',
    reason:
      'The LEAN statement extending core.verdict.carriers is future work.',
  },
];
