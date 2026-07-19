<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Plasma-Retropulsion Descent — implementation roadmap

**What this is.** The build plan that turns the design note
([`plasma-retropulsion-descent.md`](plasma-retropulsion-descent.md)) into a sequence of dedicated
OpenSpec changes. Each milestone below becomes exactly one change (propose → design → specs →
tasks → apply → archive). The milestones are ordered so that the one measurement that can
invalidate the centerpiece design is resolved **first**, in a single combined preceding spec, and
so that when the last change is archived the full example runs inside the 600 s budget and delivers
a correct simulation.

This roadmap is grounded in a fresh survey of the current tree (July 2026); every "exists" claim
carries a `file:line` anchor, and every "to build" item names its target seam. The design note is
the *why*; this document is the *what, in what order, gated how*.

Honesty convention (as in the note): **[holds]**, **[holds under precondition]**, **[measured]**,
**[open]**, **[speculative]**. The specs derived from each milestone inherit this convention in
their own acceptance text.

---

## 1. Status ledger — done vs remaining

The design note numbers its build order Stage R, 0, 1, 2, 3, 4 (§9). Every stage is now
complete — **Stage R** (the example-code refactoring), **Stage 1** (the physics-crate foundation),
**Stage 2** (the front-loaded de-risk measurement, M1), **Stage 0** (the cfd contract layer, M2),
both halves of **Stage 3** (M3's coupled physics stages and M4's guidance, ignition commit, live
envelope enforcement, and leg-re-seed visibility), and **Stage 4** (M5's example, which flies all of
it to touchdown). This roadmap keeps the note's stage names as an index and maps them onto
milestones M1–M5.

**The roadmap is closed.** The plasma-retropulsion descent runs end to end: blackout exit, an
ignition commit inside the Jarvinen–Adams band, a state-fork counterfactual on the marched
plume-coupled state, and a landing at 2.0 m/s with 1372 kg of propellant remaining. Fourteen gates
green in 256 s.

