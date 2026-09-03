<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL-2: Road map from the proposed QCL to abstraction-based causal reasoning

**What this is.** A staged plan from QCL as proposed (`proposal.md`, `design.md`,
`qcl-design-note.md`, all 2026-08-31) to a second-generation QCL that represents the relation between
a physical process and its logical description as a first-class object, and reasons across it. The
theoretical basis is Lorenz & Tull, *Causal and Compositional Abstraction* (arXiv:2602.16612, Feb
2026), read in full for this plan. The engineering basis is the shipped `deep_causality_quantum`
substrate and QCL v1 as corrected by `qcl-corrections.md`.

**What it is not.** Not a redesign of QCL v1. Every stage below consumes v1's builder, decision form,
carriers and checks. Nothing in v1's Non-Goals is reopened except the one this plan exists to open:
*relating the `qcm` and `qcode` subjects* (X-15). Decoding, device models and topology ownership
stay out.

**Status of the theory it depends on.** Three things the plan needs are supplied by the abstraction
paper, and three are not. Supplied: the abstraction object (type alignment + naturality), the
structural characterisation of mechanism-level abstraction (Thm. 51), and the *opening* query as the
quantum generalisation of `do`. Not supplied, and recorded here as QCL-2's own contributions rather
than assumptions: a compositional-model representation of general Barrett–Lorenz–Oreshkov QCMs
(the paper covers only the Costa–Shrapnel subclass); an *approximate* notion of naturality with
error propagation under composition (the paper is exact throughout); and any non-strict
quantum-to-quantum abstraction (the paper gives the strict, noiseless case and defers the rest).

---

## 0. The target, stated once

QCL v1 answers two questions separately:

1. Is this factorization of a physical process operator Markov, decomposable, and CPTP? (`qcm`)
2. Does this physical gate act correctly on this code space? (`qcode`)

QCL-2 answers one question that contains both:

> Given a low-level model **L** (a physical circuit with noise), a high-level model **H** (a logical
> circuit, or a classical causal model of syndromes and logical outcomes), and a candidate map
> **(π, τ)** between them, for which queries does τ ∘ ⟦π(Q)⟧_L = ⟦Q⟧_H ∘ τ hold, to what residual,
> and under which class of low-level interventions does it keep holding?

Every capability in §6 is a specialisation of that question. In the paper's vocabulary QCL-2 builds
*downward abstractions* between compositional models in the category **QC** of controlled quantum
instruments, checks their *naturality* approximately, and composes them.

---

## 1. Phase 0 — Prerequisites (v1 corrections)

All of `qcl-corrections.md` lands first. Five of its entries are prerequisites rather than tidy-ups:

| Entry | Why QCL-2 needs it |
|---|---|
| X-1, X-2 | QCL-2 replaces certificate inheritance with abstraction composition. The text must already say composition is marginalisation and that a failed re-check is a certificate failure, or Phase 5 will read as contradicting v1. |
| X-4 | The paper's *opening* query is the mechanism-level intervention. `intervene_mechanism` must be the named operation before Phase 2 reuses it. |
| X-5, X-6 | The Clifford-conjugation check and the normalizer check are the exact predicates Phase 3 regresses against. |
| X-15 | The Non-Goal that Phase 3 removes must exist so that its removal is a visible decision. |

**Exit criterion.** v1 §9 steps 1–6 complete; two consumers running; corrections landed.

---

## 2. Phase 1 — Carry the dilation

**Problem.** The paper's abstractions are defined on *compositional models*: functors from a
structure signature (boxes and wires) to QC. General BLO quantum causal models — process operators
with commuting factors, which is what `ProcessFactors` holds — are not compositional models in that
sense, and the paper says so. Only unitary-circuit models are.

**Resolution.** BLO's own theorem: every QCM is the marginal of a unitary circuit with broken wires
over latent systems. The circuit is the compositional model. The process operator is derived from
it. QCL v1 stores the derived object and discards the source; QCL-2 stores the source.

**Deliverables.**

- `CircuitModel<R>`: a compositional model in QC. Structure signature `S` (boxes: encoders,
  unitaries, channels, instruments, measurements; wires: quantum `H`, classical `X`), semantics
  functor into Choi operators / controlled instruments. This is `QuantumCircuit` plus typed wires plus
  a semantics map; the gate alphabet and `SimQpu` are unchanged.
- `Dilation`: `CircuitModel` → `(ProcessFactors, FactorSupports)`, the marginal over latents with
  broken wires at the declared nodes. This is the BLO construction. `check_markov` then certifies
  what `Dilation` produced, and the certificate is *about* a circuit.
