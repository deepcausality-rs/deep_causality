# Introduction to DeepCausality

Computational causality differs from deep learning (AI) as it uses a different foundation. Deep learning, at its core, is excellent at pattern matching and recognition. Object detection in computer vision excels with deep learning, fraud detection on credit card transactions excels with deep learning, and there are many more examples. Large Language Models (LLMs) like ChatGPT take the idea one step further and predict, for the most part, the next word in a sentence with stunning accuracy. When the prediction is off, you experience what is widely known as hallucination. However, LLMs and deep learning are fundamentally correlation-based methods, meaning there is no foundational concept of space, time, context, or causality. Computational causality comes in handy when you need:

1) Deterministic reasoning: Same input, same output.
2) Probabilistic reasoning: What are the odds that X is true? How confident can we be?
3) Fully explainable: You get a logical line of reasoning.

These properties of causality are valuable in high-stakes and regulated industries such as medicine, finance, robotics, avionics, and industrial control systems. However, the classical methods of computational causality work in a particular way that is important to understand first.

## The "Classical" Way of Thinking About Causality

Imagine a simple thermostat.

*   **Cause:** The room temperature drops below 68 degrees Fahrenheit.
*   **Effect:** The furnace turns on.

A typical classical causal model works because it relies on three fundamental assumptions:

1.  **Time is a straight line.** The temperature *always* drops *before* the furnace turns on. There's a clear "happen-before" relationship.
2.  **The causal rules are fixed.** The law "if temp < 68, then turn on furnace" is static and unchanging. It will be the same rule tomorrow as it is today.
3.  **Context is implicit.** Context is assumed as the implicit background, and therefore all data are captured in variables relative to the implicit context.

Previous computational causality frameworks (like those pioneered by Judea Pearl) are built on these three powerful assumptions. They provide the foundation to discover and reason about this fixed causality in a world where time moves forward predictably, the rules remain the same, and adding some variables captures the implicit context. The problem, however, emerges when these assumptions are no longer true.

## The Problem: A Dynamic World Defies Classical Causality

Next, imagine a more complex system, like a financial market or a fleet of autonomous wildfire-fighting drones, and you'll see that reality operates differently:

1.  **Time is NOT a straight line.** In a trading system, events happen on nanosecond scales, but the market context relies on different time scales, i.e., the hourly high price, the previous day's close price, or the daily trade volume. Time becomes multi-layered, multi-scaled, and complex.

2.  **The rules can change.** This is the most important point. During a normal market day, "low interest rates cause stock prices to rise." But during a market crash (a "regime shift"), that rule breaks down entirely, and a new rule like "high fear causes all assets to fall" takes over. The causal relationships within a system have changed dynamically.

3.  **Context changes dynamically.** The reason causal rules may change is because a system's context is changing dynamically. For an autonomous drone relying on a GPS signal, navigation might be valid, but the moment the drone enters a tunnel, the GPS signal gets temporarily lost and with it, the drone's ability to navigate. This is known as a regime shift and poses a fundamental challenge to all autonomous systems. Here, the context is particularly important because the computer vision system almost certainly identified the tunnel entrance, but without a workable context, the information cannot be used.

DeepCausality was created from the ground up for dynamic causality where context changes continuously and where the causal rules themselves may change in response to a changing context.

## The Core Idea of DeepCausality

DeepCausality rethinks causality from the ground up based on a single foundation:

**"Causality is a spacetime-agnostic functional dependency."**

*   **"Functional dependency":** This just means `Effect2 = function(Effect1)`. Instead of "cause and effect," think of a chain reaction where one event triggers a causal function that produces the next event. The focus is on the *process* of event propagation.
*   **"Spacetime-agnostic":** This is the radical part. Time and space are just another piece of contextual data for the causal function.
*   **"Explicit Context":** Because the causal function is independent of spacetime, any time or space-related data needs to be provided via a context. A powerful hypergraph enables flexible context modeling, and DeepCausality enables a model to access and use multiple contexts.

The core of the idea is similar to a ripple in a pond. One ripple (an effect) propagates outward and creates the next ripple (another effect). DeepCausality is a framework for defining the rules of how those ripples spread. For more information about the underlying effect propagation process, see the [Deep Dive document.](README_DEEP_DIVE.md).