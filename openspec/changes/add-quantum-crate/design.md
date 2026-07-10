# Design — `deep_causality_quantum`

The organizing principle of this change is **blockers first**. The quantum program has a small
amount of tractable theorem work sitting behind a large, front-loaded foundation and one
false-as-stated theorem. Phase 0 exists to convert every unknown into a decision before any
dependent code or proof is written, so no downstream work is built on a wrong premise.

Two quantum senses share this crate but are kept strictly apart by **modality**:

- **Verifiable (semantics-structural, `QCM-on-EPP.md`).** Deterministic simulated Choi–Jamiołkowski
  operators carried as arity-5 monad **state**; the quantum Markov condition recovered as a freeze-
  time **commutativity check**. This is what the Lean proofs attach to. Built in this change.
- **Emergent (carrier, `quantum-epp.md`).** A physical cloud-QPU call as a monadic effect; physical
  Born-rule randomness at the Kleisli/measurement cut. Seam only in this change; a concrete adapter
  (vendor SDK + network/async) is deferred. Not a Lean target by nature.

## Phase 0 — Blockers (each must land a decision + a witness before Phase 1)

### B1. `quantum.partial_trace_preservation` is false unconditionally — witness it, then scope it

Counterexample (finite, 2-qubit, explicit): on `H ⊗ ℂ²`,
`X = σ_x ⊗ |0⟩⟨0| + σ_z ⊗ |1⟩⟨1|`, `Y = σ_x ⊗ |0⟩⟨0| − σ_z ⊗ |1⟩⟨1|`. Both Hermitian,
`[X, Y] = 0` (block-diagonal, commuting blocks), but `Tr₂X = σ_x + σ_z`, `Tr₂Y = σ_x − σ_z` and
`[Tr₂X, Tr₂Y] = +4i·σ_y ≠ 0`. Root cause: partial trace is positive-linear but not an algebra
homomorphism (`Tr_B(XY) ≠ Tr_B(X)·Tr_B(Y)`), so it has no general reason to send commutators to
commutators.

**Witnessed (0.1, 2026-07-10).** Numerically confirmed: `‖[X,Y]‖ = 0` exactly,
`[Tr₂X, Tr₂Y] = [[0, 4], [−4, 0]] = +4i·σ_y` (Frobenius norm `√32`), both operators Hermitian. The
witness run corrected the sign originally recorded here (the hand-check agrees:
`[σ_x+σ_z, σ_x−σ_z] = 2[σ_z, σ_x] = +4i·σ_y`). The permanent in-repo Rust witness lands with task
5.4 (`quantum.partial_trace_nonpreservation`).

**Decision.** The theorem ships in three parts: (a) `quantum.partial_trace_nonpreservation` — the
refuting counterexample, proved and witnessed; (b) `quantum.partial_trace_preservation_boundary` —
the conditional sufficient version proved at **boundary-only shared support** (DECIDED, Q-PTP): the
exterior neighbour acts as `1_B ⊗ Z` on the traced interior — exactly the bimodule-identity
hypothesis `Tr_B((1_B ⊗ Z)·M) = Z·Tr_B(M)` — with the single-node interface as a trivially-checkable
special case (`single_node_interface ⇒ boundary_only_support`); (c) the exact necessary-and-sufficient
condition
for "valid QCM encapsulation" stays an **open target**, discovered empirically via the instrumented
freeze (see B8). No unconditional preservation claim is ever stated.

**Scope note — nesting is not a required feature.** Whether *nesting quantum causaloids* is even a
well-posed operation — let alone correct physics — is itself unestablished. The crate therefore does
**not** depend on quantum-subgraph encapsulation and does **not** claim nesting-transparency: **flat
QCM models are the supported path**, the freeze commutativity check runs on them, and the
counterexample + boundary theorem exist only to *document* the boundary honestly. The full
characterization is an open research question kept **off the critical path**, not a deliverable. This
reduces B1 from "a limitation on a needed feature" to "a documented boundary on a feature the crate
does not promise."

### B2. No partial trace / CJ layer in Mathlib — the foundation is the gating build

