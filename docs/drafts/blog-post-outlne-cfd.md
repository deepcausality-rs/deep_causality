# CfdFlow: A Language for Counterfactual Fluid Dynamics — Blog Post Outline

A dictation outline, built with `docs/writing_guides/TurnRawMaterialintoWriting.pdf`: raw
information by source, the argument, each section's topic sentence, supporting points, closing
sentence and transition, then the verb check. The appendix records the critical analysis
(`MiniatureGuideCriticalThinking.pdf`) behind this structure.

Target: 2,800 to 3,200 words, four code blocks, two output excerpts, two tables. Readers: CFD and
GNC engineers who run Fluent, SU2, OpenFOAM or in-house codes, and DeepCausality readers. Assume CFD
knowledge, none of this project's vocabulary.

---

## Part 1: Raw information, by source

Each fact carries its source letter. Strip the tags before publishing; keep a tagged copy. Committed
`output.txt` files outrank the example READMEs and `constants.rs` comments, which carry older
numbers.

| Letter | Source |
|---|---|
| A | `deep_causality_cfd/README.md` |
| K | `deep_causality_cfd/src/types/flow/` (method names checked 2026-10-21) |
| B | `examples/avionics_examples/cfd/plasma_blackout/README.md` |
| C | `examples/avionics_examples/cfd/plasma_blackout/corridor/output.txt` and `constants.rs` |
| D | `examples/avionics_examples/cfd/plasma_blackout/weather/output.txt` |
| E | `examples/avionics_examples/cfd/plasma_blackout/retropropulsion/output.txt` |
| F | `examples/avionics_examples/cfd/{flight_envelope_placard,nozzle_operating_map,viv_resonance_margin}/` README and `output.txt` |
| G | `examples/avionics_examples/cfd/turbulence_flow/` README and `output.txt` |
| V | `deep_causality_cfd/verification/README.md`, summary table |
| W | `website/cfd/README.md`, voice and vocabulary rules |

Every wall-clock figure: Apple M3 Max, 16 cores, 128 GB, release build, `f64`.

### The problem (A, B)

