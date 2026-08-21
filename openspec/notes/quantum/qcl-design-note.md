<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL: Design Note

**What this is.** The design for a Quantum Causal Language in
`deep_causality_quantum`, derived from building all three designed examples against the shipped
crates and recording what hurt.

**Method, and why it matters.** Each example was implemented twice: once in Rust using **only APIs
that exist today**, and once in Python from published formulas. The two agree on all 16 checked
quantities. That agreement is what licenses the rest of this note: the friction listed below is
friction in *expressing correct physics*, not the residue of getting the physics wrong.

The scratch implementations and the Python reference were deleted after the comparison. They were
instruments, not deliverables. This note is what they produced.

**Status.** Design, not a proposal. Supersedes the speculative sketch in
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md) §4, which guessed at a pipeline before any of it had
been written.

---

## 1. What was built, and what it agreed with

| Example | Reference source | Checks | Result |
|---|---|---|---|
| Calibration counterfactual | generalised Rabi formula; depolarising channel | 7 | exact |
| Crosstalk attribution | Markov equivalence (Verma & Pearl 1990) | 5 | exact |
| Geometric QEC | Kitaev (2003); Betti of `Tᵈ`; intersection form | 4 | exact |

Concretely: the three faults read `0.980147 / 0.980159 / 0.980150` at one pulse and
`0.086460 / 0.961136 / 0.847242` at nine, purity `0.741154`; the three causal structures all fit
`P(e₁)=P(e₂)=0.10, P(e₁,e₂)=0.04` and the cyclic fourth is rejected as a `C₃`; the toric family
comes out `[[8,2,2]]` to `[[50,2,5]]` with Betti `[1,4,6,4,1]` on `T⁴`, pairings `±L²` and triple
products `8, 27, 64`.

Two independent routes reaching the same numbers, one through `DensityMatrix`/`apply_kraus`/Born and
one through closed-form Rabi, is the strongest evidence available that the substrate is sound. **The
problem QCL solves is not correctness. It is that correct code is unreasonably hard to write.**

---

## 2. The friction, as observed

Each item below is something the scratch code actually had to do.

### F1. Operators are hand-packed tensors

Every gate, projector and Kraus operator was built as a flat `CausalTensor` with a manually supplied
shape. Building the rotation for a detuned pulse meant writing the four complex entries of
`cos(θ/2)I − i sin(θ/2)(nₓX + n_zZ)` by hand.

> There is no rotation, no Pauli, no standard-basis projector reachable as a value.

### F2. Unitary evolution has to masquerade as a channel

The only way to conjugate a state by a unitary is `apply_kraus(&[u], rho)`. It works and it is
correct, but the reader sees a channel where the physics is evolution, and the CPTP machinery runs
on every step to re-establish what a unitary guarantees by construction.

### F3. The validated type is abandoned to compute

The loop is `DensityMatrix::from_ket(..).matrix().clone()` → raw tensor arithmetic → re-wrap in
`DensityMatrix::new`. The type validates Hermiticity, positivity and trace, then hands out its
interior so the caller can work, then re-validates. Validation happens `n` times for a state that
was valid throughout.

### F4. Every stage of a causal flow costs four lines of ceremony

```rust
.bind(|_: CausalEffect<()>, st: Diag, cx: Option<()>| {
    let mut st = st;
    /* one line of actual work */
    let next = CausalEffectPropagationProcess::pure(true);
    CausalEffectPropagationProcess::with_state(next, st, cx)
})
```

The turbofished closure signature and the `with_state` re-wrap appear at every stage, and the value
channel carried `true` in both stages because nothing needed it.

### F5. Shot noise is absent, so decisions are float comparisons

Both discriminating examples adjudicate with `min_by(|a, b| (a - measured).abs().cmp(..))`. That is a
nearest-neighbour test on exact reals. Turning it into the statistical decision it should be requires
hand-building a `CountHistogram`, filling it, and bridging to `Uncertain` — so the scratch code, like
anyone in a hurry, simply did not.