The pinned Mathlib v4.15.0 has `Matrix`/`trace`/`IsHermitian`/`PosSemidef`, the Kronecker product
with `mul_kronecker_mul`, and the finite-dim Hermitian spectral theorem, but **no partial trace and
no Choi–Jamiołkowski / channel layer**. Both must be built from `TensorProduct.map` / `LinearMap.trace`
with their lemma libraries (linearity, positivity, `Tr_B(X⊗Y)=X·Tr(Y)`, the bimodule law).

**Decision.** Build the partial-trace + CJ Lean foundation as Phase 5's first deliverable; it gates
every `quantum.*` id. In-scope theorem ids for this change: `no_influence`, `markov_commutativity`,
`unitary_factorization`, `classical_embedding`, `cyclic_support`, and the conditional
`partial_trace_preservation_boundary` + counterexample. Difficulty grades (1 easy … 5 research):
`no_influence` 3, `markov_commutativity` 3, `unitary_factorization` 4, others 2–3; the cost is the
foundation, not the individual proofs. `quantum.verdict.orthomodular` is additionally in scope via
Phase 4 / task 5.6. **Deferred ids (recorded, 0.3):** `quantum.faithfulness_general` (the
routed/direct-sum Lorenz–Barrett hypothesis, open upstream per B3), the operator-level `⊕`
(deferred with it), and the exact necessary-and-sufficient "valid encapsulation" condition of B1(c)
(an open target off the critical path, narrowed empirically via the instrumented freeze — nesting
itself is unestablished).

### B3. General direct-sum faithfulness is open upstream — scope it out, but adopt the decidable traditional-circuit criterion

Causally faithful reification of a *general* unitary via direct-sum/routed structure is an **open
hypothesis** (Lorenz & Barrett 2021, §3) whose general existence **remains unknown as of Aug 2025**
(van der Lugt & Lorenz, arXiv:2508.11762 — PDF in `openspec/notes/quantum/`; the Feb 2026 Lorenz–Tull
arXiv:2602.16612 is about causal *abstraction*, unrelated). It also needs finite-dim C\*-algebra
structure theory (Artin–Wedderburn) absent from Mathlib.

**Decision.** (1) The general direct-sum/routed faithfulness claim stays **out of scope**; the
operator-level `⊕` is deferred with it. (2) The crate operates in the **traditional (non-routed)
circuit** regime — tensor + sequential composition + the `Either`/`⊕` *routing* coproduct
(`haft.arrow_choice.*`), no operator direct sum. For that regime arXiv:2508.11762 gives a **decidable
combinatorial criterion** (Thm 3.2) for exactly when a causal structure `G` admits a faithful unitary
decomposition: the **C₃-exclusion property** — `G` contains no `C₃` sub-relation between three inputs
and three outputs, equivalently the concept lattice `L_G` has ≤ 1 path between each input and output.
**Adopt it as a freeze-time faithfulness check:** a declared hypergraph whose causal structure is
C₃-exclusion is faithfully representable; a `C₃`-containing structure (canonically two commuting
CNOTs, `U₃`) provably has **no** traditional-circuit faithful decomposition and is **rejected at
freeze** with an honest `QuantumError`, never silently mis-represented. The check is purely
combinatorial, so it needs neither the open direct-sum theory nor the missing Mathlib C\*-structure
theorem to run. (Note the resonance: `C₃` is the ≥3-factor overlapping-support commutativity
phenomenon — the same content as the Layer-D Markov check, B8.)

### B4. The Rust operator/channel layer is absent — fix the representation

`HilbertState<R> = CausalMultiVector<Complex<R>>` is a pure-state ket; there is no density matrix,
CJ operator, partial trace, or CPTP/Kraus machinery anywhere in the workspace.

