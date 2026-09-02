<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL: Design Note

**What this is.** The design for a Quantum Causal Language in `deep_causality_quantum`, derived from
four consumers: the three designed examples, and the quantum causal model that already ships in the
crate.

**Status.** Design. Supersedes the sketch in [`qcl-dsl-liftback.md`](qcl-dsl-liftback.md) §4, and
revises the previous revision of this note, which counted three consumers and treated the shipped
crate as a substrate QCL would call rather than as a consumer with opinions.

**What changed in this revision (2026-09-02).** An external review produced
[`qcl-corrections.md`](qcl-corrections.md), sixteen entries, and every one of them is applied here
and in the `add-qcl` change. Two are corrections to soundness claims this note made: composition
over a shared wire *is* marginalisation, so F9 applies to it and D9's re-check is what makes it sound
(§5.3); and a failed re-check on inherited factors is a failure of the certificate rather than of
the model (§5.1). One is a finding the review asked for and did not make: the shipped C₃ check
tested for the bipartite 6-cycle, and the paper's `C₃` has seven edges, so the crosstalk example's
cyclic candidate was "rejected" by a wrong shape and is not screened by C₃ at all (§1, §7.5, §7.6).
The rest are scope, naming and coverage: `check_faithfulness` is `check_decomposable`, because the
paper's "faithful" is Lorenz–Barrett's `G_U = G_C` and not Pearl's; `H̄` is emitted and was checked
by nothing; `intervene` implements the mechanism-level intervention and now says so; and "one
language" is one builder and one decision form, not one semantics.

**What changed in the revision before (2026-08-31).** Every gap in
[`qcl-gaps.md`](qcl-gaps.md) is now closed, all eighteen, and the mathematics crates were
consolidated under `deep_causality_unified_math/`. This revision brings the note back in line with
the tree. **Nothing below is a new design decision; the design held.** What changed is that the
things it described as open are built, one of its open questions is answered, and the mathematics
layer underneath it grew parameters the note was not yet written against:

1. **F8 is closed, and not by the fix this note prescribed.** The Haruna gate layer takes a
   `Gf2Chain` and returns a gate program. There is no gauge-field seam to check, because the gauge
   field was never the computational path (§5, §6.2).
2. **F9's guard is shipped.** `partial_trace_preservation_boundary` is a Rust function, not a
   caveat, and it returns a bound rather than a boolean (§5.1).
3. **§5.3's open question is answered.** Composition over a shared wire invokes no partial trace,
   so the nesting question is narrower than F9 suggested.
4. **§9 step 0 is done.** The scalar bound table was re-derived against shipped signatures, and its
   premise did not survive (§6.4).
5. **The algebra tower is complete over the number sets**, five plus two extensions, which is what
   makes §6.4's table derivable rather than a list.
6. **`fork` versus `either` has a type-level answer**, not a preference: `Either` is the coproduct
   and a counterfactual fork is a product (§6.5).
7. **`Complex<T>` is itself a functor**, so the precision lift on an operator is two `fmap`s rather
   than one. §6.4's claim was exact for a real tensor and incomplete for the carrier QCL uses
   (§6.4, §6.6).
8. **The integers are a parameter now.** `IntType` sits beside `FloatType`, and they are different
   kinds of knob: accuracy against headroom, `epsilon()` against overflow. §6.5's ledger hardcoded
   three widths and no longer does; §7.1 names two parameters rather than one (§3.3, §6.4, §6.5,
   §7.1).

**Two of those are corrections to this note rather than news about the tree.** The precision claim
in §6.4 and the hardcoded counts in §6.5 were wrong when written, not made wrong by later work.

**What the revision before that changed.** The crate was read in full, then 3980 lines and now 4432,
plus [`LEAN_QUANTUM.md`](../../../deep_causality_quantum/LEAN_QUANTUM.md). Three things its
predecessor asserted turned out to be wrong.

1. `freeze_quantum` **is** the validate half, already written, for one class of subject. The two
   checks listed as future work are shipped functions.
2. The crate **does** contribute a verdict carrier. `Projection<R, D>` is an orthomodular lattice
   that fails distributivity, and it imposes a law on how QCL may combine verdicts (§4).
3. `predict` cannot be embed-and-contract, because **partial trace does not preserve commutation**.
   That is proved false in Lean with an explicit counterexample (§5.1).

---

## 1. Four consumers, one of them running

| Consumer | Status | Subject | What it contributed |
|---|---|---|---|
| **The shipped crate** | **4432 lines** | a CJ factorization over a frozen graph | The check-and-margin form (§3); the tolerance family (§3.3); the verdict boundary law (§4); transactional failure (§3.4) |
| `quantum_control_loop` | design | a device plant | The control stage sequence; `Ambiguous` as a required verdict; the experiment designer |
| `crosstalk_attribution` | design | a plant with declared structures | `design` returns a plan, not an experiment; the validate-to-control hand-off |
| `geometric_qec` | design | a chain complex | `validate` takes a code; the two halves are not uniformly applicable |

The three designs were each implemented twice in scratch, once in Rust against shipped APIs and once
in Python from published formulas, agreeing on all 16 checked quantities. The three faults read
`0.980147 / 0.980159 / 0.980150` at one pulse and `0.086460 / 0.961136 / 0.847242` at nine; the three
causal structures all fit `P(e₁)=P(e₂)=0.10, P(e₁,e₂)=0.04` and the cyclic fourth is rejected as
cyclic, at `build()`, by decision; the toric family comes out `[[8,2,2]]` to `[[50,2,5]]` with Betti
`[1,4,6,4,1]` on `T⁴`.

**The cyclic fourth used to be "rejected as a `C₃`", and it is not one.** The crosstalk design built
H₄ as the bipartite 6-cycle and ran it through the crate's check, which rejected it. The check was
testing for the wrong relation: van der Lugt & Lorenz's `C₃` is the causal structure of two
commuting CNOTs, seven edges of nine (Example 2.12), and Definition 3.1 excludes exactly that
relation. The 6-cycle satisfies the property, so a correct check accepts H₄. The check is corrected
(X-16), and the cyclic candidate is screened by scope rather than by criterion (§7.6).

**The problem QCL solves is not correctness. It is that correct code is unreasonably hard to write.**
The crate is the evidence that the hard version is writable, and the source of the laws that keep the
easy version honest.

### 1.1 The modality split is a compile-time guarantee, and QCL inherits it

`LEAN_QUANTUM.md` separates two senses of "quantum" by build configuration:

- **Verifiable**, the default build: deterministic simulated Choi–Jamiołkowski operators carried as a
  freeze-time decoration, the Markov condition recovered as a boundary commutativity check. This is
  what the Lean proofs attach to.
- **Emergent**, behind the `qpu` feature: a physical QPU call as a monadic effect. Its evidence is
  tests and provenance, not proof.

For QCL this decides where `Evidence` lives. Sampled read-outs are the emergent path, so a config
that names a shot budget selects the modality. `validate` is verifiable in both builds; `control`
with real shots is not. The language must not blur that, because the separation is currently
enforced by the compiler and a DSL that took shots unconditionally would erase it.

---

## 2. The shape all four agree on

Every consumer declares a **subject**, names **candidates** about it, runs **checks** that admit or
reject, and then optionally spends **evidence** to discriminate among survivors.

```
config(subject, candidates)  →  validate(checks)  →  Screened  →  control(evidence)  →  report
```

| Consumer | validate | control | Both? |
|---|---|---|---|
| the shipped QCM path | Markov commutativity, C₃-exclusion | — | no |
| `geometric_qec` | class invariance, LDPC weights | — | no |
| `quantum_control_loop` | — (mechanisms declare no structure) | full loop | no |
| `crosstalk_attribution` | Markov, C₃-exclusion | full loop | **yes** |

`crosstalk_attribution` is the keystone: the only consumer that exercises the hand-off, and the only
one whose failure would show that the halves do not compose.

The empty cells are the honest finding. A freeze-time structural check spends no device time.
`geometric_qec` carries no feedback loop; the natural control loop over a code is decoding, which is
a separate track.

---

## 3. What makes it one language: every decision is a margin

The four consumers do not share a computation. They share a **decision form**, and the crate already
shipped its canonical shape.

**And that is all they share.** The `qcm` layer — factorizations, the Markov condition, intervention
— and the `qcode` layer — a chain complex, its logical basis, class invariance — sit side by side
under one builder with no object connecting them. There is no representation of the physical-to-
logical map as a thing the pipeline can query, and no stage asks whether a Markov factorization of a
physical circuit induces one on the logical level. A reader of "a quantum causal language for QCM and
Haruna gates" will assume that bridge exists. It does not, and v1 does not build it: relating the two
subjects is a named non-goal of the `add-qcl` change, recorded as the future capability
`qcl-abstraction`, whose raw material is the stabilizer generators `LogicalBasis` now carries and
whose map is not yet a type.

