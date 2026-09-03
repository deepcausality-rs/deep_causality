# Dynamic Causality for a Dynamic World: From Causal Decisions to Safeguarded Actions

Presentation proposal for the Causal Data Science Meeting 2026 (online, November 4 to 5, 2026).

**Presenter.** Marvin Hansen, Director, Center for Dynamic Causality; creator and maintainer of
DeepCausality, a Linux Foundation AI & Data project. marvin.hansen@causalcenter.com

**Format.** Talk, 25 minutes plus questions. The material cuts to 15 minutes or extends to 30 if
the program needs it. One short live demonstration, recorded or live at the organizers' choice.

**Topics from the call.** Causal discovery and root-cause analysis. Open-source software for
causal inference. Causal ML/AI for business decision-making. Organizational challenges and best
practice for implementing causal inference. Interplay between causality and generative AI.

## Abstract

The causal frameworks in everyday use share a premise inherited from classical philosophy: causal
structure is enumerated up front and held constant, cause precedes effect along one time axis,
and the world in which the model runs holds still. Business systems break all three. A network
changes regime in the middle of an incident, a market changes its rules mid-quarter, and a
supply chain that was well described on Monday is a different system on Friday. This talk
presents dynamic causality, a working definition under which causal rules, causal structure, and
the surrounding context may change while the system is running, together with DeepCausality, the
open-source Rust implementation. The talk follows one path from decision to action: discover
structure from data, encode it with an explicit context, review a decision in counterfactual
branches before it reaches the world, and let a deontic safety layer decide which actions are
obligatory, optional, or impermissible, with an audit trail for every step. Structural causal
models, dynamic Bayesian networks, Granger causality, and the Rubin model fall out as special
cases of the same definition. A case study from a network operations product deployed at United
Airlines closes the talk.

## Extended abstract

### The premise

Pearl's structural causal model, the potential-outcomes framework, Granger causality, and dynamic
Bayesian networks are sound on their own ground. Each assumes that the analyst enumerates the
causal structure before estimation and that the structure stays put while the model is in use.
Applied work in management and econometrics has spent a decade learning how much rides on that
enumeration: which variables are controls and which are colliders, what a path model can and
cannot say, when automated confounder selection admits a bad control. Those lessons hold. They
also assume a world that keeps still long enough to be enumerated.

Operational decisions rarely get that world. A monitoring system sees a circuit degrade at
02:14 and the services that depended on it reorganize by 02:20. The causal graph that explained the
network at 02:00 is a different graph at 02:20, and the rules that mapped symptoms to causes have
changed with it. Refitting a static model after every regime change is possible in a notebook and
impossible in a control loop. The question this talk addresses is what a causal model has to look
like when the structure, the rules, and the context are allowed to move.

### One working definition

Dynamic causality is defined as a process: one propagating effect is obtained from the previous
one by applying a causal function within a composition rule. Time and space are not built into
the relation. They are declared inputs that the causal function reads from an explicit context, a
hypergraph holding anything time-like, anything space-like, and the data. Whether that context is
a calendar and a floor plan, a Euclidean grid, or a Minkowski spacetime is the modeler's choice.

The definition is small and it computes. Two primitives carry it. A *Causaloid* is a
self-contained unit of causal logic; it can stand alone, sit in a collection, or form a graph.
A *Context* is the explicit environment the Causaloids read. The composition rule threads five
things through every step: the value, the state, the context, any error, and an append-only audit
log. Everything a Causaloid emits is the same carrier, so structural reasoning over a graph and
sequential reasoning over time compose in one path.

Because the classical frameworks are the fixed-structure case of this definition, they are
implemented as runnable examples rather than replaced: a structural causal model, a dynamic
Bayesian network, Granger causality, and the Rubin causal model with conditional average treatment
effects. What the definition adds are three modalities a fixed-structure framework cannot express:
causal structure that changes as the context changes, causal rules that update themselves at
runtime, and structure that emerges at runtime and remains governable.

### From decision to action

The talk walks one decision through five stations.

**Discover.** The Causal Discovery Language surfaces candidate structure from observational data.
It ships an information-theoretic decomposition into synergistic, unique, and redundant causal
contributions (SURD), a redundancy-aware feature selector (MRMR), and Bayesian root-cause
discovery (BRCD), which reasons over the Markov equivalence class and reports its uncertainty over
the class by bootstrap. Discovery proposes; the analyst encodes. The assumptions that the
control-variable literature insists on stay visible, in code a reviewer can read.