**Decision.** Represent operators/channels as `CausalTensor<Complex<R>>` (reuse
`deep_causality_tensor`'s matmul, `eigen`, `svd_truncated`, `qr`, `trace`, `transpose`, einsum). Build
density matrix, CJ isomorphism, `Tr_B`, CPTP/Kraus, and positivity/trace checks as the crate's operator
layer (Phase 2). It is net-new but **unblocked** — the Rust linear algebra already exists (unlike the
Lean gap in B2).

**Build ladder (bottom-up).** (L0) Generic tensor primitives — reuse matmul/`eigen`/`svd`/`qr`/`trace`/
einsum; **add to `deep_causality_tensor`** the few missing generic ops (conjugate-transpose `dagger`,
Kronecker product `⊗`, reshape/index-permute), keeping quantum semantics in this crate. (L1) A
`DensityMatrix<R>` newtype with enforced invariants (Hermitian, PSD via `eigen`, unit trace → else
`QuantumError`); constructors from a ket, an ensemble, or a Choi. (L2) `Tr_B` via reshape-to-rank-4 +
`b=b'` contraction, property-tested against linearity, `Tr_B(X⊗Y)=X·Tr(Y)`, and the bimodule law
`Tr_B((1_B⊗Z)M)=Z·Tr_B(M)` (the Q-PTP boundary identity). (L3) Choi operator + Kraus, with
**CP ⟺ Choi PSD** and **TP ⟺ `Tr_out(J)=I`**, and Kraus↔Choi via the **`eigen` decomposition of the
PSD Choi** (`K_i = √λ_i · reshape(v_i)`); round-trip tested. (L4) The process operator σ = product of
per-node CJ factors as a static freeze-time `ProcessFactors` store (R3), feeding the Layer-D freeze check.

**Confirmed (0.5, 2026-07-10) — the tensor crate carries the complex load.** `matmul` (bound
`T: Ring + Copy + Default + PartialOrd`) is satisfied by `Complex<R: RealField>` (derived
`Copy/PartialEq/Default`, manual `PartialOrd`, `Ring` via the algebra blanket impl);
`svd_truncated` and `qr` are generic over `ConjugateScalar`, which is implemented for
`Complex<T: RealField>` (so `Complex<Float106>` qualifies); einsum `trace`/`transpose` exist. One
gap: the dense Hermitian eigensolver exists in-tree (`sym_eig`, cyclic-Jacobi with complex Givens,
`deep_causality_tensor/src/types/causal_tensor_network/solve/local.rs`) but is **private** to the
DMRG solver — task 2.0 promotes it to a public dense `CausalTensor` op alongside the missing
`dagger` / Kronecker / reshape-permute generics.

**Design seam — ket ↔ matrix (settle in Phase 0/2).** `HilbertState<R>` is a Clifford minimal-left-
ideal element (a multivector), not a column vector; forming `|ψ⟩⟨ψ|` and applying operators needs a
clean `HilbertState → column-vector CausalTensor` bridge (`to_ket`/`from_ket`, alongside the existing
`to_matrix()`), so the operator layer works in the tensor representation and converts `HilbertState`
only at the boundary. Freeze-critical paths run at `Complex<Float106>`; PSD/eigenvalue tolerances share
the Q-TOL condition-driven budget.

### B5. Kernel-migration scope — decide what moves and where `klein_gordon` lives

`deep_causality_physics/src/kernels/quantum/` holds: `gates.rs` (`QuantumGates`, `QuantumOps`
traits), `gates_haruna.rs` (logical gates), `mechanics.rs` (`Operator`/`Gate` aliases;
`klein_gordon_kernel`, `born_probability_kernel`, `expectation_value_kernel`, `apply_gate_kernel`,
`commutator_kernel`, `fidelity_kernel`, `haruna_*_gate_kernel`), and `wrappers.rs` (the
`PropagatingEffect` wrappers).

**Dependents (verified 0.6, 2026-07-10).** The breakage set is: (1)
`deep_causality_physics/src/lib.rs` — `pub use crate::kernels::quantum::*` splits; (2) the physics
quantum test files `deep_causality_physics/tests/kernels/quantum/{gates,mechanics,wrappers}_tests.rs`
— they move with the kernels (the `klein_gordon` tests stay); (3)
`examples/quantum_examples/ikkt_matrix_model/main.rs` — imports `Operator`, `commutator_kernel`; (4)
`examples/physics_examples/multi_physics_pipeline/main.rs` — imports `born_probability` (moves) and
`klein_gordon` (stays), so the import splits. Verified **not** dependents (contrary to the earlier
estimate): `examples/quantum_examples/quantum_counterfactual` (multivector/core-only imports) and
`examples/physics_examples/multi_physics_pipeline/model.rs` (comment-only mention). No other crate
(incl. `deep_causality_cfd`) references the moving kernels.

**Decision.** Move the quantum-information layer (gates, Haruna gates, the gate/born/expectation/
commutator/fidelity kernels + wrappers, `QuantumGates`/`QuantumOps`, `Operator`/`Gate`) into
`deep_causality_quantum`. **DECIDED (Q-KG):** `klein_gordon` **stays in `deep_causality_physics`** —
it is a relativistic field-theory PDE kernel used by `multi_physics_pipeline`, not quantum-information;
the `kernels/quantum` module splits accordingly. Update call sites directly (no long-lived re-export
shim), since the breakage set is small (5 files).

### B6. Modality boundary + feature layout

**Decision.** Default features build only the verifiable path (simulated CJ + freeze check + Lean-
witnessed kernels). The emergent QPU seam is a trait (e.g. a `QpuSampler` that returns shots as
`Uncertain`/classical bits at the measurement cut) behind an off-by-default `qpu` feature; no
network/async dependency is added in this change. The type system keeps the two modalities distinct
so a model states plainly whether a verdict rested on a checked simulation or physical evidence.

**Recommended seam shape (Q-QPU) — lock now, no adapter.** `QpuSampler` is a **generic trait** used
as a bound `S: QpuSampler` (policy forbids `dyn`), with a method
`sample(circuit, shots) -> Result<Shots, QuantumError>` where `Shots` is a **classical** outcome
histogram — never live amplitudes, which pins the Kleisli/coherence boundary at the type level. A
`to_uncertain` bridge surfaces measurement statistics as `Uncertain<T>`, and a feature-gated wrapper
`qpu_effect(sampler, circuit, shots) -> PropagatingEffect<…>` lifts a call into a causaloid `f`
(value = summary, error = job failure, log = provenance, context = device calibration/topology). The
`circuit` input is a reified gate program (reuse the migrated gate kernels) so both a local-simulator
impl and a future cloud impl satisfy the same trait. Vendor SDK, async transport, and circuit detail
stay open. This is enough for Phase 3's state channel and Phase 6's examples to compile against.

### B7. Orthomodular `Verdict` newtype design

**Decision.** A newtype over a **commuting projection family** implementing `Verdict`: `bottom = 0`,
`top = I`, `complement = I − P` (orthocomplement), meet/join on ranges — an orthomodular lattice
(fails distributivity as `Prob` fails excluded middle). Extends `core.verdict.carriers` (Boolean-only
in Lean today; MV witness-only). Guard: **no** blanket `Verdict` impl for a general
tensor/operator/process-matrix type — general effects `0 ≤ E ≤ I` form only an effect algebra with
*partial* meet/join; verdicts are extracted from operators at the measurement boundary (Born rule →
`Prob`, propositions → the projection lattice), never the operators themselves.

### B8. Depth-aware `Float106` commutator tolerance

The freeze commutativity test decides a causal verdict (commuting = valid model), and error
accumulates through iterated partial traces down encapsulation depth.

**DECIDED (Q-TOL).** The tolerance is **condition-driven, implemented as an incremental forward-error
bound — not linear-in-depth.** A linear-in-depth ε ignores the operators and
misclassifies (the accumulated error is conditioning-dependent, not depth-alone). Track a running
error budget that grows with the norms/conditioning encountered at each partial-trace/product step
(operator norms are already computed; escalate to SVD-based condition numbers only where the norm
bound misclassifies in the instrumented battery), and compare `‖[ρ_j, ρ_k]‖_F` against `C · budget`
with a small safety factor `C` over the `Float106` machine-epsilon base. Keep the policy configurable
and instrumented (record the per-check margin `‖[ρ_j,ρ_k]‖ / ε`) so Q-TOL tuning and the B1(c)
empirical discovery share one telemetry stream.

**Confirmed bound form (0.9, 2026-07-10).** A first-order incremental forward-error budget over the
`Float106` unit roundoff `u`: every operator carries `(‖·‖_F, budget)`. Construction seeds
`budget_A = u·‖A‖_F`; a product `C = A·B` propagates
`budget_C = ‖A‖_F·budget_B + ‖B‖_F·budget_A + γ_n·‖A‖_F·‖B‖_F` (Higham's matmul bound,
`γ_n = n·u/(1−n·u)`); a partial trace `T = Tr_B(M)` propagates `budget_T = budget_M + d_B·u·‖M‖_F`.
The pair test accepts `[ρ_j, ρ_k] = 0` iff
`‖[ρ_j,ρ_k]‖_F ≤ C_safety · (‖ρ_j‖_F·budget_k + ‖ρ_k‖_F·budget_j + 2·γ_n·‖ρ_j‖_F·‖ρ_k‖_F)`, with
`C_safety = 8` by default and configurable. The per-check margin `‖[ρ_j,ρ_k]‖_F / threshold` is
recorded (the shared B1(c) telemetry stream). Escalation path: only where the instrumented battery
shows the norm bound misclassifying, replace the `‖·‖_F` factors at the offending step with
SVD-based `σ_max` / condition estimates — never globally.

### B9. Crate policy + dependency set

**Decision.** `deep_causality_quantum` is a new workspace member with `[lints] workspace = true`,
`unsafe_code = "forbid"`, MSRV `rust-version = 1.93.0`, no `dyn`, no crate-defined macros, std (it
depends on `CausalTensor`). Dependencies: core, haft, algebra, num, num_complex, multivector, tensor,
uncertain, and **`deep_causality_metric`** (the metric-signature SSOT — see cross-cutting constraints). It depends on nothing that would create a cycle (physics does **not** become
a dependency; the migrated kernels have no physics-only deps beyond `PhysicsError`, replaced per Q-ERR).

**DECIDED (Q-ERR).** The migrated wrappers return a crate-local **`QuantumError`**: an outer newtype
struct wrapping a `QuantumErrorEnum` of exact variants — mirroring the repo convention
(`CausalityError(CausalityErrorEnum::…)`). Candidate variants: dimension/shape mismatch, Clifford-metric
mismatch, non-finite value, non-positive or non-unit-trace operator, non-CPTP channel, partial-trace
shape error, and a freeze-commutativity failure that names the offending operator pair. It implements
`core::error::Error + Display`; typed variants are preferred over a `String`-only catch-all wherever a
variant fits.

## Open questions — resolutions and recommendations

**Resolved (decisions folded into B4–B9 above).**

- **Q-KG — RESOLVED.** `klein_gordon` stays in `deep_causality_physics` (B5).
- **Q-ERR — RESOLVED.** Migrated wrappers return the crate-local `QuantumError` newtype over a
  `QuantumErrorEnum` of exact variants (B9).
- **Q-PTP — RESOLVED.** `partial_trace_preservation_boundary` is proved at **boundary-only shared
  support**, single-node interface a checkable special case; general condition stays open (B1).
- **Q-TOL — RESOLVED.** Condition-driven incremental forward-error bound, not linear-in-depth (B8).

- **Q-QPU — RESOLVED.** Generic (`no dyn`) `QpuSampler` returning classical shots (B6, R2), with
  `Shots` an associated type bounded by `ShotHistogram`, the `Uncertain` bridge, and the full-arity
  `CausalEffectPropagationProcess` `qpu_effect` wrapper. Seam only; no adapter built. All Phase-0
  cross-cutting decisions are now made; the residual items are numeric checks / micro-conventions
  inside R1 and R3 (ket-column ratification, `dag`=conj-transpose verification, multi-qubit leg ids).

## Non-goals

- A concrete cloud-QPU adapter (vendor SDK, network/async transport) — seam only.
- General (all-unitary) causal faithfulness — open upstream.
- Native distributed CQM / dagger-compact substrate and quantum indefinite causal order — carrier-
  gated; would require dropping the cartesian copy comonoid, a different substrate.
- Any unconditional `partial_trace_preservation` claim.
- **Quantum-subgraph nesting as a supported semantic operation.** Whether nesting quantum causaloids
  is a well-posed operation (let alone correct physics) is unestablished; the crate supports flat QCM
  models and treats nesting-transparency as an open research question, not a feature.

## Adjacent work (recorded, out of scope for this change)

- **Causal abstraction — Lorenz & Tull, arXiv:2602.16612 (Feb 2026, PDF in `openspec/notes/quantum/`).**
  A categorical account of *causal abstraction* as natural transformations between compositional models
  in a **Markov (copy/discard) category**, covering Do-interventions, counterfactual/exact/distributed
  abstraction, a new component/mechanism-level notion, and abstractions between quantum-circuit models
  and high-level classical causal models (explainable quantum AI). It does **not** touch unitary causal
  faithfulness or the C₃ criterion, so B3 is unaffected. It is a candidate theoretical scaffold for the
  future **`deep_causality_do_calculus`** crate (abstraction + interventions on the crate's
  Kleisli-of-causal-monad, itself a Markov category) and for the **quantum → classical explanation**
  direction that `quantum.classical_embedding` opens. Recorded as an input to those follow-on changes,
  not built in this one.
- **Unitary causal decompositions — van der Lugt & Lorenz, arXiv:2508.11762 (Aug 2025, PDF in
  `openspec/notes/quantum/`).** The C₃-exclusion faithfulness criterion adopted in B3.

## Gap resolutions (ARIZ, Phase 0)

Three implementation-readiness gaps were resolved from first principles (TRIZ/ARIZ); each dissolved a
contradiction using only existing resources. These are Phase-0 decisions.

### R1 — Ket ↔ matrix bridge (operator-layer seam) — separation in SPACE

The "one datum, two representations" contradiction (a pure state must be a Clifford multivector for
native geometric-product gates AND a d-column for density matrices / `CausalTensor` operators) is
**false**: `CausalMultiVector::to_matrix()` is already a bijection onto `Mat(D,ℂ)` for **even** n
(Cl(0,10): D=32, D²=2¹⁰), and a **minimal left ideal ≅ a matrix column**.

- `HilbertState::to_ket(&self) -> Result<CausalTensor<Complex<R>>, _>` = column `KET_COLUMN = 0` (the
  primitive idempotent `E = e₀e₀ᵀ`) of `to_matrix()`; `from_ket(v, metric) -> Result<Self, _>` embeds
  `v` as column 0 then `from_matrix`. Even-n metrics only (else error). Pure reuse of
  `to_matrix`/`from_matrix`; no new numeric substrate. Lives on `HilbertState` in `deep_causality_multivector`.
- `ρ = |ψ⟩⟨ψ| = to_ket(ψ).matmul(dagger(to_ket(ψ)))`; `A|ψ⟩ = A.matmul(to_ket(ψ))` on `CausalTensor`.
- **Correction (repo owner):** the quantum ops ARE implemented — `QuantumGates` / `QuantumOps<R>` for
  `CausalMultiVector<Complex<R>>` in `deep_causality_physics/src/kernels/quantum/gates.rs` (migrating
  into this crate). `QuantumOps::bracket` IS the Dirac inner product `⟨φ|ψ⟩` (the ket law's RHS), and
  `QuantumOps::dag` = `reversion()` + coefficient-conjugation is the adjoint convention. The inner
  product uses `QuantumOps::bracket`, never the unrelated multifield Lie-commutator.
- Open sub-questions: (i) ratify `KET_COLUMN = 0` as the fixed convention; (ii) numerically verify
  `to_matrix(ψ.dag()) == dagger(to_matrix(ψ))` for the Cl(0,10) Brauer–Weyl gammas (anti-Hermitian
  generators) — if it fails, use the metric-correct Clifford conjugation; (iii) confirm `from_matrix`'s
  `1/D` factor gives unit round-trip gain.

### R2 — QpuSampler (emergent seam) — separation by CONDITION + reify-tool-as-DATA

Support a physical-QPU effect without depending on the adapter or losing the verifiable/deterministic
default: off-by-default `qpu` feature (condition) + generic `S: QpuSampler` bound (no `dyn`; impurity
lives in a future out-of-crate impl) + a reified `QuantumCircuit` (inert data).

- `QuantumCircuit { num_qubits, ops: Vec<GateOp>, measure }` — reified gate program over the migrated
  gate alphabet (pure data; `Clone+Debug+PartialEq`; no amplitudes); validating `new` rejects
  out-of-range qubits with `QuantumError`.
- `trait QpuSampler { type Shots: ShotHistogram; type Calibration; fn sample(&self, &QuantumCircuit,
  shots) -> Result<Self::Shots, QuantumError>; fn calibration(&self); }` — generic bound; `Shots` =
  classical histogram (never amplitudes → pins the Kleisli boundary at the type level).
- Shots→`Uncertain` bridges (per-qubit `bernoulli`; observable expectation via `from_samples`);
  `#[cfg(feature="qpu")] qpu_effect(...) -> CausalEffectPropagationProcess<...>` routing shots→value,
  params→state, calibration→context, failure→error, provenance→log. In-process deterministic `SimQpu`
  (migrated kernels + seeded `QmcSampler`) for hardware-free tests. No network/async dep; concrete
  vendor adapter out of scope.
- **DECIDED (Q-QPU):** `Shots` is an associated type bounded by `ShotHistogram` (not a fixed struct),
  with the `Uncertain` bridge and the full-arity `CausalEffectPropagationProcess` wrapper confirmed.

### R3 — Process-operator ↔ freeze — separation in TIME (σ is STATIC decoration, not runtime STATE)

σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)} IS the process's quantum state, yet must not ride the runtime single-writer,
per-branch, never-merged PS channel (it is a whole-graph, cross-branch, checked-once object). Resolved
by separation in time: σ is **static freeze-time decoration**, absent from the runtime flow; its Markov
validity is decided once, at freeze.