### 3.1 The form

`CommutatorCheck` records, per pair tested:

```rust
pub struct CommutatorCheck<R: RealField> {
    pub node_j: usize,
    pub node_k: usize,
    pub norm: R,        // ‖[ρ_j, ρ_k]‖_F on the common support
    pub threshold: R,   // the Q-TOL acceptance threshold
    pub margin: R,      // norm / threshold; ≤ 1 accepts
    pub commutes: bool,
}
```

and `QuantumMarkovReport` exposes `worst_margin()` and `tested_pairs()`. That is
`(measured, threshold, margin, verdict)` plus a count of what was examined. Every decision QCL makes
fits it:

| Decision | Measured | Threshold | Source |
|---|---|---|---|
| `check_markov` | `‖[ρ_j, ρ_k]‖_F` | Q-TOL | shipped |
| `check_decomposable` | C₃ blocks found | zero | shipped, as `check_c3_exclusion` |
| `check_cptp` | CP and TP defect | `check_completely_positive`, `check_trace_preserving` | shipped |
| `check_class_invariance` | homology residual | tolerance | `geometric_qec` |
| `check_ldpc_weights` | max row/column weight | declared bound | `geometric_qec` |
| `gate(spec)` | sampled read-out | spec, plus shot noise | `quantum_control_loop` |
| `design` | worst-pair separation | `floor_bits` | `crosstalk_attribution` |
| `adjudicate` | separation at the taken shots | `floor_bits` | both control consumers |

**A boolean is never the return type.** `worst_margin() = 0.87` says "passed, thirteen percent from
the edge"; a `true` says nothing.

### 3.2 The count is not decoration

`quantum_markov_check` skips pairs whose Hilbert supports are disjoint, because disjoint supports
impose no commutativity obligation. A factorization whose factors never overlap therefore passes with
**zero pairs tested**, which is a vacuous pass and must be visible as one. `tested_pairs()` exists
for that, and every check inherits the obligation: report how many items were examined, not only how
many passed. A gate that examined nothing has not agreed with you.

Zero tested pairs is also the *correct* result for a factorization whose parental sets never
overlap, such as a chain `A → B → C`: Barrett–Lorenz–Oreshkov's commutation condition is non-trivial
only where parental sets meet, and a chain has no such place. The count exists so that a reader can
tell that case from a factorization that was never examined, not so that every zero reads as a
defect.

### 3.3 Tolerance is a family, and every member is derived from the scalar

The crate carries four tolerance policies today, each chosen for what it is testing:

| Policy | Value | Where | Why that shape |
|---|---|---|---|
| Q-TOL | `C·(‖ρ_j‖·b_k + ‖ρ_k‖·b_j + 2γ_n‖ρ_j‖‖ρ_k‖)`, `γ_n = nu/(1−nu)` | `CommutatorTolerance` | condition-driven forward error over a product of two operators |
| validation | `√ε`, scaled by operator norm | `Projection::default_tolerance` | a Hermitian-idempotent residual, not a product |
| numerical rank | `~D·ε·‖·‖` | `Projection::range_projector` | keeps genuine range directions; `√ε` would discard them, and the code says so |
| state validation | `DensityMatrix::default_tolerance`, overridable | `DensityMatrix::with_tolerance` | Hermiticity, positivity and trace on one operator |

All four are functions of `R::epsilon()`. Swapping `FloatType` from `f64` to `Float106` tightens
every one of them by roughly fifteen orders of magnitude with no code change.

**The family covers the real side and there is nothing to extend it with on the integer side.**
Every policy above is a tolerance because ℝ has an `epsilon()` to derive one from. ℕ and ℤ have no
analogue: a shot count is either right or overflowed, and overflow is a hard wrongness rather than a
graded error. So the integer half of the pipeline does not get a looser tolerance, it gets **checked
arithmetic** — which is what `NaturalNumber`'s `checked_difference` and `succ` return `Option` for.
A design that reached for a tolerance on a count would be answering the wrong question.

This is the concrete form of the commitment that precision is a parameter (§6.4), and it is stronger
than running an adjudication twice and comparing, because the comparison lives inside the check. **A
tolerance that does not move with the scalar was guessed.**

The design consequence: `Tolerance<R>` is a **family with named members**, not one formula.
`Spec::at_least(ft(0.999))` as written in the previous revision is a naked comparison, which is
friction F5 one layer down. Shot noise is one source of uncertainty in a read-out and floating-point
error is another; the crate handles the second and `ShotBudget` handles the first.

### 3.4 Failure is transactional

`freeze_quantum` runs its checks inside `freeze_verified_with_check`. On failure the graph **rolls
back** to its dynamic state, and the structured `QuantumError` is carried out through a `RefCell`
stash, because the hook can only return a `CausalityGraphError` and the bridge would otherwise
degrade the cause to a `Display` string.

QCL inherits both. A failed `validate` leaves nothing half-frozen, and `NoneAdmissible` carries the
structured error. `QuantumErrorEnum` already names the cases QCL needs to report, including
`CommutatorNonZero(node_j, node_k, detail)`, `NotFaithfullyRepresentable`, `NonCptpChannel` and
`PartialTraceShape`.

---

## 4. The verdict law, which the previous revision got backwards

The previous revision said QCL adds nothing to the verdict lattice because `Verdict` belongs to
`deep_causality_algebra`. The trait does, but the crate contributes a carrier with consequences.

`Projection<R, D>` is a Hermitian idempotent on a fixed `D`-dimensional space, implementing `Verdict`
with the subspace-lattice operations. Its module doc states the two facts QCL must respect:

**It is orthomodular and fails distributivity.** It satisfies the bounded-lattice, orthocomplement
and orthomodular laws, and fails distributivity the way `Prob` fails excluded middle, witnessed by
three projections in general position. `commutes_with` names the family within which the lattice
*is* distributive.

**Verdicts are extracted at the measurement boundary, not carried through operators.** No blanket
`Verdict` impl exists for a general operator or process-matrix type, because general effects
`0 ≤ E ≤ I` form only an effect algebra with *partial* meet and join.

Two rules for QCL follow, and neither is optional:

1. **A verdict enters the pipeline only at `observe`.** Stages upstream of the measurement boundary
   carry operators, not verdicts. A design that lets `gate` or `fork` fold verdicts over operator
   values has left the lattice where the laws hold.
2. **Combining verdicts requires commutation, where the verdicts are projection-valued.** When
   `adjudicate` folds verdicts carried by `Projection<R, D>`, projections that do not commute have
   no distributive joint verdict. `Projection::commutes_with` is the guard there, and a
   non-commuting fold is an `Ambiguous`, not an answer.

   **The qualifier is load-bearing, and it was missing.** Rule 2 first read as though it governed
   every fold. It cannot: rule 1 puts the measurement boundary at `observe`, and the calibration
   pipeline in §7.5 forks *after* it, on `Spec::at_least(ft(0.999))`. A threshold on a real
   quantity is a classical proposition. Those form a Boolean algebra, the distributive law holds
   unconditionally, and no pair of them fails to commute — so the guard has nothing to apply to and
   `Ambiguous` cannot arise. Applying it there would reject folds that are sound.

   **Neither `fork` nor `adjudicate` exists yet.** Searched: the only `fork` in the workspace is
   `deep_causality_cfd`'s state-fork counterfactual, an unrelated thing. So this rule constrains an
   API still to be built rather than describing one that is, and the constraint is: whichever of
   the two kinds of verdict a world carries, the fold must be the one that matches. A stage that
   folds projection-valued verdicts checks commutation. A stage that folds read-outs against a
   real-valued spec does not, and must not.

`quantum.verdict.orthomodular` is listed in `LEAN_QUANTUM.md` as complete in Rust with law tests, and
the Lean statement as future work. QCL builds on the Rust laws and should not claim more.

---

## 5. The friction, as observed

