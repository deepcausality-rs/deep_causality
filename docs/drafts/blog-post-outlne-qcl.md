# QCL: The Quantum Causal Language — Blog Post Outline

A dictation outline, built with `docs/writing_guides/TurnRawMaterialintoWriting.pdf`: raw
information by source, the argument, each section's topic sentence, supporting points, closing
sentence and transition, then the verb check. The appendix records the critical analysis
(`MiniatureGuideCriticalThinking.pdf`) behind this structure.

Target: 2,800 to 3,200 words, four code or output blocks, two tables. Readers: quantum hardware
and sensor engineers and researchers, and DeepCausality readers new to quantum causal models.
Assume they know qubits and density matrices, not Lorenz's framework.

---

## Part 1: Raw information, by source

Each fact carries its source letter. Strip the tags before publishing; keep a tagged copy.

| Letter | Source |
|---|---|
| A | `deep_causality_quantum/README.md` |
| B | `openspec/notes/quantum-site/quantum-essence.md` (checked against the tree 2026-09-30) |
| C | `examples/quantum_examples/qcl_examples/qcl_crosstalk/` source and README |
| D | `cargo run --release -p quantum_examples --example qcl_crosstalk`, run 2026-10-21 |
| E | `openspec/notes/quantum-site/03_sensing_raw_information.md` (runs of 2026-10-08) |
| F | `openspec/notes/quantum/Lorenz-Gap.md` (assessment of 2026-10-08) |
| R | `openspec/notes/quantum-site/01_raw_information.md` (runs of 2026-09-30) |
| L1 | Barrett, J., Lorenz, R. & Oreshkov, O. (2019). *Quantum causal models.* arXiv:1906.10726 |
| L2 | Barrett, J., Lorenz, R. & Oreshkov, O. (2021). *Cyclic quantum causal models.* Nat. Commun. 12, 885; arXiv:2002.12157 |
| L3 | Lorenz, R. (2022). *Quantum causal models: the merits of the spirit of Reichenbach's principle for understanding quantum causal structure.* Synthese 200:424, doi:10.1007/s11229-022-03887-5 |
| L4 | van der Lugt, T. & Lorenz, R. *Unitary causal decompositions: a characterisation via lattice theory.* Quantum (accepted 2026-09-09); arXiv:2508.11762v2 |
| L5 | Lorenz, R. & Tull, S. (2026). *Causal and compositional abstraction.* arXiv:2602.16612 |
| L6 | Lorenz, R. & Barrett, J. (2021). *Causal and compositional structure of unitary transformations.* Quantum 5, 511; arXiv:2001.07774 |
| L7 | Lorenz, R. (2020). *Quantum causal structure.* DPhil thesis, University of Oxford |
| H | Haruna (2025), gauge-field formalism for logical gates, arXiv:2511.15224 |
| S | Sarovar, M. et al. (2020). *Detecting crosstalk errors in quantum information processors.* arXiv:1908.09855 |

The Lorenz PDFs sit in `openspec/notes/quantum/Lorenz/`.

### The problem (C, L1, L3, L7)

- In the crosstalk case, three Markov-equivalent structures fit the passive observation exactly.
  (C)
- Interventions separate them; full process tomography costs the most. (C, L1)
- Lorenz calls partial interventional data the practical problem: discovery when "the given data is
  not a (full) process operator". (L1 §13.2, L3 §5, L7 outlook; F)

### Theory to check (A, B, F, L1 to L5)

| Lorenz's framework | QCL check or construct |
|---|---|
| A model multiplies per-node Choi operators into the process operator (L1) | a structural candidate: factors plus the Hilbert legs each acts on (B) |
| Factors on intersecting supports commute: the quantum Markov condition (L3, Def. 3.3) | `check_markov`: ‖[ρⱼ, ρₖ]‖_F against a threshold from the scalar's epsilon (B) |
| For acyclic graphs, Markov ⇔ compatibility with a unitary (L1, Thm 4.10) | why `check_markov` gates admission (F) |
| C₃-exclusion fixes which structures traditional circuits represent (L4, Thm 3.2) | `check_decomposable` (B) |
| An abstraction summarises a low-level model; naturality squares test it (L5) | the abstraction subject, `check_naturality`, the composition law (B) |
| With cycles, commuting channels no longer guarantee a valid process (L2) | `build()` refuses cycles by scope (D) |

