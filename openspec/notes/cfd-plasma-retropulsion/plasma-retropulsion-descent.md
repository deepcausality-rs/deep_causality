<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Plasma-Retropulsion Descent — from blackout exit through ignition to touchdown

**What this is.** A design note for the third plasma-blackout example. The corridor example today
ends at 47.6 km and Mach 8.3, thirty steps after reacquisition, with the descent's hardest guidance
act still ahead of it. This note investigates continuing that same descent through the
retropulsion-ignition threshold, a second regime cascade from supersonic through transonic to
subsonic under thrust, the guidance/navigation/control of the burn itself, and a touchdown gate;
and it wires the weather-dispersion table in as an onboard artifact that adjusts the trajectory
from the measured temperature of the day. The centerpiece is a counterfactual study that forks the
marched, plume-coupled *state* rather than the plume's *parameters*; §6 pins that distinction
down, because only one of the two actually tests the waters.

Honesty convention (as elsewhere): **[holds]**, **[holds under precondition]**, **[measured]**,
**[open]**, **[speculative]**. Status of this note: investigated and designed; nothing built.

---

## 1. Where the corridor stops, and what sits just below it

The measured terminal state of the corridor run (`plasma_blackout/corridor/output.txt`):

| Witness | Value |
|---|---|
| Altitude / Mach after reacquisition | 47.6 km, Mach 8.3 |
| Navigation error after the first folded fixes | 0.28 m (variance 3.2e-1 m²) |
| Regime | continuum (Kn = 2.7e-4), GNSS available |
| Provenance | 13 entries, 0 envelope boundings |
| Wall-clock | 42 s of the 600 s budget |

Two facts make continuing downward the natural next move rather than an arbitrary "more":

1. **The physics below 47 km is the flight-proven frontier.** NASA's SCIFLI team imaged the
   Falcon 9 first-stage entry burns precisely because the 70–40 km reentry-burn window is "powered
   flight through the Mars-relevant retropulsion regime". Supersonic retropropulsion is the
   pacing technology for high-mass Mars entry, and the flight data that exists comes from the
   altitude band where our corridor run ends. **[holds]**
2. **The three examples close a loop.** The corridor flies one descent. The weather example is the
   digital-twin table factory for that descent. A retropulsion example that *consumes* the weather
   table in flight and carries the descent to the ground turns the pair into a chain: generate the
   table, load the table, fly the table. The certification-by-analysis story the weather README
   tells becomes executable end to end.

The corridor's own vehicle makes the extension honest. Its deliberately light ballistic bundle
(`CDA_OVER_M = 5.8e-3`, β ≈ 170 kg/m²) decelerates hard below the blackout exit: by the low-30-km
band the probe is passing through Mach ≈ 2 and falling. That deceleration serves the design:
**Mach 0.4–2.0 is exactly the envelope of the Jarvinen–Adams retropropulsion dataset**
(NASA CR, 1970: conical aeroshells with central and peripheral retrorockets, thrust coefficients
to C_T = 30, angles of attack to 18°). The ignition threshold the vehicle descends into is the
band where the SRP physics is anchored, so Tier A never has to leave validated territory.
**[holds under precondition: trajectory numbers re-derived at build time; the Mach-2-by-33-km
figure is a ballistic estimate, not yet a run witness]**

---

## 2. The extended narrative, in five acts

```text
[0] PLAN      measured day temperature dT → interpolate weather_table.csv →
              deorbit/entry targeting shift + ignition margin + propellant reserve   ◀ TABLE CONSUMED
        ↓
[1] CORRIDOR  the existing descent, unchanged: onset 74.7 km → committed bank →
              peak passage (RAM-C II anchor) → flow-resolved exit → reacquisition    ◀ INHERITED, BIT-IDENTICAL
        ↓
[2] COAST     supersonic aero deceleration; Mach bands crossed and logged;
              run_until the IGNITION CORRIDOR: Mach ∈ band, q ∈ window,
              nav state post-fix, margin (from the table row) inside reserve         ◀ COMMIT EVENT
        ↓
[3] BURN      on-axis retro burn; Cordell–Braun plume imprinted on the marched
              layer; drag decrement contracted from the field; mass depletes;
              MID-BURN STATE FORK: throttle interventions on the shared marched
              plume-coupled state (the §6 experiment, two named measurements)        ◀ PHYSICS COUNTERFACTUALS
        ↓
[4] TERMINAL  cutoff event → transonic passage (M = 1 crossed under thrust,
              regime logged) → subsonic powered descent, stopping-distance
              guidance, throttle clamped by the extended envelope → touchdown
              gate: descent rate, miss to pad, propellant floor                      ◀ SECOND REGIME CASCADE
```