| # | Friction | Status |
|---|---|---|
| F1 | Gates, projectors and Kraus operators hand-packed as flat tensors with manual shapes | open; carriers, §6.2 |
| F2 | Unitary evolution masquerades as a channel through `apply_kraus(&[u], rho)` | open; carriers, §6.2 |
| F3 | The validated type is abandoned to compute, then re-validated | **pattern exists**, §6.3 |
| F4 | Every causal-flow stage costs four lines of turbofished closure ceremony | **pattern exists**, §6.1 |
| F5 | Shot noise absent, so decisions are `min_by` on exact reals | open; `ShotBudget`, and §3.3 for the other half |
| F6 | A declared structure cannot be evaluated; `predict` was a hand-written lookup table | **half built**, §6.1 |
| F7 | The experiment designer does not exist | open; §7.5 |
| F8 | The topology-to-gate seam is unchecked and silently wrong | **closed**, and the seam is gone rather than checked |
| F9 | Marginalisation is not commutation-preserving, and nothing at the call site says so | **closed**; the guard ships, §5.1 |

**F8 closed by removing the seam, not by checking it.** The friction was that a logical operator is a
cochain indexed by edge while the Haruna gate layer wanted a `CausalMultiVector` gauge field, with no
conversion between them, so scratch code packed the cochain's sum into a multivector of unrelated
dimension. That surfaced as a Taylor-series convergence complaint rather than a type error.

Reading Haruna settled it: the gauge-field column of Table 1 is the compact form that makes the
Appendix B invariance proofs tractable, and the **physical-gate column is the computational path**.
`a(γ)` is diagonal with integer eigenvalues, so every gate is a product over `supp(γ)`, its pairs and
its triples. The layer now takes a `Gf2Chain` and returns a `Vec<GateOp>`; there is no multivector,
no Taylor series, and so no seam to get wrong. The remaining four frictions are ergonomics.

### 5.1 F9: partial trace does not preserve commutation

This is the reason `predict` cannot be written the obvious way. The finding stands; what has changed
since it was written is that the guard it asked for now exists.

`LEAN_QUANTUM.md` records that `quantum.partial_trace_preservation` is **false**, refuted in Lean by
an explicit counterexample: operators with `[X, Y] = 0` whose partial traces satisfy
`[Tr_B X, Tr_B Y] = [[0, 4], [−4, 0]] ≠ 0`. Partial trace is positive-linear but not an algebra
homomorphism. What holds is the **conditional** `partial_trace_preservation_boundary`: a boundary
operator of the form `Z ⊗ 1_B` commuting with `M` forces `Z` to commute with `Tr_B(M)`.

The API used not to carry this: `partial_trace` documented its shape errors carefully and said
nothing about preservation, so a `predict` marginalising a validated factorization would produce a
model **whose Markov property validate had certified and marginalisation silently destroyed**. The
answer arrives, and nothing marks it as unsound.

**Both halves are now built.** `partial_trace`'s doc block carries the non-preservation, names the
counterexample and points at the sound path, and that path is a function rather than a caveat.

**And the ruling the conditional theorem needed is a factor, not a yes or a no.** The Lean
hypothesis is exact equality over a `CommRing` with no epsilon, so a floating-point caller cannot
discharge it: checking `‖[Z ⊗ 1_B, M]‖_F < τ` and then invoking the theorem substitutes an
approximate premise into an exact-hypothesis result. The way past it is to stop needing the
hypothesis. `quantum.partial_trace.commutator_transport` states

```text
Tr_B([Z ⊗ 1_B, M]) = [Z, Tr_B(M)]
```

**unconditionally**, and with `‖Tr_B(E)‖_F ≤ √(d_B)·‖E‖_F`, tight at `E = F ⊗ 1_B`, a residual of `ε`
certifies `√(d_B)·ε` in the conclusion. Exactly zero in, exactly zero out, so the original theorem is
the vanishing case. `partial_trace_preservation_boundary` returns that bound in a `BoundaryWarrant`,
and it constructs `Z ⊗ 1_B` from the caller's `Z` so the form holds by construction rather than by
check.

Two consequences, both unchanged as design rules:

- **`intervene` and `predict` may marginalise only across a boundary.** The precondition is checked
  now, not documented, and what it returns is the amplified bound the caller must carry.
- **The Markov report does not survive marginalisation.** A `Screened<R>` whose factorization is
  later traced has to re-run `check_markov` or carry an invalidated report. Carrying the old margins
  forward would be the same class of error. Note the sharper version the bound makes available: a
  margin that survives marginalisation survives it *degraded by* `√(d_B)`, so a report could carry
  the amplification rather than being discarded outright.

  **And a failed re-check on inherited factors is a failure of the certificate, not of the model.**
  Barrett–Lorenz–Oreshkov's representation theorem is an existence statement: a unitary circuit with
  broken wires induces a QCM that is Markov for the induced DAG, with the induced factors. Gluing two
  such circuits gives a circuit, so a composite of two QCM-representable parts always *has* a Markov
  factorization. It need not be Markov for the naive product of the parts' factors, which is what
  re-running `check_markov` on the inherited `ProcessFactors` tests. So that failure says "this
  factorization does not certify the composite", and a report that read it as "the composite is
  non-Markov" would reject a sound model with a message that reads as physics. The re-check carries
  a provenance, `Inherited` or `Rederived`, and a failure on inherited factors is
  `CertificateNotInherited`, a different error from `CommutatorNonZero`. Constructing the induced
  factorization from the parts' dilations is the open item behind this, recorded in the change.

### 5.2 What F9 blocks, and what it does not

An earlier revision derived the flatness constraint from F9. That was too broad. Three things
separate, and only the third is blocked:

| | Blocked? | By what |
|---|---|---|
| **Representation**: a node supported on several legs | no | `FactorSupports::declare(node, legs)` takes any leg list |
| **Derivation**: obtaining those legs from a graph automatically | a helper limit | `from_graph` assumes `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)` |
| **Abstraction**: replacing a sub-model by an effective operator on *fewer* legs | yes | marginalisation, and F9 |

Nesting as **composition** needs no marginalisation. A node whose factor spans the union of its
children's legs is a larger factor, and `declare` expresses it today. Nesting as **abstraction**,
hiding the internal legs behind an effective operator, is what F9 refuses.

The conditional theorem is the useful half. Read as a modelling rule,
`partial_trace_preservation_boundary` says a sub-model may be abstracted **when every external
factor acts as identity on its internal legs**, which is what a clean interface means anyway. So
`Boundary` earns its place twice: it guards `predict` against unsound marginalisation, and it is the
same check that would license sub-model abstraction later.

### 5.3 The channels do not compose alike

The causal monad carries five channels and they have different composition laws. Only one has the
physics problem, which is worth stating because it decides what an abstraction boundary may carry.

| Channel | Composition | Survives abstraction? |
|---|---|---|
| Log | `bind` appends via `LogAppend`; a free monoid, nothing lost | **yes, always** |
| Error | left zero; outcome, state, context and logs preserved verbatim | yes |
| Context | read-only, threaded | yes |
| State | the graph join policy refuses to merge state | no, deliberately |
| Value, carrying operators | composition over a shared wire | **the open question** |

The practical consequence is positive. An abstracted sub-model can surface its internal margins in
the parent's audit trail even when the parent no longer carries its operators, because log
concatenation loses nothing. The provenance half of nesting is free; the verification half is not.

**Reuse is served by expansion, not nesting.** A named template that emits flat nodes with fresh
leg-ids gives the modelling convenience that motivates sub-models, needs no composite Hilbert node,
and produces a model the checker already accepts. `transmon(q3)` expanding to four nodes and four
legs is code generation, not physics. That is the recommended reuse mechanism for v1.

**Answered, and not in the direction this section first took.** This read: whether a composed
channel can inherit a Markov certificate from its parts turns on whether composition over a shared
wire invokes a partial trace at all under this crate's input-major Choi convention. `choi_compose`
ships, and under that convention composition is

```text
J(F∘E)_{(a,c),(a',c')} = Σ_{b,b'} J(E)_{(a,b),(a',b')} · J(F)_{(b,c),(b',c')}
```

a plain double contraction over the shared wire, with no partial transpose and no partial trace
*called*. Verified to a maximum relative Frobenius residual of 3.198e-16 over 500 random CPTP pairs
across ten dimension triples, with every transposed or conjugated variant wrong by O(1). That figure
establishes that `choi_compose` computes the right **channel**. It says nothing about the
**factorization**, and the distinction is the whole point.

**Composition over a shared wire is the marginalisation of the shared node.** The distinction between
"a contraction" and "a partial trace" is representational, not mathematical. The formula above is the
link product written in components: the gap register's C-2 already records that the partial
transpose is the price of writing the contraction as a matrix product on the joint space, and the sum
over `b, b'` is the partial trace. The operation eliminates the B legs. In QCM terms that is inserting
the identity instrument at B and tracing it out, which is the operation F9 is about, whether or not
a function named `partial_trace` runs. So F9 applies to composition, and the reason composition is
sound in v1 is D9 of the change: the composite's Markov certificate is re-derived, not inherited. The
two-factor fast path is the only case where inheritance is licensed, by hermiticity, and the reason
is Lorenz (2022) footnote 11 rather than anything about the contraction.