### F6. A declared causal structure cannot be evaluated

This is the largest gap. `CausalStructure` validates: it found the `C₃` in the cyclic hypothesis and
rejected it with the offending inputs and outputs named. But nothing turns a structure into an
interventional prediction, so `predict(hypothesis, experiment)` was a **hand-written lookup table**.
The crate screens hypotheses it cannot evaluate.

### F7. The experiment designer does not exist

Bhattacharyya scoring and the minimum-cost cover were written by hand in the example, including an
exhaustive bitmask over experiment subsets. Both are general; neither is in the crate.

### F8. The topology-to-gate seam is unchecked, and silently wrong

The decisive finding. A logical operator is a cochain: `Vec<f64>` indexed by edge. A Haruna gate
wants a `CausalMultiVector` gauge field. **There is no conversion**, so the scratch code packed the
cochain's sum into a multivector of unrelated dimension and called `logical_z`. The result:

```
logical_z failed: Haruna gate exp Taylor series did not converge within the
64-term budget to tolerance 1e-12; the exponent norm is too large
```

The failure is a *numerical* complaint about magnitude. Nothing checked that the multivector was the
gauge field of that cochain, that its metric matched the complex, or that its grading meant anything.
A caller who normalised the value to make the series converge would have got a confident, meaningless
answer.

---

## 3. What QCL is

QCL is the layer that makes the above unnecessary. Three groups, in dependency order.

### 3.1 Carriers — close F1, F2, F3, F8

| Type | Replaces | Contract |
|---|---|---|
| `QubitOperator` | hand-packed 2×2 tensors | named constructors: `pauli_x`, `rotation(axis, angle)`, `phase(θ)` |
| `Channel` | a bare Kraus slice | CPTP checked **once**, at construction |
| `QuantumPlant` | state + channel juggling | a validated state that evolves in place; `step`, `evolve(n)` |
| `Observable` | ket → `Projection` → Born | a named projector carrying its own read-out |
| `GaugeField` | the unchecked F8 seam | built **from a cochain and its complex**, so degree and metric are checked at the boundary |

Each carrier is generic over its scalar under the bound §3.4 assigns it.

`GaugeField` is the one that matters most. It must be constructible only as
`GaugeField::from_cochain(&complex, &cochain, degree)`, so the conversion that silently succeeded in
the scratch code becomes the only conversion available, and it validates.

### 3.2 Evidence — closes F5

`ShotBudget { shots, seed }` with `sample(&Observable, &QuantumPlant) -> Uncertain<FloatType>`. One
call, histogram built internally.

This is not a convenience. `Uncertain<f64>` already implements `Verdict`, so a sampled reading is a
lawful lattice carrier the moment it exists, and every downstream decision becomes statistical rather
than a float comparison. F5 exists purely because the ergonomic path today is the wrong one.

### 3.3 Reasoning — closes F6, F7

- **`Hypothesis`** — a named candidate binding a `CausalStructure` to the plant modification it
  implies. This is the missing link: today the validated structure and the thing being discriminated
  are separate universes joined by an enum.
- **`intervene(&Hypothesis, &Intervention) -> QuantumPlant`** — the operation F6 wants. A structure
  plus a `do(...)` yields a plant whose predictions can be *computed* rather than tabulated.
- **`design(&[Experiment], objective) -> DesignPlan`** — Bhattacharyya scoring and minimum-cost
  cover, returning a plan rather than an experiment (established by the crosstalk example, where no
  single cheap experiment separates all three hypotheses and `{E1, E2}` at cost 2 beats tomography at
  200).
- **`adjudicate(...) -> Adjudication`** — `Abduced` / `Ambiguous` / `NoneAdmissible`. `Ambiguous`
  carries the binding pair and the shots that would resolve it.

---

### 3.4 Scalars: precision as a parameter, bounds as documentation