Act 1 is the corridor as it exists. The bank-command study stays in the corridor example; this
example flies the committed 13.5° profile and adds no second bank sweep. Everything the corridor
gates today (the blackout window, the RAM-C II anchor band, drift and reacquisition) must
reproduce **bit-identically** in Acts 0–1, because the extension only appends: new atmosphere rows
sit below the altitudes the corridor samples, and the thrust stage is inert while the commanded
throttle is zero. That inheritance is itself a gate (§9, gate 1). **[holds under precondition:
atmosphere rows appended below 30 km only; thrust stage strictly inert at zero throttle]**

The regime-change inventory of the full descent, every row an event the run finds:

| # | Transition | Trigger | New/existing |
|---|---|---|---|
| 1 | slip → continuum (Knudsen band) | mean free path vs L | existing |
| 2 | GNSS available → denied | evolved n_e vs L1 cutoff | existing |
| 3 | GNSS denied → available | recombination drains the sheath | existing |
| 4 | aero-dominated → thrust-dominated | ignition commit (Act 2 gate fires) | new |
| 5 | supersonic → transonic → subsonic | flight Mach crossings, under thrust | new |
| 6 | burn → coast (cutoff), coast → landing burn | target deceleration / stopping distance | new |
| 7 | powered descent → touchdown | altitude floor with bounded speed | new |

Rows 4–7 all ride the existing provenance discipline: a classifier detects, the log records, the
gates read the rendered log. `RegimeClassify` today knows the Knudsen axis and the GNSS axis and
nothing Mach-based (`corridor.rs:90-191`), so the Mach/thrust/touchdown axes are new classifier
work (§7).

---

## 3. The physics that is new

### 3.1 The SRP closure: Cordell–Braun plume as an imprinted obstruction

The retro-plume problem in one paragraph: firing a nozzle into an oncoming supersonic stream
wraps the jet into a barrel shock terminated by a Mach disk; a contact surface separates plume gas
from post-bow-shock air; the bow shock stands far off the body, and the high post-shock pressure
that was decelerating the forebody is displaced by low-pressure recirculating plume gas. For a
central nozzle the preserved aerodynamic drag collapses almost entirely by C_T ≈ 1, and total
axial force only recovers once thrust itself dominates. Bow-shock instabilities appear past
C_T ≈ 3 (Jarvinen–Adams; Keyes–Hefner). All of this is measured wind-tunnel physics, surveyed in
Korzun–Braun–Cruz, and modeled analytically by **Cordell & Braun** (JSR 50(4), 2013): from the
thrust coefficient and the jet-to-freestream momentum-flux ratio, the model constructs the plume
boundary as an *effective obstruction* and recovers the bow-shock response and drag decrement,
validated against the wind-tunnel data on-axis. **[holds; exact correlation curves to be digitized
from the papers at kernel-build time]**

Two coupling depths are available, and the choice decides whether §6 means anything:

- **A0 — force-channel closure (fallback).** The Cordell–Braun/Jarvinen–Adams correlation computes
  the drag decrement pointwise each step and adjusts the force channel. Cheap, cited, and honest,
  but the marched layer never learns the plume exists. A state fork of that layer carries no plume
  information, so the §6 fork-economics question cannot be asked. Kept only as the stub behind the
  contract while building. **[holds]**
