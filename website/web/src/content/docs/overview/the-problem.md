---
title: The problem
description: Why classical causality's assumptions break in dynamic systems, and what that means in practice.
section: overview
order: 2
---

Classical computational causality, pioneered by Judea Pearl and others, is powerful, well-validated, and covers a large class of useful problems. However, classical computational causality is rooted in three assumptions that prevent its application to dynamic systems.

## The classical way of thinking

Imagine a simple thermostat:

- **Cause**: room temperature drops below 68°F.
- **Effect**: the furnace turns on.

A classical model captures this because three things hold:

1. **Time is a straight line.** The temperature drops *before* the furnace turns on. There is a clear "happens-before" relationship.
2. **The causal rules are fixed.** "If temperature < 68, turn on the furnace" is the same rule tomorrow as it is today.
3. **Context is implicit.** Whatever the thermostat does not measure is absorbed into the background. You do not need to model it explicitly.

Most of classical computational causality, from Pearl's Structural Causal Models to Granger's time-series analysis, lives inside them.

## Where the assumptions break

Now imagine a financial trading system, or a fleet of autonomous wildfire-fighting drones. Reality stops cooperating:

1. **Time is not a straight line.** A trading system observes events on nanosecond scales, but its decisions depend on the hourly high, yesterday's close, and the day's volume. Time becomes multi-layered and multi-scaled.
2. **The rules can change.** During a normal market day, "low interest rates push stock prices up" is a workable rule. During a crash, that rule breaks and "high fear pushes every asset down" takes over. The causal relationships in the system have changed mid-flight.
3. **Context changes continuously.** An autonomous drone navigating by GPS works fine until it enters a tunnel and loses signal. The computer vision system saw the tunnel coming, but if context is implicit, there is nowhere to put that fact and nothing to do with it.

The third point is the deepest one. When the context changes, the rules can change. When the rules can change, you need a framework that treats both as first-class moving parts. Classical causality does not.

DeepCausality was built from the ground up for the dynamic case. Context is explicit. Causality can evolve in response to a changing context. Verifiability is preserved on the safety-critical path. The [next page](/docs/overview/core-idea/) explains the single idea that makes all of that possible.