QCL inherits the project's standing commitment that
[numerical precision is a parameter, not an assumption](../../../website/docs/src/content/docs/concepts/uniform-math.md).
Every QCL type is generic over its scalar, a program fixes one alias, and the
whole pipeline instantiates at that precision:

```rust
pub type FloatType = Float106;   // or f64, or f32
```

That is the easy half. The half worth designing is **which bound each surface
carries**, because `deep_causality_algebra` ships a tower precisely so that a
signature can say what an operation actually needs.

#### The bound each QCL surface should carry

| Surface | Operations it performs | Bound | Why not tighter or looser |
|---|---|---|---|
| Cochains, cup product | `+`, `−`, `×`, `0` | `CommutativeRing + Copy` | No division, no ordering, no analytic call. `RealField` would exclude coefficient rings a cohomology computation may want |
| Gauge field, operators | complex arithmetic | `ComplexField<R>`, `R: RealField` | Operator entries are genuinely complex; the real parameter carries the precision |
| Rotations, gate synthesis | `sin`, `cos`, `sqrt`, `π`, normalisation | `RealField` | Needs the analytic surface of `Real` **and** division from `Field`; this is the one place the full bound is honest |
| Born read-out, purity | real output compared to a spec | `Real` | Ordering yes, division no. `Real` is exactly "ordered and analytic" without field structure |
| Verdicts | bounded lattice on `[0,1]` | `Prob` | Already an MV-algebra `Verdict`; QCL adds nothing |
| Shot statistics, Bhattacharyya | `sqrt`, `log2`, ratios | `RealField` | Genuinely needs both halves |
| Costs, shot counts, cover search | integer arithmetic | none | `usize`. A scalar bound here would be noise |

The discipline is one line: **bound at the weakest structure that carries the
operation.** A signature is the cheapest documentation in the crate, and an
over-tight bound is a false statement about what the code does.

#### The worked example, verified

The cup product that shipped in `add-cup-product` was bound on `RealField`. It
adds, subtracts, multiplies and needs a zero. It never divides, never orders and
never calls an analytic function, so the bound was three levels too tight
(`RealField → Field → CommutativeRing`) and additionally dragged in the whole
`Real` analytic surface.

Relaxing it to `CommutativeRing + Copy` was tried rather than argued: the crate
compiles, the workspace compiles, and all 1471 tests pass unchanged. The
relaxation is non-breaking, since every type that satisfied the old bound
satisfies the new one, and it admits coefficient rings the old bound refused.

That is what "as appropriate" means in practice, and it is the pattern QCL
should follow from the start rather than tighten-then-discover.

#### Precision as a functor, not only as an alias

An alias makes precision a **compile-time** choice: edit the line, rebuild, and
the program runs at the new scalar. `deep_causality_haft` makes a stronger form
available.

`CausalTensor` implements `fmap<A, B>`, which maps `CausalTensor<A>` to
`CausalTensor<B>`. The map changes the *scalar type*, not merely the values. Any
QCL carrier built on tensors inherits that, so a plant, an operator or a cochain
can be lifted from one precision to another as an **operation**:

```rust
let plant_f64: QuantumPlant<f64> = ...;
let plant_hi: QuantumPlant<Float106> = plant_f64.to_precision();
```

This turns the precision ladder from a rebuild into a computation. The same
program can carry both, run a discrimination at each, and report the difference,
which is exactly the experiment the capstone spinor example runs to show `f64`
drift at `1.1e-16` against `Float106` at `1.7e-31`.

For QCL that is not a curiosity. Every example in §1 makes a decision by
comparing numbers: a spec threshold, a Bhattacharyya separation, a homology-class
invariance residual. Whether such a comparison is precision-limited is a question
the pipeline should be able to **answer about itself**, by running the same
adjudication at two scalars and reporting whether the verdict moved. A verdict
that changes with the scalar was never a verdict about the physics.

