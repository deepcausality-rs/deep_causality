---
title: Axiom
description: How DeepCausality's single axiom — every effect is derived from a prior effect by a causal function, m₂ = m₁ >>= f — unfolds into static and dynamic, Markovian, non-Euclidean, deterministic, and probabilistic causal reasoning.
sidebar:
  order: 1
---

DeepCausality is the reference implementation of the *Effect Propagation Process* (EPP), which rests on a single axiom: every effect is derived from a prior effect by a causal function embedded into a monadic composition denoted as `m₂ = m₁ >>= f`.

The EPP is an axiomatic, spacetime-agnostic foundation upon which domain-specific dynamic causal models are built and composed with one another, all derived from a single axiom. As we will see, this one axiom unlocks a number of unique properties:

- Static & dynamic causality processes
- Markovian & non-Markovian processes
- Euclidean & non-Euclidean spacetime
- Deterministic & probabilistic reasoning

## Unpacking the axiom

The equation is compact; its power rests on two deliberate symmetries, and on what those symmetries make expressible. The seven steps below illustrate the higher-order effects resulting from the axiom.

### 01 — The Axiom

Read `m₂ = m₁ >>= f` as follows: a causal function `f` derives a new effect `m₂` from a prior effect `m₁`. `bind` is that `>>=`. It takes the value held in `m₁`, applies `f`, and returns the next effect. No value is unwrapped by hand between steps.

```rust
use deep_causality::PropagatingEffect;

// m₂ = m₁ >>= f — bind feeds m₁'s output straight into the causal function f.
let m1 = PropagatingEffect::pure(10);
let m2 = m1.bind(|v, _state, _ctx| {
    PropagatingEffect::pure(v.into_value().unwrap_or_default() + 1)
});

assert_eq!(m2.into_value(), Some(11));
```

### 02 — Two Unifications

How can the output of one step serve as the input to the next without translation? Two deliberate symmetries make it possible.

