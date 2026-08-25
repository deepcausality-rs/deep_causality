# DeepCausality — Speaker Notes

LF AI & Data Technical Advisory Council, 2026.  
Marvin Hansen · Director, Center for Dynamic Causality.

Deck: `DeepCausality_TAC_2026.pptx` · 15 slides · budgeted 15–20 minutes.

These notes are also embedded in the PowerPoint notes pane, so Presenter View
shows the same text.

---

## Slide 01 — Title

Good morning, and thank you for the slot.

I am Marvin Hansen, Director of the Center for Dynamic Causality, and I am here
for DeepCausality — an LF AI & Data Sandbox project since September 2023.

I presented here in 2023 with a proposal. This time I am back with a re-introduction,
because almost everything below the surface has changed since then, and because
this council has largely turned over in the meantime.

Twenty minutes. Six parts. I will keep the mathematics to one slide, and I will
spend most of the time on what the project actually ships.

---

## Slide 02 — Agenda

Six parts.

First the premise — one slide on why we did not simply build another causal-graph
library. Then the axiom and the primitives that make it computable. Then the piece
I most want you to take away: one causal stack that runs from raw data all the way
to a governed production action.

Then two downstream projects that were built on top of that stack and that are, in
my view, the proof that the substrate generalizes: counterfactual fluid dynamics,
and quantum causal models.

Then a case study, and finally project health and where we could use help.

Roughly one minute per slide. Stop me at any point.

---

## Slide 03 — The premise

This is the slide the whole project rests on, so I will spend a full minute here.

Classical computational causality traces its definition of cause back to Seneca —
roughly two thousand years old — and it inherits the static spacetime that came with
it. Pearl's structural causal models, Granger, Rubin, dynamic Bayesian networks:
all excellent, all well validated, and all assuming that time runs straight, that
the rules are fixed at design time, and that the causal structure is enumerated up
front.

That assumption is not a flaw. It is a scope. And contemporary science has already
moved outside it. In quantum physics and in general relativity the fixed background
does not hold. Closer to home, any system that crosses a physical or operational
regime boundary changes its own causal rules mid-flight.

So DeepCausality changes the starting point. We root the project in Whitehead's
process philosophy — causality as a process of becoming — and we make spacetime
data that the model reads rather than a frame the model assumes.

The consequence is on the right: rules may evolve at runtime, structure may emerge,
and — this is the part that matters for production — it is still governable. I will
come back to that.

---

## Slide 04 — The axiom

Here is the whole premise in one line: m-two equals m-one bind f.

Read it as: dynamic causality is the spacetime-agnostic monadic process in which one
propagating effect is obtained from another by applying a causal function within the
monad.

Four things are packed in there.

Monadic process — the carrier is an arity-five record: value, state, context, error,
and an audit log. Because it obeys the monad laws, that bookkeeping is threaded
automatically. That is where end-to-end explainability comes from; nobody has to
remember to log.

Functional dependency — each effect comes from the previous one by applying a causal
function. Chain those and you get effect propagation.

Spacetime-agnostic — time and space are not in the relation. They are inputs.

And because they are not built in, anything time-like or space-like has to live
somewhere explicit: the Context, a typed hypergraph. That is what lets the same model
run in Euclidean space, in Minkowski spacetime, or in a non-Euclidean context.

If you want the intuition: a ripple in a pond. One ripple is an effect, it propagates,
it produces the next one. We define how ripples spread — and what happens when the
rules for spreading change.

---

## Slide 05 — Consequence

Two consequences, and the second one is the one I would defend hardest.

First: because the axiom sits low enough in the stack, we did not have to replace
anything. Pearl's structural causal models, dynamic Bayesian networks, Granger,
Rubin, conditional average treatment effects — each drops out as a parametric
specialization of the same relation. We do not argue that; we implement each one
directly in the classical-causality examples folder so you can read the code.

Second: it adds three modalities a fixed-structure framework cannot express.
Dynamic — structure changes with context. Adaptive — the rules update at runtime.
Emergent — structure that was not enumerable at design time.

Now, the honest objection: if the structure can emerge at runtime, you cannot
statically verify the reasoning any more. That is true, and we do not pretend
otherwise.

Our answer is the line at the bottom. Reasoning is free to be emergent; actions are
not. We move verifiability from the reasoning boundary to the action boundary. That
is a deliberate architectural trade, and it is the thing that makes emergent causality
safe enough to deploy. Two slides from now you will see the layer that does it.

---

## Slide 06 — Primitives

Three primitives operationalize the axiom, plus an optional fourth for safety.

On the left, two reasoning primitives. The Causaloid carries causal structure, and
it is isomorphic-recursive: a singleton, a collection, and a hypergraph all implement
the same trait surface, so they nest into each other to any depth. The Causal Monad
carries sequencing — pure, bind, and intervene, where intervene is Pearl's do-operator
applied mid-chain.