#### What this does not license

Genericity has a cost and the tower makes it easy to overpay:

1. **No bound wider than the operation.** A function taking `CommutativeRing`
   must not later reach for `sqrt` and force every caller up to `RealField`.
2. **No scalar parameter where there is no scalar.** Cover search, shot counts
   and cell indices are integers. `QclConfig<R>` should not thread `R` into them.
3. **`Real` and `RealField` are not synonyms.** `Real` is ordered and analytic;
   `RealField` adds division. Reaching for the latter by habit is how the cup
   product ended up over-constrained.
4. **Complex is not a precision.** `ComplexField<R>` carries its precision in
   `R`. The alias a program fixes is the real scalar; complex structure is a
   separate axis.

---

### 3.5 The ledger — cost and shot accounting in the State channel

Device time is layer 5's scarce resource, so the run has to meter it. The carrier decides where:
Context is read-only and cannot accumulate, `EffectLog` is a record rather than a running total, and
Value is the answer rather than the meter. **State is the only channel that is both writable and
threaded**, so the ledger goes there by elimination.

That also explains a split reached by iteration in §4.3: the budget accumulates, so it is State; the
floor is fixed, so it is a stage parameter. `design(MinCostCover { floor_bits })` reads spend from
State and compares it against a constant from the call site.

```rust
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::Zero;

/// Cost and shot accounting, threaded in the State channel.
///
/// Counts are integers: `deep_causality_algebra` deliberately excludes the integers from the
/// trait tower, and counting shots in floating point would be wrong regardless. The continuous
/// quantities carry the scalar parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ledger<R> {
    shots: u64,        // taken on the device
    experiments: u32,  // executed on hardware
    predictions: u32,  // model evaluations — tracked, never billed
    device_time: R,    // accumulated, measured
    cost: R,           // what MinCostCover minimises
    bits: R,           // separation achieved so far
}
```

`Real` is `Copy`, so `Ledger<R>` is `Copy` for every admissible `R`. It holds no `Vec` and no
`String`, so it survives the bare-metal path of `positioning.md` §5.3 unchanged — the ledger is not
what blocks the fast loop; the log is.

**`Default` is hand-written, not derived.** `CausalMonad::pure` requires `State: Default`, and
`#[derive(Default)]` would impose a spurious `R: Default` bound. `Real` reaches `Zero` through
`CommutativeRing → Ring`, so the zero ledger is written from `R::zero()` and the bound stays `Real`.

**The bound split follows the trait tower.** Accumulating, comparing against a budget, and computing
a remainder need `+`, `-` and ordering, all of which `Real` provides:

```rust
impl<R: Real> Ledger<R> {
    pub fn zero() -> Self { /* R::zero() in each continuous field */ }
    pub fn record_shots(&mut self, n: u64, elapsed: R, cost: R) { /* += */ }
    pub fn record_prediction(&mut self) { self.predictions += 1; }
    pub fn within(&self, budget: R) -> bool { self.cost < budget }
    pub fn remaining(&self, budget: R) -> R { budget - self.cost }
}
```

Derived ratios divide, so they need field invertibility and sit on a separate block:

```rust
impl<R: RealField> Ledger<R> {
    pub fn cost_per_bit(&self) -> R { self.cost / self.bits }
    pub fn bits_per_shot(&self) -> R { /* … */ }
}
```

Bounding accumulation on `Real` rather than `RealField` is not pedantry. `Real` admits analytic
non-field types — the dual numbers used for forward-mode differentiation — so a cost model can be
differentiated with respect to its parameters while still threading through the same ledger. Binding
the whole struct on `RealField` would forfeit that for the sake of two ratio methods.

#### Three invariants

**Increment only at the device boundary.** Example 1 forks into three hypothesis worlds, but one
experiment runs on hardware and the other two are model predictions. A ledger incremented in every
world reports three times the true device time. `observe` is the only stage that touches `shots`,
`experiments` and `device_time`; `predict`, `design`, `fork` and `adjudicate` touch at most
`predictions`.