Lucian Hardy, in his [foundational work](https://arxiv.org/abs/gr-qc/0509120), established that, absent a fixed spacetime, nothing distinguishes a cause from its effect, because the temporal order is missing. The [Causaloid](/concepts/causaloid/) therefore folds *cause and effect* into a single entity, which lets causal relations operate independently of any specific spacetime. This establishes the first symmetry: cause and effect as one entity.

The first symmetry, in turn, requires a carrier that propagates information.

The [propagating effect](/concepts/effect-propagation-process/) folds causal *input and output* into a single isomorphic carrier, so `m₁` and `m₂` share one type. This establishes the second symmetry: input and output as one entity.

```
cause  ⇄ effect    Causaloid · one entity
input  ⇄ output    propagating effect · one carrier

carrier: outcome (value-or-error) · state · context · log
```

The function `f` operates on a carrier of four fields.

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    outcome: Result<CausalEffect<Value>, Error>, // value-XOR-error, short-circuits on Err
    state:   State,                              // Markovian state
    context: Option<Context>,                    // the explicit world
    logs:    Log,                                // append-only audit trail
}
```

The carrier specializes into two subtypes. The propagating effect pins state and context to `()`, carrying only the outcome and log; the propagating process keeps all four channels. This split is deliberate. By excluding state and context, the propagating effect applies to the large class of problems that need neither, without adding complexity, while more demanding problems, which require state, context, or both, use the propagating process. The two compose cleanly because they derive from the same carrier, so stateless and stateful processes can be combined as the problem requires. This design is also what resolves Russell's contradiction.

A physical law is *time-symmetric* when it runs equally well forward and backward; Newton's equations, Maxwell's equations, and the Schrödinger equation do not record which way time flows. Film an elastic collision, play it in reverse, and the reversed motion still satisfies the same equations. Causation insists on the opposite: a cause precedes its effect, and no effect runs backward into its cause. In 1912 Bertrand Russell pressed the contradiction to its logical conclusion. If fundamental physics is time-symmetric and causation is not, then causation and modern physics are mutually exclusive.

The contradiction dissolves once two asymmetries, long conflated under one name, are distinguished. The first belongs to the *law*: whether an equation is invariant under time reversal. The second belongs to *propagation*: the direction in which one effect gives rise to the next. Classical causality conflated the two. In a structural causal model the directed assignment `Y := f(X)` carries both the mechanism and its direction in one object, so a time-symmetric law has no place; the formalism imposes a direction on a law that has none.

The central axiom, `m₂ = m₁ >>= f`, separates the asymmetry of direction from the asymmetry of time. The causal function `f` holds the law, which may be perfectly time-symmetric, while `>>=` holds the direction of propagation as a structural property of composition rather than of the law. A symmetric physical equation therefore composes inside a causal chain without contradiction, which is why a single algebra runs models of general relativity, electromagnetism, and quantum mechanics. Causality and modern physics no longer collide; in DeepCausality, they compose. The [Dynamic causality](/concepts/dynamic-causality/) page develops the argument from Russell through Whitehead, separating the notion of direction from the notion of time.

### 03 — Recovering the classical methods

Pearl's structural models, Granger's time-series test, and Rubin's potential outcomes are each the axiom with some of its freedom removed. DeepCausality generalizes the established methods of classical computational causality, and can therefore reconstruct any of them by specialization.

The base case is Pearl's biconditional, and it returns by lowering four independent parameters of the axiom to their floor. Setting the state and context channels to the unit type collapses the monad to a plain function. Restricting the value domain to two values, then the function to the identity on them, yields the biconditional itself. Reading the propagation direction as forward time restores temporal precedence.

The same reduction extends from the definition to the methods built upon it. Each established method is the axiom under a further restriction, and for the inference methods the operative step is a single mechanism, contextual alternation: clone the context, change one part of it, and re-evaluate the same causal law, which is what replaces Pearl's abduction. The dynamic Bayesian network is the exception, recovered structurally rather than by alternation.

- **Structural causal models (Pearl).** Recovered as the axiom under the unit context with bivalent values, the identity mechanism, and forward-time propagation; the three rungs of the ladder map to evaluation of a causal function in the factual context, the do-operator, and a counterfactual obtained by pinning the factual context, alternating it, and re-evaluating the unchanged causal graph.
- **Potential outcomes (Rubin).** The outcomes `Y(1)` and `Y(0)` are the same causal graph evaluated against a treatment context and a control context, and the unit-level effect is their difference. The fundamental problem of causal inference is a constraint on the physical measurement of a single unit.
- **Conditional average treatment effect.** The potential-outcomes construction with the conditioning set `X = x` represented as a static context, giving `τ(x) = E[Y(1) − Y(0) | X = x]` as the averaged difference of the two evaluations.
- **Granger causality.** Each series is represented as a temporal context of time-indexed values, and the test compares a predictor's error under the full context against its error under an alternate context that masks one series' history. A reduction in error indicates a predictive, time-ordered dependence.
- **Dynamic Bayesian networks.** Recovered structurally: one static causal graph is evaluated over a temporal context whose nodes are time slices, with each variable's conditional-probability table realized as a node's causal function and within- and cross-slice dependencies as hyperedges over the relevant slices.

Each recovery is implemented twice. The first carries context, state, and audit on the propagating effect and alternates on the carrier through the `Alternatable` mechanism. The second holds the causal law in a Causaloid and alternates it by cloning an external context and re-evaluating. Both produce identical results across all five methods, [as documented in the project repository](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples).

### 04 — Intervention across three channels

Causal reasoning rests on a distinction between *seeing* and *doing*. To observe that two quantities move together measures correlation, which cannot separate a cause from a coincidence. An *intervention* settles the matter. It forces a variable to a chosen value, cuts it off from whatever ordinarily sets it, and lets the chain run. Only this isolates one factor as the driver of another, and because the strength of a cause is the size of the change its intervention produces, the same operation measures how strongly causes act and ranks competing ones.

Judea Pearl gave this idea its formal machinery. His *do-operator*, written `do(x)`, together with the surrounding do-calculus, separated doing from seeing inside a rigorous algebra of causation, work for which he received the Turing Award. In the EPP that operator is a single method, `intervene`, applied at any point in a chain. The factual and counterfactual runs share one structure; a single call distinguishes them, and the substitution enters the log.

```rust
use deep_causality::{CausalFlow, PropagatingEffect};

// Chain: blood pressure → wall shear stress → arterial fatigue. The factual run
// and each counterfactual share one structure; a single .intervene distinguishes
// them and records do(...) in the audit log.
fn run_factual(baseline_bp: f64) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .map(shear_stress_stage)
        .map(fatigue_stage)
        .into_effect()
}

// do(BP = 120): a beta-blocker sets blood pressure before the chain runs.
fn run_medication(baseline_bp: f64, controlled_bp: f64) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .alternate_value(controlled_bp)
        .map(shear_stress_stage)
        .map(fatigue_stage)
        .into_effect()
}

