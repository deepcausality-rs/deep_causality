# QTT blunt-body bow shock — the Stage-5 rank lever (Tier-B)

The **rank-lever gate**. A blunt-body bow shock stands off the nose at a constant *physical* radius
`R`. In a body-fitted coordinate that surface is a line `η = const`, a step in one axis, so its
quantized-tensor-train bond `χ` is `O(10)` and resolution-independent. Sampled on a Cartesian
lattice the identical physical shock is curved on the grid, so `χ` grows with resolution.
Body-fittedness buys the bond reduction, and that lever is what this gate pins.

```bash
cargo run --release -p deep_causality_cfd --example qtt_blunt_body_2d
```

## What it does

`BlendedMap` carries both coordinates as one blend parameter. At `λ = 1` it is the body-fitted
polar fan, `r ∈ [1, 2]` over a ±45° fan in front of the nose; at `λ = 0` it is the
Cartesian-capture rectangle. A smoothed compression (`ρ: 1 → 1.8`, `p: 1 → 3`, `γ = 1.4`) is placed
at the standoff radius `R = 1.5` **in physical space** and sampled through `map.position`, so both
coordinates see the same shock. Each sampled density field is quantized by `quantize_2d` at
tolerance `1e-8`, and its `max_bond()` is read off over a `2^5 → 2^7` ladder.

The marcher (`CompressibleMarcher2d`) runs the **same solver** over both coordinates through the
`MetricProvider` seam (design D8). This is therefore a one-solver comparison, where the coordinate
is the only variable.

## What it verifies (exit nonzero on break)

- **BB-A**: the fitted `χ` stays bounded at 12 or below and resolution-stable, growing by at most 1
  per refinement, so no `√side` growth.
- **BB-B**: the Cartesian capture `χ` grows with resolution and overtakes the fitted bond by at
  least 2×.

Both gates are **structural**. They bound *rank*, not physical accuracy. The quantitative accuracy
gate for the compressible solver is `qtt_sod`, against the exact Riemann solution.

## Measured (f64, 2^5–2^7, ~2 s)

| resolution | fitted `χ` (λ=1) | capture `χ` (λ=0) |
|---|---|---|
| 2^5 | 3 | 16 |
| 2^6 | 4 | 32 |
| 2^7 | 5 | 61 |

Fitted runs 3 to 5 and is flat; the capture cost runs 16 to 61, growing roughly as `√side`. Both
gates **PASS**.

## Reported, not gated

The **dynamic marched** rank is an open remainder. A plain flux-through-front marcher injects
angular structure across the captured front and grows `χ` to **64** over 6 steps, even in the
fitted coordinate. Bounding the marched `χ` needs re-pinning plus an exact-RH interface, meaning
smooth on each side with no flux marched across the front. That is design D9 and the
`qtt_repin_marcher` study. This example prints the datapoint and never asserts on it.

The 3-D form of the same lever is `qtt_reentry_3d`.

See `baseline.txt` for the recorded reference output.
