# QTT Park-2T plasma blackout (Tier-A)

Closes and verifies **Gap 2 (Tier-A)** of the plasma-blackout corridor: the Park two-temperature
reacting/ionization physics that turns the closed Gap-1 QTT flowfield into the flagship's regime driver
— recovery-temperature reconstruction → ionization → electron density → plasma frequency → comms/GNSS
blackout — driven by the **Lagging-Equilibrium Relaxation (LER)** between-step coupling hosted inside
the QTT march.

```bash
cargo run --release -p deep_causality_cfd --example qtt_park2t_blackout
```

## What it does

A blunt forebody is immersed in a periodic free-stream (Brinkman penalization, the Gap-1 solver). The
coupling is hosted in the QTT march loop (`QttMarchRun::run_coupled`, design D5/D8): each step publishes
a per-cell `"speed"` projection from the dequantized tensor-train state, transports the carried
ionization fraction as a tensor train (`advance_scalar`), and applies a statically-composed LER coupling

1. **`RecoveryTemperatureStage`** — rebuilds `T_tr = T_post − ½|u|²/c_p`, with `T_post` from a
   **mandatory Rankine–Hugoniot normal-shock jump** on the configured flight Mach (isentropic recovery
   alone is too cold to ionize).
2. **`IonizationStage`** — relaxes the carried ionization fraction `α` toward the Park-2T Saha surrogate
   `α_eq(T_tr)` with `τ_ion` grounded in the dominant associative-ionization rate (N + O → NO⁺ + e⁻),
   via the closed-form LER exponential, then writes `n_e = α · n_tot`.
3. **`EosStage`** — a two-temperature pressure closure (the interface Tier-B reuses; the incompressible
   ambient effect is limited).

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

Gate (ii) is the only `[reference]` gate here: the sub-stepped integration is a derivation separate
from `ler_step`, and the `1e-6` tolerance is sized from that reference's own truncation error
(`a²/2N ≈ 4.5e-8`), not from the measurement. It replaced an earlier "exponential exactness to
round-off" check that compared `ler_step` against a re-transcription of its own body, which no input
could make fail. Gate (iv) lost its former "`τ → 0` recovers Saha" conjunct for the same reason: that
conjunct exercised an explicit `return x_eq` early exit and held for every input. The two surviving
conjuncts are falsifiable — invert the Arrhenius activation exponent and the rate check fails; make
`ler_step` jump straight to equilibrium and the lag check fails.

## Tier-A disclaimers (honest scope)

- Rides the **incompressible** QTT rollout; `T_tr` is a **recovery-temperature reconstruction**
  (RH jump + `½|u|²/c_p`), **not** a true post-shock thermodynamic path.
- Saha equilibrium at the *frozen* RH post-shock temperature drives near-full ionization
  (`n_e ~ n_tot`) here; the two-temperature (`T_ve = T_e`) lumping over-predicts peak `n_e` by ~2×
  (Farbar–Boyd–Martin 2013), and real-gas dissociation caps the post-shock temperature lower — so the
  reported peak `n_e` is an over-prediction relative to the RAM-C II ~`1e19 m⁻³` anchor.
- The operator split is first-order Lie.

**No absolute coupled-CFD match is claimed.** Published values (RAM-C II / NASA TN; *Fluid Dynamics*
2022; Aiken–Carter–Boyd 2025 review; Park two-temperature tables; the Saha limit; Apollo blackout
dwell) are reported as **cross-references**. Tier-B (`add-cfd-compressible-qtt-marcher`) retires the
reconstruction with a real transported post-shock state.

See `baseline.txt` for the recorded reference output.
