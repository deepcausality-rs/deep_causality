# Virtual Epilepsy Surgery Planning (Digital Twin)

## 1. Medical Background

**Epilepsy** surgery is a curative option for drug-resistant patients. The goal is to remove the **Seizure Onset Zone (SOZ)**—the brain region triggering seizures—without damaging essential functional areas.

The brain is a complex **Connectome** (network of connected regions). Seizures are often network phenomena, where a "hub" drives synchronization across the brain.

## 2. The Challenge

*   **Indeterminacy:** It's often unclear exactly which node is the driver. Removing the wrong node fails to stop seizures.
*   **Invasiveness:** Trial-and-error in surgery is impossible. We need a way to "test" resections virtually before cutting.
*   **Network Dynamics:** A static map isn't enough; we need to simulate how the network *behaves* when parts are removed.

## 3. The DeepCausality Solution

This example builds a **Virtual Brain (Digital Twin)** to simulate surgical outcomes.

*   **Topology (`Graph`):** We model brain regions as nodes and white-matter tracts as edges, loaded from grid-like connectome data.
*   **Dynamics (`Kuramoto Model`):** We simulate synchronization. Seizures are modeled as "hyper-synchronous" states (Order Parameter $R > 0.8$).
*   **Causal Intervention (`do(resect)`):** We perform **Virtual Resection**:
    1.  **Baseline:** Run simulation on the full graph -> Confirm Seizure.
    2.  **Intervention:** For each suspect node, strictly remove it (causal intervention implies modifying the graph topology).
    3.  **Counterfactual:** Run simulation on the *resected* graph.
    4.  **Outcome:** If synchronization drops below threshold, the resection is marked "Curative."

## 4. Gained Value

1.  **Risk Reduction:** Identifies optimal targets non-invasively, reducing the chance of failed surgeries.
2.  **Systemic View:** Treats the brain as a system, not isolated spots, capturing complex network failures.
3.  **Data-Driven:** Can directly ingest patient DTI-MRI connectivity matrices to build the graph.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example epilepsy
```
