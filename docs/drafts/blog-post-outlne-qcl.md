# QCL: The Quantum Causal Language — Blog Post Outline

Outline for dictation. Built with `docs/writing_guides/TurnRawMaterialintoWriting.pdf`: raw
information by source, then the prioritized argument, then one block per section with its topic
sentence, connected supporting points, closing sentence and transition, then the verb check. The
appendix records the critical analysis (`MiniatureGuideCriticalThinking.pdf`) that shaped this
structure.

Target: about 2,800 to 3,200 words, four code or output blocks, two tables. Audience: engineers
and researchers who work on quantum hardware or quantum sensors, and DeepCausality readers who
have not met quantum causal models. Assume they know a qubit and a density matrix; do not assume
they know Lorenz's framework.

---

## Part 1: Raw information, by source

Every fact carries the letter of its source. Strip the tags from the published post; keep a
tagged copy beside it.

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

All Lorenz PDFs are in `openspec/notes/quantum/Lorenz/`.

### The problem the language addresses (C, L1, L3, L7)

- Passive data underdetermines causal structure: in the crosstalk case three structures fit the
  passive observation exactly, because they are Markov equivalent. (C)
- Telling them apart takes interventions; full process tomography is the most expensive one. (C,
  L1)
- Lorenz names partial interventional data as the practical problem: discovery when "the given data
  is not a (full) process operator". (L1 §13.2, L3 §5, L7 outlook; F)

### The theory, and the check each part becomes (A, B, F, L1 to L5)

| Lorenz's framework | QCL check or construct |
|---|---|
| A model is a product of per-node Choi operators, the process operator (L1) | a structural candidate: factors plus the Hilbert legs each acts on (B) |
| Factors on intersecting supports must commute: the quantum Markov condition (L3, Def. 3.3) | `check_markov`, ‖[ρⱼ, ρₖ]‖_F against a threshold derived from the scalar's epsilon (B) |
| For acyclic graphs, Markov ⇔ compatibility with a unitary (L1, Thm 4.10) | why `check_markov` is the gate for admission (F) |
| C₃-exclusion characterises which structures traditional circuits can represent (L4, Thm 3.2) | `check_decomposable` (B) |
| Abstraction: a high-level model summarises a low-level one; naturality squares (L5) | the abstraction subject, `check_naturality`, the composition law (B) |
| Cycles: a product of commuting channels is no longer automatically a valid process (L2) | `build()` refuses cycles by scope today (D) |

