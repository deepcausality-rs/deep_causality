---
title: "The Seven Stages of Causal Inference"
description: "A first-principles ascent through the canon of causal inference (Granger, DBN, RCM, CATE, SCM), then past it into counterfactual rewinding and closed-loop correction. One monadic axiom subsumes them all by separating the direction of causal propagation from the order of time."
date: 2026-06-19
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## The argument in one sentence

The history of causal inference is a history of separate tools, each answering one question and falling silent on the next; DeepCausality answers all of them, and one more, with a single axiom.

That is a strong claim. The rest of this post earns it.

Causal inference did not arrive as one theory. It accreted, method by method, across half a century and several disciplines. Granger gave econometrics a test for predictive precedence in 1969. Bayesian networks gave AI a calculus for belief over time. Neyman and Rubin gave statistics the potential-outcomes contrast. Pearl gave everyone the structural language and the *do*-operator. Each method is correct. Each is also an island: its own mathematics, its own software, its own notion of what "cause" even means. A team that masters one rarely reuses the machinery for the next.

DeepCausality is the reference implementation of the **Effect Propagation Process (EPP)**, whose single axiom is `m₂ = m₁ >>= f`: effect propagation as monadic dependency, with no background spacetime assumed. The foundational move is to separate the causal law from the world state. The law is a function. The state is an explicit, queryable value. Once those two are apart, every classical method becomes the *same* computation evaluated against a different world, and two capabilities that no classical method reaches at all (counterfactual rewinding and closed-loop correction) become two more method calls on the same object.

Beneath that lies a second separation the monograph treats as its core contribution, and it is the one that lets the same notion of cause reach into modern physics: classical causality fused the *direction* of causal propagation with the *order* of time, and the axiom pulls the two apart. The argument below builds to that point.

This post climbs that ascent in seven stages. The first five replicate the classical canon. The sixth steps onto the top rung of Pearl's ladder without paying its usual price. The seventh steps off the ladder entirely, into action. Every stage carries both its theory and its running code.

| Stage | Method | Question it answers | What its predecessor could not do |
|---|---|---|---|
| 1 | Granger | Does the past of X improve prediction of the future of Y? | — (the floor) |
| 2 | DBN | What should I believe about a hidden state as evidence arrives over time? | Carry and update latent belief |
| 3 | RCM | What is the effect of treatment on this unit: `Y(1) − Y(0)`? | Contrast an action against its absence |
| 4 | CATE | For whom is the effect large, and for whom small? | Treat the effect as a function, not a scalar |
| 5 | SCM | All three rungs at once, in one structural language. | Unify association, intervention, counterfactual |
| 6 | Counterfactual | Given the world as it was, what would have happened otherwise? | Hold the actual world fixed without inferring it |
| 7 | Correction | Detect the drift and prevent the failure, in the loop. | Act on the world, not only reason about it |

## The axiom, and why classical causality is a special case of it

Detaching causality from spacetime forces a hard question, and the EPP monograph states it plainly: what is the minimum necessary and sufficient condition for a relation to be causal, once temporal precedence is no longer available to define it? For centuries the answer was temporal precedence plus counterfactual dependence. The cause comes before the effect, and the effect would not have occurred without it. Remove the first half, as a spacetime-agnostic foundation must, and you have to find what carries the rest.

The monograph finds it by decomposing the classical biconditional. Pearl's definition (if A then B, and if not A then not B) rests on three conditions that are independent of one another:

1. **Process.** Causality describes change in a system. A process carries state, the context it runs in, and the conditions under which it succeeds or fails.
2. **Determination.** Causality determines effects. We know X causes E because E follows when X is present and fails to follow when X is absent; the effect is what the relation pins down.
3. **Propagation.** Causality has a direction. One effect gives rise to the next.

Relabel "cause" and "effect" with neutral terms and the biconditional loses nothing: if effect `E₁` happens, its effect propagates to `E₂`; if `E₁` does not, it does not. X was never essentially a cause. It was a prior effect that propagates further.

