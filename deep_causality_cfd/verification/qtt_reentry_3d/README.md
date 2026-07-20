# QTT 3-D re-entry forebody sheath — the Stage-6 rank lever (Tier-B)

The 3-D form of the `qtt_blunt_body_2d` rank lever, on the crate's serial `x`-`y`-`z` codec
(`quantize_3d`). The re-entry **forebody sheath** is a curved bow-shock surface standing off the
nose at a constant *physical* radius `R`. In a body-fitted spherical coordinate (radial axis `ζ`
across the shock) that surface is a step in `ζ` — a function of one axis — so its bond `χ` is
`O(1)` and resolution-independent. Sampled on a Cartesian `2^l × 2^l × 2^l` lattice the identical
physical shell is curved, so `χ` grows with resolution (the `qtt_rank_3d` upper bound).

```bash
cargo run --release -p deep_causality_cfd --example qtt_reentry_3d
```

## What it does

A smoothed step at the standoff radius `R = 1.5` is sampled three ways and quantized (tolerance
`1e-8`), and `max_bond()` is read off over a `2^3 → 2^5` ladder:

- **fitted** — the shell as a function of the radial index alone (aligned with one axis);
- **Cartesian** — the same physical shell on a `[−2, 2]³` lattice (curved on the grid);
- **wake** — two off-axis lobes downstream of the body, a multi-feature structure no single
  fitted coordinate aligns.

## Scope (design D9)

The **forebody** is in scope and gated. The **wake** is out of scope: a separated, unsteady wake
needs turbulence, and its bond is reported only as a datapoint for the standing `qtt_rank_3d`
research question. The dynamic *marched* forebody rank is likewise reported — there is no 3-D
body-fit metric yet, so the marcher runs Cartesian.

## What it verifies (exit nonzero on break)

- **RE-A** — the fitted forebody `χ` stays bounded (`≤ 8`) and its high-resolution tail is flat
  (the last refinement adds at most 1).
- **RE-B** — the Cartesian capture `χ` grows with resolution *and* overtakes the fitted bond by
  at least 2×.

Both gates are **structural**: they bound *rank*, not physical accuracy.

## Measured (f64, 2^3–2^5, ~3 s)

| resolution | fitted `χ` (fn ζ) | Cartesian `χ` |
|---|---|---|
| 2^3 | 2 | 10 |
| 2^4 | 4 | 30 |
| 2^5 | 4 | 59 |

Fitted `2 → 4` (plateaus) against a capture cost of `10 → 59` (growing) — both gates **PASS**.

## Reported, not gated

- **wake** — `χ = 41` at `2^5`, comparable to the Cartesian capture and un-fittable by
  construction. Out of scope, never gated.
- **dynamic marched forebody** — the Cartesian 3-D marcher grows `χ` to **16** over 6 steps. A
  3-D body-fit metric plus re-pinning is the open remainder.

See `baseline.txt` for the recorded reference output.