| Note stage | Content | State | Roadmap |
|---|---|---|---|
| **Stage R** | Example-code refactoring — reorg into `cfd/plasma_blackout/{corridor,weather}/` | **done** — `reorg-plasma-blackout-examples`, archived 2026-07-16 | — |
| **Stage 1** | Propulsion kernel family + papers in `deep_causality_physics` | **done** — `close-plasma-retropulsion-physics-gaps`, archived 2026-07-17 | — |
| **Stage 2** | Measured de-risking (plume imprint fidelity + fork economics) | **done** — `plasma-retropulsion-de-risk`, measured 2026-07-17, verdict **AMBER** ([`derisk-verdict.md`](derisk-verdict.md): imprint fidelity amber, fork economics + rank green), archived 2026-07-19 | **M1** (front-loaded) |
| **Stage 0** | cfd contracts + inheritance guard | **done** — `plasma-retropulsion-cfd-contracts`, archived 2026-07-19 (M5 glue task 6.4 carried forward) | **M2** |
| **Stage 3** (physics half) | `RetroThrust`, `PlumeObstruction`, plume-imprint refresh, classifier axes | **done** — `add-retropulsion-coupled-stages`, archived 2026-07-19 | **M3** |
| **Stage 3** (guidance half) | `ThrottleGuidance`, ignition commit, live envelope enforcement, leg re-seed visibility | **done** — `add-retropulsion-terminal-descent`, archived 2026-07-19 | **M4** |
| **Stage 4** | Example wiring, counterfactuals, gates | **done** — `wire-plasma-retropulsion-example`, applied and verified 2026-07-20 (all fourteen gates green; M2's carried-forward loader glue task 6.4 closed here) | **M5** |

**Done and directly reusable (survey-verified):**

- Propulsion kernels — 19 pointwise kernels + 19 `PropagatingEffect` wrappers across
  `deep_causality_physics/src/kernels/propulsion/{performance,nozzle,srp,plume,descent,wrappers}.rs`,
  quantities in `src/quantities/propulsion/mod.rs` (`MassFlowRate`, `FlowBranch`, `NozzleExitState`,
  `PlumeGeometry`), constants + accessors in `src/constants/propulsion.rs`, and the three SRP PDFs
  in `deep_causality_physics/papers/` (Jarvinen–Adams 1970, Cordell 2013, Korzun survey). Coverage
  closed to the reachable limit. **[holds]**
- Fork machinery — `CompressiblePause`/`CarrierPause` (`carrier.rs:370-383`), `fork()` is O(1)
  `Arc`-copy-on-write (`carrier.rs:440-450`); first divergent write triggers the single field CoW
  clone (`carrier.rs:708,720`), the marched tensor train is never deep-copied (`carrier.rs:728`).
  **[holds]**
- Study grammar — typestate combinators in `src/types/flow/study/mod.rs`:
  `fork` (:376) → `branch` (:399) → `continue_for` (:437) → `reduce`/`reduce_all` (:944,:987) →
  `refine` (:501) → `record` (:1020) → `gates` (:1037) → `verdict` (:1083). **[holds]**
- Verification pattern — `deep_causality_cfd/verification/qtt_ramc_stagline/` gates a computed
  result against published flight data and exits nonzero on FAIL (`main.rs:239-246`,
  `print_utils.rs:54-83`). **[holds]**
- Command injection — `publish_constant(name, value)` (`compressible_march_config.rs:394-397`)
  lands on the coupled field at the top of every `pre_step` (`compressible_march_run.rs:195-199`);
  the `commanded_bank` counterfactual seam is the exact template for `commanded_throttle`.
  **[holds]**
- Navigation — needs **no structural change**. `ReentryNavEngine::predict` (`reentry_nav.rs:64-84`)
  and the truth propagator (`stages.rs:113-137`) both take an arbitrary specific force through the
  same KS Strang-kick closure; the ESKF documents its specific-force term as "gravity + thrust"
  (`ins_error_state.rs:75-78`); the IMU senses the burn automatically
  (`nav_sensors.rs:43-45`). **[holds]**
- Transonic behavior — the compressible carrier gates its Rankine–Hugoniot inflow jump behind
  `mach > 1.05` and enforces raw freestream below it (`compressible_march_run.rs:217-224`), so the
  supersonic→subsonic passage degrades gracefully by construction. **[holds]**

**Built since (the July-2026 survey's "not built anywhere yet" list, now closed):** `RetroThrust`,
`PlumeObstruction`, and the flight-regime axes landed with M3; `ThrottleGuidance`, `IgnitionCorridor`,
`FlightSensors`, the enforced ignition `q`-window, and the leg-re-seed entry landed with M4; the
`commanded_throttle` channel and the `mass`/`propellant`/`ignited` scalars landed with M2;
`srp_drag_decrement` was measured and superseded at M1 (see `deep_causality_cfd/reverted/`), and
`qtt_rank_plume` is its surviving study. What remains net-new is **M5's example code**.

---

## 2. The major risk, front-loaded

One survey finding sets the whole order: **the immersed-boundary / Brinkman penalization stack that
imprints an obstruction on the flow and contracts a force back out of it is incompressible-only.**
`QttImmersed2d` (`solvers/qtt/immersed_2d.rs:28-132`), the smoothed mask `body_mask_2d`
(`tensor_bridge/mask.rs:65-92`), and the force-contraction observables `drag_lift`/
`penalization_integral` (`solvers/qtt/observe.rs:23-93`) all operate on the incompressible QTT
solver. The compressible marcher has **no** masked forcing seam — only a *uniform* body
acceleration in the residuals (`theories/compressible_ns.rs:67,102`; `theories/euler.rs:37`) and
Dirichlet inflow enforcement (`compressible_march_run.rs:121-135`). Porting a spatially-masked
forcing region + a force-contraction readout onto the 4-component `EulerStateTt2d` inside
`CompressibleCarrier` is genuine implementation work, not wiring.

This single seam is load-bearing for the note's coupling-depth **A** (§3.1) — the plume imprinted
on the marched layer — and therefore for the state-fork centerpiece (§6). If the port cannot
reproduce the Jarvinen–Adams drag collapse, coupling degrades to the **A0** force-channel stub, the
marched layer never learns the plume exists, and the fork study of §6 has nothing to fork. Three
risks live and die together on this seam:

1. **Plume-imprint fidelity** — does the compressible forcing region reproduce the Jarvinen–Adams
   central-nozzle preserved-drag collapse (fraction → ~0 by C_T ≈ 1) within band? **[open]**
2. **Fork economics of the plume-coupled state** — does O(1) copy-on-write survive flow genuinely
   coupled to a per-branch throttle intervention, or does post-fork bond dimension blow past the
   cap? **[open]**
3. **Rank of a colliding-shock plume layer** — the retro-plume is a barrel-shock / Mach-disk /
   shear-layer system whose tensor-train rank is unmeasured; the body-fit coordinate
   (`qtt_blend_metric`) is the candidate lever. **[open]**

All three are answered by one harness on a bare marched layer, before any contract, stage, or
example exists. That harness is **Milestone 1** — the combined preceding spec. Its verdict is a
go/no-go: **green** confirms the state-fork centerpiece and M2–M5 proceed as designed; **amber/red**
pivots the centerpiece to the parameter-fork design (§6, shallow version), records the state-fork
result as a measured limitation, and M3/M5 adjust their plume seam accordingly — with no downstream
rewrite, because M2's contracts and M3's stages are shaped the same either way (only the *depth* of
the plume coupling changes). This is the experiment paying for itself in both outcomes.

---

## 3. Milestone dependency graph

```text
        [done] Stage R reorg ─┐
        [done] Stage 1 kernels┤
                              ▼
  M1  plasma-retropulsion-de-risk    ◀ FRONT-LOADED COMBINED RISK SPEC (go/no-go)
        │  compressible forcing seam + drag contraction (harness form)
        │  verification/srp_drag_decrement · studies/qtt_rank_plume + fork economics
        │
        ├─────────────► M2  plasma-retropulsion-cfd-contracts  (may overlap M1; see §4)
        │                     throttle channel · mass/propellant scalars · extended
        │                     SafetyEnvelope · atmosphere rows 0–30 km · weather loader
        │                     GATE: corridor inheritance BIT-IDENTICAL
        ▼                        │
  M3  add-retropulsion-coupled-stages ◀───────┘   (needs M1 mechanism + M2 contracts)
        │  RetroThrust (mass depletion) · PlumeObstruction (production) · Mach classifier
        │  GATE: regime cascade in provenance; in-flight drag decrement matches M1
        ▼
  M4  add-retropulsion-terminal-descent          (needs M3 + M2 envelope)
        │  ThrottleGuidance (stopping distance) · extended CyberneticCorrect · cutoff →
        │  transonic → subsonic terminal leg · touchdown gate
        │  GATE: powered descent to touchdown, bounded speed/miss/propellant
        ▼
  M5  wire-plasma-retropulsion-example           (needs M1–M4)
           three-act wiring · state-fork centerpiece · belief counterfactual ·
           full §10 gate set · README · output.txt
           GATE: full example in budget, all gates green, correct simulation
```

---

## 4. Milestones

Each milestone states: the OpenSpec change name; its objective; concrete deliverables with target
seams (`file:line`); entry preconditions; the verifiable exit gate; and the risk tags the derived
spec must carry. Gate numbers in parentheses reference the note's §10 gate set.

### M1 — `plasma-retropulsion-de-risk`  *(combined preceding risk spec; measured 2026-07-17 — verdict **AMBER**, see [`derisk-verdict.md`](derisk-verdict.md): M3 carries the A0 drag authority, M5's centerpiece pivots to the parameter-fork design; fork economics and rank measured green)*

**Objective.** Resolve the three §2 risks on a bare marched layer, before committing the downstream
design. Build the smallest compressible forcing region that can carry a Cordell–Braun plume, verify
its drag decrement against Jarvinen–Adams, and measure the rank and fork economics of the resulting
plume-coupled state. Emit a recorded go/no-go verdict.

**Deliverables.**

1. **Compressible forcing-region seam (harness form).** A smoothed mask over `EulerStateTt2d` and a
   penalization term inside the compressible step, porting the incompressible primitives
   (`body_mask_2d` `mask.rs:65-92`; `penalize` `immersed_2d.rs:101-115`) to the
   `CompressibleCarrier`/`CompressibleMarcher2d` path (`compressible_march_run.rs:195-263`). Interior
   forced toward the analytic jet state; outer flow left to evolve its own standoff shock. Built as
   a study/verification harness, **not** yet a production `PhysicsStage`. **[open → measured]**
2. **Force-contraction observable.** A compressible analogue of `drag_lift`/`penalization_integral`
   (`observe.rs:23-93`): forebody-strip pressure integration contracting the drag decrement from the
   evolved field, the same quantity Jarvinen–Adams measured. **[open → measured]**
3. **`verification/srp_drag_decrement/`.** Drive the plume boundary from the propulsion kernels
   (`cordell_braun_plume_boundary_kernel`, `srp_thrust_coefficient_kernel`,
   `momentum_flux_ratio_kernel`), imprint it via deliverable 1, contract the decrement via
   deliverable 2, and gate the contracted fraction against `srp_preserved_drag_fraction_kernel`
   within a pinned band across C_T. Cross-check the central-nozzle collapse (fraction < 0.10 by
   C_T ≈ 1) and locate the sign-flip band (§3.2). Follows the `qtt_ramc_stagline` PASS/FAIL-then-exit
   pattern. **[open → measured]**
4. **`studies/qtt_rank_plume/`.** Sweep the plume-imprinted layer and record `max_bond`
   (`observe.rs:132-142`) in Cartesian vs blend-metric coordinates (`qtt_blend_metric` template,
   `studies/qtt_blend_metric/main.rs`). Then the **fork-economics** measurement (§6, measurement 2):
   pause the plume-coupled march, `fork` it (`carrier.rs:440`), apply a small roster of per-branch
   throttle interventions via `publish_constant("commanded_throttle", …)`, `continue_for`, and record
   fork cost, per-branch step wall-clock ratio, and post-fork bond growth against the cap. **[open →
   measured]**

**Preconditions.** Physics kernels (done). No CoupledField extension, no example — the harness
drives throttle directly through `publish_constant`.

**Exit gate (go/no-go, recorded).**
- Contracted drag decrement reproduces the Jarvinen–Adams preserved-drag fraction within the pinned
  band, and the central-nozzle collapse and sign-flip band are present. *(pre-image of gate 4b)*
- Plume-imprinted-layer bond dimension stays under the cap; blend-metric coordinate lowers it.
- Fork cost is O(1); per-branch wall-clock and post-fork bond growth inside pinned bands. *(pre-image
  of gate 4d)*
- **Verdict written to a checked-in artifact.** Green → centerpiece confirmed. Amber/red →
  documented pivot to parameter-fork; M3/M5 plume seam adjusts; recorded as measured limitation.

**Risk tags for the spec.** The whole milestone is **[open]** until its run; its outputs are the
first **[measured]** bands the later gates regress against. This is the only milestone whose *design
downstream can change* based on its result — hence its position.

---

### M2 — `plasma-retropulsion-cfd-contracts`  *(Stage 0: contracts + inheritance guard; **archived 2026-07-19** — throttle channel + additive force, `BurnEnvelope` + gate enforcement, inert `PropulsionStub`, atmosphere to 0 km, and the reusable `KeyedTable` lookup all landed; the WorldRow CSV loader glue (task 6.4) carries to M5)*

**Objective.** Extend the coupled-field, command-bus, atmosphere, and weather-loader seams so the
burn-phase stack can be assembled and run **inert**, and prove that doing so leaves the corridor
bit-identical.

**Deliverables.**

1. **Propulsion coupling contract.** Carry `throttle`, `mass`, `propellant`, `ignited` as coupled
   state — named scalars on `CoupledField` (the `scalars` vec, `coupling.rs:169-206`) or dedicated
   fields alongside `control_action` (`coupling.rs:220-227`). Establish the force
   **read-modify-write** idiom for a thrust term: read `aero_force()`, add, `set_aero_force()`
   (`coupling.rs:210-217`), positioned after `BankSteeredLift` (`corridor.rs:522`) and before
   `SuttonGravesLoads`/`TruthGnss`/`TrajectoryNav` (`world.rs:163-166`). Before ignition the carried
   mass equals the corridor's implied `CDA_OVER_M` constant so Act-1 normalization is unchanged.
   Ship the **A0 force-channel stub** behind the future `PlumeObstruction` seam so the stack compiles
   and runs with the plume inert. **[holds]**
2. **Second command channel + extended `SafetyEnvelope`.** A typed throttle channel beside the bank
   channel, and envelope axes consumed by the unchanged `CyberneticCorrect` sense→clamp→`Err`
   pattern (`corridor.rs:637-712`): throttle floor/ceiling, a **dynamic** C_T ≤ cap (q-dependent,
   since C_T = T/(q∞·S_ref)), an ignition-window q bound, a propellant floor, and a touchdown
   descent-rate bound. Extend `SafetyEnvelope` (`corridor.rs:535-553`) — the bus is two-axis even
   though the burn runs on-axis with the bank idle. **[holds under precondition: command-bus
   extension built]**
3. **Atmosphere rows 0–30 km.** Append US-1976-shaped four-column rows below the existing floor in
   `examples/avionics_examples/src/shared/constants.rs:78-79`. `DescentSchedule::sample` clamps to
   the first table row (`compressible_march_config.rs:125-150`), so appending *below* leaves all
   corridor-altitude sampling untouched. **[holds]**
4. **Weather-table loader + dT interpolation.** Example-local `model.rs` std parser for
   `cfd/plasma_blackout/weather/weather_table.csv` (schema pinned by `WorldRow::SCHEMA`): load, sort
   ascending by `d_temp`, reject duplicate keys, select the bracketing pair **by value**, clamp
   out-of-range dT to the nearest row and stamp the clamp into provenance. **[holds]**

**Preconditions.** Independent of M1's *result* — the atmosphere/loader/envelope/mass work is pure
plumbing and can **overlap M1**. Only the shape of the A0-stubbed `PlumeObstruction` seam is
finalized after M1's verdict (a one-line depth switch), so hold that sub-item until M1 lands.

**Exit gate.**
- **(1) corridor inheritance, bit-identical.** With the propulsion stack present but throttle ≡ 0
  and atmosphere rows appended below 30 km, Acts 0–1 reproduce the corridor's blackout window,
  RAM-C II anchor band, and drift/reacquisition witnesses bit-for-bit against
  `plasma_blackout/corridor/output.txt`. **[holds under precondition: thrust stage strictly inert at
  zero throttle; atmosphere rows appended below 30 km only]**
- Extended envelope compiles into the corridor's `CyberneticCorrect` with no behavior change while
  the throttle axis is idle.
- Weather loader round-trips the six recorded rows and selects the correct bracket for a spread of
  dT inputs, clamp stamped.

---

### M3 — `add-retropulsion-coupled-stages`  *(Stage 3: physics half; **archived 2026-07-19** — `RetroThrust`, `PlumeObstruction` carrying the A0 correlation as the drag authority, the plume-imprint refresh riding the carrier's existing field-reading reconfiguration channel, and the opt-in Mach/thrust/touchdown classifier axes all landed; the opt-in is load-bearing, since the carrier publishes `"flight_mach"` every step)*

**Objective.** Make the burn physically couple: thrust felt in the force channel and the IMU, the
plume imprinted on the marched layer in flight, and the new regimes detected and logged.

**Preconditions met (2026-07-19).** M1 (validated forcing seam + AMBER verdict) and M2 (throttle
channel, mass/propellant/ignited scalars, force-RMW idiom, `BurnEnvelope` + gate enforcement, the
inert `PropulsionStub` seam) are both archived. The M1 verdict is **AMBER**, so — per the design
note §3.1 and the verdict's decision table — `PlumeObstruction` carries the **A0 correlation as the
drag authority**; a marched-layer imprint via the landed `ForcingRegion` seam is optional
*state-realism only*, never the force-channel drag closure. This is a resolved precondition, not an
open branch.

**Deliverables.**

1. **`RetroThrust` stage.** `−T/m·v̂` read-modify-write into `aero_force` (idiom from M2),
   positioned per M2's ordering; propellant depletion ṁ = T/(Isp·g₀) via
   `propellant_mass_flow_kernel`; mass-aware force normalization reading the carried mass. The IMU
   senses the burn with no new code (`nav_sensors.rs:43-45`). **[holds]**
2. **`PlumeObstruction` stage.** Productionize M1's validated forcing mechanism as a `PhysicsStage`
   (`coupling.rs:269-279`): analytic boundary from that world's `commanded_throttle` (via C_T and
   momentum-flux ratio) → forcing region on the compressible layer → drag decrement contracted from
   the evolved field. The Jarvinen–Adams correlation becomes a **cross-check gate**, not the answer.
   If M1 came back amber/red, this stage carries the A0 force-channel closure instead and the note's
   §6 shallow-fork applies. **[holds under precondition: M1 green; else A0 depth]**
3. **Mach / thrust / touchdown classifier axes.** Extend `RegimeClass` and `RegimeClassify`
   (`corridor.rs:69-191`): new fields, new `key()` terms (`corridor.rs:85-87`), reading the already
   published `"flight_mach"` scalar (`compressible_march_run.rs:238`), the `ignited` flag, and the
   altitude floor. Emits transitions 4–6 of the note's §2 inventory into the provenance log
   (`corridor.rs:174-187`). **[holds]**

**Preconditions.** M1 (validated forcing mechanism + verdict), M2 (throttle channel, mass/propellant
scalars, force-RMW idiom, A0 stub seam).

**Exit gate.**
- **(3) regime cascade.** In a burn leg, the ordered transitions (aero→thrust-dominated, the Mach
  crossings under thrust, burn↔coast) appear in provenance, each flow- or trajectory-resolved.
- In-flight contracted drag decrement matches M1's verified correlation within band. *(gate 4b, in
  flight)*