- **A — plume imprinted on the marched layer (the deliverable).** Each step, the analytic plume
  boundary (from that world's commanded throttle, via C_T and momentum-flux ratio) shapes a
  smoothed forcing region inside the compressible layer: interior forced toward the analytic jet
  state, the outer flow left to evolve its own standoff shock in response. The drag decrement is
  then **contracted from the evolved field** (forebody-strip pressure integration, the same way
  Jarvinen–Adams measured it), and the correlation becomes a *cross-check gate* instead of the
  answer. The mask/penalization pattern exists on the incompressible side (`QttImmersed2d`,
  Brinkman volume penalization with force contraction); porting a forcing region to the
  compressible marcher is new but bounded work. **[holds under precondition: compressible forcing
  seam built; contraction observable built]**

The `FloatType` convention and the physics-from-publication convention both apply: kernels live in
`deep_causality_physics/src/kernels/` (a new `propulsion/` family: mass-flow from Isp, thrust
coefficient, momentum-flux ratio, plume-boundary geometry, drag-decrement correlation,
stopping-distance ignition solution), each cited in its docstring with the PDF in
`deep_causality_physics/papers/`. The survey confirmed the propulsion family is greenfield: no
thrust, Isp, propellant, or rocket-equation code exists anywhere in the repo today.

### 3.2 The drag sign-flip band, and why the study must find it

The Jarvinen–Adams central-nozzle result gives the extension its counter-intuitive physics, the
analog of the corridor's clamped 40° branch: in the low-C_T band, lighting the engine destroys
preserved drag about as fast as it adds thrust. Marginal throttle there buys little and can buy
*negative* net deceleration. The throttle study of §6 must straddle this band and **measure the
non-monotonicity from trajectory outcomes**, with its location cross-checked against the
correlation. Peripheral-nozzle
configurations preserve drag far better at low C_T, which is precisely why the literature
recommends them; the example flies the central config because the effect it demonstrates lives
there, and says so. **[holds; exact band edges to be pinned against the digitized curve]**

### 3.3 The Mach cascade and the carrier

The survey settled a question this note would otherwise have had to hedge: **the compressible
carrier is not hypersonic-only.** The marcher is a general conservative-Euler IMEX scheme with no
Mach assumption, and the descent carrier already gates the Rankine–Hugoniot inflow jump behind
`mach > 1.05`, enforcing the raw freestream below it (`compressible_march_run.rs:217-224`). The
transonic passage therefore degrades gracefully by construction. What does change across the
cascade:

- `GAMMA_EFF = 1.1` is the reacting-shock recipe; cool low-Mach air wants γ → 1.4. Phases are
  separate march calls (§5), each with its own schedule and γ. **[holds]**
- `S_REF` (the implicit acoustic envelope) is sized for the hypersonic stations; at low Mach the
  sound speed dominates and the rebuild-on-drift mechanism will fire. Rebuilds are logged, gated
  and capped today; the terminal legs re-tune `S_REF` and inherit the same cap. **[holds]**
- The atmosphere table floors at 30 km and clamps constant below it
  (`DescentSchedule::sample` in `compressible_march_config.rs`), so every quantity below 30 km is currently frozen at
  the 30 km values. The table must be extended to ~0 km (US-1976 shape, same four-column rows).
  Rows are appended below the existing five, so all corridor-altitude sampling is untouched.
  **[holds]**

### 3.4 Mass as state

The corridor folds vehicle mass into the constant `CDA_OVER_M` bundle. A burning vehicle cannot:
thrust depletes propellant (ṁ = T/(Isp·g₀)), and both the ballistic coefficient and the achieved
acceleration move as mass falls. The propulsion stage carries `mass`, `propellant`, `throttle`,
and `ignited` as coupled-field scalars, and the aero stage's force normalization reads the carried
mass instead of the constant. Before ignition the carried mass equals the corridor's implied
constant, preserving Act-1 inheritance. **[holds]**

### 3.5 Navigation and control seams (mostly already there)

The most useful finding of the investigation: **the navigation engine needs no structural
change.** `ReentryNavEngine::predict` and the truth propagator both take an arbitrary specific
force through the same KS Strang-kick closure (its docstring calls it "the non-conformal force
per unit mass" and already anticipates more than aero), and the 17-state ESKF's specific-force term is
documented as "gravity + thrust". A retro-thrust stage sums `−T/m · v̂` into the force channel
(read-modify-write after `BankSteeredLift`, since `set_aero_force` overwrites), and the existing
`ImuModel` then *senses the burn automatically*, which is physically exactly right: accelerometers
measure thrust. Dead-reckoning quality through the burn falls out of machinery that already
exists. **[holds]**

Two control-side gaps are real:

- `CoupledField` has a single scalar command channel (`control_action`, today the bank), and
  `SafetyEnvelope` has three fixed limits (heat flux, g-load, bank). The burn needs a throttle
  channel and envelope axes for it: throttle floor and ceiling, a C_T ≤ 3 stability cap (which is
  a *dynamic* throttle ceiling, since C_T depends on q∞), an ignition-window q bound, a propellant
  floor, and a touchdown descent-rate bound. The clean shape is a second typed command channel
  plus an extended envelope consumed by the same `CyberneticCorrect` pattern: sense, clamp into
  the envelope, `Err` on unrecoverable breach. During the burn the vehicle is on-axis and the bank
  axis is idle, so no step ever has two live command axes at Tier A; but the bus should be built
  two-axis rather than overloaded. **[holds under precondition: command-bus extension built]**
- Terminal guidance: Tier A is the stopping-distance feedback law (commanded deceleration
  `a = v²/2h + g`, clamped by the envelope), which is closed-form, transparent, and gate-friendly.
  Apollo polynomial guidance (Klumpp 1974) and convex powered-descent guidance (Açıkmeşe–Ploen
  2007) are the named upgrade path, not Tier-A scope. **[holds]**

---

## 4. The weather table as an onboard artifact

The weather example's `weather_table.csv` already carries exactly the columns a day-of-entry
adjustment needs: per condition, the blackout `onset`/`exit`/`dwell`, and `drift_mean`/`drift_sd`
in the dark. The retropulsion example loads that file (a ~20-line std parser in the example's
`model.rs`; the schema is pinned by `WorldRow::SCHEMA`), takes the **measured temperature
departure dT of the day** as its input, interpolates the rows at dT, and lets the interpolated row
do three jobs. The loader's contract is pinned, because the recorded rows arrive in run order,
not temperature order (0, +20, −25, −40, −5, +5 K): sort ascending by `d_temp` after load,
reject duplicate keys, select the bracketing pair by value, and clamp a dT outside the tabulated
range to the nearest row, stamping the clamp into the provenance log. Bracketing by file order
instead of value would select non-adjacent temperatures for most dT inputs, and the row feeds
load-bearing margins. The three jobs:

1. **Trajectory adjustment (the "low-orbit" leg).** The deorbit/entry targeting shifts with the
   predicted window: a colder, denser day ionizes earlier, dwells longer, and drifts further
   (polar winter: 68.7 ± 5.2 m vs standard 45.9 ± 2.3 m), so the aim point and entry state shift
   to keep the exit-to-ignition gap inside margins. Tier A realizes this as config-time targeting
   with full provenance (the chosen row and dT stamped into the log); flying the actual low-orbit
   arc through the KS propagator is a stretch goal the nav engine already supports.
2. **Ignition margin.** The commit gate requires the navigated state to support the burn; the
   margin it demands is `drift_mean + k·drift_sd` from the interpolated row. The table's error
   bars become load-bearing flight inputs.
3. **Propellant reserve.** The reserve above the floor is sized from the same dispersion.

The practice precedent is direct: the Shuttle program regenerated its ascent guidance I-loads on
the day of launch from measured balloon soundings (DOLILU; wind *and thermodynamic* data), because
tables tuned to a standard day are wrong on the flown day. This example is the descent counterpart
of that operational pattern. **[holds as precedent; cite the NTRS DOLILU operations paper]**

**The table's value gets its own gate.** The example flies the belief counterfactual: the
same measured cold atmosphere in two worlds, one whose guidance interpolated the table at the
measured dT, one that assumed the standard-day row. Both are whole-world alternations from one
baseline, `!!ContextAlternation!!`-marked like every other counterfactual in the family. The gate
requires a material separation (the uninformed world must measurably breach its ignition margin or
land worse; band pinned at first build). **[holds under precondition: separation band earned from
the first measured run]**

---

## 5. Phase architecture: legs, stacks, and the one hard rule

The survey pinned the architectural fact the whole design must respect: **a coupling stack is
fixed per march call, and `MarchState` carries the coupled field (nav, regime, scalars, log) but
not the marched fluid tensor.** Chaining `march → state() → march` re-seeds the flow from the next
world's seed; only the trajectory side is continuous. The corridor already lives with this (its
four legs re-couple from carried state, and the quasi-steady layer re-converges within a few
steps), and the extension adopts the same defense where it applies.

The consequence that matters: **ignition cannot be a stack swap.** If the burn required a new
coupling stack, the fork study would fork a freshly re-seeded field and the plume-coupled state it
exists to measure would have been thrown away at the leg boundary. So:

- The **burn-phase stack** (Act 2 + Act 3 in one march call) contains the propulsion, plume, and
  extended-envelope stages from the start, inert while the world-published `commanded_throttle`
  is zero. Ignition is a *published-command event inside one world*, the same mechanism as
  `commanded_bank`. The coast, the commit, the burn, and the mid-burn fork all share one marched
  layer; the fork carries the live plume-imprinted tensor copy-on-write, exactly as the corridor's
  onset fork carried the sheath. **[holds]**
- Leg boundaries with re-seeds are placed only where the quasi-steady defense is honest: at the
  Act-1/Act-2 boundary (blackout exit; the corridor does this today) and at cutoff before the
  terminal leg (subsonic re-seed under its own γ and `S_REF`). Each re-seed is logged. **[holds]**

The coupling stack for the burn leg, reading top to bottom like the corridor's:

```text
VibrationalLag → FiniteRateIonization → FreestreamFeeds → RegimeClassify(+Mach axis)
→ BankSteeredLift(mass-aware) → PlumeObstruction (analytic boundary → forcing region
  → contracted drag decrement) → RetroThrust (−T/m·v̂ into the force channel; ṁ depletion)
→ SuttonGravesLoads → TruthGnss → TrajectoryNav(with_imu) → ThrottleGuidance
→ Telemetry → CyberneticCorrect(extended envelope)
```

---

## 6. Fork the state, not the parameters (the centerpiece)

"Fork the physics" has two readings in the retropulsion case, and only one of them tests anything
new.

**The shallow version forks the plume's inputs.** Branch worlds publish different thrust
coefficients; the Cordell–Braun model recomputes each branch's plume boundary and drag decrement;
each branch flies genuinely different flow. That is already a step past the corridor's bank study,
where the flow columns were branch-invariant to three digits (peak heat 1.703e6 across all six
coarse branches; the branches diverged by decision, not by flow). But structurally it re-runs a
cheap model per branch from different inputs: the weather example's shape, independent worlds
per condition. Legitimate; the experiment is the second version.

**The real test forks the marched state.** March the coupled descent into the burn. Pause at a
chosen point *inside* the retropulsion phase. Fork the paused, plume-imprinted field
copy-on-write (`CompressiblePause::fork` is the existing O(1) Arc share). Apply a small set of
throttle interventions, one per branch, each published into its branch's world so that **each
branch's intervention feeds back into its own plume and drag through the model, step by step**.
Continue every branch from the shared forked state. Then read two measurements:

1. **Does the divergence reflect the coupling?** Three witnesses, gated together:
   - the per-branch flow observables (plume length, standoff, contracted drag decrement) must
     spread across branches beyond a pinned threshold (the corridor's branch-invariant flow
     columns are the explicit foil);
   - the net-deceleration-vs-throttle ordering must show the **drag sign-flip band** of §3.2,
     found from trajectory outcomes and agreeing with the correlation's predicted location;
   - each branch's trajectory divergence must differ measurably from a **frozen-drag prediction**
     (same thrust schedule, drag held at the fork value). That last witness isolates the coupling:
     if thrust-only kinematics predicts the divergence, the flow was along for the ride.
2. **Does the fork stay cheap when the state carries the plume?** The corridor's fork was cheap on
   a smooth RAM-C sheath. The question here is whether copy-on-write survives flow that is
   genuinely coupled to the intervention: fork setup cost (it is O(1) by construction; the cost
   appears at first divergent write), per-branch step wall-clock relative to the trunk, and
   post-fork bond-dimension growth per branch through the continuation, against the cap. This is
   the miniature of the turbulence question run on tractable physics. If the plume-coupled state
   forks cheap, copy-on-write is established for intervention-coupled flow. If it degrades, the
   measured bond/wall-clock trace says *where* it breaks, on a case cheap enough to diagnose.
   Either outcome is a result. **[open until measured; first run pins the bands, later runs gate
   regressions, the corridor's earned-band convention]**

**Discipline pins, verbatim design constraints:**

- The intervention stays **on-axis** and inside the Cordell–Braun validated envelope. No angle of
  attack, however tempting the realism: off-axis leaves the model's validity, and a surprising
  result would then be unattributable, real physics or model extrapolation with no way to tell. The
  value of this phase is that the physics is trustworthy, so the design stays where it is
  trustworthy. Off-axis SRP and thrust vectoring are named Tier-C, not scope creep. **[holds]**
- The branch roster is small and purposeful: a coast branch (throttle 0), two or three candidates
  straddling the sign-flip band, one nominal, one engine-degraded (a fixed fractional thrust, the
  contingency world). A refinement round around the committed throttle mirrors the corridor's
  coarse/fine pattern only if the first measured landscape warrants it.
- Scoring mirrors the corridor: trajectory-derived terminal miss to a shared aim, propellant
  consumed, peak loads; committed rule pinned in `constants.rs`. The knowledge-floor argument
  returns unchanged: commit no finer than the navigated state supports, and the weather row's
  drift number *is* that floor.

---

## 7. What exists vs. what must be built

**Exists and is directly reusable** (survey-verified, file references in the survey record):

- The generic specific-force channel in both truth and nav propagation; the ESKF already models
  thrust in its specific-force term. The IMU senses the burn with no new code.
- The compressible carrier's graceful sub-Mach-1 behavior (RH jump gated at M > 1.05), the
  schedule/rebuild mechanism, and the 2-D/3-D marchers.
- The event-driven pause/fork/continue machinery and the full study grammar
  (`fork/branch/continue_for/refine/reduce_all/record/gates/verdict`), including O(1)
  copy-on-write forks of a paused marched state.
- `publish_constant` as the counterfactual command-injection mechanism (`commanded_bank` today,
  `commanded_throttle` tomorrow).
- The hypersonic reacting-air kernel suite, the KS/relativistic kernels, the provenance log, the
  gate sequences, the scoped parallel fan-out, and the weather table with its `record` seam.
- The self-verifying study pattern in `deep_causality_cfd/studies/` and the gated verification
  pattern in `verification/`.

**To build (bounded engineering):**

| Item | Where | Note |
|---|---|---|
| Propulsion kernel family: mass flow (Isp), C_T, momentum-flux ratio, Cordell–Braun plume boundary, drag-decrement correlation, stopping distance | `deep_causality_physics/src/kernels/propulsion/` + `papers/` | greenfield; cite-and-validate pointwise, the crate's convention |
| `RetroThrust` + mass-depletion stage | `deep_causality_cfd` corridor stages | read-modify-write on the force channel, after the lift stage |
| `PlumeObstruction` stage: analytic boundary → forcing region on the compressible layer → contracted drag decrement | `deep_causality_cfd` | the incompressible Brinkman mask + force contraction is the template |
| Mach/thrust/touchdown axes on the regime classifier | `deep_causality_cfd` | new keys in the transition log |
| Second command channel + extended `SafetyEnvelope` (throttle floor/ceiling, dynamic C_T cap, q window, propellant floor, descent-rate bound) | `deep_causality_cfd` | `CyberneticCorrect` pattern unchanged |
| Terminal guidance stage (stopping-distance feedback) | `deep_causality_cfd` or example-local | Klumpp / convex PDG named as upgrades |
| Atmosphere rows 0–30 km | `examples/avionics_examples/src/shared/constants.rs` | appended below; corridor sampling untouched |
| Weather-table loader + dT interpolation | example-local `model.rs` | std only, schema from `WorldRow::SCHEMA` |
| Example wiring: acts, three counterfactual layers, gates, README | `examples/avionics_examples/cfd/plasma_blackout/retropulsion/` | after the reorg (§8) |

**Open research (named, not hidden):**

- Plume-resolved compressible SRP on tensor trains. The retro-plume is a colliding-shock system
  (bow shock, barrel shock, Mach disk, shear layers) and its rank behavior on a quantized tensor
  train is unmeasured; the body-fitted lesson from the rank studies suggests the effective-body
  coordinate is again the lever, and the blend-metric dial (`qtt_blend_metric`) is the natural
  instrument. This is Tier-C; the §9 rank study is its smallest honest slice. **[open]**
- Long-penetration-mode unsteadiness. The low-C_T instability band is avoided by the envelope at
  Tier A (C_T cap and floor), not simulated. **[open]**
- Off-axis SRP, thrust vectoring, 6-DOF: excluded by the validity discipline of §6 and by the
  absence of a flight anchor, respectively. **[open]**
- Plume-induced signal attenuation (retro-plume as its own comms regime). Real for solid motors;
  kerolox plumes are cleaner; out of scope and flagged only so nobody mistakes silence for
  ignorance. **[speculative]**
- High-Mach SRP (the Falcon-9 entry-burn band, Mach ≈ 6–8). Above the wind-tunnel envelope; the
  SCIFLI observations are imagery, not force data. Tier A stays at Mach ≤ 2 where Jarvinen–Adams
  lives. **[open]**

---

## 8. The three-example reorganization

The blackout family becomes one folder with three siblings:

```text
examples/avionics_examples/cfd/plasma_blackout/
  corridor/        ← moved from the flat corridor folder (git mv, history-preserving;
  weather/           done: reorg-plasma-blackout-examples)
  retropulsion/    ← new (this note)
```

Mechanics, in order:

1. `git mv` both existing folders (authorized; preserves history). Do the reorg **first**, before
   any new code, so the retropulsion example lands in the final layout once.
2. `Cargo.toml`: the two `[[example]]` entries keep their names (`plasma_blackout_corridor`,
   `plasma_blackout_weather` — binary names are user-facing muscle memory and CI strings) and
   change only their `path`; a third entry `plasma_blackout_retropulsion` points at
   `cfd/plasma_blackout/retropulsion/main.rs`.
3. Embedded `CARGO_MANIFEST_DIR`-relative paths move with the folders: the corridor's
   `corridor_branches.csv` path in `main.rs`, the weather example's `get_table_path()` /
   `get_audit_dir()` in `model.rs`. The retropulsion example reads the sibling's table at
   `cfd/plasma_blackout/weather/weather_table.csv`, which the reorg makes short and obvious.
4. README cross-links between the two examples update (`../weather/README.md` style), and any
   website/docs pages that reference the old example paths get the same sweep.
5. Bazel: `examples/avionics_examples/` carries no `BUILD.bazel` today (the examples are
   cargo-run artifacts), so no Bazel change is expected. Verify at reorg time. **[holds under
   precondition: verified during the mv]**
6. The shared library (`avionics_examples::shared` under `src/shared/`) does not move; all three
   examples import it as before. (Both example READMEs and both `constants.rs` doc headers used
   a stale `blackout` module name for it; all four were corrected to `shared` during the reorg
   change.)

---

## 9. Build order (contract-first, measurement-gated)

The build-order lesson from the corridor applies unchanged: define the seams first, put stubs
behind them, and never let a placeholder shape the design. One addition specific to this
extension: **the fork-economics measurement is a go/no-go gate placed before the example is
wired**, because the centerpiece design (state-fork counterfactuals) depends on its answer, and
the user-facing example should be committed to whichever design the measurement supports.

```text
Stage R  Reorganization                    §8, mechanical, first
Stage 0  Contracts + inheritance guard
  ├─ propulsion coupling contract          throttle/mass/ignited scalars + force
  │                                        read-modify-write idiom; A0 stub behind it
  ├─ command-bus + envelope extension      second typed channel; extended SafetyEnvelope
  ├─ atmosphere rows to ~0 km              corridor gates re-run BIT-IDENTICAL (gate 1)
  └─ weather-table loader seam             dT → interpolated row, provenance-stamped
Stage 1  Physics kernels + papers          propulsion/ family, pointwise-validated
Stage 2  Measured de-risking (the waters test)
  ├─ verification/srp_drag_decrement       plume-imprinted layer vs Jarvinen–Adams
  │                                        correlation, contracted force cross-check
  └─ studies/qtt_rank_plume                rank of the plume-imprinted layer (Cartesian
                                           vs blend-metric) + FORK ECONOMICS on the
                                           plume-coupled state (§6 measurement 2)
Stage 3  Stages                            RetroThrust, PlumeObstruction, Mach classifier,
                                           ThrottleGuidance, extended CyberneticCorrect
Stage 4  Example wiring                    acts, the three counterfactual layers, gates,
                                           README, output.txt
```

Stage 2 answers §6's second measurement on a bare marched layer before any example exists. If the
fork degrades there, the finding is documented, the centerpiece pivots to the parameter-fork
design with the state-fork recorded as a measured limitation, and downstream work proceeds without
a rewrite. That is the experiment paying for itself either way.

---

## 10. Proposed gate set

Mirroring the corridor's numbered convention; bands marked *pinned* are earned from the first
measured run, then gate regressions.

| Gate | Checks |
|---|---|
| (0) integrity | no leg captured a step error; the envelope held |
| (1) corridor inheritance | Acts 0–1 reproduce the corridor's window, anchor band, drift/reacquisition witnesses bit-identically |
| (2) ignition corridor | commit fired as an event inside the Mach band, q window, post-fix nav state, and table-sized margin |
| (3) regime cascade | provenance shows the ordered transitions of §2's inventory, each flow- or trajectory-resolved |
| (4a) flow spread | plume observables differ across throttle branches beyond the pinned threshold |
| (4b) sign-flip found | deceleration vs throttle non-monotone; band location within tolerance of the correlation |
| (4c) coupling load-bearing | branch divergence differs from the frozen-drag prediction beyond the pinned threshold |
| (4d) fork economics | fork cost, per-branch step cost ratio, and post-fork bond growth inside the pinned bands |
| (4e) audit trail | every branch and belief world carries `!!ContextAlternation!!` naming its baseline |
| (5) table earns its place | informed vs uninformed worlds separate beyond the pinned band on the measured cold day |
| (6) touchdown | descent rate, miss to pad, and propellant floor inside limits |
| (7) compression | committed branch's final state re-quantizes inside the bond cap |
| (8) bounded rebuilds | carrier rebuilds across all legs under the cap |
| (9) wall-clock | full example inside the 600 s budget (estimate: corridor 42 s + ~1200–1800 further coupled steps + one branch fan-out ≈ low minutes) |

---

## 11. Validation anchors (verify before any paper use)

- **Jarvinen & Adams (1970)** — *The aerodynamic characteristics of large angled cones with
  retrorockets*, NASA CR (NTRS 19720005324). Mach 0.4–2.0, C_T to 30, central and peripheral
  nozzles; the drag-preservation dataset and the analytic flowfield construction (terminal shock,
  jet boundary, interface, bow shock within ~10% of shadowgraphs).
- **Cordell & Braun (2013)** — *Analytical Modeling of Supersonic Retropropulsion Plume
  Structures*, Journal of Spacecraft and Rockets 50(4), 763–770. The analytic plume-as-effective-
  obstruction model this design imprints; on-axis validated envelope.
- **Korzun, Braun & Cruz** — *Survey of Supersonic Retropropulsion Technology for Mars Entry,
  Descent, and Landing*. The survey pinning central-nozzle drag collapse, peripheral preservation,
  and the C_T ≈ 3 instability observations (with Keyes–Hefner).
- **NASA Langley UPWT / Ames SRP test + CFD validation series** (Berry et al.; Schauerhamer et
  al., incl. *Supersonic Retropropulsion CFD Validation with Ames 9×7 Test Data*, JSR). The
  modern wind-tunnel + CFD lineage for the closure's cross-checks.
- **NASA SCIFLI infrared observations of Falcon 9 first-stage entry burns** — the flight-data
  precedent and the "Mars-relevant retropulsion regime" 70–40 km window framing.
- **Klumpp (1974)** — *Apollo Lunar Descent Guidance*, Automatica 10(2); **Açıkmeşe & Ploen
  (2007)** — convex powered-descent guidance, JGCD 30(5). The terminal-guidance upgrade path.
- **Shuttle DOLILU** — *Space Shuttle Day-of-Launch Trajectory Design Operations* (NTRS
  20110003654). The measured-day table-update practice precedent for §4.
- **RAM-C II** — inherited unchanged through Act 1.

---

## 12. Related

- [`../cfd-plasma-blackout/plasma-blackout-corridor.md`](../cfd-plasma-blackout/plasma-blackout-corridor.md)
  — the corridor specification this descent extends.
- [`../cfd-plasma-blackout/gap-analysis.md`](../cfd-plasma-blackout/gap-analysis.md) — the gap
  tracker; its Gap-3 Encke↔Cowell regime switch (`RegimeSwitch`, built but unwired) becomes
  physically live in the dense-atmosphere legs and can be wired opportunistically.
- [`../cfd-plasma-blackout/build-order.md`](../cfd-plasma-blackout/build-order.md) — the
  contract-first principle §9 reuses.
- `examples/avionics_examples/cfd/plasma_blackout/corridor/README.md` and
  `examples/avionics_examples/cfd/plasma_blackout/weather/README.md` — the flown corridor and
  the table factory (the §8 layout, applied by the `reorg-plasma-blackout-examples` change).
- `deep_causality_cfd/studies/` — the self-verifying study pattern Stage 2 instantiates
  (`qtt_rank_study` lineage; `qtt_blend_metric` is the coordinate dial the plume study reuses).
- `deep_causality_cfd/verification/qtt_ramc_stagline` — the pattern for the Jarvinen–Adams
  verification target.