---

## 6. Carriers

### 6.1 What the crate already supplies

| Piece | Location | What it gives |
|---|---|---|
| `ProcessFactors<R>` | `qcm/process_factors.rs` | one CJ operator per node, keyed by node index |
| `FactorSupports` | `qcm/process_factors.rs` | ascending leg-ids per node, per-leg dimensions, `space_map` |
| `FactorSupports::validate` | `qcm/process_factors.rs` | factor shape against declared support, overflow-checked |
| `embed_on_legs`, `partial_trace` | `qgates/operator_linalg.rs` | lift onto a larger space; trace out (subject to F9) |
| `partial_trace_preservation_boundary` | `qgates/operator_linalg.rs` | the §5.1 guard, returning the `√(d_B)` bound rather than a boolean |
| `choi_compose`, `choi_identity` | `qgates/channel.rs` | channel composition as a plain contraction, and its unit |
| `matrix_commutator`, `frobenius_norm`, `hermiticity_defect` | `qgates/operator_linalg.rs` | the operator metrics the checks compare |
| `choi_from_kraus`, `kraus_from_choi`, `apply_kraus`, `apply_choi` | `qgates/channel.rs` | the CJ round-trip, both directions |
| `check_completely_positive`, `check_trace_preserving` | `qgates/channel.rs` | the CPTP checks `Channel` should run once |
| `CausalStructure::from_graph_reachability` | `qcm/faithfulness.rs` | derive the input/output relation from a frozen graph |
| `FactorSupports::from_graph` | `qcm/process_factors.rs` | derive supports as `{Aᵢ} ∪ Pa(Aᵢ)` |
| `Projection<R, D>` | `verdict/projection.rs` | the orthomodular verdict carrier of §4 |
| `born_projective_probability`, `born_projective_prob` | `verdict/born.rs` | the measurement boundary where verdicts are extracted |
| `logical_z/x/s/t/cz/hadamard`, `logical_multi_cz` | `qgates/gates_haruna.rs` | Table 1's gates, taking a `Gf2Chain` and emitting a `Vec<GateOp>` |
| `LogicalPauli<W>`, `LogicalBasis<W>` | `qcode/` | B.1's logical-equivalence predicate, as inner products over bitsets |
| `GateOp`, `QuantumCircuit` | `qpu/circuit.rs` | the physical-gate alphabet of Table 1, no longer behind the `qpu` feature |
| `qgates/wrappers.rs` | 11 functions | **the F4 pattern, already written** |
| `Gf2Chain<W>` | `deep_causality_homology` | a bit-packed chain carrying its degree: support, pairs, triples, the mod-2 pairing |
| `homology_representatives`, `cohomology_representatives`, `dual_representative` | `deep_causality_homology`, on `ChainComplex` | `H_k` and `H^k` bases over 𝔽₂, and the Poincaré-dual pairing |
| `Cochain<R>` | `deep_causality_topology` | the ring-valued dual, binding values to degree |
| `GaugeField`, `LatticeGaugeField` | `deep_causality_topology`, `types/gauge/` | a genuine gauge connection over a manifold: U(1)/SU(3)/SO(3,1), Wilson loops, plaquettes, topological charge |

Two of these change the plan.

**F4 has a house pattern.** Every kernel in `mechanics.rs` has a wrapper that lifts it into the
monad:

```rust
pub fn born_probability<R>(state: &HilbertState<R>, basis: &HilbertState<R>) -> PropagatingEffect<R> {
    match mechanics::born_probability_kernel(state, basis) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
```

The ceremony is hidden once, at the kernel boundary, and the caller sees a value. QCL's stages follow
the same shape rather than inventing one; F4 is a matter of extending an in-crate convention to the
pipeline stages.

**F6 is half built.** The factor store, the supports, the embedding and the CPTP checks exist. So
`Hypothesis` is a **factorization**, not a new type pairing a structure with a plant modification: a
structural candidate is `{ name, ProcessFactors<R>, FactorSupports }`, and its `CausalStructure` is
*derived* rather than stored beside it. `intervene_mechanism(do(node ← factor))` is
`factors.insert(node, f)` followed by `supports.validate(&factors)`; Pearl's cut is a factor
replacement and the store already supports it by key. What is **not** free is `predict`, for the
reason in §5.1.

**The operation is named for what it is, because a QCM has two interventions and they differ.** A
node has an input and an output. The factor `ρ_{A|Pa(A)}` is the *mechanism* delivering A's input
from its parents' outputs, and replacing it is the mechanism-level `do()`, the classical analogue.
Barrett–Lorenz–Oreshkov's canonical intervention is the other one: it fixes the *instrument* at the
node, what happens between A's input and A's output. `predict` differs under the two, and a probe is
arguably the second, since a probe *is* an instrument choice. v1 supplies only the first, and models
a probe as a mechanism replacement with that consequence stated where `predict` is specified;
`intervene_instrument(node, instrument)` is the name reserved for the second.

### 6.2 What must be added

| Type | Replaces | Contract |
|---|---|---|
| `QubitOperator` | hand-packed 2×2 tensors | named constructors: `pauli_x`, `rotation(axis, angle)`, `phase(θ)` |
| `Channel` | a bare Kraus slice | CPTP checked once at construction, via the shipped checks |
| `QuantumPlant` | state and channel juggling | a sealed validated state that evolves in place |
| `Observable` | ket → `Projection` → Born | a named projector carrying its own read-out |
| `Tolerance<R>` | naked float comparison | the §3.3 family, generalised off the four shipped policies |
| `Check<R>` / `CheckReport<R>` | pass/fail | the §3.1 form, generalised off `CommutatorCheck` |

**Two rows left this table, and for opposite reasons.**

`Boundary` **shipped**, as `partial_trace_preservation_boundary` (§5.1). It was one of the two that
prevented wrong answers rather than verbose code.

`GaugeField` was **withdrawn**, and the distinction matters because a gauge field is not
unavailable. This table proposed a *new* type in this crate, constructible only as
`from_cochain(&complex, &cochain, degree)`, to make the F8 seam checkable. That is not what closed
F8: the seam was removed instead, because Haruna's construction never needed a gauge field to
compute with (§5). Meanwhile `deep_causality_topology` ships a real `GaugeField<G, M, R>` and
`LatticeGaugeField<G, D, M, R, S>` — a connection over a base manifold with a metric and a Lie-algebra
valued tensor, carrying Wilson loops, plaquette and Symanzik actions, and topological charge. That is
a gauge field in the QED/QCD/GR sense, not Haruna's `a(γ)`, and §8's second rule already says QCL
does not own topology. **If QCL ever wants a gauge field it uses that one; it does not define one.**

So everything remaining in this table is ergonomics. Both of the wrong-answer seams are shut.

### 6.3 The seal rule, which closes F3

`EnvironmentalPrep` wraps a validated `DensityMatrix` and exposes **only** read accessors; there is by
construction no method that mutates it, so a model threading `ρ_A` cannot alter the preparation
mid-pass and the result is reproducible.

Stated as a rule for every QCL carrier: **seal the interior and expose the operations.** A carrier
that hands out `&mut` to its validated contents has moved its invariant back into the caller's head,
which is where F3 found it.

### 6.4 Scalars: precision as a parameter, bounds as documentation

Every QCL type is generic over its scalar; a program fixes one alias and the pipeline instantiates at
that precision. See the project's standing position on
[uniform math](../../../website/docs/src/content/docs/concepts/uniform-math.md).

```rust
pub type FloatType = Float106;   // or f64, or f32
```

**The tower under this is now complete over the number sets, and that is what makes the table below
derivable rather than a list.** `deep_causality_algebra` covers **five sets and two extensions**:
ℕ at `CommutativeSemiring`, ℤ at `CommutativeRing` and `EuclideanDomain`, ℚ and ℂ at `Field`, ℝ at
`RealField`, then ℍ at `AssociativeDivisionAlgebra` and 𝕆 at `DivisionAlgebra`. The full trait
hierarchy, the marker laws and the per-type implementation matrix are in
[`README_ALGEBRA_TRAITS.md`](../../../deep_causality_unified_math/deep_causality_algebra/README_ALGEBRA_TRAITS.md).

Three of its facts decide rows below, and none of them is about QCL:

