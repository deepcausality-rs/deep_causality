
## Part C. Paragraph plan (steps 1 to 4)

Format per paragraph: **Main point** (stub for your topic sentence) · **Facts, in order** · **Connect**
(connector types from the guide's list) · **Close** (how the last sentence returns to the main point)
· **Transition** · **Verbs** (candidates to check in step 4).

Section targets: about 32 paragraphs plus the abstract, roughly 9,000–11,000 words with tables and
listings.

### Abstract (one paragraph, five moves)

1. Context: decisions inside coupled flow problems (reentry, powered descent) depend on a state the
   run reaches. (A, K)
2. Gap: a what-if costs a rerun or a restart saved at a time chosen in advance. (K)
3. Method: march to a condition, pause, fork by reference, branch, reduce, gate; one Rust library.
   (A, E)
4. Evidence: 17 branches from a flow-resolved blackout onset; 48-descent weather table; mid-burn fork
   of five throttles; table read in flight moves burn light 18.61 m at 10.54 kg; fan-out step cost
   1.16×. (F-run, G-run, H-run)
5. Scope: drag from a cited correlation; one flight anchor at order of magnitude; 32 × 32 layer.
   (H, B, I)

Verbs: pause, fork, branch, reduce, gate, move, spend.

### 1. Introduction (5 paragraphs)

**P1.1 The cost of a what-if.**
- Main point: asking what-if from a state a coupled run has reached costs a rerun, or a restart file
  saved at a time someone chose in advance.
- Facts: rerun repeats the shared past (K); restart costs the same steps as a fork but needs the
  time chosen before the run (K); the corridor's numbers as the example (A.7).
- Connect: contrast ("usually … here"); exemplifying ("for example").
- Close: the shared past is paid once per alternative.
- Transition: to problems where the decision depends on an instant the flow selects.
- Verbs: repeat, pay, choose, save.

**P1.2 Problems where flow and decision interlock.**
- Main point: in reentry blackout and supersonic retropropulsion the flow sets when a decision
  matters and what it does.
- Facts: shock layer ionizes, sheath cuts GNSS (plasma_blackout README via F); RAM-C II measured it
  (Grantham 1970); retro plume displaces the bow shock and collapses drag (Jarvinen & Adams 1970, H);
  "four things must hold at once" (physics, degraded navigation, cheap counterfactuals, audit).
- Connect: causal ("because", "therefore"); addition.
- Close: the decision point is an event in the flow, not a time in a schedule.
- Transition: so the operation needed is pause-on-condition plus branch.
- Verbs: ionize, cut, displace, collapse, decide.

**P1.3 What this paper presents.**
- Main point: a two-level language that marches to a condition, pauses, forks, and compares under
  gates, in one Rust process.
- Facts: trajectory level and campaign level (A); fork by reference, field copy-on-write (E); gates
  with evidence classes (B); listing of the corridor code (A or F `main.rs`, condensed).
- Connect: chronological (first … then); exemplifying.
- Close: one program carries a decision study from the shared past to a verdict.
- Transition: the contributions.
- Verbs: march, pause, fork, compare, return.

**P1.4 Contributions.** (a list; one stub each)
- The pause-fork-branch-reduce-gate grammar with compile-time phase order (A.9).
- The fork mechanism and its cost model, measured (A.6, A.7).
- An evidence discipline that labels every gate and runs it in CI (A.10).
- A three-example reentry application that composes a sweep with a state fork (A.14).
- Close: each contribution maps onto a section.

**P1.5 Scope and plan of the paper.**
- Main point: the paper reports what the application shows and where its physics authority sits.
- Facts: drag from the correlation (H); the corridor's decision reads no marched-field quantity
  (A.4); one flight anchor (B); 32 × 32 (I); one machine (D).
- Connect: qualification ("in particular"); chronological for the section roadmap.
- Close: the reader knows which results carry the method and which carry the physics.
- Transition: to related work.
- Verbs: report, anchor, limit, follow.

### 2. Related work

Prepared separately in `cfd_relatedwork.md`. Transition into Section 3: the closest prior work shares
state between simulation clones; this paper does that for a continuous coupled field with a pause the
field selects.

### 3. Method (7 paragraphs and two listings)

**P3.1 One shared field, one stage stack.**
- Main point: a coupling is a statically composed list of stages that step one shared field.
- Facts: cons-tuple, no `dyn` (A); corridor stack order (I); named fields; `Err` short-circuits the
  step (A); listing of the `Coupling::between_steps()` chain (A).
- Connect: chronological (stage order); causal (an error stops the step).
- Close: the disciplines never run in separate processes, so nothing is exchanged.
- Transition: what each step learns about its regime.
- Verbs: compose, step, read, write, short-circuit.
- Concession to place here or in P7.6: fields are named by strings and missing producers default
  silently (A.3).

**P3.2 Regime and events from the evolved state.**
- Main point: each step classifies its regime from the evolved state, so a march can stop when a
  regime changes.
- Facts: Knudsen bands, link regime (A); the four log lines from F-run; classification is a
  diagnostic, continuum closures only (A, K).
- Connect: exemplifying ("for instance, the corridor log"); qualification.
- Close: the run finds its events.
- Transition: stopping on such an event is the pause.
- Verbs: classify, band, log, stop.

**P3.3 The pause.**
- Main point: a march returns a resumable pause when its condition on the evolved field becomes true.
- Facts: `until(predicate)` (A); the pause holds state and field behind `Arc` (E); `MarchState` export
  carries the field and step only and re-seeds the layer, so a resumed leg differs from a continued
  pause (E).
- Connect: contrast ("a resumed leg … a fork"); causal.
- Close: two ways to continue, and only the fork keeps the marched layer.
- Transition: the fork.
- Verbs: return, hold, export, re-seed, continue.

**P3.4 The fork and its cost.**
- Main point: a fork shares the paused state by reference and clones the coupled field once, at the
  branch's first write.
- Facts: two `Arc::clone`s (E); marched tensor never cloned (E); field cost O(cells) (A, E); scoped
  threads; sequential-identical (E, M); cost model N(P + B) against P + N·B (A.7).
- Connect: chronological (fork, first write, continue); reformulating ("in other words, …").
- Close: the expensive past is computed once.
- Transition: what makes each branch different.
- Verbs: share, clone, write, run, match.

**P3.5 Context alternation.**
- Main point: a branch differs from its siblings by one alternated context, recorded in its log.
- Facts: `alternate_context` keeps value, state, error, log (L); "same causal law against a different
  world" (L); errored chains are not alternated (L); the marker line (F-run); the belief
  counterfactual runs as two marches with no marker (K).
- Connect: reformulating; qualification.
- Close: here the paper defines its use of "counterfactual" (D2).
- Transition: the grammar that drives many branches.
- Verbs: replace, preserve, record, name.

**P3.6 The campaign grammar.**
- Main point: a study composes cases, an origin, branches, reduction and gates into one monadic
  chain whose order the compiler checks.
- Facts: fork form and sweep form (A.9); `StudyEffect` functor/applicative/monad (E); five
  compile-fail doctests (O); `verdict()` returns data (A); listing of both forms (A).
- Connect: contrast (fork origin against baseline origin: "only the origin changes", K); addition.
- Close: a sweep and a fork share every verb except their origin.
- Transition: how a study judges itself.
- Verbs: compose, reduce, refine, gate, return.

**P3.7 Gates and evidence classes.**
- Main point: every gate states whether its bound is a reference or a tripwire, and CI runs the
  suite.
- Facts: the two labels and their meanings (B); failing baselines committed (B); CI lists (P); the
  audit and its counts (J).
- Connect: causal ("because a PASS can mean either, the label …"); qualification.
- Close: a reader can tell evidence about physics from evidence of non-regression.
- Transition: to the solvers those gates check.
- Verbs: declare, pin, commit, run, distinguish.

### 4. Solvers and verification (4 paragraphs and one table)

**P4.1 Three solver families, one scalar.**
- Main point: a DEC solver, QTT marchers and closed-form closures sit behind one language and one
  scalar type.
- Facts: A ("Multiple Solver Paradigms"); `SolenoidalField` type-state (A); QTT storage χ²·L (A);
  stagnation line with no grid (A).
- Connect: addition; exemplifying.
- Close: each problem uses the family that fits it.
- Transition: what the reference cases show.
- Verbs: march, project, store, fit.

**P4.2 Reference-class verification.** (Table 1: the reference rows of A.11)
- Main point: on closed-form references the solvers reproduce the expected answer or order.
- Facts: Sod, MMS, graded MMS, QTT TG order, Ghia primary vortex (A.11).
- Connect: addition; qualification ("in particular").
- Close: these are code-verification results at small grids.
- Transition: where the harnesses fall short.
- Verbs: match, converge, reproduce.

**P4.3 Where verification falls short.**
- Main point: several harnesses report a stated shortfall or fail by design.
- Facts: TGV endpoint dissipation 0.0025 against the DNS peak 0.0124 (the 16³ run ends before its peak); cylinder friction split; QTT cylinder not converging (committed failing); QTT TG at
  level 7; park2t +3.0 decades as internal-invariant scope (A.11).
- Connect: contrast; causal (each shortfall's cause).
- Close: the failures stay visible because the baselines stay committed.
- Transition: the studies that set the solver design.
- Verbs: under-resolve, cancel, fail, saturate.

**P4.4 Design studies.**
- Main point: measured studies, not assumptions, chose the 2-D fitted carrier.
- Facts: alignment drives rank (C); 3-D χ ~ side^0.53 (C); carrier timing from C-run; SRP momentum jet
  refuted the plume as drag authority (C, K).
- Connect: chronological (first … then); causal.
- Close: the application flies the configuration the studies left standing.
- Transition: the application.
- Verbs: measure, refute, select, budget.

### 5. Application: the plasma-blackout family (4 paragraphs and one figure)

**P5.1 Three examples, one descent.**
- Main point: three examples fly one reentry from entry at Mach 28 to touchdown as a chain: the
  corridor starts it, and each later example consumes what the earlier ones produced.
- Facts: corridor → weather table → retropropulsion (plasma_blackout README via F); run order.
- Connect: chronological.
- Close: the third reads the second's table in flight.
- Transition: the vehicle and flight model.
- Verbs: fly, consume, produce, close.

**P5.2 Vehicle, trajectory and time.**
- Main point: a point-mass 3-DOF vehicle in compressed time.
- Facts: 90 km start, ~7.69 km/s; L/D 0.3; β ≈ 170 kg/m²; 0.1 s per coupled step; no winds or aero
  dispersions (I, H).
- Connect: addition; qualification.
- Close: every simplification is labelled in the constants files.
- Transition: the flow layer.
- Verbs: start, steer, compress, label.

**P5.3 Flow layer, chemistry, link.**
- Main point: a 2-D compressible tensor-train layer feeds an uncalibrated ionization network whose
  electron density decides the GNSS link.
- Facts: 32 × 32, cap 16, γ_eff 1.1, RH inflow strip (I); quasi-steady (I); network RP-1232, no Saha
  target (I); ω_p against GPS L1 9.899e9 rad/s (F-run).
- Connect: causal chain (density → frequency → link).
- Close: the link regime is the event the corridor pauses on.
- Transition: navigation and control.
- Verbs: march, impose, ionize, cut off.

**P5.4 Navigation, control, and what reads the flow.**
- Main point: the marched layer reaches the decision loop through the electron density alone; drag,
  lift and heating read the atmosphere table.
- Facts: ESKF, IMU bias, GNSS variance, envelope (I); A.4 in full.
- Connect: contrast (n_e from the field; forces from the table); qualification.
- Close: the reader knows which results can depend on the flow.
- Transition: results.
- Verbs: dead-reckon, fold, clamp, read.

### 6. Results (7 paragraphs, three tables, two figures)

**P6.1 Fork economics.** (Q1)
- Main point: a branch costs about one trunk step per step and copies no tensor at fork time.
- Facts: gates 4d, 4g; 0.66× coasting (C, K); A.7 counts; 29.5 MB peak RSS (H); wall clocks with
  machine.
- Connect: addition; reformulating ("that is").
- Close: the shared past is paid once.
- Transition: the corridor descent.
- Verbs: enter, cost, share, grow.

**P6.2 The corridor descent and its blackout window.** (Q3, Q4)
- Main point: the run finds blackout onset at 76.8 km and exit at 45.7 km from the evolved electron
  density.
- Facts: F-run leg lines; n_e at the 71.93 km anchor crossing (−0.16 dec, matched speed); descent
  peak at 44.4 km inside the blackout; exit against the RAM-C 25–30 km window and the
  ballistic explanation; navigation error and variance growth and collapse.
- Connect: chronological (onset, peak, exit, reacquisition); contrast (exit altitude vs flight).
- Close: the window is an interval the run discovers.
- Transition: the branches forked at its start.
- Verbs: find, pass, recover, drift, collapse.

**P6.3 The bank-angle branches.** (Q1, Q3)
- Main point: seventeen branches from one onset resolve the miss landscape to half a degree; their
  flow observables agree to three digits.
- Facts: coarse and fine misses (F-run); 30 m by construction (F `constants.rs`); flow observables'
  ranges (A.14); website caveat (K).
- Connect: chronological (coarse, fine); contrast (miss varies, flow does not).
- Close: the corridor demonstrates the mechanism; the decision rests on the trajectory model.
- Transition: a study where the alternatives differ in the world, not the command.
- Verbs: bracket, refine, commit, clamp, agree.

**P6.4 The weather table.** (Q2, Q3)
- Main point: six atmospheres with eight draws each move the blackout window by seconds and the drift
  in the dark by 16 m, mostly through the typed IMU model.
- Facts: G-run table; 3.1σ; G README attribution; deterministic draws; no error bar on flow columns.
- Connect: causal (atmosphere → window → drift); qualification ("most of it").
- Close: the table separates the flow's share from the instrument's.
- Transition: the table becomes an input to a landing.
- Verbs: disperse, widen, integrate, separate.

**P6.5 The mid-burn fork.** (Q3)
- Main point: forking the burning vehicle into five throttles shows a deceleration minimum at mid
  throttle, from the cited drag correlation.
- Facts: H-run table with the derived departure column; gate 4b; coast-branch foil caveat; gate 4a
  scope sentence (H); the two SRP validity bands (H).
- Connect: contrast (coast vs mid); causal; qualification.
- Close: the fork shows what the drag law does to each trajectory; the plume does not supply the law.
- Transition: the decision the table and the fork make together.
- Verbs: fork, shed, depart, flip, bottom out.

**P6.6 Composition: the table read in flight.** (Q3)
- Main point: guidance sized with the measured day's table row lights the landing burn 18.61 m
  higher and spends 10.54 kg more than standard-day guidance.
- Facts: Act 0 numbers; gate 5; margin does not bind at commit; day-of-entry targeting buys nothing
  (H).
- Connect: causal; contrast (informed vs uninformed); qualification.
- Close: the reserve is measured, not configured.
- Transition: the landing itself and run totals.
- Verbs: interpolate, size, light, spend.

**P6.7 Landing and run totals.**
- Main point: the descent lands at 1.80 m/s after 5425 coupled steps in 335.3 s.
- Facts: H-run terminal act and gates 6–9; 1 rebuild; 8 regime transitions.
- Connect: chronological.
- Close: one program carries the vehicle from blackout exit to the ground.
- Transition: discussion.
- Verbs: cut off, re-seed, touch down.

### 7. Discussion (6 paragraphs)

**P7.1 What the evidence supports.**
- Main point: the measurements carry the method claims: cost, reproducibility, provenance and
  composition.
- Facts: C1, C2, C4, C5, C6 (Part B.6).
- Connect: addition; summary.
- Close: these hold at the tested scale.
- Transition: what the flow field contributes.
- Verbs: carry, hold, trace.

**P7.2 What the marched field contributes.**
- Main point: the field reaches decisions through the blackout window; forces and heating come from
  the table and correlations.
- Facts: A.4; weather attribution (G); retro drag authority (H); the D4 ablation result, or its
  design if it has not run.
- Connect: contrast; causal.
- Close: the ablation answers the question this paper cannot yet answer by inspection.
- Transition: alternatives to forking.
- Verbs: reach, gate, ablate, isolate.

**P7.3 Fork, restart, cloning, adjoint.**
- Main point: each alternative answers a different question at a different cost.
- Facts: restart equals fork in steps (K); `MarchState` drops the layer (E); simulation cloning (R1);
  adjoints give gradients for smooth objectives in one extra solve (R1); the corridor's 40° branch is
  clamped by the envelope (F-run), a non-smooth response; gates are pass/fail.
- Connect: on the one hand … on the other hand; contrast.
- Close: forking suits discrete, clamped or event-triggered alternatives.
- Transition: what to call a branch.
- Verbs: differentiate, clone, restart, clamp.

**P7.4 Terminology.**
- Main point: your D2 decision, stated and placed on Pearl's hierarchy.
- Facts: L; K definition; R2 (Pearl, Lewis, Woodward, Gerstenberg).
- Connect: reformulating ("in other words"); qualification.
- Close: one definition, used throughout.
- Transition: how the gates map onto V&V.
- Verbs: define, intervene, infer, share.

**P7.5 Evidence classes and V&V categories.** (Table: each gate → code verification / solution
verification / validation / regression)
- Main point: the reference/tripwire label maps onto the V&V categories of Oberkampf and Roy and
  makes the mapping executable.
- Facts: A.10, A.11; R5 (Oberkampf & Trucano 2002; Oberkampf & Roy 2010; Roache 1998).
- Connect: exemplifying; addition.
- Close: an executable evidence chain, with its validation share stated.
- Transition: threats to validity.
- Verbs: map, label, execute.

**P7.6 Threats to validity.**
- Main point: list them as positive statements of scope.
- Facts: S1–S9 (B.5); one machine; automated audit; string-named fields; A.15 discrepancies.
- Connect: addition; qualification.
- Close: each threat names the experiment that would retire it.
- Transition: limitations and future work.
- Verbs: bound, scope, retire.

### 8. Limitations and future work (3 paragraphs)

**P8.1 Physics reach.**
- Main point: turbulence, rarefied closures, plume-resolved SRP and off-axis flight are staged or
  open.
- Facts: turbulence in scope, staged as LES on the DEC solver after the spectra observables (K
  roadmap, "Open"); continuum closures only (A); plume-resolved SRP open (K); off-axis excluded for
  traceability (K).
- Connect: addition; chronological (staged order).
- Verbs: stage, close, resolve.

**P8.2 Scale.**
- Main point: 3-D fitted marching and production grids are open.
- Facts: C-run 3049 s; χ ~ side^0.53; parallel crossover at 256² (D); GPU deferred to batched branches
  (K).
- Connect: causal.
- Verbs: grow, cap, batch.

**P8.3 Integration.**
- Main point: wrapping external solvers needs a shareable in-process state.
- Facts: K ("does not wrap"); restart-file path; integrator switch uncalled (A); Float106 corridor
  compile errors (K).
- Connect: qualification.
- Verbs: wrap, share, compile.

### 9. Conclusion (2 paragraphs)

**P9.1** Return to the main question (B.2) and answer it with C1, C2, C6 and their numbers.
**P9.2** Return to scope (B.10, last bullet) and the one experiment that would extend the claim (D4).
- Verbs: answer, extend.

### Appendix A. Reproducibility

- Commands, in this order (Cargo runs one `--example` per invocation):
  `cargo run --release -p avionics_examples --example plasma_blackout_corridor`;
  `cargo run --release -p avionics_examples --example plasma_blackout_weather`;
  `cargo run --release -p avionics_examples --example plasma_blackout_retropropulsion`;
  then each harness, e.g. `cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline`
  (A, B).
- Run order: weather before retropropulsion (it writes the table). (F README)
- Machine, toolchain (rustc version from D or a fresh `rustc -V`), commit hash of the measured tree.
- Committed artifacts: every `output.txt`, `baseline.txt`, and the CSVs the examples write.

---

## Part D. Revision checks on the dictated text

1. **Numbers.** Each number matches its Part A source, keeps that source's precision, and carries its
   label where the label matters (typed / computed / by construction). Timings carry the machine.
2. **Verb pass (guide step 4).** Read only the main verbs of each paragraph in order; they should tell
   the paragraph's story. Replace *is used to, allows, enables, provides, leverages, facilitates* with
   the action.
3. **State, don't argue** (project rule). Phrase scope as what the system does: "the drag authority
   is the Jarvinen–Adams correlation" in place of a sentence about what the plume lacks. Grep for
   `rather than`, `instead of`, `unlike`, `not only`, `cannot`, `lacks`, `missing`.
4. **No uniqueness claims.** No "first", "only", "novel" without a citation search behind it
   (`cfd_relatedwork.md`, R1 novelty check).
5. **Banned phrases and punctuation** (`AiStyleguide.md`, `ElementsOfStyle.md`): *delve into, shed
   light on, game-changer, unlock the potential, not only … but also*; *very, really*; at most one em
   dash per 250 words; semicolons where a period over-breaks; vary sentence length.
6. **Paramedic pass** (`ParamedicEditing.md`): circle prepositions, box "is" verbs, move the action
   into the verb and the doer into the subject.
7. **Names.** "The plasma-blackout example", never "flagship"; never "toy".
8. **Read it aloud.**
