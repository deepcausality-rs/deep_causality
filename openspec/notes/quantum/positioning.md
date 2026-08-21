<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Dynamic Quantum Causality: Positioning

**What this is.** The positioning document for `dqc.deepcausality.com`, the planned project
subpage for `deep_causality_quantum`. It names the audience the site is written for, the problems
that audience has, and where the crate stands against them today. It is the input to a site build,
not site copy.

**Status.** Draft for review, revision 3. Revision 1 positioned the crate as a validator and a
seam. Revision 2 made classical-quantum control the frame. Revision 3 adds published evidence for
each challenge and gives the Haruna logical gates their own avenue.

**Honesty convention**, as elsewhere in these notes: **[ships]** marks behaviour in the crate today
with a test or a runnable example behind it; **[planned]** marks work with a written specification
and no code; **[open]** marks a research question with no answer.

---

## 1. The position in one line

> A quantum device is a plant. Something classical has to close the loop around it, and that
> classical part is where the system fails. Dynamic Quantum Causality is that loop written as one
> causal process: it validates the quantum causal structure before the run, monitors and intervenes
> during it, and records what every correction rested on.

For the hero, shorter: **the quantum part ran; the loop around it is what breaks.**

The crate is not a simulator and not a compiler. It is a controller, a gate constructor, and a
validator, sharing one substrate.

---

## 2. The primary audience

### 2.1 The prime persona

**The engineer who owns a control loop with a quantum element inside it.**

Every serious quantum system in operation today is a classical-quantum control system. Something
measures, something classical decides, something acts, and the loop closes. Name the loop and the
audience names itself:

| The loop | Plant | Controller | Correction |
|---|---|---|---|
| Error correction | logical qubit under noise | decoder | Pauli frame update |
| Calibration | drifting device | characterization routine | pulse or frequency retune |
| Variational optimization | parameterized circuit | classical optimizer | new parameters |
| Quantum sensing | clock, gravimeter, magnetometer | estimator and fusion filter | navigation correction |
| Measurement feedback | cavity or trapped ion | real-time controller | conditional pulse |

Five domains, one shape. A monitor reads state; an envelope test decides whether the state has
drifted out of bounds; an intervention snaps it back; the process continues from the corrected
state. That shape is what the substrate under this crate already implements, with five worked
examples driving classical plants and one driving a qubit.

Three groups sit inside the frame, ranked by how much of the site should speak their language.

**P1. Quantum error correction and logical-gate design.** They own the fastest and least forgiving
of these loops, and they also own the encoding the loop protects. They work on qLDPC and CSS
codes, and they think in chain complexes and homology classes whether or not they say so. Their
first pain is that the decoder assumes independent errors and the device does not oblige. Their
second is that logical gate construction is per-code-family craft. Their vocabulary is the site's
vocabulary.

**P2. Device and noise characterization.** They own the calibration loop, and they own the model
every other loop depends on. Process tomography, process tensors, crosstalk and leakage budgets,
non-Markovianity. Their pain: they can measure that two error channels are correlated and cannot
say whether one drives the other or both hang off a shared bath. That distinction changes what the
hardware team does next, and a correlation weight does not carry it.

**P3. Quantum causal model researchers.** The Allen–Barrett–Lorenz–Oreshkov line, and the Vienna
indefinite-causal-order community. Small, formally exacting, and the group the crate serves
*completely* today: the quantum Markov condition and the C₃-exclusion faithfulness criterion both
run as code. They will not become a market. Their reading of the artifact decides whether P1 and P2
take it seriously.

The site is written in P1's language, aimed at P2's daily problem, and built to survive P3's
review. That ordering matters, and it should be visible in the copy without being stated.

### 2.2 Who this is not for

Say this early on the site. It costs one section and saves every misdirected reader.

- **Anyone shopping for a faster simulator.** Stim, Qiskit Aer and QuTiP do that, and do it better.
  The dense operator kernels here exist to make checks decidable, not to scale.
- **Anyone expecting a quantum speedup.** Nothing in the crate computes faster because it is
  quantum. Speedup, when it exists, lives inside a single causal function, behind the seam.
- **Anyone needing a microsecond real-time decoder.** Syndrome rounds run on dedicated FPGA or ASIC
  hardware. This is the supervisory layer above such a loop; see boundary 8.
- **Anyone needing fault-tolerant gate guarantees today.** The gauge-field construction the crate
  implements explicitly relaxes fault tolerance and depth optimality; see boundary 10.
- **ZX-calculus and categorical-quantum-mechanics users.** That formalism reasons about the
  internals of a circuit. This one reasons about the process a circuit sits inside.
- **Anyone wanting a Qiskit replacement.** There is no transpiler, no pulse layer, no vendor
  backend. The `qpu` feature is a typed boundary with an in-process simulator behind it.

### 2.3 How this audience differs from the CFD audience

The CFD site is written for a practitioner with a deadline who must defend a number to a chief
engineer. It leads with validation tables and wall-clock figures because that reader buys evidence.

This reader buys *provenance of reasoning*. They read arXiv daily, they have seen four years of
quantum marketing, and their first move against any claim is to look for the assumption it hides. A
benchmark does not move them; a stated boundary does. Three consequences for the build:

1. **Every claim carries a citation or a theorem id.** Paper, arXiv number, or a `THEOREM_MAP` row.
   A claim without one reads as marketing to this reader and does damage.
2. **The proof artifact is a proof, not a wall clock.** Where the CFD site opens on a 44-second run,
   this site opens on a freeze that *aborts* and names the pair of operators responsible.
3. **The refuted theorem goes near the front.** The project's own roadmap proposition,
   `partial_trace_preservation`, is false; the counterexample is proved in Lean and a feature was
   dropped because of it. That is the single most credible thing the crate owns, and it is exactly
   the kind of thing this audience checks for and almost never finds.

---

## 3. The challenges

Each current challenge below is backed by published work, cited in §12. Three papers carry most of
the weight, and they are recent and from inside the field rather than from its commentary.