**Spec correction:** σ is NOT "carried on the arity-5 STATE channel." It is an external node-keyed
store — the operator analogue of `LambdaEdges` — consulted only at the freeze boundary. PS stays the
model's ordinary runtime state.

- **M1** `ProcessFactors<R> { factors: BTreeMap<usize, CjFactor<R>> }` (node index → `CjFactor<R> =
  CausalTensor<Complex<R>>`), external parameter, mirrors `LambdaEdges`; `get` returns a borrow
  (`CausalTensor` is not `Copy`).
- **M2** `FactorSupports { supports: BTreeMap<usize, BTreeSet<usize>> }` built from `inbound_edges`:
  `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)` — the parent index IS the Hilbert leg id (one system per node;
  multi-qubit nodes via an optional `NodeLegs` map — open).
- **M3** freeze wiring reuses the existing hook:
  `graph.freeze_verified_with_check(writers, |g| quantum_markov_check(g, &factors, &supports).map_err(CausalityGraphError::from))`
  — the closure CAPTURES the external store. Error bridge: `impl From<QuantumError> for
  CausalityGraphError` (orphan-legal; `QuantumError` is crate-local) + a `freeze_quantum(...) ->
  Result<(), QuantumError>` public wrapper (RefCell stash to preserve the structured error on
  rollback). The same store feeds the C₃-exclusion check.
- Open: (i) error-recovery fidelity (variant-match via the RefCell stash vs. Display message);
  (ii) multi-qubit node leg ids; (iii) commutator leg-alignment (Kronecker-with-identity on the union
  support — an operator-layer op); (iv) Q-TOL threading (per-pair, no cross-pair budget — compatible
  with the `Fn` hook).

### Cross-cutting Phase-0 constraints (repo owner)

- **Metric single source of truth.** Any metric SIGNATURE (`Metric` Cl(p,q,r), Minkowski, sign
  conventions) MUST come from `deep_causality_metric` — the zero-dependency SSOT, already re-exported
  through multivector. The quantum crate depends on `deep_causality_metric` and defines no metric type
  of its own. (Distinct from the computed "quantum metric" / QGT tensor, which is not a signature.)
- **README.** `deep_causality_multivector/README.md`'s Quantum Operations section is outdated — it
  imports `QuantumGates`/`QuantumOps` from multivector, but they live in physics and are impl'd for
  `CausalMultiVector` (not `HilbertState`). Fixed as a migration dependent (task 1.5): re-point to
  `deep_causality_quantum` (their post-migration home) and correct the impl target/example.