// do(WSS = 8): a surgical clip sets wall shear stress mid-chain.
fn run_surgical(baseline_bp: f64, clipped_wss: f64) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .map(shear_stress_stage)
        .alternate_value(clipped_wss)
        .map(fatigue_stage)
        .into_effect()
}

// alternate_value is the value-level do(x); context and state use alternate_context / alternate_state.
```

In the example above, the factual run gives the outcome as it stands; each counterfactual gives the outcome that a different action would have produced. The causal effect of a treatment is the difference between the two arterial-fatigue summaries, and because the stages, the chain, and the model are identical across runs, that difference is attributable to the intervention alone. The three runs walk Pearl's ladder in order: the factual run observes, the `.alternate_value` call does, and the comparison of their outcomes is the counterfactual. Setting the medication result beside the surgical one then ranks the two treatments by effect, and the audit log records each `do(...)`, so every verdict traces to the substitution that produced it.

Pearl's operator reaches exactly one thing: the value of a single variable. A full counterfactual then needs more apparatus. The model graph is surgically altered, and the unobserved background is recovered by abduction, a posterior over hidden noise reconstructed from what was seen. The reach is one value at a time, the world is inferred rather than held, and the surgery occurs at the level of the model.

DeepCausality widens the operation and removes the complexity. The carrier holds the world explicitly in its context and the history in its state, so an intervention rewrites a channel on the running chain instead of operating on the model, and abduction is no longer required. Three channels are open, each with its own operator and audit marker: *value*, *context*, and *state*.

The two added channels change the category of question that can be asked. A *context* intervention substitutes the world the chain reasons against, so a [counterfactual](/concepts/counterfactuals/) can vary the entire environment rather than a single variable: what the model would have concluded under a different regime, or the outcome under treatment placed beside the outcome under control, with no model rebuilt. A *state* intervention forces the running memory of the process, so a counterfactual can vary the accumulated history: a simulator reset, a regime change that has already marked the counters, or a trajectory rewound to an earlier point. Pearl's `do(x)` asks what follows if one variable were set to `x`. The EPP also asks what follows if the world were different, and what follows if the history were different.

### 05 — From intervention to correction

Intervention so far has been analysis: a person poses a what-if question and reads the answer. The same operation becomes control once the chain performs it on itself. Because value, state, and context are externalized on the carrier, a step can read them mid-flow and branch on what it finds, and a branch that ends in an intervention is a correction. The chain watches its own trajectory and intervenes the moment an intervention becomes necessary.

The shape is the control loop, written in the DSL. Each tick advances the model, then branches on the carried value: inside the safe envelope the tick passes through untouched; outside it, the corrective arm fires `alternate_value`, and the next tick continues from the corrected value.

```rust
use deep_causality::CausalFlow;

// Lane keeping. Each tick advances the model; the chain then inspects its own
// value and corrects itself when the car drifts past the safe threshold.
CausalFlow::from(model::initial_process())
    .iterate_n(N_TICKS, |tick| {
        tick.bind(model::simulate_step).branch(
            // necessity condition detected?
            |offset| offset.abs() > cfg.anomaly_threshold,
            // if so, self-correct
            |hot|  hot.alternate_value_if(|_| true, |o| model::correction(o, &cfg)),
            // if not, no action
            |cold| cold,
        )
    })
    .into_process()
```

```
advance → inspect → correct if needed ↺ repeat each tick
```

Dynamics and control remain fully separate: the model knows nothing of correction, and the monitor knows nothing of the physics; the two meet only through the carried value. Every correction is logged like any other intervention, so a closed loop stays as auditable as the open run it replaces. As a direct consequence, simulating adverse conditions through a targeted intervention ahead of the correction becomes programmable. Where before we could only ask what would have happened if a value were different, we can now ask whether the fail-safe mechanism still holds under adverse conditions. We can also determine the range over which it is reliable in simulation and, just as important, where it stops.

### 06 — First-class uncertainty

Classical causal inference rests on statistics, yet it admits uncertainty only at the edges. It estimates a treatment effect and puts a confidence interval around it, while the quantities inside the model stay point estimates: a sensor reads `50.0`, a covariate is one number, a reading is either present or dropped. That breaks in two common cases. When the effect itself is probabilistic, a point estimate discards its shape. And when a value is missing, the missingness usually carries information: the patient who feels worse skips the daily report, and the sensor drops frames under the very vibration it watches. Impute a default for those gaps, and the bias runs the wrong way.

DeepCausality makes uncertainty a [first-class type](/concepts/uncertainty/), after Bornholt and colleagues. `Uncertain<T>` carries a value together with the distribution that produced it; arithmetic and comparison build a computation graph rather than collapsing to a number, and evaluation samples lazily to a requested confidence.

`MaybeUncertain<T>` goes one step further, factoring a reading into two questions held apart: a presence probability, carried as an `Uncertain<bool>`, and the distribution the value follows when present. A reading present eighty percent of the time and one present five percent of the time are different objects, and the difference survives: `is_some()` returns the presence distribution itself, not a flag. Given presence, the original distribution remains, ready to sample.

```rust
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