The important part is the middle. Both emit the same carrier. That means you can take
a Causaloid's verdict and bind directly onto it, or run a bind chain and feed the
result into a Causaloid. The boundary between structural and sequential reasoning
moves as the problem moves. You do not pick a framework and then glue.

The Causal State Machine is the bridge to the outside world. Its active state space
is inferred at runtime from the propagating effect, not enumerated at design time —
that is how it avoids the classical finite-state-machine limitation.

And then the Effect Ethos. Every action the CSM proposes gets intercepted and
evaluated against an immutable graph of computable norms, under a defeasible deontic
calculus — Olson and Forbus. The verdict is obligatory, impermissible, or optional
with a cost. If it is stopped, the reason is locked to the audit log alongside the
line of reasoning that produced it. That is what an auditor gets to read afterwards.

Underneath everything: the Context. Because spacetime is not built into the relation,
it has to live here — and counterfactuals run as parallel extra-contexts without
disturbing the primary one.

---

## Slide 07 — Uniform mathematics

One slide on the mathematics, as promised, and then straight back to the product.

Most scientific computing stacks make you bridge silos. One library for tensors,
another for geometric algebra, a third for topology, and glue code between them.
You end up spending more time on adapters than on the problem.

Here every layer implements the same categorical surface. A tensor is a functor.
A multivector is a monad. A manifold is a comonad. The propagating effect is a
monad. fmap, bind, extend, extract mean the same thing everywhere. That is done with
arity-five higher-kinded types on stable Rust, using a witness pattern — so it
monomorphizes, with no boxing and no virtual calls.

Two payoffs. First, a single bind chain can step from general relativity through
geometric algebra onto topology and finish in causal logic. The GRMHD example does
exactly that.

Second, precision becomes one type alias for the entire pipeline. Change one line
and the whole computation runs at f32, f64, or 106-bit — about thirty-two decimal
digits, several times faster than IEEE binary128.

That is the mathematics. Moving on.

---

## Slide 08 — One causal stack

This is the slide I would ask you to remember.

Read it top to bottom. Discover: the Causal Discovery Language is a typestate builder
DSL that takes raw observational data and finds the structure worth modelling. SURD
tells you which variables are uniquely or synergistically causal; MRMR does feature
selection; BRCD ranks the root cause of a regime shift.

Model: that structure becomes a Causaloid with an explicit Context.

Act: the Causal State Machine turns a verdict into a proposed action.

Govern: the Effect Ethos decides whether the action is permissible.

Run: the whole thing is thread-safe by construction, so deployment is embedding the
model in an ordinary async request handler.

Two things make this one stack rather than five tools with a pipeline between them.
First, every stage exchanges work through the same propagating effect — there is no
serialization boundary anywhere in that column. Second, the audit log accumulates
across all five, so the explanation you hand a regulator spans discovery through
deployment, not just the inference step.

And you are not obliged to use all of it. You can enter at any layer and leave at any
layer. Plenty of users only ever touch Model and Act.

---

## Slide 09 — Downstream: CFD

Two downstream projects, and I show them because they are the evidence that the
substrate generalizes beyond toy causality.

The first is counterfactual fluid dynamics. It shipped to crates.io this August and
has its own documentation site.

Three things it does. It couples disciplines — compressible flow, plasma chemistry,
navigation and control march inside one typed process rather than four codes
exchanging files. It forks the running simulation: march until a predicate fires,
then fork the paused state in constant time and fly every counterfactual branch
concurrently. That is the causal monad's intervene operator, applied to a fluid
solver. And it carries uncertainty through to a scored verdict.

On the right is why I am comfortable saying that out loud. Sod shock tube to an L1
of 0.027 against the exact Riemann solution. Ghia cavity. RAM-C II peak electron
density within about a factor of two of the flight anchor from an uncalibrated
finite-rate network. And the number that shows what forking buys: a mid-burn fork of
the marched, plume-coupled state departs a frozen-drag prediction by 139 metres per
second.

The framing at the bottom: Teschner's survey of the six biggest unsolved problems in
CFD, drawn from NASA's CFD Vision 2030 study. Four of the six informed this design.

One thing I want to flag for this council specifically: the roadmap page also lists
non-goals with reasons. Distributed execution and GPU acceleration are both deferred,
and we say why. An unstated decision reads as a gap.

---

## Slide 10 — Downstream: quantum

The second downstream project, and this is the one that tests the claim that the
axiom is genuinely spacetime-agnostic.

A classical causal model factorizes a joint distribution over its graph. A quantum
causal model — Lorenz, 2022 — does the same thing for a process operator, factorized
into per-node Choi–Jamiołkowski operators. So the conditional-probability tables
become quantum channels, and the model is their product.

The interesting engineering is the middle card. Not every product of operators is a
legal quantum causal model: factors whose Hilbert supports intersect have to pairwise
commute. We make that a freeze-time gate. When you freeze the graph, we embed each
intersecting pair onto its common support, form the commutator, and compare against
a depth-aware tolerance. The check is sound — it never accepts a non-commuting model
— and it may be incomplete. A failure names the exact offending pair and rolls the
graph back.

