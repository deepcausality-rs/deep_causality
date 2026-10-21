# CfdFlow: A Language for Counterfactual Fluid Dynamics — Blog Post Outline

Outline for dictation. Built with `docs/writing_guides/TurnRawMaterialintoWriting.pdf`: raw
information by source, then the prioritized argument, then one block per section with its topic
sentence, connected supporting points, closing sentence and transition, then the verb check. The
appendix records the critical analysis (`MiniatureGuideCriticalThinking.pdf`) that shaped this
structure.

Target: about 2,800 to 3,200 words, four code blocks, two output excerpts, two tables. Audience: a
working CFD or GNC engineer who runs Fluent, SU2, OpenFOAM or an in-house code, plus DeepCausality
readers. Assume CFD knowledge; assume none of this project's vocabulary.

---

## Part 1: Raw information, by source

Every fact carries the letter of its source. Strip the tags from the published post; keep a
tagged copy beside it. Committed `output.txt` files outrank the example READMEs and the comments
in `constants.rs`, which carry older numbers.

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

Machine for every wall-clock figure: Apple M3 Max, 16 cores, 128 GB, release build, `f64`.

### The problem a language has to address (A, B)

- A design question about a coupled simulation ("which bank angle?", "how much margin in cold
  weather?", "which throttle?") is answered by many runs that share a prefix and differ after one
  moment. (author's framing; label it)
- The moment that matters is often found by the physics, not set in advance: in the
  plasma-blackout example no onset constant exists; the evolved chemistry predicts the blackout
  window. (B)
- What such a question needs expressed: what to couple, when to stop, what to vary from there, how
  to judge the outcomes. (author's framing, matched to A's two levels)

### The trajectory level (A, K, C)

- `CfdFlow::march(&config)` with `.couple(stack)`, `.from_field(..)` or `.from(state)`, then
  `.until(condition)` or `.run_for(steps)`. (A, K)
- `.until` returns a resumable pause when the condition becomes true. (A)
- The condition reads the evolved field: `field.regime()` returns the flow regime from the Knudsen
  number, the plasma frequency, whether GNSS is denied, and powered-descent axes. (A)
- Corridor instance: `.until(|f, _| f.regime().map(|r| r.gnss_denied).unwrap_or(false))` paused at
  step 106, 76.8 km, Mach 26.8. (A, C)
- The Knudsen classification is a diagnostic: the crate does not switch closures on it, and no
  slip, transitional or free-molecular closure is implemented. (A)

### The coupling stack (A)

- A static chain of `PhysicsStage`s stepping one shared `CoupledField`:
  `Coupling::between_steps().then(..)…build()`. (A)
- Stages communicate through named fields; an `Err` from any stage short-circuits the step. (A)
- The same stack is the loop body at both levels: a march couples it directly; a study couples it
  per case and per draw. (A)
- Corridor stack: vibrational relaxation, finite-rate ionization, regime classification, steered
  lift, navigation (17-state error-state Kalman filter), bounded-correction safety gate. (A)

### The campaign level (A, K)

- `CfdFlow::study(name)` with `.cases(..)`, then either a state fork, `.fork(&pause).branch(..)
  .continue_for(steps)`, or a world alternation, `.baseline(..).alternate(..).ensemble(draws)
  .couple(..).march_for(..)`; then `.reduce_all(..)` or `.reduce_ensemble(..)`, optional
  `.refine(&pause, ..)`, `.gates(..)`, `.verdict()`. (A, K)
- A closed-form study shape: `.matrix(..)`, `.prepare(..)`, `.sweep(..)`, `.record(..)`,
  `.gates(..)`. (F, K)
- `verdict()` returns data; the DSL never prints or exits. The gate sequence is a named value
  (`GateSeq<Row>`) inserted whole. (A)
- A fork of the pause (`CarrierFork`) implements the causal monad's alternation traits from
  `deep_causality_core` (`AlternatableContext`, `AlternatableState`, `AlternatableValue`), each
  appending to the event log. (K: `carrier.rs`, `qtt_march_pause.rs`)

### What a fork costs and guarantees (A, C, E)

- A fork shares the marched state without copying it; each branch takes one clone at its first
  field write, so per-branch field cost is proportional to the cell count. (A)
- Fan-outs run concurrently on scoped threads and produce bits identical to the sequential run.
  (A, B)
- Retropropulsion: 5 of 5 branches entered by reference, no tensor copied at fork time; the fan-out
  marched at 1.16× the trunk's per-step cost. (E)
- Every branch stamps `!!ContextAlternation!!` naming its baseline into the event log. (A, C)
- Limit: `save_log(path)` writes per-branch files only on the `march_for` path; the
  `fork`/`branch`/`continue_for` chain writes no files, though each branch still carries its full
  log in the returned report. (A)

### Configuration separate from execution; one grammar over several solvers (A, F, G)

- `flow_config` holds owned descriptions (grids, schedules, seeds, stop conditions, observables);
  `flow` materializes runs from them. A what-if is the same flow handed a different description.
  (A)
- `CfdConfigBuilder` is the one entry for every config family; each validates at `build()`. (A)
- Three solver families behind the same language and scalar: a discrete-exterior-calculus
  incompressible Navier-Stokes solver, compressible marchers on quantized tensor trains, and
  pointwise closed forms. (A)
- Precision is one `FloatType` alias: `f32`, `f64` or `Float106`. (A)

### Examples as instances (C, D, E, F, G)

- Corridor (state fork on a parameter): coarse six bank commands, fine eleven at 0.5° from the same
  pause; 11.0° committed at 2.865 m. The 0° branch's 30.000 m miss is set by construction
  (`AIM_CROSS_RANGE_M = 30.0`). Navigation error 0.31 m at onset, 3.07 m at the 61 km peak, 0.20 m
  after reacquisition. Whole example 45.9 s. (C)
- Weather (world alternation): six atmospheres × eight draws = 48 descents; polar winter drift
  58.20 m against standard day 42.13 m, 3.1 sigma; whole table 179.2 s. (D)
- Retropropulsion (state fork on a plume-coupled burn): reads the weather table in flight (measured
  −32 K → ignition margin 66.99 m); five throttle branches; preserved drag 0.4335 → −0.0329. The
  drag change comes from the cited Jarvinen-Adams A0 correlation, and the de-risk verdict on its
  fidelity is AMBER. Informed guidance lit the burn 18.61 m higher and spent 10.54 kg more
  propellant; touchdown 1.80 m/s; whole descent 335.3 s. (E)
- Placard: max q 23.7 kPa inside 60 kPa; max T0 1502.1 K inside 1700 K; closed forms, no solver.
  (F)
- Nozzle operating map: 1-D duct march; every row gated against gas-dynamics closed forms. (F)
- Vortex-shedding margin: DEC solver, four airspeeds, minimum margin 0.236 against 0.15. (F)
- Turbulence horizon on the Lorenz system: t ≈ 21.5 at `f32`, 44.5 at `f64`; the law puts
  `Float106` near 81. (G)

### What the solvers under the language rest on (V, C)

- Gate labels: `[reference]` (analytic or published bound) and `[tripwire]` (pinned from this
  code's prior output; non-regression only). (V)
- Lid cavity Re 1000: primary vortex (0.5312, 0.5625) against Ghia (0.5313, 0.5625). (V)
- Cylinder Re 100: St 0.171 against 0.164 (+4.3 %), C_d 1.342 inside 1.314–1.370; friction share
  about 13 % against the reference's 25 %. (V)
- Sod: L1 errors 0.018 / 0.027 / 0.015 against the exact Riemann solution. (V)
- Taylor-Green Re 1600 at 16³: peak dissipation −80 %, under-resolved. (V)
- RAM-C II: n_e −0.16 decades from the station-1 datum, inside a ±0.70 allowance; blackout exit at
  45.7 km against the flight's 25–30 km. (C)
- QTT cylinder verification is known-failing, offline, and the gate is not widened. (V)
- CI runs ten harnesses per pull request and three monthly. (A, V)

---

## Part 2: Prioritize

**Question at issue.** How do you ask "what if" of a running, coupled simulation without rerunning
it from the start and without a script that glues the runs together?

**Main point (thesis).** CfdFlow is a two-level language for that question. The trajectory level
marches a coupled simulation until a physical condition holds and returns the paused state as a
value. The campaign level forks or alternates what-ifs from that value and reduces them to a gated
table, returned as data.

**The argument in five steps:**

1. A design question is a family of runs that share a prefix and diverge at a moment the physics
   finds (Section 1).
2. The trajectory level expresses "run until the physics says so" and hands back that moment
   (Sections 2 and 3).
3. The campaign level expresses "vary from there and judge the outcomes", in two forms (Sections 4
   and 5).
4. The fork is cheap, concurrent, deterministic and logged, and the grammar does not depend on
   which solver sits underneath (Sections 6 and 7).
5. The language is only as trustworthy as its solvers and its own limits, and both are stated
   (Section 8).

**Nine sections:**

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
hook (Section 1) and the instances in Sections 2 to 6; the placard, nozzle and vortex-shedding
studies supply the instances in Section 7. No section's topic sentence names a vehicle.

**Logical spine.** Each section closes on the premise the next opens with.

---

## Part 3: The sections

### Section 1 — A design question is a family of runs

**Topic sentence (main point).** Most design questions asked of a simulation are a family of runs
that share everything up to one moment and differ after it.

**Supporting points:**

- The hook, one paragraph: a reentry vehicle at Mach 28 ionizes the air around it until the plasma
  blocks every GNSS signal; guidance must choose a bank angle in the dark. Every candidate bank
  angle shares the descent up to the blackout. (B)
- The moment is found by the physics: no onset constant exists in the example; the evolved
  chemistry predicts it. (B)
- The general shape: which throttle, which weather, which bank angle; a shared prefix, one moment,
  many continuations. (author's framing; label it)
- What a language for this has to express: what to couple, when to stop, what to vary from there,
  how to judge. (author's framing)

**Connectors:** "for example" (the hook), "in general" (the shape), "therefore" (what the language
must express).

**Closing sentence direction:** Those four needs split into two levels: one run, and a family of
runs.

**Transition to Section 2:** The first level runs one simulation until the moment arrives.

### Section 2 — The trajectory level: march until the physics says so

**Topic sentence.** At the trajectory level, a CfdFlow program marches a coupled simulation until a
condition on the evolved field becomes true, and receives the paused state as a value.

**Supporting points:**

- Code block: `CfdFlow::march(&config).couple(stack).from_field(..).until(condition)?` with the
  corridor's `gnss_denied` condition. (A)
- Define *pause* once: the state a run returns when its condition becomes true; it can be resumed,
  forked or alternated. (W, A)
- The condition reads the field's regime: Knudsen regime, plasma frequency, link state. (A)
- Instance: the corridor paused at step 106, 76.8 km, Mach 26.8. Event-log excerpt: slip →
  continuum → GNSS-denied → paused. (C)
- Qualify: the Knudsen regime is logged as a diagnostic; no rarefied-gas closure switches in. (A)

**Connectors:** "first" (march), "then" (pause), "because" (the condition reads the field),
"however" (the diagnostic qualification).

**Closing sentence direction:** The moment becomes something the program holds, not a number it
was given.

**Transition to Section 3:** What the paused value holds depends on what was marched.

### Section 3 — The coupling stack is the loop body

**Topic sentence.** A coupling stack declares which physics steps together, and the same stack is
the loop body at both levels of the language.

**Supporting points:**

- Code block: `Coupling::between_steps().then(..)…build()` from A. (A)
- Stages share one field and communicate through named fields; an error in any stage stops the
  step. (A)
- Instance: in the corridor, electron density gates which measurements the Kalman filter may use,
  the safety gate clamps the bank command, the lift stage flies it. Navigation error 0.31 m at
  onset, 3.07 m at the 61 km peak, 0.20 m after GNSS returns. (A, C)
- The stack is reused unchanged by a study, per case and per draw. (A)

**Connectors:** "in particular" (the corridor's loop), "as a result" (the navigation numbers),
"similarly" (reuse in a study).

**Closing sentence direction:** A pause therefore holds flow, chemistry and navigation at one
instant.

**Transition to Section 4:** That is the state worth varying.

### Section 4 — The campaign level: fork the pause, judge the branches

**Topic sentence.** At the campaign level, a study forks the paused state into branches, continues
each one, reduces them to rows, and gates the rows into a verdict it returns as data.

**Supporting points:**

- Code block: `CfdFlow::study(..).cases(..).fork(&pause).branch(..).continue_for(..)
  .reduce_all(..).refine(&pause, ..).branch(..).continue_for(..).reduce_all(..).gates(..)
  .verdict()?` (A)
- Define *branch*, *table* and *gate* on first use. (W)
- `verdict()` returns data; the caller maps it to an exit code. The gate sequence is a value. (A)
- `refine` runs a second round from the same pause. (A)
- Instance: seventeen branches, six coarse and eleven fine; 11.0° committed at 2.87 m. State that
  the ballistic 30 m is set by construction. Output excerpt: fine-sweep rows 10.0° to 12.0°. (C)

**Connectors:** "first" / "then" (fork, continue, reduce, gate), "in addition" (refine), "to
clarify" (the 30 m).

**Closing sentence direction:** The question, the alternatives and the judgement fit in one
expression.

**Transition to Section 5:** A bank angle is one kind of variation; the grammar has a second.

### Section 5 — Two kinds of what-if

**Topic sentence.** CfdFlow expresses two kinds of what-if: forking a paused state, and alternating
a baseline into whole worlds, and a study can feed the table of one into the other.

**Supporting points:**

- State fork: `fork(&pause).branch(..)`. Varies what happens after a moment; shares everything
  before it. (A)
- World alternation: `.baseline(..).alternate(..).ensemble(draws).couple(..).march_for(..)
  .reduce_ensemble(..)`. Varies the world a whole run lives in. Code block from A. (A)
- A fork implements the alternation traits of `deep_causality_core` (`alternate_context`,
  `alternate_state`, `alternate_value`), the same vocabulary the causal monad uses for
  counterfactuals. (K) Check before dictating whether the `alternate` world path goes through the
  same traits; only the fork's implementation was confirmed.
- Instance of alternation: six atmospheres × eight draws; polar winter drift 58.20 m against
  42.13 m, 3.1 sigma. (D)
- Instance of a state fork on a hard state: retropropulsion forks the marched plume-coupled burn
  into five throttle branches. Qualify immediately: the drag change comes from the Jarvinen-Adams
  correlation, AMBER on fidelity. (E)
- The two compose: retropropulsion reads the weather table in flight and sizes its ignition margin
  from it (66.99 m); informed guidance spent 10.54 kg more propellant than standard-day guidance.
  (E)

**Connectors:** "on the one hand … on the other hand" (the two forms), "because" (shared
vocabulary), "for instance" (the instances), "even so" (the AMBER qualification), "building on"
(the composition).

**Closing sentence direction:** A what-if is either a different future from one moment or a
different world from the start, and a study can use the output of one as the input of the other.

**Transition to Section 6:** A family of branches is only useful if it is affordable and
reproducible.

### Section 6 — What a fork costs, and what it guarantees

**Topic sentence.** A fork shares the paused state instead of copying it, runs its branches
concurrently with bit-identical results, and logs every alternation.

**Supporting points:**

- Cost model: branches share the paused state; each clones once at its first field write, so
  per-branch field cost scales with the cell count. (A) Keep "copy-on-write" in parentheses after
  the plain statement. (W)
- Measured: retropropulsion's fan-out marched at 1.16× the trunk's per-step cost; 5 of 5 branches
  entered by reference. (E)
- Arithmetic, labelled as such: in the corridor, seventeen branches skip re-marching the 106-step
  prefix. (C; derived)
- Determinism: concurrent fan-outs match the sequential run bit for bit. (A, B)
- Audit: each branch stamps `!!ContextAlternation!!` naming its baseline. (A, C)
- Limit: `save_log(path)` writes files only on the `march_for` path; the fork chain keeps its log in
  the returned report. (A)

**Connectors:** "in other words" (cost model), "for example" (measured), "in addition"
(determinism, audit), "however" (the `save_log` limit).

**Closing sentence direction:** The cost of a what-if is the part that differs.

**Transition to Section 7:** None of this depends on which solver the branches march.

### Section 7 — One grammar, any solver, any precision

**Topic sentence.** Because configuration is separate from execution, the same study grammar drives
a full march, a 1-D duct solver or a closed form, at whatever precision one alias names.

**Supporting points:**

- `flow_config` describes; `flow` runs. A what-if is the same flow with a different description.
  `CfdConfigBuilder` is the single entry, validated at `build()`. (A)
- Three solver families behind one language, one sentence each. (A)
- Instances, as a three-row table: placard (closed forms, `matrix`/`prepare`/`sweep`/`record`/
  `gates`, max q 23.7 kPa inside 60 kPa); nozzle operating map (duct march, every row gated against
  closed forms); vortex-shedding margin (DEC solver, minimum margin 0.236 against 0.15). (F)
- Precision as one alias; instance: the turbulence forecast horizon moves from 21.5 (`f32`) to 44.5
  (`f64`), and the law puts `Float106` near 81. (A, G)

**Connectors:** "because" (separation enables reuse), "for instance" (the table), "in particular"
(precision).

**Closing sentence direction:** The study shape stays the same whether the per-case body is a
closed form or a full march.

**Transition to Section 8:** A language over solvers is only as trustworthy as those solvers.

### Section 8 — What the language rests on, and where it stops

**Topic sentence.** The solvers under CfdFlow publish their matches and their misses, and the
language states its own limits where a user will meet them.

**Supporting points:**

- Gate labels: `[reference]` versus `[tripwire]`. (V)
- Matches and misses as one table: Ghia vortex to 1e-4; cylinder St +4.3 %, C_d −2.0 %, friction
  share 13 % against 25 %; Sod L1 under 0.03; Taylor-Green Re 1600 −80 % under-resolved; RAM-C II
  −0.16 decades inside ±0.70, blackout exit 45.7 km against 25–30 km; QTT cylinder red and offline.
  (V, C)
- Language limits: the regime classification is a diagnostic; `save_log` covers only `march_for`;
  the retropropulsion drag comes from a correlation. (A, E)
- CI runs ten harnesses per pull request, three monthly. (A, V)

**Connectors:** "in contrast" (matches to misses), "in addition" (language limits), "even so" (what
the misses leave standing).

**Closing sentence direction:** A red gate tells the reader exactly where to stop trusting the
result.

**Transition to Section 9:** Back to the question the post began with.

### Section 9 — Close

**Topic sentence.** A design question became one expression: march until the physics says so, fork
from that moment, judge the branches, return the table.

**Points:** the three commands to run the plasma-blackout family in order; the crate on crates.io;
the CFD project website. End on the language, then the run.

---

## Part 4: Verb check (step 4 of the guide)

**Verb chain that tells the whole story:** share (a prefix) → march → pause → couple → fork →
alternate → reduce → gate → return (data) → reuse (the grammar) → state (the limits).

**Weak verbs to replace:** is, has, can be, involves, provides, enables, allows, leverages,
supports.

**Verbs that must not repeat as a section's main verb:** *runs*, *shows*, *demonstrates*.

**Tense:** present for what the language does; past for what a run did.

---

## Style gates (from W and the writing guides)

- Each section leads with its finding; show a result before the code that produced it only within
  an instance paragraph.
- One word per concept: *what-if*, *pause*, *branch*, *table*, *gate*, *event log*, *condition*;
  *world* only where the code names it. Define *counterfactual* once, then use it rarely.
- Keep off the first paragraph: copy-on-write, O(1), tensor train, bond dimension, cochain, Leray
  projection, quasi-steady, causaloid.
- State, do not argue: no "not A, but B", no uniqueness claims.
- Say "the plasma-blackout example", never "flagship"; never "toy".
- Em dashes at most one per 250 words; no paragraph opens with *Additionally*, *Furthermore* or
  *Moreover*.
- Every wall-clock number names the M3 Max.
- Copy numbers from `output.txt`, never from example READMEs or `constants.rs` comments.
- `FloatType` alias in every snippet.

---

## Appendix: Critical analysis of the previous outline

Applied with the elements of thought and the intellectual standards of Paul and Elder,
*The Miniature Guide to Critical Thinking*.

**Purpose and question at issue (relevance, significance).** The purpose was to present CfdFlow,
but the previous outline's question at issue was the example's: where does the vehicle lose GPS,
and which bank angle wins? Five of eight topic sentences named the vehicle. A reader skimming
topic sentences would have read a reentry story. The question at issue is now the language's, and
no topic sentence names a vehicle.

**Point of view (fairness, breadth).** The previous structure took the CFD website's point of view,
whose job is to let a cold reader decide in seconds whether the result matters. A post named after
the language has a different job: explain the language. Results now serve as instances.

**Concepts (clarity, depth).** The language's constructs (the pause as a value, the coupling stack
as loop body, the two kinds of what-if, `verdict()` returning data, configuration separate from
execution, the shared alternation vocabulary with the causal monad) appeared as bullets under
example headings. Each now carries a section. The link to `deep_causality_core`'s alternation
methods was missing entirely.

**Information and relevance.** The previous evidence section was about solver accuracy (Ghia, Sod,
cylinder). Those numbers bear on the solvers, not on the language's claims. The evidence relevant to
the language (fork step cost 1.16×, branches entered by reference, bit-identical concurrency,
alternation markers, one grammar over closed forms and solvers) was scattered. It now has its own
sections (6 and 7), and solver validation is scoped as what the language rests on (Section 8).

**Inference (logic, accuracy).** The previous main point claimed a study replaces "a stack of
reruns". No artifact measures a rerun baseline. The outline now states what is measured (1.16×
per-step cost of the fan-out) and labels the prefix count as arithmetic.

**Assumptions.** The previous outline assumed that a vivid example makes the strongest opening. For
this audience a vivid example does make a strong hook, so it stays, as one paragraph in Section 1.

**Implications.** The previous outline omitted the language's own limits (`save_log` covers only
`march_for`). They are now in Section 6 and Section 8.
