# Vascular Hemodynamics & Aneurysm Rupture Risk

## 1. Medical Background

**Aneurysms** are abnormal bulges in blood vessel walls that can rupture, causing life-threatening internal bleeding (e.g., Subarachnoid Hemorrhage).

A key driver of aneurysm growth and rupture is **wall shear stress**, the tangential drag of flowing blood on the vessel wall. Healthy cerebral arteries run at roughly 1 to 7 Pa. Inside an aneurysm dome the lumen widens, the flow slows, and the shear collapses; sustained shear below about 0.4 Pa leaves the endothelium unable to maintain the wall. This example models that low-shear pathway. The second recognised factor is the spatial gradient of the shear, which peaks at the dome neck.

Clinical decisions often rest on **geometric size** alone (e.g., "treat if > 7mm"), yet many small aneurysms rupture and many large ones remain stable. A physics-based risk assessment addresses that gap.

## 2. The Challenge

*   **Static vs. Dynamic:** Clinical decisions rely on static images (CTA/MRA), but rupture is a failure process that accumulates over time.
*   **Complex Geometry:** Each patient's vessel anatomy differs, and so do the flow patterns.
*   **Computational Cost:** Full computational fluid dynamics (CFD) is too expensive and slow for routine clinical use.

## 3. The DeepCausality Solution

The vessel centreline is a simplicial manifold whose payload is the lumen radius at each node.
Three categorical operations carry the analysis:

| Operation | Reads | Produces |
|---|---|---|
| `fmap` | one radius | the wall shear stress there, from the Poiseuille closure `τ = 4μQ / (πR³)` |
| `extend` | a node and its neighbours | the spatial gradient of the shear, which needs the cursor the comonad supplies |
| `fold` | the whole payload | the dome minimum and the peak gradient |

Degeneration accrues over cardiac cycles wherever the shear sits below the threshold, in
proportion to the deficit, and the run reports the epoch at which the index crosses the
rupture-risk line.

Precision is a parameter: the `FloatType` alias in `main.rs` switches the geometry, the stress
profile, its gradient and the accumulation to `f32`, `BFloat16` or `Float106`.

## 4. Gained Value

1.  **Precision Medicine:** Assesses hemodynamic stress rather than size, which can flag high-risk small aneurysms.
2.  **Speed:** Simplified physics kernels within a causal framework produce a risk score faster than a full Navier-Stokes simulation.
3.  **Interpretability:** The run flags each epoch past the threshold with a "rupture risk" marker and names the first one.

## 5. Running the Example

```bash
cargo run -p medicine_examples --example aneurysm_risk
```