- **(0) integrity** holds across the burn leg — no step captured an error, envelope held.

---

### M4 — `add-retropulsion-terminal-descent`  *(Stage 3: guidance half; **archived 2026-07-19**)*

**Scope grew at spec time.** The survey behind the derived specs found M4 is not three additions but
three additions plus four repairs, because the M2 gate has never been reachable in a composed world:
nothing outside tests writes the throttle channel, no world attaches a `BurnEnvelope`, and
`"q_inf"`/`"descent_rate"` are written only by the gate's own unit tests — so two axes work under
test with no path to work in flight. On top of that, a crossed clamp window (dynamic `C_T` ceiling
below the throttle floor) emits an out-of-envelope throttle in *either* direction; a heat/g breach
masks a simultaneous burn breach (a diagnostic loss, not a safety hole — the `Err` short-circuits
later stages); the rebuild budget has no statable bound and no reader, only an example log-grep; and
a leg boundary discards the marched fluid state with no provenance entry at all. The derived change
also **modifies** `powered-descent-envelope`'s "Inactive axes change nothing" requirement, whose
"no throttle channel was written" wording conflicts with the crate's channel-or-scalar definition of
a commanded throttle. **[measured against the tree 2026-07-19]**

**Objective.** Guide the burn and land it: the ignition-corridor commit, envelope enforcement of the
throttle, and the cutoff → transonic → subsonic terminal leg to a touchdown gate.

