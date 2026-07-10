<!--
Blockers-first ordering (per the change directive): Phase 0 resolves every open question and
false-premise before any dependent work. Phases 1–6 do not start until Phase 0 lands its decisions.
Each phase ends with the established loop where Lean is involved: bare-`lean` exit 0 + zero `sorry`,
Rust witness, THEOREM_MAP row, clippy -D warnings, `bazel test //...` green.
-->

## 0. Phase 0 — Resolve blockers and open questions FIRST (gate)

- [ ] 0.1 (B1) Witness the partial-trace counterexample in a standalone test/notebook:
      `X = σx⊗|0><0| + σz⊗|1><1|`, `Y = σx⊗|0><0| − σz⊗|1><1|`; confirm `[X,Y]=0` and
      `[Tr₂X,Tr₂Y] = −4i·σy ≠ 0`. Record it as the refuting witness for `quantum.partial_trace_nonpreservation`
- [ ] 0.2 (B1/Q-PTP) Fix the CONDITIONAL statement to prove: `partial_trace_preservation_boundary`
      (single-node interface / boundary-only shared support ⇒ preservation via `Tr_B((1⊗Z)M)=Z·Tr_B(M)`).
      (DECIDED, Q-PTP: prove at BOUNDARY-ONLY shared support — the `1_B ⊗ Z` bimodule case — with
      single-node interface as a checkable sufficient predicate). Record the general necessary-and-
      sufficient condition as an explicit OPEN target. Note: quantum-causaloid nesting is itself
      unestablished, so this stays OFF the critical path — flat QCM models are the supported path and
      the counterexample/boundary theorem only document the boundary
- [ ] 0.3 (B2) Confirm the in-scope `quantum.*` id set for this change and that the partial-trace + CJ
      Lean foundation is Phase 5's first deliverable (gates all quantum ids); record the deferred ids
- [ ] 0.4 (B3) Write the faithfulness scope guard: claims limited to the proven classes; the general
      Lorenz–Barrett hypothesis and the operator-level `⊕` are explicitly deferred
- [ ] 0.5 (B4/Q-ERR) Fix the operator representation (`CausalTensor<Complex<R>>`) and define the
      crate-local `QuantumError` (outer newtype over `QuantumErrorEnum` of exact variants) replacing
      `PhysicsError`; confirm `deep_causality_tensor` provides the needed complex matmul/SVD/QR
- [ ] 0.6 (B5/Q-KG) Fix the migration scope: the quantum-information kernels move; `klein_gordon`
      STAYS in `deep_causality_physics` (Q-KG decided) — the `kernels/quantum` module splits; list the
      exact 5-file breakage set and the update plan (direct call-site updates, no long-lived shim)
- [ ] 0.7 (B6/Q-QPU) Fix the verifiable/emergent modality boundary and the crate feature layout
      (verifiable default; `qpu` seam off by default, no network/async dep here); lock the `QpuSampler`
      seam shape: generic (`no dyn`), `sample(circuit, shots) -> Result<Shots, QuantumError>` with
      classical `Shots`, an `Uncertain` bridge, and a feature-gated `PropagatingEffect` wrapper
- [ ] 0.8 (B7) Fix the orthomodular `Verdict` newtype design (commuting projection family; no blanket
      operator `Verdict`; Born-rule extraction boundary)
- [ ] 0.9 (B8/Q-TOL) Fix the `Float106` commutator-tolerance policy: condition-driven forward-error
      bound (NOT linear-in-depth); error-propagation analysis through iterated partial trace;
      configurable + instrumented per-check margin capture (shared telemetry with B1(c))
- [ ] 0.10 (B9) Fix crate policy + dependency set (workspace member, `[lints] workspace = true`,
      `unsafe_code = "forbid"`, MSRV 1.93.0, no `dyn`/macros, std; deps listed in the proposal)
- [ ] 0.11 Phase-0 gate: all decisions recorded in `design.md` (update in place if any decision
      changes); prepare the Phase-0 commit message. No Phase-1+ task starts before this is done

## 1. Phase 1 — Crate scaffold + kernel migration

- [ ] 1.1 Create `deep_causality_quantum/` (`Cargo.toml` with `[lints] workspace = true`, MSRV,
      `unsafe_code = "forbid"`, the Phase-0 dependency set); add it to the workspace `members`
- [ ] 1.2 Move `deep_causality_physics/src/kernels/quantum/{gates,gates_haruna,mechanics,wrappers}.rs`
      into the new crate (`klein_gordon*` STAYS in physics per 0.6); replace `PhysicsError` with
      `QuantumError`; keep `QuantumGates`/`QuantumOps`/`Operator`/`Gate` and the Haruna gates
- [ ] 1.3 `HilbertState` stays in `deep_causality_multivector` (unchanged); the new crate depends on it
- [ ] 1.4 Update `deep_causality_physics/src/lib.rs` (drop the moved re-exports; keep `klein_gordon` if
      it stays) and confirm physics still builds
- [ ] 1.5 Re-point the four dependents: `examples/quantum_examples/{quantum_counterfactual,ikkt_matrix_model}`
      and `examples/physics_examples/multi_physics_pipeline/{main,model}.rs`
- [ ] 1.6 `bazel test //...` green; clippy `-D warnings`; prepare the phase commit message

## 2. Phase 2 — The operator / channel layer (net-new foundation, bottom-up)

- [ ] 2.0 L0 primitives: reuse `deep_causality_tensor` matmul/`eigen`/`svd_truncated`/`qr`/`trace`/
      einsum; add the missing GENERIC ops to the tensor crate — conjugate-transpose (`dagger`),
      Kronecker product `⊗`, reshape/index-permute