- **The multiplicative column is what discriminates.** Every type in that matrix associates and
  commutes additively; the structure is fixed by what happens under `×`. ℍ associates and does not
  commute, 𝕆 does neither. QCL touches ℝ and ℂ only, so it lives entirely in the commutative part
  and never meets the two extensions, which is why no QCL bound mentions them.
- **The markers are hand-written, never blanket-implemented over `Num` or `Float`.** A marker exists
  to record what the compiler cannot check, so granting it by inference would hand the promise to
  any type meeting the structural bound. A QCL check that claims a law inherits that discipline:
  state it per type, in the crate that owns the type.
- **`Invertible` is what separates a `Field` from a ring that merely owns `/`.** `i64` has `Div` and
  `1 / 5 == 0`. Without the marker the tower would conclude ℤ is a field, and a tolerance derived
  from `R::epsilon()` over a non-field would be meaningless.

| Surface | Operations | Bound as shipped | Why it sits there |
|---|---|---|---|
| Cochains, cup product | `+`, `−`, `×`, `0` | `CommutativeRing + Copy` | No division, ordering or analytic call. The operation's floor, and what ships |
| Operators, CJ factors, gauge field | complex arithmetic | `Complex<R>`, `R: RealField + FromPrimitive` | Fixed by the carrier, not by the operations. Every impl for `Complex<T>` in `deep_causality_num_complex` is written `impl<T: RealField>`, `Zero` included, so `Complex<R>` reaches no algebraic structure at all below it |
| Rotations, gate synthesis | `sin`, `cos`, `sqrt`, `π`, one division | `RealField + FromPrimitive` | `sin`, `cos` and `sqrt` are on `Real`. What forces `RealField` is the `R::one() / n_r` in the Taylor `exp`; `π` arrives through `FromPrimitive::from_f64`, not `Real::pi()` |
| Born read-out, purity | real output against a spec | `RealField + FromPrimitive + Default + Debug` | The operations need only `Real`: `+`, `*`, `−`, `abs`, `sqrt`, `clamp`, no division. The carrier is `Projection<R, D>` over `Complex<R>`, so row 2 pins this row too |
| Tolerances (§3.3) | `ε`, `sqrt`, `+`, `×`, one division | `RealField + FromPrimitive` | What the shipped policies declare, and the division is real |
| Verdicts | orthomodular lattice | `Verdict` on `Projection<R, D>`, `R: RealField + FromPrimitive + Default + Debug` | §4. `FromPrimitive` is forced transitively: `range_projector` calls `eigen_hermitian`, which binds `ConjugateScalar` |
| Shot statistics, separation | `sqrt`, `log2`, ratios | `Real + FromPrimitive` suffices | The one row that is genuinely over-tight. The Bhattacharyya formula contains no ratio, and this surface touches no complex carrier, so the relaxation would compile. It has no shipped signature yet, so it is the row to write down carefully rather than to copy |
| Costs, shot counts, cover search | accumulation, ratios, counting | `Real` for the ledger, `NaturalNumber` for counts | §6.5's `Ledger<R>` carries `device_time`, `cost` and `bits` as `R` and binds accumulation on `Real`. `shots`, `experiments` and `predictions` are counts, so they are ℕ and belong on `NaturalNumber` rather than on a hardcoded width |

The discipline is one line: **bound at the weakest structure that carries the operation.** The cup
product is the worked example: bound on `RealField`, relaxed to `CommutativeRing + Copy`, workspace
compiles and 1471 tests pass unchanged.

**And one line re-derives the rest of the table.** `Real` has two implementor families here: the
`Float` blanket, which covers `f32`, `f64` and `Float106` and which also reaches `RealField`; and
`Dual<T>`, which does not. So `Real` versus `RealField` on any row means exactly one thing: **does
this surface admit dual numbers, and so stay differentiable?** Anything routed through `Complex`
cannot, because the complex carrier requires `RealField` before it offers even `Zero`. That is why
four of the eight rows sit at `RealField` regardless of what their function bodies do, and it is
why relaxing them is not a matter of editing a `where` clause.

Four things this does not license. No bound wider than the operation. No scalar parameter where there
is no scalar. `Real` and `RealField` are not synonyms. And a bound the carrier forces should say so
rather than be read as the operation's floor: rows 2, 4 and 6 are carrier-pinned, and a reader who
takes them for operation floors will look for a relaxation that is not there.

**And the integers are a parameter too, of a different kind.** `deep_causality_num` splits ℤ's
representation into `Integer` over all the primitives, with `SignedInt` and `UnsignedInt` beside it,
and `NaturalNumber` builds on `UnsignedInt` to give ℕ its own vocabulary: `succ`, `pred`, `monus`,
`checked_difference`, `div_rem`, `gcd`, `lcm`. `deep_causality_core` names the alias
`IntType = i64` next to `FloatType = f64`. So the same discipline applies on the integer axis, write
against the bound and name the width once, and **the two knobs are not the same knob**: widening
`FloatType` buys accuracy and its failure mode is rounding, bounded by `epsilon()`; widening
`IntType` buys headroom and its failure mode is overflow, with no analogue of `epsilon()` to bound
it. §3.3's tolerance family therefore has no integer member and should not grow one.

**ℕ's missing `Sub` is a feature here.** ℕ is a `CommutativeSemiring` and stops before `AbelianGroup`,
so `3 - 5` has no value and `NaturalNumber` exposes subtraction only as `checked_difference`,
returning `None`, and `monus`, truncating to zero. A ledger draw-down is exactly that operation: an
overdrawn shot budget should either report the shortfall or clamp, and the algebra supplies both
without a hand-written guard.

`CausalTensor::fmap<A, B>` changes the scalar *type* rather than the values, so a carrier can be
lifted between precisions as an operation. Combined with §3.3, that lets the pipeline answer "is this
verdict precision-limited?" about itself: run at two scalars and note that the tolerances moved with
them. On a complex-valued operator the lift is two `fmap`s rather than one, because the element is
`Complex<R>` and the parameter being lifted is its `R`; see §6.6.

### 6.5 The ledger