**Deliverables.**

1. **`ThrottleGuidance` stage.** The stopping-distance feedback law a_cmd = v²/2h + g
   (`suicide_burn_deceleration_kernel`, `stopping_distance_kernel`), clamped by the envelope. The
   **ignition-corridor commit** fires as a published-command event inside one world (Mach band, q
   window, post-fix nav state, and the table-sized margin `drift_mean + k·drift_sd`). Klumpp /
   convex PDG are named upgrades, not scope. **[holds]**
2. **Extended `CyberneticCorrect` enforcement.** The same sense→clamp→`Err` loop now enforces the M2
   envelope axes: throttle floor/ceiling, dynamic C_T cap, q window, propellant floor, descent-rate
   bound; `Err` on unrecoverable breach. **[holds]**
3. **Terminal leg re-seed.** Cutoff placed at a leg boundary where the quasi-steady defense is honest
   (§5): a subsonic re-seed under its own γ → 1.4 and retuned `S_REF` (module constants today,
   `constants.rs:50-52`), each re-seed logged; carrier rebuilds stay under the existing cap
   (`compressible_march_run.rs:242-256`). **[holds]**

**Preconditions.** M3 (thrust + plume stages, Mach classifier), M2 (envelope axes, atmosphere to
0 km).

**Exit gate.**
- **(2) ignition corridor.** Commit fired as an event inside the Mach band, q window, post-fix nav
  state, and table-sized margin.