- `.over_circuit(model)` on `QclBuilder`, with `.over_model(graph, factors, supports)` kept for
  callers who have only the marginal. A `Screened<R>` records which it came from.

**Decision D2-1.** A QCM without a dilation can still be validated (v1 behaviour) but cannot enter an
abstraction. The error names the reason: `NoCompositionalModel`.

**Verification.** `Dilation` of a circuit is Markov for the circuit's induced DAG — that is BLO's
theorem and is asserted on every fixture, at Q-TOL. The Lean statement is deferred and says so.

**Exit criterion.** The crosstalk consumer runs `.over_circuit` and reproduces its v1 result.

---

## 3. Phase 2 — The abstraction object and the naturality check

**Deliverables.**

- `TypeAlignment`: for each high-level type `X`, a list `π(X)` of low-level types and a channel
  `τ_X : π(X) → X` in QC. In the paper τ is epic (surjective onto the high level). For a code, `π`
  sends a logical qubit to its block and `τ` is the ideal decoding channel (recovery composed with
  the code-space isometry inverse). For a decoder, `τ` is quantum-to-classical. `TypeAlignment`
  seals both and validates that `τ` is CPTP once at construction (v1's `Channel` does this).
- `QuerySignature`: `Io` (inputs → outputs), `Open(S)` (delete the mechanisms of `S`, make them
  inputs — the paper's quantum generalisation of abstract `do`), `Inc(S₁…Sₙ)` (interchange, defined
  only on *parallelisable* sets — the paper's condition, enforced at construction), and `Observe(O)`.
  `Open(S)` **is** v1's `intervene_mechanism`; QCL-2 renames nothing, it adds the query wrapper.
- `Abstraction<L, H>`: `{ alignment, query_map: π: Q_H → Q_L }`. A *downward* abstraction in the
  paper's terms. Upward abstractions (concrete interventions with values) are derived from it by
  composing with sharp states, per the paper's Prop. 18, and are not stored separately.
- `check_naturality`: for each `Q_H` in the declared signature, compute both sides of
  `τ ∘ ⟦π(Q_H)⟧_L = ⟦Q_H⟧_H ∘ τ` as Choi operators and report a `CheckReport<R>` with the residual,
  the tolerance, the margin and the count of queries examined. **This is the check v1 could not
  express**, and it is a `Check<R>` like every other.

**Decision D2-2 — the norm.** The paper's equality is exact in QC. The operationally correct
distance between two channels is the diamond norm; QCL has Frobenius on Choi operators. Frobenius on
the Choi operator upper-bounds the diamond distance up to a dimension factor and is what the shipped
`CommutatorTolerance` machinery already handles. v1 ships Frobenius with the amplification factor
recorded in the report, as G-16 did for `√(d_B)`; diamond-norm evaluation (an SDP) is a later
optional member of the `Tolerance<R>` family, not a blocker. The report says which norm it used.

**Decision D2-3 — approximate naturality is QCL-2's contribution, and is named as such.** An
`Abstraction` whose squares commute to residual `ε` is an *ε-abstraction*. The paper has no such
notion. Its doc block cites the paper for the exact case and states that the approximate case is
this crate's definition, with the composition law of Phase 5 as its one theorem.

**Exit criterion.** A strict abstraction (noiseless logical → physical, the paper's Example 58)
passes `check_naturality` with residual exactly zero on the `[[18,2,3]]` torus for every diagonal
Table 1 gate.

---

## 4. Phase 3 — Structural precheck, and the code as an abstraction

Two deliverables that together close X-15.

### 4.1 `check_alignment_structure`

Theorem 51 of the paper characterises when a constructive abstraction extends to the *mechanism
level* — when each high-level mechanism is implemented by a specific low-level diagram — purely in
terms of the partition π: whether the low-level ancestors of each high-level block are screened off by
the blocks of its parents (the paper's *simple* / *extra-simple* / *full* conditions, depending on
whether the structure category is Cartesian, Markov or cd). This is a graph computation on the two
DAGs and costs nothing.

It goes in `validate`, before any operator is formed. For a code layout it answers: *can this
assignment of logical qubits to physical blocks support a mechanism-level abstraction at all?* A
layout that fails here fails before a single Choi operator is built, which is v1's §3.2 principle
applied one level up.

### 4.2 `CodeAbstraction`

A CSS code from a chain complex is an `Abstraction<L, H>` with:

- `π`: logical qubit `k` ↦ the physical qubits of its block (for the toric code, all data qubits,
  with the two logical qubits distinguished by τ, not by π; for a block code, the block).
- `τ`: the ideal decoding channel, built from the stabilizer generators D1 already added to
  `LogicalBasis`.
- Query map: a logical gate `Ū` ↦ the physical program `gates_haruna` emits for it; `Open(S̄)` ↦
  `Open(π(S̄))`; `Observe(Ō)` ↦ measurement of the corresponding logical operator.

**The regression that makes this safe.** For a *noiseless* physical model, `check_naturality` on the
strict `CodeAbstraction` must agree exactly with v1's `check_class_invariance` (diagonal gates) and
X-5's Clifford check (`H̄`), on every gate and every fixture. If it does not, one of the three is
wrong, and v1's two are the exact predicates, so the new one is. This test is the bridge between
generations and it is the first thing Phase 3 writes.

**What it unlocks now.** Nothing new *numerically* in the strict case — that is the point of the
regression. What changes is that the code is now an object the pipeline holds, composes, and
perturbs, which is what every later phase needs.

**Exit criterion.** Regression passes; X-15's Non-Goal is removed from `proposal.md` and replaced by
the `qcl-abstraction` capability.

---

## 5. Phase 4 — Fault sets and the fault-tolerance predicate

**The reframing.** Fault tolerance is not a new kind of check. It is `check_naturality` over an
enlarged low-level query signature. A *fault* is an opening at a low-level wire followed by insertion
of an error channel — a comb, in the paper's terms a general (non-`do`) intervention. A *fault set*
`F` is a finite signature of such queries: single-qubit Paulis at every location, or every weight-≤t
combination, or an imported detector-error-model's mechanisms. The FT predicate is:

> For every `f ∈ F`, the naturality square for `(f composed with π(Q_H))` still commutes with the
> unperturbed high-level side, to residual ε_f.

The ones that fail are the faults the code does *not* tolerate, and the report names them.

**Deliverables.**

- `FaultSet`: a query signature over `L`. Constructors: `pauli_weight(t)`, `from_dem(...)`, and
  `declared(&[...])`. Counts are ℕ on `NaturalNumber`; `pauli_weight(t)` on `n` locations is
  `C(n,t)·3^t` queries and is capped like D7, erroring above the cap.
- `check_fault_tolerance(abstraction, fault_set)`: `check_naturality` over the enlarged signature,
  reporting per-fault residuals, the worst, the count, and the *witness* (the fault that broke the
  square) — a witness, not a margin, for the same reason as D10: which fault broke it is the
  information; how badly is secondary.
- **The Haruna filter.** Run `check_fault_tolerance` on each Table 1 gate's emitted program under
  `pauli_weight(1)`. The output is the subset of Haruna's non-FT construction that *is* FT under
  single-fault noise on this code — the first thing the paper's construction has been systematically
  filtered by, and the first result QCL-2 produces that no existing tool does.

**Decision D2-4 — v1's Non-Goal "fault-tolerance claims" is narrowed, not removed.** QCL-2 claims
"the naturality square holds under fault set `F` to residual ε" and nothing stronger. It does not
claim a threshold, a distance, or asymptotic suppression. §8 rule 7 stands: the claim names its
Rust witness and its fault set.

**Exit criterion.** The Haruna filter runs on `[[18,2,3]]` and `[[32,2,4]]` and its output agrees
with the known transversality facts (Z̄, X̄ transversal and FT under weight-1; S̄ with CZ pairs
not, on the toric code) — an external oracle, not this implementation.

---

## 6. Phase 5 — Composition, and the replacement for certificate inheritance

**The law.** Abstractions compose (the paper's Prop. 17). QCL-2 needs the *approximate* version:
if `L → M` is an ε₁-abstraction and `M → H` is an ε₂-abstraction, the composite `L → H` is an
ε-abstraction with `ε ≤ ε₁·‖τ₂‖ + ε₂` (or the Choi-norm analogue, with dimension factors), by the
triangle inequality on the pasted squares. This is the composition theorem QCL-2 owes, it is
elementary, and it goes in Lean as the exact statement (`ε₁ = ε₂ = 0 ⇒ ε = 0`, which is Prop. 17)
with the bound in Rust as G-16 did.

**What it replaces.** D9's problem — whether a Markov certificate survives composition — dissolves.
QCL-2 does not compose *certificates*; it composes *abstractions*, each link checked once, with the
residuals adding under a stated law. X-2's `CertificateNotInherited` is the v1 behaviour; QCL-2
callers who hold abstractions never hit it.

**Deliverables.**

- `Abstraction::compose(self, next) -> Abstraction` with the residual law applied and recorded in
  the report's provenance.
- Three consumers, each an abstraction chain: **concatenated codes** (inner code abstraction ∘ outer
  code abstraction); **code switching** (abstraction into code A, identity on the logical level,
  abstraction out of code B, with the switching gadget as the low-level query); **a distillation
  round** (noisy inputs → a logical magic state, as a quantum-to-quantum non-strict abstraction).
  The third is the case the paper explicitly leaves open; QCL-2 builds it as an example, not a
  theorem.

**Exit criterion.** Composite residuals are bounded by the law on all three consumers, and the
bound is tight on a constructed case.

---

## 7. Phase 6 — The decoder as an abstraction, and logical-level attribution

**The observation.** The paper's semantic category QC contains classical probability (FStoch) as a
subcategory, so quantum-to-classical maps are ordinary morphisms. Syndrome extraction followed by
decoding is a quantum-to-classical abstraction: `L` is the physical circuit, `H` is a *classical
causal model* over detectors and logical observables — which is what a detector error model is —
and `τ` is the decoder. The FT stack already holds all three objects and has never stated that they
form an abstraction.

**Deliverables.**

- `DemModel`: import a Stim detector-error-model as a classical causal model `H` in FStoch (a
  `CausaloidGraph` over detector and observable variables, with error mechanisms as latent parents).
  This is *consumption* of Stim's output; QCL-2 does not decode, per v1's Non-Goals.
- `DecoderAbstraction`: `Abstraction<CircuitModel, DemModel>` with `τ` = the decoder's channel from
  syndromes to logical outcomes. `check_naturality` on it asks: *does the decoder's classical picture
  commute with the physical circuit under openings?* A failure is a fault the DEM does not model —
  correlated errors, leakage, hook errors the DEM's mechanisms omit — and the witness names the
  physical query that exposed it. This is a **decoder-model validation** that is causal, not
  statistical, and it is available to nothing today.
- **Logical attribution query.** Given a logical fault observed at `H`, enumerate the low-level
  queries in the fault set whose squares fail and rank them by residual. Because `L` is a QCM with a
  dilation, entanglement-mediated correlations are represented as such and not misattributed to a
  classical common cause — the one thing a classical causal-discovery tool on the same data cannot
  do.

**Decision D2-5.** `DecoderAbstraction` is the only place QCL-2 touches decoding, and it touches it
as a *black box τ* to be validated, never implemented. If a `Decoder` trait appears, something has
gone wrong (cf. v1 §7.7).

**Exit criterion.** On a surface-code memory experiment, injected correlated two-qubit errors absent
from the DEM are reported as naturality failures with the correct physical location.

---

## 8. What each phase unlocks

| Capability (from the assessment) | Phase | Precondition |
|---|---|---|
| Physical→logical map as a queryable object | 3 | 1, 2 |
| "Does the physical factorization induce a logical one" | 2–3 | 1 |
| FT as a predicate; Haruna gate filter | 4 | 3 |
| Composable FT arguments; end of certificate inheritance | 5 | 3, 4 |
| Concatenation, code switching, distillation as chains | 5 | 5 |
| Decoder-model validation | 6 | 2 |
| Logical-level causal attribution with quantum correlations handled | 6 | 1, 4 |
| Compiler verification w.r.t. causal structure | 3 (strict), 4 (noisy) | 2 |

---

## 9. What QCL-2 must not do

Everything in v1 §8, plus:

1. **Implement a decoder.** `τ` is validated, not built.
2. **Claim exact naturality from a measured residual.** Every abstraction report names its norm, its
   tolerance and its amplification, as G-16 taught.
3. **Claim thresholds or distances.** The FT predicate is "holds under fault set `F`". Asymptotics
   are a different discipline.
4. **Enter an abstraction without a dilation.** A bare process operator validates (v1) and stops.
5. **Invent a compositional-model representation of general BLO QCMs.** QCL-2 carries the circuit
   and cites BLO's theorem. Making the process operator itself a compositional model is upstream
   research the paper names as future work, and §8 rule 7 applies.

---

## 10. Verification obligations

| Statement | Where | Status at plan time |
|---|---|---|
| Dilation of a circuit is Markov for its induced DAG | Rust, at Q-TOL, every fixture | to write; BLO's theorem |
| Exact naturality composes (Prop. 17) | Lean | to write; elementary |
| ε-naturality composition bound | Rust, with the law in the doc | to write; QCL-2's theorem |
| Strict `CodeAbstraction` ≡ `check_class_invariance` ∪ Clifford check | Rust regression, every gate, every fixture | the generation bridge |
| Frobenius-on-Choi bounds diamond distance with stated factor | doc + one test on a constructed pair | standard; cite |
| Thm. 51 conditions are what `check_alignment_structure` computes | Rust, on the paper's Examples 54–55 as fixtures | the paper supplies the examples |

Every check names its witness through `lean/THEOREM_MAP.md`; a check with no proof says so.

---

## 11. Risks

**[The Choi–Frobenius proxy is loose on wide registers]** → Report the amplification. Diamond-norm
SDP is an optional `Tolerance` member for callers who need the tight number; it is not on the
critical path.

**[Fault-set enumeration is the new exponential]** → `C(n,t)·3^t` at weight `t` on `n` locations.
Cap it like D7, default weight 1, and import DEMs for realistic sets rather than enumerating.

**[The dilation is bigger than the process operator]** → It is. A QCM with `m` latent legs of
dimension `d` costs `d^m` more to store than its marginal. The design-time positioning absorbs this:
QCL-2 runs once at freeze time, not per tick. Report the cost; error above a cap.

**[QCL-2's approximate naturality is not in the literature]** → Correct, and the plan says so
(D2-3). Its one theorem is elementary; its definition should be written up as a note alongside the
crate so the claim is reviewable outside the code.

**[τ for a code is not deterministic and not epic in the paper's Markov sense]** → QC is not a
Markov category, and the paper does not require determinism there. `τ` is required to be a CPTP
channel that is surjective onto the logical space; `TypeAlignment` checks the second by rank.

**[`DemModel` import couples QCL-2 to Stim's format]** → Behind a feature, like `qpu`; the
`DecoderAbstraction` type takes any `CausaloidGraph`, and Stim is one constructor.

---

## 12. Sequencing summary

```
Phase 0  v1 complete + corrections                (prerequisite)
Phase 1  CircuitModel, Dilation, .over_circuit    ── unblocks everything
Phase 2  TypeAlignment, QuerySignature,
         Abstraction, check_naturality            ── the object and the check
Phase 3  check_alignment_structure,
         CodeAbstraction, generation regression   ── closes X-15
Phase 4  FaultSet, check_fault_tolerance,
         Haruna filter                            ── first novel result
Phase 5  compose, ε-law, three chain consumers    ── retires certificate inheritance
Phase 6  DemModel, DecoderAbstraction,
         logical attribution                      ── the capability nobody has
```

Phases 1–3 are construction against a published definition and carry no research risk. Phase 4 is
construction plus one external-oracle validation. Phases 5 and 6 each build a case the paper names
as open; both are examples with checks, not theorems, and are labelled as such.

**The failure mode to avoid is the same as v1's**: writing the abstraction layer first and shaping a
code example to justify it. The order above is the reverse — the generation regression in Phase 3 is
what licenses everything after it, and nothing is claimed about a noisy or composite abstraction
until the strict one agrees, exactly, with the two predicates v1 already trusts.

---

## 13. Sources

- Lorenz, R. & Tull, S. (2026). *Causal and Compositional Abstraction.* arXiv:2602.16612 — the
  abstraction object (Defs. 14–16), composition (Prop. 17), concrete-from-abstract (Prop. 18),
  mechanism-level characterisation (Thm. 51), the category QC (Ex. 57), quantum implementation as
  strict component-level abstraction (Ex. 58), opening and interchange queries for quantum models
  (§7.2), and the stated limits: BLO QCMs not compositional models (Ex. 62, §8); non-strict
  quantum-to-quantum abstraction deferred (§7.1).
- Barrett, J., Lorenz, R. & Oreshkov, O. (2019). *Quantum Causal Models.* arXiv:1906.10726 — the
  dilation theorem Phase 1 rests on.
- Haruna, J. (2025). arXiv:2511.15224 — the gate family Phase 4 filters.
- Bombin, H. et al. (2024). *Unifying flavors of fault tolerance with the ZX calculus.* Quantum 8,
  1379; Gidney, C. — Stim and detector error models — the classical high-level model Phase 6
  consumes.
- `qcl-corrections.md` (2026-09-02) — Phase 0.
- `qcl-design-note.md`, `qcl-gaps.md`, `proposal.md`, `design.md` (2026-08-31) — the substrate.