- The theory defines a valid model. It leaves open which valid candidate explains the data, which
  experiments to run, and at what cost. (author's reading of L1 to L5; label it)

### The language (A, B)

- `QclBuilder`, an embedded Rust builder: a program names a subject and its candidates; the library
  refuses, screens, plans and adjudicates. (B)
- Pipeline: `config(subject, candidates) → validate(checks) → Screened → control(evidence) →
  report`. (B)
- Four subjects: `over_plant` (validate, then control), `over_model` (validate), `over_code`
  (validate only), `over_circuit` (validate on the dilation). (B)
- `QclBuilder::config::<R, N>()` names two working types once: `R: RealField` sets accuracy, and
  every tolerance derives from `R::epsilon()`; `N: NaturalNumber` sets count headroom. Swapping `R`
  among `f32`, `f64`, `Float106` and `BFloat16` recomputes every threshold. (B)
- `design` picks the cheapest experiment set that separates every candidate pair by a floor in bits,
  by exact dynamic programming over 2^C(n,2) subsets; it refuses more than 7 candidates by default.
  Separation: the Bhattacharyya distance of n draws, n · (−log₂(√(pq) + √((1−p)(1−q)))). (B)
- `adjudicate` names a survivor only when one world alone holds and separates from every rival by
  the floor; otherwise it names the ambiguity. (B)
- The campaign runs the cheapest planned experiment, re-plans when the context moves a prediction,
  and stops at a survivor, outside the model, or when nothing left separates the rest. It steps
  through `CausalFlow::iterate_until`, the causal monad's flow. (A)
- Forked worlds keep separate ledgers; the report compares them and never sums them. (B)

### What the types enforce (A, B, D)

- A plant configuration with structural candidates has no `ControlSource` implementation, so the
  compiler rejects `control` before `validate`. (B)
- `build()` refuses unsound configurations before any stage runs. (B, D)
- `validate` ends in `finalize()`, which returns a `Screened`; only a `Screened` enters `control`.
  (B)

### What a verdict carries (B, R)

- `Check`: item, measured quantity, threshold, margin (measured over threshold; above 1 rejects),
  outcome. `CheckReport`: the checks, the count examined, and `Accepted`, `Rejected`, or `Vacuous`
  when it examined nothing. (B)
- On a two-node chain, `check_decomposable` finds no 3 × 3 block and reads `Vacuous`. (R)
- Abstraction reports name their path, exact or numeric. (B)
- Named refusals: `CyclicStructureUnsupported`, `CommutatorNonZero`, `BoundaryNotHeld`,
  `SectionNotInverse`, `NaturalityDimensionExceeded`, a refused cross-block gate. (B §9)
- Named ambiguities: no survivor, several survivors, an unseparated survivor, non-commuting
  projections, a single world. (B)

### The crosstalk run, 2026-10-21 (C, D)

- Candidates: H1 Q1→Q2, H2 Q2→Q1, H3 a shared bath, H4 the cycle Q1→Q2→B→Q1. (C)
- `build()` refuses H4: "cyclic causal structures are outside v1's scope by decision, not because
  they fail a criterion". (D)
- Screens accept: normalization 7 examined, Markov 5, decomposable 3. (D)
- Predictions under H1/H2/H3: E0 passive, cost 1, 0.04/0.04/0.04; E1 do(Q1=|1⟩), cost 1,
  0.40/0.10/0.10; E2 do(Q2=|1⟩), cost 1, 0.10/0.40/0.10; E3 echo, cost 2, 0.01/0.01/0.04. (D)
- Plan: E1 and E2, cost 2 of 5; tightest pair 99.5 bits against a 5-bit floor. (D)
- Campaign on H1-drawn observations: E1 read 0.396 ± 0.015 over 1024 shots; H1 survives at 99.5
  bits after one experiment. "a scheduling or echo fix applies; frequency reallocation is not
  required." Precision `Float106`, seed 20260821. (D)
- Typed in: conditional tables, costs, shots, floor, seed. Computed: the refusal, the screens, the
  predictions, the plan, the verdict. Sampled: 1024 shots at H1's prediction. (B §11)
- E1 cannot split H2 from H3, so the plan holds two experiments. (B §11)

### Evidence of correct decisions (B, E, H, S)

- On Sarovar et al.'s four published crosstalk scenarios, QCL names the generating model in all
  four. (E, S)
- Gravimeter: seven candidate systematics each reproduce the −5 µGal passive reading; each campaign
  names its generating cause in 166 s to 4,953 s of instrument time. Setup times are placeholders
  except one, from Karcher et al. (E)
- Eight sensing verifications against eight instrument papers: 53 checks pass. (E)
- Four exact checks over 𝔽₂ and rationals decide the [[32, 2]] toric code; the in-process simulator
  stops at 24 qubits. A weight bound of 3 rejects with margin 4/3. Gates follow Haruna's Table 1.
  (B, H)
- Composition law: ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂; Lean proves the exact case; the approximate
  law is the crate's own. Code switching at noise 1/10: measured 0.4619, bound 3.695. (B)

### Boundaries (B, D, F, L2)

- Evidence: simulated, recorded, or published with its standard error; the `qpu` feature adds a
  typed seam and a 24-qubit in-process simulator; no vendor adapter. (A, B)
- No decoder. (B)
- Caps: 7 candidates, 2^24 Choi entries, 10-qubit dense recovery. (B)
- Under L2's validity condition H4 fails as a process: probabilities sum to 1.037037 with identity
  at every node, 0.962963 with a NOT at Q1. QCL refuses it by scope, not yet for this reason.
  (F §3.1)
- Open against Lorenz's framework: an input/output leg split, a global validity check, per-parent
  faithfulness, discovery from a process operator, quantum d-separation constraints as an
  experiment family. (F §4, §6)

### Check before quoting

- **Lean counts.** A grep of `lean/THEOREM_MAP.md` on 2026-10-21 finds 22 `quantum.*` rows, 20
  marked proved. Recount.
- **van der Lugt & Lorenz.** Cite v2. `faithfulness.rs` still cites v1's Proposition 6.1 and
  Theorem 4.9(v), which v2 drops. Do not quote 4.9(v). (F §5)
- **Sarovar comparison.** Source E adds that the paper's own method, rerun on the same data, misses
  the published answer in three of four scenarios. Quote it as a measured result with its
  conditions, or omit it.

---

## Part 2: Prioritize

**Question at issue.** How does a program turn a question about a quantum system's causes into a
decision it can check step by step?

**Thesis.** QCL writes a quantum question as a subject and its candidates. Its stages apply checks
from Lorenz's quantum causal models, its types fix their order, and every verdict reports what it
examined and by what margin.

**The argument:**

1. Quantum causes need interventions, and interventions need a plan (Section 1).
2. Lorenz's framework defines valid candidates; a language adds planning and judgement (Section 2).
3. QCL's builder, types and verdict record supply both, in a compiler-enforced order (Sections 3
   to 5).