Second card: the projection lattice carries the Verdict, with Born-rule read-out. So
quantum logic is not sitting beside the causal engine; it is a verdict type inside it.

Third card, and this is a governance point as much as a technical one. There are two
modalities and we keep them apart: a verifiable path backed by Lean proofs, and a
physical-QPU seam behind a feature flag. We also deliberately do not offer nested
quantum causal models, because that is unestablished in the literature. We would
rather ship the flat model honestly than ship something that looks more general than
it is.

---

## Slide 11 — Case study

PLACEHOLDER — content to be written.

Suggested shape when you fill it in, so the slide matches the rest of the deck:
one line on what Service Radar is and where DeepCausality sits inside it; one line
on what the deployment does that a correlational monitor cannot; and one measured
number. Keep it to three claims.

Timing note: this slide is budgeted at roughly 90 seconds, which is the single
longest block in the deck. If it runs short on the day, the natural place to spend
the recovered time is the causal-stack slide.

---

## Slide 12 — Verification

This is where I would spend your scepticism, so let me be specific.

183 theorems machine-checked in Lean 4 against Mathlib, on a pinned toolchain. Zero
sorry — no unproved statement anywhere in the gated tree. Eleven Kani harnesses doing
bounded model checking on the core carrier.

The bridge matters more than the count. There is no tool that turns a Lean proof into
a Rust test. So each property is stated twice — once in Lean, once as a Rust witness —
and both carry the same identifier. CI fails if an identifier is missing either side.
That means the proofs cannot silently drift away from the code, which is the usual
failure mode of formalization efforts.

Now the right-hand panel, which is the part I actually want you to notice.

We publish the edges. The Float106 bit-exact error bounds are open — Lean proves the
real-field laws, the empirical bounds are Rust tests only. Octonions are outside the
proved layer because Mathlib does not carry them.

And the third one: in the quantum layer, the unconditional partial-trace preservation
property is proved false. Not unproven — false, with a witnessed counterexample. Only
a conditional boundary version holds. We found that by trying to prove it, and we
publish it as a headline result rather than quietly narrowing the claim.

That is the standard I would like the project judged against.

---

## Slide 13 — By the numbers

Quickly, because numbers are only context.

When I stood here in 2023, the repository held four crates, about twenty thousand
lines, and five hundred tests. Today: twenty-nine crates, four hundred and
thirty-eight thousand lines, and just under eleven thousand tests. Roughly a
twenty-one-fold increase on both lines and tests, which means test density held while
the codebase grew — that was deliberate.

A hundred and thirteen runnable examples across fourteen domains: avionics, physics,
medicine, materials, quantum, classical causality, and so on. Every example is a
build target in both build systems, and CI fails if a Cargo example is missing its
Bazel target, so the examples cannot rot.

Twenty-three of twenty-nine crates have no external runtime dependency at all. The
six that do are narrow and mostly optional. For a project aimed at regulated domains,
that supply-chain surface is a feature, not an accident.

And unsafe is forbidden repo-wide by a workspace lint. Three crates are exempt; each
exemption carries a written, irreducible justification, and two of them are marked
for removal when the compiler limitation behind them is fixed.

---

## Slide 14 — Governance and support

Briefly, on how the project is actually run.

Sandbox project at LF AI & Data since September 2023. MIT licensed across every
crate — no dual licensing, no contributor licence agreement friction. OpenSSF Best
Practices badge, a published security policy, Miri in CI for the crates where it is
meaningful, and coverage and lint gates on every pull request.

There is also a written policy for AI coding assistants in the repository, which is
not yet common and which I would be happy to discuss separately — several of you have
raised that question in other projects.

Community runs through Discord, GitHub Discussions, and the LF mailing lists.

Two sponsors. JetBrains provides all-product licences to core maintainers, and has
renewed that. And the Center for Dynamic Causality — which is the organization I
direct — contributes ongoing research and resources. I want to be transparent about
that relationship rather than have it discovered: the Center funds research that
lands in the project under the project's own MIT licence and open governance.

---

## Slide 15 — Close

Three places where help would compound, and then I will stop.

First, domain partners. The stack is strongest where a regulated domain needs an
audit trail — avionics, medicine, industrial control, finance. One partner with a
real decision boundary teaches us more than another benchmark does.

Second, adjacent LF projects. The propagating effect is a natural seam for anything
that already produces a verdict and then has to explain it. If your project has that
shape, the integration surface is genuinely small, and I would rather find that out
in a fifteen-minute call than in a design document.

Third, reviewers and provers. The Lean layer publishes its open edges deliberately.
Extending the proved surface — especially the quantum foundation, which we built from
first principles because Mathlib does not carry it — is well-scoped, self-contained
work for anyone who enjoys that.

Thank you. Happy to take questions, and happy to go deeper on any slide.

---
