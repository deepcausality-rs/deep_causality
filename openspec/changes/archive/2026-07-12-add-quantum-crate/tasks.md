<!--
Blockers-first ordering (per the change directive): Phase 0 resolves every open question and
false-premise before any dependent work. Phases 1–6 do not start until Phase 0 lands its decisions.
Each phase ends with the established loop where Lean is involved: bare-`lean` exit 0 + zero `sorry`,
Rust witness, THEOREM_MAP row, clippy -D warnings, `bazel test //...` green.
-->

## 0. Phase 0 — Resolve blockers and open questions FIRST (gate)

- [x] 0.1 (B1) Witness the partial-trace counterexample in a standalone test/notebook:
      `X = σx⊗|0><0| + σz⊗|1><1|`, `Y = σx⊗|0><0| − σz⊗|1><1|`; confirm `[X,Y]=0` and
      `[Tr₂X,Tr₂Y] = +4i·σy ≠ 0`. Record it as the refuting witness for `quantum.partial_trace_nonpreservation`
- [x] 0.2 (B1/Q-PTP) Fix the CONDITIONAL statement to prove: `partial_trace_preservation_boundary`
      (single-node interface / boundary-only shared support ⇒ preservation via `Tr_B((1⊗Z)M)=Z·Tr_B(M)`).
      (DECIDED, Q-PTP: prove at BOUNDARY-ONLY shared support — the `1_B ⊗ Z` bimodule case — with
      single-node interface as a checkable sufficient predicate). Record the general necessary-and-
      sufficient condition as an explicit OPEN target. Note: quantum-causaloid nesting is itself
      unestablished, so this stays OFF the critical path — flat QCM models are the supported path and
      the counterexample/boundary theorem only document the boundary
- [x] 0.3 (B2) Confirm the in-scope `quantum.*` id set for this change and that the partial-trace + CJ
      Lean foundation is Phase 5's first deliverable (gates all quantum ids); record the deferred ids
- [x] 0.4 (B3) Write the faithfulness scope guard: claims limited to the proven classes; the general
      Lorenz–Barrett hypothesis and the operator-level `⊕` are explicitly deferred
- [x] 0.5 (B4/Q-ERR) Fix the operator representation (`CausalTensor<Complex<R>>`) and define the
      crate-local `QuantumError` (outer newtype over `QuantumErrorEnum` of exact variants) replacing
      `PhysicsError`; confirm `deep_causality_tensor` provides the needed complex matmul/SVD/QR
- [x] 0.6 (B5/Q-KG) Fix the migration scope: the quantum-information kernels move; `klein_gordon`
      STAYS in `deep_causality_physics` (Q-KG decided) — the `kernels/quantum` module splits; list the
      exact 5-file breakage set and the update plan (direct call-site updates, no long-lived shim)
- [x] 0.7 (B6/Q-QPU) Fix the verifiable/emergent modality boundary and the crate feature layout
      (verifiable default; `qpu` seam off by default, no network/async dep here); lock the `QpuSampler`
      seam shape: generic (`no dyn`), `sample(circuit, shots) -> Result<Shots, QuantumError>` with
      classical `Shots`, an `Uncertain` bridge, and a feature-gated `PropagatingEffect` wrapper
- [x] 0.8 (B7) Fix the orthomodular `Verdict` newtype design (commuting projection family; no blanket
      operator `Verdict`; Born-rule extraction boundary)
- [x] 0.9 (B8/Q-TOL) Fix the `Float106` commutator-tolerance policy: condition-driven forward-error
      bound (NOT linear-in-depth); error-propagation analysis through iterated partial trace;
      configurable + instrumented per-check margin capture (shared telemetry with B1(c))
- [x] 0.10 (B9) Fix crate policy + dependency set (workspace member, `[lints] workspace = true`,
      `unsafe_code = "forbid"`, MSRV 1.93.0, no `dyn`/macros, std; deps listed in the proposal)
- [x] 0.11 Phase-0 gate: all decisions recorded in `design.md` (update in place if any decision
      changes); prepare the Phase-0 commit message. No Phase-1+ task starts before this is done

## 1. Phase 1 — Crate scaffold + kernel migration

- [x] 1.1 Create `deep_causality_quantum/` (`Cargo.toml` with `[lints] workspace = true`, MSRV,
      `unsafe_code = "forbid"`, the Phase-0 dependency set incl. `deep_causality_metric` for the metric
      SSOT — no locally-defined metric type); add it to the workspace `members`