4. One run shows the language working; independent data shows its decisions holding (Sections 6
   and 7).
5. The framework reaches past the language, and the post names the gap (Section 8).

**Sections:**

1. Answers about quantum causes arrive without their reasons
2. What the theory settles, and what it leaves to a language
3. The language: a subject, candidates, four verbs
4. Types enforce the order
5. A verdict is a record
6. One question, end to end
7. Evidence that the decisions hold
8. Where the language stops
9. Close

**Balance.** The language carries Sections 2 to 5 and 8. The crosstalk run supplies the hook in
Section 1, code fragments in Sections 3 to 5, and the full run in Section 6 only. Section 7 carries
the correctness evidence the crosstalk run cannot (see the appendix).

**Spine.** Each section closes on the premise the next opens with; first and last sentences alone
should carry the argument.

---

## Part 3: The sections

### Section 1 — Answers about quantum causes arrive without their reasons

**Topic sentence.** When two qubits fail together, passive data often cannot say why, and an answer
without its reason cannot be checked.

**Supporting points:**

- Hook, one paragraph: Q1 drives Q2, Q2 drives Q1, or a shared bath drives both. A direct cause
  calls for a scheduling or echo fix; a bath may call for frequency reallocation. Markov equivalence
  lets all three fit the passive data. (C, D)
- The general shape: passive observation underdetermines structure; interventions separate it;
  tomography costs the most. (C, L1)