### 3.1 Current

#### The loop

**C1. The loop has to close in real time, and the classical half is the constraint.**
Battistel et al. state the requirement plainly: the decoder "needs to process the syndrome data at
the same rate as it is received or close to it, i.e. in real time, to avoid an exponential slowdown
of the computation, known as the *backlog problem*." For superconducting transmons the benchmark is
roughly one microsecond per QEC cycle, and they note that throughput matters even more than latency.
This is a hard real-time control problem stated by QEC researchers as such, and their survey frames
system integration, not qubit quality, as what stands between today and fault-tolerant operation.

The engineering consequence is the one this crate addresses: the plant model, the correction policy
and the safety envelope live in three tools with three type systems and no shared notion of what an
intervention is. When the loop misbehaves, there is no single object to inspect; the engineer
reconstructs the run from three logs with three clocks.

**C2. Correlated noise is a causal question filed under statistics.**
Matching and belief-propagation decoders are built on independent error mechanisms. Real devices
deliver crosstalk, leakage, drift and shared-bath coupling. Kam, Gicev, Modi, Southwell and Usman
measured what that costs on surface-code memory and found that the *structure* of the correlation
decides the damage: multi-time "streaky" correlations on syndrome qubits and two-qubit gates
severely degrade logical error rate scaling, while other temporal correlations are comparatively
benign. That is a statement about causal structure, and the field's standard representation, a
correlation weight in a detector error model, cannot carry it. A weight does not separate a direct
cause from a common cause. The engineer learns that two detectors fire together and learns nothing
about which intervention would stop it.

**C3. The classical scaffolding dominates the run, and nothing manages it.**
Mesman et al. profiled a QAOA workload end to end on a transmon stack and found the execution
profile dominated by classical control infrastructure and synchronization rather than by gate
operations. Fixing qubit reset bought 1.40×; parallelizing control-module initialization bought
1.37×. Around every variational loop sits queueing, calibration drift, job failure, shot budgeting
and mid-circuit classical control, rebuilt per project as imperative glue with no error algebra, no
typed failure, and no record of what happened.

#### The encoding

**C4. Logical gates for general CSS codes have no systematic construction.**
Haruna states the gap in the paper this crate implements: existing approaches, transversal gates,
lattice surgery, code deformation and magic-state distillation, "typically apply only to specific
code families or rely on particular geometric structures. A general and systematic framework for
constructing logical gates for general QEC codes remains lacking." Within the gauge-field formalism
specifically, he notes that a unified treatment of non-diagonal gates such as Hadamard is still
missing, and that a systematic procedure for decomposing logical gates into physical operations has
not been fully established. This is a live gap in a fast-moving corner of QEC, published in
November 2025.

#### The evidence

**C5. Quantum-derived evidence has no provenance.**
When a result rests on a measurement, almost nothing records which backend produced it, under which
calibration snapshot, over how many shots, and what classical post-processing intervened before the
number was believed. Reproducibility suffers. Any future certification argument that touches a
quantum device has no artifact to point at.

**C6. Causal inference tooling stops at classical relata.**
Pearl's machinery assumes classical variables. Quantum causal models exist as mathematics (Allen et
al. 2017, Barrett–Lorenz–Oreshkov 2019 and 2021, Lorenz 2022) with essentially no executable
implementation. A structure with both classical and quantum nodes is not something either side's
tooling will accept.

**C7. Verification stops at the circuit.**
SQIR/VOQC, CoqQ and Qbricks verify circuits and programs. The causal-model layer above them, where
process operators, the quantum Markov condition and faithfulness live, appears unformalized. The
layer holding the modelling assumptions is the layer with no proofs.

**C8. Structural validity is argued on paper.**
Two questions get settled by hand today. Is this factorization a legal quantum causal model? Can any
circuit realize this declared influence structure faithfully? Both are decidable, and both are
currently answered in prose in an appendix.

### 3.2 Emerging

**E1. Fault tolerance moves the design questions into topology.**
CSS and qLDPC codes are chain complexes; the boundary maps are the parity-check matrices; logical
qubits are homology classes. Logical gates arrive as cohomology operations (Hsin–Kobayashi–Zhu,
arXiv:2411.15848; Haruna, arXiv:2511.15224). The design question becomes: given an algorithm's gate
demand, which code topology serves it cheaply? That is a search over cell complexes, and the
instrument it needs is a gate catalogue computed from a Betti vector. None of the standard QEC
toolchains compute one.

**E2. Indefinite causal order becomes something to check.**
The quantum switch has moved from thought experiment to laboratory. Causal separability, meaning
whether a process admits any causal order at all, turns into a property an engineer wants tested
rather than proved by hand.

**E3. Quantum measurements enter safety cases.**
Optical clocks, gravimeters and magnetometers are heading into navigation and infrastructure
control. A quantum sensor becomes one input to a classical estimator whose output steers a vehicle.
Within the decade a quantum measurement will sit inside a certification argument, and there is no
accepted way to write that argument down.

**E4. The advantage question arrives with a budget attached.**
"Where does quantum actually pay?" is now asked by procurement. The honest answer is often no.
Tooling that can produce a reasoned no, with the causal and resource-level reason attached, is what
makes any yes bankable. Current resource estimators do not produce it; they price a circuit that has
already been chosen.

---

## 4. Where DQC stands