**Forking is QCL's, not core's.** `branch`, `branch_with` and `either` route to *one* arm and move
the state; the flow stays linear, and `alternate_value` is value substitution rather than Pearl's
`do(...)`, which `intervene.rs` states directly — at the value level there is no graph to cut. So
`fork` is built above core by cloning the carrier. `State: Clone` is deliberately not required (the
sliding-window state in `corrective_ddos_detector` is not `Clone`), and a `Copy` ledger sidesteps
that, but a plant state pairing ρ with the ledger needs ρ to be `Clone` for forking to work.

**Do not merge forked ledgers with ∇.** `Ledger<R>` under field-wise addition is a commutative
monoid, so `SymMonoidal::merge` will typecheck and give the wrong answer. Summation is correct for
*sequential* composition, which `bind` already performs by threading. At a *counterfactual* fork
exactly one branch was factual, and the merged ledger is that branch's. If the monoid instance
exists it should be named for sequential accumulation, so nobody reaches for it at a fork.

---

## 4. Configuration and the pipeline

### 4.1 One origin for configuration

Every configuration in QCL comes from `QclBuilder::config()`. There is no other constructor, no
free-standing struct literal, and no stage that accepts loose parameters standing in for a config.
The builder is also the single site where the scalar type is named:

```rust
QclBuilder::config::<FloatType>()
```

Every bound in §3.4 — `Real`, `RealField`, `ComplexField<R>`, `CommutativeRing` — is discharged from
that one parameter. Swapping `FloatType` for `Float106` re-types the whole run, and no second site
can disagree with the first.

### 4.2 The config branches on the subject

A config is built *over* what the model is about, and that choice decides which pipelines are
reachable:

| Constructor | Subject | Candidates are | Reachable pipelines |
|---|---|---|---|
| `.over_plant(p)` | a system that evolves and is measured | `Hypothesis` | `validate`, `control` |
| `.over_code(c)` | a complex, evaluated exactly | candidate complexes | `validate` |

The split is observed, not imposed: the geometric-QEC example never measures anything, and the two
device examples never evaluate a chain complex. Because the candidate type follows the subject,
"hypothesis" keeps one meaning instead of two.

`build()` returns a `Result`. It is where cross-field validation lives: a probe naming an observable
the plant does not expose, a zero shot count, an empty candidate set.

### 4.3 What the config holds, and what it does not

**The config holds what several stages share**, or what must be validated together: the subject, the
evidence policy, the baseline experiment, the probe family, the candidates.

**A stage's parameter belongs to the stage.** `spec` goes to `.gate(spec)`, the selection objective
to `.design(objective)`, the depth to `.evolve(n)`. This is not cosmetic. When `spec` was a required
config field, the crosstalk example had to invent one to satisfy the builder — a field existing only
to keep the type honest, which is the same defect as an all-optional config wearing a better
disguise. Parameters are now written once, where they are read.

### 4.4 The hand-off is a type

`validate` terminates in `Screened<R>`, which carries the config together with the admitted subset.
`control` accepts either a plant config or a `Screened<R>`. A config carrying **structural**
candidates therefore has no path into `control` that skips validation, while a config carrying only
mechanism candidates — which declare no structure, so there is nothing to screen — needs no extra
step. The ordering is a fact about the types rather than a convention in the prose.

### 4.5 The three examples

**Calibration counterfactual.** Mechanism candidates, so `control` directly.