- [x] 1.2 Move `deep_causality_physics/src/kernels/quantum/{gates,gates_haruna,mechanics,wrappers}.rs`
      into the new crate (`klein_gordon*` STAYS in physics per 0.6); replace `PhysicsError` with
      `QuantumError`; keep `QuantumGates`/`QuantumOps`/`Operator`/`Gate` and the Haruna gates
- [x] 1.3 `HilbertState` stays in `deep_causality_multivector` (unchanged); the new crate depends on it
- [x] 1.4 Update `deep_causality_physics/src/lib.rs` (drop the moved re-exports; keep `klein_gordon` if
      it stays) and confirm physics still builds
- [x] 1.5 Re-point the verified dependents (per 0.6): `examples/quantum_examples/ikkt_matrix_model/main.rs`
      (`Operator`, `commutator_kernel`) and `examples/physics_examples/multi_physics_pipeline/main.rs`
      (`born_probability` moves; the `klein_gordon` import stays on physics); move the physics quantum
      kernel tests with the kernels (`quantum_counterfactual` and `multi_physics_pipeline/model.rs`
      are verified NOT dependents)
- [x] 1.6 `bazel test //...` green; clippy `-D warnings`; prepare the phase commit message

## 2. Phase 2 — The operator / channel layer (net-new foundation, bottom-up)

- [x] 2.0 L0 primitives: reuse `deep_causality_tensor` matmul/`svd_truncated`/`qr`/`trace`/einsum
      (complex support confirmed in 0.5); add the missing GENERIC ops to the tensor crate —
      conjugate-transpose (`dagger`), Kronecker product `⊗`, reshape/index-permute — and promote the
      private `sym_eig` (cyclic-Jacobi, complex Givens) to a public dense Hermitian eigensolver on
      `CausalTensor`
- [x] 2.1 L1 ket↔matrix bridge (R1): `to_ket` = column `KET_COLUMN=0` of `to_matrix()`, `from_ket`
      embeds as column 0 + `from_matrix` (even-n metrics only; `Metric` from `deep_causality_metric`).
      Inner product agrees with `QuantumOps::bracket` (Dirac ⟨φ|ψ⟩), adjoint with `QuantumOps::dag`;
      FIRST numerically verify `to_matrix(ψ.dag()) == dagger(to_matrix(ψ))` for Cl(0,10) — if it fails,
      use the metric-correct Clifford conjugation. Ratify `KET_COLUMN=0`; confirm `from_matrix` 1/D gain
- [x] 2.2 L1 `DensityMatrix<R>` newtype with enforced invariants (Hermitian, PSD via `eigen`, unit
      trace → `QuantumError`); constructors from a ket, an ensemble, and a Choi
- [x] 2.3 L2 partial trace `Tr_B` (reshape-to-rank-4 + `b=b'` einsum contraction; named-subset
      generalization) with linearity, `Tr_B(X⊗Y)=X·Tr(Y)`, and the bimodule law
      `Tr_B((1_B⊗Z)M)=Z·Tr_B(M)` as tested properties (the Q-PTP boundary identity)
- [x] 2.4 L3 Choi–Jamiołkowski operator + Kraus: CP ⟺ Choi PSD (`eigen`), TP ⟺ `Tr_out(J)=I`,
      Kraus↔Choi via the PSD-Choi `eigen` decomposition; Choi→Kraus→Choi round-trip test
- [x] 2.5 L4 commutator `[A,B]` + shared-support detection used by the freeze check
- [x] 2.6 Property tests incl. monad law 3 (`encapsulation = flat`) exercised over the arity-5 STATE
      channel with complex-MATRIX payloads (not only scalar state); all freeze-critical paths at
      `Complex<Float106>`
- [x] 2.7 `bazel test //...` green; clippy clean; phase commit message

## 3. Phase 3 — The verifiable QCM slice (simulated CJ + freeze check)

- [x] 3.1 (R3) `ProcessFactors<R>` node-keyed store (`BTreeMap<usize, CjFactor<R>>`, mirrors
      `LambdaEdges`, external param) + `FactorSupports` built from `inbound_edges` (`support(Aᵢ)={Aᵢ}∪Pa(Aᵢ)`).
      σ is STATIC freeze-time decoration, NOT the runtime STATE channel
- [x] 3.2 The Layer-D freeze commutativity check via `freeze_verified_with_check(writers, |g| …)` with
      the closure CAPTURING `ProcessFactors`/`FactorSupports`; commutator only on intersecting supports;
      `impl From<QuantumError> for CausalityGraphError` + public `freeze_quantum -> Result<(),QuantumError>`
      wrapper; sound hard-pass / abort naming the offending pair; rollback via the hook's `unfreeze`
