# QTT immersed cylinder — Brinkman-penalized drag (tensor-train)

Verifies the immersed-body QTT solver (`QttImmersed2d`): a cylinder in a periodic free-stream, enforced
by **Brinkman volume penalization**, with drag read as a **tensor-train contraction** of the body mask
with the velocity deficit. This is the last piece of **Gap 1** of the plasma-blackout analysis (the
immersed body + surface observables). Driven through `CfdFlow::qtt_march`.

## The method

The body is a smoothed volume-fraction mask `χ_body ∈ [0, 1]` (no cut cells — the periodic
power-of-two grid is uniform). Each step adds the forcing `−(1/η)·χ_body ⊙ (u − u_body)` to the velocity
rate, driving the velocity to zero inside the solid; the divergence-free projection then cleans up. Drag
falls out as `F = (1/η) ∫ χ_body ⊙ (u − u_body) dV` — a single train `inner` product, no surface
reconstruction — nondimensionalized to `C_d = F_x / (½ ρ U² D)`.

## What is verified (3 gates, exit nonzero on break)

1. **No-slip** — the velocity inside the body (mask > 0.9) falls to the penalization floor.
2. **Accuracy vs bond** — the drag coefficient **converges** as the round bond cap is raised (the headline
   QTT-CFD metric: accuracy traded against tensor-train rank).
3. **Physical drag** — the streamwise drag is positive and finite.

## Measured (f64, Apple M3 Max, release, ~1 s)

```
Accuracy vs bond: immersed cylinder, drag from the penalization contraction
  bond <=   4   C_d = 24.0543   |dC_d| =    --    interior_max|u| = 4.88e-2   divergence = 3.82e-1
  bond <=   8   C_d = 23.7649   |dC_d| = 2.89e-1   interior_max|u| = 4.35e-2   divergence = 3.25e-2
  bond <=  16   C_d = 23.7577   |dC_d| = 7.22e-3   interior_max|u| = 4.22e-2   divergence = 3.01e-7
  bond <=  24   C_d = 23.7577   |dC_d| = 1.89e-11   interior_max|u| = 4.22e-2   divergence = 5.47e-14
```

- **Accuracy vs bond:** `C_d` settles `24.05 → 23.76 → 23.7577 → 23.7577`, with the successive change
  collapsing `2.9e-1 → 7.2e-3 → 1.9e-11` — clean convergence as the tensor-train is allowed more rank. The
  divergence residual likewise drops `3.8e-1 → 5.5e-14`: at a tight bond cap the projection can't fully
  enforce incompressibility; by bond 16 it is at machine precision. **This convergence is the verification
  result.**
- **No-slip:** interior `max|u| ≈ 4.2e-2` vs the free-stream `1.0` — the penalization brakes the flow to a
  few-percent floor inside the body.

## Honest reading of the absolute C_d

The absolute `C_d ≈ 23.8` is **not** the isolated-cylinder value (DEC reports `C_d ≈ 1.345` at Re 100).
The difference is expected and disclaimed:

- **~30 % blockage** — the cylinder spans a large fraction of the periodic box (a small body is
  under-resolved at 32²), so the effective drag is far above the unconfined value.
- **Penalization-integral force** — `F` counts the momentum sink over the whole *smoothed skirt*, not just
  pressure + friction on a sharp surface; with a 2-cell smoothing this inflates the magnitude.
- **Transient** — a periodic box has no momentum source to hold the free-stream, so `C_d` is read at a
  fixed horizon, not a true steady state.

So the committed DEC `C_d` is a **cross-reference**, not a target: the verification claim is the
*convergence trend* + no-slip + positivity, **not** the absolute number. Reproducing an absolute
isolated-cylinder `C_d` would need an inflow/outflow domain (the DEC solver's configuration), which is out
of scope for the periodic QTT solver.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example qtt_cylinder_verification
```

The accuracy-vs-bond table and the closing verdict are on stdout; the cross-reference and any `FAIL:` line
are on stderr (exit nonzero on a broken gate).

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The `FloatType` alias, the accuracy-vs-bond ladder driven through `CfdFlow::qtt_march`, and the self-verify gate. |
| `config.rs` | Case parameters, the cylinder mask, and the `QttMarchConfig` body case builder. |
| `print_utils.rs` | The no-slip / drag measurement, the table, and the three gates. |
| `baseline.txt` | A captured reference run (f64). |

## Reference

- **Angot, P., Bruneau, C.-H. & Fabrie, P.** (1999). *A penalization method to take into account obstacles
  in incompressible viscous flows.* Numer. Math. **81**, 497–520 — the Brinkman volume-penalization method.
- **Peddinti et al.** (2024), Commun. Phys. **7**, 135 — MPS incompressible NS around immersed objects.
- DEC isolated-cylinder cross-reference: `verification/dec_cylinder_verification` (`C_d ≈ 1.345`, Re 100).