Device time is the scarce resource. Context is read-only and cannot accumulate, `EffectLog` is a
record rather than a running total, and Value is the answer rather than the meter, so **State is the
only channel both writable and threaded.**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ledger<R, N> {
    shots: N,          // taken on the device
    experiments: N,    // executed on hardware
    predictions: N,    // model evaluations; tracked, never billed
    device_time: R,    // accumulated, measured
    cost: R,           // what MinCostCover minimises
    bits: R,           // separation achieved so far
}
```

**The count fields take a parameter, and this note previously hardcoded them.** They read `u64`,
`u32` and `u32`, three concrete widths in a design that insists everywhere else on writing against a
bound and naming the width once. They are counts, so they are ℕ, so `N: NaturalNumber` is the bound
and `NumberType` names the width at the same single site `FloatType` is named (§7.1). The correction
matters beyond tidiness for the reason §6.4 gives: an integer's failure mode is overflow, and
overflow has no `epsilon()` to bound it, so the width is a decision rather than a default.

**And the alias is the unsigned one.** An earlier revision wrote `IntType` here. `IntType` is
`i64`, ℤ for ring arithmetic, and cannot instantiate `NaturalNumber`, which is unsigned. The alias
file names the counting one: `NumberType`, "which is what ℕ is for". Found when the shot budget was
built against the bound and the signed alias would not compile.

`NaturalNumber` also supplies the draw-down. ℕ has no `Sub`, and `checked_difference` returning
`None` on an overdrawn budget is the semantics a ledger wants, with `monus` as the clamping variant
where a floor at zero is the right answer.

`Real` is `Copy`, so `Ledger<R, N>` is `Copy`; it holds no `Vec` and no `String`. `Default` is
hand-written from `R::zero()`, because `CausalMonad::pure` requires `State: Default` and deriving
would impose a spurious `R: Default`. Accumulation binds on `Real`; only the ratio methods need
`RealField`, which keeps dual numbers admissible so a cost model stays differentiable.

**Three invariants.** *Increment only at the device boundary*: `observe` is the only stage touching
`shots`, `experiments` and `device_time`, because a fork into three hypothesis worlds runs one
experiment and two predictions. *Forking is QCL's, not core's*, for the reason below. *Do not merge
forked ledgers with ∇*: the monoid typechecks and gives the wrong answer, because at a
counterfactual fork exactly one branch was factual.

**Why `fork` is not `either`, stated as the type rather than as a preference.** `Either<L, R>` lives
in [`deep_causality_haft`](../../../deep_causality_unified_math/deep_causality_haft/src/either/mod.rs)
and is the **coproduct**: a value is `Left` or `Right`, exactly one. It carries the arrow algebra's
choice fragment (`ArrowChoice` in `arrow/choice.rs`) and it is what `CausalFlow`'s `branch`,
`branch_with` and `either` are built on. `CausalFlow::either` consumes the flow, matches the
coproduct, and runs **one** arm with the state moved into it.

A counterfactual fork needs the opposite shape. Three hypothesis worlds are all live at once, each
with its own copy of the ledger, and the whole point of `adjudicate` is that they are compared
afterwards. That is a **product**, and no eliminator of a coproduct will produce one. So `fork` is
built above core by cloning, and the reason is a type-level fact rather than a missing feature: it is
not that core's routing is inconvenient for forking, it is that routing and forking are dual.

`Either` is still the right carrier on the way *out*. An `adjudicate` that returns one surviving
hypothesis against a residual ambiguity is a coproduct, and it should be that type rather than an
ad-hoc enum.

### 6.6 The HKT seam, and what QCL takes from it

That `fmap` is not a convenience method. It is the composition seam the mathematics crates share, and
it decides more of QCL's shape than the precision trick alone suggests. Composition across those
crates runs through `deep_causality_haft`: a crate owning a container generic in its element declares
a **witness** type, binds `type Type<T>` to the container, and implements the categorical traits
against the witness. See
[the unified-math README](../../../deep_causality_unified_math/README.md).

**Three consequences for this design.**

**A `CausalTensor<Complex<R>>` is the seam, not a coincidence.** A witness accepts any element type,
including one another crate owns, so the operator carrier QCL uses throughout is an instance of the
nesting mechanism. The same mechanism is what lets the scalar be swapped, so §6.4's precision claim
and §6.1's operator carrier are the same fact seen twice.

**And `Complex<T>` is itself a functor, which is what makes the precision lift actually work on that
carrier.** `deep_causality_num_complex` declares `ComplexWitness`, `QuaternionWitness` and
`OctonionWitness`, and each implements `HKT`, `Functor`, `Foldable`, `Semigroupal`, `LaxMonoidal`,
`MonoidalApplicative` and `Convolutional`. `ComplexWitness::fmap` maps the two real slots,
`re` and `im`, so `Complex<A> → Complex<B>` under any `f: A → B`.

§6.4 states the precision lift as `CausalTensor::fmap<A, B>` changing the scalar type. That is exact
for a real-valued tensor and incomplete for the carrier QCL actually uses, whose element is
`Complex<R>` rather than `R`. Lifting the *real* parameter of a complex operator is a composition of
two functors, the outer over cells and the inner over `re`/`im`:

```rust
let lifted: CausalTensor<Complex<Float106>> =
    CausalTensorWitness::fmap(op, |z| ComplexWitness::fmap(z, lift));
```

That is worth stating because it is the difference between the claim being true of the note's
example and true of its operators.

**Two further facts about those instances.** `Complex` has no `Monad`, and should not: there is no
`join` on a two-slot record that respects the identity laws, which is the same reason
`CausalMultiVectorWitness` gave one up. And the applicative on offer is the **monoidal** route rather
than the monadic one. `MonoidalApplicative` derives `apply` from `φ` and pairs slot with slot,
consuming nothing twice, where the monadic route induces `ap(ff, fa) = bind(ff, |f| fmap(fa, f))` and
re-runs the continuation once per function. For zipping two complex fields the monoidal route needs
no `Clone`, which on operator-sized data is the difference that matters.

The three Cayley-Dickson types carry the same instance set, 𝕆 included. Nothing there is surprising
once stated: `fmap` never touches multiplication, so non-associativity costs the functor nothing.

**`CausalTensor` carries the richest instance set of any crate here**, which bounds what a stage may
assume. It implements `HKT`, `Functor`, `Foldable`, `Applicative`, `Pure`, `CoMonad`, `Monad`,
`Semigroupal`, `MonoidalApplicative` and `Arrow`. Two measured facts about those instances matter to
an operator pipeline, and both were found by running code rather than reading signatures:

- **`CausalTensorWitness::bind` preserves the input's shape** when the map is shape-preserving, so a
  `[2, 3]` no longer comes back `[6]`. For QCL that is load-bearing rather than tidy: an operator is
  a shaped tensor, and a `bind` that flattened it would turn a matrix into a vector silently.
- **`CausalMultiVectorWitness` gave up `Monad`**, because no metric choice satisfies both identity
  laws. A reader who finds the retired multivector gate layer in this note's history should not
  reach for `bind` over multivectors to revive it; the instance is absent by proof, not by omission.

**QCL consumes the seam and does not extend it.** `deep_causality_quantum` sits outside
`deep_causality_unified_math/`, declares no witness, and appears in none of the README's instance
rows. That is the right position and it should stay: the categorical structure QCL threads is the
causal monad's `PropagatingEffect`, which is §7.7's point, while the math-side instances are for
element-wise work on the carriers. Conflating the two layers is how a `QclEffect` gets invented.

---

## 7. The pipeline

### 7.1 One origin for configuration

Every configuration comes from `QclBuilder::config()`, which is also the single site where the
working types are named. There are **two**, because §6.4's two axes are independent:

```rust
QclBuilder::config::<FloatType, NumberType>()
```

Every bound in §6.4 is discharged from those parameters, tolerances included. Swapping `FloatType`
re-types the whole run, thresholds and all. Swapping `NumberType` re-types the ledger's counts, and
buys headroom rather than accuracy: there is no threshold to move, because the failure it guards
against is overflow rather than rounding. It is the unsigned alias, because a count is ℕ and
`NaturalNumber` is unsigned; `IntType` is ℤ and belongs to ring arithmetic (§6.5).

An earlier revision named one parameter here. That was correct while the counts were hardcoded and
is not correct now that they are a bound (§6.5).

### 7.2 The config branches on the subject

| Constructor | Subject | Candidates | Reachable |
|---|---|---|---|
| `.over_plant(p)` | a system that evolves and is measured | `Hypothesis` | `validate` if structural, `control` always |
| `.over_model(graph, factors, supports)` | a CJ factorization over a frozen graph | the factorization itself | `validate` |
| `.over_code(c)` | a complex, evaluated exactly | candidate complexes | `validate` |

`.over_model` is the degenerate case of `.over_plant` with one structural candidate and no evidence.
It exists as its own constructor because its checks examine a factorization rather than choose
between them, and because it is what the shipped `freeze_quantum` callers map onto.

**Preconditions `build()` enforces.** Both graph bridges reject an unfrozen graph, because
`remove_node` tombstones a slot without compacting, so a live node can have an id past
`number_nodes()` and its edges would be silently dropped, which is an unsound false negative in a
faithfulness gate. `build()` also rejects a probe naming an observable the plant does not expose, a
zero shot count, and an empty candidate set.

### 7.3 What the config holds, and what it does not

The config holds what several stages share: the subject, the evidence policy, the baseline
experiment, the probe family, the candidates, the tolerance policy.

**A stage's parameter belongs to the stage.** `spec` goes to `.gate(spec)`, the objective to
`.design(objective)`, the depth to `.evolve(n)`. When `spec` was a required config field, the
crosstalk example had to invent one to satisfy the builder.

### 7.4 The hand-off is a type, and failure is a rollback

`validate` terminates in `Screened<R>`, carrying the config, the admitted subset, and the
`CheckReport<R>` of §3.1. `control` accepts either a plant config or a `Screened<R>`, so a config
carrying structural candidates has no path into `control` that skips validation. On failure §3.4
applies. And per §5.1, a `Screened<R>` whose factorization is later marginalised carries an
invalidated report, not a stale one.

### 7.5 The four consumers expressed

**The QCM path.** Validate only; maps one-to-one onto `freeze_quantum`.

```rust
let cfg = QclBuilder::config::<FloatType, NumberType>()
    .over_model(graph, factors, supports)               // rejects an unfrozen graph
    .tolerance(Tolerance::q_tol().with_safety_factor(ft(8.0)))
    .declare_systems(&inputs, &outputs)
    .build()?;

QclBuilder::validate(&cfg)
    .check_markov()                                     // ‖[ρ_j,ρ_k]‖_F vs Q-TOL, per intersecting pair
    .check_decomposable()                               // C₃-exclusion over derived reachability
    .finalize().print_results();                        // worst_margin, tested_pairs
