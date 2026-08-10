# QTT 2-D Taylor–Green — quantized-tensor-train incompressible solver

Verifies the `QttIncompressible2d` solver — a 2-D incompressible Navier–Stokes flowfield that lives
in, and evolves entirely as, a **tensor train** (the CFD ↔ tensor-network bridge) — against the
closed-form 2-D **Taylor–Green vortex** (Taylor & Green, 1937), the standard analytic reference for a
periodic incompressible solver. The whole rollout is driven through the **CfdFlow** DSL
(`CfdFlow::march`), so this also exercises the new DSL wiring and observable extraction.

## The reference

On a periodic `[0, 2π]²` box the single-mode Taylor–Green field

```
u = −cos(x) sin(y)
v =  sin(x) cos(y)
```

is an exact solution whose amplitude decays as `e^{-2νt}`: every velocity component is an eigenfunction
of the Laplacian (eigenvalue −2), and the nonlinear convection `u·∇u = ∇[¼(cos2x + cos2y)]` is a **pure
gradient**, absorbed entirely by the pressure. So the incompressible dynamics reduces to pure diffusion
and the decay rate is analytic.

## What is verified (4 gates, exit nonzero on break)

1. **Convergence to the published reference.** A `2^L × 2^L` refinement ladder (`L = 3, 4, 5` → 8², 16²,
   32²) marches the seed to `t = 0.2` and compares the final field to `e^{-2νt}` × the analytic vortex.
   The max-norm error must **strictly decrease under refinement** to within a pinned bound, at the
   expected **~2nd order in space** (centered finite differences + the spectral projection). Time
   integration is first-order (explicit Euler) and `dt` is held fixed across the ladder, so the
   measured order is the spatial one.

2. **Correct nonlinear convection.** This is the subtle one. Because single-mode Taylor–Green's
   convective term is a *pure gradient the projection removes*, the marched decay is **insensitive to
   whether convection is computed correctly** — a solver with a broken (or zero) `u·∇u` would still pass
   gate 1. So the nonlinear term is checked **directly**, through the **shipped** convection path: the
   harness calls the solver's own `rate_pair` on a zero-viscosity instance, which returns the pure
   convection `−(u·∇)u`, and that must reproduce the closed form `−½ sin(2x)` with a small error and a
   **non-zero** amplitude. Calling the shipped path matters: a re-assembled copy of the operator would
   pass even if the marcher's own convection were wrong.

3. **Incompressibility.** The post-projection divergence residual stays at the projection floor.

4. **MPS compression.** The headline tensor-network metric — maximum bond dimension vs. dense element
   count — is reported per level.

## Measured (f64, Apple M3 Max, release, <1 s)

```
Convergence: refinement ladder vs the analytic e^(-2 nu t) decay
  N =   8   max_err = 9.789e-4   l2_err = 4.894e-4   order =   --    bond =   8   divergence = 1.06e-14
  N =  16   max_err = 2.411e-4   l2_err = 1.206e-4   order = 2.02   bond =  16   divergence = 1.45e-14
  N =  32   max_err = 5.316e-5   l2_err = 2.658e-5   order = 2.18   bond =  32   divergence = 4.14e-14
  observed order = 2.18 (2nd-order in space: centered FD + spectral projection; 1st-order in time: explicit Euler at fixed dt)
```

- **Convergence:** error `9.8e-4 → 2.4e-4 → 5.3e-5`, observed order **2.02 → 2.18**: 2nd-order in space
  (centered FD + spectral projection), first-order in time (explicit Euler at fixed `dt`), so the
  ladder measures the spatial order. Finest default-grid (32²) max error **5.3e-5**.
- **Temporal floor, and the ladder's maximum usable length.** Because `dt` is held fixed, the
  first-order Euler error is a floor under the total error, and it has the **opposite sign** to the
  spatial error. The signed error of the decay factor runs `+9.8e-4, +2.4e-4, +5.3e-5, +5.9e-6,
  −5.9e-6, −8.8e-6` for `N = 8 … 256`: the two contributions **cancel** near `N = 64–128`. So the
  `N = 64` observed order of **3.16 is a cancellation artifact, not super-convergence**, and by
  `N = 128` the order collapses to **0.02**. **Use `max_level = 5` (the committed default);** at the
  documented `max_level 7` the finest-pair order falls below `MIN_ORDER` and the harness **fails**.
  Making the extended ladder honest needs a two-sided order gate or `dt` refined as `dt ∝ dx²`.
- **Convection:** `u·∇u` vs `−½ sin(2x)` — max abs error **3.2e-3** at 32² (the centered-difference
  truncation of `∂ₓ` on the field), signal amplitude 0.5 → the nonlinear term is **real and correct**.
- **Divergence:** **~1e-14** at every level — the spectral Leray projection is exact to machine
  precision, not merely to an iterative tolerance.
- **Compression:** bond `= N` here (a smooth low-frequency field), so storage is `O(N)` vs the dense
  `O(N²)` — the `N×` compression that grows with resolution.

## Why these checks, and their honest limits

The Taylor–Green vortex is the *correctness* anchor: a closed-form solution lets gate 1 measure true
discretization error and its convergence order, the gold standard of code verification. Gate 2 closes the
gap that single-mode TG leaves — convection masking — so a passing run genuinely exercises the nonlinear
term, not just diffusion + projection.

Limits, stated plainly: this is a **periodic, smooth, low-Reynolds, single-mode** case. It does **not**
test immersed-body boundary conditions (not yet encoded in QTT), turbulent rank growth, or multi-mode
energy cascade. Those are the next verification anchors once the immersed-body QTT encoding lands; the
rank-vs-accuracy curve of Peddinti et al. / Gourianov et al. is the headline metric to reproduce there.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example qtt_taylor_green_verification [max_level]
```

`max_level` (default 5) extends the ladder to a `2^max_level` grid. The labeled report and the closing
verdict are on stdout; any gate's `FAIL:` line is on stderr and the process exits nonzero the moment a
gate breaks.

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The `FloatType` alias, the refinement ladder driven through `CfdFlow::march`, the convection-operator check (through the shipped `rate_pair`), and the call into the self-verify gate. |
| `config.rs` | Case parameters (`NU`, `DT`, `STEPS`, `MAX_BOND`), the `QttMarchConfigBuilder` case builder, and the analytic-field helpers (`ft`, `spacing`, `decay`, `tg_u`, `tg_v`, `trunc`). |
| `print_utils.rs` | The analytic-reference error and compression measurement (`measure`), the rendered ladder and summary, and the published-reference gates (`verify`). |
| `baseline.txt` | A captured reference run (f64). |

## Precision as a parameter

Change `FloatType` in `main.rs` (`f64` → `f32` or `Float106`) and the whole computation — seed,
quantization, every per-step round, the projection, and the error metrics — re-runs at that precision;
the exact `f64` specifications enter once through `config::ft` (`from_f64`) and never come back down.

## Reference

- **Taylor, G. I. & Green, A. E.** (1937). *Mechanism of the production of small eddies from large ones.*
  Proc. R. Soc. Lond. A **158**, 499–521.
- **Peddinti, R. D., Pisoni, S., Marini, A., Lott, P., Argentieri, H., Tiunov, E. & Aolita, L.** (2024).
  *A quantum-inspired framework for computational fluid dynamics.* Commun. Phys. **7**, 135 — the
  MPS-encoded incompressible-NS method this solver follows.
- **Gourianov, N. et al.** (2022). *A quantum-inspired approach to exploit turbulence structures.* Nat.
  Comput. Sci. **2** — the original MPS-CFD demonstration and the rank-vs-accuracy metric.