// One trial patient's daily pain-reduction reading: reported on ~70% of days,
// and, when reported, distributed Normal(4.0, 2.5). MaybeUncertain holds both.
let reading = MaybeUncertain::from_bernoulli_and_uncertain(0.7, Uncertain::normal(4.0, 2.5));

// Presence is itself uncertain: is_some() returns the Bernoulli(0.7), not a flag.
let present: Uncertain<bool> = reading.is_some();

// Commit to a value only when presence clears a confidence bar (here P > 0.5 at
// 95% confidence). Below it the gate fails and the chain short-circuits, instead
// of imputing a number. This is the per-patient step of the clinical-trial flow.
let score: Uncertain<f64> = reading.lift_to_uncertain(0.5, 0.95, 0.05, 1_000)?;
```

```
classical          → 50.0                       a point
Uncertain<T>       → Normal(50, 2.5)            a distribution
MaybeUncertain<T>  → P(present) × distribution  presence and distribution
```

The two channels propagate together. Add two such readings and the values combine as distributions, while the result counts as present only when both are; absence flows through the arithmetic with the right probability. A gate, `lift_to_uncertain`, commits to a plain `Uncertain<T>` only when presence clears a confidence bar. Below it the gate fails, and the chain short-circuits rather than inventing a number, the same way any failed step stops the propagation. The model can therefore distinguish *the value is zero* from *I never observed the value*, and decline to answer when the evidence for presence is too thin.

### 07 — Expressive range: non-Euclidean, relativistic, quantum

The foundational axiom, as implemented in DeepCausality, supports a broad range of expression:

- Markovian & non-Markovian processes
- Euclidean & non-Euclidean spacetime
- Static & dynamic causality
- Deterministic & probabilistic reasoning

The Markovian property enables stateful processes, which cover a broad variety of advanced engineering and physics domains, as demonstrated in the [avionics examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples).

Non-Euclidean spacetime representation is common in advanced physics, for example when modeling general relativity for satellite navigation. Combined with the Markovian property, stateful physics simulations across Euclidean and non-Euclidean regimes compose natively, as demonstrated in the [physics example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples).

Static and dynamic causal processes enable the combination of pre-existing background knowledge with current measurements, as demonstrated in the [medicine examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/medicine_examples).

Probabilistic reasoning paves the way for advanced applications in frontier fields that are inherently probabilistic, as demonstrated in the [quantum examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/quantum_counterfactual) and the [uncertain examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_uncertain_examples).

Each property alone already enables a category of advanced science and engineering that was previously difficult to address with computational causality. Taken together, multiple fields of advanced science and technology compose, as demonstrated in the [GRMHD example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/grmhd), which models magnetohydrodynamics under general relativity, or the [event horizon example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/event_horizon_probe), which simulates a space probe descending into a black hole while transitioning seamlessly through Newtonian and relativistic regimes to compute its trajectory. Inverting general relativity itself is even possible, as demonstrated in the [chronometric example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/chronometric_examples/gm_recovery), which recovers the GM constant to compute planetary mass from time dilation. Such complex, dynamic causal processes occur in countless domains, and accordingly the DeepCausality project provides [over one hundred code examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples) to illustrate dynamic causality across them.

A single axiom gives rise to nearly limitless possibilities. The future is now.

## Where to look next

[Dynamic causality](/concepts/dynamic-causality/) develops the framing the axiom sits inside. [Causal Monad](/concepts/causal-monad/) is the `pure`/`bind` algebra the axiom names, and [Effect Propagation Process](/concepts/effect-propagation-process/) is the five-field carrier it operates over. [Counterfactuals](/concepts/counterfactuals/) and [Uncertainty](/concepts/uncertainty/) cover the intervention and probabilistic channels in depth.
</content>
</invoke>
