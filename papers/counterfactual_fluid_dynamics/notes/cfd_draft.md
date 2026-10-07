# Preprint scaffold: Counterfactual Fluid Dynamics

This file is a dictation scaffold, not a draft of the paper. It follows
`docs/writing_guides/TurnRawMaterialintoWriting.pdf` (step 0 raw information with sources, then
prioritize, connect, transition, check verbs) and builds the argument with the elements of thought and
the intellectual standards of `docs/writing_guides/MiniatureGuideCriticalThinking.pdf`.

Everything a reader of the paper would see is left for you to say. Paragraph entries hold a main point
as a stub, the facts in order with their source letters, the connector types the guide lists, the
transition, and verbs to check. None of it is meant to be read aloud as written.

How to use it:

1. Settle the six decisions in Part 0. They change what several sections claim.
2. Skim Part B, the argument. It records which claims the evidence carries and which it does not.
3. Dictate section by section from Part C. Every fact carries a source letter from Part A.
4. Run the Part D checks on the dictated text.

Conventions used throughout:

- **Source letters** (A, B, F-run, ...) point to Part A.1. A `-run` suffix is a committed `output.txt`
  or `baseline.txt`. Where a README and an output disagree, the output wins (Part A.15 lists the cases).
- **Number labels.** `typed` is a constant someone wrote into `constants.rs`. `computed` comes out of
  the run. `sampled` is a deterministic noise draw. `by construction` is a value the setup forces.
  `measured` is a wall clock or memory figure tied to one machine.
- **Evidence classes** are the crate's own: `[reference]` is a bound from an analytic solution or
  published value, and `[tripwire]` is a bound pinned from the code's own earlier output (B).
- **Machine.** Every wall-clock figure was measured on an Apple M3 Max, 16 cores (12 performance +
  4 efficiency), 128 GB RAM, release build (D, H). Attach this to every timing in the paper.

---

## Part 0. Decisions before dictating

**D1. How broad is the thesis?** (Recommended: the narrow option.)

| Option | Claim | What the evidence does to it |
|---|---|---|
| Narrow (method + evidence) | A coupled flow simulation can pause at a state the flow selects, branch into decision alternatives inside one typed program, and reduce them to gated rows. The branching costs one shared march plus one continuation per branch and keeps provenance per branch. On one reentry-and-landing problem, composing a dispersion sweep with a mid-burn fork changes an in-flight decision by a measured amount. | Carried by C1–C6 in Part B. |
| Broad (physics changes decisions) | Forking the *flow state* changes engineering decisions that a trajectory model alone would get wrong. | Contradicted in the corridor (the bank decision reads no marched-field quantity, F, I) and unsupported in the retropropulsion example (drag comes from a cited correlation, H). A reviewer who reads `stages.rs` finds this in minutes. |

Why narrow: the corridor's flow observables agree to three digits across its 17 branches (K, H-run
gate 4a), and the site already calls the corridor "the weakest evidence for the coupling" (K). A paper
that claims the broad thesis loses credibility on its strongest material too.

**D2. What does "counterfactual" mean in this paper?** The branches are interventions on a fully known
simulated state. A causal-inference reviewer will ask whether that is a counterfactual or an
intervention. The literature check (`cfd_relatedwork.md`, cluster R2) supports keeping the word on a
precise footing:

- Pearl (2009), Def. 7.1.5, p. 204, defines the unit-level counterfactual Y_x(u) = y, "Y would be y
  (in situation u), had X been x". A branch evaluates the modified model at a fully known u, the
  paused state. That is Def. 7.1.5.
- Pearl (2009), Theorem 7.1.7, p. 206: abduction, action, prediction. The abduction step updates
  P(u) from evidence e. It is needed when u is unknown. In a simulation u is known exactly, so the step
  reduces to reading the paused state.
- Bareinboim et al. (2022): every Layer-3 quantity "can be directly determined from a fully specified
  SCM" (Def. 27.6). A deterministic simulator is a fully specified model.
- Balke & Pearl (1994): holding the background fixed when an observation precedes the action
  assumes "the environment remains relatively static" between the two. The pause (an observed
  condition) followed by a branch (an action) is that case. In the simulator the background state is
  carried exactly, which discharges the assumption.
- Jiménez (2020), "Monte Carlo science": perturbed-simulation experiments yield "counterfactual
  relations (if not this, then not that)". Precedent for the word in fluid mechanics.
- Gerstenberg et al. (2021): "counterfactual simulation" of physical events, in cognitive science.
- What a reader may still object to: the real vehicle's state is not known exactly, so for a flight
  decision abduction returns (estimating the state from sensors). The library's navigation filter is
  that estimate inside the simulation; the paper should say that the counterfactual is over the
  simulated world, and say what an onboard use would add.

Options:

- (a) Keep the word and define it as Pearl's unit-level counterfactual (Def. 7.1.5) at a fully
  known simulated state, with the library's context alternation as the operator: "re-evaluate the
  same causal law against a different world by swapping the carried context mid-chain" (L). Mention
  Lewis's closest-world reading as background only; Pearl (2009, p. 37) distinguishes his account
  from similarity among worlds. (Recommended.)
- (b) Keep "counterfactual" for the title and the crate name, and use "branch" or "what-if" in the
  body, with one paragraph on terminology (Section 7, P7.4).
- (c) Rename the operation (for example "state-fork intervention").

The website already defines it as "a what-if that starts from something that did happen" (K). In a
simulation the "something that happened" is a simulated state, and a causal-inference reviewer will
press on that. Option (a) or (b) survives review; leaving the term undefined does not.

**D3. Paper type and template.** The skeleton in `../main.tex` uses `elsarticle` in `preprint`
mode: arXiv accepts it as is, and it maps onto *Computer Physics Communications*, *Computers &
Fluids* and *Journal of Computational Physics* without restructuring. If the target is an AIAA
venue (SciTech, *Journal of Spacecraft and Rockets*), switch to the AIAA template before the first
full pass; the section order carries over.

**D4. Run the ablation before submission?** (Recommended: yes.) The paper cannot currently show that
the marched layer changes any decision. One negative control would settle it (Part B.6, claim C3, and
Section 7, P7.2):

- Feed the ionization network the exact Rankine–Hugoniot post-shock state in closed form (the path the
  `qtt_ramc_stagline` harness already runs with no grid) instead of the evolved layer, then rerun the
  corridor, the weather table and the retropropulsion descent.
- Compare onset, dwell, drift in the dark, ignition margin, and burn-light altitude.
- If nothing moves, the paper says the marched layer carries realism and no decision. If something
  moves, that difference is the first direct evidence that the field changes a decision.
