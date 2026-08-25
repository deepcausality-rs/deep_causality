/**
 * The public surface of `deep_causality_quantum`, as it is exported today.
 *
 * Every row was read off the source under `deep_causality_quantum/src/` and
 * checked against the re-export chain in `src/lib.rs`. Signatures are quoted
 * with their generic parameters and dropped `where` clauses; the bounds live
 * with the source, which is linked from every layer page.
 *
 * `feature` records the cargo feature that gates the item. An item with no
 * feature compiles in every build, including `no-std`.
 */

export type Layer = 'operator' | 'qcm' | 'verdict' | 'gates' | 'qpu' | 'error';
export type Kind = 'struct' | 'enum' | 'trait' | 'fn' | 'alias';

export interface ApiItem {
  name: string;
  kind: Kind;
  layer: Layer;
  /** Signature or declaration, trimmed of its `where` clause. */
  signature: string;
  /** One line: what it is for. */
  role: string;
  /** Path under `src/`, without the extension. */
  module: string;
  /** Cargo feature gating the item, if any. */
  feature?: 'qcm' | 'qpu';
}

export const api: ApiItem[] = [
  // ---------------------------------------------------------------- error ---
  {
    name: 'QuantumError',
    kind: 'struct',
    layer: 'error',
    signature: 'pub struct QuantumError(QuantumErrorEnum)',
    role: 'The outer newtype every fallible operation returns, with one constructor per variant.',
    module: 'error/quantum_error',
  },
  {
    name: 'QuantumErrorEnum',
    kind: 'enum',
    layer: 'error',
    signature: 'pub enum QuantumErrorEnum { … }',
    role: 'Twelve variants naming the exact failure. Listed in full on the error page.',
    module: 'error/quantum_error',
  },

  // ------------------------------------------------------------- operator ---
  {
    name: 'DensityMatrix',
    kind: 'struct',
    layer: 'operator',
    signature: 'pub struct DensityMatrix<R: RealField>',
    role: 'A validated mixed state. Every constructor enforces Hermitian, positive semidefinite and unit trace.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::new',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn new(matrix: CausalTensor<Complex<R>>) -> Result<Self, QuantumError>',
    role: 'Validates a square complex matrix at the default tolerance.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::with_tolerance',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn with_tolerance(matrix: CausalTensor<Complex<R>>, tol: R) -> Result<Self, QuantumError>',
    role: 'The same validation against a caller-supplied tolerance.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::from_ket',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn from_ket(ket: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError>',
    role: 'The pure state |ψ⟩⟨ψ| built from a column vector.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::from_ensemble',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn from_ensemble(ensemble: &[(R, CausalTensor<Complex<R>>)]) -> Result<Self, QuantumError>',
    role: 'A convex mixture Σ pᵢ|ψᵢ⟩⟨ψᵢ| from weighted kets.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::from_choi',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn from_choi(choi: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError>',
    role: 'A normalized state read out of a Choi operator.',
    module: 'types/density_matrix',
  },
  {
    name: 'DensityMatrix::purity',
    kind: 'fn',
    layer: 'operator',
    signature: 'pub fn purity(&self) -> R',
    role: 'Tr(ρ²). Paired with `is_pure(tol)` for the pure-state test.',
    module: 'types/density_matrix',
  },
  {
    name: 'choi_from_kraus',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn choi_from_kraus<R>(kraus: &[CausalTensor<Complex<R>>]) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: 'Builds the Choi operator J(E) = Σ_ik |i⟩⟨k| ⊗ E(|i⟩⟨k|) from a Kraus family.',
    module: 'types/qgates/channel',
  },
  {
    name: 'kraus_from_choi',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn kraus_from_choi<R>(choi: &CausalTensor<Complex<R>>, d_in: usize, d_out: usize, tol: R) -> Result<Vec<CausalTensor<Complex<R>>>, QuantumError>',
    role: 'The other direction, through the Hermitian eigendecomposition. Gated on finiteness and Hermiticity.',
    module: 'types/qgates/channel',
  },
  {
    name: 'apply_kraus',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn apply_kraus<R>(kraus: &[CausalTensor<Complex<R>>], rho: &CausalTensor<Complex<R>>) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: 'Applies a channel through its Kraus family.',
    module: 'types/qgates/channel',
  },
  {
    name: 'apply_choi',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn apply_choi<R>(choi: &CausalTensor<Complex<R>>, rho: &CausalTensor<Complex<R>>, d_in: usize, d_out: usize) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: 'Applies the same channel through its Choi operator. The two routes agree, and a test holds that line.',
    module: 'types/qgates/channel',
  },
  {
    name: 'check_completely_positive',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn check_completely_positive<R>(choi: &CausalTensor<Complex<R>>, tol: R) -> Result<(), QuantumError>',
    role: 'E is completely positive iff J ⪰ 0. Fails with NonPositiveOperator.',
    module: 'types/qgates/channel',
  },
  {
    name: 'check_trace_preserving',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn check_trace_preserving<R>(choi: &CausalTensor<Complex<R>>, d_in: usize, d_out: usize, tol: R) -> Result<(), QuantumError>',
    role: 'E is trace-preserving iff Tr_out(J) = I_in. Fails with NonCptpChannel.',
    module: 'types/qgates/channel',
  },
  {
    name: 'partial_trace',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn partial_trace<R>(op: &CausalTensor<Complex<R>>, dims: &[usize], traced: &[usize]) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: 'Traces out a named subset of tensor legs. The classic Tr_B is partial_trace(op, &[d_a, d_b], &[1]).',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'embed_on_legs',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn embed_on_legs<R>(op: &CausalTensor<Complex<R>>, op_legs: &BTreeSet<usize>, space: &BTreeMap<usize, usize>) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: 'Lifts an operator from its own legs onto a larger labelled space. This is what puts two factors on a common support.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'matrix_commutator',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn matrix_commutator<R>(a: &CausalTensor<Complex<R>>, b: &CausalTensor<Complex<R>>) -> Result<CausalTensor<Complex<R>>, QuantumError>',
    role: '[A, B] = AB − BA on square complex matrices.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'matrix_trace',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn matrix_trace<R>(op: &CausalTensor<Complex<R>>) -> Result<Complex<R>, QuantumError>',
    role: 'The trace of a square complex matrix.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'frobenius_norm',
    kind: 'fn',
    layer: 'operator',
    signature: 'pub fn frobenius_norm<R>(op: &CausalTensor<Complex<R>>) -> R',
    role: 'The norm the freeze check measures a commutator in.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'hermiticity_defect',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn hermiticity_defect<R>(op: &CausalTensor<Complex<R>>) -> Result<R, QuantumError>',
    role: 'How far an operator is from Hermitian. Positive-semidefiniteness needs this first.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'identity_matrix',
    kind: 'fn',
    layer: 'operator',
    signature: 'pub fn identity_matrix<R>(d: usize) -> CausalTensor<Complex<R>>',
    role: 'The d×d identity, used by the trace-preservation check and the orthocomplement.',
    module: 'types/qgates/operator_linalg',
  },
  {
    name: 'supports_intersect',
    kind: 'fn',
    layer: 'operator',
    signature:
      'pub fn supports_intersect(a: &BTreeSet<usize>, b: &BTreeSet<usize>) -> bool',
    role: 'Whether two Hilbert supports share a leg. Disjoint supports impose no commutativity obligation.',
    module: 'types/qgates/operator_linalg',
  },

  // ------------------------------------------------------------------ qcm ---
  {
    name: 'ProcessFactors',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct ProcessFactors<R: RealField>',
    role: 'The external store of per-node Choi–Jamiołkowski factors, keyed by graph node index.',
    module: 'types/qcm/process_factors',
    feature: 'qcm',
  },
  {
    name: 'CjFactor',
    kind: 'alias',
    layer: 'qcm',
    signature: 'pub type CjFactor<R> = CausalTensor<Complex<R>>',
    role: 'One factor: a square complex matrix on that node’s support.',
    module: 'types/qcm/process_factors',
    feature: 'qcm',
  },
  {
    name: 'FactorSupports',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct FactorSupports',
    role: 'Which Hilbert legs each factor lives on, and how wide each leg is.',
    module: 'types/qcm/process_factors',
    feature: 'qcm',
  },
  {
    name: 'FactorSupports::from_graph',
    kind: 'fn',
    layer: 'qcm',
    signature:
      'pub fn from_graph<T, G, R>(graph: &G, factors: &ProcessFactors<R>) -> Result<Self, QuantumError>',
    role: 'Derives the supports from the graph rather than having the caller restate them.',
    module: 'types/qcm/process_factors',
    feature: 'qcm',
  },
  {
    name: 'CommutatorTolerance',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct CommutatorTolerance<R: RealField>',
    role: 'The forward-error budget the commutator is measured against. Defaults to C = 8 and u = ε.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'CommutatorTolerance::threshold',
    kind: 'fn',
    layer: 'qcm',
    signature:
      'pub fn threshold(&self, node_j: usize, node_k: usize, dim: usize, norm_j: R, norm_k: R) -> R',
    role: 'The acceptance threshold for one pair, embedded on its common support.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'CommutatorCheck',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct CommutatorCheck<R: RealField>',
    role: 'One recorded pair test: the norm, the threshold, the margin, and whether it passed.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'QuantumMarkovReport',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct QuantumMarkovReport<R: RealField>',
    role: 'One entry per intersecting-support pair, with `tested_pairs()` and `worst_margin()`.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'quantum_markov_check',
    kind: 'fn',
    layer: 'qcm',
    signature:
      'pub fn quantum_markov_check<R>(factors: &ProcessFactors<R>, supports: &FactorSupports, tolerance: &CommutatorTolerance<R>) -> Result<QuantumMarkovReport<R>, QuantumError>',
    role: 'The check on its own, without a graph. Returns the report, or names the first offending pair.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'freeze_quantum',
    kind: 'fn',
    layer: 'qcm',
    signature:
      'pub fn freeze_quantum<T, G, R>(graph: &mut G, state_writers: &[usize], factors: &ProcessFactors<R>, supports: &FactorSupports, tolerance: &CommutatorTolerance<R>, faithfulness: Option<(&[usize], &[usize])>) -> Result<QuantumMarkovReport<R>, QuantumError>',
    role: 'The freeze gate: the engine’s built-in checks, then commutativity, then C₃-exclusion when input and output systems are declared.',
    module: 'types/qcm/markov_freeze',
    feature: 'qcm',
  },
  {
    name: 'CausalStructure',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct CausalStructure',
    role: 'A bipartite influence relation between declared input and output systems.',
    module: 'types/qcm/faithfulness',
    feature: 'qcm',
  },
  {
    name: 'CausalStructure::from_graph_reachability',
    kind: 'fn',
    layer: 'qcm',
    signature:
      'pub fn from_graph_reachability<T, G>(graph: &G, inputs: &[usize], outputs: &[usize]) -> Result<Self, QuantumError>',
    role: 'Reads the relation off the frozen graph instead of trusting a hand-written one.',
    module: 'types/qcm/faithfulness',
    feature: 'qcm',
  },
  {
    name: 'CausalStructure::find_c3',
    kind: 'fn',
    layer: 'qcm',
    signature: 'pub fn find_c3(&self) -> Option<([usize; 3], [usize; 3])>',
    role: 'Searches every 3×3 induced block for the C₃ obstruction and returns the three inputs and three outputs that carry it.',
    module: 'types/qcm/faithfulness',
    feature: 'qcm',
  },
  {
    name: 'CausalStructure::check_c3_exclusion',
    kind: 'fn',
    layer: 'qcm',
    signature: 'pub fn check_c3_exclusion(&self) -> Result<(), QuantumError>',
    role: 'The faithfulness check itself. Fails with NotFaithfullyRepresentable, naming the obstruction.',
    module: 'types/qcm/faithfulness',
    feature: 'qcm',
  },
  {
    name: 'EnvironmentalPrep',
    kind: 'struct',
    layer: 'qcm',
    signature: 'pub struct EnvironmentalPrep<R: RealField>',
    role: 'An immutable environmental preparation wrapping a validated DensityMatrix.',
    module: 'types/qcm/environment',
    feature: 'qcm',
  },

  // -------------------------------------------------------------- verdict ---
  {
    name: 'Projection',
    kind: 'struct',
    layer: 'verdict',
    signature: 'pub struct Projection<R: RealField, const D: usize>',
    role: 'A Hermitian idempotent on a fixed D-dimensional space, carrying the `Verdict` impl.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::new',
    kind: 'fn',
    layer: 'verdict',
    signature: 'pub fn new(p: CausalTensor<Complex<R>>) -> Result<Self, QuantumError>',
    role: 'Rejects anything that is not Hermitian and idempotent to tolerance.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::from_ket',
    kind: 'fn',
    layer: 'verdict',
    signature:
      'pub fn from_ket(ket: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError>',
    role: 'The rank-one projector onto a normalized ket.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::zero / one',
    kind: 'fn',
    layer: 'verdict',
    signature: 'pub fn zero() -> Self · pub fn one() -> Self',
    role: 'The lattice bounds. The const dimension is what lets them take no arguments.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::rank',
    kind: 'fn',
    layer: 'verdict',
    signature: 'pub fn rank(&self) -> usize',
    role: 'The dimension of the subspace the projection selects.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::leq',
    kind: 'fn',
    layer: 'verdict',
    signature: 'pub fn leq(&self, other: &Self) -> bool',
    role: 'Subspace containment, the lattice order.',
    module: 'types/verdict/projection',
  },
  {
    name: 'Projection::commutes_with',
    kind: 'fn',
    layer: 'verdict',
    signature: 'pub fn commutes_with(&self, other: &Self) -> bool',
    role: 'Whether two propositions are compatible. Distributivity holds inside a commuting family and fails outside it.',
    module: 'types/verdict/projection',
  },
  {
    name: 'born_projective_probability',
    kind: 'fn',
    layer: 'verdict',
    signature:
      'pub fn born_projective_probability<R, const D: usize>(rho: &DensityMatrix<R>, projection: &Projection<R, D>) -> Result<R, QuantumError>',
    role: 'Tr(Pρ) as a real number, clamped to [0, 1].',
    module: 'types/verdict/born',
  },
  {
    name: 'born_projective_prob',
    kind: 'fn',
    layer: 'verdict',
    signature:
      'pub fn born_projective_prob<R, const D: usize>(rho: &DensityMatrix<R>, projection: &Projection<R, D>) -> Result<Prob, QuantumError>',
    role: 'The same number as a `Prob` MV-algebra verdict, which is what crosses the measurement boundary.',
    module: 'types/verdict/born',
  },

  // ---------------------------------------------------------------- gates ---
  {
    name: 'QuantumGates',
    kind: 'trait',
    layer: 'gates',
    signature: 'pub trait QuantumGates',
    role: 'The standard gate constructors: identity, X, Y, Z, Hadamard, CNOT.',
    module: 'types/qgates/gates',
  },
  {
    name: 'QuantumOps',
    kind: 'trait',
    layer: 'gates',
    signature: 'pub trait QuantumOps<R: RealField>',
    role: 'Dirac-notation state operations: `dag`, `bracket`, `expectation_value`, `normalize`.',
    module: 'types/qgates/gates',
  },
  {
    name: 'Operator / Gate',
    kind: 'alias',
    layer: 'gates',
    signature:
      'pub type Operator<R> = HilbertState<R> · pub type Gate<R> = HilbertState<R>',
    role: 'Both name the same carrier. The ket lives in deep_causality_multivector.',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'born_probability_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn born_probability_kernel<R>(state: &HilbertState<R>, basis: &HilbertState<R>) -> Result<R, QuantumError>',
    role: '|⟨basis|state⟩|².',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'expectation_value_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn expectation_value_kernel<R>(state: &HilbertState<R>, operator: &Operator<R>) -> Result<R, QuantumError>',
    role: '⟨ψ|A|ψ⟩ for a Hermitian A.',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'apply_gate_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn apply_gate_kernel<R>(state: &HilbertState<R>, gate: &Gate<R>) -> Result<HilbertState<R>, QuantumError>',
    role: 'U|ψ⟩.',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'commutator_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn commutator_kernel<R>(a: &Operator<R>, b: &Operator<R>) -> Result<HilbertState<R>, QuantumError>',
    role: '[A, B] on the ket carrier.',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'fidelity_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn fidelity_kernel<R>(ideal: &HilbertState<R>, actual: &HilbertState<R>) -> Result<R, QuantumError>',
    role: 'State fidelity between an intended and a realized state.',
    module: 'types/qgates/mechanics',
  },
  {
    name: 'clifford_conjugation',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn clifford_conjugation<R>(mv: &CausalMultiVector<Complex<R>>) -> CausalMultiVector<Complex<R>>',
    role: 'The Dirac conjugation used as the adjoint on a negative-signature metric.',
    module: 'types/qgates/bridge',
  },
  {
    name: 'dirac_bracket_kernel',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn dirac_bracket_kernel<R>(phi: &HilbertState<R>, psi: &HilbertState<R>) -> Result<Complex<R>, QuantumError>',
    role: 'The inner product on Cl(0,n), where the reversion adjoint is the wrong one.',
    module: 'types/qgates/bridge',
  },
  {
    name: 'logical_z / x / s / hadamard / cz / t',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn logical_z<R>(a_gamma: &CausalMultiVector<Complex<R>>) -> Result<CausalMultiVector<Complex<R>>, QuantumError>',
    role: 'The Haruna gauge-field logical gates. The matrix exponential surfaces overflow and non-convergence as typed errors.',
    module: 'types/qgates/gates_haruna',
  },
  {
    name: 'born_probability',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn born_probability<R>(state: &HilbertState<R>, basis: &HilbertState<R>) -> PropagatingEffect<R>',
    role: 'The causal-monad wrapper. Failure routes to the error channel instead of unwinding.',
    module: 'types/qgates/wrappers',
  },
  {
    name: 'expectation_value',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn expectation_value<R>(state: &HilbertState<R>, operator: &Operator<R>) -> PropagatingEffect<R>',
    role: 'Wrapper over expectation_value_kernel.',
    module: 'types/qgates/wrappers',
  },
  {
    name: 'apply_gate',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn apply_gate<R>(state: &HilbertState<R>, gate: &Gate<R>) -> PropagatingEffect<HilbertState<R>>',
    role: 'Wrapper over apply_gate_kernel.',
    module: 'types/qgates/wrappers',
  },
  {
    name: 'commutator',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn commutator<R>(a: &Operator<R>, b: &Operator<R>) -> PropagatingEffect<HilbertState<R>>',
    role: 'Wrapper over commutator_kernel.',
    module: 'types/qgates/wrappers',
  },
  {
    name: 'fidelity',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn fidelity<R>(ideal: &HilbertState<R>, actual: &HilbertState<R>) -> PropagatingEffect<R>',
    role: 'Wrapper over fidelity_kernel.',
    module: 'types/qgates/wrappers',
  },
  {
    name: 'haruna_*_gate',
    kind: 'fn',
    layer: 'gates',
    signature:
      'pub fn haruna_s_gate<R>(field: &CausalMultiVector<R>) -> PropagatingEffect<Operator<R>>',
    role: 'Wrappers for the six Haruna gates: s, z, x, hadamard, cz, t. The wrapper takes a real field and lifts it into the complex algebra.',
    module: 'types/qgates/wrappers',
  },

  // ------------------------------------------------------------------ qpu ---
  {
    name: 'GateOp',
    kind: 'enum',
    layer: 'qpu',
    signature: 'pub enum GateOp { H, X, Y, Z, S, T, Cnot, Cz }',
    role: 'A reified gate. Plain data: no function pointers, no amplitudes.',
    module: 'types/qpu/circuit',
    feature: 'qpu',
  },
  {
    name: 'QuantumCircuit',
    kind: 'struct',
    layer: 'qpu',
    signature: 'pub struct QuantumCircuit',
    role: 'A register width, an ordered gate program, and a measurement list. Storable and comparable.',
    module: 'types/qpu/circuit',
    feature: 'qpu',
  },
  {
    name: 'ShotHistogram',
    kind: 'trait',
    layer: 'qpu',
    signature: 'pub trait ShotHistogram',
    role: 'Classical outcome counts. It never exposes amplitudes, which is what pins the coherence boundary at the type level.',
    module: 'types/qpu/sampler',
    feature: 'qpu',
  },
  {
    name: 'CountHistogram',
    kind: 'struct',
    layer: 'qpu',
    signature: 'pub struct CountHistogram',
    role: 'The concrete histogram the in-process simulator returns.',
    module: 'types/qpu/sampler',
    feature: 'qpu',
  },
  {
    name: 'QpuSampler',
    kind: 'trait',
    layer: 'qpu',
    signature: 'pub trait QpuSampler',
    role: 'The sampler seam, used only as a generic bound. `sample` returns shots; `calibration` returns device metadata.',
    module: 'types/qpu/sampler',
    feature: 'qpu',
  },
  {
    name: 'SimQpu',
    kind: 'struct',
    layer: 'qpu',
    signature: 'pub struct SimQpu',
    role: 'A deterministic dense state-vector simulator, capped at 24 qubits. Amplitudes never leave it.',
    module: 'types/qpu/sim',
    feature: 'qpu',
  },
  {
    name: 'SimCalibration',
    kind: 'struct',
    layer: 'qpu',
    signature: 'pub struct SimCalibration { name: String, seed: u64 }',
    role: 'What the simulator surfaces on the context channel.',
    module: 'types/qpu/sim',
    feature: 'qpu',
  },
  {
    name: 'QpuParams',
    kind: 'struct',
    layer: 'qpu',
    signature:
      'pub struct QpuParams { num_qubits, num_ops, num_measured, shots }',
    role: 'The requested-parameter summary routed to the state channel.',
    module: 'types/qpu/bridge',
    feature: 'qpu',
  },
  {
    name: 'shots_to_qubit_bernoulli',
    kind: 'fn',
    layer: 'qpu',
    signature:
      'pub fn shots_to_qubit_bernoulli<H: ShotHistogram>(hist: &H, bit_index: usize) -> Result<Uncertain<bool>, QuantumError>',
    role: 'One measured qubit as a Bernoulli `Uncertain<bool>` at the measured frequency.',
    module: 'types/qpu/bridge',
    feature: 'qpu',
  },
  {
    name: 'shots_to_observable',
    kind: 'fn',
    layer: 'qpu',
    signature:
      'pub fn shots_to_observable<H, F>(hist: &H, value_of: F) -> Result<Uncertain<f64>, QuantumError>',
    role: 'An observable as a normal `Uncertain<f64>`, accumulated from the count entries rather than one sample per shot.',
    module: 'types/qpu/bridge',
    feature: 'qpu',
  },
  {
    name: 'qpu_effect',
    kind: 'fn',
    layer: 'qpu',
    signature:
      'pub fn qpu_effect<S>(sampler: &S, circuit: &QuantumCircuit, shots: u64) -> CausalEffectPropagationProcess<S::Shots, QpuParams, S::Calibration, CausalityError, EffectLog>',
    role: 'The lift into the arity-5 causal monad: shots on value, parameters on state, calibration on context, provenance on log, a job failure on error.',
    module: 'types/qpu/bridge',
    feature: 'qpu',
  },
];

/** Layer display names and the order the site presents them in. */
export const layers: { key: Layer; label: string; href: string }[] = [
  { key: 'qcm', label: 'Quantum causal models', href: '/qcm/' },
  { key: 'operator', label: 'Operator layer', href: '/operators/' },
  { key: 'gates', label: 'Gate kernels', href: '/gates/' },
  { key: 'verdict', label: 'Verdicts', href: '/verdicts/' },
  { key: 'qpu', label: 'QPU seam', href: '/modalities/' },
  { key: 'error', label: 'Errors', href: '/errors/' },
];

export const byLayer = (layer: Layer) => api.filter((i) => i.layer === layer);
