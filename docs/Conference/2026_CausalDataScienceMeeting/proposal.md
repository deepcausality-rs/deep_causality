# Dynamic Causality for a Dynamic World: From Causal Decisions to Safeguarded Actions

Presentation proposal for the Causal Data Science Meeting 2026 (online, November 4–5, 2026).

**Presenter.** Marvin Hansen, Director, Center for Dynamic Causality; creator and maintainer of
DeepCausality, a project of the LF AI & Data Foundation, part of the Linux Foundation.
marvin.hansen@causalcenter.com

## Problem: Static Causal Structure

The world is dynamic and in constant flux, and yet the methods of computational causality remain stuck in a static assumption that presupposes causal invariance. Evidence from financial markets, dynamic systems, and advanced engineering such as avionics shows that causal mechanisms do change whenever the governing regime changes. The presupposed causal invariance craters the moment the governing regime dictates a different causal mechanism. This occurs in avionics whenever an aircraft transitions from subsonic to transonic flight, or in financial markets when the governing market regime shifts from a bear market to a bull market. Consumer spending typically changes when the governing economic regime shifts into a recession, which changes spending behavior at a fundamental level. And yet, static causal models are structurally ill-equipped to handle the dynamic reality of a changing world.

## Solution: Dynamic Causality via DeepCausality

DeepCausality provides dynamic causality by making dynamic processes, context, and reasoning first-class citizens. The project is fully implemented in Rust and hosted by the LF AI & Data Foundation, part of the Linux Foundation. At its foundation, it separates causal structure from causal composition and, one step further, separates causal composition from temporal order, which allows it to reintroduce spacetime via an explicit context. As a result, dynamic causal processes built with DeepCausality natively handle regime changes and adapt to changing contexts.

The project supports multiple types of causal reasoning:
- Static: The causal graph is fixed and traversed along a predefined pathway.
- Dynamic: The graph is fixed, but the pathway through it is selected at runtime. A subgraph or a shortest-path traversal answers a query.
- Adaptive: The causal model itself dispatches to the next step based on its own internal logic. A clinical model, for example, hands off to a “normal blood pressure” or “high blood pressure” subgraph depending on the latest vital reading, normalized to the specific patient.
- Emergent: The graph is constructed and modified at runtime in response to a changing context.

Static, dynamic, and adaptive reasoning all remain deterministic. Their state space is bounded, their dispatch rules are known upfront, and formal verification remains practical. Emergent reasoning, by contrast, is fundamentally non-deterministic because it responds dynamically to a changing context, and it is therefore not recommended for safety-critical environments.

DeepCausality also reasons across multiple modalities:
- Deterministic: The reasoning rules are certain.
- Probabilistic: The reasoning contains uncertainty.
- Mixed: Both modalities coexist in different parts of a causal model.

Multi-modal support enables modular models: deterministic parts are modeled, tested, and certified in isolation, then combined with modules that provide probabilistic reasoning or even an encapsulated LLM for classification.

DeepCausality reasons natively over context through first-class support for:
1. No context
2. Single context
3. Multiple contexts
4. Dynamic context changes in content (update) or in structure (reshape)

DeepCausality handles counterfactuals natively across three distinct channels:
1. Alternate Value: What if the value itself were different?
2. Alternate Context: What if the same reasoning occurred in a different context?
3. Alternate State: What if the state threaded through the reasoning were different?

## Examples
- Retropulsion orbit entry and counterfactual trajectory correction (with 3D render)
- GPS blackout handling
- Digital twin for simulating weather impact on navigation

## Takeaways

- Conventional computational causality works when causal invariance holds.
- Dynamic computational causality begins where requirements demand dynamic change.
- Dynamic counterfactuals for dynamic causal models can be expressed by altering value, state, or context.

## Links

- Project website: https://www.deepcausality.com
- Documentation: https://docs.deepcausality.com
- Repository: https://github.com/deepcausality-rs/deep_causality
- Classical frameworks as special cases: `examples/classical_causality_examples` in the repository
- Blog: "Counterfactuals via the Causal Monad" and "The Causal Discovery Language, Rebuilt"

---