- Calibrate the control (AGENTS.md: "a negative control that returns zero may mean the control is
  miscalibrated"): perturb the density scale in both paths and confirm both respond.

A second, cheaper experiment answers the restart question (P7.3). Write the paused state with
`save_resume_state`, branch from disk, and compare wall clock and bits with the in-memory fork. The
crate's own `MarchState` export drops the marched layer (E), so the two paths differ; measure by how
much.

**D5. Disclosure.** Two disclosures are yours to word: the pre-certification audit was run by 16
parallel automated auditors with an adversarial re-check stage (J); and whatever policy the target
venue sets for AI-assisted code and text.

**D6. Title.** Keyword pool, for you to assemble: pause, fork, branch, coupled flow simulation,
flow-resolved state, decision, reentry, plasma blackout, retropropulsion, provenance, executable
evidence, counterfactual.

---

## Part A. Raw information (step 0)

### A.0 Read scope

Read on 2026-10-06: `deep_causality_cfd/{README.md, verification/README.md, studies/README.md,
benches/PERFORMANCE.md, papers/README.md, CHANGELOG.md, Cargo.toml}`; the fork and pause code in
`src/types/flow/{carrier.rs, study/event_fork.rs, study_effect/mod.rs}`; the fork tests in
`tests/types/flow/compressible_march_run/fork_tests.rs`; every source file and output of the three
plasma-blackout examples and the shared module under `examples/avionics_examples/src/shared/`
(`constants.rs`, `world.rs`, the `FreestreamFeeds`, `SuttonGravesLoads` and `WeatherTelemetry`
stages); the outputs of the four smaller CFD examples; `.github/workflows/cfd_verification.yml`; the
audit `README.md` and the summary, method and recommendation sections of `AUDIT-REPORT.md`; the
reader-visible text of the website's home, why, fork-or-sweep, boundaries and roadmap pages; and
`deep_causality_core`'s `AlternatableContext` trait.

Not read: most of the 27,535 lines under `deep_causality_cfd/src/`, the other 139 test files, the 16
per-module audit reports, the website tutorial pages, the PDFs in `papers/`, and the archived OpenSpec
notes for the CFD changes. Claims in Part A about those rest on the documents that cite them.

### A.1 Sources

| Letter | File or run |
|---|---|
| A | `deep_causality_cfd/README.md` |
| B | `deep_causality_cfd/verification/README.md`; B-run: each harness's committed `baseline.txt` as quoted there |
| C | `deep_causality_cfd/studies/README.md`; C-run: `studies/compressible_carrier_timing/output.txt` |
| D | `deep_causality_cfd/benches/PERFORMANCE.md` |
| E | `deep_causality_cfd/src/types/flow/carrier.rs` (`CarrierPause::state`, `fork`, `continue_branches`), `study/event_fork.rs`, `study_effect/mod.rs` |
| F | `examples/avionics_examples/cfd/plasma_blackout/corridor/{main,model,constants}.rs`; F-run: `corridor/output.txt` |
| G | `examples/avionics_examples/cfd/plasma_blackout/weather/{constants.rs,README.md}`; G-run: `weather/output.txt` |
| H | `examples/avionics_examples/cfd/plasma_blackout/retropropulsion/README.md`; H-run: `retropropulsion/output.txt` |
| I | `examples/avionics_examples/src/shared/{constants,world,stages}.rs` |
| J | `openspec/audits/cfd_audit/{README,AUDIT-REPORT}.md` |
| K | `website/cfd/dist/{index,why,fork-or-sweep,boundaries,roadmap}/index.html` (reader-visible text) |
| L | `deep_causality_core/src/traits/alternatable_context/mod.rs` |
| M | `deep_causality_cfd/tests/types/flow/compressible_march_run/fork_tests.rs` |
| N | `examples/avionics_examples/cfd/{turbulence_flow,nozzle_operating_map,viv_resonance_margin,flight_envelope_placard}/output.txt` |
| O | `deep_causality_cfd/Cargo.toml` (0.3.3), `CHANGELOG.md` (0.3.3 on 2026-09-29), a count of `#[test]` in `deep_causality_cfd/tests/` (942), `grep compile_fail deep_causality_cfd/src` (5) |
| P | `.github/workflows/cfd_verification.yml` |

### A.2 What the crate is

- Rust crate `deep_causality_cfd`, version 0.3.3, on crates.io, MIT licensed. (A, O)
- It couples fluid dynamics, other physics, navigation and control in one typed process. (A)
- Three solver families share one language (`CfdFlow`) and one scalar type: a DEC incompressible
  Navier–Stokes solver, QTT compressible Euler marchers (1-D to 3-D, plus a body-fitted variant), and
  closed-form closures (exact Rankine–Hugoniot, Park two-temperature, a finite-rate ionization
  network). (A)
- 942 `#[test]` functions under `tests/`; 14 verification programs; 5 compile-fail doctests. (O, B)
- It runs its own solvers; it does not wrap Fluent, SU2 or OpenFOAM; it reads no mesh format, and
  geometry is declared in code. (K)
- MSRV Rust 1.98; single machine; distributed execution is a stated non-goal; GPU deferred. (K)

### A.3 The coupled process

- A coupling is a static cons-tuple of `PhysicsStage`s stepping one shared `CoupledField`; no `dyn`.
  (A, I)
- The corridor stack, in order: vibrational lag (Millikan–White bath on the evolved per-cell
  pressure), finite-rate ionization (on the evolved per-cell density), freestream feeds, regime
  classifier, bank-steered lift, Sutton–Graves loads, truth/GNSS, navigation with IMU, commanded bank,
  telemetry, bounded-correction safety gate. (I `world.rs`)
- Stages communicate through named fields on the evolved state; an `Err` from any stage
  short-circuits the step. (A)
- Field names are strings, and a missing producer degrades to a default: `utils::scalar0` returns 0
  when a field is absent, and `FreestreamFeeds` returns early when `freestream_n <= 0`. The
  powered-descent stack's own comment records that, without `FlightSensors`, "the descent-rate bound
  cannot fire and the dynamic C_T cap collapses to the static ceiling: two safety axes reporting as
  enforcing while unable to". (I `stages.rs`, `world.rs`)
- Each step classifies the regime from the evolved state: Knudsen band (continuum, slip,
  transitional, free-molecular), link (plasma frequency against the GNSS band), and for the powered
  descent Mach band, thrust state and touchdown. (A)
- The classification is a diagnostic. No slip, transitional or free-molecular closure exists; the
  crate implements continuum closures only. (A, K)
- `RegimeSwitch` and `aero_gravity_ratio` (the KS-conformal core versus Cowell integration on
  `ε = a_aero/a_grav`) are public API that the shipped navigation engine never calls. (A, K)

### A.4 Which quantities read the marched field (load-bearing for the argument)

- Drag and lift: `BankSteeredLift` with constant `L/D = 0.3` and `C_dA/m = 5.8e-3 m²/kg`, driven by
  the equivalent airspeed `EAS = V·√(ρ∞/ρ_ref)`. `ρ∞` comes from the atmosphere table at the truth
  altitude. (I `constants.rs`, `stages.rs` `FreestreamFeeds`)
- Heating: Sutton–Graves `q = k·√(ρ∞/R_n)·V³` from the table density and the flight speed. (I
  `stages.rs` `SuttonGravesLoads`)
- Electron density: the finite-rate network and the vibrational bath read the evolved per-cell
  density and pressure of the marched layer. (I `world.rs`)
- So in the corridor the marched layer reaches the decision loop through one path: evolved state →
  `n_e` → plasma frequency → GNSS denied or available → navigation aided or dead-reckoning. (I, F)
- The truth trajectory, and the miss distance that picks the bank angle, read no marched-field
  quantity. (F `model.rs` `score_one`, I)
- Retropropulsion drag: the Jarvinen–Adams A0 correlation evaluated at each branch's thrust
  coefficient, not a decrement from the field. (H)

### A.5 The pause

- `CfdFlow::march(..).couple(..).from_field(..).until(predicate)` marches until the predicate on the
  evolved field fires and returns a resumable pause. In the corridor the predicate is the classifier's
  GNSS-denied flag. (A, F)
- "No onset constant exists anywhere in the corridor." (plasma_blackout README via F)
- The pause holds the marched state and the coupled field each behind an `Arc`. (E)
- `pause.state()` exports a `MarchState`: the coupled field plus the step index. A leg resumed from
  it rebuilds its carrier and re-seeds the marched layer from the world's uniform seed: "a resumed leg
  is therefore not bit-identical to continuing this pause directly". (E)
- Fork and `continue_with` are the paths that continue the marched trajectory: they share the evolved
  `Arc<M::State>`. (E)
- The corridor's diagnostic legs go through `MarchState`, so each logs "marched fluid state re-seeded
  from the world seed" (F-run). The bank branches go through the fork. (F `main.rs`, E)

### A.6 The fork

- `fork()` is two `Arc::clone`s; it copies nothing. (E)
- The marched tensor state is never cloned: the continued loop reads the shared state and replaces
  the `Arc` with each new state. (E)
- The `CoupledField` is copy-on-write. Each branch clones it once, at its first write, which
  `continue_march` always performs, so the per-branch field cost is O(cells): per-cell scalar
  vectors, the navigation engine, and the provenance log. (A, E)
- Each branch writes a `!!ContextAlternation!!: world 'X' replaced with 'Y' at step N` entry into a
  fresh branch log. (E, F-run)
- `continue_branches` runs the branches on scoped threads under the `parallel` feature
  (`deep_causality_par::scoped_map`) and inline otherwise. (E)
- Test `continue_branches_matches_the_manual_fork_chain_in_world_order` compares the batch fan-out
  with a manual sequential fork chain; test `fork_economics_are_reproducible_across_runs` checks that
  the recorded fork sample is identical across runs. (M)
- The `fork → branch → continue_for` chain does not thread the disk audit sink, so it writes no log
  files; each branch keeps its log in the returned report. (A, E)
- Measured in the retropropulsion fork: 5/5 branches entered by reference, worst post-fork bond
  growth 0 (cap 8), fan-out step cost 1.17× the trunk's per-step cost (cap 3.00). All `[tripwire]`.
  (H-run gates 4d, 4g)
- From the plume study: each powered continuation costs about 1× an unforked trunk, with post-fork
  bond flat at 16; a coasting branch costs 0.66×. (C, K)
- Whole retropropulsion descent: 346.9 s wall clock, __RETRO_RSS__ MB peak resident set, 5425 coupled steps,
  measured on the built binary. (H)

### A.7 Step arithmetic, fork against rerun (K)

- Corridor: 106 steps to the pause, 100 per branch.
- Six branches: rerun 6 × 206 = 1,236 solver steps; fork 106 + 6 × 100 = 706.
- Seventeen branches: rerun 3,502; fork 1,806.
- Shared past ten times longer (1,060 steps), six branches: fork 1,660, rerun 6,960.
- "A restart file saved at the pause costs the same steps as a fork." (K)
- General form, for P shared steps, B steps per branch and N branches: rerun N·(P + B), fork P + N·B,
  ratio N(P + B)/(P + N·B), which tends to N as P grows. (derived from K; not in any source file)

### A.8 Context alternation (the counterfactual operator in the host library)

- `alternate_context` replaces the context channel of an in-flight carrier and preserves value,
  state, error and logs. (L)
- "Re-evaluate the same causal law against a different world by swapping the carried context
  mid-chain instead of building a separate pipeline." (L)
- If the chain has already errored, the error propagates and the alternation is not applied. (L)
- The log gains one `!!ContextAlternation!!` entry. (L)
- The retropropulsion "belief counterfactual" (informed against uninformed guidance) runs as two
  separate marches and carries no alternation marker. (K fork-or-sweep, "What a fork does not do")

### A.9 The campaign grammar

- Two levels. Trajectory: `CfdFlow::march` to a pause. Campaign: `CfdFlow::study` runs a family of
  cases to a `Verdict`. (A)
- Fork form: `.cases(..).fork(&pause).branch(world_fn).continue_for(n).reduce_all(score)
  [.refine(&pause, next_cases).branch(..).continue_for(..).reduce_all(..)] .record(path)
  .gates(seq).verdict()`. (A, F `main.rs`)
- Sweep form: `.cases(..).baseline(world).alternate(world_fn).ensemble(draws)
  .couple(|case, draw| ..).march_for(steps, seed).reduce_ensemble(row).gates(seq).verdict()`. (A, G)
- Every phase rides `StudyEffect`, a carrier of `Result<T, StudyError>` plus a warning log, with
  lawful `Functor`, `Applicative` and `Monad` instances; it mirrors the Causal Discovery Language's
  `CdlEffect`. (E `study_effect/mod.rs`)
- Phase order is checked at compile time: five compile-fail doctests reject a mis-ordered study.
  (K roadmap, O)
- `verdict()` returns data; the language never prints or exits; `main` maps the verdict to an exit
  code (0 pass, 1 gate regression, 2 setup failure). (A, F)
- A gating sequence is a named value (`GateSeq<Row>`) the study inserts whole. (A)
- Configuration is separate from execution: `flow_config` holds owned descriptions, `flow`
  materializes runs from them, "so a counterfactual is the same flow handed a different
  description". (A)

### A.10 The evidence discipline

- Every gate line declares `[reference]` or `[tripwire]`; unlabelled defaults to tripwire; "a
  tripwire is never presented as validation against a reference". (B)
- Every harness commits a `baseline.txt` of a complete run, stdout and stderr; a failing baseline is
  committed as failing (`qtt_cylinder_verification`, exit 1). (B)
- CI: 10 fast harnesses on every pull request, 3 slow harnesses monthly, 1 offline; a completeness
  check fails when a declared harness is in no list. (P, B)
- Audit, 2026-07-21: 16 parallel automated auditors, one per subsystem, plus an adversarial re-check
  of all 294 findings (190 confirmed, 100 partially, 4 refuted); 412 source files read; after review 4
  critical, 72 major, 179 minor, 35 info. (J)
- The audit's verdict at the time: "a strong research instrument with a weak assurance case". 72 of
  294 findings were tautology or circular-reasoning defects; several headline gates "are
  algebraically incapable of failing"; no CI job ran any verification program. (J)
- Remediation: four phases complete by 2026-07-26; all four blockers resolved; gate lines with an
  evidence class 0 → 38/38 after Phase 1. (J)
- A remediation finding: a documented fold check in `BlendedMap` was absent, and the documented
  argument that a fold was impossible was false; a sweep found 275 accepted configurations that fold.
  (J)
- One blocker (B-1): the Millikan–White reduced mass was 7.0 amu for a pair documented as N₂–N₂
  (correct 14.007). At the cited 71 km freestream the corrected Park-2T controller lands −1.97 decades
  from the RAM-C II station-1 anchor, and that offset is reported, not tuned away. (J, B)
- Standing rule adopted during remediation: no test may assert on source text. (J)
- "A passing gate means the measured structure is reproducible, not that a physics target was met."
  (K)

### A.11 Verification results (B, B-run; f64, Apple M3 Max)

| Harness | Result | Class | Shortfall stated by the source |
|---|---|---|---|
| `qtt_sod` | L1 ρ 0.0175, u 0.0274, p 0.0151 against 0.03; p* 0.3031 | reference | first-order Rusanov smears the contact; "the only quantitative physical-accuracy gate in the QTT compressible set" |
| `mms_taylor_green_verification` | residual 1.11e-16, amplitude error 6.66e-16 | analytic, reported | f32 ≈ 3e-8, f64 ≈ 1e-16, Float106 ≈ 8e-33 |
| `dec_graded_mms_verification` | observed order 1.98–2.00 (finest pair), grading 0.0–0.3, 8²–64² | analytic | coarse-pair order dips to ~1.7 at strong grading |
| `qtt_taylor_green_verification` | error 9.8e-4 → 2.4e-4 → 5.3e-5 (8² → 32²), order 2.02–2.18; convection 3.2e-3 | gated | fails at its own documented `max_level 7` (cancellation; order collapses to 0.02 at N = 128) |
| `dec_lid_cavity_re1000_verification` | primary vortex (0.5312, 0.5625) vs Ghia (0.5313, 0.5625); centerline RMSE 0.0617 at 65², t = 100 | reported (default); tripwire (`trend` mode, 17² → 33²) | the 65² run has no pass/fail bound; a quarter of Ghia's cells |
| `dec_cylinder_verification` | St 0.1710 (+4.3 % vs 0.164); C_d 1.342 inside 1.314–1.370 (Parvar et al. 2023, Table 1: seven Newtonian sources), −2.0 % of the band top | tripwire | friction 13 % of C_d vs ~25 %; C_d lands in band by cancellation; 8 cells/D |
| `dec_cylinder_wake_verification` | max divergence 3.33e-15; log 80 = 2 × 40 dropouts | internal | no shedding (25 % blockage); no reference claim |
| `dec_taylor_green_re1600_verification` | energy monotone (E*/E0 0.893) | internal invariant | peak dissipation 0.0025 vs DNS ≈ 0.0124, −80 % at 16³ |
| `qtt_ramc_stagline` | cited 71 km freestream (Parent et al., M 25.9, 217.9 K, 2.28 kPa): network peak n_e 1.643e19 (+0.22 dec of 9.93e18, the station-1 Ka-band crossing at 71.93 km); Park-2T controller 1.070e17 (−1.97 dec, reported); cited 61 km freestream: network 1.811e20 meets the anchor as a lower bound (+1.26 dec) | tripwire (71 km band); reference (61 km lower bound) | ±0.70-decade band is a chosen chemistry-spread allowance; uncalibrated network; one body station |
| `qtt_park2t_blackout` | six coupling gates pass; peak n_e 1.000e22 (+3.0 dec) | internal invariant | Saha surrogate saturates at α = 1 at γ = 1.4; not comparable to the stagline γ = 1.1 |
| `qtt_blunt_body_2d` | fitted χ 3, 4, 5; Cartesian χ 16, 32, 61 (2⁵ → 2⁷) | tripwire (rank, not accuracy) | marched rank grows to 64 over 6 steps; reported, never asserted |
| `qtt_reentry_3d` | fitted χ 2, 4, 4; Cartesian 10, 30, 59 (2³ → 2⁵) | tripwire (rank) | wake χ 41, out of scope |
| `qtt_cylinder_verification` | η ladder C_d 17.39, 24.02, 26.25, 23.76, 21.40: NOT CONVERGING | reference (fails) | Brinkman layer √(ην) = 0.144·dx unresolved at L = 5; offline, ~4–9 h at L = 8 |
| `dec_wall_heat_flux_verification` | in CI fast list; `|q − analytic| = 0` on a spacing ladder | reference | <1 s (measured 0.30 s) (B, P) |

### A.12 Design studies (C, C-run)

- Rank follows coordinate alignment, not sharpness: a captured misaligned shock reaches χ ≈ 151–394,
  larger than dense; shock-aligned or body-fitted χ ≈ 5, about 290× smaller. (C)
- In 3-D a captured curved shock grows as χ ~ side^0.53 (45 → 135 over 16³ → 128³); a body-fitted
  coordinate is "mandatory for 3-D tractability". (C)
- A static fitted coordinate does not stay low-rank under marching; re-pinning alone does not fix it;
  re-pinning plus an exact Rankine–Hugoniot interface holds bond 8. Scalar Burgers, not Euler. (C, K)
- Carrier timing (C-run, the committed output): 3-D fitted 16³ at bond cap 16 runs 15.245 s/step and
  projects 3049 s (50.8 min) for a 200-step corridor against a 600 s budget; GO is 2-D at 64², cap 32,
  0.248 s/step. The bond cap is a cost parameter "carried forward untested for accuracy". (C-run, C)
- SRP momentum jet: drag rises with thrust (annulus fraction 1.03 → 3.61 over C_T 0.25 → 8; 1.413 at
  C_T 1.00 against the Jarvinen–Adams 0.124); the stagnation interface stays at x = 0.469–0.531
  across a 32× thrust range; cause attributed to the dissipation floor `ν = ½·s_ref·Δx` and the
  domain, not the model class. (C, K)
- Trajectory axis: the KS generator reproduces a Kepler orbit to 2.3e-15·a; a Strang aero kick is
  second order (observed 2.000); a proper-time kernel reproduces the GPS clock split (+45.65 /
  −7.21 / +38.44 µs/day). (C)

### A.13 Performance and scale (D, K)

- DEC march, `parallel` feature: slower below 256² (0.31× at 16², 0.72× at 48²); break-even near
  256² (1.04×); 1.26× at 384²; 1.47× at 512². Serial projection and orchestration cap the speedup
  (Amdahl). (D)
- The what-if examples run on 32 × 32 cells; "nothing has run at production mesh scale". (K)

### A.14 The application: three plasma-blackout examples

**Corridor setup (I, F):**

- Truth vehicle starts at 90 km with velocity (−1255.1, 7588.6, 0) m/s, about 7.69 km/s (derived), sized
  so the descent crosses 71.93 km at the flight's 7.66 km/s (measured 7662 m/s).
- Point-mass 3-DOF; drag along velocity; lift `(L/D)·D` rotated by the clamped bank, one-step lag;
  6-DOF out of scope.
- Compressed time: one coupled step is 0.1 s of flight and one solver pseudo-time step toward the
  quasi-steady layer at that instant (`DT_FLIGHT` 0.1, `DT_SOLVER` 0.002).
- Carrier: 2-D compressible TT layer, 32 × 32 (L = 5), bond cap 16, γ_eff 1.1; the exact
  Rankine–Hugoniot state from the truth Mach (via the table) is imposed on the inflow strip, and the
  layer behind it evolves.
- Atmosphere: US Standard Atmosphere 1976 at 1 km spacing, 0–90 km, interpolated log-linearly in
  density; at 71 km it reproduces the cited RAM-C II freestream (Parent et al.).
- Chemistry: uncalibrated finite-rate network (RP-1232 Table II pairs, no Saha target), sheath
  renewed each step at the transit-age peak `t_res·ln 65`.
- Navigation: 17-state ESKF; IMU accelerometer bias (2.0e-2, −1.4e-2, 1.0e-2) m/s² (~2 mg, typed);
  GNSS fix variance 1 m² with deterministic receiver noise matching R.
- Envelope: heat flux 2e7 W/m², g-load 100, bank 0.5 rad (28.6°).
- Branch study: coarse banks 0, 5, 10, 15, 20, 40°; fine ±5 steps of 0.5° around the coarse winner;
  100 steps (10 s) per branch; aim point = ballistic terminal state offset 30 m cross-range
  (`AIM_CROSS_RANGE_M`, typed; sized so the optimum falls inside the fine bracket).
- Scoring rule: minimum trajectory-derived miss to the aim point; "a flight design would weight all
  four outcome components". (F `model.rs`)

**Corridor results (F-run; all 14 gates `[tripwire]`; 45.9 s measured):**

- Onset at step 106, 76.8 km, Mach 26.8, Kn 8.71e-3, n_e 3.274e16 m⁻³.
- Coarse round misses: 30.000 (0°, by construction), 16.463, 3.818, 11.516, 24.988, 48.426 m (40°,
  clamped by the envelope).
- Fine round: 11.0° at 2.865 m (committed); 10.0° at 3.818 m; spread 6.935 m over 11 branches.
- Across all branches peak heat 2.543e6–2.546e6 W/m², thermal load 1.779e7–1.780e7 J/m², dwell 10.00
  s, t² cross-check 1.3191 m, peak n_e 7.943e19–7.974e19 m⁻³.
- RAM-C II anchor (gate 2): n_e 6.837e18 m⁻³ where the descent crosses 71.93 km, −0.16 dec from the
  9.93e18 station-1 Ka-band datum (allowance ±0.70 dec), crossing speed 7662 m/s against the flight's
  7660; at 61 km n_e 1.303e20 m⁻³ at 7457 m/s, reported against the flight lower bound.
- Descent peak (gate 2c): n_e 2.830e20 m⁻³ at 44.4 km with the link denied; no flight datum applies
  below 56.39 km, the end of the RAM-C II primary data period.
- Peak-passage leg: 61.0 km, Mach 23.8, n_e 1.306e20 m⁻³.
- Exit: 45.7 km, Mach 9.2, after a 56.0 s dwell; RAM-C II flight exit window 25–30 km; offset
  attributed to the light ballistic bundle (β ≈ 170 kg/m²).
- Navigation error 0.3143 m at onset → 3.0698 m at the peak passage → 0.2043 m after reacquisition;
  variance 2.739e-1 → 2.778e2 → 3.226e-1 m²; 2.11 m when the branches are scored; 42.8 m on the last
  step before the link returns.
- Four regime entries in the log: slip (available, Kn 0.0771) → continuum (available, 0.00997) →
  continuum (denied, 0.00871) → continuum (available, 0.000148).
- 0 carrier rebuilds; 16 log entries.
- The website states: "This shows the sweep at work, not guidance accuracy: the example scores each
  branch on the same model that flew it." (K)

**Weather table (G, G-run; 8 gates `[tripwire]`; 179.2 s measured):**

- Six atmospheres as (dT, density scale): standard (0, 1.00), hot (+20, 0.90), cold (−25, 1.10),
  polar winter (−40, 1.20), thin (−5, 0.75), dense (+5, 1.30). Day-to-day shapes, not reanalysis.
  (typed)
- IMU thermal model `bias·(1 + 0.01·|dT|)` (typed); filter priors stay standard-day.
- 8 deterministic low-discrepancy receiver-noise draws per condition: 48 full descents of 850 coupled
  steps, flown concurrently. (sampled)
- Onset 9.3–12.1 s (spread 2.8 s); dwell 55.3–56.2 s (spread 0.9 s).
- Drift in the dark: standard 42.13 ± 3.05 m; polar winter 58.20 ± 4.09 m (1.38×; separation 16.07 m
  against combined σ 5.10 m, 3.1σ).
- The README: "The IMU departure accounts for nearly all of the measured deviation"; the dwell moves
  the drift by at most 3.3 %, under 2 m of the 16. (G)
- Flow and window columns carry no error bar: receiver noise does not touch the flow, chemistry or
  truth trajectory. (G)
- Worst-draw terminal navigation error under 1.0 m in all 48 descents.
- Whole-descent peak n_e 3.007e20–3.110e20 m⁻³ (see A.15).

**Retropropulsion (H, H-run; 16 gates `[tripwire]`; 346.9 s, __RETRO_RSS__ MB measured):**

- Vehicle 3400 kg wet, 2200 kg propellant, 70 kN central nozzle, Isp 282 s. (typed)
- Act 0: the measured day is dT = −32 K (typed); interpolating the weather table gives drift
  54.28 ± 4.24 m and an ignition margin of 66.99 m (k = 3); standard-day belief gives 51.30 m.
- Ignition corridor commits at step 2145 of the coast-and-burn leg: Mach 1.9988, q 2903 Pa, navigation
  σ 0.382 m; the leg pauses one step later at 30.84 km.
- The coast, commit and burn acts run in one march call because a leg boundary does not carry the
  marched tensor; splitting at ignition would discard the plume-coupled layer before the fork. (H)
- Mid-burn fork, five branches:

  | branch | cmd | flown | preserved drag | axial m/s² | prop kg | dv m/s | dv frozen | dv − frozen |
  |---|---|---|---|---|---|---|---|---|
  | coast | 0.00 | 0.0000 | — | 13.9101 | 1.83 | 183.202 | 3.282 | 179.920 |
  | low | 0.20 | 0.2000 | 0.4335 | 11.6754 | 62.07 | 137.776 | 52.988 | 84.788 |
  | mid | 0.40 | 0.4000 | 0.1800 | 11.6531 | 122.31 | 137.634 | 103.394 | 34.240 |
  | high | 0.60 | 0.6000 | 0.0171 | 13.3226 | 182.56 | 158.870 | 154.650 | 4.220 |
  | hard | 0.85 | 0.8500 | −0.0329 | 18.5463 | 257.86 | 215.696 | 220.012 | −4.316 |

  The last column is derived here. The 179.9 m/s headline of gate 4c is the coast branch, where the
  closure applies nothing and the frozen foil holds drag at the trunk's fork value. No branch was
  clamped by the envelope in this run. (H-run, K)
- Net deceleration is non-monotone in throttle, minimum at 0.40 (11.653 m/s² against 13.910
  coasting). (H-run gate 4b)
- Gate 4a's flow spread (0.0844 against 0.0150) measures throttle → trajectory → post-shock density,
  "not the imprint witness the design note describes". (H)
- The two SRP models meet at one point: Jarvinen–Adams covers Mach 0.4–2.0, Cordell–Braun Mach 2–4.
  (H, I)
- Terminal leg: subsonic re-seed under γ = 1.4; 1 carrier rebuild (s_ref 1.4 → 2.768); touchdown
  1.80 m/s against a commanded 2.0 m/s; 1227.0 kg propellant left; 8 regime transitions.
- Belief counterfactual: informed guidance lights the landing burn at 146.91 m, uninformed at
  128.29 m (18.61 m apart against an arithmetic margin difference of 15.69 m); contact 1.80 against
  1.76 m/s; informed spends 10.54 kg more propellant. (H-run gate 5)
- The margin does not bind at ignition commit: σ 0.38 m against margins of 67.0 and 51.3 m; both
  beliefs commit on the same step. (H)
- Day-of-entry targeting buys nothing here: blackout exit lands within 3.5 s on every tabulated day,
  against a 158 s gap to the commit. (H)
- Corridor inheritance: onset 10.00 s against the table's 10.01 s; dwell 55.60 against 55.46 s.
  (H-run gate 1)

### A.15 Discrepancies across sources (settle before citing)

| Item | Source 1 | Source 2 | Use |
|---|---|---|---|
| 3-D carrier timing | settled: C README now matches C-run | C-run: 15.245 s/step, 3049 s; GO 64² at 0.248 s/step | C-run |
| Altitude of the RAM-C anchor | settled: 71.93 km (236 000 ft), Grantham (1970) p. 18, in I, B and F | the stagline harness flies the cited 71 km freestream (Parent et al.); the corridor reads its descent at 71.93 km | 71.93 km |
| What the anchor measures | settled: station 1 (x/D = 0.15) Ka-band (35 GHz) critical-density crossing, N_pk = 0.63 · 1.287e-8 f² = 9.93e18 m⁻³ (Grantham p. 11, Table I) | below 71.93 km station 1 is overdense at Ka-band, so the same value is a lower bound down to 56.39 km | a point value at 71.93 km; a lower bound at 61 km (the stagline `[reference]` gate) |
| Corridor onset / exit | settled: comments no longer quote altitudes | F-run: 76.8 km, 45.7 km | F-run |
| Entry Mach | plasma_blackout README: "Mach 28" at 90 km | F-run: Mach 26.8 at onset (76.8 km), 23.8 at 61.0 km | quote per station |
| Weather peak n_e | G-run: 3.0e20–3.1e20 over the whole descent | F-run gate 2c: the descent peak, 2.830e20 at 44.4 km, lies inside the blackout and below the RAM-C II primary data period (ends 56.39 km) | reported; no flight datum applies at that altitude |
| Verification count | settled: A, B, P: fourteen programs | B summary table: 14 rows | 14 |
| CI cadence | J README (2026-07-21): 9 per PR, 4 nightly | P (current): 10 per PR, 3 monthly, 1 offline | P |
| 106-bit corridor | K why: identical gates at f64 and 106-bit, ~11× cost | K boundaries: 44 compile errors today; result is a record of an earlier build | report as historical or omit |
| Cylinder C_d reference band | settled: B and `dec_cylinder_verification/main.rs` cite Parvar et al. (2023) Table 1, seven Newtonian sources, C_D 1.314–1.370 | C_d 1.342, −2.0 % of the band top | 1.314–1.370 |
| Peddinti et al. 2024 title | settled: every repository citation uses the published title | "Quantum-inspired framework for computational fluid dynamics" | published title |

### A.16 Number provenance (memory rule: label every headline number)

| Number | Label | Why |
|---|---|---|
| 30.000 m zero-bank miss | by construction | `AIM_CROSS_RANGE_M` places the aim point 30 m off the ballistic terminal |
| 2.87 m best miss, 11.0° | computed | scored on the same 3-DOF model that flew it |
| 76.8 km onset, 56.0 s dwell | computed | flow-resolved event |
| 9.93e18 m⁻³ anchor at 71.93 km | external reference, derived | Grantham (1970): Ka-band critical density × 0.63 at station 1 |
| ±0.70-decade band | tripwire | chosen chemistry-spread allowance (rate sets spread 2×–5×) |
| 0.01/K IMU coefficient | typed | labelled tactical-grade assumption |
| 42.13 → 58.20 m drift | computed | dominated by the typed IMU coefficient (G) |
| −32 K measured day | typed | stands in for a day-of-flight measurement |
| 66.99 / 51.30 m margins | computed | table interpolation, k = 3 typed |
| 18.61 m, 10.54 kg | computed | the composition result |
| preserved drag 0.434 → −0.033 | computed from a cited correlation | Jarvinen–Adams at each branch's C_T |
| 1.17× fan-out cost, 0 bond growth | measured / computed | gates 4g, 4d |
| 45.9 s, 179.2 s, 346.9 s | measured | Apple M3 Max |

---

## Part B. The argument (elements of thought)

### B.1 Purpose

To present a method and a library for pausing a coupled flow simulation at a state the flow selects
and branching it into decision alternatives inside one typed program, and to report what one
reentry-and-landing application shows and does not show about that method.

### B.2 Question at issue

Main question: can a coupled flow simulation pause at a flow-resolved state and branch into decision
alternatives cheaply, reproducibly and auditably, and does doing so change a decision on a realistic
problem?

Sub-questions, each answered by one section of results:

- Q1 Cost: what does a branch cost against a rerun and against a restart file?
- Q2 Reproducibility and audit: are concurrent branches identical to sequential ones, and can a reader
  trace each branch's difference?
- Q3 Coupling: does the flow state reach the decision, and through which path?
- Q4 Physics reach: how far do the solvers and closures hold, against which references?

### B.3 Information

Part A. The facts that carry the most weight: A.4 (which quantities read the marched field), A.6
(fork mechanics and measured cost), A.7 (step arithmetic), A.10 (evidence discipline), A.11
(verification classes), A.14 (application numbers).

### B.4 Concepts the paper must define

Define each once, where the reader first needs it, and keep one word per concept (the website's
vocabulary table in `website/cfd/README.md` is a good model).

| Concept | Definition material | Source |
|---|---|---|
| coupled field | the one shared state every stage reads and writes each step | A, I |
| stage | one physics or control step in the static stack | A |
| pause | the resumable state a march returns when its condition becomes true | A, E |
| fork | a branch sharing the paused state by reference; the field cloned on first write | E |
| world | a run description with one declared difference | A, K |
| branch | one alternative continued from the pause in its own world | K |
| context alternation | swap the carried context, keep value, state, error and log; one marker entry | L |
| counterfactual | your D2 decision | L, K, R2 |
| gate, evidence class | pass/fail check; `[reference]` or `[tripwire]` | B |
| regime | Knudsen, link, Mach, thrust, touchdown bands classified from the evolved state | A |
| quasi-steady layer | one pseudo-time step per coupled step toward the layer at that instant | I |
| compressed time | one coupled step = 0.1 s of flight | I |
| bond dimension χ | size of the tensor-train cores; storage `χ²·L` for a 2^L grid | A |

### B.5 Assumptions (state them; mark which are checked)

| # | Assumption | Checked? |
|---|---|---|
| S1 | The quasi-steady layer re-converges within a few steps after a re-seed | asserted in E; not measured in any source read |
| S2 | One solver pseudo-step per 0.1 s of flight is enough for the layer to track the trajectory | not measured |
| S3 | Point-mass 3-DOF with constant L/D represents the vehicle well enough for the bank decision | scope choice (I) |
| S4 | A single flight anchor at order of magnitude is enough to call the blackout window flow-resolved | partly: the ±0.70-decade band is a chosen allowance (B); one body station; the 61 km lower bound is the only `[reference]` gate on the chemistry |
| S5 | The IMU thermal coefficient 0.01/K represents a tactical-grade unit | typed, labelled (G) |
| S6 | Bit-identical concurrent and sequential branches | tested on small worlds (M); single machine |
| S7 | Fork cost stays near 1× trunk per step at larger grids | measured at 32² only (H-run) |
| S8 | String-named fields are produced by the stage that should produce them | not enforced by the type system (I) |
| S9 | A correlation evaluated per branch is an acceptable drag authority for SRP at this fidelity | stated, AMBER de-risk (H) |

### B.6 Inferences: the claims register

Strength: strong (direct measurement or code, independent reference), moderate (measured but scoped
or tripwire), weak (indirect or contradicted).

| # | Claim stub | Evidence | Class | Strength | Strongest attack | What to concede |
|---|---|---|---|---|---|---|
| C1 | A branch costs one shared march plus one continuation; the saving over rerun grows with the shared past | A.6, A.7; gates 4d, 4g | code + tripwire | strong | "A restart file does the same." | Equal step count to a restart at the pause (K). The differences: the pause point is found at run time by a condition on the evolved flow; branches share memory with no I/O; they run concurrently with sequential-identical results; each carries its own log. The crate's own disk checkpoint (`MarchState`) drops the marched layer (E). |
| C2 | Concurrent branches reproduce the sequential result bit for bit, and noise draws are deterministic | M; G | unit test | moderate | Tested on 2–3-step worlds; one machine; no cross-architecture check | Say exactly what is tested; claim no cross-platform bit identity. |
| C3 | The pause is flow-resolved: the run finds blackout onset from the evolved electron density | F, F-run, G-run, I | computed | strong for "found by the run"; open for "needs the marched layer" | "The closed-form post-shock state would find the same onset." | Untested until the D4 ablation runs. |
| C4 | Flow, chemistry, link, vehicle, navigation and safety gate step one shared state in one process | A, I | by construction | strong as description | "String-named fields fail silently." | S8 and the `FlightSensors` comment (A.3). |
| C5 | Every gate declares its evidence class, CI runs the suite, and failing baselines stay committed | A.10, P | process | strong as practice | "Most gates are tripwires; the audit was automated." | All application gates are tripwires; disclose the audit method (D5). |
| C6 | Composing a dispersion sweep with a mid-burn fork changes an in-flight decision: burn lit 18.61 m higher at 10.54 kg | H-run gate 5 | computed, tripwire | moderate | "The effect size is the typed IMU coefficient pushed through a stopping-distance kernel." | Concede. The claim is that the composition carries the table into flight with provenance. The physics does not predict the 18.6 m. |
| C7 | Branches depart a frozen-drag prediction; deceleration is non-monotone in throttle | H-run gates 4b, 4c | computed from correlation | moderate | "A 1-D trajectory code with the same correlation reproduces the table without CFD." | Probably true; the drag law is the correlation. The largest departure is the coast branch's foil artefact; burning branches depart by 38.9, 20.4, 0.6 and −4.1 m/s. |
| C8 | The solvers pass code verification on closed-form cases | A.11 reference rows | reference | strong at code-verification level | "Small grids, 2-D, low Re." | TGV −80 % at 16³; cylinder friction 13 % vs 25 %; QTT cylinder failing; QTT TG fails at its documented level 7. |
| C9 | The blackout chemistry lands within an order of magnitude of RAM-C II | B (stagline gates 5 and 8), F-run gate 2 | tripwire at 71.93 km; reference for the 61 km lower bound | moderate | "The band is a chosen allowance; one body station; exit 45.7 km vs flight 25–30 km; the anchor is one Ka-band crossing." | Concede the allowance, the single station and the exit offset. The anchor's altitude and meaning are settled (A.15): +0.22 dec in the harness at the cited freestream, −0.16 dec in the corridor at a matched crossing speed, and the 61 km lower bound met. Jones & Cross (1972) add electrostatic-probe ion densities on RAM C-I and C-II as an independent aft-body check. |
| C10 | Precision is a parameter | B (MMS ladder), N (Lorenz) | reference / computed | moderate | "The corridor does not compile at Float106." | Concede; demonstrate on MMS and the Lorenz example only. |
| C11 | The corridor's bank sweep selects a steering command | F-run gates 4c–4f | tripwire, by construction | weak as physics; strong as mechanism | "The decision reads no flow quantity; the gap it closes is set by construction." | Present it as the mechanism demonstration it is (K says so). |

### B.7 Points of view (fairness: represent each sympathetically)

| Reader | Strongest objection | Response material |
|---|---|---|
| CFD practitioner | A 32 × 32 quasi-steady 2-D layer at one pseudo-step per 0.1 s is not flight-fidelity CFD, and the library does not wrap an established solver. | Concede the scale (A.13, A.14). The contribution is the process pattern and its evidence discipline. State what an external solver would need: an in-memory state the host can share and clone, otherwise branching degrades to restart files. |
| GN&C engineer | Entry and landing decisions run on aerodatabases in 6-DOF Monte Carlo, not with CFD in the loop; the corridor's bank decision is computed by the 3-DOF model alone. | Concede (A.4). The weather → retropropulsion composition is the decision-relevant result, and the composition, not the CFD, carries it (C6). |
| Simulation researcher (parallel discrete-event simulation) | Simulation cloning shared state among clones copy-on-write two decades ago; checkpoint/restart is routine. | Cite and align (`cfd_relatedwork.md`, R1). The differences: a continuous coupled field, a pause found by a condition on that field, a typed campaign grammar, per-branch gates and logs. |
| Causal-inference researcher | No abduction: a branch from a known simulated state is an intervention, not a counterfactual. | D2. Define the term and place it on Pearl's hierarchy (R2). |
| V&V / certification reviewer | Tripwires are regression tests, not validation; one flight anchor; no grid convergence on the application; no model-form uncertainty. | Map each gate onto code verification, solution verification, validation or regression (P7.5). The crate's labels already separate reference from tripwire; the paper adds the V&V category. |
| Software-assurance reviewer | Gates that cannot fail; an audit run by automated agents. | The audit counted 72 tautology defects and the remediation made them falsifiable (J). Disclose the method. Mutation testing is the standing tool (AGENTS.md); report whether it ran on the fork and gate code. |

### B.8 Implications and consequences

- If the narrow thesis holds: decision studies on coupled simulations can share the expensive past,
  and the evidence discipline (classes, committed baselines, CI) transfers to any solver.
- If the paper claims the broad thesis: a reviewer who checks `stages.rs` finds that the corridor's
  decision reads no marched-field quantity. Credibility drops for the whole paper.
- If the D4 ablation shows the marched layer changes no decision: the paper says so, and the method
  claim stands unchanged. If it shows a change, the paper gains its first direct flow-to-decision
  result.
- Cost scales with the length of the shared past, N(P + B)/(P + N·B) (A.7). Short pasts (the
  corridor's P = 106 against B = 100) buy about 2×; long pasts approach N.
- Practical consequence for adopters: branches must run on a solver whose state lives in the host
  process; restart-file branching remains the path for external codes.

### B.9 Intellectual-standards check on the recommended thesis

| Standard | Question from the guide | Status |
|---|---|---|
| Clarity | Could you give an example? | Yes: the corridor fork code and the retropropulsion table |
| Accuracy | How could we check that? | Every number traces to a committed artifact (Part A); A.15 lists open discrepancies |
| Precision | Could you be more specific? | Costs as step counts and measured ratios, not adjectives |
| Relevance | How does that bear on the question? | Q3 needs A.4; without it the coupling claim floats |
| Depth | What makes this hard? | The flow only reaches the decision through n_e; the ablation is the hard part |
| Breadth | Another point of view? | B.7, six readers |
| Logic | Does it follow from the evidence? | Narrow thesis yes; broad thesis no (D1) |
| Significance | Is this the central idea? | The pause-fork-compare pattern plus evidence classes, not the reentry numbers |
| Fairness | Vested interest? Sympathetic to others? | Author maintains the library: disclose; cite cloning, restart and adjoint work as peers |

### B.10 Recommended thesis, as components (assemble in your own words)

- Subject: a coupled flow simulation, paused at a state the flow selects.
- Operation: branch into decision alternatives inside one typed program; reduce to gated rows.
- Cost: one shared march plus one continuation per branch; memory shared by reference.
- Guarantee: concurrent equals sequential; one log per branch naming its difference.
- Application result: a dispersion sweep composed with a mid-burn fork moves the landing-burn light
  altitude by 18.61 m at 10.54 kg of propellant.
- Scope: drag from a cited correlation; the flow sets the blackout window; one flight anchor at order
  of magnitude; 32 × 32 cells; one machine.

---

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
   1.17×. (F-run, G-run, H-run)
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
- Facts: TGV −80 %; cylinder friction split; QTT cylinder not converging (committed failing); QTT TG at
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
- Main point: three examples fly one reentry from Mach 27 at blackout onset to touchdown, each
  consuming the previous one's output.
- Facts: corridor → weather table → retropropulsion (plasma_blackout README via F); run order.
- Connect: chronological.
- Close: the third reads the second's table in flight.
- Transition: the vehicle and flight model.
- Verbs: fly, consume, produce, close.

**P5.2 Vehicle, trajectory and time.**
- Main point: a point-mass 3-DOF vehicle in compressed time.
- Facts: 90 km start, ~7.97 km/s; L/D 0.3; β ≈ 170 kg/m²; 0.1 s per coupled step; no winds or aero
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
- Facts: gates 4d, 4g; 0.66× coasting (C, K); A.7 counts; __RETRO_RSS__ MB peak RSS (H); wall clocks with
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
- Main point: the descent lands at 1.80 m/s after 5425 coupled steps in 346.9 s.
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

- Commands: `cargo run --release -p avionics_examples --example plasma_blackout_{corridor,weather,
  retropropulsion}`; `cargo run --release -p deep_causality_cfd --example <harness>` (A, B).
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