- **(6) touchdown.** Powered descent reaches the altitude floor with descent rate, miss to pad, and
  propellant floor inside limits; the M = 1 crossing under thrust is logged.
- **(8) bounded rebuilds.** Carrier rebuilds across the terminal leg stay under the cap.

---

### M5 — `wire-plasma-retropulsion-example`  *(Stage 4: example, counterfactuals, gates; **applied and verified 2026-07-20** — the terminal milestone, and the one that closed the roadmap)*

**Objective.** Assemble the example, run the two counterfactual studies that are the reason the
example exists, and pass the full gate set — the point at which the Plasma-Retropulsion Descent
delivers a correct end-to-end simulation.

**Deliverables.**

1. **Example folder + wiring.** `examples/avionics_examples/cfd/plasma_blackout/retropulsion/` in the
   house layout (`main.rs`, `model_config.rs`, `model.rs`, `model_types.rs`, `utils_print.rs`,
   `constants.rs`; config/execution separation), a third `[[example]]` entry
   `plasma_blackout_retropulsion` in `Cargo.toml`, and the burn-phase coupling stack (§5) assembled
   in `shared/world.rs` alongside `corridor_coupling` (`world.rs:121-181`). The five acts
   (PLAN/CORRIDOR/COAST/BURN/TERMINAL) run as the leg structure the note pins in §5. **[holds]**