```

**Calibration counterfactual.** Mechanism candidates, so `control` directly.

```rust
let cfg = QclBuilder::config::<FloatType, NumberType>()
    .over_plant(transmon)
    .evidence(Evidence::shots(1024).seed(20260821))     // selects the emergent modality, §1.1
    .baseline(Experiment::probe("check", excited_population, 1, cost = 1))
    .probes(&amplification_family)                      // depths 1..40
    .candidates(&[
        Hypothesis::mechanism("amplitude",   amp_drift),
        Hypothesis::mechanism("detuning",    det_drift),
        Hypothesis::mechanism("decoherence", depolarising),
    ])
    .build()?;

QclBuilder::control(&cfg)
    .observe()                                          // the measurement boundary; verdicts enter here, §4
    .gate(Spec::at_least(ft(0.999)).within(Tolerance::shot_noise()))
    .fork()                                             // one world per fault
    .design(MinCostCover { floor_bits: ft(5.0) })
    .predict()                                          // marginalise only through the §5.1 boundary check
    .adjudicate()
    .finalize().print_results();
```

**Crosstalk attribution.** The only consumer running both halves.

```rust
let cfg = QclBuilder::config::<FloatType, NumberType>()
    .over_plant(two_qubit)
    .evidence(Evidence::shots(1024).seed(20260821))
    .baseline(Experiment::probe("passive", joint_error, 1, cost = 1))
    .probes(&[do_q1, do_q2, echo_both, process_tomography])
    .candidates(&[
        Hypothesis::structural("Q1->Q2", f_d12),         // each is a factorization, §6.1
        Hypothesis::structural("Q2->Q1", f_d21),
        Hypothesis::structural("common", f_com),
        Hypothesis::structural("cyclic", f_cyc),
    ])
    .build()?;

let screened = QclBuilder::validate(&cfg)
    .check_markov()
    .check_decomposable()                               // C₃-exclusion; the cyclic candidate never reaches it, §7.6
    .finalize();                                        // -> Screened<FloatType>

QclBuilder::control(screened)                           // unreachable unscreened
    .observe()
    .gate(Spec::uncorrelated())
    .fork()
    .design(MinCostCover { floor_bits: ft(5.0) })       // {E1, E2} at cost 2 beats tomography at 200
    .predict()
    .adjudicate()                                       // Boolean fold: real-valued spec, §4 rule 2 does not apply
    .finalize().print_results();
```

**Geometric QEC.** A code subject, validate only.

```rust
let cfg = QclBuilder::config::<FloatType, NumberType>()
    .over_code(LatticeComplex::<2, FloatType>::square_torus(4))
    .build()?;

QclBuilder::validate(&cfg)
    .derive_code()                                      // [[32,2,4]]
    .check_ldpc_weights()
    .check_class_invariance()                           // the diagonal gates of Table 1, over the code space
    .finalize().print_results();