Now ask what object satisfies all three conditions at once. A pure function `E₂ = f(E₁)` captures determination, but a pure function is timeless and stateless; it fails the process condition. The structure that carries determination *and* process *and* direction is the monad. Its `bind` operation is unidirectional by construction, which is propagation. The context it wraps around a value carries state and more, which is process. Composition preserves the input-output relation, which is determination. That is the derivation of the axiom, and it runs from the classical definition outward rather than being imposed over it:

`m₂ = m₁ >>= f`

A causal function `f` derives a new propagating effect `m₂` from a prior one `m₁`, and `bind` (`>>=`) carries value, state, and context through the composition. The carrier has five fields: a `value`, a Markov `state`, an explicit `context`, an `error` that short-circuits the chain, and an append-only `log`. The stateless `PropagatingEffect<T>` uses value, error, and log; the stateful `PropagatingProcess<T, S, C>` uses all five. Two operators substitute a channel mid-flight: `alternate_context(c)` swaps the world the chain reads, and `intervene(v)` (the causal-inference name for `alternate_value`) substitutes the carried value. Each appends its own marker to the log (`!!ContextAlternation!!`, `!!Intervention!!`), and neither can repair an already-errored chain.

### Two asymmetries, finally separated

The decomposition's third condition, propagation, is where the framework's central claim lives. Classical causality fused two different asymmetries under the single phrase "cause precedes effect":

- The asymmetry of **propagation**: the direction in which one effect gives rise to the next. It is structural and always present. An effect chain runs one way and never the other.
- The asymmetry of **time**: whether the law relating the two is invariant under time reversal. Newton's equations, Maxwell's equations, and the Schrödinger equation are time-symmetric. Film an elastic collision, play it backward, and the reversed motion satisfies the same law.

A structural causal model writes both into one object. The assignment `Y := f(X)` carries the mechanism and its direction together, so a time-symmetric law has nowhere to sit; the formalism stamps a temporal direction onto a law that has none. This is Russell's 1912 objection in its exact form: if fundamental physics is time-symmetric and causation is not, the two cannot both be fundamental.

The axiom takes the two apart and gives each its own home. `>>=` carries the propagation direction, as a structural property of composition. `f` carries the law, and the law's temporal symmetry is now free: asymmetric for an econometric forecast, perfectly symmetric for Maxwell's equations, either way embedded inside the same `bind`. The carrier is always unidirectional. The temporal order is whatever the law inside `f` says it is, including none. Russell's contradiction does not get solved so much as dissolved: it was never between causation and physics, only between physics and a definition that hard-coded a direction into the law.

This separation is the load-bearing wall under every advanced example. You cannot evaluate a time-symmetric physical equation inside a causal process unless the process draws its directionality from somewhere other than the law, and the EPP draws it from `>>=`. One algebra therefore runs a forward-time Granger test and a time-symmetric Schrödinger evolution without changing what it means by cause. Stage six is the in-scope instance: a quantum history that the chain rewinds while still composing forward. The general-relativistic and electromagnetic models elsewhere in the project rest on the same wall.

### The axiom fixes the form; autonomy supplies the content

One qualification the monograph insists on, and it matters. The axiom fixes the *form* of a causal process and nothing more. `bind` and an unconstrained `f` are available to any monadic program, so monadic composition is necessary for causality but not sufficient. What separates a causal function from an arbitrary one is **mechanism autonomy**: the function reads only the value, state, and context threaded to it (informational locality), and it holds invariant when other mechanisms in the model are perturbed (stability). This is the same modularity Pearl's structural equations assume when each is taken to be an autonomous mechanism rather than a curve fit. Locality is enforced by the `bind` interface together with externalizing all state onto the carrier; stability is a postulate the modeller makes, as it is in every causal framework. A causal relation, stated in full, is a monadic composition of *autonomous* causal functions. The adjective carries the causal content; the composition carries the process form. Neither alone is causality.

### The reconstruction: classical causality at four floors

With the form and the content fixed, the classical definition returns as a specialization, in two reductions.

The first reduction collapses the monad to a pure function. Set every context parameter, state included, to the unit type. Under that unit context the monad is isomorphic to the identity monad, and `m₂ = m₁ >>= f` reduces to `E₂ = f(E₁)`.

