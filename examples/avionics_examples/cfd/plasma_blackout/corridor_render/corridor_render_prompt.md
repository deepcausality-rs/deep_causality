# Physics-simulation prompt — plasma-blackout corridor, the onset fork

A render brief for the counterfactual fork in
`examples/avionics_examples/cfd/plasma_blackout/corridor`. Every number below is traced to `output.txt`,
`corridor_branches.csv`, `corridor/constants.rs` (the sweep: `BRANCH_STEPS`, `BANK_ANGLES_DEG`,
`AIM_CROSS_RANGE_M`), or `avionics_examples::shared::constants` at
`examples/avionics_examples/src/shared/constants.rs` (the vehicle, atmosphere, flight physics, navigation and
envelope; cited below as `shared::constants::NAME`). Nothing is invented; the one reconstruction is
derived and verified in §7, and the non-physical framing choices are named in §9.

---

## 1. The instant being simulated

The march paused at **coupled step 131** on a flow-resolved event, not a scheduled station:
the evolved electron density crossed the GPS L1 cutoff and the classifier flipped the link to DENIED. That paused state
is forked 17 ways.

| Quantity                    | Value                     | Source              |
|-----------------------------|---------------------------|---------------------|
| Altitude                    | 73.2 km                   | `output.txt`, leg 1 |
| Mach                        | 27.2                      | `output.txt`        |
| Knudsen                     | 1.27e-2 → **slip** regime | `output.txt`        |
| Peak electron density `n_e` | 3.270e16 m⁻³              | `output.txt`        |
| Plasma frequency `ω_p`      | 1.020e10 rad/s            | `output.txt`        |
| Stagnation heat flux `q`    | 1.027e6 W/m²              | `output.txt`        |
| Axial load                  | 0.4 g                     | `output.txt`        |
| Nav error vs truth          | 0.1823 m                  | `output.txt`        |
| Link state                  | **GNSS DENIED**           | `output.txt`        |

The blackout is *marginal here*. The L1 critical density is

    n_crit = ε₀ mₑ ω² / e² = 3.080e16 m⁻³   (ω = shared::constants::COMMS_BAND_RAD_S = 9.899e9 rad/s)

so `n_e / n_crit = 1.062`. The sheath cuts the link by **6 %**. The render must not show a blaze: at this station the
sheath is thin and marginal. The 61 km peak passage is 796× denser (2.600e19 m⁻³) and is a different picture entirely.

## 2. The vehicle

The flow, chemistry and Knudsen number in this example are all driven by RAM-C II probe scales, so that is the body to
model.

| Parameter                      | Value                            | Source                             |
|--------------------------------|----------------------------------|------------------------------------|
| Nose radius `R_n`              | 0.1524 m (the 6-inch hemisphere) | `shared::constants::NOSE_RADIUS_M` |
| Forebody characteristic length | 0.30 m                           | `shared::constants::L_CHAR`        |
| Cone half-angle                | 9°                               | RAM-C II configuration             |
| Body length                    | 1.30 m                           | RAM-C II configuration             |
| Sphere/cone tangency           | x = 0.1286 m, r = 0.1505 m       | derived                            |
| Base radius                    | 0.3361 m (0.672 m diameter)      | derived                            |

**Flag.** `CDA_OVER_M`, `VEHICLE_MASS_KG` and `VEHICLE_CD` (all in `shared::constants`) describe a *different* body — a 4.23 m lifting aeroshell at
β ≈ 172 kg/m². The corridor's Sutton-Graves heating runs on the 0.1524 m RAM-C nose while its trajectory runs on the
4.23 m capsule's ballistic bundle. That is a documented modelling seam, not a render decision; the render takes the
probe because the probe is what the plasma physics describes.

## 3. Freestream

US-1976-shaped table pinned to the RAM-C 61 km freestream (`shared::constants::ATMOSPHERE`). At 73.2 km, between the 75
km and 61 km rows: `n ≈ 3.6e20 m⁻³`, `T_tr ≈ 210 K`. Flight speed `V = M·√(γRT) = 27.2 × 290.5 = 7.90 km/s`.

## 4. Shock layer

Strong normal shock on the effective-gamma closure, `shared::constants::GAMMA_EFF = 1.1`:

    ρ₂/ρ₁ = (γ+1)/(γ−1) = 21.0
    Δ ≈ 0.78 · R_n · (ρ₁/ρ₂) = 5.66 mm      (Δ/R_n = 0.037)

So the stagnation standoff is **5.7 mm on a 152 mm nose** — the layer is a skin, not a halo. Down the flank it opens
to ≈ 37 mm at the base, on a ≈ 10.5° cone shock.

Two emission mechanisms, and they must not be conflated:

1. **The wall.** Radiative equilibrium at the logged heat flux,
   `T_w = (q/εσ)^¼ = 2148 K` at the stagnation point with ε = 0.85. Lees' distribution
   `q/q_stag ≈ cos θ = n̂·(−v̂)` carries it back, giving **1351 K on the cone flank**
   (the cone sees `sin 9° = 0.156` of stagnation flux). This is a blackbody gradient:
   yellow-white at the nose, deep orange down the flank.
