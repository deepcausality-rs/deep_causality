# Virtual Epilepsy Surgery Planning (Digital Twin)

This example builds a digital twin of a brain network, confirms that it seizes, then disconnects each region in turn and re-simulates to find the resections that stop the seizure.

## 1. Medical Background

Surgery can cure **epilepsy** in patients who do not respond to drugs. The surgeon removes the **Seizure Onset Zone (SOZ)**, the brain region that triggers seizures, and spares essential functional areas.

The brain is a **connectome**: a network of connected regions. Seizures often spread through the network, with a "hub" region driving synchronization across the brain.

## 2. The Challenge

*   **Indeterminacy:** Which node drives the seizure is often unclear, and removing the wrong one leaves the seizures in place.
*   **Invasiveness:** Surgery allows no trial and error, so resections must be tested virtually before cutting.
*   **Network dynamics:** A static map does not show how the network *behaves* once parts are removed; that takes simulation.

## 3. The DeepCausality Solution

*   **Topology (`Graph`):** Brain regions are nodes and white-matter tracts are edges. Region 0 is the hub and connects to every other region; the remaining regions form a chain.
*   **Dynamics (`Kuramoto Model`):** Each region is an oscillator pulled toward the phase of its neighbours. A seizure is a hyper-synchronous state (order parameter $R > 0.8$).
*   **Causal intervention (`do(resect)`):** The example performs a **virtual resection**:
    1.  **Baseline:** Simulate the full graph and confirm the seizure.
    2.  **Intervention:** For each suspect region, remove its connections from the graph.
    3.  **Counterfactual:** Simulate the resected graph.
    4.  **Outcome:** If synchronization drops below the threshold, the resection is marked "Curative."

## 4. Gained Value

1.  **Risk reduction:** Identifies targets non-invasively, lowering the chance of a failed surgery.
2.  **Systemic view:** Treats the brain as a network, not a set of isolated spots, and so captures failures that arise from the network.
3.  **Data-driven:** The graph can take a patient's DTI-MRI connectivity matrix in place of the synthetic connectome.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example epilepsy
```
