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

**What changed in this revision.** The crate was read in full: 3980 lines of source plus
[`LEAN_QUANTUM.md`](../../../deep_causality_quantum/LEAN_QUANTUM.md). Three things the previous
revision asserted turn out to be wrong.

1. `freeze_quantum` **is** the validate half, already written, for one class of subject. The two
   checks listed as future work are shipped functions.
2. The crate **does** contribute a verdict carrier. `Projection<R, D>` is an orthomodular lattice
   that fails distributivity, and it imposes a law on how QCL may combine verdicts (§4).
3. `predict` cannot be embed-and-contract, because **partial trace does not preserve commutation**.
   That is proved false in Lean with an explicit counterexample, and nothing at the call site says
   so (§5.1).

---

## 1. Four consumers, one of them running

| Consumer | Status | Subject | What it contributed |
|---|---|---|---|
| **The shipped crate** | **3980 lines** | a CJ factorization over a frozen graph | The check-and-margin form (§3); the tolerance family (§3.3); the verdict boundary law (§4); transactional failure (§3.4) |
| `quantum_control_loop` | design | a device plant | The control stage sequence; `Ambiguous` as a required verdict; the experiment designer |
| `crosstalk_attribution` | design | a plant with declared structures | `design` returns a plan, not an experiment; the validate-to-control hand-off |
| `geometric_qec` | design | a chain complex | `validate` takes a code; the two halves are not uniformly applicable |

The three designs were each implemented twice in scratch, once in Rust against shipped APIs and once
in Python from published formulas, agreeing on all 16 checked quantities. The three faults read
`0.980147 / 0.980159 / 0.980150` at one pulse and `0.086460 / 0.961136 / 0.847242` at nine; the three
causal structures all fit `P(e₁)=P(e₂)=0.10, P(e₁,e₂)=0.04` and the cyclic fourth is rejected as a
`C₃`; the toric family comes out `[[8,2,2]]` to `[[50,2,5]]` with Betti `[1,4,6,4,1]` on `T⁴`.

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
| `check_faithfulness` | C₃ blocks found | zero | shipped |
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
2. **Combining verdicts requires commutation.** When `adjudicate` folds verdicts from forked worlds,
   projections that do not commute have no distributive joint verdict. `Projection::commutes_with`
   is the guard, and a non-commuting fold is an `Ambiguous`, not an answer.

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
| F8 | The topology-to-gate seam is unchecked and silently wrong | open, and it produces confident wrong answers |
| **F9** | **Marginalisation is not commutation-preserving, and nothing at the call site says so** | **new, §5.1** |

F8 remains sharp. A logical operator is a cochain indexed by edge; a Haruna gate wants a
`CausalMultiVector` gauge field; no conversion exists, so the scratch code packed the cochain's sum
into a multivector of unrelated dimension. The failure surfaced as a Taylor-series convergence
complaint rather than a type error, and normalising the value to make it converge would have produced
a confident, meaningless answer.

### 5.1 F9: partial trace does not preserve commutation

This is new to this revision and it is the reason `predict` cannot be written the obvious way.

`LEAN_QUANTUM.md` records that `quantum.partial_trace_preservation` is **false**, refuted in Lean by
an explicit counterexample: operators with `[X, Y] = 0` whose partial traces satisfy
`[Tr_B X, Tr_B Y] = [[0, 4], [−4, 0]] ≠ 0`. Partial trace is positive-linear but not an algebra
homomorphism. What holds is the **conditional** `partial_trace_preservation_boundary`: a boundary
operator of the form `Z ⊗ 1_B` commuting with `M` forces `Z` to commute with `Tr_B(M)`.

The API does not carry this. `partial_trace` in `operator_linalg.rs` documents its shape errors
carefully and says nothing about preservation. So a `predict` that marginalises a validated
factorization over the traced-out legs would produce a model **whose Markov property validate had
certified and marginalisation silently destroyed**. That is F8's failure mode in a different seam:
the answer arrives, and nothing marks it as unsound.

Two consequences:

- **`intervene` and `predict` may marginalise only across a boundary**, where the operator has the
  `Z ⊗ 1_B` form the conditional theorem requires. QCL should encode that as a precondition it
  checks, not a caveat it documents.
- **The Markov report does not survive marginalisation.** A `Screened<R>` whose factorization is
  later traced has to re-run `check_markov` or carry an invalidated report. Carrying the old margins
  forward would be the same class of error.

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

**Open, under investigation.** Whether a composed channel can inherit a Markov certificate from its
parts turns on whether composition over a shared wire invokes a partial trace at all under this
crate's input-major Choi convention. If it does, validating parts does not validate the whole and
nesting buys no verification saving. If it does not, the question is narrower than F9 suggests. The
crate has no composition function today, so this is a design boundary rather than a withdrawn
capability.

---

## 6. Carriers

### 6.1 What the crate already supplies

| Piece | Location | What it gives |
|---|---|---|
| `ProcessFactors<R>` | `qcm/process_factors.rs` | one CJ operator per node, keyed by node index |
| `FactorSupports` | `qcm/process_factors.rs` | ascending leg-ids per node, per-leg dimensions, `space_map` |
| `FactorSupports::validate` | `qcm/process_factors.rs` | factor shape against declared support, overflow-checked |
| `embed_on_legs`, `partial_trace` | `qgates/operator_linalg.rs` | lift onto a larger space; trace out (subject to F9) |
| `matrix_commutator`, `frobenius_norm`, `hermiticity_defect` | `qgates/operator_linalg.rs` | the operator metrics the checks compare |
| `choi_from_kraus`, `kraus_from_choi`, `apply_kraus`, `apply_choi` | `qgates/channel.rs` | the CJ round-trip, both directions |
| `check_completely_positive`, `check_trace_preserving` | `qgates/channel.rs` | the CPTP checks `Channel` should run once |
| `CausalStructure::from_graph_reachability` | `qcm/faithfulness.rs` | derive the input/output relation from a frozen graph |
| `FactorSupports::from_graph` | `qcm/process_factors.rs` | derive supports as `{Aᵢ} ∪ Pa(Aᵢ)` |
| `Projection<R, D>` | `verdict/projection.rs` | the orthomodular verdict carrier of §4 |
| `born_projective_probability`, `born_projective_prob` | `verdict/born.rs` | the measurement boundary where verdicts are extracted |
| `logical_z/x/s/hadamard/cz/t` | `qgates/gates_haruna.rs` | the logical gate layer `geometric_qec` reaches for |
| `qgates/wrappers.rs` | 11 functions | **the F4 pattern, already written** |

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
*derived* rather than stored beside it. `intervene(do(node ← factor))` is `factors.insert(node, f)`
followed by `supports.validate(&factors)`; Pearl's cut is a factor replacement and the store already
supports it by key. What is **not** free is `predict`, for the reason in §5.1.

### 6.2 What must be added

| Type | Replaces | Contract |
|---|---|---|
| `QubitOperator` | hand-packed 2×2 tensors | named constructors: `pauli_x`, `rotation(axis, angle)`, `phase(θ)` |
| `Channel` | a bare Kraus slice | CPTP checked once at construction, via the shipped checks |
| `QuantumPlant` | state and channel juggling | a sealed validated state that evolves in place |
| `Observable` | ket → `Projection` → Born | a named projector carrying its own read-out |
| `GaugeField` | the unchecked F8 seam | constructible **only** as `from_cochain(&complex, &cochain, degree)` |
| `Tolerance<R>` | naked float comparison | the §3.3 family, generalised off the four shipped policies |
| `Check<R>` / `CheckReport<R>` | pass/fail | the §3.1 form, generalised off `CommutatorCheck` |
| `Boundary` | an unchecked `partial_trace` call | the `Z ⊗ 1_B` precondition of §5.1, checked |

`GaugeField` and `Boundary` are the two that prevent wrong answers rather than verbose code. The
rest are ergonomics.

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

