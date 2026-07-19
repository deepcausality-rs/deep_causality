## 1. Flight sensor producers (deep_causality_cfd)

- [x] 1.1 Add a flight-sensor `PhysicsStage` publishing `"q_inf"` and `"descent_rate"`, both field
      names configurable (mirroring the gate's own sensing configuration). `q∞ = ½·(n∞·m̄)·V²` from
      the carrier-published `"freestream_n"` and `"flight_speed"`, with `m̄` a **constructor
      parameter** — the carrier has no molecular mass and the physics crate has no air `m̄`; the
      example's `FreestreamFeeds` already does the same `n·m̄` conversion
- [x] 1.2 `"descent_rate"` = `−(r·v)/|r|` from `"truth_state"`, **positive downward**. The sign is
      load-bearing: the gate tests `descent_rate > bound` after a max reduction, so an
      ascent-positive convention makes the bound unreachable in the regime it protects
- [x] 1.3 Inert when `"truth_state"` is absent or `len() < 6` — publish nothing rather than a value
      from a partial state
- [x] 1.4 Tests: both scalars published with correct units and sign; descending yields positive rate;
      absent/short truth state publishes nothing; field names honour the configured overrides

## 2. ThrottleGuidance stage (deep_causality_cfd)

- [x] 2.1 Add `ThrottleGuidance` beside `RetroThrust`/`PlumeObstruction`: `a_cmd = v²/2h + g` via
      `suicide_burn_deceleration_kernel` (returns an `Acceleration`, **not** a throttle), then
      `θ = m·a_cmd / T_full` saturated to `[0,1]`, written with `set_throttle_action`.
      **No thrust-from-throttle kernel** — the physics crate deliberately leaves throttle mapping to
      the CFD stages and `RetroThrust` already hardcodes `T = θ·T_full`; share that one convention
- [x] 2.2 Write the channel on **every** step, θ = 0 before commit. The gate's burn block is guarded
      on `throttle_action().is_some()`, so deferring the first write to ignition would leave the
      propellant floor, descent-rate bound, and throttle clamp unenforced on every pre-ignition step
- [x] 2.3 Propagate the kernel's singularity error at altitude ≤ 0 rather than producing a throttle
- [x] 2.4 Tests: the commanded θ matches the law; θ saturates at 1; θ = 0 pre-commit leaves the
      thrust/plume stages inert; ground-contact altitude errors; the guidance/RetroThrust round trip
      realizes `θ·T_full` with no second mapping

## 3. Ignition-corridor commit

- [x] 3.1 The four-condition conjunction: Mach band ∧ `q∞ ∈ [q_min, q_max]` ∧ post-fix nav ∧ position
      uncertainty inside the table-sized margin. The margin is **caller-supplied** —
      `ignition_altitude_kernel` takes it as an input by design because it is sized from the
      dispersion table's drift row
- [x] 3.2 Rising-edge detection with a **one-way latch carried on a field scalar** (not
      carrier-internal state, which resets at every leg boundary). The throttle channel cannot serve
      as the edge source: it is an `Option` never cleared between steps, carried by `Clone`, so a
      positive value persists after its producer goes quiet and a fork inherits its parent's command
- [x] 3.3 Read nav through the published `"nav_mode"` and `"nav_position_variance"` scalars, **never**
      `field.nav()`. Pin the two properties: `nav_mode` does not distinguish GNSS from optical fixes;
      `nav_position_variance` is a covariance **trace in m²**, so a metre-valued margin compares
      against its **square root**
- [x] 3.4 Log one commit entry naming the step and the four sensed values, transition-only (the
      nav-mode flip is the precedent)
- [x] 3.5 Tests: all four hold → commit; each one short → no commit; commit fires exactly once across
      many satisfying steps; latch survives a condition lapse; latch survives a leg boundary; the
      variance-to-metres conversion is a sqrt

## 4. Live envelope enforcement (defect fixes + new axis)

- [x] 4.1 **Enforce `[q_min, q_max]`** — today stored on `BurnEnvelope` with zero read sites and zero
      tests. Refuse (log + `Err`) on a throttle rising from zero outside the window; do **not**
      re-apply it as a running constraint once the burn is under way
- [x] 4.2 **Fix the crossed clamp window.** `clamp(commanded, throttle_floor, ceiling)` has no
      `lo ≤ hi` precondition and tests the lower bound first, so when `ct_ceiling < throttle_floor`
      a command between the bounds is clamped **up past the C_T cap** and a command at or above the
      floor is clamped **down below the floor** (a central-SRP stability constraint, per
      `physics/kernels/propulsion/srp.rs:217-220`). Refuse. Do **not** "fix" it by reordering the
      helper — that satisfies a one-sided "never below the floor" reading while violating the cap
- [x] 4.3 **Fix breach masking.** The thermal/g refusal returns before the burn block, so a heat or g
      breach hides a simultaneous propellant/descent-rate breach. This costs **diagnosis, not
      safety** — the `Err` short-circuits every later stage, so no throttle consumer runs. Sense all
      axes, log every breach, then return; the error names the first breach in a fixed axis order so
      the string is deterministic
- [x] 4.4 **Refuse the blind-gate case**: burn axes attached ∧ positive `"commanded_throttle"` scalar
      ∧ absent channel. Do **not** widen the gate's sensing to the scalar — the clamp writes back
      into the channel, and a channel written from a scalar source then outranks that world's
      published constant on every later step, freezing a counterfactual branch at its first clamped
      value. **This requires the MODIFIED delta on "Inactive axes change nothing"** (re-keying
      invisibility from "no channel written" to "neither seam driven") — the refusal fires inside
      the configuration the original wording appeared to protect
- [x] 4.5 Surface the unusable burn-sensing configuration (burn axes attached with a non-positive
      thrust reference). Exactly **one** axis degrades silently today — `max_ct` — while every other
      burn axis keeps enforcing; say that precisely rather than "burn sensing is off"
- [x] 4.6 Document the per-axis reduction and its safety direction: max-over-cells folded from zero is
      **conservative** for heat flux / g load / descent rate, **permissive** for `q_inf` (largest q →
      loosest ceiling) and propellant (fullest cell vs the floor)
- [x] 4.7 Tests for every case listed as untested today: the q window, breach masking, `with_burn`
      without `with_burn_sensing`, `s_ref`/`thrust_ref` ≤ 0 with positive q, the inverted clamp
      interval, the scalar-driven blind gate, and the exact bounding log texts
- [x] 4.8 Re-run the corridor inheritance guard: no world that omits `with_burn` reaches any new
      refusal. Both examples' **physics witnesses and all gates are bit-identical**; the corridor's
      provenance gains exactly the three leg-re-seed entries task 5.1 adds (one per `.from(state)`
      boundary), so its `output.txt` is regenerated — the Migration Plan's stated consequence, not a
      regression. Weather is untouched (it flies the origin `march_for` path, not `.from(state)`)

## 5. Terminal leg: re-seed visibility and rebuild budget

- [x] 5.1 Log a leg-boundary re-seed entry naming the incoming world and recording that the marched
      conserved state was re-seeded rather than carried. Today the boundary writes **nothing**, while
      the fork path — which genuinely preserves the marched state — logs its resume. Do not change the
      existing `"carrier rebuilt at step"` / `"march paused at step"` texts; downstream gates match on
      them
- [x] 5.2 Add an explicit rebuild bound enforced in-loop (`Err` at the bound) **and a
      rebuild-count accessor**. The carrier is not unbounded today — the `1.2×` re-pin against the
      `(1+tol)` trigger is a hysteresis ratchet needing ~1.44× further growth per rebuild, so the
      "a re-pin gate caps it" doc is accurate — but nothing states or exposes a bound, so the only
      machine-checkable budget is an example log-substring tally that breaks if the wording changes
- [x] 5.3 Document the trigger's scope and blind spot: wave-speed-keyed, **per-carrier** (s_ref
      resets at every leg boundary, re-arming at the baseline), and **density never enters
      `s_needed`** — so it never corrects a bad density anchor. Do **not** claim it cannot fire while
      decelerating: `t̂` rises as the atmosphere warms from ~217 K (15–20 km) to 288 K at sea level
