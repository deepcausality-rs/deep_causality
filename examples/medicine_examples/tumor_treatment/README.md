# Glioblastoma TTFields Optimization

## 1. Medical Background

**Glioblastoma Multiforme (GBM)** is an aggressive brain cancer. **Tumor Treating Fields (TTFields)** are a therapy using alternating electric fields to disrupt cancer cell division (mitosis).

TTFields work by exerting dielectrophoretic forces on polar molecules (like tubulin) during cell division. Crucially, the force is maximal when the **Electric Field (E)** is parallel to the **Axis of Cell Division**.

## 2. The Challenge

*   **Anisotropy:** Tumor cells don't all divide in the same direction. They are disorganized.
*   **Placement:** The electric field direction depends on where electrodes are placed on the patient's scalp.
*   **Blindspots:** A suboptimal placement might hit 50% of cells but leave the other 50% (dividing orthogonally) unaffected, leading to recurrence.

## 3. The DeepCausality Solution

This example uses **Geometric Algebra** and **Causal Optimization** to find the personalized "perfect angle."

*   **Geometric Algebra (`MultiVector`):** We treat the Electric Field $E$ and Cell Division Axes $a_i$ as geometric vectors. The alignment efficacy is computed via the **Inner Product** (contraction) $E \cdot a_i$. We aggregate this over the entire tumor volume (Mocked as a Voxel Grid).
*   **Causal Optimization:** We use a **Simulated Annealing** process wrapped in a `PropagatingEffect` monad.
    *   **Intervention:** "Set electrode angle to $(\theta, \phi)$".
    *   **Observation:** Calculate total disruption score.
    *   **Feedback:** If score improves, update state. If not, maybe explore (probabilistic).
    
## 4. Gained Value

1.  **Personalization:** Customizes therapy to the specific geometry of the patient's tumor (from MRI DTI).
2.  **Maximized Efficacy:** Ensuring the field aligns with the majority of dividing cells could significantly improve survival rates.
3.  **Mathematical Elegance:** Geometric Algebra provides a coordinate-free, robust way to handle 3D alignments and rotations compared to traditional linear algebra.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example ttfields
```
