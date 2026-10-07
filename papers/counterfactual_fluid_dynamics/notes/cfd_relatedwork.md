# Related work: raw material and paragraph plan

This file prepares the related-work section with the method of
`docs/writing_guides/TurnRawMaterialintoWriting.pdf`. It does not write the section. Part A is the raw
information, one block per work, each with its citation key in `../refs.bib`. Part B is the paragraph
plan: main point as a stub, facts in order, connectors, transition and verbs. You dictate the
sentences.

The search ran on 2026-10-06 in four clusters. Every bibliographic field in `../refs.bib` comes from a
fetched record (Crossref, DataCite, NASA NTRS, arXiv, OSTI, OpenAlex, PubMed, publisher PDFs). None
was filled from memory. Entries that could not be fully confirmed are marked `PARTIAL` in the
`.bib` file and listed in Part D.

Cluster IDs match the LaTeX skeleton (`../sections/02_related_work.tex`):

| ID | Cluster | Used in |
|---|---|---|
| R1 | Branching, cloning and checkpointing running simulations; adjoint alternative | Related work RW1, RW2; Discussion P7.3 |
| R2 | Interventions and counterfactuals | Related work RW3; Method P3.5; Discussion P7.4 |
| R3 | Causal analysis of flows | Related work RW4 |
| R4 | Coupled multiphysics, MDAO, co-simulation, digital twins | Related work RW5 |
| R5 | Numerical methods and benchmarks | Related work RW6; Section 4 |
| R6 | Reentry plasma, SRP, guidance and navigation | Section 5 background (not the related-work section) |
| R7 | Verification, validation and test adequacy | Related work RW7; Method P3.7; Discussion P7.5 |

---

## Part 0. Positioning facts (settle before dictating)

**Novelty search result.** The search found no prior work that combines, for a coupled CFD or
multiphysics solver, all of: a pause triggered by a condition on the evolved state, an in-memory
copy-on-write fork, one decision changed per branch, concurrent execution, and a gated reduction.