```rust
let cfg = QclBuilder::config::<FloatType>()
    .over_plant(transmon)
    .evidence(Evidence::shots(1024).seed(20260821))
    .baseline(Experiment::probe("check", excited_population, 1, cost = 1))
    .probes(&amplification_family)                      // depths 1..40
    .candidates(&[
        Hypothesis::mechanism("amplitude",   amp_drift),
        Hypothesis::mechanism("detuning",    det_drift),
        Hypothesis::mechanism("decoherence", depolarising),
    ])
    .build()?;

QclBuilder::control(&cfg)
    .observe()                                          // runs the baseline -> Uncertain
    .gate(Spec::at_least(ft(0.999)))                    // 0.9801 fails, loop proceeds
    .fork()                                             // one world per fault
    .design(MinCostCover { floor_bits: ft(5.0) })       // picks depth 9
    .predict()
    .adjudicate()
    .finalize().print_results();
```

**Crosstalk attribution.** Structural candidates, so validation is unavoidable.

```rust
let cfg = QclBuilder::config::<FloatType>()
    .over_plant(two_qubit)
    .evidence(Evidence::shots(1024).seed(20260821))
    .baseline(Experiment::probe("passive", joint_error, 1, cost = 1))
    .probes(&[do_q1, do_q2, echo_both, process_tomography])
    .candidates(&[
        Hypothesis::structural("Q1->Q2", s_d12, ext_d12),
        Hypothesis::structural("Q2->Q1", s_d21, ext_d21),
        Hypothesis::structural("common", s_com, ext_com),
        Hypothesis::structural("cyclic", s_cyc, ext_cyc),
    ])
    .build()?;

let screened = QclBuilder::validate(&cfg)
    .check_markov()
    .check_faithfulness()                               // rejects the cyclic C3
    .finalize();                                        // -> Screened<FloatType>

QclBuilder::control(screened)                           // unreachable unscreened
    .observe()
    .gate(Spec::uncorrelated())
    .fork()
    .design(MinCostCover { floor_bits: ft(5.0) })
    .predict()
    .adjudicate()
    .finalize().print_results();
```

**Geometric QEC.** A code subject, so `validate` only, and the config is short because there is
little to validate across fields.

```rust
let cfg = QclBuilder::config::<FloatType>()
    .over_code(LatticeComplex::<2, FloatType>::square_torus(4))
    .build()?;

QclBuilder::validate(&cfg)
    .derive_code()                                      // [[32,2,4]]
    .check_ldpc_weights()
    .check_class_invariance()                           // closes F8
    .finalize().print_results();
```

The code pipeline has no probe family. Its selection over candidate complexes is scored against
requirements, not against evidence, so it is a different computation that merely shares a shape with
`design` (established in [`qcl-dsl-liftback.md`](qcl-dsl-liftback.md) §8.9). v1 does not unify them.

### 4.6 Why this shape and not the three that preceded it

Four config designs were written and each was scored against §2. `Cfg-sens` marks whether the config
shape can move the score at all — F1–F3 are settled by the carriers however the config is spelled.

Scale: 0 not addressed, 1 partly, 2 addressed, 3 settled at the type level.

| Friction | Cfg-sens | One config, all optional | Three shapes | Subject-keyed | **This design** |
|---|---|---|---|---|---|
| F1 operators hand-packed | no | 2 | 2 | 2 | 2 |
| F2 unitary as channel | no | 2 | 2 | 2 | 2 |
| F3 validated type abandoned | no | 2 | 2 | 2 | 2 |
| F4 stage ceremony | yes | 1 | 2 | 2 | **3** |
| F5 shot noise absent | yes | 1 | 2 | 2 | **3** |
| F6 structure not evaluable | yes | 1 | 1 | 1 | **3** |
| F7 designer missing | yes | 1 | 1 | 1 | **3** |
| F8 topology-to-gate seam | yes | 0 | 1 | 2 | 2 |
| **friction subtotal / 24** | | 10 | 13 | 14 | **20** |
| precision as a parameter / 3 | yes | 1 | 2 | 3 | **3** |
| **total / 27** | | 11 | 15 | 17 | **23** |

Where the movement comes from:

- **F4** rises on the last step not through line count — the last two designs are within a line of
  each other — but because no line exists solely to satisfy the builder.
