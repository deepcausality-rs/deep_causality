# Glioblastoma TTFields Optimization

This example finds the electrode orientation that aligns a Tumor Treating Field with the most cell-division axes in a glioblastoma.

## 1. Medical Background

**Glioblastoma Multiforme (GBM)** is an aggressive brain cancer. **Tumor Treating Fields (TTFields)** disrupt cancer cell division (mitosis) with alternating electric fields.

The field exerts dielectrophoretic forces on polar molecules such as tubulin during cell division. The force peaks when the **Electric Field (E)** runs parallel to the **Axis of Cell Division**.

## 2. The Challenge

*   **Anisotropy:** Tumor cells divide in disorganized directions.
*   **Placement:** The field direction depends on where the electrodes sit on the patient's scalp.
*   **Blind spots:** A poor placement might reach 50% of the cells and leave the other 50% (dividing orthogonally) unaffected, which leads to recurrence.

## 3. The DeepCausality Solution

The example maximizes the mean alignment $\langle|E(\theta,\phi) \cdot a_i|\rangle$ between the field direction and the division axes $a_i$ over the electrode angles $(\theta, \phi)$.

*   **Tumor model:** A seeded generator samples 100 voxels in a box, each with a division axis biased toward the tumor's invasion axis, so every run reports the same anatomy.
*   **Exact gradient:** The objective is written once over the `Scalar` bound. Evaluated over `Dual`, it returns its exact gradient, and the optimizer ascends that gradient.
*   **Causal monad:** `CausalFlow` sequences the ascent, and each step returns a `PropagatingEffect`. A gradient or value that leaves the finite range ends the run through the error channel.
*   **Precision as a parameter:** The `FloatType` alias in `main.rs` re-runs the objective and its gradient at another scalar.

## 4. Gained Value

1.  **Personalization:** Fits the therapy to the geometry of the patient's tumor, for example from MRI DTI.
2.  **Efficacy:** Aligning the field with most dividing cells could improve survival.
3.  **Exact derivatives:** Automatic differentiation gives the gradient without finite-difference error or a hand-derived formula.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example tumor_treatment
```
