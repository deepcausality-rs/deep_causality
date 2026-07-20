<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `srp_momentum_jet` — does a momentum-carrying jet recover the Jarvinen–Adams collapse?

```bash
cargo run --release -p deep_causality_cfd --example srp_momentum_jet
SRP_MJ_L=6 SRP_MJ_SWEEP=1.0,2.0,4.0 cargo run --release -p deep_causality_cfd --example srp_momentum_jet
```

**What it tests.** The imprint-fidelity follow-up (risk 1) to the de-risk verification
`srp_drag_decrement` and its recorded amber finding. That verification's whole-envelope
**pinned-state** plume shields the forebody monotonically but cannot produce the Jarvinen–Adams
central-nozzle drag collapse. The verdict named "a momentum-carrying jet interaction rather than a
pinned obstruction state" as the first upgrade path, and this study measures exactly that variant
on the **same harness**: same freestream, body, sponge, strip, and at defaults the same grid and
bond cap, with the plume envelope pin replaced by a nozzle-exit **patch** at the body face pinned
to a supersonic upstream-firing exit state. The plume is not imposed. It forms, spreads, and
interacts in the marched field.

The question is an attribution. Is the missing collapse a property of the **model class**, the
static imprint, or of the **harness**, meaning the 2-D plane, coarse grid, and dissipation floor
`ν = ½·s_ref·Δx`?

**Instrumentation.** The de-risk adversarial review pre-registered the evidence bar:

- **time-averaged tail read**: strip force sampled every step over the tail window, with mean, std,
  and a first-half/second-half drift witness. The committed harness contracted a single terminal
  snapshot;
- **three strip bands**: the verification's full strip verbatim, an **annulus** excluding the jet's
  rows (the J–A aeroshell surface, free of the exit pin), and an **outer** band one further cell
  out, since a collapse must appear off-axis to count;
- **mechanism witnesses**: centerline interface location, an upstream freestream probe, global min
  ρ̂/p̂ floor monitors, and the realized injected momentum flux audited against the analytic pin;
- **robustness dials**: `SRP_MJ_L`, `SRP_MJ_CAP`, `SRP_MJ_STEPS`, `SRP_MJ_TAIL`, `SRP_MJ_SWEEP`.

**Findings.** Grid 2⁵ × 2⁵ over 4 m, dt 8e-4, 2000 steps with the tail mean over the last 500, bond
cap 24.

| C_T | p_e/p_∞ | frac full | frac annulus | frac outer | J–A | drift % | iface x |
|---|---|---|---|---|---|---|---|
| 0.25 | 0.23 | 1.055 | 1.031 | 1.021 | 0.576 | +0.01 | 0.500 |
| 1.00 | 0.67 | 1.588 | 1.413 | 1.339 | 0.124 | +0.02 | 0.500 |
| 2.00 | 1.25 | 2.282 | 1.928 | 1.774 | −0.031 | +0.02 | 0.531 |
| 4.00 | 2.43 | 3.456 | 2.819 | 2.531 | −0.093 | −0.03 | 0.500 |
| 8.00 | 4.78 | 4.592 | 3.614 | 3.150 | −0.125 | −0.14 | 0.469 |

- **No collapse, and the attribution moves from the model class to the harness.** With the plume
  formed dynamically, tail-averaged reads at a drift of 0.02 % or less show monotone drag
  **augmentation**. The annulus fraction runs 1.03 to 3.61 across C_T 0.25 to 8, against a J–A
  reference that goes negative. The total-axial-force dip is absent.
- **The mechanism is the dissipation floor.** A jet-cell Péclet of roughly 1.3 to 1.8 freezes the
  stagnation interface at the face, x = 0.469 to 0.531 across a **32× thrust range**, so injected
  momentum reads as face pressure. That is the inverse of the J–A blanketing reorganization.
- **Compression is innocent.** Raising the bond cap 24 to 32, exact at 2⁵, leaves every observable
  unchanged at displayed precision. The discretization is the limit, not the tensor-train truncation.
- **The domain is a limit too.** The upstream probe leaves freestream from C_T ≈ 0.25 at +6.5 % to
  +285 % at C_T 8, so the correlation's own transition variable, `p_e/p∞ ≈ 7`, is unreachable on
  this harness.

**Conclusion.** Neither coupling model can host the collapse on this harness, so the **A0
correlation channel keeps the drag authority**. The J–A miss is the recorded finding, not a
regression; the structural bands gate the default configuration only, and
`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md` with its addendum is the authority. This
study supersedes the reverted `verification/srp_drag_decrement/` pinned-envelope harness (see
`reverted/README.md`).

**Caveats.** Carried unchanged from the verification: a 2-D plane rather than axisymmetric, a
smoothed-mask body, periodic and sponge blockage, and a single marched γ. The J–A correlation is
axisymmetric, so the quantitative fractions do not transfer. The collapse *structure* is what is
being measured, and J–A is the structural reference, not a quantitative fit target. `C_T` is the
declared 2-D per-depth definition (`config.rs`).

See `output.txt` for the recorded reference output.