2. **State-fork counterfactual centerpiece (§6), at the hybrid depth.** March into the burn, pause,
   `fork` the marched plume-imprinted field copy-on-write, apply the small throttle roster (coast,
   sign-flip straddlers, nominal, engine-degraded) each `publish_constant`-injected so every branch's
   intervention feeds its own plume and drag, `continue`, and score. On-axis, inside the
   Cordell–Braun envelope — **no angle of attack** (§6 discipline pin). Reuses M1's fork-economics
   harness. The derived specs fly the **hybrid** the M1 verdict named as measurement-consistent and
   left for M5 to propose: the A0 correlation is the in-flight drag authority (risk 1 amber) while
   the state fork carries the flow-realism and fork-economics witnesses (risks 2 and 3 green, so
   gates 4a and 4d exist at all — a parameter fork cannot express either). Gate 4b states its own
   authority: under A0 it tests that the correlation's non-monotonicity survives trajectory
   integration, not that an independent flowfield reproduced Jarvinen–Adams. **[measured basis;
   depth decided at M5 design time per the verdict's decision table]**
3. **Belief counterfactual (§4).** Informed (table-interpolated at the measured dT) vs uninformed
   (standard-day row) worlds on the same measured cold day, both `!!ContextAlternation!!`-marked;
   the gate requires a material separation. **[holds under precondition: separation band earned from
   the first run]**
4. **Full §10 gate set.** Gates 0–9, including the counterfactual suite (4a flow spread, 4b
   sign-flip, 4c coupling-load-bearing vs a frozen-drag prediction, 4d fork economics, 4e audit
   trail), (5) table earns its place, (7) compression under the bond cap, and (9) wall-clock inside
   600 s. Bands pinned on the first measured run, then regressed. **[holds under precondition: bands
   earned from first run]**
5. **README + `output.txt`.** The three-example loop closed — generate the table (weather), fly the
   table (retropulsion), with cross-links swept.

**Preconditions.** M1–M4 all archived (M4 archived 2026-07-19 — the last one). Two M5 items were
reconciled by M4: the `"q_inf"`/`"descent_rate"` producers moved from example-local to the library
(`FlightSensors`), and the ignition `q`-window enforcement question is answered — `CyberneticCorrect`
refuses an ignition outside the window and does not re-apply it to a burn under way.

**Exit gate.** The full example runs inside the 600 s budget; every §10 gate passes; the touchdown
witnesses, regime cascade, and both counterfactuals are present and material. **This is the
terminal milestone — its archive means the Plasma-Retropulsion Descent works and simulates
correctly.**

### M-later — field-channel fidelity and acceleration *(unscheduled, measurement-gated)*

Not a milestone; the recorded path by which the field channel could ever take drag authority
back from the A0 correlation, per the M1 verdict addendum
([`derisk-verdict.md`](derisk-verdict.md), "What would upgrade this to green (revised)").
The 2⁶ probe measured the mechanism's *direction* (the jet penetrates once the dissipation
floor halves; the outer strip band crossed below 1); the magnitude needs, in dependency
order: (1) front-respecting transport (locally wave-speed-scaled dissipation and/or the
Stage-4 tracked Rankine–Hugoniot interface), (2) a domain that holds the displaced bow shock,
(3) an axisymmetric or 3-D formulation.