- Design questions about a coupled simulation ("which bank angle?", "how much margin in cold
  weather?", "which throttle?") take many runs that share a prefix and diverge at one moment.
  (author's framing; label it)
- Often the physics finds that moment: the plasma-blackout example holds no onset constant; the
  evolved chemistry predicts the window. (B)
- Such a question must express what to couple, when to stop, what to vary from there, and how to
  judge. (author's framing, matched to A's two levels)

### The trajectory level (A, K, C)

- `CfdFlow::march(&config)` with `.couple(stack)`, `.from_field(..)` or `.from(state)`, then
  `.until(condition)` or `.run_for(steps)`. (A, K)
- `.until` returns a resumable pause once the condition holds. (A)
- The condition reads the evolved field: `field.regime()` reports the Knudsen flow regime, the
  plasma frequency, GNSS denial, and powered-descent axes. (A)
- Corridor: `.until(|f, _| f.regime().map(|r| r.gnss_denied).unwrap_or(false))` paused at step 106,
  76.8 km, Mach 26.8. (A, C)
- The Knudsen classification only diagnoses: the crate switches no closures on it and implements no
  slip, transitional or free-molecular closure. (A)

### The coupling stack (A)

- A static chain of `PhysicsStage`s steps one shared `CoupledField`:
  `Coupling::between_steps().then(..)…build()`. (A)
- Stages exchange named fields; an `Err` from any stage short-circuits the step. (A)
- Both levels use the same stack as loop body: a march couples it directly; a study couples it per
  case and draw. (A)
- Corridor stack: vibrational relaxation, finite-rate ionization, regime classification, steered
  lift, navigation (17-state error-state Kalman filter), bounded-correction safety gate. (A)

### The campaign level (A, K)

- `CfdFlow::study(name)`, `.cases(..)`, then a state fork, `.fork(&pause).branch(..)
  .continue_for(steps)`, or a world alternation, `.baseline(..).alternate(..).ensemble(draws)
  .couple(..).march_for(..)`; then `.reduce_all(..)` or `.reduce_ensemble(..)`, optional
  `.refine(&pause, ..)`, `.gates(..)`, `.verdict()`. (A, K)
- Closed-form study shape: `.matrix(..)`, `.prepare(..)`, `.sweep(..)`, `.record(..)`,
  `.gates(..)`. (F, K)
- `verdict()` returns data; the DSL never prints or exits. The gate sequence, `GateSeq<Row>`, enters
  whole as a value. (A)
- A fork of the pause (`CarrierFork`) implements `deep_causality_core`'s alternation traits
  (`AlternatableContext`, `AlternatableState`, `AlternatableValue`), each appending to the event
  log. (K: `carrier.rs`, `qtt_march_pause.rs`)

### Fork cost and guarantees (A, C, E)

- Branches share the marched state; each clones once at its first field write, so per-branch field
  cost scales with the cell count. (A)
- Fan-outs run concurrently on scoped threads and match the sequential run bit for bit. (A, B)
- Retropropulsion: all 5 branches entered by reference, no tensor copied at fork time; the fan-out
  marched at 1.16× the trunk's per-step cost. (E)
- Each branch stamps `!!ContextAlternation!!`, naming its baseline, into the event log. (A, C)
- Limit: `save_log(path)` writes per-branch files only on the `march_for` path; the
  `fork`/`branch`/`continue_for` chain writes none, though each branch keeps its full log in the
  returned report. (A)

### Configuration, solvers, precision (A, F, G)

- `flow_config` holds owned descriptions (grids, schedules, seeds, stop conditions, observables);
  `flow` runs them. A what-if hands the same flow a different description. (A)
- `CfdConfigBuilder` starts every config family; each validates at `build()`. (A)
- One language and scalar over three solver families: a discrete-exterior-calculus incompressible
  Navier-Stokes solver, compressible marchers on quantized tensor trains, and pointwise closed
  forms. (A)
- One `FloatType` alias sets precision: `f32`, `f64` or `Float106`. (A)

### Instances (C, D, E, F, G)

- Corridor, a state fork on a parameter: six coarse bank commands, then eleven fine at 0.5° from the
  same pause; 11.0° committed at 2.865 m. The 0° branch's 30.000 m miss comes from the setup
  (`AIM_CROSS_RANGE_M = 30.0`). Navigation error: 0.31 m at onset, 3.07 m at the 61 km peak, 0.20 m
  after reacquisition. Whole example: 45.9 s. (C)
- Weather, a world alternation: six atmospheres × eight draws = 48 descents; polar winter drifts
  58.20 m against standard day's 42.13 m, 3.1 sigma; whole table 179.2 s. (D)
- Retropropulsion, a state fork on a plume-coupled burn: reads the weather table in flight
  (measured −32 K → ignition margin 66.99 m); five throttle branches; preserved drag 0.4335 →
  −0.0329. The cited Jarvinen-Adams A0 correlation drives the drag change, and the de-risk verdict
  rates its fidelity AMBER. Informed guidance lit the burn 18.61 m higher and spent 10.54 kg more
  propellant; touchdown 1.80 m/s; whole descent 335.3 s. (E)
- Placard: max q 23.7 kPa inside 60 kPa; max T0 1502.1 K inside 1700 K; closed forms, no solver.
  (F)
- Nozzle operating map: a 1-D duct march, every row gated against gas-dynamics closed forms. (F)
- Vortex-shedding margin: DEC solver, four airspeeds, minimum margin 0.236 against 0.15. (F)
- Turbulence horizon, Lorenz system: t ≈ 21.5 at `f32`, 44.5 at `f64`; the law puts `Float106` near
  81. (G)

### What the solvers rest on (V, C)

- Gate labels: `[reference]` cites an analytic or published bound; `[tripwire]` pins this code's
  prior output and guards only against regression. (V)
- Lid cavity, Re 1000: primary vortex (0.5312, 0.5625) against Ghia's (0.5313, 0.5625). (V)
- Cylinder, Re 100: St 0.171 against 0.164 (+4.3 %); C_d 1.342 inside 1.314–1.370; friction share
  about 13 % against the reference's 25 %. (V)
- Sod: L1 errors 0.018 / 0.027 / 0.015 against the exact Riemann solution. (V)
- Taylor-Green, Re 1600 at 16³: peak dissipation −80 %, under-resolved. (V)
- RAM-C II: n_e −0.16 decades from the station-1 datum, inside ±0.70; blackout exit 45.7 km against
  the flight's 25–30 km. (C)
- The QTT cylinder verification fails, runs offline, and keeps its gate unwidened. (V)
- CI runs ten harnesses per pull request and three monthly. (A, V)

---

## Part 2: Prioritize

**Question at issue.** How do you ask "what if" of a running, coupled simulation without rerunning
it from the start or gluing runs together with scripts?

**Thesis.** CfdFlow answers with two levels. The trajectory level marches a coupled simulation
until a physical condition holds and returns the paused state as a value. The campaign level forks
or alternates what-ifs from that value and reduces them to a gated table, returned as data.

**The argument:**

1. A design question is a family of runs sharing a prefix and diverging where the physics says
   (Section 1).
2. The trajectory level expresses "run until the physics says so" and returns that moment
   (Sections 2 and 3).
3. The campaign level expresses "vary from there and judge", in two forms (Sections 4 and 5).
4. The fork runs cheap, concurrent, deterministic and logged, and the grammar ignores which solver
   sits underneath (Sections 6 and 7).
5. The solvers and the language state their limits (Section 8).

**Sections:**

1. A design question is a family of runs
2. The trajectory level: march until the physics says so
3. The coupling stack is the loop body
4. The campaign level: fork the pause, judge the branches
5. Two kinds of what-if
6. What a fork costs, and what it guarantees
7. One grammar, any solver, any precision
8. What the language rests on, and where it stops
9. Close

**Balance.** The language carries every topic sentence. The plasma-blackout family supplies the
Section 1 hook and the instances in Sections 2 to 6; the placard, nozzle and vortex-shedding studies
supply Section 7's. No topic sentence names a vehicle.

**Spine.** Each section closes on the premise the next opens with.

---

## Part 3: The sections

### Section 1 — A design question is a family of runs

**Topic sentence.** Most design questions put to a simulation become a family of runs that share
everything up to one moment and differ after it.

**Supporting points:**

- Hook, one paragraph: at Mach 28 a reentry vehicle ionizes the surrounding air until the plasma
  blocks every GNSS signal, and guidance must choose a bank angle in the dark. Every candidate
  shares the descent up to blackout. (B)
- The physics finds the moment: the example holds no onset constant; the evolved chemistry
  predicts it. (B)
- The general shape: which throttle, which weather, which bank angle; one shared prefix, one
  moment, many continuations. (author's framing; label it)
- A language for this must express what to couple, when to stop, what to vary, and how to judge.
  (author's framing)

**Connectors:** "for example" (hook), "in general" (shape), "therefore" (needs).

**Closing direction:** Those needs split into two levels: one run, and a family of runs.

**Transition:** The first level runs one simulation until the moment arrives.

### Section 2 — The trajectory level: march until the physics says so

**Topic sentence.** At the trajectory level, a CfdFlow program marches a coupled simulation until a
condition on the evolved field holds, then receives the paused state as a value.

**Supporting points:**

- Code block: `CfdFlow::march(&config).couple(stack).from_field(..).until(condition)?` with the
  corridor's `gnss_denied` condition. (A)
- Define *pause* once: the state a run returns when its condition holds, ready to resume, fork or
  alternate. (W, A)
- The condition reads the field's regime: Knudsen regime, plasma frequency, link state. (A)
- Instance: the corridor paused at step 106, 76.8 km, Mach 26.8. Event-log excerpt: slip →
  continuum → GNSS-denied → paused. (C)
- Qualify: the Knudsen regime only diagnoses; no rarefied-gas closure switches in. (A)

**Connectors:** "first" (march), "then" (pause), "because" (condition reads the field), "however"
(qualification).

**Closing direction:** The program holds the moment instead of receiving it as a number.

**Transition:** The march decides what the paused value holds.

### Section 3 — The coupling stack is the loop body

**Topic sentence.** A coupling stack declares which physics steps together, and both levels of the
language use it as their loop body.

**Supporting points:**

- Code block: `Coupling::between_steps().then(..)…build()` from A. (A)
- Stages share one field and exchange named fields; an error in any stage stops the step. (A)
- Instance: in the corridor, electron density gates the Kalman filter's measurements, the safety
  gate clamps the bank command, the lift stage flies it. Navigation error: 0.31 m at onset, 3.07 m
  at the 61 km peak, 0.20 m once GNSS returns. (A, C)
- A study reuses the stack unchanged, per case and draw. (A)

**Connectors:** "in particular" (corridor loop), "as a result" (navigation numbers), "similarly"
(reuse).

**Closing direction:** A pause holds flow, chemistry and navigation at one instant.

**Transition:** That state merits variation.

### Section 4 — The campaign level: fork the pause, judge the branches

**Topic sentence.** At the campaign level, a study forks the paused state into branches, continues
each, reduces them to rows, and gates the rows into a verdict returned as data.

**Supporting points:**

- Code block: `CfdFlow::study(..).cases(..).fork(&pause).branch(..).continue_for(..)
  .reduce_all(..).refine(&pause, ..).branch(..).continue_for(..).reduce_all(..).gates(..)
  .verdict()?` (A)
- Define *branch*, *table* and *gate* at first use. (W)
- `verdict()` returns data for the caller to map to an exit code; the gate sequence travels as a
  value. (A)
- `refine` runs a second round from the same pause. (A)
- Instance: seventeen branches, six coarse and eleven fine; 11.0° committed at 2.87 m. The
  ballistic 30 m comes from the setup. Output excerpt: fine-sweep rows 10.0° to 12.0°. (C)

**Connectors:** "first" / "then" (fork, continue, reduce, gate), "in addition" (refine), "to
clarify" (30 m).

**Closing direction:** One expression holds the question, the alternatives and the judgement.

**Transition:** A bank angle offers one kind of variation; the grammar offers a second.

### Section 5 — Two kinds of what-if

**Topic sentence.** CfdFlow expresses two kinds of what-if, forking a paused state and alternating a
baseline into whole worlds, and one study can feed its table to the other.

**Supporting points:**

- State fork: `fork(&pause).branch(..)` varies the future after a moment and shares everything
  before it. (A)
- World alternation: `.baseline(..).alternate(..).ensemble(draws).couple(..).march_for(..)
  .reduce_ensemble(..)` varies the world a whole run lives in. Code block from A. (A)
- A fork implements `deep_causality_core`'s alternation traits (`alternate_context`,
  `alternate_state`, `alternate_value`), the causal monad's counterfactual vocabulary. (K) Before
  dictating, check whether the `alternate` world path uses the same traits; only the fork's
  implementation is confirmed.
- Alternation instance: six atmospheres × eight draws; polar winter drifts 58.20 m against 42.13 m,
  3.1 sigma. (D)
- Hard-state fork instance: retropropulsion forks the marched plume-coupled burn into five throttle
  branches. Qualify at once: the Jarvinen-Adams correlation drives the drag change, rated AMBER on
  fidelity. (E)
- Composition: retropropulsion reads the weather table in flight and sizes its ignition margin from
  it (66.99 m); informed guidance spent 10.54 kg more propellant than standard-day guidance. (E)

**Connectors:** "on the one hand … on the other hand" (two forms), "because" (shared vocabulary),
"for instance" (instances), "even so" (AMBER), "building on" (composition).

**Closing direction:** A what-if offers either a different future from one moment or a different
world from the start, and one can feed the other.

**Transition:** A branch family helps only if it stays affordable and reproducible.

### Section 6 — What a fork costs, and what it guarantees

**Topic sentence.** A fork shares the paused state instead of copying it, runs its branches
concurrently with bit-identical results, and logs every alternation.

**Supporting points:**

- Cost model: branches share the paused state and clone once at first field write, so per-branch
  field cost scales with the cell count. (A) Give "copy-on-write" in parentheses after the plain
  statement. (W)
- Measured: retropropulsion's fan-out marched at 1.16× the trunk's per-step cost, all 5 branches
  entered by reference. (E)
- Arithmetic, labelled: the corridor's seventeen branches skip re-marching the 106-step prefix. (C;
  derived)
- Determinism: concurrent fan-outs match the sequential run bit for bit. (A, B)
- Audit: each branch stamps `!!ContextAlternation!!` with its baseline. (A, C)
- Limit: `save_log(path)` writes files only on the `march_for` path; the fork chain keeps its log in
  the returned report. (A)

**Connectors:** "in other words" (cost model), "for example" (measured), "in addition"
(determinism, audit), "however" (`save_log`).

**Closing direction:** A what-if costs only the part that differs.

**Transition:** None of this depends on the solver under the branches.

### Section 7 — One grammar, any solver, any precision

**Topic sentence.** Because configuration stays separate from execution, one study grammar drives a
full march, a 1-D duct solver or a closed form, at whatever precision one alias names.

**Supporting points:**

- `flow_config` describes; `flow` runs. A what-if hands the same flow a different description.
  `CfdConfigBuilder`, validated at `build()`, starts every config. (A)
- The three solver families, one sentence each. (A)
- Instances, a three-row table: placard (closed forms; `matrix`/`prepare`/`sweep`/`record`/`gates`;
  max q 23.7 kPa inside 60 kPa); nozzle operating map (duct march, every row gated against closed
  forms); vortex-shedding margin (DEC solver; minimum margin 0.236 against 0.15). (F)
- Precision, one alias; instance: the turbulence forecast horizon moves from 21.5 (`f32`) to 44.5
  (`f64`), and the law puts `Float106` near 81. (A, G)

**Connectors:** "because" (separation enables reuse), "for instance" (table), "in particular"
(precision).

**Closing direction:** The study keeps its shape whether each case runs a closed form or a full
march.

**Transition:** A language over solvers earns no more trust than its solvers.

### Section 8 — What the language rests on, and where it stops

**Topic sentence.** CfdFlow's solvers publish their matches and misses, and the language states its
limits where users will meet them.

**Supporting points:**

- Gate labels: `[reference]` and `[tripwire]`. (V)
- Matches and misses in one table: Ghia vortex to 1e-4; cylinder St +4.3 %, C_d −2.0 %, friction
  share 13 % against 25 %; Sod L1 under 0.03; Taylor-Green Re 1600 −80 %, under-resolved; RAM-C II
  −0.16 decades inside ±0.70, blackout exit 45.7 km against 25–30 km; QTT cylinder red and offline.
  (V, C)
- Language limits: regime classification only diagnoses; `save_log` covers only `march_for`; a
  correlation drives the retropropulsion drag. (A, E)
- CI runs ten harnesses per pull request, three monthly. (A, V)

**Connectors:** "in contrast" (matches to misses), "in addition" (language limits), "even so" (what
survives the misses).

**Closing direction:** A red gate shows readers where to stop trusting the result.

**Transition:** Back to the opening question.

### Section 9 — Close

**Topic sentence.** A design question became one expression: march until the physics says so, fork
from that moment, judge the branches, return the table.

**Points:** the three commands for the plasma-blackout family, in order; the crate on crates.io; the
CFD project website. End on the language, then the run.

---

## Part 4: Verb check

**Verb chain:** share (a prefix) → march → pause → couple → fork → alternate → reduce → gate →
return (data) → reuse (the grammar) → state (the limits).

**Replace:** is, has, can be, involves, provides, enables, allows, leverages, supports.

**Never twice as a section's main verb:** *runs*, *shows*, *demonstrates*.

**Tense:** present for the language; past for a run.

---

## Style gates (from W and the writing guides)

- Lead each section with its finding; inside an instance paragraph, show the result before its
  code.
- One word per concept: *what-if*, *pause*, *branch*, *table*, *gate*, *event log*, *condition*;
  *world* only where the code says it. Define *counterfactual* once; use it rarely.
- Keep out of the first paragraph: copy-on-write, O(1), tensor train, bond dimension, cochain, Leray
  projection, quasi-steady, causaloid.
- State, do not argue: no "not A, but B", no uniqueness claims.
- Say "the plasma-blackout example", never "flagship"; never "toy".
- At most one em dash per 250 words; no paragraph opens with *Additionally*, *Furthermore* or
  *Moreover*.
- Every wall-clock number names the M3 Max.
- Copy numbers from `output.txt`, never from example READMEs or `constants.rs` comments.
- `FloatType` alias in every snippet.

---

## Appendix: Critical analysis of the previous outline

Elements of thought and intellectual standards from Paul and Elder, *The Miniature Guide to
Critical Thinking*.

**Purpose and question at issue (relevance, significance).** The old outline meant to present
CfdFlow but asked the example's question: where does the vehicle lose GPS, and which bank angle
wins? Five of eight topic sentences named the vehicle, so a skimming reader met a reentry story. The
question now belongs to the language, and no topic sentence names a vehicle.

**Point of view (fairness, breadth).** The old structure borrowed the CFD website's view, built to
let a cold reader judge a result in seconds. A post named after the language must explain the
language; results now serve as instances.

**Concepts (clarity, depth).** The language's constructs (the pause as a value, the stack as loop
body, the two kinds of what-if, `verdict()` returning data, configuration apart from execution, the
alternation vocabulary shared with the causal monad) lay as bullets under example headings. Each now
carries a section. The old outline omitted the link to `deep_causality_core`'s alternation traits.

**Information and relevance.** The old evidence section measured solver accuracy (Ghia, Sod,
cylinder), which bears on the solvers, not the language. The language's own evidence (1.16× fan-out
step cost, branches entered by reference, bit-identical concurrency, alternation markers, one
grammar over closed forms and solvers) lay scattered. Sections 6 and 7 now carry it; Section 8
scopes solver validation as the language's foundation.

**Inference (logic, accuracy).** The old thesis claimed a study replaces "a stack of reruns", but no
artifact measures a rerun baseline. The outline now states the measurement (1.16× fan-out step
cost) and labels the prefix count as arithmetic.

**Assumptions.** The old outline assumed a vivid example makes the strongest opening. For this
audience it makes a strong hook, so it stays: one paragraph in Section 1.

**Implications.** The old outline omitted the language's own limits (`save_log` covers only
`march_for`). Sections 6 and 8 now state them.