```

Verified by exact 𝔽₂ predicates; not simulated. `SimQpu` caps at 24 qubits, below this code's 32,
so no gate this consumer emits is ever run through the simulator, and the only dynamical evidence for
any Haruna gate is the gate-alphabet identity tests on small registers. The checks are combinatorial
and exact, which is why that is enough; it is stated so that "tested on the simulator" is never
added to a claim about this path. `H̄` is neither a Pauli nor diagonal and is covered by the Clifford
check the change specifies, not by class invariance.

The code pipeline has no probe family. Its selection over candidate complexes is scored against
requirements rather than evidence, so it shares a shape with `design` and none of its content
(established in [`qcl-dsl-liftback.md`](qcl-dsl-liftback.md) §8.9). v1 does not unify them.

### 7.6 Three constraints the crate imposes

**Flatness, and where it actually lives.** The convention is `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)`, one
system per node, and it is implemented by exactly one function: `FactorSupports::from_graph`. The
representation does not require it, since `declare` accepts any leg list (§5.2). Note also that the
causal layer is not the constraint: `Causaloid::from_causal_graph` builds a `CausaloidType::Graph`
node holding an `Arc<CausaloidGraph<Self>>`, and the uniform `Causable::evaluate` routes such a node
to `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful` rather than
refusing it. Recursion is wired below QCL; the flat convention is a QCM derivation choice above it.

**The model subject is std-only.** The QCM path reaches `deep_causality` for `CausableGraph`, which
is why it sits behind the `qcm` feature. So `.over_model` and structural `.over_plant` validation
cannot reach bare metal, while `.over_plant` with mechanism candidates plus `control` can.

**Decomposability is C₃-exclusion only**, per van der Lugt & Lorenz (arXiv:2508.11762,
Definition 3.1 and Theorem 3.2), and it is decidable by Theorem 4.9(v) directly from the relation.
"Faithful" there is Lorenz–Barrett's `G_U = G_C`, a circuit decomposition whose connectivity equals
the unitary's causal structure, and not Pearl's, which is why the stage is `check_decomposable` and
not `check_faithfulness`. The general routed and direct-sum Lorenz–Barrett hypothesis is open
upstream. QCL names the scope it inherits and claims nothing wider.

**Cyclic causal structures are out of scope for v1 by decision, not because they fail a check.**
Cyclic QCMs exist (Barrett, Lorenz & Oreshkov, arXiv:2002.12157). The C₃ criterion is applied to
acyclic influence relations; a cyclic candidate is rejected at `build()` with
`CyclicStructureUnsupported`, before the check runs. It has to be, because the criterion would not
reject it: the crosstalk example's cyclic H₄ satisfies C₃-exclusion under Definition 3.1, and its
reachability on a cyclic graph is complete, which satisfies it more plainly still.

### 7.7 The DSL adds ordering and naming, not a monad

`PropagatingEffect` already short-circuits and carries an `EffectLog`, and `wrappers.rs` shows the
lifting pattern. If a `QclEffect` type appears, something has gone wrong.

---

## 8. What QCL must not do

1. **Own graph traversal.** That is `deep_causality`'s `CausaloidGraph` and `ultragraph`. The bridges
   exist; QCL calls them.
2. **Own topology.** Chain complexes, Betti numbers and the cup product are
   `deep_causality_topology`'s.
3. **Own the `Verdict` trait.** The trait is `deep_causality_algebra`'s. The *carrier*
   `Projection<R, D>` is this crate's, and §4 is a law QCL obeys rather than a component it adds.
4. **Model devices.** The rotation-with-detuning plant belongs to an example.
5. **Reimplement the shipped layer.** The commutativity check, the C₃ search, the tolerances, the CJ
   round-trip and the factor store are written. QCL wraps them in stage names and generalises their
   report shape.
6. **Marginalise without a boundary.** §5.1.
7. **Claim fault tolerance, or Lean coverage it does not have.** `LEAN_QUANTUM.md` lists what is
   proved and what is deferred; a QCL check that claims a theorem names its Rust witness through
   `lean/THEOREM_MAP.md`, and a check with no proof says so.

---

## 9. Sequencing

~~0. **Fix the scalar bounds** (§6.4).~~ **Done.** The table was re-derived against shipped
   signatures and its premise did not survive: one row is over-tight, three are *under*-specified,
   one is not implementable as written, and one contradicted §6.5. See §6.4.

~~2. **`Boundary`, then `GaugeField::from_cochain`** (§6.2).~~ **Done and withdrawn respectively.**
   `partial_trace_preservation_boundary` ships; the gauge-field seam was removed rather than checked.
   Neither of the two wrong-answer seams is open, so nothing in the remaining sequence is load-bearing
   for soundness.

What is left, renumbered:

1. **Generalise `Check<R>`, `CheckReport<R>` and `Tolerance<R>`** off the four shipped policies (§3).
   Doing this first means no stage is ever written returning a boolean. Each new check inherits
   §3.2's obligation to report what it examined, not only what passed.
2. **Carriers** (§6.2), each under the seal rule of §6.3, and each with a `wrappers.rs`-style lift.
3. **`ShotBudget`** (§6.2). Small, and it turns every downstream decision from a float comparison
   into a statistical one.
4. **`Hypothesis` and `intervene`** (§6.1). Smaller than previously sequenced; the store, supports and
   embedding exist, so this is the `do` operation plus the boundary-checked contraction.
5. **`design` and `adjudicate`**, the latter with the §4 commutation guard, and the former sized off
   §10.3's corrected exponent rather than the sweep it used to name.
6. **`QclBuilder::config`, then the stages** (§7), last, once at least two consumers run against the
   layers beneath.

**The prerequisites are done, and none of what remains is a gap.** Every item in
[`qcl-gaps.md`](qcl-gaps.md) is closed, so the geometric-QEC example is no longer blocked on the
substrate, and the six steps above are construction against settled designs rather than open
questions.

The failure mode to avoid is unchanged: writing the pipeline first and shaping examples to justify
it. The ordering is the reverse, and it is now anchored on a running implementation rather than three
designs alone.

---

## 10. Benchmarks

Two claims nothing currently measures: that a tick is cheap enough for a control loop, and that
precision is a parameter. `criterion` with `harness = false` is the repo's convention, and every
figure carries the machine. The workspace reference is **M3 Max, 16 cores, 128 GB**.

### 10.1 Reference points

| Quantity | Published value | Source |
|---|---|---|
| Superconducting code cycle | 0.2–10 µs | arXiv:2108.12371 |
| Real-time decode, per round | sub-1 µs mean | Nature Comms (2026) |
| Feedback latency, superconducting | 9.6 µs | Nature Comms (2026) |
| Trapped-ion shuttling code cycle | ~235 µs | arXiv:2108.12371 |
| Crosstalk detection, experiment count | O(n²)–O(n³) | Sarovar et al. (2020) |

### 10.2 Tier 1: tick latency

`bind` on `PropagatingEffect<FloatType>` as the floor; `bind` on
`PropagatingProcess<Rho, Ledger<FloatType>, Config>` as a realistic carrier; the same with the log
enabled, bounded and off; a full observe → gate → `alternate_value` tick.

Read against 235 µs and 0.2–10 µs. **Allocations per tick matter more than wall clock.** A
bounded-time claim needs allocations/tick = 0, which a timing harness does not show.

### 10.3 Tier 2: stage cost

| Benchmark | Watch for |
|---|---|
| `check_markov` per intersecting pair | pairs grow O(n²) in factors; `embed_on_legs` dominates |
| `check_decomposable` | `find_c3` is `C(m,3)²` over declared inputs and outputs |
| `Projection::range_projector` | one `eigen_hermitian` per lattice join; the verdict fold is not free |
| `design`, k experiments × n hypotheses | **linear in k, exponential in n**: the exact cover is a DP over covered-pair subsets, `O(2^C(n,2) · k)`. Watch the coverage-matrix build, which dominates at this note's own scale |

Three cliffs, not one. `design` needs a sweep of **n**, the hypothesis count, not of k. §8.8 of the
liftback note formulates the stage as minimum-cost set cover whose universe is the `C(n,2)`
hypothesis pairs and whose sets are the k experiments. Enumerating subsets of experiments costs
`2^k`, which is where "exponential in k" came from, but it is the wrong enumeration: the exact
answer is a DP over subsets of the *universe*, `dp[S | cover(e)] = min(dp[S], dp[S] + cost(e))`, at
`O(2^C(n,2) · k)`. That is linear in k, so sweeping k from 4 to 20 finds no cliff. The cliff is in n
and arrives around n = 7 or 8, where `C(n,2)` reaches 21 to 28 and `2^C(n,2)` stops being free.
`2^C(n,2)` is `2^15` at n = 6, `2^28` at n = 8, `2^45` at n = 10, so the bound is a decision rather
than a benchmark observation: `MinCostCover` carries `max_hypotheses`, default 7, and `design`
returns `HypothesisCountExceeded { n, pairs }` above it before allocating the table, in the sense
§6.5 uses for `NumberType`. A caller with ten hypotheses gets an error, not a hang.

At this note's own numbers the solve is not the cost at all. §8.6 sizes the scan at `|E| × |H|`
plant evolutions plus `|E| × C(|H|, 2)` closed-form coefficients, 120 and 120 for 40 depths and 3
hypotheses. The cover DP over those same numbers is `2^3 × 40 = 320` arithmetic steps against 120
plant evolutions. **Benchmark the coverage-matrix build, and sweep n to find the exponent.**

`find_c3` needs a sweep of declared system counts, because its sextuple loop grows in a parameter
the user chooses. The verdict fold needs measuring because §4 puts an eigendecomposition inside
`adjudicate`.

### 10.4 Tier 3: the price of precision

Every Tier 1 and Tier 2 benchmark at `f32`, `f64` and `Float106`, recording the **tolerance** at each
scalar alongside the time, so the report shows what the extra precision bought as well as what it
cost.

### 10.5 What these benchmarks must not do

They do not compare QCL against Qiskit Experiments, Qibocal or Stim on shared work; those pipelines
compute different things. What is measured is distance from a published physical budget, not a
ranking.

---

## 11. Sources

**The composition seam and the tower underneath.**
[`deep_causality_unified_math/README.md`](../../../deep_causality_unified_math/README.md) — the
seventeen crates, the seven tiers, and the witness table §6.6 reads from.

[`README_ALGEBRA_TRAITS.md`](../../../deep_causality_unified_math/deep_causality_algebra/README_ALGEBRA_TRAITS.md)
— the trait hierarchy, the five marker laws, and the per-type matrix over ℕ, ℤ, ℚ, ℝ, ℂ, ℍ and 𝕆
that §6.4's bounds are read off.

[`deep_causality_num/README.md`](../../../deep_causality_unified_math/deep_causality_num/README.md)
— the representation half: `Integer`, `SignedInt`, `UnsignedInt` and `NaturalNumber`, and the
`FloatType` / `IntType` split, and the `NumberType` alias for ℕ that §7.1's second parameter is.

**Shipped code this design wraps.** `deep_causality_quantum/src/`, 4432 lines, in particular
`types/qcm/{markov_freeze, faithfulness, process_factors, environment}.rs`,
`types/qgates/{operator_linalg, channel, mechanics, wrappers, gates_haruna}.rs`,
`types/verdict/{projection, born}.rs`, `types/qcode/`, `types/density_matrix.rs`,
`error/quantum_error.rs`, and `types/qpu/`, whose `circuit.rs` is always compiled while the sampler
seam stays behind the `qpu` feature. The mathematics crates it reaches sit under
`deep_causality_unified_math/`, in particular `deep_causality_homology` for the chain layer and
`deep_causality_topology` for the cup product and the gauge fields.

**Verification status.** [`LEAN_QUANTUM.md`](../../../deep_causality_quantum/LEAN_QUANTUM.md) and
`lean/THEOREM_MAP.md`, for what is proved, what is deferred, and the counterexample behind F9.

**Consumer designs.** [`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md),
[`example-geometric-qec.md`](example-geometric-qec.md). Prior sketch:
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md). Positioning: [`positioning.md`](positioning.md).

**Physics and method.**

- `website/docs/src/content/docs/concepts/uniform-math.md` — precision as a parameter and the
  algebraic trait floor §6.4 builds on.
- Lorenz, R. (2022); Lorenz, R. & Barrett, J. (2021) — the quantum causal model the crate
  reconstructs, and the Markov condition Def 3.3 that `markov_freeze.rs` implements.
- van der Lugt, T. & Lorenz, R. (2025). *Unitary causal decompositions.* arXiv:2508.11762 — the
  C₃-exclusion criterion and the scope limit in §7.6. `C₃` is stated at Example 2.12, the property
  at Definition 3.1, the equivalence at Theorem 3.2, the relation-level reformulation at
  Theorem 4.9(v), and what the theorem does not say at Remark 3.3.
- Barrett, J., Lorenz, R. & Oreshkov, O. (2021). *Cyclic quantum causal models.* Nat. Commun. 12,
  885; arXiv:2002.12157, in `deep_causality_quantum/papers/` — the reason cyclic structures are a
  scope decision rather than a failed check.
- Birkhoff, G. & von Neumann, J. (1936) — the quantum logic `Projection` realises.
- Nielsen, M. & Chuang, I. *Quantum Computation and Quantum Information* — the generalised Rabi
  formula and the depolarising channel behind the calibration example.
- Verma, T. & Pearl, J. (1990); Pearl, *Causality*, 2nd ed., ch. 1 — Markov equivalence behind the
  crosstalk example's structural degeneracy.
- Sarovar, M., Proctor, T., Rudinger, K., Young, K., Nielsen, E. & Blume-Kohout, R. (2020).
  *Detecting crosstalk errors in quantum information processors.* Quantum **4**, 321 — the prior work
  the crosstalk example defers to.
- Kitaev, A. (2003). Ann. Phys. **303**, 2–30 — the toric code parameters in the QEC example.
- Chen, Y.-A. & Tata, S. (2023). arXiv:2106.05274 — the cup product, in
  `deep_causality_unified_math/deep_causality_topology/papers/`.
- Haruna, J. (2025). arXiv:2511.15224 — the logical gates, in `deep_causality_quantum/papers/`.
- Kelly, J. et al. (2018). arXiv:1803.03226 — the `check_data` trichotomy `Ambiguous` improves on.