That bar is compute-bound — steps double per L while per-step cost rises; the domain-widened
2⁷–2⁸ runs it implies sit near a hundred single-core hours today — so the **acceleration
ladder is part of the path**, each rung behind a timing gate (the
`compressible_carrier_timing` pattern), in expected-leverage order: TT cross-interpolation
for the nonlinear flux (removes the per-step dense dequantize→flux→requantize round-trip at
the rounding tolerance); fused rounding (once per component per step, not per operation);
component- and roster-level `deep_causality_par::scoped_map` (four independent component
chains per step; independent sweep points and counterfactual branches — the descent note's
§6 pin); locally scaled dissipation (also upgrade item 1 — it lowers the required L
outright). Tensor-crate kernels (randomized/sketched rounding and below) stay gated behind
[`../tensor-network/ACCELERATION-SOTA-FIRST.md`](../tensor-network/ACCELERATION-SOTA-FIRST.md);
GPU work is **deferred** (user decision 2026-07-17 — single-train bond-≤64 work is
latency-bound; revisit only for batched branch rosters, at the survey stage).

---

## 5. Cross-cutting constraints every derived spec inherits

- **Physics-from-publication.** Any new kernel cites its source in the docstring with the PDF in
  `deep_causality_physics/papers/`; public/open-access only. The propulsion family and its three
  PDFs already satisfy this; M1–M5 add no new kernels beyond what Stage 1 shipped (they consume
  them), so this mainly binds any incidental kernel that surfaces during the forcing-seam port.
