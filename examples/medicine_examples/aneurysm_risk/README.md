# Vascular Hemodynamics & Aneurysm Rupture Risk

## 1. Medical Background

**Aneurysms** are abnormal bulges in blood vessel walls that can rupture, causing life-threatening internal bleeding (e.g., Subarachnoid Hemorrhage).

A key driver of aneurysm growth and rupture is **Wall Shear Stress (WSS)**—the frictional force exerted by flowing blood against the vessel wall. High WSS can trigger inflammation and wall weakening, while low oscillatory WSS can cause plaque formation.

Currently, clinical decisions are often based on simple **geometric size** (e.g., "treat if > 7mm"). However, many small aneurysms rupture, and many large ones remain stable. A physics-based risk assessment is needed.

## 2. The Challenge

*   **Static vs. Dynamic:** Clinical decision-making relies on static images (CTA/MRA), but rupture is a dynamic failure process accumulated over time.
*   **Complex Geometry:** Every patient's vessel anatomy is unique, altering blood flow patterns.
*   **Computational Cost:** Full computational fluid dynamics (CFD) is expensive and slow for routine clinical use.

## 3. The DeepCausality Solution

This example demonstrates a **Digital Twin** approach using `deep_causality_physics` and `deep_causality_topology`.

*   **Topology (`Manifold`):** We model the vessel surface as a `Manifold` constructed from specific patient data (mocked here as a cylinder with a bulge).
*   **Physics (`Fluid Dynamics`):** We approximate WSS using velocity gradients near the manifold surface, simulating the effect of pulsatile blood flow.
*   **Causality (`PropagatingEffect`):** We treat vessel wall fatigue not as a number, but as a **Causal Process**.
    *   **State:** Accumulated Fatigue.
    *   **Effect:** WSS > Critical Threshold causes cumulative damage.
    *   **Monad:** The `PropagatingEffect` monad manages the accumulation state and handles "Failure Events" (Rupture) cleanly, separating the happy path (stable vessel) from the critical path (rupture).

## 4. Gained Value

1.  **Precision Medicine:** Moves beyond "size" to "hemodynamic stress," potentially identifying high-risk small aneurysms.
2.  **Safety & Speed:** By using simplified physics kernels within a causal framework, we get actionable risk scores faster than full Navier-Stokes simulations.
3.  **Integration:** The system outputs a clear, interpretable "Rupture Risk" alert, ready for integration into clinical dashboards.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example hemodynamics
```
