<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL: corrections register

**What this is.** Corrections to `proposal.md`, `design.md`, `qcl-design-note.md` and `qcl-gaps.md`
found in an external review on 2026-09-02, recorded before implementation so the spec does not inherit
them. Each entry names where the claim lives, what it says, why it is wrong or incomplete, and the
replacement. Severity follows the gap register: **S1** produces a wrong answer or an unsound claim
with nothing raised; **S2** blocks or misdescribes a designed capability; **S3** is wording or
documentation.

**What this does not touch.** F9 and its `√(d_B)` resolution, D1's code-space criterion, D2's exact
rational phases, the `Gf2Chain` retyping of the Haruna layer, the `dual_representative` scan, the
verdict law, and the mutation-tested closures in the gap register were all reviewed and stand as
written. Nothing below reopens a closed gap.

**The one item the reviewer left unverified has been verified, and it failed.** X-3 took the mapping
from `check_faithfulness` to van der Lugt & Lorenz (arXiv:2508.11762) as the docs stated it. Re-deriving
the criterion from the paper on 2026-09-02 found that the shipped `is_c3_block` tests for a different
relation from the paper's `C₃`, and answers wrongly on both canonical cases. That is X-16 below, and it
is the one entry in this register that changed code.

---

## 1. Corrections to soundness claims

### X-1 — Composition over a shared wire is marginalisation, and §5.3 says it is not — **S1**

