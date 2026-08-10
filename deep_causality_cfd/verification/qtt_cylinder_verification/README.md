# QTT immersed cylinder — Brinkman-penalized drag (tensor-train)

Verifies the immersed-body QTT solver (`QttImmersed2d`): a cylinder in a periodic free-stream, enforced
by **Brinkman volume penalization**, with drag read as a **tensor-train contraction** of the body mask
with the velocity deficit. This is the last piece of **Gap 1** of the plasma-blackout analysis (the
immersed body + surface observables). Driven through `CfdFlow::march`.

## The method

The body is a smoothed volume-fraction mask `χ_body ∈ [0, 1]` (no cut cells — the periodic
power-of-two grid is uniform). Each step adds the forcing `−(1/η)·χ_body ⊙ (u − u_body)` to the velocity
rate, driving the velocity to zero inside the solid; the divergence-free projection then cleans up. Drag
falls out as `F = (1/η) ∫ χ_body ⊙ (u − u_body) dV` — a single train `inner` product, no surface
reconstruction — nondimensionalized to `C_d = F_x / (½ ρ U² D)`.

The shipped `config.rs` constants are `ν = 0.05`, `U = 1`, `RADIUS_FRAC = 0.15` on a `[0, 2π]²` box, so
the diameter is `D = 2 · 0.15 · 2π = 1.8850` and the Reynolds number is `Re_D = U·D/ν = 37.7`. That is
the case this harness runs. Neither the program banner nor the gates print `Re`; it follows from the
constants above.

## What is verified (5 gates, exit nonzero on break)

1. **No-slip** — the velocity inside the body (mask > 0.9) falls to the penalization floor. Tripwire.
2. **Bond saturation** — the drag coefficient stops moving as the round bond cap is raised. Tripwire.
   It says the tensor-train compression has converged; it says nothing about whether the compressed
   quantity is the right one.
3. **Physical drag** — the streamwise drag is positive and below `DRAG_SANITY_MAX = 100`. Tripwire, a
   wide blow-up guard, not an `O(1)` claim.
4. **η ladder** — whether the reported `C_d` settles as the Brinkman parameter is reduced. Reference
   class: the `η → 0` limit is what would license calling the penalization integral a drag.
5. **Mask-smoothing ladder** — whether `C_d` settles as the mask skirt width is varied. Reference
   class: a `C_d` that tracks the skirt width is reporting a numerical choice.

Gates 4 and 5 are the ones that constrain the number. On the superseded `L = 5` configuration both
reported `NOT CONVERGING`, and the harness exited nonzero on them rather than passing them over.

## Measured (superseded `L = 5` configuration, f64, Apple M3 Max, release, ~1 s)

The table below is the last captured run of this harness. It was taken **before** `L` was raised from
5 (32²) to 8 (256²) and the bond ladder was changed from `[4, 8, 16, 24]` to `[24, 48]`, so it does not
describe the shipped configuration: its `ν = 0.05` is the same, but its `dt = 0.004`, `steps = 40`, and
`η = 0.016` are not. The `L = 8` acceptance run costs hours, not seconds; `verification/README.md`
carries this harness as ⚠ *(offline)* with a `~4-9 h` estimate and the acceptance run pending.
`baseline.txt` is the same superseded capture.

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

All three readings above are `L = 5` readings. Whether they carry over to `L = 8` is untested here.

## Honest reading of the absolute C_d

The absolute `C_d` reported here is **not** an isolated-cylinder value. The DEC harness reports
`C_d ≈ 1.345`, and that number is measured at `Re = 100`. The difference is expected and disclaimed:

- **Reynolds mismatch** — the shipped constants put this case at `Re_D = 37.7`, not at the `Re = 100`
  of the DEC cross-reference. The two numbers therefore differ in Reynolds number as well as in
  domain and force definition. No published `C_d` at `Re ≈ 37.7` is cited here, so the DEC value is
  the only external number in the file and it describes a different flow.
- **~30 % blockage** — the cylinder spans a large fraction of the periodic box, so the effective drag
  is far above the unconfined value.
- **Penalization-integral force** — `F` counts the momentum sink over the whole *smoothed skirt*, not just
  pressure + friction on a sharp surface; with a 2-cell smoothing this inflates the magnitude.
- **Transient** — a periodic box has no momentum source to hold the free-stream, so `C_d` is read at a
  fixed horizon, not a true steady state.

So the committed DEC `C_d` is a **cross-reference**, not a target: the verification claim is the
*convergence trend* + no-slip + positivity, **not** the absolute number. Reproducing an absolute
isolated-cylinder `C_d` would need an inflow/outflow domain (the DEC solver's configuration) and a
retune of `ν`, `U`, or the radius onto the reference Reynolds number. Both are out of scope for the
periodic QTT solver as configured.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example qtt_cylinder_verification
```

The accuracy-vs-bond table, the two parameter ladders, the five gate lines, and the closing verdict are
on stdout; the finest-bond summary and the DEC cross-reference are on stderr. Exit is nonzero on a
broken gate. At the shipped `L = 8` this is an hours-long run, not a smoke test.

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The `FloatType` alias, the grid level `L`, the bond ladder and the two parameter ladders driven through `CfdFlow::march`. |
| `config.rs` | Case parameters, the cylinder mask, the `η` and smoothing ladders, and the `QttMarchConfig` body case builder. |
| `print_utils.rs` | The no-slip / drag measurement, the tables, and the five gates. |
| `baseline.txt` | A captured reference run (f64) at the superseded `L = 5` configuration. |

## Reference

- **Angot, P., Bruneau, C.-H. & Fabrie, P.** (1999). *A penalization method to take into account obstacles
  in incompressible viscous flows.* Numer. Math. **81**, 497–520 — the Brinkman volume-penalization method.
- **Peddinti et al.** (2024), Commun. Phys. **7**, 135 — MPS incompressible NS around immersed objects.
- DEC isolated-cylinder cross-reference: `verification/dec_cylinder_verification` (`C_d ≈ 1.345` at
  `Re = 100`). This QTT case runs at `Re_D = 37.7`, so the two are not the same flow.