- The theory defines what a valid model is. It does not say which valid candidate explains the
  data, which experiments to run, or at what cost. (author's reading of L1 to L5; label it)

### The language (A, B)

- `QclBuilder` is an embedded Rust builder. A program names a subject and its candidates; the
  library refuses, screens, plans and adjudicates. (B)
- Pipeline: `config(subject, candidates) → validate(checks) → Screened → control(evidence) →
  report`. (B)
- Four subjects: `over_plant` (validate, then control), `over_model` (validate), `over_code`
  (validate only), `over_circuit` (validate on the circuit's dilation). (B)
- Two working types named once at `QclBuilder::config::<R, N>()`: `R: RealField` for accuracy,
  every tolerance derived from `R::epsilon()`; `N: NaturalNumber` for count headroom. Switching
  `R` between `f32`, `f64`, `Float106` and `BFloat16` recomputes every threshold. (B)
- `design` finds the minimum-cost set of experiments whose best separation covers every pair of
  candidates at or above a floor in bits; exact dynamic program over 2^C(n,2) subsets; refuses more
  than 7 candidates by default. Separation is the Bhattacharyya distance of n draws,
  n · (−log₂(√(pq) + √((1−p)(1−q)))). (B)
- `adjudicate` names a survivor only when exactly one world holds and separates from every rival
  by the floor; otherwise a named ambiguity. (B)
- The campaign runs the cheapest planned experiment first, re-plans when the context moves a
  prediction, and stops at a survivor, outside the model, or when nothing left resolves the rest.
  It is a step for `CausalFlow::iterate_until`, the causal monad's flow. (A)
- Forked worlds keep separate ledgers that are compared, never summed. (B)

### What the types enforce (A, B, D)

- A structural candidate cannot reach `control` without `validate`: a plant configuration with
  structural candidates has no `ControlSource` implementation. The compiler, not a runtime check,
  refuses the shortcut. (B)
- `build()` refuses what would answer unsoundly before any stage runs. (B, D)
- `validate` ends in `finalize()`, which returns a `Screened`; only a `Screened` enters `control`.
  (B)

### What a verdict carries (B, R)

- `Check`: item, measured quantity, threshold, margin (measured over threshold, rejected above 1),
  accepted. `CheckReport`: the checks, the count examined, a verdict of `Accepted`, `Rejected`, or
  `Vacuous` when nothing was examined. (B)
- `check_decomposable` on a two-node chain examines no 3 × 3 block and reads `Vacuous`. (R)
- Every report on an abstraction names its path, exact or numeric. (B)
- Refusals by name: `CyclicStructureUnsupported`, `CommutatorNonZero`, `BoundaryNotHeld`,
  `SectionNotInverse`, `NaturalityDimensionExceeded`, a refused cross-block gate. (B §9)
- Ambiguities by name: no survivor, several survivors, a survivor not separated, non-commuting
  projections, a single world. (B)

### The crosstalk run, 2026-10-21 (C, D)

- Candidates: H1 Q1→Q2, H2 Q2→Q1, H3 a shared bath, H4 the cycle Q1→Q2→B→Q1. (C)
- `build()` refuses H4: "cyclic causal structures are outside v1's scope by decision, not because
  they fail a criterion". (D)
- Screens: normalization 7 examined, Markov 5, decomposable 3, all accepted. (D)
- Experiments and predictions under H1/H2/H3: E0 passive cost 1, 0.04/0.04/0.04; E1 do(Q1=|1⟩)
  cost 1, 0.40/0.10/0.10; E2 do(Q2=|1⟩) cost 1, 0.10/0.40/0.10; E3 echo cost 2, 0.01/0.01/0.04.
  (D)
- Plan: E1 and E2, cost 2 of 5; tightest pair at 99.5 bits against a 5-bit floor. (D)
- Campaign on observations drawn from H1: E1 observed 0.396 ± 0.015 over 1024 shots; H1 survives
  at 99.5 bits after one experiment. "a scheduling or echo fix applies; frequency reallocation is
  not required." Precision `Float106`, seed 20260821. (D)
- Typed in: the conditional tables, the costs, shots, floor, seed. Computed: the refusal, the
  screens, every prediction, the plan, the verdict. Sampled: the 1024 shots, at H1's own
  prediction. (B §11)
- E1 cannot separate H2 from H3; that is why the plan holds two experiments. (B §11)

### Evidence the decisions are right (B, E, H, S)

- Published crosstalk data: on Sarovar et al.'s four scenarios, QCL names the generating crosstalk
  model in all four. (E, S)
- Gravimeter: seven candidate systematics each reproduce the −5 µGal passive reading; each
  campaign names the cause its observations came from, in 166 s to 4,953 s of instrument time.
  Setup times are placeholders except one, derived from Karcher et al. (E)
- Sensing verifications: eight, 53 checks, all pass, against eight published instrument papers.
  (E)
- Codes: the [[32, 2]] toric code decided by four exact checks over 𝔽₂ and rationals; the
  in-process simulator stops at 24 qubits. A weight bound of 3 rejects with margin 4/3. Gates
  follow Haruna's Table 1. (B, H)
- Composition law: ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂; exact case in Lean, approximate law is the
  crate's own statement. Code switching with noise 1/10: measured 0.4619, bound 3.695. (B)

### Boundaries (B, D, F, L2)

- Evidence is simulated, recorded, or a published value with its standard error; the `qpu` feature
  is a typed seam with a 24-qubit in-process simulator; no vendor adapter ships. (A, B)
- No decoder. (B)
- Caps: 7 candidates, 2^24 Choi entries, 10-qubit dense recovery. (B)
- Under L2's validity condition H4 is not a valid process: probabilities sum to 1.037037 with
  identity at every node and 0.962963 with a NOT at Q1. QCL refuses it by scope, not yet for this
  physical reason. (F §3.1)
- Open against Lorenz's framework: input/output leg split, global validity check, per-parent
  faithfulness, discovery from a process operator, quantum d-separation constraints as an
  experiment family. (F §4, §6)

### Check before quoting

- **Lean counts.** A grep of `lean/THEOREM_MAP.md` on 2026-10-21 shows 22 rows named `quantum.*`,
  20 marked proved. Recount before quoting.
- **van der Lugt & Lorenz.** Cite v2. `faithfulness.rs` still cites v1's Proposition 6.1 and
  Theorem 4.9(v), which v2 drops. Do not quote Theorem 4.9(v). (F §5)
- **Sarovar comparison.** Source E also records that the paper's own detection method, rerun on
  the same data, misses the published answer in three of four. That is a comparison against
  another method; state it only as a measured result with its conditions, or leave it out.

---

## Part 2: Prioritize

**Question at issue.** How does a program turn a question about a quantum system's causes into a
decision whose every step can be checked?

**Main point (thesis).** QCL is a language in which a quantum question is written as a subject and
its candidates. Its stages are checks taken from Lorenz's quantum causal models, its types fix the
order of those stages, and every verdict reports what it examined and by what margin.

**The argument in five steps:**

1. Answers about quantum causes need interventions, and interventions need a plan (Section 1).
2. Lorenz's framework says which candidates are valid; a language has to add the planning and the
   judgement (Section 2).
3. QCL's builder, types and verdict record supply those, in an order the compiler enforces
   (Sections 3 to 5).
4. One run shows the whole language working; independent data shows its decisions holding
   (Sections 6 and 7).
5. The framework reaches further than the language today, and the gap is named (Section 8).

**Nine sections:**

1. Answers about quantum causes arrive without their reasons
2. What the theory settles, and what it leaves to a language
3. The language: a subject, candidates, four verbs
4. The order is enforced by types
5. A verdict is a record
6. One question, end to end
7. Evidence that the decisions hold
8. Where the language stops
9. Close

**Balance.** The language carries Sections 2 to 5 and 8. The crosstalk run appears as a hook in
Section 1, as code fragments in Sections 3 to 5, and in full only in Section 6. Section 7 carries
the evidence of correctness, which the crosstalk run cannot carry (see the appendix).

**Logical spine.** Each section closes on the premise the next opens with. Read only the first and
last sentence of each section; the argument should still hold.

---

## Part 3: The sections

### Section 1 — Answers about quantum causes arrive without their reasons

**Topic sentence (main point).** When two qubits fail together, the passive data often cannot say
why, and an answer that skips the reason cannot be checked.

**Supporting points:**

- The hook, one paragraph: Q1 drives Q2, Q2 drives Q1, or a shared bath drives both. The fix
  differs: a scheduling or echo fix for a direct cause, possibly frequency reallocation for a
  bath. All three fit the passive data, because they are Markov equivalent. (C, D)
- The general shape: passive observation underdetermines structure; interventions separate it;
  full tomography is the most expensive intervention. (C, L1)
- Lorenz names this situation, partial interventional data, as the practical problem. (L1 §13.2,
  L3 §5)
- What is usually missing: the record of which candidates were considered, which were invalid,
  which experiment separated the rest, and by how much. (author's assessment; label it)

**Connectors:** "for example" (the hook), "in general" (the shape), "in fact" (Lorenz names it),
"yet" (what is missing).

**Closing sentence direction:** A checkable answer needs a definition of a valid candidate, a plan
for telling candidates apart, and a judgement that reports its margin.

**Transition to Section 2:** The first of those three already exists, in Lorenz's quantum causal
models.

### Section 2 — What the theory settles, and what it leaves to a language

**Topic sentence.** Lorenz's quantum causal models define which candidate structures are valid; QCL
turns each defining condition into a check and adds what the theory does not cover: planning and
judgement under a budget.

**Supporting points:**

- Define once, plainly: a quantum causal model swaps the conditional-probability tables of a causal
  graph for quantum channels; the model is the product of per-node operators, the process
  operator. (A, L1)
- Show the theory-to-check table from Part 1 (five rows; leave the cycle row for Section 8). (A, B,
  L1, L3, L4, L5)
- The Markov condition is the admission gate because, for acyclic graphs, it is equivalent to
  compatibility with an underlying unitary. (L1, Thm 4.10)
- What the theory leaves open for a program: which valid candidate explains the data, which
  experiments to run, how many shots, at what cost, and when to stop. (author's reading; label it)

**Connectors:** "in other words" (the plain definition), "in particular" (Markov as the gate),
"however" (what the theory leaves open).

**Closing sentence direction:** Validity comes from the theory; the plan and the verdict have to
come from the language.

**Transition to Section 3:** So what does a program in that language look like?

### Section 3 — The language: a subject, candidates, four verbs

**Topic sentence.** A QCL program names a subject and its candidates, and the library refuses,
screens, plans and adjudicates.

**Supporting points:**

- Code block: the builder skeleton, condensed from `qcl_crosstalk/main.rs`:
  `QclBuilder::config::<FloatType, N>()`, `over_plant(...)`, the candidates, `build()`,
  `validate(...)`, `control(...)`, `fork().design(MinCostCover::new(floor))`, `adjudicate`. Keep
  the `FloatType` alias; no raw `f64`. (C)
- The four verbs, one sentence each: refuse at `build()`; screen at `validate`; plan at `design`;
  adjudicate at `adjudicate`. (B)
- The four subjects in a short table: plant, model, code, circuit, with the stages each offers.
  (B)
- The two working types: `R` for accuracy, every tolerance from `R::epsilon()`; `N` for counts.
  (B)
- The campaign is a step for `CausalFlow::iterate_until`, so QCL runs on the same causal monad as
  the rest of DeepCausality. (A)

**Connectors:** "first", "then", "next", "finally" (the verbs are ordered), "similarly" (the four
subjects).

**Closing sentence direction:** The verbs come in a fixed order, and planning on an unscreened
candidate would be planning on an invalid model.

**Transition to Section 4:** The order is not a convention the user has to remember.

### Section 4 — The order is enforced by types

**Topic sentence.** QCL's types make it impossible to plan experiments over a structural candidate
that has not been screened.

**Supporting points:**

- `validate` ends in `finalize()`, which returns a `Screened`; `control` accepts a `Screened` or a
  plant configuration whose candidates are mechanisms. (B)
- A plant configuration with structural candidates has no `ControlSource` implementation, so
  passing it to `control` fails to compile. (B)
- `build()` refuses what would answer unsoundly before any stage runs; show the H4 refusal text.
  (D)
- Tolerances are a function of `R`, so changing precision changes every threshold consistently.
  (B)

**Connectors:** "because" (why the shortcut fails to compile), "in addition" (the `build()`
refusal), "consequently" (thresholds follow the scalar).

**Closing sentence direction:** What reaches a verdict has passed every check the theory requires.

**Transition to Section 5:** What the verdict says about those checks is the next question.

### Section 5 — A verdict is a record

**Topic sentence.** Every QCL check returns a record of what it examined, the threshold, the margin
and the outcome, and a check that examined nothing says so.

**Supporting points:**

- The `Check` and `CheckReport` fields; margin as measured over threshold. (B)
- `Vacuous` with one instance: `check_decomposable` on a two-node chain examines no 3 × 3 block.
  (R)
- Refusal and ambiguity by name: a five-row table of refusals and the five ambiguities. (B)
- Abstraction reports name their path, exact or numeric, so a numeric verdict is not read as
  exact. (B)
- Forked worlds keep separate ledgers, compared and never summed. (B)

**Connectors:** "in particular" (`Vacuous`), "similarly" (refusal and ambiguity), "in addition"
(path and ledgers).

**Closing sentence direction:** A verdict that reports what it did not examine can be trusted on
what it did.

**Transition to Section 6:** Here is the whole language on one question.

### Section 6 — One question, end to end

**Topic sentence.** On the crosstalk question, QCL refuses the cyclic candidate, screens three,
plans two experiments at cost 2 of 5, and names the direct cause after the first.

**Supporting points:**

- Output block: the `[validate]`, `[design]` and `[campaign]` sections of run D, verbatim. (D)
- Read the output stage by stage and point back to the construct from Sections 3 to 5 that printed
  each line. (C, D)
- Why two experiments: E1 cannot separate H2 from H3. (B §11)
- Label the numbers: tables and costs typed in; predictions, plan and verdict computed; shots
  sampled. (B §11)
- State what the run shows and what it does not: the observations are drawn from H1's own
  prediction, so the run shows the language's mechanics on a case with a known answer. It is not
  evidence that QCL finds the right answer on data it did not generate. (D; see appendix)

**Connectors:** "first" / "then" / "finally" (the stages), "because" (two experiments), "to
clarify" (what the run shows).

**Closing sentence direction:** The run shows how a decision is made; whether decisions hold needs
data the program did not generate.

**Transition to Section 7:** That data exists, from published experiments.

### Section 7 — Evidence that the decisions hold

**Topic sentence.** On published crosstalk and sensor data, and on codes too large for its own
simulator, QCL's decisions match the independent answer.

**Supporting points:**

- Published crosstalk: on Sarovar et al.'s four scenarios, QCL names the generating model in all
  four. (E, S)
- Sensors: eight verifications against eight instrument papers, 53 checks, all pass; one sentence
  on the gravimeter (seven systematics, each campaign names its cause), with the placeholder setup
  times stated. (E)
- Codes: the [[32, 2]] toric code decided exactly, past the 24-qubit simulator; the weight-3
  rejection with margin 4/3. (B, H)
- Abstractions: the composition bound holds and can sit a factor of 8 above the measurement
  (0.4619 against 3.695). (B)

**Connectors:** "for example" (crosstalk), "similarly" (sensors), "in contrast to simulation"
(codes decided exactly), "even so" (the loose bound).

**Closing sentence direction:** The decisions hold where they have been tested, and the tests name
their own limits.

**Transition to Section 8:** So do the language's boundaries.

### Section 8 — Where the language stops

**Topic sentence.** QCL decides among named, acyclic candidates on simulated, recorded or published
evidence, and Lorenz's framework already reaches past each of those limits.

**Supporting points:**

- Evidence sources and the `qpu` seam; no vendor adapter; no decoder; caps. (A, B)
- Cycles: the last row of the theory table. QCL refuses by scope. Under L2, H4 is not a valid
  process: 1.037 under identity, 0.963 under a NOT. Refusing it for that physical reason is the next
  step. (F §3.1, L2)
- Named candidates only: Lorenz's discovery from a process operator, and quantum d-separation
  constraints as a way to plan on partial data, are the open list. (F §4, §6, L1)

**Connectors:** "however" (cycles), "in fact" (H4 invalid), "similarly" (discovery and
d-separation).

**Closing sentence direction:** Each open item has a test waiting for it.

**Transition to Section 9:** Back to the question the post opened with.

### Section 9 — Close

**Topic sentence.** A question about two qubits became a program: four candidates, three checks
from Lorenz's theory, a plan of two experiments, and one named cause with its margin.

**Points:** one command to run `qcl_crosstalk`; the crate; the Lorenz papers as further reading;
the quantum project website. End on the language, then the run.

---

## Part 4: Verb check (step 4 of the guide)

**Verb chain that tells the whole story:** fit (all three) → define (the theory) → name (subject,
candidates) → refuse → screen → enforce (types) → record → plan → adjudicate → hold (on published
data) → stop (at named limits).

**Weak verbs to replace:** is, has, can be, involves, provides, enables, allows, leverages,
supports.

**Verbs that must not repeat as a section's main verb:** *decides*, *checks*, *shows*.

**Tense:** present for what the code does; past for what a run did.

---

## Style gates

- Each section leads with its finding.
- One word per concept after first definition: *candidate*, *subject*, *check*, *verdict*, *plan*,
  *survivor*.
- State, do not argue: no "not A, but B", no uniqueness claims.
- Em dashes at most one per 250 words.
- Banned: delve, shed light, game-changer, unlock, seamless, robust, powerful, leverage, flagship,
  toy.
- `FloatType` alias in every snippet.
- Every number traces to D, E, R or B.

---

## Appendix: Critical analysis of the previous outline

Applied with the elements of thought and the intellectual standards of Paul and Elder,
*The Miniature Guide to Critical Thinking*.

**Question at issue (relevance).** The previous outline's stated main point was about the language,
but its Section 1 asked the example's question ("which structure caused these errors?"). A post
whose opening question belongs to one example invites the reader to judge the language by that
example alone. The question at issue is now the language's: how a question becomes a checkable
decision.

**Concepts (clarity, significance).** The language's constructs were scattered as bullets under
example-shaped headings: the typestate guarantee was one bullet in Section 3, the four subjects a
side table, the campaign's link to the causal monad absent. Each construct now carries a section
(3 to 5), and the example illustrates them.

**Information and inference (logic, accuracy).** The previous Section 4 argued that every verdict is
traceable from one run. That generalizes from a single instance. The claim is a property of the
`CheckReport` type, so Section 5 now argues from the type and uses runs as instances.

**Evidence (relevance, depth).** The previous outline leaned on the crosstalk run as its central
evidence. The run samples its observations from H1's prediction and then names H1, so it confirms
the mechanics on a case with a known answer; it cannot show that QCL decides correctly on data it
did not generate. The independent evidence (Sarovar's published scenarios, the eight sensing
verifications, the exact code checks) sat in a section titled "beyond crosstalk", as an
afterthought. It now carries its own section (7), and Section 6 states the limit of the run.

**Assumptions (fairness, precision).** The previous outline did not separate the theory's
contribution from the language's. That risks crediting QCL with what Lorenz proved. The
theory-to-check table now draws the line, and Section 2 says what the theory leaves to a program.

**Breadth.** Four subjects exist; the previous outline treated three as extras to the plant subject.
Section 3 now presents all four as part of the language.

**Implications.** The limits section was sound and is kept. The cycle row is now carried from the
theory table in Section 2 to Section 8, so the reader sees the boundary coming.
