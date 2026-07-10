# deep_causality_quantum

Quantum causal models (QCM) on the causal monad for
[DeepCausality](https://deepcausality.com).

This crate carries the quantum-information layer of the workspace:

- **`QuantumOps<R>`** — Dirac-notation state operations (`dag`, `bracket`,
  `expectation_value`, `normalize`), implemented for
  `CausalMultiVector<Complex<R>>` from `deep_causality_multivector`.
- **`QuantumGates`** — the standard gate interface (identity, X, Y, Z,
  Hadamard, CNOT).
- **Haruna logical gates** — gauge-field-formalism logical gates
  (S, Z, X, Hadamard, CZ, T) after Haruna (2025), arXiv:2511.15224.
- **Kernels** — Born probability, expectation value, gate application,
  commutator, and fidelity over `HilbertState<R>` (the pure-state ket, which
  stays in `deep_causality_multivector`), with `PropagatingEffect` wrappers
  lifting each kernel into the causal monad.
- **`QuantumError`** — a typed error (outer newtype over `QuantumErrorEnum`)
  naming the exact failure: dimension/metric mismatch, non-finite values,
  normalization, non-positive operators, non-CPTP channels, and freeze-time
  commutativity failures.

All Clifford metric signatures come from `deep_causality_metric` — the
workspace's metric single source of truth; this crate defines no metric type
of its own.

The crate separates two quantum modalities by construction: the **verifiable**
path (deterministic simulated Choi–Jamiołkowski operators, checked at the
freeze boundary and backed by Lean proofs) is the default build; the
**emergent** path (a physical QPU call as a monadic effect) is a typed seam
only.

## License

MIT. See [LICENSE](LICENSE).