- [ ] 2.1 L1 ket↔matrix bridge: `HilbertState → column-vector CausalTensor` (`to_ket`/`from_ket`
      alongside `to_matrix()`), so the layer works in the tensor rep and converts at the boundary
- [ ] 2.2 L1 `DensityMatrix<R>` newtype with enforced invariants (Hermitian, PSD via `eigen`, unit
      trace → `QuantumError`); constructors from a ket, an ensemble, and a Choi
- [ ] 2.3 L2 partial trace `Tr_B` (reshape-to-rank-4 + `b=b'` einsum contraction; named-subset
      generalization) with linearity, `Tr_B(X⊗Y)=X·Tr(Y)`, and the bimodule law
      `Tr_B((1_B⊗Z)M)=Z·Tr_B(M)` as tested properties (the Q-PTP boundary identity)
- [ ] 2.4 L3 Choi–Jamiołkowski operator + Kraus: CP ⟺ Choi PSD (`eigen`), TP ⟺ `Tr_out(J)=I`,
      Kraus↔Choi via the PSD-Choi `eigen` decomposition; Choi→Kraus→Choi round-trip test
- [ ] 2.5 L4 commutator `[A,B]` + shared-support detection used by the freeze check
- [ ] 2.6 Property tests incl. monad law 3 (`encapsulation = flat`) exercised over the arity-5 STATE
      channel with complex-MATRIX payloads (not only scalar state); all freeze-critical paths at
      `Complex<Float106>`
- [ ] 2.7 `bazel test //...` green; clippy clean; phase commit message

## 3. Phase 3 — The verifiable QCM slice (simulated CJ + freeze check)

- [ ] 3.1 Operator-valued process (CJ) state carried on the arity-5 STATE channel; the factorization
      `σ = ∏ ρ_{Ai|Pa(Ai)}` as hyperedge parent-set data
- [ ] 3.2 The Layer-D freeze commutativity check: recursive walk computing `[ρ_j, ρ_k]` on shared
      Hilbert supports over hyperedges sharing a source node; sound hard-pass / abort
- [ ] 3.3 Depth-aware `Float106` tolerance (per 0.9) + instrumented-freeze failure capture
- [ ] 3.4 C₃-exclusion faithfulness freeze check (B3, van der Lugt & Lorenz 2508.11762): reject a
      declared causal structure `G` that contains a `C₃` sub-relation (three inputs / three outputs)
      with a `QuantumError`; a C₃-exclusion `G` is faithfully representable. Implement the combinatorial
      `C₃` detection now; the full concept-lattice `L_G` construction may follow later
- [ ] 3.5 Immutable-context handle for the environmental Bell-prep `ρ_A` (write methods unreachable),
      keeping the simulated model in the verifiable region
- [ ] 3.6 The `QpuSampler` seam trait (emergent modality) behind the off-by-default `qpu` feature —
      trait only, no adapter
- [ ] 3.7 `bazel test //...` green; clippy clean; phase commit message

## 4. Phase 4 — The orthomodular Verdict carrier

- [ ] 4.1 Rust newtype over a commuting projection family: `impl Verdict` (`bottom=0`, `top=I`,
      `complement=I−P`, meet/join on ranges); orthomodular-law note; no blanket operator `Verdict`
- [ ] 4.2 Born-rule / projection extraction at the measurement boundary (operator → `Prob` / proposition)
- [ ] 4.3 Tests for the orthomodular laws (and the distributivity failure); `bazel test //...` green;
      phase commit message

## 5. Phase 5 — Lean formalization

- [ ] 5.1 Lean foundation `Quantum/PartialTrace.lean` + `Quantum/Choi.lean` (import Mathlib Hilbert-
      space/linear-algebra; the project's first such consumer): partial trace + CJ + their lemma
      libraries (linearity, positivity, `Tr_B(X⊗Y)=X·Tr(Y)`, bimodule law); zero `sorry`
- [ ] 5.2 `quantum.no_influence` (Def 1 marginal condition) + `quantum.markov_commutativity`
      (Lorenz Def 3.3; the 2-factor free-commutation lemma explicit) — Lean + witness + THEOREM_MAP
- [ ] 5.3 `quantum.unitary_factorization` (Lorenz–Barrett Thm 1; the ≥3-factor commutation) — Lean +
      witness + THEOREM_MAP
- [ ] 5.4 `quantum.partial_trace_nonpreservation` (the B1 counterexample) + `quantum.partial_trace_preservation_boundary`
      (the conditional sufficient theorem) — Lean + witness; the general condition recorded as an open target
- [ ] 5.5 `quantum.classical_embedding` (diagonal-σ special case; the meet point with `deep_causality_do_calculus`)
      + `quantum.cyclic_support` (non-DAG hypergraph; leans on `core.context_graph.acyclicity_separable`)
- [ ] 5.6 `quantum.verdict.orthomodular` — extend `core.verdict.carriers` with the orthomodular carrier
      (Lean statement + Rust witness for the Phase-4 newtype)
- [ ] 5.7 Add all `quantum.*` THEOREM_MAP rows; extend the CI witness-search scope to
      `deep_causality_quantum`; `lake build` + bare-`lean` exit 0 + traceability green + `bazel test //...`

## 6. Phase 6 — Examples + docs

- [ ] 6.1 A simulated-CJ QCM example exercising the freeze commutativity check (a valid commuting model
      vs. a rejected non-commuting one)
- [ ] 6.2 Re-point / refresh the migrated quantum examples; confirm all `examples/quantum_examples/*`
      build and run
- [ ] 6.3 `deep_causality_quantum/LEAN_QUANTUM.md` (verification-status note mirroring the other
      `LEAN_*.md`), stating the verifiable/emergent split and the conditional partial-trace scope
- [ ] 6.4 Final `bazel test //...` green; clippy `-D warnings`; traceability green; prepare the final
      commit message
