# QTT Park-2T plasma blackout (Tier-A)

This example closes and verifies **Gap 2 (Tier-A)** of the plasma-blackout corridor: the Park
two-temperature reacting/ionization physics that turns the Gap-1 QTT flowfield into the corridor's
regime driver (recovery-temperature reconstruction → ionization → electron density → plasma
frequency → comms/GNSS blackout). The **Lagging-Equilibrium Relaxation (LER)** between-step coupling,
hosted inside the QTT march, drives the chain.

```bash
cargo run --release -p deep_causality_cfd --example qtt_park2t_blackout
```

## What it does

A blunt forebody is immersed in a periodic free-stream (Brinkman penalization, the Gap-1 solver). The
QTT march loop hosts the coupling (`QttMarchRun::run_coupled`, design D5/D8). Each step publishes
a per-cell `"speed"` projection from the dequantized tensor-train state, transports the carried
ionization fraction as a tensor train (`advance_scalar`), and applies a statically composed LER coupling:

1. **`RecoveryTemperatureStage`** rebuilds `T_tr = T_post − ½|u|²/c_p`, with `T_post` from a
   **mandatory Rankine–Hugoniot normal-shock jump** on the configured flight Mach (isentropic recovery
   alone is too cold to ionize).
2. **`IonizationStage`** relaxes the carried ionization fraction `α` toward the Park-2T Saha surrogate
   `α_eq(T_tr)` through the closed-form LER exponential, with `τ_ion` grounded in the dominant
   associative-ionization rate (N + O → NO⁺ + e⁻), then writes `n_e = α · n_tot`.
3. **`EosStage`** applies a two-temperature pressure closure (the interface Tier-B reuses; its effect
   on the incompressible ambient is limited).

A `BlackoutTrigger` maps the peak electron density to the plasma frequency and compares it to the
configured comms band, raising the GNSS/comms-denied flag and accumulating the blackout dwell.

## What it verifies (exit nonzero on break)

The six LER acceptance gates, each printed with its evidence class (`verification/README.md`
defines the convention):

| Gate | Class | Criterion |
|---|---|---|
| (i)   | `[tripwire]` | **Stability at stiffness** — at `τ = Δt/1000` the relaxation stays monotone inside `[x, x_eq]` over 50 steps and settles to within 1.0 of `x_eq`, where a single explicit Euler rate step overshoots past `100·x_eq` |
| (ii)  | `[reference]` | **Relaxation kernel vs an independent reference** — `ler_step` agrees with a 10⁶-substep forward-Euler integration of `dx/dt = (x_eq − x)/τ` to `1e-6` relative |
| (iii) | `[tripwire]` | **Rankine–Hugoniot band** — `T_post` at `M = 25` lands inside `(1e4, 1e5) K`, not the cold isentropic value |
| (iv)  | `[tripwire]` | **Ionization lag real, rate grounded in `T`** — the associative-ionization Arrhenius rate is higher at 9000 K than at 6000 K, and a short step leaves `α` strictly below `α_eq` |
| (v)   | `[tripwire]` | **Counterfactual path-dependence** — two temperature histories reaching the same target carry different ionization (the LER memory) |
| (vi)  | `[tripwire]` | **Ionized species present** — the marched electron density is strictly positive |

Gate (ii) is the only `[reference]` gate: the sub-stepped integration is a derivation separate
from `ler_step`, and the `1e-6` tolerance is sized from that reference's own truncation error
(`a²/2N ≈ 4.5e-8`), not from the measurement. Both conjuncts of gate (iv) are falsifiable: invert
the Arrhenius activation exponent and the rate check fails; make `ler_step` jump straight to
equilibrium and the lag check fails. Neither gate compares `ler_step` with a copy of its own body
or with its `τ → 0` early exit, checks that would hold for every input.

## Tier-A disclaimers (honest scope)

- The case rides the **incompressible** QTT rollout. `T_tr` is a **recovery-temperature
  reconstruction** (RH jump + `½|u|²/c_p`), **not** a true post-shock thermodynamic path.
- Saha equilibrium at the *frozen* RH post-shock temperature drives near-full ionization
  (`n_e ~ n_tot`) here; the two-temperature (`T_ve = T_e`) lumping over-predicts peak `n_e` by ~2×
  (Farbar–Boyd–Martin 2013), and real-gas dissociation caps the post-shock temperature lower, so the
  reported peak `n_e` over-predicts relative to the RAM-C II ~`1e19 m⁻³` anchor.
- The operator split is first-order Lie.

**No absolute coupled-CFD match is claimed.** Published values (RAM-C II / NASA TN; *Fluid Dynamics*
2022; Aiken–Carter–Boyd 2025 review; Park two-temperature tables; the Saha limit; Apollo blackout
dwell) appear as **cross-references**. Tier-B (`add-cfd-compressible-qtt-marcher`) replaces the
reconstruction with a transported post-shock state.

See `baseline.txt` for the recorded reference output.