- Lorenz names this partial-data situation the practical problem. (L1 §13.2, L3 §5)
- Usually missing: a record of the candidates considered, the invalid ones, the separating
  experiment, and its margin. (author's assessment; label it)

**Connectors:** "for example" (hook), "in general" (shape), "in fact" (Lorenz), "yet" (what is
missing).

**Closing direction:** A checkable answer needs three things: a definition of a valid candidate, a
plan to separate candidates, and a judgement that reports its margin.

**Transition:** Lorenz's quantum causal models supply the first.

### Section 2 — What the theory settles, and what it leaves to a language

**Topic sentence.** Lorenz's quantum causal models define valid candidate structures; QCL turns each
defining condition into a check and adds planning and judgement under a budget.

**Supporting points:**

- Plain definition, once: a quantum causal model replaces a causal graph's conditional-probability
  tables with quantum channels and multiplies the per-node operators into the process operator.
  (A, L1)
- The theory-to-check table, five rows; hold the cycle row for Section 8. (A, B, L1, L3, L4, L5)
- Markov gates admission because, for acyclic graphs, it equals compatibility with an underlying
  unitary. (L1, Thm 4.10)
- The theory leaves the program to choose the explaining candidate, the experiments, the shots, the
  cost, and when to stop. (author's reading; label it)

**Connectors:** "in other words" (definition), "in particular" (Markov), "however" (what remains).

**Closing direction:** Validity comes from the theory; the plan and the verdict come from the
language.

**Transition:** What does a program in that language look like?

### Section 3 — The language: a subject, candidates, four verbs

**Topic sentence.** A QCL program names a subject and its candidates; the library refuses, screens,
plans and adjudicates.

**Supporting points:**

- Code block: the builder skeleton from `qcl_crosstalk/main.rs`:
  `QclBuilder::config::<FloatType, N>()`, `over_plant(...)`, candidates, `build()`, `validate(...)`,
  `control(...)`, `fork().design(MinCostCover::new(floor))`, `adjudicate`. `FloatType` alias, no
  raw `f64`. (C)
- One sentence per verb: `build()` refuses, `validate` screens, `design` plans, `adjudicate`
  judges. (B)
- A short table of the four subjects and their stages. (B)
- The two working types: `R` for accuracy, `N` for counts. (B)
- The campaign steps through `CausalFlow::iterate_until`, so QCL shares DeepCausality's causal
  monad. (A)

**Connectors:** "first", "then", "next", "finally" (verb order), "similarly" (subjects).

**Closing direction:** The verbs run in a fixed order; planning over an unscreened candidate would
plan over an invalid model.

**Transition:** The user need not remember that order.

### Section 4 — Types enforce the order

**Topic sentence.** QCL's types forbid planning experiments over an unscreened structural
candidate.

**Supporting points:**

- `validate` ends in `finalize()`, returning a `Screened`; `control` accepts a `Screened` or a plant
  configuration of mechanism candidates. (B)
- A plant configuration with structural candidates lacks a `ControlSource` implementation, so
  passing it to `control` fails to compile. (B)
- `build()` refuses unsound configurations first; quote the H4 refusal. (D)
- Tolerances follow `R`, so a precision change moves every threshold consistently. (B)

**Connectors:** "because" (compile failure), "in addition" (`build()`), "consequently" (thresholds).

**Closing direction:** Whatever reaches a verdict has passed every check the theory requires.

**Transition:** Next: what the verdict reports about those checks.

### Section 5 — A verdict is a record

**Topic sentence.** Every QCL check records what it examined, the threshold, the margin and the
outcome, and a check that examined nothing says so.

**Supporting points:**

- The `Check` and `CheckReport` fields; margin as measured over threshold. (B)
- `Vacuous`, one instance: `check_decomposable` on a two-node chain. (R)
- Named refusals and ambiguities: a five-row table and the five ambiguities. (B)
- Abstraction reports name their path, so nobody mistakes a numeric verdict for an exact one. (B)
- Forked worlds keep separate ledgers, compared, never summed. (B)

**Connectors:** "in particular" (`Vacuous`), "similarly" (refusals, ambiguities), "in addition"
(path, ledgers).

**Closing direction:** A verdict that reports what it skipped earns trust on what it checked.

**Transition:** Now the whole language on one question.

### Section 6 — One question, end to end

**Topic sentence.** On the crosstalk question, QCL refuses the cyclic candidate, screens three,
plans two experiments costing 2 of 5, and names the direct cause after the first.

**Supporting points:**

- Output block: run D's `[validate]`, `[design]` and `[campaign]` sections, verbatim. (D)
- Walk the output and tie each line to its construct from Sections 3 to 5. (C, D)
- E1 cannot split H2 from H3, hence two experiments. (B §11)
- Label the numbers: typed tables and costs; computed predictions, plan and verdict; sampled shots.
  (B §11)
- State the run's reach: it samples from H1's prediction, so it shows the mechanics on a known
  answer, not correctness on data it did not generate. (D; see appendix)

**Connectors:** "first" / "then" / "finally" (stages), "because" (two experiments), "to clarify"
(reach).

**Closing direction:** The run shows how QCL decides; testing whether its decisions hold takes data
the program did not generate.

**Transition:** Published experiments supply that data.

### Section 7 — Evidence that the decisions hold

**Topic sentence.** On published crosstalk and sensor data, and on codes too large for its own
simulator, QCL's decisions match the independent answer.

**Supporting points:**

- Sarovar et al.'s four scenarios: QCL names the generating model in all four. (E, S)
- Sensors: eight verifications against eight instrument papers, 53 checks pass; one sentence on the
  gravimeter's seven systematics, with the placeholder setup times stated. (E)
- Codes: exact decision of the [[32, 2]] toric code beyond the 24-qubit simulator; weight-3
  rejection at margin 4/3. (B, H)
- Abstractions: the composition bound holds, a factor of 8 above the measurement (0.4619 against
  3.695). (B)

**Connectors:** "for example" (crosstalk), "similarly" (sensors), "in contrast to simulation"
(codes), "even so" (loose bound).

**Closing direction:** The decisions hold where tested, and the tests name their limits.

**Transition:** So do the language's boundaries.

### Section 8 — Where the language stops

**Topic sentence.** QCL decides among named, acyclic candidates on simulated, recorded or published
evidence, and Lorenz's framework already reaches past each limit.

**Supporting points:**

- Evidence sources and the `qpu` seam; no vendor adapter, no decoder; caps. (A, B)
- Cycles, the held-back table row: QCL refuses by scope. Under L2, H4 fails as a process (1.037
  under identity, 0.963 under a NOT); refusing it for that physical reason comes next. (F §3.1, L2)
- Named candidates only: discovery from a process operator and quantum d-separation planning on
  partial data remain open. (F §4, §6, L1)

**Connectors:** "however" (cycles), "in fact" (H4), "similarly" (discovery, d-separation).

**Closing direction:** Each open item has a test waiting.

**Transition:** Back to the opening question.

### Section 9 — Close

**Topic sentence.** A question about two qubits became a program: four candidates, three checks
from Lorenz's theory, a two-experiment plan, and one named cause with its margin.

**Points:** the command to run `qcl_crosstalk`; the crate; the Lorenz papers; the quantum project
website. End on the language, then the run.

---

## Part 4: Verb check

**Verb chain:** fit (all three) → define (theory) → name (subject, candidates) → refuse → screen →
enforce (types) → record → plan → adjudicate → hold (on published data) → stop (at named limits).

**Replace:** is, has, can be, involves, provides, enables, allows, leverages, supports.

**Never twice as a section's main verb:** *decides*, *checks*, *shows*.

**Tense:** present for the code; past for a run.

---

## Style gates

- Lead each section with its finding.
- After first definition, one word per concept: *candidate*, *subject*, *check*, *verdict*, *plan*,
  *survivor*.
- State, do not argue: no "not A, but B", no uniqueness claims.
- At most one em dash per 250 words.
- Banned: delve, shed light, game-changer, unlock, seamless, robust, powerful, leverage, flagship,
  toy.
- `FloatType` alias in every snippet.
- Every number traces to D, E, R or B.

---

## Appendix: Critical analysis of the previous outline

Elements of thought and intellectual standards from Paul and Elder, *The Miniature Guide to
Critical Thinking*.

**Question at issue (relevance).** The old main point concerned the language, but its Section 1
asked the example's question: which structure caused these errors? Readers would judge the language
by one example. The question now belongs to the language: how a question becomes a checkable
decision.

**Concepts (clarity, significance).** The language's constructs lay scattered under example
headings: the typestate guarantee in one bullet, the four subjects in a side table, the causal
monad link absent. Sections 3 to 5 now carry them; the example illustrates.

**Inference (logic, accuracy).** The old Section 4 inferred traceability of every verdict from one
run, a hasty generalization. Traceability follows from the `CheckReport` type, so Section 5 argues
from the type and cites runs as instances.

**Evidence (relevance, depth).** The old outline rested on the crosstalk run, which samples from
H1's prediction and then names H1: it confirms mechanics on a known answer and cannot show correct
decisions on foreign data. The independent evidence (Sarovar's scenarios, the sensing
verifications, the exact code checks) sat in an afterthought section, "beyond crosstalk". Section 7
now carries it, and Section 6 states the run's reach.

**Assumptions (fairness, precision).** The old outline blurred the theory's contribution and the
language's, risking credit to QCL for Lorenz's results. The theory-to-check table now draws the
line, and Section 2 states what the theory leaves to a program.

**Breadth.** The old outline treated three of the four subjects as extras to the plant. Section 3
presents all four.

**Implications.** The limits section stays. The cycle row now travels from Section 2's table to
Section 8, so readers see the boundary coming.