| Challenge | What the crate does | Status |
|---|---|---|
| C1 one loop, one formalism | Monitor, envelope test, intervention and continuation are one causal process on one substrate; every intervention lands in the effect log by construction | **[ships]** at supervisory rate |
| C4 general-CSS logical gates | Haruna's gauge-field construction implemented: logical `S`, `H`, `T`, `CZ` and the Pauli generators, as exponentials of gauge-field polynomials, with overflow and non-convergence surfaced as typed errors | **[ships]** |
| C8 structural validity | Quantum Markov condition as a freeze-time commutativity check over intersecting Hilbert supports; C₃-exclusion faithfulness at the same boundary | **[ships]** |
| C7 verification altitude | Lean 4 proofs of the partial-trace and Choi layer, each bound to an executable Rust witness; the false roadmap theorem refuted in Lean | **[ships]** |
| C3 hybrid scaffolding | The arity-5 causal monad supplies state, typed error, log and context; the `qpu` seam lifts a device call into it with an in-process simulator behind it | **[ships]** as an algebra; **[planned]** as a vendor integration |
| C5 provenance | The log channel records what a step rested on; the Effect Ethos gates an action on it | **[ships]** as channels; **[planned]** as a worked audit trail |
| C6 mixed relata | One hypergraph holds classical and quantum nodes, joined by the same hyperedges | **[ships]** structurally; the classical intervention calculus is **[planned]** (`deep_causality_do_calculus`) |
| C2 correlated noise | Complete common cause has an operator form (Allen et al. 2017): factorization into pairwise-commuting Choi factors. The freeze gate tests exactly that | **[ships]** as the check; the decoder is **[planned]** (Track D) |
| E1 topology | The Haruna gates and the **cup product** they are built from both run: binary and `n`-fold, generic over `ChainComplex`, simplicial and cubical. Steenrod squares and the Betti gate catalogue remain specified | **[ships]** through the cup product; **[planned]** for the catalogue |
| E2 indefinite order | The hypergraph is order-neutral and admits cyclic structure; superposition of orders needs a linear carrier | **[open]**, stated as out of scope |
| E3 safety cases | Quantum sensors feeding classical fusion under a deontic gate; the navigation link | **[planned]** |
| E4 advantage qualification | Counterfactual value attribution with SURD, then kernel extraction and resource estimation | **[planned]** |

Four rows carry the site: C1, C4, C8 and C7. They group into three avenues.

### 4.1 Avenue one: the loop is inherited, not built

`deep_causality_quantum` does not implement a control loop. It inherits one. The `AlternatableValue`
trait lives in `deep_causality_core`, the crate the quantum layer is built on, and it defines a
single operation: replace the value carried by an in-flight causal chain. That operation is the
intervention. State and read-only context survive it; an error state makes it a no-op, so a
correction cannot paper over an upstream fault.

Five worked examples drive that loop for classical plants: lane keeping under crosswind, a
closed-loop insulin pump, Bühlmann decompression stops, enterprise switch failover, and a
sliding-window DDoS detector engaging a rate limiter. Each runs the same chain twice, open loop and
closed loop, so the correction's effect is measured rather than asserted. Lane keeping, sixty ticks:

```
Open loop  : ticks=60  corrections= 0  max_|offset|= 3.65 m  outcome=OFF-ROAD at tick 24
Closed loop: ticks=60  corrections=12  max_|offset|= 0.37 m  outcome=stayed in lane
```

Every intervention appends to the effect log by construction, with the replaced value beside the
replacement:

```
tick  4: offset = +0.31 m [anomaly]
!!ValueAlternation!!: Value(0.3114814193869514) replaced with Value(0.046722212908042716)
```

The quantum error-correction example is that machinery with a Hilbert state on the channel. A
syndrome fires, the history carried in the state channel is rewound, a corrective X gate is applied,
and the qubit comes back:

```
[ALARM] Bit Flip Error Detected! P(|1>) = 0.9801
[t=3] History Rewound. State restored to t=0.
[SUCCESS] Qubit is alive and corrected.
```

One substrate, one intervention operation, one audit trail. The plant changed; nothing else did.
That is the claim the control frame rests on, and a reader checks it by opening two example
directories.

### 4.2 Avenue two: logical gates for general CSS codes

The gauge-field formalism identifies a CSS code with a chain complex, expresses Pauli operators as
exponentials of operator-valued cochains, and reads logical operators as Wilson loops on homology
and cohomology classes. Haruna's November-2025 paper uses it to construct logical `S`, Hadamard, `T`
and (multi-)controlled-`Z` for **general** CSS codes, with no special manifold or product structure
required, as exponentials of polynomial functions of the electric and magnetic gauge fields. He
proves the logical action depends only on the (co)homology class, which makes the physical
decompositions well defined at the logical level.

The crate implements that construction: `logical_z`, `logical_x`, `logical_s`, `logical_hadamard`,
`logical_cz` and `logical_t`, over multivectors carrying complex coefficients. The matrix
exponential is a guarded Taylor series that surfaces overflow and non-convergence as typed errors,
because a silently truncated series and a genuine identity gate are indistinguishable to the caller
and must not be.

Two things must be said in the same breath, every time, because QEC reviewers police both:

- **Fault tolerance is not claimed.** Haruna writes that his focus "is not on fault tolerance such
  as constant depth or locality, but rather on establishing an algebraically transparent foundation."
  The Hsin–Kobayashi–Zhu constructions are the fault-tolerant, constant-depth line, and they are
  roadmap here, not shipped.
- **Logical gates are not quantum algorithms.** A cohomology-operation gate set is an instruction
  set. Compiling an algorithm onto it is a separate discipline.

Within those bounds this is a real capability, and it is the avenue with the shortest path to a
result P1 cares about: the homology-class invariance Haruna proves is a *check*, and this codebase
turns checks into freeze-time gates for a living.

As of 2026-08-21 the operation beneath the gates also runs. `deep_causality_topology` carries the
cup product, generic over `ChainComplex` rather than specialised to lattices, which is what lets an
arbitrary CSS code be asked what it natively supports. The roadmap had recorded a branching
structure as the keystone blocking this; the audit found it already present in both cell families,
so the rung cost documentation rather than construction. What remains above it is `SPEC-T3`, the
higher cup products, and the catalogue they feed.

### 4.3 Avenue three: the freeze gate

A quantum causal model factorizes a process operator into per-node Choi–Jamiołkowski operators. Not
every product of operators is one. The factors whose Hilbert supports intersect must pairwise
commute (Lorenz 2022, Def. 3.3), and that clause is free at two factors and substantial at three or
more.