2. **The gas.** The shock layer itself is optically thin. Head-on it is 6 mm of path; grazing the silhouette it is tens
   of centimetres. The render should therefore show a bright **rim** at the body's edge and almost nothing across the
   face. That limb brightening is the real signature of a thin shock layer, and getting it wrong in the optically-thick
   direction is what makes a reentry render look like a lamp.

The wake carries the recombining plasma the RAM-C reflectometers actually sampled (dissociative recombination
`NO⁺ + e⁻ → N + O`), cooler and dimmer than the shock layer.

## 5. Navigation state

Link DENIED; the 17-state ESKF is dead-reckoning on a tactical-grade accelerometer bias
(`shared::constants::IMU_ACCEL_BIAS = [2.0e-2, −1.4e-2, 1.0e-2]` m/s²), error growing as t². At the fork the error is 0.1823 m; by the 61
km passage it is 1.5637 m.

## 6. The fork

`run_until` pauses the march; the paused state is forked in O (1) by copy-on-write, once per candidate bank command.
Each branch flies **BRANCH_STEPS = 100** steps (`corridor/constants.rs`) at
`shared::constants::DT_FLIGHT = 0.1 s` → a **10.0 s dwell**, i.e. 79.0 km of true downrange.

* Coarse round: 0, 5, 10, 15, 20, 40 deg (`corridor/constants.rs`, `BANK_ANGLES_DEG`)
* Fine round: 11 candidates at 0.5 deg spacing, 7.5 → 12.5 deg
* **17 branches total**, each in its own alternated world (`!!ContextAlternation!!`)
* The 40 deg command exceeds `shared::constants::MAX_BANK_RAD = 0.5` (28.648 deg). `CyberneticCorrect`
  clamps it every step, so it flies at 28.648 deg and overshoots to a 28.881 m miss. The render should show both the
  clamped path and, distinctly, where the unclamped command would have gone — the gate is the difference between them.
* Aim point: the ballistic terminal offset `AIM_CROSS_RANGE_M = 20.0` m cross-range (`corridor/constants.rs`).
* Committed branch: **11.5 deg, miss 2.0699 m** (minimum trajectory-derived miss).

## 7. The reachable set — derived, then verified

`BankSteeredLift` rotates a constant-magnitude lift vector about the velocity by the bank angle. Over a fixed dwell, the
resulting terminal displacement therefore sweeps a **circular arc**, not an arbitrary curve:

    terminal(φ) − ballistic = R·[ sin φ · ĉ − (1 − cos φ) · n̂ ]

with `ĉ` the cross-range direction the aim sits in and `n̂` the in-plane lift-up direction. Fitting the single parameter
`R` against all 17 logged miss distances gives

    R = 96.667 m,   max|residual| = 3.1 mm,   rms = 1.5 mm

One parameter reproduces seventeen logged numbers to millimetres, which is what makes the ribbon geometry defensible
rather than decorative. Consequences worth rendering:

* The reachable set over the dwell is an arc of **48.33 m** length (`R × 0.5 rad`) — the envelope cap is a hard end-stop
  on the arc, and the 40 deg branch sits at that stop.
* The arc's true closest approach to the aim is **2.047 m at 11.689 deg**. The fine sweep's committed 11.5 deg lands at
  2.073 m — within **25 mm** of the achievable optimum. The sweep did not stop early; it stopped at the curve.

## 8. Acceptance criteria for the render

1. Sheath reads as marginal, not saturated (`n_e/n_crit = 1.062`).
2. Stagnation point visibly hotter than the flank, at the 2148 K / 1351 K ratio.
3. Shock layer bright at the silhouette, near-transparent across the face.
4. 17 branches, ordered by miss, the committed 11.5 deg unmistakable.
5. The clamped 40 deg branch and its refused unclamped command both present.
6. Terminals lie on one circular arc; the aim point sits off that arc, not on it.
7. Earth limb consistent with 73.2 km: horizon 965.8 km away, dip angle 8.65 deg.

## 9. What in the frame is *not* physical

* **Downrange compression 500 : 1.** True dwell downrange is 79.0 km against at most 46.3 m of cross-range. Rendered 1:1
  the fan is a straight line 0.034 deg wide. Downrange is compressed to 158.0 m; **cross-range and the lift-plane drop
  are true metres.** Change the one constant `COMP` to change the exaggeration.
* Star field is procedural. Earth surface is procedural noise, not imagery.
* Branch ribbon thickness encodes round (coarse / fine), not any physical quantity.
* Emission strengths are tone-mapped for AgX; ratios between surfaces follow the physics, absolute values do not.

## 10. Two stale claims in `corridor/README.md`

Checked against the committed `output.txt`:

* README says onset at **74.7 km** and the committed branch at **13.5 deg / 2.39 m**. The committed run says onset
  **73.2 km**, committed **11.5 deg / 2.07 m**, coarse winner 10 deg / 3.53 m.
* README's "2.39 m is that curve's closest approach" is now **2.047 m** (§7).

The gates pass either way because they are banded, so the prose drifted without the run failing.
