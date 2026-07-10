## Why

The causaloid formalization roadmap
(`openspec/notes/archive/causal-algebra/causaloid-formalization-roadmap.md`) closed Stages 1–5 on
2026-07-10 and names `deep_causality_quantum` (Goal C) as a downstream crate that inherits the
proven core — the causal monad, the catamorphism keystone, the `⊕` choice generator, and the
Verdict closure — and reconstructs the quantum causal model (QCM) of Lorenz 2022 and Lorenz &
Barrett 2021 on that foundation (`openspec/notes/quantum/QCM-on-EPP.md`, roadmap §7). The
carrier-sense corollary — a physical QPU call is a monadic effect
(`openspec/notes/quantum/quantum-epp.md`) — is a separate, emergent-modality concern that shares the
same crate.

An evaluation of that program against the pinned Lean/Mathlib toolchain and the current code found
that the value is real but **the effort is front-loaded almost entirely into blockers**, and one
headline theorem is **false as stated**:

- **`quantum.partial_trace_preservation` is not true unconditionally.** Concrete counterexample:
  `X = σ_x ⊗ |0⟩⟨0| + σ_z ⊗ |1⟩⟨1|` and `Y = σ_x ⊗ |0⟩⟨0| − σ_z ⊗ |1⟩⟨1|` commute (`[X,Y]=0`), yet
  their partial traces do not (`[Tr₂X, Tr₂Y] = −4i·σ_y ≠ 0`) — partial trace is positive-linear but
  not an algebra homomorphism. The roadmap already anticipated this; the proof obligation must be
  stated **conditionally** (boundary-only / single-node interface), never unconditionally.
- **The pinned Mathlib (v4.15.0) has no partial trace and no Choi–Jamiołkowski / channel layer.**
  Partial trace is the load-bearing operation of every quantum theorem in scope. That foundation is
  net-new and gates all `quantum.*` proofs.
- **General direct-sum causal faithfulness is blocked upstream** — it is an open hypothesis in
  Lorenz & Barrett 2021 and additionally needs finite-dimensional C\*-algebra structure theory
  (Artin–Wedderburn) that Mathlib lacks. Crate faithfulness claims must be scoped to the proven
  classes.
- **The Rust operator/channel layer is absent** (no density matrix, CJ operator, partial trace,
  CPTP/Kraus). `HilbertState` is a pure-state ket only.

The carrier is strong (`Complex`/`Quaternion`/`Float106`, `HilbertState`, `CausalTensor<Complex<R>>`
with SVD/QR, five working quantum examples, quantum gate kernels), so once the operator foundation
and the blockers are resolved the theorem work is tractable. This change scaffolds the crate,
**resolves every blocker in Phase 0 before any dependent work**, migrates the existing quantum
kernels out of `deep_causality_physics`, builds the operator/channel layer, and lands the
**verifiable** simulated-CJ QCM slice with its Lean proofs. The **emergent** cloud-QPU adapter
(carrier sense) is defined as a seam only; a concrete vendor adapter is out of scope for this change.

## What Changes

- **Phase 0 — resolve blockers and open questions first (gate).** No Phase-1+ task starts until
  Phase 0 lands its decisions: (0a) witness the partial-trace counterexample and re-scope
  `quantum.partial_trace_preservation` to a conditional theorem; (0b) commit to building the
  partial-trace + CJ Lean foundation and scope which `quantum.*` ids are in this change vs deferred;
  (0c) scope faithfulness claims to the proven classes, exclude the general hypothesis; (0d) fix the
  operator representation (`CausalTensor<Complex<R>>` reusing `deep_causality_tensor`); (0e) decide
  the kernel-migration scope and the `klein_gordon` disposition; (0f) fix the verifiable/emergent
  modality boundary and the crate feature layout; (0g) fix the orthomodular `Verdict` newtype design;
  (0h) fix the depth-aware `Float106` commutator tolerance policy; (0i) fix crate policy (workspace
  member, `unsafe_code = "forbid"`, `[lints] workspace = true`, MSRV 1.93.0) and the dependency set.

