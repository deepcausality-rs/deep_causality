# QTT Sod shock tube (Tier-B Stage 2)

The **compressible-flux gate**: marches the 1-D conservative compressible Euler equations in
quantized-tensor-train form and verifies them against the **exact Riemann solution** on the classic Sod
shock tube.

```bash
cargo run --release -p deep_causality_cfd --example qtt_sod
```

## What it does

`CompressibleEuler1d` carries the conservative state `U = (ρ, ρu, ρE)` as three tensor trains and
marches a **global Lax–Friedrichs** update: Rusanov with a global wave-speed estimate. The
dissipation coefficient `s_max` is one maximum over the whole domain, not a per-face maximum over the
two states adjacent to a face, so the coefficient is uniform in space instead of varying face by
face. The update rearranges to a **conservative central flux difference plus a scalar artificial
viscosity**,

```text
dU/dt = −∂ₓF(U) + ½·s_max·Δx·∂²ₓU,   s_max = max over all cells of (|u| + c),
```

so it is assembled from the §0 `gradient` / `laplacian` MPOs (conservative, telescoping) applied to the
flux and the state, recompressed each step. The ideal-gas EOS `p = (γ−1)(E − ½ρu²)` and the flux are
evaluated pointwise (dequantize → compute → requantize); the rank-preserving TT-cross
(`apply_nonlinear`) form is the large-`L` upgrade. A per-face local wave speed, which is what makes a
scheme local Lax–Friedrichs, is the other named upgrade; it is not implemented, and the harness is
gated in its global form.

The Sod initial data (`ρ,u,p`: `1,0,1` | `0.125,0,0.1`, `γ=1.4`) is marched to `t = 0.2` on the wide
domain `[−1,1]` (so the periodic boundary-jump waves stay outside the measurement window `|x| ≤ 0.5`)
and compared to the exact Riemann solution.

## What it verifies (exit nonzero on break)

The L1 error of density / velocity / pressure against the **exact Riemann solution** over `|x| ≤ 0.5`,
within a recorded tolerance (`0.03`). The first-order global Lax–Friedrichs update smears the contact,
so the bound is on mean accuracy; the star pressure `p* = 0.3031` (the canonical Sod value), the left
expansion fan, the contact, and the right shock are all at the correct positions/speeds.

The companion unit tests (`tests/solvers/qtt/compressible_tests.rs`) gate **conservation** (`∫ρ`, `∫ρu`,
`∫ρE` preserved to the rounding floor on a periodic smooth state), **free-stream preservation** (a
uniform state is a fixed point), and the **EOS**.

See `baseline.txt` for the recorded reference output.