| Surface | Operations | Bound | Why not tighter or looser |
|---|---|---|---|
| Cochains, cup product | `+`, `−`, `×`, `0` | `CommutativeRing + Copy` | No division, ordering or analytic call |
| Operators, CJ factors, gauge field | complex arithmetic | `ComplexField<R>`, `R: RealField` | Entries are genuinely complex; the real parameter carries the precision |
| Rotations, gate synthesis | `sin`, `cos`, `sqrt`, `π` | `RealField` | Needs the analytic surface and division |
| Born read-out, purity | real output against a spec | `Real` | Ordering yes, division no |
| Tolerances (§3.3) | `ε`, `sqrt`, `+`, `×`, one division | `RealField + FromPrimitive` | What the shipped policies declare |
| Verdicts | orthomodular lattice | `Verdict` on `Projection<R, D>` | §4; the carrier is the crate's, the trait is not |
| Shot statistics, Bhattacharyya | `sqrt`, `log2`, ratios | `RealField` | Genuinely needs both halves |
| Costs, shot counts, cover search | integer arithmetic | none | `usize` |

The discipline is one line: **bound at the weakest structure that carries the operation.** The cup
product is the worked example: bound on `RealField`, relaxed to `CommutativeRing + Copy`, workspace
compiles and 1471 tests pass unchanged.

Four things this does not license. No bound wider than the operation. No scalar parameter where there
is no scalar. `Real` and `RealField` are not synonyms. Complex is not a precision;
`ComplexField<R>` carries its precision in `R`.

`CausalTensor::fmap<A, B>` changes the scalar *type* rather than the values, so a carrier can be
lifted between precisions as an operation. Combined with §3.3, that lets the pipeline answer "is this
verdict precision-limited?" about itself: run at two scalars and note that the tolerances moved with
them.

### 6.5 The ledger

Device time is the scarce resource. Context is read-only and cannot accumulate, `EffectLog` is a
record rather than a running total, and Value is the answer rather than the meter, so **State is the
only channel both writable and threaded.**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ledger<R> {
    shots: u64,        // taken on the device
    experiments: u32,  // executed on hardware
    predictions: u32,  // model evaluations; tracked, never billed
    device_time: R,    // accumulated, measured
    cost: R,           // what MinCostCover minimises
    bits: R,           // separation achieved so far
}
```

`Real` is `Copy`, so `Ledger<R>` is `Copy`; it holds no `Vec` and no `String`. `Default` is
hand-written from `R::zero()`, because `CausalMonad::pure` requires `State: Default` and deriving
would impose a spurious `R: Default`. Accumulation binds on `Real`; only the ratio methods need
`RealField`, which keeps dual numbers admissible so a cost model stays differentiable.

**Three invariants.** *Increment only at the device boundary*: `observe` is the only stage touching
`shots`, `experiments` and `device_time`, because a fork into three hypothesis worlds runs one
experiment and two predictions. *Forking is QCL's, not core's*: `branch` and `either` route to one arm
and move the state, so `fork` is built above core by cloning. *Do not merge forked ledgers with ∇*:
the monoid typechecks and gives the wrong answer, because at a counterfactual fork exactly one branch
was factual.

---

## 7. The pipeline

### 7.1 One origin for configuration

Every configuration comes from `QclBuilder::config()`, which is also the single site where the scalar
is named:

```rust
QclBuilder::config::<FloatType>()
```

Every bound in §6.4 is discharged from that parameter, tolerances included. Swapping `FloatType`
re-types the whole run, thresholds and all.

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
let cfg = QclBuilder::config::<FloatType>()
    .over_model(graph, factors, supports)               // rejects an unfrozen graph
    .tolerance(Tolerance::q_tol().with_safety_factor(ft(8.0)))
    .declare_systems(&inputs, &outputs)
    .build()?;

QclBuilder::validate(&cfg)
    .check_markov()                                     // ‖[ρ_j,ρ_k]‖_F vs Q-TOL, per intersecting pair
    .check_faithfulness()                               // C₃-exclusion over derived reachability
    .finalize().print_results();                        // worst_margin, tested_pairs
```