The crate makes it a gate at the graph freeze boundary. Factors ride an external decoration as
static freeze-time data; `freeze_quantum` embeds each intersecting-support pair on its common
support, forms the commutator, and tests the Frobenius norm against a condition-driven forward error
budget rather than a fixed epsilon. The check is sound: it never accepts a non-commuting model. It
may be incomplete. On failure it names the offending pair and rolls the graph back to its dynamic
state.

Alongside it runs the faithfulness gate. Van der Lugt and Lorenz (arXiv:2508.11762, August 2025)
give a decidable criterion: a causal structure is faithfully representable by a traditional circuit
exactly when it contains no `C₃` sub-relation, canonically two commuting CNOTs. The crate derives
the structure from the frozen graph's reachability and rejects `C₃`-containing structures at freeze.
An August-2025 iff-criterion, running as a check.

For a control engineer this is the plant model's admissibility test, and it runs before the loop
starts rather than after it misbehaves:

```
[2] Non-commuting model: σx and σz on leg 0
    ✓ freeze aborted: factors at nodes 0 and 1 do not commute
    is_frozen() = false (rolled back)
```

### 4.4 The refutation is the trust anchor

The roadmap wanted `partial_trace_preservation`: if a subgraph's factors commute and its neighbours
commute, does the marginalized factor still commute after encapsulation? It does not. Partial trace
is positive and linear, and it is not an algebra homomorphism. The Lean proof closes the
counterexample over the integers by `decide`, and the residual commutator is the matrix
`[[0,4],[−4,0]]`, which is `+4i·σy`.

What followed is the part worth showing. Quantum-subgraph nesting was dropped as a crate feature,
flat QCM became the supported path, and the conditional boundary case that *is* true
(`partial_trace_preservation_boundary`) shipped alongside the refutation. Ten quantum theorems close
with zero `sorry`, each bound to a Rust test through `THEOREM_MAP.md`.

A formalization that killed a feature the project wanted is worth more to this audience than any
benchmark.

### 4.5 The verdict carrier is a shared, closed lattice

One small fact evidences the "one substrate" claim better than any architecture diagram.
`deep_causality_algebra` defines `Verdict` as a bounded lattice with complement, and its own
documentation names exactly three lawful carrier classes: Boolean (`bool`), MV (`Prob` and `f64`,
where `meet = min`, `join = max`, `complement = 1 − p`, and excluded middle fails), and, marked
planned at the time it was written, the orthomodular projection lattice. It then states the scope
guard: general effects `0 ≤ E ≤ I` form only an effect algebra with *partial* meet and join, so no
blanket operator instance is lawful.

`deep_causality_quantum` supplies that third class. `Projection<R, D>` implements `Verdict`, and
the Born read-out lands in `Prob`, the second. The merge itself is not in either crate: `∇` is
`SymMonoidal::merge` in `deep_causality_haft`, whose monoid laws are machine-checked in
`Haft/SymmetricMonoidal.lean` and witnessed by Rust tests.

So a quantum measurement outcome and a classical Boolean verdict combine at a graph join through
the same lattice operation, under laws proved once, in a crate that knows nothing about quantum
mechanics. `deep_causality_uncertain` sits on the same trait, so a shot-noise reading is a lawful
carrier too. That is what "one substrate" has to mean if it is to mean anything, and it is
checkable in four files.

### 4.6 The seam is stated, not blurred

The categorical substrate is the Kleisli category of a classical monad. Coherence therefore lives
inside one causal function; hyperedges between causaloids carry classical data or operators to be
checked, never live amplitudes. The measurement cut and the Kleisli boundary are the same line.

That line is where real hardware already puts it. The dynamic-circuit model is coherent evolution
within a circuit and classical control between measurements. A control engineer recognizes the
picture immediately: the quantum plant is observed through a measurement, and everything after that
measurement is classical. The crate's seam sits where the device's seam sits.

The two senses of "quantum" are kept apart by the build. The verifiable path is the default:
deterministic simulated operators, checked at freeze, backed by Lean. The emergent path lifts a
physical device call into the monad as a typed effect, and sits behind an off-by-default feature
that adds no network or async dependency. A model states plainly which kind of evidence a verdict
rested on, and the compiler enforces the split.

---

## 5. The value chain, and where QCL sits

### 5.1 The eleven tiers

| # | Layer | Representative tooling | Scarce resource | QCL |
|---|---|---|---|---|
| 1 | Materials, fabrication | foundries, in-house | yield | — |
| 2 | QPU / chip | IBM, Google, Quantinuum, QuEra | coherence | — |
| 3 | Cryogenics, wiring, amplifiers | Bluefors, in-house | thermal budget, I/O lines | — |
| 4 | Control electronics | Quantum Machines, QICK/RFSoC, Zurich | channel count | — |
| 5 | **Quantum firmware** — calibration, tune-up, characterization, stabilization | Qiskit Experiments, Qibocal | **device time** | **primary** |
| 6 | Real-time control and feedback | Guppy/Selene, `cudaq-realtime`, QUA | latency inside the coherence window | **reachable, unmeasured** |
| 7 | QEC — code design, simulation, decoding | Stim, PyMatching, Qualtran | decoder throughput | **secondary** |
| 8 | Compilation, transpilation | tket, BQSKit, Qiskit transpiler | circuit depth | — |
| 9 | Circuit and algorithm frameworks | Qiskit, Cirq, PennyLane, Q# | developer attention | — |
| 10 | Applications | domain libraries | problem fit | — |
| 11 | Cloud orchestration | Braket, Azure Quantum, IBM Platform | access | — |

Layer 5 is the recognised quantum-firmware tier: the layer that minimises hardware error through
calibration, tune-up, characterization and automation. Its scarce resource is **device time**, and
the chain above it optimises for circuit depth, developer attention and access instead. That is the
economic argument for the position, and it stands without any claim about causality.

### 5.2 Two loops, not one

QCL runs a **slow loop**. Its unit of work is a shot budget: declare hypotheses, screen them, buy
the cheapest experiment that separates the survivors by a stated number of bits, adjudicate. A
thousand shots is the granularity, minutes is the cadence, and the output is a repair decision — 
which knob, which coupler, which pulse.

