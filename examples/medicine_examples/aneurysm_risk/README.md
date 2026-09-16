# Vascular Hemodynamics & Aneurysm Rupture Risk

## 1. Medical Background

**Aneurysms** are abnormal bulges in blood vessel walls that can rupture, causing life-threatening internal bleeding (e.g., Subarachnoid Hemorrhage).

A key driver of aneurysm growth and rupture is **wall shear stress**, the tangential drag of flowing blood on the vessel wall. Healthy cerebral arteries run at roughly 1 to 7 Pa. Inside an aneurysm dome the lumen widens, the flow slows, and the shear collapses; sustained shear below about 0.4 Pa leaves the endothelium unable to maintain the wall. That low-shear pathway is what this example models. The second recognised factor is the spatial gradient of the shear, which peaks at the dome neck.

Currently, clinical decisions are often based on simple **geometric size** (e.g., "treat if > 7mm"). However, many small aneurysms rupture, and many large ones remain stable. A physics-based risk assessment is needed.

## 2. The Challenge

*   **Static vs. Dynamic:** Clinical decision-making relies on static images (CTA/MRA), but rupture is a dynamic failure process accumulated over time.
*   **Complex Geometry:** Every patient's vessel anatomy is unique, altering blood flow patterns.
*   **Computational Cost:** Full computational fluid dynamics (CFD) is expensive and slow for routine clinical use.

## 3. The DeepCausality Solution

The vessel centreline is a simplicial manifold whose payload is the lumen radius at each node, and
three categorical operations carry the whole analysis.

| Operation | Reads | Produces |
|---|---|---|
| `fmap` | one radius | the wall shear stress there, from the Poiseuille closure `τ = 4μQ / (πR³)` |
| `extend` | a node and its neighbours | the spatial gradient of the shear, which needs the cursor the comonad supplies |
| `fold` | the whole payload | the dome minimum and the peak gradient |

Degeneration then accrues over cardiac cycles wherever the shear sits below the threshold, in
proportion to how far below it sits, and the run reports the epoch at which the index crosses the
rupture-risk line.

Precision is a parameter: the `FloatType` alias in `main.rs` switches the geometry, the stress
profile, its gradient and the accumulation to `f32`, `BFloat16` or `Float106`.

## 4. Gained Value

1.  **Precision Medicine:** Moves beyond "size" to "hemodynamic stress," potentially identifying high-risk small aneurysms.
2.  **Safety & Speed:** By using simplified physics kernels within a causal framework, we get actionable risk scores faster than full Navier-Stokes simulations.
3.  **Integration:** The system outputs a clear, interpretable "Rupture Risk" alert, ready for integration into clinical dashboards.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example aneurysm_risk
```