- Coverage: web search engines and publisher/indexing APIs. Scopus, Web of Science and patent
  databases were not searched. Two patents surfaced and were not verified: EP2652600 ("Virtual
  machine branching and parallel execution") and US 8805664 ("System and method for simulating
  branching behavior").
- Queries run included: "counterfactual" with "computational fluid dynamics"; "counterfactual digital
  twin"; "what-if" restart from checkpoint CFD; "simulation cloning" with CFD or multiphysics;
  reentry bank-angle branching from a mid-trajectory state; copy-on-write fork of a running
  simulation; "scenario tree" or "branching ensemble" with a restart file; DDDAS what-if simulations
  forked from the current state; adaptive multilevel splitting and rare-event cloning in turbulence.
- The "counterfactual digital twin" hits are outside CFD (healthcare, social simulation, world-model
  datasets).

**Precedent for each element except the gated reduction:**

| Element | Closest prior work | Key |
|---|---|---|
| Clone a running simulation at a decision point; share state among clones | cloning parallel discrete-event simulations | `hybinette2001cloning` |
| Replicate only what diverges, share the rest | incremental HLA federate cloning | `chen2005hlacloning` |
| Logical-copy cloning of time-stepped grid simulations | CloneX on GPUs | `yoginath2018clonex` |
| OS-level copy-on-write of a running computation | DMTCP forked checkpointing | `ansel2009dmtcp` |
| Branch a CFD run from an intermediate state with changed inputs | HDF5 "time reversible steering" | `ertl2017hdf5steering` |
| Clone CFD trajectories mid-run by a score on the evolved flow | rare-event sampling (AMS, GKTL) | `lestang2020rareevent` |
| Interventional branches from one DNS state | perturb-and-compare in turbulence | `jimenez2018machine`, `jimenez2020montecarlo` |
| Evaluate control decisions from a mid-run DNS state | DNS-based predictive control (by optimization) | `bewley2001dns` |
| Branch many coupled-model members from one state | climate large ensembles | `kay2015cesmle` |
| "Counterfactual world" as a model run with one factor changed | event attribution | `stott2004heatwave`, `nasem2016attribution`, `shepherd2016common` |

The search found no precedent for the gated reduction: branch rows reduced to a table and checked by
named pass/fail gates, each declaring whether its bound is a published reference or a pinned tripwire.

What follows for the paper: claim the combination and the evidence discipline, cite each element's
precedent where one exists, and use no "first" or "novel" without this table beside it (cfd_draft.md, Part D, check
4).

---

## Part A. Raw information

Format per entry: **key** · what it does (from the abstract or the source) · what it shares with the
pause-fork-compare method · where it differs. The "shares" and "differs" lines are factual
comparisons for you to draw on; they are not sentences for the paper.

### R1. Branching, cloning and checkpointing running simulations

**`hybinette2001cloning`** Hybinette & Fujimoto, ACM TOMACS 11(4) 2001.
- Does: clones a running parallel discrete-event simulation at decision points so that alternative
  execution paths run concurrently; "virtual logical processes" keep clones from repeating shared
  computation; demonstrated on a commercial air-traffic-control simulation.
- Shares: branching at a decision point, shared state across clones, concurrent what-if paths.
- Differs: discrete-event logical processes and messages, not a time-stepped field state; branches at
  decision points, not on a predicate over the evolved state.

**`chen2005hlacloning`** Chen, Turner, Cai, Gan & Low, ACM TOMACS 15(4) 2005.
- Does: a federate spawns clones at a decision point so alternative scenarios run in one session;
  incremental cloning replicates only the federates whose state will change and shares the rest.
- Shares: replicate-on-divergence, share-the-rest (copy-on-write at federate granularity).
- Differs: the unit of sharing is a whole HLA federate in a distributed federation.

**`yoginath2018clonex`** Yoginath & Perumalla, ACM TOMACS 28(1) 2018. `PARTIAL` (pages).
- Does: CloneX creates logical copies of a dynamic tree of simulations without full physical
  duplication; up to 1,024 GPUs; heat diffusion, forest fire, disease propagation; over 100× faster
  than replicated runs.
- Shares: logical-copy cloning of time-stepped grid simulations into what-if trees.
- Differs: no CFD or coupled multiphysics; no condition-triggered decision branches.

**`jefferson1985virtual`** Jefferson, ACM TOPLAS 7(3) 1985.
- Does: virtual time and the Time Warp mechanism; optimistic execution with state saving and
  rollback via antimessages.
- Shares: state saving and restoring an earlier state as runtime operations.
- Differs: rollback produces one causally correct history; alternative histories are not kept alive.

**`ansel2009dmtcp`** Ansel, Arya & Cooperman, IPDPS 2009.
- Does: transparent user-level checkpoint/restart for distributed multithreaded programs; "forked
  checkpointing" cuts checkpoint time to about 0.2 s.
- Shares: process `fork` is copy-on-write of a running computation's state.
- Differs: serves fault recovery of the same computation; no change of inputs per branch.

**`egwutuoha2013survey`** Egwutuoha, Levy, Selic & Chen, J. Supercomputing 65(3) 2013.
- Does: surveys HPC fault tolerance centred on rollback-recovery; taxonomy of over twenty
  checkpoint/restart systems.
- Shares: capture and restore the full state of a long HPC job.
- Differs: failure recovery throughout. (Alternative survey, verified: Shahzad et al., Parallel
  Processing Letters 23(4):1340011, 2013, doi 10.1142/S0129626413400112; not in `refs.bib`.)

**`ertl2017hdf5steering`** Ertl, Frisch & Mundani, Concurrency Computat. Pract. Exper. 29(24) 2017.
- Does: an HDF5 I/O kernel for massively parallel CFD with "time reversible steering": reload a
  checkpoint, alter boundary conditions; "having two or more possibilities leads to branching
  simulation paths".
- Shares: the closest CFD precedent for branching a run from an intermediate state with changed
  inputs.
- Differs: user-initiated from on-disk checkpoints for interactive design; no condition trigger, no
  in-memory copy-on-write, no concurrent reduction.

**`lestang2020rareevent`** Lestang, Bouchet & Lévêque, J. Fluid Mech. 895 A19 (2020).
- Does: AMS and GKTL algorithms on drag of a square obstacle in 2-D turbulent channel flow; many short
  parallel runs "with dynamics that is replicated or pruned".
- Shares: CFD trajectories cloned mid-run by a score on the evolved flow (state-triggered branching).
- Differs: clones differ by perturbation and noise; the goal is rare-event probability, not a
  decision. (Same group, climate model: Ragone, Wouters & Bouchet, PNAS 115(1):24–29, 2018, doi
  10.1073/pnas.1712645115; title not recorded in the check, so not in `refs.bib`.)

**`rabault2019drl`** Rabault, Kuchta, Jensen, Réglade & Cerardi, J. Fluid Mech. 865 (2019) 281–302.
- Does: deep reinforcement learning controls two jets on a Re = 100 cylinder; drag down about 8 %.
- Shares: "the simulation is first performed with no active control until a well developed unsteady
  wake is obtained, and the corresponding state is saved and used as a start for each subsequent
  learning episode" (arXiv v5): a saved mid-run state reused as the start of many control
  trajectories.
- Differs: episodes run in sequence for training; no condition-triggered fork, no comparison of fixed
  decisions.

**`bewley2001dns`** Bewley, Moin & Temam, J. Fluid Mech. 447 (2001) 179–225. `PARTIAL` (author given
names).
- Does: optimizes controls over a finite prediction horizon from the current DNS state by iterative
  DNS.
- Shares: evaluates control decisions from a mid-run CFD state.
- Differs: by adjoint-based optimization, not finite branching.

**`kay2015cesmle`** Kay et al., BAMS 96(8) 2015.
- Does: 40-member CESM1 ensemble, 1920–2100; each member "begins from a slightly different initial
  atmospheric state (created by randomly perturbing temperatures at the level of round-off error)".
- Shares: many coupled-model branches from one shared state, then compared.
- Differs: branches differ by round-off to sample internal variability; no decision changed; no
  condition trigger.

**`leutbecher2008ensemble`** Leutbecher & Palmer, J. Comput. Phys. 227(7) 2008.
- Does: reviews ensemble forecasting with initial-condition perturbations from leading singular
  vectors of the linearized dynamics (ECMWF EPS).
- Shares: many forecasts from one analysed state, flow-dependent perturbations.
- Differs: branches sample state uncertainty, not control decisions.

**`stott2004heatwave`** Stott, Stone & Allen, Nature 432 (2004) 610–614.
- Does: estimates that human influence very likely at least doubled the risk of the 2003 European
  summer-temperature extreme (erratum Nature 436:1200, 2005).
- Shares: the root of "counterfactual world" usage in climate science, with and without one factor.
- Differs: event probabilities across forcing regimes, not branches forked mid-run.

**`nasem2016attribution`** National Academies, 2016.
- Does: assesses extreme-event attribution; influence of climate change estimable for heat waves,
  drought and heavy precipitation.
- Shares: standard reference for factual-against-counterfactual model comparison.
- Differs: a consensus report, no mechanism.

**`shepherd2016common`** Shepherd, Current Climate Change Reports 2(1) 2016; **`shepherd2018storylines`**
Shepherd et al., Climatic Change 151 (2018).
- Does: contrasts risk-based attribution with a storyline approach that conditions on the event as it
  unfolded; a storyline is "a physically self-consistent unfolding of past events, or of plausible
  future events or pathways", with no a-priori probability.
- Shares: conditions on the realized trajectory and asks what changes under an altered factor; close
  to forking the actual state.
- Differs: a conceptual framework, no runtime mechanism.

**Adjoint alternative:**

**`giles2000adjoint`** Giles & Pierce, Flow Turbul. Combust. 65 (2000) 393–415.
- Does: introduction to adjoint-based design in CFD; one adjoint solve serves many design variables
  (their §2.4).
- Shares: computes d(objective)/d(design), the quantity branches approximate by differences.
- Differs: local gradients of smooth objectives; branches give full nonlinear outcomes for a few
  discrete decisions, including clamped ones (the corridor's 40° branch).

**`jameson1988control`** Jameson, J. Sci. Comput. 3(3) 1988.
- Does: control theory through adjoint equations gives feasible aerodynamic shape design.
- Shares and differs: as `giles2000adjoint`.

### R2. Interventions and counterfactuals

**`pearl2009causality`** Pearl, *Causality*, 2nd ed., CUP 2009. Locations confirmed:
- Def. 7.1.3 (effect of action), 7.1.4 (potential response), 7.1.5 (counterfactual): p. 204.
  Def. 7.1.5 reads Y_x(u) = y: "Y would be y (in situation u), had X been x".
- Theorem 7.1.7, p. 206, §7.1.1: P(B_A | e) by "1. Abduction – Update P(u) by the evidence e to obtain
  P(u | e). 2. Action – Modify M by the action do(A) … to obtain the submodel M_A. 3. Prediction".
- Informal version: §1.4.4, p. 37. Hierarchy "prediction, intervention, and counterfactuals …
  increasing demands on the knowledge required": p. 38.
- Twin network method: §7.1.4, p. 213. Relation to Lewis: §7.4.1, p. 238. Pearl (p. 37) says his
  account is "not based on an abstract notion of similarity among" worlds.
- p. 220, §7.2.2: a counterfactual's empirical content ties to the "uncertainty-producing variables
  (U) remain[ing] constant (until our next prediction or action)".
- Shares: Def. 7.1.5 at a fully known u is what a branch from a known simulated state computes.
- Differs: Theorem 7.1.7 handles unknown u by abduction; a simulation knows u.

**`bareinboim2022pearl`** Bareinboim, Correa, Ibeling & Icard, in *Probabilistic and Causal Inference:
The Works of Judea Pearl*, ACM 2022, pp. 507–556.
- Does: derives the three layers (associational, interventional, counterfactual) from an SCM; Causal
  Hierarchy Theorem: "data at one layer virtually always underdetermines information at higher
  layers".
- Layer 2 "encodes information about what would happen, hypothetically speaking, were some
  intervention to be performed" (Def. 27.5). Layer 3 asks "what would have happened … had some
  intervention been performed, given that something else in fact occurred (possibly conflicting with
  the hypothetical intervention)" (Def. 27.6).
- Every Layer-3 term "can be directly determined from a fully specified SCM".
- Shares: a deterministic simulator is a fully specified model.
- Differs: its Layer-3 example conditions on a factual outcome that conflicts with the intervention,
  for which "there is no conceivable experiment". The literature check derived (not stated in the
  paper) that with a point-mass P(U) the Layer-2 and Layer-3 valuations coincide; do not cite that
  derivation to them.

**`lewis1973counterfactuals`** Lewis, *Counterfactuals*, Harvard UP 1973; **`lewis1973causation`**
Lewis, "Causation", J. Philosophy 70(17) 1973.
- Does: semantics of counterfactual conditionals by comparative similarity of worlds; "A □→ C is true
  … iff … some A-world where C holds is closer (to w) than is any A-world where C does not hold".
- "We think of a cause as something that makes a difference, and the difference it makes must be a
  difference from what would have happened without it."
- Shares: a branch as a nearby world that matches the actual run up to the pause.
- Differs: closeness weighs law against particular fact ("It may be worth a small miracle to prolong
  or expand a region of perfect match"); a simulator fork fixes the laws and changes one input.

**`woodward2003making`** Woodward, *Making Things Happen*, OUP 2003.
- Does: "a manipulationist theory of causation and explanation: causal and explanatory relationships
  are relationships that are potentially exploitable for purposes of manipulation and control" (OUP
  abstract).
- Shares: changing one decision per branch with everything else fixed is an ideal intervention.
- Differs: no step infers a unit's background state.

**`halpern2016actual`** Halpern, *Actual Causality*, MIT Press 2016.
- Does: structural definitions of actual causation, responsibility, blame and explanation.
- "With actual causality, we know the outcome … and we retrospectively ask why it occurred."
- Shares: "did this bank angle cause this outcome in this run" is that kind of question.
- Differs: forward branch comparison gives but-for answers only; Halpern–Pearl add contingency
  conditions; "the choice of model can have a significant impact".

**`balke1994probabilistic`** Balke & Pearl, AAAI-94, pp. 230–237.
- Does: evaluates counterfactual queries in three steps with a twin network (factual and
  counterfactual copies of the variables); reads the antecedent "as an external action … consistent
  with Lewis' Miraculous Analysis".
- Holding response functions fixed assumes "the factual observations are presumed to occur not
  earlier than the counterfactual action"; otherwise it holds only "if the environment remains
  relatively static" between observation and action.
- Shares: a pause on an observed condition followed by a branch (action) is that case; a simulator
  carries the background exactly.

**`peters2017elements`** Peters, Janzing & Schölkopf, MIT Press 2017.
- Does: introduction to causal inference: structural assignments, noise variables, interventions,
  learning causal models.
- Use: textbook reference for SCM vocabulary only; its counterfactual definition and section number
  were not verified, so cite no definition number.

**`gerstenberg2021counterfactual`** Gerstenberg, Goodman, Lagnado & Tenenbaum, Psychol. Rev. 128(5)
2021.
- Does: the counterfactual simulation model "predicts causal judgments in physical settings by
  comparing what actually happened with what would have happened in relevant counterfactual
  situations"; tested on collision events.
- Shares: "counterfactual" as re-simulating physics with one factor changed.
- Differs: human causal judgment over simple rigid-body scenes.

**Host library (not external; cite the repository):** context alternation
(`deep_causality_core/src/traits/alternatable_context/mod.rs`): "re-evaluate the same causal law
against a different world by swapping the carried context mid-chain"; the effect propagation
process (`Hansen2025EPP`).

**Closing note for the terminology paragraph (from the literature check):** abduction requires a
prior P(U), evidence e, and a model detailed enough to compute P(u | e). With u fully known (a paused
simulated state), the abduction step reads the state, and a branch computes Pearl's Def. 7.1.5 at
that u. See cfd_draft.md, D2.

### R3. Causal analysis of flows

**`martinezsanchez2024surd`** Martínez-Sánchez, Arranz & Lozano-Durán, Nat. Commun. 15 (2024) 9296.
- Does: "SURD quantifies causality as the increments of redundant, unique, and synergistic information
  gained about future events from past observations. The formulation is non-intrusive."
- It calls interventions "intrusive (i.e., it requires modification of the system) and costly", and
  asks whether an intervention could change the outcome "as a consequence of forcing the system out of
  its natural state".
- Shares: the DeepCausality workspace implements SURD (`deep_causality_algorithms`, per AGENTS.md).
- Differs: SURD estimates causality from observed data; forking intervenes.

**`lozanoduran2020causality`** Lozano-Durán, Bae & Encinar, J. Fluid Mech. 882 A2 (2020).
- Does: transfer-entropy causality between eddies in channel-flow DNS.
- States "instantiated causality is only unambiguously identified by intrusively perturbing the
  system and observing the consequences (Pearl 2009)".
- Differs: its own measure is observational ("a probabilistic measure of causality rather than …
  individual events").

**`lozanoduran2021cause`** Lozano-Durán, Constantinou, Nikolaidis & Karp, J. Fluid Mech. 914 A8 (2021).
- Does: "cause-and-effect analysis based on interventions": modifies the Navier–Stokes equations in DNS
  to suppress chosen linear mechanisms.
- Shares: interventional DNS.
- Differs: intervenes on the governing equations for whole runs, not on decisions at a pause.

**`lozanoduran2022information`** Lozano-Durán & Arranz, Phys. Rev. Research 4 023195 (2022). (Venue is
PRR, not Data-Centric Engineering.)
- Does: causality as information flux among variables; control as a sensor-actuator pair "reducing
  the unknown information of the state"; applied to the turbulent cascade, LES subgrid modelling and
  drag reduction.
- Shares: links causal analysis of flows to control.
- Differs: estimated from trajectories.

**`jimenez2018machine`** Jiménez, J. Fluid Mech. 854 R1 (2018).
- Does: significance of a flow subvolume "defined as influence on the future evolution of the flow";
  perturbs regions of 2-D decaying turbulence DNS and measures the effect, as a "game" whose rules are
  the Navier–Stokes equations.
- Shares: interventional branches from one DNS state compared later.
- Differs: local flow perturbations to find causal structures, not control decisions. Follow-ups
  (verified, not in `refs.bib`): Encinar & Jiménez, JFM 965:A20 (2023); Osawa & Jiménez, JFM 1000:A4
  (2024).

**`jimenez2020montecarlo`** Jiménez, J. Turbulence 21(9–10) 2020, 544–566.
- Does: prepares flow fields, perturbs each "in various ways", runs 10–30 turnovers, ranks
  perturbations by divergence from the unperturbed run; "blind" randomised experiments in place of
  planned hypothesis tests.
- "interventional science relies on the results of experiments in which some condition is changed by
  the observer, generally in the hope of uncovering 'causal' (i.e., if this, then that), or
  'counterfactual' relations (if not this, then not that)". The closest fluid-mechanics use of the
  word for branched simulations; Jiménez cites Popper for it, not Pearl.

**`runge2019inferring`** Runge et al., Nat. Commun. 10 (2019) 2553.
- Does: reviews causal inference for time series in Earth system science; launches causeme.net.
- "real experiments are rarely feasible. However, a rapidly increasing amount of observational and
  simulated data opens up the use of … causal methods".
- Differs: simulated data as input to discovery, not a means of intervening.

**`campsvalls2023discovering`** Camps-Valls et al., Physics Reports 1044 (2023) 1–68.
- Does: reviews causal and equation discovery in physics, with fluid-dynamics case studies.
- "Discoveries emerge from observing the world and, when possible, performing interventional studies
  in the system under study." Its taxonomy is observational.

### R4. Coupled multiphysics, MDAO, co-simulation, digital twins

**`gray2019openmdao`** Gray, Hwang, Martins, Moore & Naylor, Struct. Multidisc. Optim. 59(4) 2019.
- Does: MDO framework that solves coupled models and computes their derivatives for gradient-based
  optimization.
- Shares: in-process composition of coupled disciplines.
- Differs: runtime component hierarchy with coupled solvers; the library composes a static stage
  stack stepping one shared field.

**`martins2013mdo`** Martins & Lambe, AIAA J. 51(9) 2013.
- Does: surveys MDO architectures. Shares: taxonomy of coupling. Differs: optimization architectures,
  not time-stepped composition.

**`chourdakis2022precice`** Chourdakis et al., Open Research Europe 2:51 (2022).
- Does: open-source library for partitioned multiphysics, coupling separate codes.
- Differs: partitioned coupling of separate solvers; the library runs one process over one field.

**`gomes2018cosim`** Gomes, Thule, Broman, Larsen & Vangheluwe, ACM CSUR 51(3) 2018.
- Does: surveys co-simulation of coupled heterogeneous systems modelled with different tools.
- Differs: the library's stages share one in-memory field, so it is outside that design space.

**`blochwitz2011fmi`** Blochwitz et al., 8th Modelica Conference, 2011, pp. 105–114.
- Does: the Functional Mock-up Interface, a tool-independent standard for model exchange and
  co-simulation.
- Differs: exchange of black-box units; the library composes stages at compile time.

**`glaessgen2012digitaltwin`** Glaessgen & Stargel, AIAA 2012-1818.
- Does: proposes the digital twin, which "integrates ultra-high fidelity simulation with the vehicle's
  on-board integrated vehicle health management system, maintenance history and all available
  historical and fleet data to mirror the life of its flying twin".
- Differs: vision for structural life, no branching mechanism.

**`kapteyn2021pgm`** Kapteyn, Pretorius & Willcox, Nat. Comput. Sci. 1(5) 2021.
- Does: asset–twin pair as coupled dynamical systems in a probabilistic graphical model;
  demonstrated on a UAV structural twin for in-flight decisions.
- Shares: decisions from the current estimated state.
- Differs: Bayesian inference and policies over a graphical model, not a forked PDE solver.

**`nasem2024digitaltwins`** National Academies, 2024.
- Does: defines a digital twin as modeling and simulation of a physical counterpart plus a
  bidirectional loop of data-driven updating and decision-making; sets a research agenda.

### R5. Numerical methods and benchmarks

**Discrete exterior calculus:**

- **`hirani2003dec`** Hirani, PhD thesis, Caltech 2003: DEC on simplicial complexes with circumcentric
  duals. The library uses cubical lattice complexes.
- **`desbrun2005dec`** Desbrun, Hirani, Leok & Marsden, arXiv:math/0508341: DEC on simplicial
  complexes of any dimension.
- **`elcott2007stable`** Elcott et al., ACM TOG 26(1) 2007: circulation-preserving simplicial fluids
  that evolve vorticity; graphics. The library evolves the velocity 1-form with a Leray projection.
- **`pavlov2011structure`** Pavlov et al., Physica D 240(6) 2011: variational Lie-group integrator with
  an exact discrete Kelvin theorem. The library uses a projection method.
- **`mohamed2016dec`** Mohamed, Hirani & Samtaney, J. Comput. Phys. 312 (2016) 175–191: DEC
  Navier–Stokes on surface simplicial meshes; second order on structured triangular meshes; mass and
  vorticity conserved to machine precision. The crate's solver follows this formulation on a lattice
  complex (`deep_causality_cfd/papers/README.md`).
- **`jagad2021primitive`** Jagad, Abukhwejah, Mohamed & Samtaney, Phys. Fluids 33 017114 (2021):
  primitive-variable (velocity–pressure) DEC; the closest DEC prior art to a velocity-form solver.
- **`arnold2006feec`** Arnold, Falk & Winther, Acta Numerica 15 (2006): finite element exterior
  calculus; the same de Rham viewpoint with polynomial spaces.

**Tensor networks for flows:**

- **`oseledets2011tt`** Oseledets, SISC 33(5) 2011: the tensor-train format and rounding.
- **`khoromskij2011quantics`** Khoromskij, Constr. Approx. 34(2) 2011: quantics (QTT) approximation,
  O(d log N); exponentials rank 1, degree-m polynomials rank ≤ m + 1.
- **`kazeev2012laplace`** Kazeev & Khoromskij, SIMAX 33(3) 2012: explicit low-rank QTT Laplacian and
  its 1-D inverse.
- **`gourianov2022quantum`** Gourianov et al., Nat. Comput. Sci. 2 (2022): incompressible turbulence in
  MPS form with over 10× fewer parameters than DNS.
- **`kiffner2023tensor`** Kiffner & Jaksch, Phys. Rev. Fluids 8 124101 (2023): tensor-network reduced
  order models for wall-bounded flow; lid-driven cavity benchmark.
- **`peddinti2024quantum`** Peddinti et al., Commun. Phys. 7 135 (2024): MPS solver for incompressible
  flow around immersed objects; memory and runtime logarithmic in mesh size. Published title has no
  leading "A".
- **`peddinti2025compressible`** Peddinti, Pisoni, Tiunov, Marini & Aolita, arXiv:2506.03833 (2025):
  MPS solver for 2-D Euler, central differences plus artificial dissipation, bodies as indicator masks;
  transonic NACA 0012 at χ = 100 and Sod. **The closest prior art to the crate's compressible QTT
  marcher.** Differs: the crate uses a Rusanov flux, Brinkman penalization and a Rankine–Hugoniot
  inflow strip.
- **`danis2025ttweno`** Danis et al., J. Comput. Phys. 529 (2025) 113891: tensor-train WENO for
  compressible Euler with cross interpolation; fifth order on smooth problems; captures shocks.
  (Companion: "Tensor-Train TENO Scheme for Compressible Flows", AIAA SciTech 2025, doi
  10.2514/6.2025-0304; not in `refs.bib`.)
- **`holscher2025gpu`** Hölscher et al., Phys. Rev. Research 7 013112 (2025): GPU-accelerated QTT 2-D
  turbulence to Re 1e7; χ = O(poly(1/ε)).

**Immersed boundaries and cut cells:**

- **`peskin2002ibm`** Peskin, Acta Numerica 11 (2002): the immersed boundary method.
- **`mittal2005ibm`** Mittal & Iaccarino, Annu. Rev. Fluid Mech. 37 (2005): continuous-forcing and
  discrete-forcing (including cut-cell) classes. The crate has one of each: cut cells in DEC,
  penalization in the QTT path.
- **`kirkpatrick2003cutcell`** Kirkpatrick, Armfield & Kent, J. Comput. Phys. 184 (2003): cut cells on a
  staggered 3-D Cartesian grid; the crate's wall-normal gradient recipe for friction.
- **`droge2005symmetry`** Dröge & Verstappen, IJNMF 47 (2005): symmetry-preserving cut-cell method;
  the crate's secondary reference for the cylinder pressure/friction split.
- **`angot1999penalization`** Angot, Bruneau & Fabrie, Numer. Math. 81 (1999): convergence and error
  estimates for Brinkman penalization.

**Benchmarks:**

- **`ghia1982high`** Ghia, Ghia & Shin, J. Comput. Phys. 48 (1982): lid-driven cavity to Re 10,000 on
  grids to 257², vorticity–stream-function, multigrid.
- **`williamson1996vortex`** Williamson, Annu. Rev. Fluid Mech. 28 (1996) 477–539: cylinder wake
  review.
- **`sod1978survey`** Sod, J. Comput. Phys. 27 (1978) 1–31: shock-tube comparison of schemes, Rusanov
  included.
- **`toro2009riemann`** Toro, 3rd ed., Springer 2009.
- **`taylor1937mechanism`**, **`brachet1983small`**, **`vanrees2011comparison`**: Taylor–Green origin,
  spectral DNS, Re 1600 reference curves.
- **`parvar2023evp`** Parvar, Iqbal, Ardekani, Brandt & Tammisola, arXiv:2303.09262 (2023). An
  elastoviscoplastic cylinder study. Its Newtonian Re = 100 validation table lists C_D: Kravchenko
  et al. 1.314, Oliveira 1.370, Sivakumar et al. 1.325, Posdziech & Grundmann 1.325, Mossaz et al.
  1.328, Qu et al. 1.319, Peng et al. 1.361, present 1.359; St 0.164–0.167. Cite the primary sources
  for the cylinder band (they were not individually verified in this check), and see Part D.

**Extended precision:**

- **`dekker1971floating`** Dekker, Numer. Math. 18 (1971): double-length arithmetic from exact
  operations on two floats; the basis of the 106-bit double-double scalar.
- **`hida2001quaddouble`** Hida, Li & Bailey, ARITH-15 (2001): quad-double algorithms (four
  components; the paper does not discuss double-double).

### R6. Application background (Section 5)

**Blackout and plasma:**

- **`grantham1970flight`** NASA TN D-6062: RAM C-II reflectometer data at 7.62 km/s at four body
  stations; four frequencies covering about 1e10–1e13 cm⁻³ (1e16–1e19 m⁻³). The anchor the code uses
  is the station-1 (x/D = 0.15) Ka-band (35 GHz, Table I) critical-density crossing at 236 000 ft
  (71.93 km, p. 18): N_cr = 1.287e-8 f² and N_pk/N_cr = 0.63 (p. 11) give 9.93e18 m⁻³. Below that
  altitude station 1 is overdense at Ka-band, so the value is a lower bound down to 56.39 km
  (cfd_draft.md, A.15).
- **Parent, Thoguluva Rajendran & Omprakas**, "Electron Losses in Hypersonic Flows", arXiv:2111.09432:
  the RAM-C II freestream the stagline harness flies (71 km: M 25.9, 217.9 K, q 2.28 kPa; 61 km:
  M 23.9, 255.9 K, q 8 kPa). Not in `refs.bib` yet; take the fields from the arXiv record.
- **`jones1972electrostatic`** NASA TN D-6617: electrostatic-probe ion densities 1–7 cm off the aft
  surfaces of RAM C-I and C-II, 85.3–53.3 km, 1e8–1e12 cm⁻³; an independent aft-body check.
- **`hartunian2007causes`** Aerospace Corp. ATR-2007(5309)-1: causes and mitigation of RF blackout for
  reusable launch vehicles.
- **`rybak1971progress`** IEEE TAES 7(5) 1971: classic review of reentry communications.
- **`kim2008analysis`** JSR 45(6) 2008: E×B mitigation; names GPS loss. (The other 2008 JSR paper is
  Keidar, Kim & Boyd, 45(3):445–453, doi 10.2514/1.32147; not in `refs.bib`.)
- **`farbar2013numerical`** JTHT 27(4) 2013: separate electron temperature changes predicted n_e; the
  source of the crate's ~2× lumping caveat.
- **`boyd2007modeling`** Phys. Fluids 19 096102 (2007): DSMC of associative ionization, the dominant
  path near 8 km/s.

**Thermochemistry:** `park1990nonequilibrium`, `park1993review`, `gupta1990review` (title says
30000 K; the NTRS abstract's "3000 K" is a typo), `millikan1963systematics`, `sutton1971general`.

**Supersonic retropropulsion:**

- **`jarvinen1970aerodynamic`** NASA-CR-124720 (Mithras report MC 70-3001-R2(BNY)), Feb 1970: wind
  tunnel at Mach 0.4–2.0, α to 18°, C_T to 30, with an analytic single-jet model. Companion: Adams &
  Jarvinen, AIAA Paper 70-219, doi 10.2514/6.1970-219 (not in `refs.bib`).
- **`keyes1967effect`** JSR 4(4) 1967, 533–534: Mach 6 forward-facing jets, "emphasizing drag
  increase", the opposite sign to the low-C_T collapse.
- **`korzun2009survey`**, **`edquist2014development`**, **`braun2007mars`**: SRP survey, maturation
  roadmap, Mars EDL challenges.
- **`cordell2013analytical`** JSR 50(4) 2013: analytic SRP plume structure.

**Guidance and navigation:** `meditch1964problem` (minimum-fuel soft landing is bang-bang),
`acikmese2007convex`, `blackmore2010minimum`, `harpold1979shuttle` (`PARTIAL`), `lu2014entry`,
`sola2017quaternion`, `kustaanheimo1965perturbation`, `stiefel1971linear` (`PARTIAL`).

### R7. Verification, validation and test adequacy

- **`oberkampf2002verification`** Prog. Aerosp. Sci. 38 (2002): separates code from solution
  verification, and model from solution validation.
- **`roache1998verification`** Hermosa 1998 (`PARTIAL`, ISBN): code and solution verification, MMS,
  Grid Convergence Index.
- **`oberkampf2010verification`** CUP 2010. A 2nd edition exists (CUP 2025, doi
  10.1017/9781009031004) titled *Verification, Validation, and Uncertainty Quantification in
  Scientific Computing*; choose one edition.
- **`roy2011comprehensive`** CMAME 200 (2011): aleatory as distributions, epistemic as intervals
  (p-boxes), numerical error included. The crate's gates are pass/fail.
- **`aiaa1998guide`** AIAA G-077-1998(2002): V&V definitions for CFD.
- **`asme2009vv20`** ASME V&V 20-2009 (R2021) (`PARTIAL`): validation "for a specified variable at a
  specified validation point", with a quantified validation uncertainty that order-of-magnitude gates
  do not provide.
- **`nasa2024std7009b`** NASA-STD-7009B (2024-03-05), supersedes 7009A w/Change 1 (2016): M&S
  credibility; revision B adds tracking of M&S defects.
- **`slotnick2014cfd`** NASA/CR-2014-218178 and **`cary2021cfd`** AIAA 2021-2726: Vision 2030 and its
  progress review; both name V&V and UQ as gaps. (A 2026 update exists: AIAA 2026-4020, Wissink et al.;
  not verified, not in `refs.bib`.)
- **`roache2002code`**, **`salari2000code`**: MMS; Salari & Knupp's blind planted-fault tests are
  close to the crate's tautology audit.
- **`demillo1978hints`**: mutation analysis, the coupling effect; basis of `cargo mutants`
  (`scripts/mutants.sh`).
- **`jia2011analysis`**: mutation-testing survey, equivalent mutants (`.cargo/mutants.toml`).
- **`kanewala2014testing`**: testing scientific software; the oracle problem.
- **`barr2015oracle`**: oracle taxonomy. `[reference]` gates map to specified oracles, `[tripwire]`
  gates to derived (regression) oracles.

---

## Part B. Paragraph plan (steps 1 to 4)

Seven paragraphs, about 1,200–1,600 words. Each paragraph names prior work, says what it does, and
places the paper beside it. Keep the comparisons as statements of what each does (project rule:
state, don't argue).

**RW1. Branching a running simulation.** (R1)
- Main point: branching a running simulation from shared state is established in discrete-event and
  grid simulation, and has appeared in CFD as checkpoint steering and rare-event cloning.
- Facts, in order: `hybinette2001cloning` (decision-point cloning, shared state); `chen2005hlacloning`
  (incremental, share what does not diverge); `yoginath2018clonex` (time-stepped grids, logical
  copies); `jefferson1985virtual` (state saving for rollback); `ansel2009dmtcp`,
  `egwutuoha2013survey` (checkpoint/restart for recovery); `ertl2017hdf5steering` (CFD branches from
  checkpoints); `lestang2020rareevent` (state-scored cloning of CFD trajectories).
- Connect: chronological (1985 → 2001 → 2005 → 2017–2020); addition; contrast at the end.
- Close: what the paper adds, as a statement: a condition on the evolved field selects the pause;
  branches share memory and change one decision; the reduction is gated (Part 0 table).
- Transition: branching is one way to evaluate decisions; optimization and training are others.
- Verbs: clone, share, replicate, roll back, steer, prune.

**RW2. Evaluating decisions from a flow state.** (R1)
- Main point: CFD evaluates control decisions from a mid-run state by optimization and by training.
  Ensembles also branch many members from one state, but to sample uncertainty in the state, not to
  compare decisions.
- Facts: `giles2000adjoint`, `jameson1988control` (one adjoint solve, many design variables, smooth
  objectives); `bewley2001dns` (predictive control from the current DNS state); `rabault2019drl`
  (a saved developed wake reused for every episode); `kay2015cesmle`, `leutbecher2008ensemble` (many
  members from one state, sampling uncertainty).
- Connect: on the one hand … on the other hand; exemplifying.
- Close: branching evaluates a few discrete or clamped decisions with their full nonlinear outcome
  (the corridor's 40° branch flies clamped, cfd_draft.md A.14).
- Transition: what to call such a branch.
- Verbs: differentiate, optimize, train, sample, evaluate.

**RW3. Interventions and counterfactuals.** (R2, with the climate usage from R1)
- Main point: the causal-inference literature separates interventions from counterfactuals by what is
  known about the background; a fully known simulated state collapses that difference.
- Facts: `pearl2009causality` (Def. 7.1.5 p. 204; Theorem 7.1.7 p. 206; hierarchy p. 38);
  `bareinboim2022pearl` (layers; fully specified model determines Layer 3); `balke1994probabilistic`
  (observation before action, "relatively static"); `lewis1973counterfactuals`,
  `lewis1973causation` (closest worlds; difference-making); `woodward2003making` (ideal
  intervention); `halpern2016actual` (actual causation, model choice); usage in other fields:
  `stott2004heatwave`, `nasem2016attribution`, `shepherd2016common`, `shepherd2018storylines`
  (climate), `gerstenberg2021counterfactual` (cognitive science); the host library's context
  alternation (`Hansen2025EPP`, repository).
- Connect: reformulating ("in other words"); qualification; exemplifying for the other fields.
- Close: the definition the paper adopts (cfd_draft.md D2; Definition in `03_method.tex`).
- Transition: causal analysis inside fluid mechanics.
- Verbs: intervene, abduct, evaluate, condition, attribute.

**RW4. Causal analysis of flows.** (R3)
- Main point: causal analysis in fluid mechanics runs mostly on observational information measures,
  with a growing interventional strand that perturbs simulations.
- Facts: `lozanoduran2020causality` (transfer entropy; "instantiated causality is only unambiguously
  identified by intrusively perturbing"); `martinezsanchez2024surd` (SURD, non-intrusive; implemented
  in the same workspace); `lozanoduran2022information` (information flux and control);
  `lozanoduran2021cause` (interventions on the governing equations); `jimenez2018machine`,
  `jimenez2020montecarlo` (perturbed-simulation experiments; "counterfactual relations");
  `runge2019inferring`, `campsvalls2023discovering` (observational discovery reviews).
- Connect: contrast (observational … interventional); chronological within the interventional strand.
- Close: those interventions perturb the flow to find structure; the paper's branches change a
  decision to choose one.
- Transition: where the flow sits among the other disciplines.
- Verbs: measure, perturb, suppress, rank, decompose.

**RW5. Coupled multiphysics and digital twins.** (R4)
- Main point: multiphysics frameworks couple disciplines by exchange between solvers or by
  in-process component graphs; digital twins put a decision loop around such a model.
- Facts: `chourdakis2022precice`, `gomes2018cosim`, `blochwitz2011fmi` (partitioned and co-simulation,
  exchange of units); `gray2019openmdao`, `martins2013mdo` (in-process MDAO with derivatives);
  `glaessgen2012digitaltwin`, `kapteyn2021pgm`, `nasem2024digitaltwins` (twin definition, decisions
  from the current state).
- Connect: addition; contrast (exchange … one shared field).
- Close: the library composes stages at compile time over one field (cfd_draft.md A.3), and forks
  that field.
- Transition: the solvers that step it.
- Verbs: couple, exchange, compose, mirror, decide.

**RW6. Numerical methods.** (R5; may split into two paragraphs: DEC and immersed bodies; tensor
networks)
- Main point: the solvers follow established DEC, cut-cell, penalization and tensor-train methods on
  a cubical lattice and in quantized form.
- Facts: DEC lineage (`hirani2003dec`, `desbrun2005dec`, `elcott2007stable`, `pavlov2011structure`,
  `mohamed2016dec`, `jagad2021primitive`, `arnold2006feec`); immersed bodies (`peskin2002ibm`,
  `mittal2005ibm`, `kirkpatrick2003cutcell`, `droge2005symmetry`, `angot1999penalization`); tensor
  trains (`oseledets2011tt`, `khoromskij2011quantics`, `kazeev2012laplace`); tensor-network CFD
  (`gourianov2022quantum`, `kiffner2023tensor`, `peddinti2024quantum`, `holscher2025gpu`), compressible
  (`danis2025ttweno`, `peddinti2025compressible`); precision (`dekker1971floating`).
- Connect: chronological within each lineage; contrast for what the crate does differently (cubical
  complexes; Rusanov flux with Brinkman bodies and a fitted inflow strip).
- Close: the methods are inherited; the contribution sits in the process around them.
- Transition: how the paper's evidence relates to V&V practice.
- Verbs: discretize, project, penalize, quantize, round.

**RW7. Verification, validation and test adequacy.** (R7)
- Main point: CFD credibility rests on separating code verification, solution verification and
  validation; software testing adds the question of whether a check can fail.
- Facts: `oberkampf2002verification`, `roache1998verification`, `oberkampf2010verification`,
  `roy2011comprehensive`, `aiaa1998guide`, `asme2009vv20`, `nasa2024std7009b` (categories,
  credibility, defect tracking); `slotnick2014cfd`, `cary2021cfd` (V&V and UQ named as gaps);
  `roache2002code`, `salari2000code` (MMS, planted faults); `demillo1978hints`, `jia2011analysis`
  (mutation testing); `kanewala2014testing`, `barr2015oracle` (oracle problem; specified against
  derived oracles).
- Connect: addition; reformulating (reference ≈ specified oracle, tripwire ≈ derived regression
  oracle).
- Close: the crate's labels make the oracle class of every gate explicit; the evidence-map table
  (`../tables/tab_evidence_map.tex`) maps them onto V&V categories.
- Transition: into Section 3, the method.
- Verbs: verify, validate, label, mutate, pin.

---

## Part C. Fairness check (represent each position at its strongest)

| Position | Its strongest form | Material |
|---|---|---|
| Simulation cloning already does this | Shared-state cloning at decision points with concurrent branches has existed since 2001 and scales to 1,024 GPUs | R1: Hybinette & Fujimoto; Chen et al.; Yoginath & Perumalla |
| Checkpoint/restart is enough | A restart file at the pause costs the same solver steps as a fork (website, cfd_draft.md A.7) | R1: DMTCP; Ertl et al. |
| Adjoints are better for decisions | One adjoint solve gives the gradient for all design variables | R1: Giles & Pierce; Jameson; Bewley et al. |
| It is an intervention, not a counterfactual | Abduction is what makes a counterfactual; a known state skips it | R2: Pearl Thm 7.1.7; Bareinboim et al. |
| Interventional simulation is already used for causality in turbulence | Jiménez perturbs DNS states and calls the relations counterfactual | R3: Jiménez 2018, 2020; Lozano-Durán et al. 2021 |
| Compressible tensor-train CFD with bodies exists | MPS Euler with body masks and a Sod check, and TT-WENO | R5: Peddinti et al. 2025; Danis et al. 2025 |
| V&V practice already separates these | Code verification, solution verification and validation are standard categories | R7: Oberkampf & Trucano; Oberkampf & Roy |

---

## Part D. Bibliography notes and corrections

**`PARTIAL` entries in `../refs.bib`** (complete before submission):

- `yoginath2018clonex`: pages / article number (Crossref 11–26 against ORNL "Article 5").
- `bewley2001dns`: author given names and initials.
- `harpold1979shuttle`: journal volume, issue and pages from secondary citations only.
- `stiefel1971linear`: series volume number.
- `roache1998verification`: ISBN from a bookseller listing.
- `asme2009vv20`: publisher city and ISBN.
- `parvar2023evp`: journal venue (only arXiv and SSRN found).
- `hansen_deepcausality`: no year (BibTeX warns).

**Corrections applied in the repository after the check:**

- The cylinder C_d reference band is 1.314–1.370, cited to Parvar et al. (2023) Table 1, seven
  Newtonian sources (`deep_causality_cfd/verification/README.md`,
  `deep_causality_cfd/verification/dec_cylinder_verification/main.rs` and its README).
- Every citation of Peddinti et al. (2024) uses the published title.
- `deep_causality_physics/src/kernels/propulsion/srp.rs` and `.../propulsion/plume.rs` cite
  Cordell & Braun (2013) under its published
  title, "Analytical Modeling of Supersonic Retropropulsion Plume Structures".
- `deep_causality_cfd/papers/README.md` and `deep_causality_cfd/src/tensor_bridge/mod.rs` carry the
  Kazeev & Khoromskij
  venue (SIMAX 33(3):742–758, 2012).

The two SRP PDFs in `deep_causality_physics/papers/` are the IEEE Aerospace 2008 version of Korzun,
Cruz & Braun and Cordell's 2013 thesis; the code cites them as those documents.

**Other notes:**

- Keyes & Hefner (1967) is a two-page JSR article, not a NASA TN.
- Jones & Cross (1972) covers RAM C-I and C-II.
- Kustaanheimo & Stiefel: the Crossref record wrongly lists two extra co-authors; `refs.bib` carries
  the two correct authors. Do not re-import that record automatically.
- AIAA ARC pages returned 403 during the check; AIAA fields come from Crossref, which AIAA deposits.