**Calibration counterfactual.** Mechanism candidates, so `control` directly.

```rust
let cfg = QclBuilder::config::<FloatType>()
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
    .predict()                                          // no marginalisation without a Boundary, §5.1
    .adjudicate()
    .finalize().print_results();
```

**Crosstalk attribution.** The only consumer running both halves.

```rust
let cfg = QclBuilder::config::<FloatType>()
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
    .check_faithfulness()                               // rejects the cyclic C₃ before any shots
    .finalize();                                        // -> Screened<FloatType>

QclBuilder::control(screened)                           // unreachable unscreened
    .observe()
    .gate(Spec::uncorrelated())
    .fork()
    .design(MinCostCover { floor_bits: ft(5.0) })       // {E1, E2} at cost 2 beats tomography at 200
    .predict()
    .adjudicate()                                       // non-commuting projections fold to Ambiguous, §4
    .finalize().print_results();
```

**Geometric QEC.** A code subject, validate only.

```rust
let cfg = QclBuilder::config::<FloatType>()
    .over_code(LatticeComplex::<2, FloatType>::square_torus(4))
    .build()?;

QclBuilder::validate(&cfg)
    .derive_code()                                      // [[32,2,4]]
    .check_ldpc_weights()
    .check_class_invariance()                           // closes F8 via GaugeField::from_cochain
    .finalize().print_results();
```

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

**Faithfulness is C₃-exclusion only**, per van der Lugt & Lorenz (arXiv:2508.11762). The general
routed and direct-sum Lorenz–Barrett hypothesis is open upstream. QCL names the scope it inherits and
claims nothing wider.

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

0. **Fix the scalar bounds** (§6.4). Costs nothing; every layer below inherits what the carriers
   declare.
1. **Generalise `Check<R>`, `CheckReport<R>` and `Tolerance<R>`** off the four shipped policies (§3).
   Doing this first means no stage is ever written returning a boolean.
2. **`Boundary`, then `GaugeField::from_cochain`** (§6.2). The two seams that produce confident wrong
   answers. F9 first because `predict` depends on it.
3. **Carriers** (§6.2), each under the seal rule of §6.3, and each with a `wrappers.rs`-style lift.
4. **`ShotBudget`** (§6.2). Small, and it turns every downstream decision from a float comparison
   into a statistical one.
5. **`Hypothesis` and `intervene`** (§6.1). Smaller than previously sequenced; the store, supports and
   embedding exist, so this is the `do` operation plus the boundary-checked contraction.
6. **`design` and `adjudicate`**, the latter with the §4 commutation guard.
7. **`QclBuilder::config`, then the stages** (§7), last, once at least two consumers run against the
   layers beneath.

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
| `check_faithfulness` | `find_c3` is `C(m,3)²` over declared inputs and outputs |
| `Projection::range_projector` | one `eigen_hermitian` per lattice join; the verdict fold is not free |
| `design`, k experiments × n hypotheses | **exponential in k**: exhaustive bitmask over subsets |

Three cliffs, not one. `design` needs a sweep of k from 4 to 20. `find_c3` needs a sweep of declared
system counts, because its sextuple loop grows in a parameter the user chooses. The verdict fold
needs measuring because §4 puts an eigendecomposition inside `adjudicate`.

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

**Shipped code this design wraps.** `deep_causality_quantum/src/`, 3980 lines, in particular
`types/qcm/{markov_freeze, faithfulness, process_factors, environment}.rs`,
`types/qgates/{operator_linalg, channel, mechanics, wrappers, gates_haruna}.rs`,
`types/verdict/{projection, born}.rs`, `types/density_matrix.rs`, `error/quantum_error.rs`, and
`types/qpu/` behind the `qpu` feature.

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
  C₃-exclusion criterion and the scope limit in §7.6.
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
  `deep_causality_topology/papers/`.
- Haruna, J. (2025). arXiv:2511.15224 — the logical gates, in `deep_causality_quantum/papers/`.
- Kelly, J. et al. (2018). arXiv:1803.03226 — the `check_data` trichotomy `Ambiguous` improves on.
