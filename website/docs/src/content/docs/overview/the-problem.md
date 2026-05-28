---
title: Problem
description: What problems dynamic causality can solve that classical causality cannot.
sidebar:
  order: 3
---

Classical computational causality, pioneered by Judea Pearl and others, is powerful, well-validated, and covers a large class of useful problems. However, classical computational causality is rooted in three assumptions that prevent its application to dynamic systems.

## The classical way of thinking

Imagine a simple thermostat:

- **Cause**: room temperature drops below 68°F.
- **Effect**: the furnace turns on.

A classical model captures this because three things hold:

1. **Time is a straight line.** The temperature drops *before* the furnace turns on. There is a clear "happens-before" relationship.
2. **The causal rules are fixed.** "If temperature < 68, turn on the furnace" is the same rule tomorrow as it is today.
3. **Context is implicit.** Whatever the thermostat does not measure is absorbed into the background.

Most of classical computational causality, from Pearl's Structural Causal Models to Granger's time-series analysis, lives inside them.

## Where the assumptions break

Now imagine a financial trading system, or a fleet of autonomous wildfire-fighting drones:

1. **Time is not a straight line.** A trading system observes events on nanosecond scales, but its decisions depend on the hourly high, yesterday's close, and the day's volume. Time becomes multi-layered and multi-scaled.
2. **The rules can change.** During a normal market day, "low interest rates push stock prices up" is a workable rule. During a crash, that rule breaks and "high fear pushes every asset down" takes over. The causal relationships in the system have changed mid-flight.
3. **Context changes continuously.** An autonomous drone navigating by GPS works fine until it enters a tunnel and loses signal. The computer vision system saw the tunnel coming, but if context is implicit, there is nowhere to put that fact and nothing to do with it.

The third point is the critical one. When the context changes, the rules can change. When the rules can change, you need a framework that treats both as first-class moving parts.

## What dynamic causality enables

When causality is treated as a dynamic process rather than a static graph, the problems classical causality cannot reach come back into scope. DeepCausality is built around exactly this premise: an explicit Context that holds the world, a Causaloid that can be a singleton or a hypergraph for arbitrary structure, a Causal Monad for sequential composition with first-class intervention, an Effect Ethos that verifies actions when the reasoning itself can no longer be statically verified, and a deployment surface that runs on Tokio for production. 

This page first walks through the problem categories that become tractable once the static assumption is dropped. Each category is a problem class where a real practitioner today either picks two or three separate libraries and writes glue forever, or ends up writing a custom solution because nothing else worked. DeepCausality was built from the ground up for the dynamic case, which enables a number of use cases that conventional tooling cannot reach. Where you find yourself on the list is roughly the answer to whether DeepCausality is the right tool for the work you are doing.

### 1. Dynamic control systems

Classical control theory does well when the plant model is fixed. A PID controller for a thermostat, a Kalman filter for an inertial measurement unit, a model-predictive controller for a refinery column. The industry has spent decades building qualified toolchains around exactly this case. Simulink Embedded Coder, SCADE Suite, Ansys SCADE, certified compilers under DO-178C and DO-330. Each of them translates a fixed plant model into proven-correct generated code, and the regulatory trajectory points toward more of that, not less. Static control is largely a solved problem and is not the gap DeepCausality fills.

Dynamic control is the gap. A loop whose plant model evolves with the operating regime, whose causal structure rewires when the environment changes, whose adaptation cannot be captured as a parametric gain schedule, has no settled answer. Adaptive wind turbine control under shifting atmospheric stratification. Power grid management as the renewable mix oscillates intraday. Adaptive control surfaces on a vehicle whose aerodynamics shift with payload, fuel state, or damage. The plant itself is dynamic, and the rules that govern it are moving targets that no code-gen pipeline can pin down ahead of time.

DeepCausality is built for exactly this case. The four reasoning modalities (static, dynamic, adaptive, emergent) map onto a set of control problems. The Context primitive carries the operating regime explicitly, so a regime change becomes a Context update rather than a model rebuild. The Causaloid graph can be reshaped at runtime when the structure of the plant changes. The Causal State Machine proposes control actions whose state combinations were not enumerated at design time, and the Effect Ethos checks each action against an immutable graph of computable norms before it actuates.

### 2. Multi-physics and multi-regime simulation

Some simulations cross a single mathematical regime: a tensor solver for fluid flow, a manifold integrator for orbital mechanics, a Clifford-algebra rotor for rigid-body dynamics. Each one is well served by an existing library. The harder problems cross regimes mid-pipeline. A relativistic plasma that needs Newtonian magnetohydrodynamics in the weak-field zone, full general relativity near a compact object, and breaks down at the event horizon where the physics itself stops being defined. A multi-scale climate model that switches from cloud-resolving physics to large-eddy simulation to global circulation along one atmospheric column. A weld pool that goes from solid mechanics to viscous flow to free-surface thermodynamics in a few millimeters of travel.

The conventional alternative is to stitch separate solvers together across language boundaries. General-relativity code in C++, plasma code in Fortran, orchestration in Python, regime transitions in a hand-written switching layer that is hard to debug. The handoffs leak precision, lose audit trails, and obscure where the model actually crosses from one regime into the next.

DeepCausality lets you write the physics the way you would write it on paper, regime changes included. The [`grmhd`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/grmhd) example is a five-step `bind` chain that crosses two regime boundaries inside one closure. Step 1 builds the Schwarzschild metric and the Einstein tensor in tensor algebra. Step 2 inspects the local curvature and selects the metric signature for what comes next: Minkowski for the relativistic regime, Euclidean for the classical regime. That is the first regime transition, captured as a single closure inside the chain. Step 3 leaves tensor algebra entirely and computes the Lorentz force as a Clifford bivector in the metric just chosen dynamically in the previous step. Step 4 returns to tensor algebra and contracts the electromagnetic field tensor with the spacetime metric to produce the stress-energy tensor that feeds back into Step 1. Step 5 is a stability analysis that decides whether the simulation is still physically meaningful or whether it is approaching a singularity where the physics itself stops being defined. That is the second regime boundary. Two regime changes, four mathematical domains, one `bind` chain that reads top to bottom like the derivation.