- **F5** is skippable while evidence is optional, which is exactly how the friction arose in the
  scratch code. Making `Evidence` the sole source of shot counts on every plant config closes it.
- **F6 and F7** are the two that the first three designs all failed. `Screened<R>` settles the
  screening half of F6; splitting the probe family (shared) from the objective (per stage) settles
  F7, since one family is scored under several objectives.
- **F8** caps at 2 rather than 3. `over_code` makes the code checks reachable and the plant checks
  unreachable, but the gauge-field conversion is a carrier obligation under §3.1, so no config shape
  can settle it outright.
- **Precision** reaches 3 once the scalar is named at exactly one site. The first design named it at
  each of three entry points, where two of them can drift.

**The DSL adds ordering and naming, not a monad.** `PropagatingEffect` already short-circuits and
carries an `EffectLog`; every stage returns one. If a `QclEffect` type appears, something has gone
wrong. F4 is solved by the stage methods hiding the closure ceremony, not by a new effect system.

---

## 5. What QCL must not do

1. **Own graph traversal.** That is `deep_causality`'s `CausaloidGraph` and `ultragraph`.
2. **Own topology.** Chain complexes, Betti numbers and the cup product are
   `deep_causality_topology`'s. QCL names their QEC meaning and calls them.
3. **Own the verdict lattice.** `Verdict` is `deep_causality_algebra`'s.
4. **Model devices.** The rotation-with-detuning plant belongs to an example.
5. **Claim fault tolerance.** QCL computes logical actions. Constant-depth fault-tolerant circuits
   are a compiler, and a separate change.

---

## 6. Sequencing

0. **Fix the scalar bounds before writing carriers** (§3.4). It costs nothing, it is the cheapest
   documentation in the crate, and every layer below inherits whatever the carriers declare. The
   cup-product relaxation shows the cost of the other order: tighten by habit, discover later.
1. **Carriers next** (§3.1). They close five of the eight frictions and need nothing else. Start
   with `GaugeField::from_cochain`, because F8 is the one that produces confident wrong answers
   rather than merely verbose code.
2. **`ShotBudget`** (§3.2). Small, and it changes every decision downstream from a float comparison
   into a statistical one.
3. **`Hypothesis` and `intervene`** (§3.3). The largest piece, and the one that makes the crate able
   to evaluate what it already validates.
4. **`design` and `adjudicate`.** Both general, both currently hand-rolled in examples.
5. **`QclBuilder::config`** (§4.1–§4.3), then the stages (§4.5), last — once at least two
   examples run against the layers beneath. Config comes first within this step because it is
   the sole origin of configuration and the single site naming the scalar; stages read it.

The failure mode to avoid is unchanged from `qcl-dsl-liftback.md` §7: writing the pipeline first and
shaping examples to justify it. The ordering above is the reverse, and it is now grounded in three
programs that compiled and produced verified numbers.

---

## 7. Benchmarks: what "normal" means, and how far off we are

QCL makes two performance claims that nothing currently measures: that a tick is cheap enough for a
control loop, and that precision is a parameter. Neither should be stated without numbers, and both
have published reference points to be measured against. `criterion` with `harness = false` is the
repo's existing convention.

Every figure carries the machine. The workspace reference is **M3 Max, 16 cores, 128 GB**; bare-metal
figures carry the board instead.

### 7.1 Reference points

| Quantity | Published value | Source |
|---|---|---|
| Superconducting code cycle | 0.2–10 µs | arXiv:2108.12371 |
| Real-time decode, per round | sub-1 µs mean | Nature Comms (2026) |
| Feedback latency, superconducting | 9.6 µs | Nature Comms (2026) |
| Trapped-ion shuttling code cycle | ~235 µs | arXiv:2108.12371 |
| Crosstalk detection, experiment count | O(n²)–O(n³) | Sarovar et al. (2020) |