**Where.** `qcl-design-note.md` §5.3 ("Answered, and the answer is the favourable one … F9 does not
bite on composition"); `design.md` D9 ("Composition itself is not the obstacle … F9 does not bite
here").

**As written.** Because `choi_compose` is a double contraction "with no partial transpose and no
partial trace", F9 does not apply to composition, and the only open question is whether the
contraction preserves commutation.

**Why it is wrong.** The distinction is representational, not mathematical. The formula

```text
J(F∘E)_{(a,c),(a',c')} = Σ_{b,b'} J(E)_{(a,b),(a',b')} · J(F)_{(b,c),(b',c')}
```

is the link product written in components. The register's own C-2 says the partial transpose is
"the price of writing the contraction as a matrix product on the joint space"; the partial trace is
the sum over `b, b'`. The operation eliminates the B legs. In QCM terms that is exactly
*marginalising node B*: inserting the identity instrument at B and tracing it out. That is the
operation F9 is about, whether or not a function named `partial_trace` is called.

What makes composition safe is not that F9 does not bite. It is D9: the composite is re-checked.

**Correction.**

- §5.3: replace the conclusion with: *"Composition over a shared wire is the marginalisation of the
  shared node. F9 therefore applies to it, and the reason composition is sound in v1 is D9: the
  composite's Markov certificate is re-derived, not inherited. The two-factor fast path is the only
  case where inheritance is licensed, by hermiticity."*
- D9: delete "Composition itself is not the obstacle … F9 does not bite here." Keep the rest.
- Retain the `3.198e-16` verification: it establishes that `choi_compose` computes the right
  *channel*, which is a different claim from preserving the *factorization*, and the text should say
  which of the two it verifies.

**Impact.** Prose only. No code changes; D9's default already does the right thing. The risk being
removed is a future reader skipping the re-check because §5.3 told them composition was safe.

### X-2 — A failed re-check on inherited factors does not mean the composite is non-Markov — **S1**

**Where.** `design.md` D9; `qcl-design-note.md` §5.1 second consequence ("The Markov report does not
survive marginalisation … has to re-run `check_markov`").

**As written.** The default is "re-run `check_markov` on the composite", and a composite "violating
pairwise commutation" is treated as the failure being guarded against.

**Why it is incomplete.** Barrett–Lorenz–Oreshkov's theorem is an *existence* statement: any unitary
circuit with broken wires induces a QCM that is Markov for the induced DAG. Gluing two circuits gives
a circuit, so a composite of two QCM-representable parts *always has* a Markov factorization — for
the induced DAG and the induced factors. It need not be Markov for the naive product of the parts'
factors, which is what re-running `check_markov` on the inherited `ProcessFactors` tests. A failure
there says "this factorization does not certify the composite", not "the composite is non-Markov".
A sound model would be rejected with a report that reads as a physics failure.

**Correction.**

- `CheckReport<R>` for a re-check after composition or marginalisation carries a provenance field:
  `factorization: Inherited | Rederived`. A failure with `Inherited` is reported as
  `CertificateNotInherited`, distinct from `CommutatorNonZero`, and its message says that a Markov
  factorization for the composite may exist under a different factor assignment.
- D9 gains one sentence: *"A failed re-check on inherited factors is a failure of the certificate,
  not of the model; the composite may be Markov under the induced factorization, which v1 does not
  construct."*
- Add to Open Questions: constructing the induced factorization of a composite from the dilations
  of its parts. This is the same open item as D9's operator-algebra question seen from the other
  side, and it should be named so nobody builds `CertificateNotInherited` into a rejection.

**Impact.** One enum variant, one report field, one sentence. Blocks nothing.

### X-3 — `check_faithfulness` is not faithfulness, and cyclic rejection is a scope choice — **S2**

**Where.** `qcl-design-note.md` §2, §3.1, §7.5, §7.6; `proposal.md` (`check_faithfulness` in
`qcl-hypothesis`); `qcl-gaps.md` §1 ("the cyclic fourth is rejected as a `C₃`").

**As written.** The stage is named `check_faithfulness`, described as "C₃-exclusion over derived
reachability", and it rejects the crosstalk example's cyclic hypothesis "before any shots".

**Why it misleads.** In causal-model vocabulary *faithfulness* means the distribution has no
conditional independences beyond those the graph implies. C₃-exclusion, per the docs' reading of
van der Lugt & Lorenz, is a criterion on an input/output influence relation for whether a causally
faithful *unitary decomposition* exists. Those are different properties, and a QCM practitioner
will read the stage name as the first. Separately, rejecting a cyclic structure outright is a
*scope* decision — cyclic QCMs exist (Barrett, Lorenz & Oreshkov, Nat. Commun. 2021) — and the
docs present it as a faithfulness failure, which it is not.

**Correction.**

- Rename the stage `check_decomposable` (or `check_c3_admissible`). Keep `NotFaithfullyRepresentable`
  as the error name only if its doc comment says "faithfully" in the Lorenz–Barrett sense of a
  circuit decomposition and not the Pearl sense.
- §7.6's scope statement gains: *"Cyclic causal structures are out of scope for v1 by decision, not
  because they fail a check. The C₃ criterion is applied to acyclic influence relations; a cyclic
  candidate is rejected at `build()` with `CyclicStructureUnsupported`, before the check runs."*
- §1 of the gap register and the crosstalk example: "rejected as a C₃" → "rejected as cyclic".
- **Verify** the mapping to arXiv:2508.11762 against the paper's own statement of the criterion and
  record the theorem number. The reviewer did not.

**Impact.** A rename, an error variant, a doc check. The crosstalk pipeline in §7.5 changes one
method name.

### X-4 — `intervene` implements one of two QCM interventions and does not say which — **S2**

**Where.** `qcl-design-note.md` §6.1 ("`intervene(do(node ← factor))` is `factors.insert(node, f)`
… Pearl's cut is a factor replacement"); `proposal.md` (`qcl-hypothesis`); `design.md` D3.

**As written.** Intervention is keyed replacement of the node's factor followed by revalidation.

**Why it is incomplete.** In a QCM a node has an input and an output. The factor `ρ_{A|Pa(A)}` is the
*mechanism* delivering A's input from its parents' outputs. Barrett–Lorenz–Oreshkov's canonical
intervention is different: it fixes the *instrument at the node*, i.e. what happens between A's
input and A's output. Replacing the factor is the mechanism-level `do()`, which is the classical
analogue and is legitimate. But the two are distinct operations, `predict` differs under them, and
the calibration and crosstalk examples arguably want the second (a probe *is* an instrument choice).
The docs use one word for both.

**Correction.**

- §6.1: name the operation `intervene_mechanism(do(node ← factor))` and document it as the
  mechanism-level intervention. Add `intervene_instrument(node, instrument)` as a second operation,
  or record explicitly that v1 supplies only the first and that probes are modelled as mechanism
  replacements with the consequence stated.
- D3: `predict` "evolves each forked world under the chosen probe" — say which of the two
  interventions a probe is.

**Impact.** Naming plus one design sentence if v1 keeps one operation; one new function if it
adds the second. Either is small; the point is that the semantics be named before the stage is
written.

### X-16 — `is_c3_block` tests for the 6-cycle, and the paper's `C₃` has seven edges — **S1**

**Where.** `deep_causality_quantum/src/types/qcm/faithfulness.rs` (module docs, `find_c3`,
`is_c3_block`); `tests/types/qcm/faithfulness_tests.rs` (`canonical_c3`); `example-crosstalk-attribution.md`
§7.1 ("H₄ is exactly `K₃,₃` minus a perfect matching … REJECTED, C3"); `qcl-design-note.md` §1 ("the
cyclic fourth is rejected as a `C₃`").

**As written.** The module doc: *"`C₃` is the bipartite 6-cycle `K_{3,3}` minus a perfect matching — a
3×3 induced sub-relation in which every one of the three inputs relates to exactly two of the three
outputs and every output to exactly two of the inputs (canonically two commuting CNOTs, `U₃`)."*
`is_c3_block` returns true iff every row and column degree is exactly two. The test fixture named
`canonical_c3` is that six-edge relation.

**Why it is wrong.** The paper states `C₃` in words at Example 2.12, and the words are unambiguous:
`U₃` is two commuting CNOTs, *"the no-influence relation `A₃ ↛ B₁` holds … we also have `A₁ ↛ B₃`.
There is influence between all other systems."* Two non-edges, seven edges. Input `A₂` reaches every
output and output `B₂` is reached by every input — the proof that `U₃` has no faithful decomposition
turns on exactly those two facts, `A₂ → B₁` and `A₂ → B₃`. The 6-cycle has six edges and every degree
two. They are different relations, and neither is a sub-relation of the other on three systems.

Definition 3.1 says `G` fails C₃-exclusion iff some 3×3 induced block *equals* `C₃` up to the
labelling it quantifies over. So on three inputs and three outputs the paper's `C₃` fails and the
6-cycle passes. The shipped check does the reverse.

**Measured.** Both relations were run through the shipped `check_c3_exclusion`: the paper's `C₃` is
accepted, the 6-cycle is rejected with a `C₃` witness. Then all 512 relations on three inputs and
three outputs were enumerated: 18 are isomorphic to the paper's `C₃` (`3!·3!` labellings over an
automorphism group of order two); the shipped test fires on 6 and disagrees with isomorphism on 24;
the degree test `{2, 2, 3}` on rows and on columns agrees with isomorphism on all 512, and so does
Theorem 4.9(v), the paper's own "algorithmically most straightforward" reformulation.

**Consequence for the crosstalk example.** H₄, the cyclic candidate, was constructed *as* the
6-cycle and reported "REJECTED, C3 … computed against a transcription of the crate's `is_c3_block`".
The transcription was faithful to a wrong implementation. Under Definition 3.1 the 6-cycle satisfies
C₃-exclusion, so a correct check accepts H₄, and nothing in the C₃ criterion screens the cyclic
candidate. That is X-3's point made concrete: the cyclic rejection is a scope decision at `build()`
or it does not happen.

**Correction, applied.**

- `is_c3_block` decides on sorted degree sequences: `[2, 2, 3]` on rows and `[2, 2, 3]` on columns.
  Seven edges force the row degrees to be `{3, 2, 2}` or `{3, 3, 1}`, and only the first leaves the
  two non-edges in distinct rows; likewise the columns. That is isomorphism to `C₃`, exactly.
- The module docs state `C₃` as Example 2.12 states it, cite Definition 3.1, Theorem 3.2 and
  Remark 3.3, and say which sense of "faithful" the error name carries.
- The tests are rewritten against the paper: the two canonical cases, a relabelling, an embedding in
  a 4×4 relation, a seven-edge non-`C₃`, and a brute-force test holding `find_c3` to Theorem 4.9(v)
  over all 512 relations.
- The crosstalk example's §7.1 result is corrected to what the paper gives.

**Impact.** A soundness fix in shipped code, versioned as a patch because no signature changed. The
error variant `NotFaithfullyRepresentable` keeps its name with its doc comment fixed to the
Lorenz–Barrett sense, as X-3 allows.

---

## 2. Corrections to coverage claims

### X-5 — The logical Hadamard has no verification path — **S2**

**Where.** `proposal.md` (`qcl-code-checks`); `design.md` D1, D2 ("Every diagonal Haruna gate");
`qcl-design-note.md` §6.1 (`logical_hadamard` in the shipped table); `qcl-gaps.md` G-07, G-09.

**As written.** The Haruna gate layer emits all of Table 1 including `H̄`; `LogicalBasis` decides
Paulis (G-09); `check_class_invariance` decides diagonal gates over the code space (D1).

**Why it is a gap.** `H̄` is neither a Pauli nor diagonal. It is the one Table 1 gate the pipeline
constructs and never checks. Nothing in the capability list says so.

**Correction, in two parts.**

1. State it. `qcl-code-checks` gains: *"`check_class_invariance` covers the diagonal gates of
   Table 1 (Z̄, S̄, T̄, CZ̄, CS̄†, CC̄Z, C^{m-1}Z̄). `H̄` is emitted but not checked by this stage."*
2. Close it cheaply. `H̄ = S̄(γ)·∏H·S̄(γ̃)·∏H·S̄(γ)` is built from S, CZ and H, all Clifford, so
   `H̄` is a Clifford circuit. Clifford conjugation of a Pauli is a symplectic 𝔽₂ computation on
   `(x, z)` bit vectors — a stabilizer-tableau update — and the crate already has the symplectic
   Pauli type (`LogicalPauli<W>`). A `check_clifford_action` stage propagates each logical Pauli
   generator through the emitted `Vec<GateOp>` and tests, via `LogicalBasis`, that the image of
   `Z̄(γ)` is logically equivalent to `X̄(γ̃)` and vice versa, up to phase. No state vector, no
   register-width limit, exact. This also cross-checks S̄, Z̄ and CZ̄ by an independent route from
   the diagonal check. It cannot cover T̄, CS̄† or CC̄Z, which are non-Clifford; those remain the
   diagonal check's. Between the two stages every Table 1 gate is checked by exactly one exact
   predicate.

**Impact.** One new stage under `qcl-code-checks`, a tableau update over `GateOp`, and the
symplectic action of `H`, `S`, `CZ` on `(x, z)`. Phase tracking is the only subtlety, and G-09
already records that the symplectic form is phase-blind, so the equivalence should be tested up
to phase and the global phase from `logical_hadamard` recorded beside it (see X-11).

### X-6 — `is_logically_trivial`'s normalizer precondition can now be checked — **S2**

**Where.** `qcl-gaps.md` G-09 ("This crate carries no stabilizer group, so that precondition is
stated at the method rather than checked"); `design.md` D1 (`LogicalBasis` gains stabilizer
generators).

**As written.** G-09 states, correctly, that the B.1 criterion decides triviality only for operators
that preserve the code space, and could not check that precondition because no stabilizer group was
carried. D1 then adds the stabilizer generators to `LogicalBasis` for the diagonal check.

**Why it is now a gap.** Once D1 lands, the precondition is checkable for the Pauli predicate too:
a Pauli `(x, z)` is in the normalizer iff it commutes with every stabilizer generator, which is
`⟨x, s_z⟩ = 0` for every Z-generator and `⟨z, s_x⟩ = 0` for every X-generator — two more loops of
`Gf2Chain::inner`. Leaving it "stated at the method" after the data is present would be a silent
wrong answer on an operator outside the normalizer.

**Correction.** `is_logically_trivial` returns `Err(NotInNormalizer { witness })` when the check
fails, using the generators D1 supplies. Update G-09's limitation paragraph to say the precondition
is checked as of the D1 constructor and stated only for a `LogicalBasis` built by `from_complex`
without generators — or remove `from_complex` from the code-check path entirely.

**Impact.** Two loops, one error variant, one test on a Pauli that anticommutes with a stabilizer.

### X-7 — No dynamical check exercises the geometric-QEC gates — **S3**

**Where.** `qcl-design-note.md` §7.5 (`square_torus(4)` → `[[32,2,4]]`); `qcl-gaps.md` G-06
(`SimQpu::sample` caps at 24 qubits; 18 identity tests).

**As written.** Two facts stated separately.

**Why it matters together.** The geometric-QEC consumer's subject is 32 qubits and the simulator
stops at 24, so no gate the consumer emits is ever simulated. The only dynamical evidence for any
Haruna gate is G-06's 18 identity tests on small registers. That is fine given D1 and X-5 make the
checks combinatorial and exact — but the docs should say that the code path is verified
algebraically and not dynamically, so nobody adds "tested on the simulator" to a claim about it.

**Correction.** §7.5 geometric-QEC block gains one line: *"Verified by exact 𝔽₂ predicates; not
simulated. `SimQpu` caps below this code's width."* Optionally add a `square_torus(3)` ([[18,2,3]])
variant to the example so at least one code in the family is also run through the simulator.

---

## 3. Corrections to design text

### X-8 — D10: a phase has no order, not no metric — **S3**

**Where.** `design.md` D10 ("A phase has no order … A margin presupposes a metric on failure, and
this quantity has none").

**Why it is wrong.** ℝ/ℤ has a natural metric (circular distance). What it lacks is an *order*,
and what makes a margin meaningless here is that the decided question — integrality of an exact
rational — is discrete. The conclusion (witness, not margin) is correct; the premise is not.

**Correction.** *"A phase has no order, and the question decided is integrality of an exact
rational, which is discrete. A margin presupposes an ordered, graded failure, and this quantity
has neither."* Delete "metric".

### X-9 — D2's scope wording — **S3**

**Where.** `design.md` D2 ("Every diagonal Haruna gate is `exp(2πi·Q(n)/M)`"); `proposal.md`
("the Haruna gate layer").

**Why it misleads.** D2 is correct as stated, but read beside "the Haruna gate layer" it implies the
whole layer is covered by the rational check. It is not (X-5). Say "the diagonal gates of Table 1"
wherever the check's coverage is described, and reserve "the Haruna gate layer" for the emitter.

### X-10 — §7.5 crosstalk comment contradicts G-15 — **S3**

**Where.** `qcl-design-note.md` §7.5, crosstalk pipeline, `.adjudicate()` annotated
"non-commuting projections fold to Ambiguous, §4".

**Why it is stale.** G-15 narrowed rule 2 to projection-valued verdicts. The crosstalk worlds gate on
`Spec::uncorrelated()`, a real-valued spec, so their verdicts are Boolean and `Ambiguous` cannot
arise from the fold. Either a projection-valued verdict enters the crosstalk pipeline somewhere the
example does not show, or the comment predates G-15.

**Correction.** Change the annotation to "Boolean fold; §4 rule 2 does not apply", or show where a
projection-valued verdict enters. If the intent was to demonstrate `Ambiguous`, the demonstration
belongs on a pipeline that folds `Projection<R, D>` values before `observe` reduces them.

### X-11 — The dropped global phase of `H̄` needs a home — **S3**

**Where.** `qcl-gaps.md` G-07 ("`logical_hadamard` returns `(Vec<GateOp>, Complex<R>)` … The causal
wrapper drops it and says so").

**Why it matters.** The register itself notes the phase becomes relative and observable once `H̄` is
used as a controlled operation. If X-5's Clifford check is built, it needs the phase to test
equivalence exactly rather than up to phase. A wrapper that discards it forecloses that.

**Correction.** Carry the phase on the program: `QuantumCircuit` (or a thin `LogicalProgram` over
`Vec<GateOp>`) gains an optional `global_phase: Option<Complex<R>>`. The wrapper populates it
instead of dropping it. Cost: one field.

### X-12 — D7 should state its supported `n` — **S3**

**Where.** `design.md` D7; `qcl-design-note.md` §10.3 ("the cliff is in n and arrives around
n = 7 or 8").

**Why it is incomplete.** The exponent is correctly located, but the design leaves the bound as a
benchmark observation. `2^C(n,2)` is `2^15` at n=6, `2^28` at n=8, `2^45` at n=10. A caller with
ten hypotheses gets a hang, not an error, and §3.2's own principle is that cost is reported before
it becomes a hang.

**Correction.** `MinCostCover` carries `max_hypotheses` (default 7) and `design` returns
`Err(HypothesisCountExceeded { n, pairs })` above it, with a doc pointer to the heuristic that a
later version would supply. The cap is a decision, in the sense §6.5 uses for `IntType`.

### X-13 — `support_pairs` / `support_triples` allocate eagerly, and qLDPC is the stated target — **S3**

**Where.** `qcl-gaps.md` G-05 ("Correct, and the wrong shape for a large support"); G-02 (the 𝔽₂
rank fix exists *for* qLDPC codes).

**Why it matters together.** T̄ on a weight-`w` representative materialises `C(w,2)` pairs and
`C(w,3)` triples before emitting anything. On toric codes `w` is the lattice extent and this is
nothing; on the qLDPC family G-02 was fixed for, representatives can have weight in the tens to
hundreds and `C(w,3)` is `10⁵`–`10⁶` tuples per gate. The register records the shape as wrong but
does not connect it to the code family the rest of the work targets.

**Correction.** Either make the two iterators lazy, or have `logical_t` / `logical_multi_cz`
report the tuple count in their `CheckReport` cost field and error above a configurable cap — the
same pattern D1 uses for the code-space enumeration. Record in G-05 that the eager shape is
acceptable for the toric fixtures and a known cost on qLDPC.

### X-14 — The vacuous pass on a chain factorization is correct physics — **S3**

**Where.** `qcl-design-note.md` §3.2 ("A gate that examined nothing has not agreed with you").

**Why it needs a qualifier.** The obligation to report zero pairs is right. But zero pairs on a
chain `A → B → C` is not a defect: Barrett–Lorenz–Oreshkov's commutation condition is non-trivial
only where parental sets overlap, and a chain has none. A reader of §3.2 will treat every zero
count as suspicious.

**Correction.** Add: *"Zero tested pairs is the correct result for a factorization whose parental
sets never overlap, such as a chain; the count exists so that a reader can tell that case from a
factorization that was never examined."*

---

## 4. Correction to what QCL claims to be

### X-15 — "One language" is one builder and one report shape, not one semantics — **S2**

**Where.** `proposal.md` Why ("Express all four consumers through one builder and one stage
vocabulary"); `qcl-design-note.md` §3 ("What makes it one language: every decision is a margin").

**As written.** Correct, and the goals list is careful to claim shared *form*, not shared
*computation*.

**Why it needs saying anyway.** The `qcm` layer (factorizations, Markov, intervention) and the
`qcode` layer (chain complex, logical basis, class invariance) sit side by side under `QclBuilder`
with no object connecting them. There is no representation of the physical→logical map as a thing
the pipeline can query, and no stage that asks whether a Markov factorization of a physical circuit
induces one on the logical level. Anyone reading "a Quantum Causal Language for QCM and Haruna
gates" will assume that bridge exists.

**Correction.** Add to Non-Goals: *"Relating the `qcm` and `qcode` subjects. QCL v1 does not
represent the code as an abstraction from a physical model to a logical one, and no stage reasons
causally about encoded computation. The two subjects share the builder and the decision form and
nothing else."* Record `qcl-abstraction` as a named future capability, with the note that
`LogicalBasis`'s stabilizer generators (D1) are the raw material for the abstraction map and that
the map itself is not yet a type.

**Impact.** Non-goals text. Its purpose is to keep the claim honest, not to add work.

---

## 5. Summary table

| # | Sev | Where | One line |
|---|---|---|---|
| X-1 | S1 | note §5.3, D9 | Composition over a wire *is* marginalisation; D9's re-check is what makes it sound, not "F9 does not bite" |
| X-2 | S1 | D9, note §5.1 | Failed re-check on inherited factors ≠ non-Markov composite; report `CertificateNotInherited` |
| X-3 | S2 | note §2–§7.6, proposal | `check_faithfulness` → `check_decomposable`; cyclic rejection is scope, not a check; verify the 2508.11762 mapping |
| X-4 | S2 | note §6.1, D3 | Name mechanism- vs instrument-level intervention; say which `predict` uses |
| X-5 | S2 | proposal, D1/D2, G-07/G-09 | `H̄` is unchecked; add a Clifford-conjugation check on the symplectic side |
| X-6 | S2 | G-09, D1 | Check the normalizer precondition now that stabilizers are carried |
| X-7 | S3 | note §7.5, G-06 | Geometric-QEC gates are verified algebraically, never simulated; say so |
| X-8 | S3 | D10 | "no order", not "no metric" |
| X-9 | S3 | D2, proposal | "diagonal gates of Table 1" where coverage is meant |
| X-10 | S3 | note §7.5 | Crosstalk `Ambiguous` comment contradicts G-15 |
| X-11 | S3 | G-07 | Carry `H̄`'s global phase on the program instead of dropping it |
| X-12 | S3 | D7, note §10.3 | Cap `n` and error above it |
| X-13 | S3 | G-05, G-02 | Eager `C(w,3)` allocation meets the qLDPC target; cap or make lazy |
| X-14 | S3 | note §3.2 | Zero pairs on a chain is correct; say so |
| X-15 | S2 | proposal, note §3 | "One language" is shared form; add the missing bridge as a Non-Goal and a named future capability |
| X-16 | S1 | `faithfulness.rs`, its tests, crosstalk §7.1 | `is_c3_block` tested for the 6-cycle; the paper's `C₃` has seven edges; fixed, and H₄ is not screened by C₃ |

**Sequencing against §9.** X-1, X-2, X-8, X-9, X-10, X-14 and X-15 are prose and can land before
step 1. X-12 belongs with step 5 (`design`). X-4 belongs with step 4 (`intervene`). X-6, X-5 and X-11
belong with `qcl-code-checks`, in that order, because X-5's check uses X-6's normalizer test and
X-11's phase. X-3 is a rename that should happen before any consumer is written against the old
name. X-7 and X-13 are documentation now and code only if the qLDPC example is built.

---

## 6. Disposition, 2026-09-02

Every entry was applied on 2026-09-02. Prose corrections landed in the documents named; corrections
that change code landed as requirements in the `add-qcl` specs and as tasks in its `tasks.md`, in the
groups the sequencing paragraph names, so the implementation inherits them rather than the errors.
Two entries changed code today: X-16, because it is a wrong answer in a shipped check, and the part of
X-3 that its verification forced.

| # | Landed in | How |
|---|---|---|
| X-1 | note §5.3; `design.md` D9; `qcl-hypothesis` | Conclusion replaced; "F9 does not bite" deleted; the `3.198e-16` figure kept and scoped to the channel |
| X-2 | note §5.1; `design.md` D9 + Open Questions; `qcl-hypothesis`; `qcl-decision-form`; tasks 2.6 | `CertificateNotInherited` and the `Inherited \| Rederived` provenance are requirements; the induced-factorization item is an open question |
| X-3 | note §1, §3.1, §7.5, §7.6, §10.3; `proposal.md`; `qcl-hypothesis`, `qcl-pipeline`, `qcl-experiment-design`, `qcl-decision-form`, `qcl-evidence`; crosstalk §7.1; `faithfulness.rs`; tasks 7.4 | Stage renamed `check_decomposable`; cyclic rejected at `build()` as `CyclicStructureUnsupported`; mapping verified — see X-16 — with Definition 3.1, Theorem 3.2 and Theorem 4.9(v) recorded |
| X-4 | note §6.1; `design.md` D3; `qcl-hypothesis`; tasks 5.2 | `intervene_mechanism` named; v1 supplies only the mechanism-level operation and says what a probe is |
| X-5 | `proposal.md`; `design.md` D1, D2; gaps G-07, G-09; `qcl-code-checks`; tasks 1.8 | Coverage stated; `check_clifford_action` is a requirement with the symplectic images pinned |
| X-6 | gaps G-09; `design.md` D1; `qcl-code-checks`; tasks 1.6 | X-stabilizers derived beside the Z-stabilizers; `is_logically_trivial` returns `NotInNormalizer` |
| X-7 | note §7.5; gaps G-06; `qcl-code-checks` | "Verified by exact 𝔽₂ predicates; not simulated" |
| X-8 | `design.md` D10; `qcl-code-checks` | "no order", integrality is discrete; "metric" deleted |
| X-9 | `design.md` D2; `proposal.md`; `qcl-code-checks` | "the diagonal gates of Table 1" where coverage is meant |
| X-10 | note §7.5 | Crosstalk `.adjudicate()` annotated as a Boolean fold |
| X-11 | gaps G-07; `qcl-code-checks`; tasks 1.7 | `global_phase` on the emitted program is a requirement |
| X-12 | note §10.3; `design.md` D7; `qcl-experiment-design`; tasks 6.2 | `max_hypotheses` default 7; `HypothesisCountExceeded { n, pairs }` |
| X-13 | gaps G-02, G-05; `qcl-code-checks`; tasks 1.9 | Eager tuple shape recorded against the qLDPC target; tuple-count cap is a requirement |
| X-14 | note §3.2 | Zero pairs on a chain is correct; the count separates it from "never examined" |
| X-15 | `proposal.md`; `design.md` Non-Goals; note §3 | The `qcm`/`qcode` bridge is a Non-Goal; `qcl-abstraction` named as a future capability |
| X-16 | `faithfulness.rs`; its tests; crosstalk §7.1; note §1; `qcl-hypothesis`; `design.md` D11 | Fixed in code; the spec pins `C₃` to Definition 3.1 with the two canonical cases as scenarios |