- [x] 5.4 Terminal-leg configuration: its own reference scales, acoustic reference, and seed, with
      **both** gammas set for cool low-Mach air. The carrier keeps them separate — the schedule's
      `gamma_eff` builds the RH jump only; `cfg.gamma` is what the marcher evolves with — and the
      corridor pins both to one literal, hiding the distinction
- [x] 5.5 Tests: the re-seed entry appears and existing texts are unchanged; the cap refuses at the
      bound while the imprint cap keeps its softer stop-refreshing behavior; a terminal config marches
      stably from its own anchors

## 6. Terminal-descent harness (test, not an example)

- [x] 6.1 The **first** test anywhere composing `CyberneticCorrect` **with** burn axes into a marching
      stack — the M2/M3 burn integration test omits the gate entirely and drives ignition with a bare
      channel write, so the burn leg runs ungated today
- [x] 6.2 Drive a coast → commit → burn → cutoff → terminal leg through the guidance stage; assert
      **gate (2) ignition corridor** (the commit fired inside the band, window, nav state, and margin)
- [x] 6.3 Assert **gate (6) touchdown** — altitude floor reached with descent rate, miss, and
      propellant inside bounds — and the transonic crossing under thrust logged as a regime transition
- [x] 6.4 Assert **gate (8) bounded rebuilds** against the now-explicit bound via the new accessor
      (not a log grep), and exercise the rebuild path on a real descent leg: four integration tests
      drive it with an undersized `s_ref`, but the corridor's committed run records 0 rebuilds
      despite the "deliberately snug" `S_REF` doc claiming it fires where the descent steepens —
      correct that doc comment or re-pin the constant
- [x] 6.5 Assert integrity across the leg — no step captured an error, the envelope held

## 7. Downstream reconciliation

- [x] 7.1 Amend `openspec/changes/wire-plasma-retropulsion-example/`: the `"q_inf"`/`"descent_rate"`
      producers move from example-local to library (M5 wiring requirement + design D6), and M5's Open
      Question about where the ignition `q` window is enforced is answered here
- [x] 7.2 Update `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md` — M4 archived,
      M5 unblocked

## 8. Verification and PR preparation (SDD)

- [x] 8.1 `bazel test //deep_causality_cfd/...` green; the corridor and weather examples re-run
      bit-identical
- [x] 8.2 `make format && make fix` — clippy clean without suppressions
- [ ] 8.3 `make test` and `make check` (full-repo SDD pre-PR gate)
- [x] 8.4 Prepare the commit message(s) per task group and hand to the user (never commit); draft the
      PR summary referencing this change and roadmap milestone M4