**Model.** The encoded structure becomes a Causaloid graph over an explicit context. The regime is
a property the model reads off the context, so one model serves several regimes and reports which
one it is in. The same mechanism carries a model from one population or one site to another
without silently assuming the two are alike.

**Review before acting.** A running model can be forked into counterfactual branches. Each branch
receives an intervention, continues in its own world, and carries its own audit log; the branches
reduce to a verdict that the caller maps to a decision. A proposed action is tried in the model
before it is tried on the business, and the record shows which alternatives were considered and
why they lost.

**Act.** A Causal State Machine infers the active states at runtime and turns a causal verdict into
an action against the outside world.

**Govern.** Before an action fires, the Effect Ethos evaluates it under a defeasible deontic
calculus: rules with exceptions and priorities, encoded by the people accountable for the system.
The verdict is one of three. An obligatory or optional action fires and writes an audit entry. An
impermissible action is stopped and the reason is written to the log. The gate is the same whether
a person, a statistical model, or an AI agent proposed the action, which is where this design meets
the question of what an autonomous system may be allowed to do.

One consequence deserves to be stated plainly. Where structure emerges at runtime, static
verification of the reasoning stops being deterministic. Verifiability is restored at the action
boundary, because the Effect Ethos is deterministic and its verdicts are logged. The composition
laws and the graph algebra underneath are formalized in Lean 4; at the time of writing 183
theorems are bound to executable Rust witnesses.

### Case study

ServiceRadar, a network operations product by Carver Automation, is deployed at United Airlines
for network monitoring, network management, security management, and IT operations. DeepCausality
provides its real-time anomaly detection at about one million operations per second, built on the
project's hypergraph store, its core reasoning engine, and dynamic context. The integration is
structured to continue toward cross-domain diagnostics and dynamic incident response. The talk
shows what the integration required from the causal model and what it did not.

### The project

DeepCausality entered the Linux Foundation AI & Data sandbox in September 2023 with four crates.
In September 2026 it has 29 crates, 14,463 tests, and 113 runnable examples across 14 domains,
under the MIT license; 23 of the 29 crates have no external dependencies. Documentation, a project
website, and the classical-causality examples are public.

### What the audience takes away

- A working definition of causality that keeps the classical frameworks and adds regime change,
  adaptive rules, and emergent structure.
- A pattern for taking a causal decision to a governed action, with a record of the alternatives
  considered and the rule that permitted or stopped the action.
- A place to start: the classical-causality examples, the discovery language, and the Effect Ethos
  examples, each runnable with one command.

### Outline, 25 minutes

| Minutes | Section |
|---|---|
| 0 to 3 | The premise: the fixed-structure assumption meets an operational incident |
| 3 to 9 | The definition, the two primitives, and the classical frameworks as special cases |
| 9 to 17 | From decision to action: discover, model, review, act, govern; one live example |
| 17 to 22 | ServiceRadar at United Airlines |
| 22 to 25 | What to run tomorrow, open questions, discussion |

### Speaker

Marvin Hansen is Director of the Center for Dynamic Causality and the creator of DeepCausality,
which he has maintained since its admission to the Linux Foundation AI & Data sandbox in 2023.

### Links

- Project website: https://www.deepcausality.com
- Documentation: https://docs.deepcausality.com
- Repository: https://github.com/deepcausality-rs/deep_causality
- Classical frameworks as special cases: `examples/classical_causality_examples` in the repository
- Blog: "Counterfactuals via the Causal Monad" and "The Causal Discovery Language, Rebuilt"

---

## Cover email

To: submission@causalscience.org
Subject: Submission: CDSM2026 · Dynamic Causality for a Dynamic World

Dear Paul, Jermain, and Beyers,

Please find attached a presentation proposal for the Causal Data Science Meeting 2026.

Title: Dynamic Causality for a Dynamic World: From Causal Decisions to Safeguarded Actions.

The talk presents a working definition of causality under which structure, rules, and context may
change while a system runs, and DeepCausality, the open-source Rust implementation hosted by the
Linux Foundation AI & Data. It follows one decision from causal discovery through counterfactual
review to a governed action with an audit trail, and closes with a production case study from a
network operations product deployed at United Airlines. Structural causal models, dynamic Bayesian
networks, Granger causality, and the Rubin model run as special cases of the same definition.

Format: 25 minutes plus questions; the material adapts to a shorter slot. The abstract is repeated
below the signature.

Kind regards,

Marvin Hansen
Director, Center for Dynamic Causality
marvin.hansen@causalcenter.com
https://www.deepcausality.com