- **Phase 1 — crate scaffold + kernel migration.** Create `deep_causality_quantum` as a workspace
  member under the repo lint/MSRV policy; **move** the quantum-information kernels from
  `deep_causality_physics/src/kernels/quantum/` (the `QuantumGates`/`QuantumOps` traits, the Haruna
  logical gates, the `Operator`/`Gate` aliases and gate/born/expectation/commutator/fidelity kernels,
  and the `PropagatingEffect` wrappers) into the new crate; `HilbertState` **stays** in
  `deep_causality_multivector` (foundational Clifford alias). Update the dependents
  (`deep_causality_physics` re-exports and the four affected example files).

- **Phase 2 — the operator/channel layer.** On `CausalTensor<Complex<R>>`: density matrix (mixed
  state), the Choi–Jamiołkowski isomorphism, **partial trace** (`Tr_B`), CPTP maps / Kraus
  representation, and positivity/trace checks. This is the net-new foundation everything downstream
  needs.

- **Phase 3 — the verifiable QCM slice.** Operator-valued process (CJ) state on the arity-5 **state**
  channel; the freeze-time **Layer-D commutativity check** (`[ρ_j, ρ_k] = 0` on shared Hilbert
  supports) with the depth-aware `Float106` tolerance; an immutable-context handle for the
  environmental Bell-prep `ρ_A` so the simulated model stays in the verifiable region.

- **Phase 4 — the orthomodular Verdict carrier.** A newtype over a commuting projection family
  implementing `Verdict` (`bottom = 0`, `top = I`, `complement = I − P`, meet/join on ranges),
  extending `core.verdict.carriers` (today Boolean-only in Lean); no blanket operator/process-matrix
  `Verdict` impl (general effects form only a partial effect algebra).

- **Phase 5 — Lean formalization.** The partial-trace + CJ Lean foundation (importing Mathlib's
  Hilbert-space machinery — the project's first such consumer), then `quantum.no_influence`,
  `quantum.markov_commutativity`, `quantum.unitary_factorization`, `quantum.classical_embedding`,
  `quantum.cyclic_support`, and the **conditional** `quantum.partial_trace_preservation` (with the
  refuting counterexample witnessed). Each id follows the established loop: Lean proof + Rust witness
  + `THEOREM_MAP.md` row + `bazel test //...`.

- **Phase 6 — examples + docs.** Re-point the migrated quantum examples; add a simulated-CJ QCM
  example exercising the freeze commutativity check; add `deep_causality_quantum/LEAN_QUANTUM.md`.

- **Deferred (seam only, not built here).** The emergent cloud-QPU adapter (`quantum-epp.md` carrier
  sense) — the crate defines the modality-separated seam; a concrete vendor/network/async adapter is
  a later change. Native distributed CQM and quantum indefinite causal order remain carrier-gated and
  out of scope.

## Impact

- **New crate `deep_causality_quantum`** (workspace member; `unsafe_code = "forbid"`,
  `[lints] workspace = true`, MSRV 1.93.0). Dependencies: `deep_causality_core`, `deep_causality_haft`,
  `deep_causality_algebra`, `deep_causality_num`, `deep_causality_num_complex`,
  `deep_causality_multivector`, `deep_causality_tensor`, and `deep_causality_uncertain` (measurement
  statistics).
- **BREAKING — `deep_causality_physics`:** `kernels::quantum` (`QuantumGates`, `QuantumOps`, the
  Haruna gates, the mechanics kernels, and the `PropagatingEffect` wrappers) moves out. Re-export
  shim vs. call-site updates decided in Phase 0; the `klein_gordon` disposition is a Phase-0 decision.
- **`deep_causality_multivector`:** unchanged — `HilbertState` stays as the ket carrier.
- **Examples:** `quantum_counterfactual`, `ikkt_matrix_model`, and the two `multi_physics_pipeline`
  files re-point to the new crate.
- **Lean:** new `Quantum/` directory (partial trace, CJ, the `quantum.*` theorems) — the first
  DeepCausalityFormal consumer of Mathlib's Hilbert-space/linear-algebra machinery; new
  `THEOREM_MAP.md` rows; CI witness-search scope gains `deep_causality_quantum`.
- **Scope of claims:** the verifiable simulated-CJ QCM path is Lean-proven; the emergent QPU path is
  seam-only and, when later built, test-and-provenance-verified (not Lean). `partial_trace_preservation`
  ships as a conditional theorem with a witnessed counterexample; general faithfulness is out of scope.
- **Constraints preserved:** `unsafe_code = "forbid"`, no `dyn`, no crate-defined macros, clippy
  `-D warnings`, Bazel-registered tests, `bazel test //...` green per phase.