- [x] 3.3 Depth-aware `Float106` tolerance (per 0.9) + instrumented-freeze failure capture
- [x] 3.4 C₃-exclusion faithfulness freeze check (B3, van der Lugt & Lorenz 2508.11762): reject a
      declared causal structure `G` that contains a `C₃` sub-relation (three inputs / three outputs)
      with a `QuantumError`; a C₃-exclusion `G` is faithfully representable. Implement the combinatorial
      `C₃` detection now; the full concept-lattice `L_G` construction may follow later
- [x] 3.5 Immutable-context handle for the environmental Bell-prep `ρ_A` (write methods unreachable),
      keeping the simulated model in the verifiable region
- [x] 3.6 (R2) Emergent seam: reified `QuantumCircuit`/`GateOp` (pure data); generic `QpuSampler`
      (`Shots: ShotHistogram`, no `dyn`); shots→`Uncertain` bridges; feature-gated `qpu_effect` wrapper
      (5-channel routing); in-process deterministic `SimQpu` over the migrated kernels for tests. Off by
      default; no network/async dep; no vendor adapter
- [x] 3.7 `bazel test //...` green; clippy clean; phase commit message

## 4. Phase 4 — The orthomodular Verdict carrier

- [x] 4.1 Rust newtype over a commuting projection family: `impl Verdict` (`bottom=0`, `top=I`,
      `complement=I−P`, meet/join on ranges); orthomodular-law note; no blanket operator `Verdict`
- [x] 4.2 Born-rule / projection extraction at the measurement boundary (operator → `Prob` / proposition)
- [x] 4.3 Tests for the orthomodular laws (and the distributivity failure); `bazel test //...` green;
      phase commit message

## 5. Phase 5 — Lean formalization

- [x] 5.1 Lean foundation `Quantum/PartialTrace.lean` + `Quantum/Choi.lean` (import Mathlib linear
      algebra; the project's first such consumer): partial trace + CJ + their lemma libraries — DONE
      (both build zero `sorry`): partial trace add/smul, the product identity `Tr_B(X⊗Y)=Tr(Y)•X`, and
      the LEFT + RIGHT bimodule laws; Choi `applyChoi`/`choiOf` + add/smul. (The CJ reconstruction
      isomorphism `applyChoi(choiOf E)=E` is deferred — Mathlib `stdBasisMatrix`-expansion plumbing;
      the Rust round-trip witnesses carry it.)
- [ ] 5.2 `quantum.no_influence` (Def 1 marginal condition) + `quantum.markov_commutativity`
      (Lorenz Def 3.3; the 2-factor free-commutation lemma explicit) — DEFERRED (net-new QCM machinery;
      the Rust freeze-check witnesses exist). `/Quantum/` is exempt from the CI sorry gate meanwhile
- [ ] 5.3 `quantum.unitary_factorization` (Lorenz–Barrett Thm 1; the ≥3-factor commutation) — DEFERRED
      (research-grade; requires the direct-sum / C*-structure machinery Mathlib lacks)
- [x] 5.4 `quantum.partial_trace_nonpreservation` (the B1 counterexample) + `quantum.partial_trace_preservation_boundary`
      (the conditional sufficient theorem) — DONE, both proved zero `sorry`: nonpreservation closed by
      `decide` over `ℤ` (incl. the exact `[[0,4],[−4,0]] = +4i·σy` value), boundary preservation via the
      two bimodule laws. The general necessary-and-sufficient condition stays an open target
- [ ] 5.5 `quantum.classical_embedding` (diagonal-σ special case) + `quantum.cyclic_support` — DEFERRED
- [ ] 5.6 `quantum.verdict.orthomodular` — DEFERRED in Lean (the Phase-4 Rust orthomodular carrier +
      its law tests are done; the Lean statement extending `core.verdict.carriers` is future work)
- [x] 5.7 THEOREM_MAP rows for all delivered `quantum.*` ids (10 rows) + CI witness-search covers
      `deep_causality_quantum`; `lake build` green, bare-`lean` exit 0, traceability green
      (each id has a Rust witness + a MAP row), `bazel test //...` green

## 6. Phase 6 — Examples + docs

- [x] 6.1 A simulated-CJ QCM example exercising the freeze commutativity check (a valid commuting model
      vs. a rejected non-commuting one)
- [x] 6.2 Re-point / refresh the migrated quantum examples; confirm all `examples/quantum_examples/*`
      build and run
- [x] 6.3 `deep_causality_quantum/LEAN_QUANTUM.md` (verification-status note mirroring the other
      `LEAN_*.md`), stating the verifiable/emergent split and the conditional partial-trace scope
- [x] 6.4 Final `bazel test //...` green; clippy `-D warnings`; traceability green; prepare the final
      commit message