### 7.2 Tier 1 — tick latency, the layer-6 question

| Benchmark | Measures |
|---|---|
| `bind` on `PropagatingEffect<FloatType>` | the floor: one stage, unit state, no context |
| `bind` on `PropagatingProcess<Rho, Ledger<FloatType>, Config>` | a realistic carrier |
| the same, log enabled vs bounded vs off | isolates the per-tick `String` allocation of §5.3 |
| a full monitor→gate→`alternate_value` tick | the `corrective_ddos_detector` shape |

Read against 235 µs and against 0.2–10 µs. These are the two numbers that decide whether the layer-6
position in `positioning.md` §5.1 is a position or a footnote.

**Allocations per tick matter more than wall clock here.** A bounded-time claim needs
allocations/tick = 0, which a timing harness does not show. Count them separately with a counting
allocator; a fast mean with an unbounded tail is the failure mode this is looking for.

### 7.3 Tier 2 — stage cost, the layer-5 question

`validate`, `design`, `predict` and `adjudicate` run once per calibration decision, so the budget is
seconds, not microseconds. What matters is that nothing explodes.

| Benchmark | Watch for |
|---|---|
| `check_markov` + `check_faithfulness` per hypothesis | scaling in qubit count against Sarovar's O(n²)–O(n³) |
| `design` — min-cost cover, k experiments × n hypotheses | **exponential in k**: exhaustive bitmask over subsets |
| `predict` — interventional evaluation per hypothesis | linear, and small |
| `adjudicate` — Bhattacharyya plus decision | linear in hypothesis count |

`design` is the one with a cliff. Sweep k from 4 to 20 and publish where exhaustive enumeration stops
being usable; that number decides whether a heuristic cover is needed and when.

### 7.4 Tier 3 — the price of precision

Every Tier 1 and Tier 2 benchmark, repeated at `f32`, `f64` and `Float106`. Precision as a parameter
(§3.4) is a claim about what a caller may choose; it is honest only if the cost of each choice is
published. `Float106` in particular is expected to be expensive, and the number belongs next to the
claim rather than in a reader's assumptions.

### 7.5 Tier 4 — the bare-metal figure

Once the port of `positioning.md` §5.4 lands, Tier 1 repeated on an aarch64 board with a bounded log,
reported against the same reference points. Until then Tier 1 runs host-side and is an upper bound on
what the hardware would do, not a substitute for it.

### 7.6 What these benchmarks must not do

They do not compare QCL against Qiskit Experiments, Qibocal or Stim on shared work — the pipelines
compute different things, and a same-axis comparison would be measuring the difference in task.
Stim's decoding throughput is the right reference for Stim's job and is not QCL's job. What is being
measured is distance from a published physical budget, not a ranking.

---

## 8. Sources

- `website/docs/src/content/docs/concepts/uniform-math.md` — the project's standing position on
  precision as a parameter, the algebraic trait floor, and the witness pattern §3.4 builds on.
- Nielsen, M. & Chuang, I. *Quantum Computation and Quantum Information* — generalised Rabi
  formula and the depolarising channel, behind example 1.
- Verma, T. & Pearl, J. (1990). *Equivalence and synthesis of causal models*; Pearl, *Causality*,
  2nd ed., ch. 1 — Markov equivalence, behind example 2's structural degeneracy.
- Kitaev, A. (2003). *Fault-tolerant quantum computation by anyons*, Ann. Phys. **303**, 2–30 — the
  toric code parameters in example 3.
- Chen, Y.-A. & Tata, S. (2023). arXiv:2106.05274 — the cup product the pairings run through, in
  `deep_causality_topology/papers/`.
- Haruna, J. (2025). arXiv:2511.15224 — the logical gates example 3 reaches for, in
  `deep_causality_quantum/papers/`.

Example designs: [`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md),
[`example-geometric-qec.md`](example-geometric-qec.md). Prior sketch:
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md).