The second reduction collapses that function to Pearl's biconditional through three restrictions:

1. **Bivalent effects.** Restrict the domain and codomain of `f` to two values, `{0, 1}`.
2. **Biconditional dependence.** Require `f(1) = 1` and `f(0) = 0`, which on a two-valued domain forces `f` to be the identity. The negation case is "X causes the absence of E," recovered by reading the outcome as `¬E`.
3. **Temporal precedence.** Interpret the propagation direction as forward time.

At that floor you have Pearl's definition exactly, recovered without remainder: four independent restrictions, unit context, bivalence, identity, forward time. The significance runs the other way. Each restriction you decline to apply is a generalization classical causality cannot express. Keep a non-trivial state channel and causality carries memory. Keep a real-valued domain and causes become graded rather than present-or-absent. Keep an arbitrary mechanism and `f` may be a differential equation or a neural network. Decline the forward-time floor and the two asymmetries stay apart: propagation keeps its direction while the law keeps its own temporal character, including none. That is the regime the earlier subsection described, and it is where the physics examples live.

Intervention is the same story. Pearl's `do(x)` returns as one operator on the carrier. Under the unit context with no prior error, `intervene(v)` forces the value to `v`, leaves state and context untouched, and the following binds re-evaluate the unchanged mechanisms on `v`. That is the do-operator's surgery exactly, with no model rebuilt. Outside that floor the same operator does more: it preserves a prior error, so it cannot paper over a failed chain, and it records itself in the log, so the action stays auditable. Intervention is therefore *derived* from the carrier, not a primitive of the axiom, which is why causes that cannot be manipulated, the moon on the tides or the early universe on the microwave background, remain causal under this account.

### The classical methods as restrictions

The five classical methods follow the same pattern. Each is the axiom under a further restriction, and four of the five are recovered by one mechanism: **contextual alternation**, which clones the context, changes one part of it, and re-evaluates the same mechanism graph against it. Because the alternate contexts are independent, their evaluations are independent too, which makes counterfactual reasoning embarrassingly parallel.

| Method | The axiom, restricted to |
|---|---|
| SCM (Pearl) | unit context, bivalent values, identity mechanism, forward time; the three rungs become factual evaluation, `intervene`, and pin-alternate-re-evaluate |
| RCM (Rubin) | one mechanism graph evaluated against a treatment context and a control context; the effect is their difference |
| CATE | RCM with the conditioning set `X = x` carried as a static context; `τ(x)` is the difference of the two evaluations |
| Granger | each series carried as a temporal context of time slices; compare a predictor's error under the full context against a history-masked one |
| DBN | one static mechanism graph evaluated over a temporal context whose nodes are time slices; recovered structurally, not by alternation |

One caveat the monograph is careful about, and so am I. These mappings operationalize causal *inference*; they presuppose a mechanism that has already been discovered and validated. The EPP does not learn the graph from raw data, and existing discovery tools remain indispensable for that first step. What follows demonstrates the inference, not the discovery.

The first five stages below run this table as code. The last two restore freedoms the classical definition discarded: stage six keeps the explicit history that turns Pearl's abduction into a pin rather than an inference, and stage seven keeps the running loop that turns a single intervention into continuous control. The [Axiom concept page](https://docs.deepcausality.com/concepts/axiom/) and the EPP monograph carry the full reduction; what follows is its demonstration.

## Stage 1. Granger: predictive precedence

Start at the floor. Clive Granger's 1969 test, which earned a Nobel in 2003, makes the most modest claim in the field: X *Granger-causes* Y if the past of X improves the prediction of Y beyond what Y's own past already provides. This is predictability, not mechanism. Granger himself was careful about the word. The test cannot tell a true cause from a shared driver, and it has been criticised on exactly that ground for fifty years. It is the right place to begin precisely because it is the weakest thing the field is willing to call "causality."