The simulation reads the way the physicist thinks about it: step one, two, three, regime change, step four, five, regime boundary.

Precision is one type alias for the whole pipeline; flip `f64` to `Float106` and composition drift drops from ~10⁻¹⁶ to ~10⁻³¹ on the capstone spinor example. The algebraic trait hierarchy enforces associativity, commutativity, and distributivity at compile time, so an algorithm that requires associativity cannot silently accept a type that violates it.

### 3. Financial systems under regime change with compliance pressure

Trading systems where "low rates lift stocks" inverts during a credit crisis. Risk engines that must explain why they sized a hedge the way they did. Anti-fraud platforms where the adversary evolves faster than the model. The rules that govern the system are themselves moving targets, and compliance asks for explanations that survive the next regime.

The Context primitive is a typed hypergraph mutated in place across the lifetime of a run, so changing the regime does not require rebuilding the model. The Adjustable trait absorbs sensor drift, missing prints, and calibration deltas without losing history. Counterfactual analysis is a first-class operation: `intervene` rewrites a value mid-chain, and `extra_contexts` carry parallel hypothetical worlds for what-if replay. The propagating effect carries the audit log alongside the value, so the compliance report comes out of the same pipeline that produced the decision.

A typical trading stack hard-codes the regime in static configuration. When the regime breaks, someone rewrites the config. A counterfactual replay tool is usually a separate offline pipeline that approximates the production engine. DeepCausality runs the same engine in both modes, so the replay is the production behavior with one input swapped.

### 4. Scientific discovery

Turbulence forecasting at the [MIT Aerofluids, Learning & Discovery Lab](https://www.adrianld.mit.edu/). Computational biology pipelines that screen for drug targets. Materials science loops that propose alloys and feed the proposals back into a simulator. The structure of the problem is part of the answer; discovery and inference are two halves of the same workflow, and shipping a model that took six months to discover usually means rewriting it in a serving language.

The Causal Discovery Language is a typestate-builder DSL that walks raw data through configure, load, clean, select features, discover, analyze, and finalize. SURD and MRMR live in [`deep_causality_algorithms`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms) and feed the pipeline directly. The output of the final stage is a `Causaloid` indistinguishable from a hand-written one, so the discovered model feeds the rest of the framework with no translation step. `Uncertain<T>` keeps noisy measurements honest end to end. Discovery in Python lands in a notebook. Productionizing it means a rewrite into C++ or Java behind a serving framework. DeepCausality closes the gap between the discovery stage and the inference stage because both can be built in the same ecosystem.


### 5. Autonomous systems in open environments

The Causal State Machine infers active states from the propagating effect rather than enumerating them at design time, so combinations the designer did not foresee still produce sensible actions. Emergent causality lets the reasoning graph rewire itself in response to context. The Effect Ethos restores verifiability at the action layer when verifiability at the reasoning layer is no longer feasible. Every action goes through the deontic check. Reasoning is free to be emergent. Actions are not.

A classical finite-state machine cannot represent the compound condition "GPS loss and low battery and a passenger-corridor restriction and deteriorating wind" unless someone enumerated it at design time. A learned policy has no permissibility layer. A static rule engine cannot rewire itself when the world changes shape. DeepCausality is the substrate that lets reasoning be emergent while keeping actions verifiably safe.

### 6. The intersection that needs almost everything

Some problems sit at the intersection of multiple categories above. An [avionics flight envelope monitor](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples/flight_envelope_monitor) is at once a real-time safety-critical control loop, a multi-physics computation over sensor fusion, an open-environment autonomous system whose state space cannot be enumerated, and a regulated artifact that an authority will audit. Each axis would individually be hard. Together they are intractable without a unifying substrate.

The same propagating effect carries a tensor verdict from a physics stage, a state from a Markovian step, an uncertain reading from a noisy sensor, and an audit log from every prior step, all under one composition law. The CSM proposes actions whose state combinations no one enumerated. The Effect Ethos checks each action against the permissibility graph. DeepCausality provides one carrier and one composition law that cover the entire pipeline.

### 7. Dynamic emergent causality (frontier)

A class of problems sits beyond what production engineering currently handles. The causal structure of the system is itself co-evolving with a dynamic context. Examples include long-running closed-loop control of self-modifying biological systems, multi-agent ecologies where the agents redesign each other, and certain frontier safety questions about AI systems whose internal causal graph rewires in response to inputs. The central problem is: "how do we even formalize it?"

Emergent causality is the experimental fourth modality in DeepCausality. New Causaloids and new edges can be introduced by a generative process at runtime. The Causaloid graph is allowed to take shapes no upfront proof can foresee. The Effect Ethos is what makes the experiment safe to run: reasoning evolves freely, but every action is checked against an immutable ethos of computable norms before it leaves the system. The mode is deliberately experimental, and the work is currently confined to research.

## Where to go from here

The [next page](/overview/core-idea/) gives you the single axiom on which all of this is built. If you recognized your problem in one of the seven categories above, the rest of the documentation describes the primitives in detail; the [Concepts](/concepts/) index is the entry point. If you did not recognize your problem in any of them, the conventional Pearl, Granger, or Rubin tooling probably already covers what you need, and DeepCausality would be more substrate than the work asks for.