DeepCausality's substrate also runs a **fast loop**, and it ships today. The five programs in
[`causal_correction_examples`](../../../examples/causal_correction_examples) are closed-loop control:
a monitor inspects each tick, `alternate_value` replaces the in-flight value when it leaves the safe
envelope, and the chain advances from the corrected state. `corrective_ddos_detector` is the closest
in shape to a syndrome loop — a sliding-window baseline carried in `State`, five consecutive samples
above 3σ declare the event, and the intervention engages within one tick.

The two loops sit at different tiers and have different budgets. Conflating them understates the
substrate and overstates QCL.

### 5.3 What layer 6 would require, in numbers

| Modality | Code cycle | Demonstrated feedback |
|---|---|---|
| Superconducting | 0.2–10 µs | sub-1 µs mean decode per round, 9.6 µs feedback latency |
| Trapped ion, shuttling design | ~235 µs | — |

The gap between the two is a factor of 25 to 1000, and it decides the question. Sustaining a decision
per round at 0.2–10 µs is FPGA and ASIC territory; a general-purpose causal engine does not belong
there. A ~235 µs code cycle is a different proposition, and it is the reason Guppy and Selene target
that class of machine.

**The loop shape is native and demonstrated. The port is verified. The latency budget is still
unmeasured.**

Measured on 2026-08-21:

```
rustup target add aarch64-unknown-none
cargo build -p deep_causality_core --no-default-features --features no-std \
      --target aarch64-unknown-none          →  Finished
cargo clippy  … same flags                   →  clean
cargo build -p deep_causality_core           →  Finished  (std path unaffected)
cargo test   -p deep_causality_core          →  pass
```

`deep_causality_core` compiles for bare-metal ARM64. It is `no_std` **plus `alloc`**: building with
no allocator at all fails with 17 errors, because `EffectLog` holds a `Vec<LogEntry>` and
`CausalityError` holds an `alloc::string::String`. An allocator is available on the class of board
that matters, so this is a constraint rather than an obstacle.

Reaching that took a two-line fix. `deep_causality_haft`, `deep_causality_algebra` and
`deep_causality_num` already declared `default-features = false` on their dependants and carried a
`no-std` feature; `deep_causality_core` declared neither, so its `--no-default-features` build
silently pulled `std` back in through `haft`. Core's own code was always `no_std`-clean — the
dependency wiring was the gap, and the earlier host-side build that appeared to succeed had simply
linked a `std` dependency chain underneath a `no_std` crate.

What this changes: the relevant hardware is the **processing system**, not the fabric. Zynq
UltraScale+ RFSoC boards — ZCU111, ZCU216, RFSoC 4x2, the class QICK targets — carry hard ARM A53
application cores and R5F real-time cores on the same die as the converters, so the decision logic
sits on-chip with no PCIe or network hop. Rust synthesised to fabric is not a path and is not
proposed. The fabric generates pulses and streams converters; the cores sequence and decide.

**One known obstacle remains, and it is the log.** `EffectLog::add_entry` performs
`entries.push(LogEntry::new(ts, message.to_string()))` — a `String` allocation per entry, per stage,
per tick. Allocation is not bounded-time, so as written the channel that makes a device-time decision
defensible is the channel a hard deadline cannot tolerate. The resolution follows the two-loop split
of §5.2: the fast loop writes fixed-capacity, preallocated event codes; the slow loop drains that
ring and inflates it into the full provenance record. A design item with a clear shape, not a
research question.

What is still not measured is the tick cost itself. That is what the benchmark suite in
[`qcl-design-note.md`](qcl-design-note.md) §7 exists to establish, against the 235 µs and 0.2–10 µs
reference points above.

### 5.4 The port surface above core

`deep_causality_quantum` does not declare `no_std`, so whether it can reach bare metal was open.
Surveyed on 2026-08-21, it can, and nothing in the way is architectural.

| Crate | Status | What stands in the way |
|---|---|---|
| `deep_causality_num` | `no_std` ✓ | — (already carries `libm_math` for bare-metal float math) |
| `deep_causality_algebra` | `no_std` ✓ | — |
| `deep_causality_haft` | `no_std` ✓ | — |
| `deep_causality_core` | `no_std` ✓ | — (wired 2026-08-21, §5.3) |
| `deep_causality_num_complex` | `no_std` ✓ | — |
| `deep_causality_num_dual` | `no_std` ✓ | — |
| `deep_causality_metric` | `no_std` ✓ | — |
| `deep_causality_ast` | std only | 11 sites / 9 files; `Arc` and `VecDeque` → `alloc` |
| `deep_causality_tensor` | std only | 38 sites / 27 files |
| `deep_causality_multivector` | std only | 54 sites / 17 files |
| `deep_causality_quantum` | std only | 10 sites / 6 files |
| `deep_causality` (graph) | std only | reached from 3 files, all under `types/qcm/` |
| `deep_causality_uncertain` | std only | optional already, behind the `qpu` feature |
| `deep_causality_rand` | std only | **dev-dependency** of tensor; does not affect the library build |

**No genuine std blocker exists in the numeric stack.** `deep_causality_tensor` and
`deep_causality_multivector` contain no `std::collections`, no `std::sync`, no threads and no IO.
Their `std::` imports are `ops`, `iter::Sum`, `fmt` and `marker::PhantomData` — every one of them
reachable as `core::`. Neither depends on `deep_causality_par` or rayon, which was the risk worth
checking and is absent.

The three items that are not a plain `core::` rewrite are all `alloc::` rewrites:

- `std::error::Error`, twice in tensor. `core::error::Error` has been stable since Rust 1.81 and the
  workspace MSRV is 1.93.0, so this is a rename.
- `BTreeMap` and `BTreeSet`, seven sites in quantum. These live in `alloc::collections`; they were
  never std-only.
- `Arc` and `VecDeque` in `deep_causality_ast`. `alloc::sync::Arc` needs atomic pointers, which
  aarch64 has.

