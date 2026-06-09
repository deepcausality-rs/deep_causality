---
title: "The Causal Flow Language: The Monad's Power, Without Its Complexity"
description: "The Effect Propagation Process gave causality one axiom and one carrier. The causal monad realized it in software, and it worked, but it proved too complex to write by hand. The Causal Flow Language reaches the monad's full expressiveness without its complexity, and the Arrow algebra is the foundation that made that possible."
date: 2026-06-09
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## A new foundation

DeepCausality implements the *Effect Propagation Process* (EPP), a single axiomatic foundation for dynamic causality. The axiom is one line, `m₂ = m₁ >>= f`: every effect is derived from a prior effect by a causal function composed in a monadic context. The EPP folds cause and effect into one entity, the [Causaloid](https://docs.deepcausality.com/concepts/causaloid/), and input and output into one carrier, the [propagating effect](https://docs.deepcausality.com/concepts/effect-propagation-process/). It separates the asymmetry of causal *direction* from the asymmetry of *time*, which is how a time-symmetric physical law composes inside a causal chain alongside time-asymmetric processes.

The software realization of that axiom is the [causal monad](https://docs.deepcausality.com/concepts/causal-monad/). 
The EPP derives the causal monad from a decomposition of causality and shows that the classical methods (Pearl's structural models, Rubin's potential outcomes, Granger causality, the conditional average treatment effect, dynamic Bayesian networks) are each the axiom under a restriction. Pearl's `do`-operator becomes a single method. Counterfactuals need no abduction step, because the world is held explicitly in the propagating effect. 

And yet, written by hand, DeepCausality is hard to use. The monadic composition sits awkwardly in Rust, whose type system forces the receiving type to be separated from the algebra, so the type signatures grow complex. The arity-five monad, with its embedded value, state, context, error, and log, is especially awkward inside closures, precisely because of those embedded values. The original workaround split the causal monad into two type aliases: the propagating effect, which reduces the embedded types to three, and the propagating process, which still holds all five. That eased some of the pain, but it did not remove the underlying complexity.

## The friction 

The monad's strength is that one carrier threads five channels at once: the value, the state, the explicit context, the error short-circuit, and the append-only audit log. The cost is that a developer must address all five through a low-level interface. A stage seeds the chain with `pure`, matches the `EffectValue` wrapper inside every `bind` closure, threads state and context by hand, short-circuits the error channel manually, and unwraps the result at the end:

```rust
let pipeline = PropagatingEffect::pure(())
    .bind(move |_, _, _| stage1(inputs))
    .bind(move |s1, _, _| match s1.into_value() {
        Some(s) => stage2(s),
        None => error_effect("stage 1 produced no value"),
    });

match pipeline.value.into_value() {
    Some(report) => print(report),
    None => eprintln!("failed: {:?}", pipeline.error),
}
```

The causal logic is just three function calls, and yet the API surface makes it needlessly complex, even with the simplified propagating-effect alias that carries only three embedded values.

The friction grew worse the moment a pipeline needed control flow. The monad composes one way: a linear chain of binds. However, it has no loop and no branch of its own. So every time-stepped model (lane keeping, collision avoidance, and so on) reached outside the monad and hand-rolled the loop:

```rust
let mut process = model::initial_process();
for _ in 0..N_TICKS {
    process = process.bind(model::simulate_step);   // advance one tick
    // read the value out of process.value, test it, rebind on a correction ...
}
```

That loop sits *outside* the causal algebra of the EPP. The error short-circuit and the log accumulation, automatic inside a bind, become manual again across ticks. The conditional correction at the heart of closed-loop control is a plain `if` around a `let mut` rebind. Each property the monad guarantees within a single step is surrendered the moment the step runs in a loop.
The question that produced the Causal Flow Language eventually became:

> Can the full expressiveness of the causal monad be reached without its complexity?

## The answer is more algebra

Sequential composition, iteration, and choice are the combinator algebra of an **Arrow**. In his 2000 paper *Generalising Monads to Arrows*, John Hughes introduced the abstraction precisely to name the wiring a bare monad leaves implicit: `arr` to lift a function, `>>>` to compose two arrows, and the family that follows. A monad gives composition through `bind`, but its surface offers no loop, no branch, and no pipeline you can build once and run on many inputs. Those are combinators, and combinators are what an Arrow supplies. The Kleisli arrows of any monad form an Arrow. Therefore, in principle, the causal monad already contains an arrow. At least, that is what the theory says.

The value-level Arrow algebra in `deep_causality_haft` (`Id`, `Lift`, `Compose`, the strength operators, and the `EndoArrow` iteration) was added as the foundation for the control flow in the Causal Flow Language. The Causal Arrow then realizes that algebra over the causal monad: its Kleisli arrow, where every combinator lowers to `bind` and threads all five channels. 

The arrow algebra enables:

- **Sequential composition** makes a stage a reusable value. 
- **Endomorphism iteration** makes the loop a first-class step. `iterate_n`, `iterate_until`, and `iterate_to_fixpoint` iterate inside the algebra, with the iteration bounded by the type, the error short-circuit automatic, and the log accumulating across every tick.
- **Choice** makes the branch a first-class step. `branch` routes on the current value, `branch_with` reads state and context as well, and each arm is itself a flow.

A fluent chain that wires arrows together is the textual form of a wiring diagram, the graphical calculus of monoidal categories that Joyal and Street established in 1991. The builder *is* the algebra's term syntax. A clean line of the language corresponds to a well-typed diagram by construction. An earlier attempt in this project, a `ControlFlowBuilder`, reached for the same surface but stood disconnected from the causal monad. Because the causal monad is the central carrier, a control-flow surface that bypassed it was structurally the wrong choice, and the `ControlFlowBuilder` was removed eventually. The Causal Arrow shares the same ambition with the correct foundation: a builder *over* the propagating effect that specializes *into* the monad's composition.

## The language

The Causal Flow Language (CFL) is the surface a developer uses to write dynamic causal processes in DeepCausality. `CausalFlow` is its type, a fluent facade; the Causal Arrow is the engine underneath, and it stays out of the way for almost all everyday use. The smallest example looks like this:

```rust
use deep_causality::CausalFlow;

let outcome = CausalFlow::value(2_i64)
    .try_step(|x| Ok(x + 3))
    .map(|x| x * 10)
    .finish();

assert_eq!(outcome, Ok(50));
```


In the example, `value` sets the initial value. `try_step` runs a fallible step and hands the closure the unwrapped value. `map` transforms it. `finish` returns a `Result`. No `EffectValue`, no `pure`, no manual error check between steps. 

A more complex, five-stage dynamic causal process is written just as simply:

```rust
CausalFlow::value(inputs)
    .try_step(stage_load)
    .try_step(stage_align)
    .try_step(stage_pair)
    .try_step(stage_solve_gm)
    .try_step(stage_aggregate)
    .run(print_report, |err| eprintln!("Pipeline failed:\n  {err:?}"));
```

This recovers Earth's gravitational constant from time-dilation data. Put simply, it inverts 
the Einstein field equations in just seven lines of Rust, as demonstrated in the [`gm_recovery`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/chronometric_examples/gm_recovery) example. 
The complexity of the causal monad is gone, for good.

### The algebra loop

In the [`corrective_lane_keeping`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_lane_keeping) example, a vehicle drifts under a crosswind; a monitor watches the offset each tick and a controller corrects it when the drift crosses the safe threshold. The whole closed loop is just:

```rust
CausalFlow::from(model::initial_process())
    .iterate_n(N_TICKS, |tick| {
        tick.bind(model::simulate_step).branch(
            // is necessity condition met?
            |offset| offset.abs() > cfg.anomaly_threshold, 
            // if so, self-correct
            |hot|  hot.intervene_if(|_| true, |o| model::correction(o, &cfg)),
            // otherwise pass through
            |cold| cold,                                                      
        )
    })
    .into_process()
```

The iteration is bounded by `iterate_n`. The correction is a `branch`, not an `if`. The intervention is the monad's `do`-operator, `intervene`, and it enters the audit log like any other. Run the same flow without the branch and the vehicle leaves the road; run it with the branch and it holds the lane for the full simulation. The difference between failure and safety is one verb, and the closed loop stays as auditable as the open run it replaces.

The shape generalizes. In [`geometric_tcas`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples/geometric_tcas), an avionics collision-avoidance system runs a thirty-tick encounter in which the autopilot takes over (intervenes) only if the pilot ignores the advisory:

```rust
CausalFlow::value(engagement).iterate_n(30, |tick| {
    tick.next(assess)
        .branch(|e| e.will_intervene, |hot| hot.next(intervene), |cold| cold)
        .next(output)
        .next(integrate)
});
```

`next` composes a whole named sub-pipeline, so each stage of the tick reads as one word, and the conditional takeover is a branch on the carried value. The navigation loop in [`magnav`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples/magnav) follows the identical form, then feeds `finish()` straight into ordinary Rust `Result` handling. One small set of verbs covers stateless transforms, stateful Markov loops, conditional control, and reusable composition.

### Reusable pipelines

Because the engine is an arrow, a pipeline is also a value. That means a complex process can be broken into smaller, more manageable processes, then combined into a larger one.

```rust
fn sense(raw: Raw)    -> CausalFlow<Reading> { CausalFlow::value(raw).try_step(parse).try_step(calibrate) }
fn decide(r: Reading) -> CausalFlow<Plan>    { CausalFlow::value(r).try_step(estimate).map(plan) }
fn act(p: Plan)       -> CausalFlow<Command> { CausalFlow::value(p).try_step(authorize).map(emit) }

let command = CausalFlow::value(raw)
                    .next(sense)  // runs the sensing process.
                    .next(decide) // runs the decision process based on the sensing.
                    .next(act)    // runs the act process based on the decision.
                    .finish()?;
```

It also means that a process, once built, can run over arbitrary data that matches its input type.

```rust
let controller = causal_arrow(sense).next(decide).next(act).build();

let a = controller.run(raw_a); // built once, run on many inputs
let b = controller.run(raw_b);
```

## What it means to write a foundation

The CausalFlow DSL removes the complexity that previously plagued the causal monad, and it does so in a principled way. 
It is the term syntax of the arrow the monad already contained, so its readability and its rigor are the same property seen from two sides. Seven fluent lines now express what once demanded the full machinery, and each line
is a control law you can now read as plainly as you can state it. The embedded functions are simple physics wrapped in a causal effect propagation process that lifts the physics result into the propagating effect the CausalFlow DSL needs. That simple functional wrapper enables all the power of the CausalFlow DSL. 

This is the first release of the CausalFlow DSL, and it is deliberately a foundation, the beginning of something larger. As our understanding of dynamic causality evolves, the need for expressiveness only grows, while complexity must stay as low as possible. The CausalFlow DSL is meant to tilt that balance toward more expressiveness with less complexity.
The single axiom of the EPP gave causality a theory that composes with modern physics. The Causal Flow Language gives that theory a language you can write, and, with any luck, will want to write.

## Getting started

* Read [Hello, Causal Flow](https://docs.deepcausality.com/getting-started/hello-causal-flow/) for the hands-on introduction.
* Read the [Causal Flow](https://docs.deepcausality.com/concepts/causal-flow/) concept page for the full verb set and what each one lowers to.
* Read the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) and [Effect Propagation Process](https://docs.deepcausality.com/concepts/effect-propagation-process/) pages for the algebra underneath.
* Run the examples this post draws from:
  * `git clone https://github.com/deepcausality-rs/deep_causality.git && cd deep_causality`
  * `cargo run -p chronometric_examples --example gm_recovery`
  * `cargo run -p causal_correction_examples --example corrective_lane_keeping`
  * `cargo run -p avionics_examples --example geometric_tcas`
* Join the [community](https://www.deepcausality.com/community/) or the [Discord server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