- **`FloatType` alias** in all example code; `f64` only at the display boundary. The per-example
  alias is deliberate — do not consolidate.
- **One type, one module; tests mirror src** with `_tests.rs` suffix, registered in the module tree
  **and** in `tests/BUILD.bazel` (`rust_test_suite` globs). 100% coverage of added/edited code
  except provably unreachable lines.
- **No `unsafe`** — every new crate/module inherits `[lints] workspace = true`. (All targets here
  are existing crates that already opt in.)
- **Static dispatch, no `dyn`** — the `PhysicsStage` tuple-cons composition (`coupling.rs:282-335`)
  is the pattern; new stages join it the same way.
- **Build & test with Bazel** during implementation: `bazel build //deep_causality_cfd/...` and
  `bazel test //deep_causality_cfd/...` (and `//deep_causality_physics/...` for any kernel touch);
  cargo only for tight single-crate iteration.
- **OpenSpec validator** — the normative SHALL/MUST clause must lead the requirement body's first
  physical line, or archive fails.
- **Golden rules** — never commit (prepare the message, ask); never delete (ask; `git mv` is
  authorized).

---

## 6. Validation anchors

Carried from the design note §11; verify before any paper use. M1's `srp_drag_decrement` gates
against **Jarvinen–Adams 1970** (preserved-drag dataset, `jarvinen_adams_1970_ntrs_19720005324.pdf`)
and **Cordell 2013** (analytic plume model, `cordell_2013_srp_analytic.pdf`); the central-nozzle
collapse and C_T ≈ 3 instability come from **Korzun–Braun–Cruz** (`korzun_braun_cruz_srp_survey.pdf`)
with Keyes–Hefner. Act-1 inheritance (M2 gate 1) is anchored on **RAM-C II** through the corridor's
existing verification. The §4 belief counterfactual cites **Shuttle DOLILU** as the measured-day
table-update precedent. Terminal-guidance upgrades name **Klumpp 1974** and **Açıkmeşe–Ploen 2007**.

---

## 7. Pre-flight housekeeping (trivial, non-blocking)

The Stage R reorg left two **empty, untracked** directory husks —
`examples/avionics_examples/cfd/plasma_blackout_corridor/` and `.../plasma_blackout_weather/` — from
the `git mv` (git does not track empty directories, and `Cargo.toml` already points at the new
`plasma_blackout/{corridor,weather}/` paths). They are harmless but stale. Removing them is a
one-line cleanup that needs user permission (golden rule); fold it into M2 or do it standalone. Not
a milestone.

---

## 8. Related

- [`plasma-retropulsion-descent.md`](plasma-retropulsion-descent.md) — the design note this roadmap
  sequences; §6 (fork the state), §9 (build order), §10 (gates), §11 (anchors).
- [`derisk-verdict.md`](derisk-verdict.md) — the M1 measured go/no-go (AMBER, 2026-07-17): the
  authority M3 and M5 cite for coupling depth and centerpiece design.
- `openspec/changes/archive/2026-07-16-reorg-plasma-blackout-examples/` — Stage R, done.
- `openspec/changes/archive/2026-07-17-close-plasma-retropulsion-physics-gaps/` — Stage 1, done.
- `deep_causality_cfd/verification/qtt_ramc_stagline/` — the verification template M1 instantiates.
- `deep_causality_cfd/studies/qtt_blend_metric/` — the rank-study template `qtt_rank_plume` reuses.
- `deep_causality_cfd/src/solvers/qtt/immersed_2d.rs`, `tensor_bridge/mask.rs`,
  `solvers/qtt/observe.rs` — the incompressible Brinkman primitives M1 ports to the compressible
  marcher.