**The one dependency worth restructuring is the graph.** `deep_causality_quantum` reaches into
`deep_causality` for `CausableGraph` and `CausalityGraphError` from exactly three files, all under
`types/qcm/`. That pulls a large std-only crate in for a narrow surface. Feature-gating those three
files keeps the QCM validation path off the bare-metal build without touching the gate kernels,
density matrices, or the QPU seam.

**The feature scaffolding is most of the work, and it does not exist yet.** `deep_causality_ast`,
`deep_causality_tensor` and `deep_causality_multivector` carry **no `[features]` section at all**.
`deep_causality_quantum` has one, but only for the `qpu` seam, and it declares `default = []`, so
giving it `default = ["std"]` changes what downstream consumers get by default. Across the four
crates there are 22 dependency blocks and **not one** sets `default-features = false` — which is
exactly the omission that made core's `--no-default-features` build silently link `std` underneath a
`no_std` crate (§5.3).

**Two naming conventions are already in the tree, and mixing them will produce confusing feature
graphs.** `deep_causality_metric` uses `std` / `alloc`. The numeric chain — `num`, `algebra`, `haft`,
`num_complex`, `num_dual` — uses `std` / `no-std`. `deep_causality_core` now carries all three,
`std` / `alloc` / `no-std`, because it needs `alloc` as a distinct level and a `no-std` name to
forward. Settle on core's three-level shape before porting anything above it; retrofitting a
convention across four more crates costs more than choosing one now.

**Per crate, the work is four mechanical steps:** add the `[features]` block; add
`#![cfg_attr(not(feature = "std"), no_std)]` and `extern crate alloc` to `lib.rs`; add
`default-features = false` to every dependency block; rewrite the `std::` imports.

**Ordering.** `ast` → `tensor` → `multivector` → `quantum`, bottom-up, each verified with a
cross-compile before the next begins. About 113 import sites across 59 files, plus 22 dependency
blocks and four feature sections.

The estimate is a survey, not a build. The claim that survives without one is narrow: nothing found
requires std, and the work is feature wiring plus import rewriting. Only a green cross-compile
settles it.

---

### 5.5 Five gaps, ranked

**A — diagnosis is projected out at the 5↔7 seam.** Stim's documentation states the design directly:
the detector error model "focuses on the relationship between error mechanisms and their detector
signatures rather than tracking individual fault types." That is correct for decoding, where the
likely error *pattern* is what a correction needs. It means the ecosystem's standard artifact
structurally cannot answer which coupler to recalibrate. A decoder repairs the state; nothing
attributes the cause.

**B — no experiment design under a cost budget.** Layer 5 runs known protocols and fits them.
Bayesian adaptive design exists and is mature, over *continuous parameters* — adaptive tomography,
Hamiltonian learning. Selecting a **discrete** experiment subset under a cost budget against a stated
separation floor is the open case, and device time is what it spends.

**C — no admissibility screening before device time.** Nothing checks that a proposed explanation is
a valid causal model before shots are spent. A supporting feature rather than a product.

**D — code design upstream of Stim.** Real, contested, and distant. qLDPC construction is an active
field.

**E — drift attribution over time.** Optimus solved calibration *scheduling*; the attribution inside
it is a human reading plots.

**A and B are the target.** They are one capability seen from two tiers: attribute a symptom to a
mechanism, then buy the cheapest evidence that confirms it.

---

## 6. Related work

Three lines of published work sit close enough that the delta has to be stated rather than implied.

**Causal inference for crosstalk.** Rudinger et al., *Detecting crosstalk errors in quantum
information processors* (Quantum 4, 321, 2020) adapts the PC algorithm to crosstalk, uses conditional
independence tests under the Markov condition, and determines direction as well as existence, at
O(n²) to O(n³) experiments on 2- and 6-qubit processors. The causal framing for crosstalk is six
years old and belongs to that paper. The delta is that the protocol runs a **fixed experiment set**:
it does not enumerate named hypotheses, does not reject inadmissible ones before spending device
time, and does not select experiments under a cost budget.

**Bayesian adaptive experiment design.** Adaptive Hamiltonian estimation, adaptive tomography, and
transmon-qutrit identifiability all select the next experiment to maximise information. They target
continuous parameter estimation. Discrete discrimination among named causal structures under a cost
budget is the case that remains.

**DEM reconstruction and equivalence.** Recent work learns detector error models from syndrome data
(arXiv 2606.16288, 2601.22286) and checks DEM equivalence in quasilinear time (arXiv 2606.14677). The
first is parameter estimation over a known structure; the second is verification. Neither closes gap
A, and both are close enough that any claim in this area should cite them.

**Calibration frameworks.** Qiskit Experiments and Qibocal fit a model to data from a chosen
experiment. QCL chooses the experiment by what it would discriminate. These compose: QCL decides,
layer 5 executes and fits, layer 6 runs it against the device.

---

## 7. What ships today, measured

On the reference machine (Apple M3 Max, 16 cores, 128 GB):

- `deep_causality_quantum`: 3,930 lines across 26 source files; **190 tests, all passing**, in
  roughly 17 s with `--all-features`.
- **10 Lean theorems**, zero `sorry`, each bound to a named Rust witness in `lean/THEOREM_MAP.md`.
- The operator layer: validated `DensityMatrix`, Choi–Jamiołkowski in both directions, Kraus and
  Choi channel application, CPTP checks, partial trace, leg embedding, commutator, Frobenius norm,
  Hermitian eigendecomposition.
- **Six Haruna logical gates** on the gauge-field construction, with typed failure on overflow and
  non-convergence.
- In `deep_causality_topology`, the **cup product** those gates are built from: binary and `n`-fold,
  generic over any `ChainComplex` whose cells split, covering the simplicial (Alexander–Whitney) and
  cubical (Serre) families. Pinned by the Leibniz identity against the crate's own coboundary
  operators, associativity, graded commutativity on cohomology, the `H¹ × H¹ → H²` torus pairing
  (`±L²`) and the 3-torus triple product (`L³`). 72 tests, whose expectations were fixed before the
  implementation existed and cross-checked against an independent reference in exact rational
  arithmetic.