In the EPP the test has a clean shape. The two competing worlds (with the predictor's history, and without it) are two values of the Context. One predictor runs against both.

```rust
let factual         = factual_series();            // shipping + oil history
let counterfactual  = without_oil(&factual);       // same shipping, oil removed

let factual_pred = run(factual.clone()).value;     // predict with oil
let counter_pred = start(factual)
    .alternate_context(counterfactual)             // swap to the no-oil world
    .bind(predict_shipping)
    .value;                                         // predict without oil

// Oil Granger-causes shipping iff including its history lowers prediction error.
if err_factual < err_counter { /* yes */ }
```

Whichever world predicts the next quarter with lower error wins. The judgement is a comparison of two errors, and the only operator that differs between the two runs is one `alternate_context`. Granger's whole apparatus reduces to: hold the model fixed, vary the world, compare. Remember that sentence. It is the entire post in miniature.

## Stage 2. DBN: probabilistic dynamics

Granger predicts a number. It does not maintain a belief. The Dynamic Bayesian Network does: it carries a probability distribution over a hidden state and updates that belief as evidence arrives, tick by tick. This is the machinery behind every Kalman filter, every hidden Markov model, every robot that estimates where it is from noisy sensors. The new capability over Stage 1 is memory with structure. The chain now has a `State` that persists and a `Context` of conditional probability tables that govern transitions.

The DBN is the one method recovered structurally rather than by alternation: a single static mechanism graph evaluated over a temporal context whose nodes are time slices, with each variable's conditional-probability table realised as a node's causal function. The regime change below is an extra that the externalized context makes available for free, not part of the base recovery.

The canonical demonstration is the Umbrella World over ten days. What makes it a causality demonstration rather than a filtering exercise is the regime change. Halfway through, the climate itself shifts, and the chain must keep its accumulated state across the discontinuity.

```rust
// Baseline ten-day run: dry-leaning climate throughout.  umbrellas_carried = 0
// Regime change: five baseline days, then swap the climate's CPTs mid-stream.
let switched = baseline_five_days(process)
    .alternate_context(monsoon_climate())   // the world changes underneath the chain
    .bind(day_six)
    .bind(day_seven) /* ... */;             // umbrellas_carried = 5
```

The Markov state threads through the switch untouched. The day counter keeps counting `1..10` across the boundary; the umbrella tally accumulates correctly on both sides. The log carries exactly one `!!ContextAlternation!!` entry pinpointing the day the world changed. A classical DBN would force you to rebuild the network to change its transition model. Here the transition model lives in the Context, so changing it is the same `alternate_context` from Stage 1, applied to a richer world.

## Stage 3. RCM: the interventional contrast

The first two stages are observational. They watch the world and predict it. They never ask what would happen under an action. Stage 3 does. The Neyman-Rubin potential-outcomes model defines the effect of a treatment on a single unit as a contrast between two outcomes: the outcome under treatment, `Y(1)`, and the outcome under control, `Y(0)`. The effect is their difference. The deep difficulty, which the framework names the fundamental problem of causal inference, is that you only ever observe one of the two for any given unit. The other is counterfactual by definition.

The EPP makes the counterfactual outcome a second evaluation against an alternated world, not a quantity you must estimate around.

```rust
// Factual: this patient is treated.  Run the chain against the treatment world.
let treated = start(treatment_ctx.clone())
    .bind(apply_drug_effect)
    .bind(compute_final_bp);
let y1 = treated.value;                 // Y(1)

// Counterfactual: the same patient, untreated. Swap the world before the binds run.
let control = start(treatment_ctx)
    .alternate_context(control_ctx)     // do(T = 0)
    .bind(apply_drug_effect)
    .bind(compute_final_bp);
let y0 = control.value;                 // Y(0)

let ite = y1 - y0;                      // Individual Treatment Effect
```

`Y(1)` and `Y(0)` come from one chain definition run against two worlds. The estimand is their difference, written as code. There is no separate potential-outcomes calculus and no assumption smuggled in to recover the missing outcome: the second outcome is computed directly, because the world it depends on is an explicit value you are free to substitute.

## Stage 4. CATE: heterogeneity of effect

Stage 3 produces one number for one unit, and the textbook average treatment effect collapses a whole population to a single scalar. That scalar lies to you whenever the treatment helps one group and harms another. The Conditional Average Treatment Effect refuses the collapse. It treats the effect as a function of who you are: `CATE(S) = E[ Y(1) − Y(0) | X ∈ S ]`. This is the quantity that precision medicine, targeted policy, and uplift modelling actually need, because the decision is never "treat everyone" but "treat whom."

The implementation is Stage 3 evaluated over a subgroup and averaged. Nothing new in the machinery; a sharper question asked of it.

```rust
let subgroup: Vec<&PatientContext> = population
    .iter()
    .filter(|p| p.age > AGE_THRESHOLD)   // the "for whom"
    .collect();

let ites: Vec<f64> = subgroup
    .iter()
    .map(|p| individual_treatment_effect(p))   // Stage 3, per patient
    .collect();

let cate = ites.iter().sum::<f64>() / ites.len() as f64;
```

Each `individual_treatment_effect` runs the same two-world contrast from Stage 3. The CATE is their mean over the subgroup. The effect is no longer a number reported about a population; it is a function you evaluate wherever a decision has to be made. This operationalizes the inference and nothing more: it presupposes that the per-patient mechanism was already discovered and validated, which is the separate first step the EPP leaves to existing causal-discovery tools.

## Stage 5. SCM: the full ladder

The first four stages each answer one question. Pearl's Structural Causal Model answers all of them in one language, and this is its enduring achievement. The SCM expresses the world as a graph of variables tied by structural equations, and it organises every causal question onto three rungs: **association** (seeing), **intervention** (doing), and **counterfactual** (imagining otherwise). The *do*-operator severs a variable from its causes and sets it by fiat. The ladder is the map of the whole territory the first four stages explored piecemeal.

Classically the third rung is expensive. Pearl's counterfactual procedure runs **abduction → action → prediction**, and the abduction step (inferring the posterior over hidden noise that explains the factual data) is the documented bottleneck; for general non-linear and neural SCMs it is often intractable, which is why a literature ([Pawlowski, Castro, Glocker, NeurIPS 2020](https://proceedings.neurips.cc/paper/2020/file/0987b8b338d6c90bbedd8631bc499221-Paper.pdf)) exists to work around it.

On the monad each rung is one operator. The smoking-tar-cancer chain shows all three:

| Rung | Operator | Channel |
|---|---|---|
| 1. Association | a plain `bind` chain | none |
| 2. Intervention | `.intervene(0.0)` mid-chain, applying `do(Tar := 0.0)` | value |
| 3. Counterfactual | `.alternate_context(other_world)` at the seed | context |

```rust
// Rung 2: sever Smoking -> Tar before stage 2 reads it.
let intervened = start(smoker_ctx)
    .bind(smoking_to_tar)
    .intervene(0.0)            // do(Tar := 0.0): the tar is gone
    .bind(tar_to_cancer);      // cancer risk drops, despite high nicotine
```

The intervention cuts the link before the downstream stage reads it; cancer risk falls to false even though the patient still has high nicotine. The counterfactual swaps to a world where nicotine is low but the tar already accumulated; risk stays high, because the tar is still in the lungs. Same chain, two operators, two log markers. Crucially, no abductive inference appears anywhere. The reason is structural: "the world as it actually was" is the Context itself, a value you already hold, not a posterior you must infer. That single fact is what Stage 6 makes its centrepiece.

## Stage 6. Counterfactual: the abduction step, collapsed

Here is the top rung, and the place where the EPP's foundation pays its largest dividend. The Causal Hierarchy Theorem ([Bareinboim, Correa, Ibeling, Icard, 2020](https://causalai.net/r60.pdf)) proves that counterfactuals carry strictly more information than interventions, which carry strictly more than associations. You cannot, in general, climb the ladder from below; the higher rung needs information the lower one discards. In the EPP that extra information has a home: the explicit Context and the explicit history threaded through `State`. The counterfactual question becomes mechanical. Hold the recorded world fixed, change one thing in it, re-run the same law.

Pearl's three counterfactual steps survive one-to-one. Abduction becomes context pinning, action becomes contextual alternation, and prediction becomes re-evaluation of the unchanged model against the altered context. What changes is the cost of the first step. Abduction stops being the inference of a posterior over hidden noise and becomes the act of holding a world that is already written down.

The sharpest demonstration is quantum. A qubit in superposition is among the most fragile objects an engineer can be asked to protect; a stray interaction flips it, and the computation dies. Quantum error correction is therefore not a convenience but a precondition for the entire field. The "dead qubit" example treats correction as counterfactual debugging: thread the full history of quantum states through the chain, detect the bit-flip, then rewind history and ask what the state would have been had the flip never happened.

```rust
// The quantum history rides the State channel; each stage may rewrite it.
let result = CausalFlow::process(history)
    .bind(stage_apply_gate)        // t=1: a drift flips |0> toward |1>
    .bind(stage_measure_syndrome)  // t=2: P(|1>) > 0.9  -> error detected
    .bind(stage_correct)           // t=3: rewind history, re-apply |0>
    .into_process();
```

The correction stage is the counterfactual made literal:

```rust
if error_detected {
    hist.states.pop();                       // rewind: undo the corrupted step
    let corrected = HilbertState::new(        // re-apply the world that should have been
        vec![Complex::new(0.99, 0.0), Complex::new(0.01, 0.0)],
        Metric::Euclidean(1),
    ).unwrap();
    hist.states.push(corrected);             // |0> restored
}
```

The run ends with `P(|0>) = 0.98`: the qubit is alive. What deserves attention is not the arithmetic but the shape. The framework did not estimate a posterior over hidden noise to recover the pre-error state. It held the actual recorded history and edited it. Abduction did not vanish; it collapsed to a pin, because the history was never thrown away. This is rung 3 of Pearl's ladder, reached by a `pop` and a `push` on a state the chain was already carrying.

Notice the two directions at work, because this is the foundation's central separation made concrete. The chain composes forward (gate, then measurement, then correction) while the modeled history runs backward (pop the corrupted step, restore the prior state). Propagation direction and temporal order are doing separate jobs here, exactly as the axiom pulled them apart. A formalism that fused the two could not express a forward-composing process that rewinds the world it reasons about.

The same retrospective move generalises far beyond physics. The [counterfactual examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_counterfactual_examples) apply it to a fluid network's N-k contingency analysis (one trigger collapses the grid; another is absorbed), to epilepsy surgery screening (`do(connectome = resected_at_R)` for each candidate region, ranking the hub-node resection as most curative), and to treatment-effect estimation as a difference of two interventional chains. In each, the artefact that matters in review (which region was resected, which value was substituted, when) is in the log, because the counterfactual is an operation on an audited object rather than a separate analysis run on the side.

## Stage 7. Correction: causality that acts

Every stage so far reasons about the world. None of them changes it. Stage 7 takes the step that no classical inference method takes: it closes the loop. A monitor runs *inside* the chain, watches the model's own state each tick, and when the trajectory drifts toward failure it intervenes to prevent the failure rather than measure it afterward.

This is the difference between asking "what would have happened if the car had drifted?" and keeping the car in its lane. The retrospective counterfactual of Stage 6 becomes a real-time control action. Because `PropagatingProcess` threads `state` and `context` through every `bind`, the controller is not a separate process bolted onto an inference engine. The controller is the model, reading its own understanding of the world.

The lane-keeping example shows the entire mechanism. The open loop is a bare `bind` chain. The closed loop adds a monitor that decides, each tick, whether to correct.

```rust
for _ in 0..N_TICKS {
    process = process.bind(model::simulate_step);

    let current = match &process.value { EffectValue::Value(v) => *v, _ => continue };
    let cfg = process.context.clone().expect("LaneConfig present");
    if current.abs() > cfg.anomaly_threshold {
        let corrected = model::correction(current, &cfg);  // same model, computes the fix
        process.state.correction_count += 1;
        process = process.intervene(corrected);            // act: next tick continues corrected
    }
}
```

Run open loop, the vehicle leaves the road at tick 24. Run closed loop, the same vehicle under the same crosswind stays in its lane indefinitely. The diff between catastrophe and safety is the four-line monitor block. The same shape, with a different anomaly predicate and a different correction function, holds a blood-glucose pump below the ketoacidosis line, inserts a diver's decompression stops before the DCS threshold, reroutes a network onto its standby switch on detected packet loss, and clamps a volumetric DDoS attack within one second of a five-sigma detection. Five domains, one architecture, one operator: `intervene`.

Two properties make this fit for systems where failure is not an option. The detector reads the same Context the forward steps read, so it cannot drift out of sync with the model it guards. And every correction lands in the same `EffectLog` as the rest of the run, so a post-incident review and a regulator audit read the same record a debugger would. Corrective autonomy that is auditable by construction is the precondition for deploying any of this near a patient, an aircraft, or a power grid.

## What the seven stages share

Now the deduction. Read the seven stages back to back and the surprise is how little of the machinery changed.

Granger varied the Context and compared two prediction errors. DBN varied the Context mid-stream and threaded state across the change. RCM varied the Context to get `Y(0)` against `Y(1)`. CATE ran RCM's contrast over a subgroup. SCM placed all three rungs on one chain with `intervene` and `alternate_context`. The counterfactual rewound an explicit history. The correction loop computed a fix from the model and fed it back with `intervene`. One carrier. One `bind`. Two substitution operators. That is the whole toolkit.

This is the table from the top of the post, now executed. The classical canon needed five separate mathematical apparatuses to cover the first five stages, and a practitioner who learned one gained little leverage on the next. The EPP needs one, because of two foundational separations. The first splits the causal law from the world state, which makes the state an explicit value you can hold, query, alternate, and rewind. The second, and deeper, splits the direction of propagation from the order of time: direction lives in `>>=`, and the law's temporal symmetry stays free in `f`. The first separation buys counterfactuals without an inference step. The second lets one notion of cause run a forward-time forecast and a time-symmetric physical law alike. Everything above follows from those two decisions the way theorems follow from an axiom. The classical definition itself is the floor case, recovered by lowering four parameters; counterfactuals with abduction collapsed to a pin, intervention without graph surgery, control without a separate controller are each a freedom restored, not a feature bolted on.

## The conclusion this earns

The seven stages are usually taught as seven subjects. They are one subject, seen at seven settings of the same instrument. A framework that holds the world as data rather than as implicit noise can climb the entire ladder of causation with three operators, and then take the step the ladder never describes: it can act, in the loop, on the world it reasons about, and leave an audit trail a regulator can read.

That last stage is where the gravity of this work lives. Reasoning about counterfactuals is an intellectual achievement. Preventing a failure before it happens, with a record of why, in a system where a wrong answer costs a life, is an engineering one. The same `PropagatingProcess` does both, because both are the same `bind`. That is the claim the first sentence made, and the seven stages have now paid for it.

## Getting started

Run the ascent yourself:

```bash
git clone https://github.com/deepcausality-rs/deep_causality.git && cd deep_causality

# Stages 1-5: the classical canon, twice each (causaloid and monad forms)
cargo run -p classical_causality_examples --example granger_via_monad
cargo run -p classical_causality_examples --example dbn_via_monad
cargo run -p classical_causality_examples --example rcm_via_monad
cargo run -p classical_causality_examples --example cate_via_monad
cargo run -p classical_causality_examples --example scm_via_monad

# Stage 6: counterfactual rewinding (the quantum dead qubit, and five more)
cargo run -p quantum_examples --example quantum_counterfactual
cargo run -p causal_counterfactual_examples --example counterfactual_cascade_failure

# Stage 7: closed-loop correction
cargo run -p causal_correction_examples --example corrective_lane_keeping
```

- The [Axiom](https://docs.deepcausality.com/concepts/axiom/) and [Premise](https://docs.deepcausality.com/overview/premise/) pages give the foundational argument and the full reduction of the classical methods.
- The [classical-examples README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) carries the full side-by-side comparison and a practitioner decision guide.
- The [Counterfactuals](https://docs.deepcausality.com/concepts/counterfactuals/) and [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) concept pages give the long-form algebra.
- The companion post, [Counterfactuals via the Causal Monad](https://www.deepcausality.com/blog/counterfactuals-via-the-causal-monad/), develops Stages 6 and 7 in depth.
- Join the [community](https://www.deepcausality.com/community/) or the [Discord server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
</content>