- Two freeze-time gates: quantum Markov commutativity, C₃-exclusion faithfulness.
- An orthomodular projection lattice as a `Verdict` carrier, with Born read-out to `Prob`.
- Seven runnable quantum examples: the freeze check, error correction by history rewind,
  electroweak symmetry breaking, a Chern number, the quantum geometric tensor, the IKKT matrix
  model, and a Hopf-fibration Bloch projection.
- **Five corrective-control examples on the same core**, each measured open loop against closed
  loop. They carry classical plants today, and they are the working proof that the intervention
  operation belongs to the substrate rather than to any one domain.
- No `unsafe`, and no external runtime dependency in the default build.

---

## 8. The boundaries

These go on their own page, phrased as situations rather than as feature gaps. The CFD site's
boundaries page is the model.

1. **Coherence stops at the causaloid.** Two causal functions cannot be placed in superposition, and
   entanglement is not maintained across the graph. The crate orchestrates quantum black boxes; it
   is not a substrate for coherent distributed quantum computation.
2. **Flat models only.** Quantum-subgraph nesting is not supported, because the theorem it would
   need is false and its physical meaning is unestablished.
3. **Few nodes.** Estimating a process operator needs informationally complete interventions at
   every node, which is exponential. The freeze check does not repeal that.
4. **Faithfulness is scoped to traditional circuits.** The C₃ criterion covers the non-routed
   regime. The general routed and direct-sum hypothesis is open upstream.
5. **Soundness without completeness.** The Markov check never accepts an invalid model and may
   reject a valid one.
6. **No speedup.** The crate adds no performance of its own.
7. **Unpublished.** Git dependency only; `publish = false` today.
8. **Supervisory rate, not inner-loop rate.** Real-time syndrome decoding runs on microsecond rounds
   against dedicated hardware. This substrate is the coupling, decision and provenance layer around
   such a loop, not the loop's inner kernel. No latency envelope has been measured, and the site
   must not imply one.
9. **The quantum control examples are one qubit.** The corrective-loop evidence is strong on
   classical plants and thin on quantum ones. A worked closed-loop quantum example beyond the
   single-qubit rewind is the most valuable thing that could be added before launch.
10. **The logical gates are not fault-tolerant.** The construction they implement explicitly relaxes
    fault tolerance and depth optimality. Constant-depth fault-tolerant gates are the
    Hsin–Kobayashi–Zhu line and are roadmap, not shipped.

---

## 9. Why "dynamic" earns its place

Quantum causal models are written as static objects: a fixed graph, a fixed factorization, a theorem
about it. A control loop is the opposite kind of object. The word *dynamic* claims three specific
things, and each has code behind it.

**The structure is decided, then re-decided.** A causaloid graph is mutable while it is being built
and analytic once frozen. The Markov and faithfulness gates run at the freeze boundary, so a model
that changes shape gets re-checked rather than re-argued.

**Branching happens on measurement outcomes.** `continue_with` forks a process on a classical
result, and `alternate_value` replaces the value an in-flight chain carries. That pair is the
dynamic-circuit model raised to the level of a whole causal process, with the fork and the
correction both recorded rather than implied.

**History is a first-class channel.** The error-correction example rewinds a qubit to a prior state
after a syndrome fires, because the state channel carried the history. Counterfactual reasoning over
quantum state is an ordinary use of the monad, not an extension to it.

---

## 10. What the site should therefore be

Ten pages, grouped by the three avenues. Same discipline as the CFD site: a claim per page, evidence
under it.

| Page | Job |
|---|---|
| Home | The plant-and-loop thesis, the open-versus-closed-loop numbers, the install line, the non-audience note |
| The loop | Monitor, envelope, intervene, continue; the five classical examples and the qubit one; boundaries 8 and 9 stated on the same page |
| The gates | The gauge-field construction, the six implemented gates, the homology-class invariance, and boundary 10 in the same breath |
| The checks | The two freeze-time gates, each with its paper, its code path, and its failure output |
| Formalization | The Lean table, the refuted theorem first, `THEOREM_MAP` traceability, and the shared verdict lattice of §4.5 |
| Modalities | Verifiable against emergent, and why the split is a compile-time guarantee |
| Examples | Every runnable one, with its field and its exact command |
| Boundaries | Section 8, written as situations |
| Roadmap | Tracks Q, T, G and D with their gating assumptions, and what a failed gate kills |
| Papers | The bundled PDFs and the evidence citations, all properly attributed |

What does **not** go on the site: funding strategy, target programmes, named prospective
collaborators, and any claim about correlated-noise decoding before SPEC-D1 produces a number. That
material lives in `dynamic-qcm.md` and stays there.

---

## 11. Decisions I need from you

1. **Prime audience.** Set to the classical-quantum control engineer, with P1 error correction and
   logical-gate design and P2 characterization as the entry personas, and P3 as the review audience.
   *Recommendation: keep it.*
2. **Avenue weighting.** Three avenues now compete for the home page: the loop, the gates, the
   checks. *Recommendation: the loop leads, the gates get equal billing in the nav, the checks are
   the proof page both point at. Three is the ceiling; a fourth would read as sprawl.*
3. **How hard to lean on control.** The loop is the strongest frame and its quantum evidence is one
   qubit deep. *Recommendation: lead with it, put boundaries 8 and 9 on the same page, and add a
   second closed-loop quantum example before launch if you want the claim to carry weight under P1
   scrutiny.*
4. **Roadmap depth.** Tracks with gating assumptions, or a short "what is next" list?
   *Recommendation: tracks with gates, no funding material.*
5. **The decoding claim.** Track D is the strongest story and has no number behind it.
   *Recommendation: named research track, no performance claim until SPEC-D1 runs.*
6. **Publication.** The hero install line reads differently for a crates.io release than for a git
   dependency. Publish `deep_causality_quantum`, or keep the git line?
7. **Name usage.** Is "DQC" used as an abbreviation in body copy, or only "Dynamic Quantum
   Causality" spelled out with the crate name alongside it?

---

## 12. Sources

### Ecosystem and related work (§5, §6)

- Sarovar, M., Proctor, T., Rudinger, K., Young, K., Nielsen, E. & Blume-Kohout, R. (2020).
  *Detecting crosstalk errors in quantum information processors.* Quantum **4**, 321.
  DOI 10.22331/q-2020-09-11-321. The PC
  algorithm adapted to crosstalk; conditional independence under the Markov condition; direction as
  well as existence; a fixed experiment set at O(n²)–O(n³).
- *Qiskit Experiments: A Python package to characterize and calibrate quantum computers.* JOSS
  **8**(84), 5329 (2023). Layer 5: run a known protocol, fit, update calibration.
- *Qibocal: an open-source framework for calibration of self-hosted quantum devices.*
  arXiv:2410.00101 (2024). Layer 5, self-hosted platforms, live fitting.
- Gidney, C. (2021). *Stim: a fast stabilizer circuit simulator.* Quantum **5**, 497. And the DEM
  file-format specification, `doc/file_format_dem_detector_error_model.md` in `quantumlib/Stim`.
  The source for gap A: the DEM tracks detector signatures rather than individual fault types.
- Ferrie, C., Granade, C.E. & Cory, D.G. (2012). *Adaptive Hamiltonian Estimation Using Bayesian
  Experimental Design.* arXiv:1111.0935. Adaptive design over continuous parameters, the closest
  prior work to the `design` stage. See also *Identifiability and Characterization of Transmon
  Qutrits Through Bayesian Experimental Design*, arXiv:2312.10233.
- *Demonstrating real-time and low-latency quantum error correction with superconducting qubits.*
  Nature Communications (2026). Sub-1 µs mean decode per round, 9.6 µs feedback latency — two of the
  numbers behind §5.3. The 0.2–10 µs code-cycle range and the ~235 µs trapped-ion shuttling estimate
  are from arXiv:2108.12371, *The Impact of Hardware Specifications on Reaching Quantum Advantage in
  the Fault Tolerant Regime*.
- DEM reconstruction from syndrome data, arXiv:2606.16288 and arXiv:2601.22286; quasilinear DEM
  equivalence checking, arXiv:2606.14677. Parameter estimation and verification respectively,
  adjacent to gap A without closing it.
- *Quantum firmware and the quantum computing stack*, Physics Today. The layer-5 naming used in §5.1.

### Evidence for the challenges

- **C1.** Battistel, F., Chamberland, C., Johar, K., Overwater, R.W.J., Sebastiano, F., Skoric, L.,
  Ueno, Y. & Usman, M. (2023). *Real-time decoding for fault-tolerant quantum computing: progress,
  challenges and outlook.* Nano Futures **7**(3), 032003. DOI 10.1088/2399-1984/aceba6. The backlog
  problem; the roughly one-microsecond QEC cycle for superconducting transmons; throughput over
  latency.
- **C2.** Kam, J.F., Gicev, S., Modi, K., Southwell, A. & Usman, M. (2025). *Detrimental
  non-Markovian errors for surface code memory.* Quantum Science and Technology **10**(3), 035060;
  arXiv:2410.23779. Multi-time "streaky" correlations on syndrome qubits and two-qubit gates
  severely degrade logical error rate scaling; correlation *structure* decides the harm.
- **C3.** Mesman, K.J., Battistel, F., Reehuis, E., de Jong, D., Tiggelman, M.J., Gloudemans, J.,
  van Oven, J.C. & Bultink, C.C. (2024). *Q-Profile: Profiling Tool for Quantum Control Stacks
  applied to the Quantum Approximate Optimization Algorithm.* IEEE QSW 2024, 116–124;
  arXiv:2303.01450. Classical control infrastructure and synchronization dominate the QAOA execution
  profile; 1.40× from active reset, 1.37× from parallel control-module initialization.
- **C4.** Haruna, J. (2025). *Note on Logical Gates by Gauge Field Formalism of Quantum Error
  Correction.* arXiv:2511.15224, 19 November 2025. "A general and systematic framework for
  constructing logical gates for general QEC codes remains lacking."

Two of these share authors with each other and with the crate's own reading list, which is a
reasonable sign the citations sit inside the conversation rather than beside it: Battistel appears
on C1 and C3, Usman on C1 and C2, and Modi on C2 is the process-tensor line the roadmap names as a
natural contact.

### Papers bundled in `deep_causality_quantum/papers/`

- Lorenz, *Quantum causal models* (2022). Process operator; quantum Markov condition, Def. 3.3.
- Barrett, Lorenz & Oreshkov, *Cyclic Quantum Causal Models*, arXiv:2002.12157.
- Lorenz & Barrett, *Causal and compositional structure of unitary transformations*, arXiv:2001.07774.
- van der Lugt & Lorenz, *Unitary causal decompositions*, arXiv:2508.11762. The C₃-exclusion criterion.
- Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction*, arXiv:2511.15224.

Referenced but not bundled: Hsin, Kobayashi & Zhu, arXiv:2411.15848, the fault-tolerant
constant-depth line.

### Code the claims rest on

`deep_causality_core/src/traits/alternatable_value/`,
`deep_causality_quantum/src/types/qgates/gates_haruna.rs`,
`deep_causality_quantum/src/types/qcm/`,
`lean/DeepCausalityFormal/Quantum/`,
`examples/causal_correction_examples/`,
`examples/quantum_examples/`.

### Related notes in this folder

[`dynamic-qcm.md`](dynamic-qcm.md) (the master roadmap and its falsifiable gates),
[`QCM-on-EPP.md`](QCM-on-EPP.md) (the reconstruction argument),
[`full-stack.md`](full-stack.md) (the four causal regimes on one substrate),
[`quantum-epp.md`](quantum-epp.md) (the hybrid orchestration corollary).
