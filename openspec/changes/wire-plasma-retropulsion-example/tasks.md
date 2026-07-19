## 0. Entry gate (blocking)

- [x] 0.1 **M4 (`add-retropulsion-terminal-descent`) is archived.** Acts 2 and 4 and gates (2) and
      (6) consume `ThrottleGuidance`, live `CyberneticCorrect` burn-axis enforcement, and the
      terminal-leg re-seed. Do not start group 4 or 6 before this holds; groups 1–3 and 5 are
      M4-independent and may proceed once M3 is archived (it is)
- [x] 0.2 Confirm where the ignition-window `q` bounds are enforced: `BurnEnvelope::q_min`/`q_max`
      are stored but read by no code path today (the library defers them to M4). If M4 did not
      enforce them, the Act-2 commit gate carries the check and the design's Open Question is
      resolved in this change's design.md before implementation

## 1. Example scaffold + shared powered-descent stack

- [x] 1.1 Create `examples/avionics_examples/cfd/plasma_blackout/retropulsion/` with the house file
      set (`main.rs`, `model.rs`, `constants.rs`, `utils_print.rs`); append the family's third
      `[[example]]` stanza `plasma_blackout_retropulsion` →
      `cfd/plasma_blackout/retropulsion/main.rs`; per-example `FloatType` alias in `main.rs` with
      the sibling docstring; the 0/1/2 exit-code map; **no tests in the example crate**
- [x] 1.2 `src/shared/constants.rs`: a propulsion section — thrust, `Isp`, initial mass and
      propellant, ignition-corridor bands, and the reference **area** named `PLUME_S_REF_M2`.
      **Never pass the shared `S_REF` (an acoustic wave speed) as the plume stage's `s_ref`** — same
      type, plausible name, silently wrong `C_T` all the way down
- [x] 1.3 `src/shared/world.rs`: a `powered_descent_coupling` assembler beside `corridor_coupling`,
      composing `RetroThrust` + `PlumeObstruction` **between** `BankSteeredLift` (the ④ writer) and
      `SuttonGravesLoads` (the first ④ consumer), per the `add_aero_force` ordering contract.
      **Do not compose `PropulsionStub` alongside them** — the stub bundles thrust + depletion + the
      A0 decrement, so the pair would double both
- [x] 1.4 `src/shared/world.rs`: a `powered_initial_field()` seeding `"mass"` and `"propellant"`
      **once via `set_scalar`**, never via `publish_constant` — `pre_step` re-writes every published
      constant at the head of each step, which would silently restore the seed after each step's
      depletion. Pre-ignition mass equals the mass implied by `CDA_OVER_M`
- [x] 1.5 Compose M4's library flight-sensor stage so `"q_inf"` and `"descent_rate"` are produced
      (`peak` of an absent scalar is 0, so their absence silently disables the dynamic C_T cap and
      the descent-rate breach). Do **not** author a second example-local producer — M4's
      `flight-sensor-scalars` owns it. Ensure M4's `ThrottleGuidance` drives **both**
      `set_throttle_action` **and** the published `"commanded_throttle"` for this world
- [x] 1.6 Enable the opt-in axes: `RegimeClassify::with_flight_axes(...)` and
      `CyberneticCorrect::with_burn_sensing(...)` on the powered stack, plus
      `SafetyEnvelope::with_burn(BurnEnvelope::new(...))`. Without these the regime cascade never
      logs and the dynamic C_T cap never binds, with no error either way

## 2. Acts 0–1: plan and the inherited corridor

- [x] 2.1 Act 0 (PLAN): load the weather table and interpolate at the measured departure (group 5
      delivers the loader); stamp the chosen row, the departure, and any clamp into provenance
- [x] 2.2 Act 1 (CORRIDOR): fly the existing corridor legs unchanged with the burn stack composed
      and the throttle at zero
- [x] 2.3 **Gate (1) corridor inheritance:** Acts 0–1 reproduce the corridor's blackout window,
      RAM-C II anchor band, and drift/reacquisition witnesses **bit-for-bit** against
      `plasma_blackout/corridor/output.txt`; re-run the corridor and weather examples and confirm
      both are unchanged

## 3. Acts 2–3: coast, commit, burn, and the state-fork centerpiece

- [x] 3.1 Acts 2+3 as **one march call** with the burn stack carried from step 0 — a leg boundary at
      ignition would re-seed the flow and the fork would fork a state with the plume already
      discarded. Re-seeds only at the Act-1/Act-2 blackout exit and at cutoff, each logged
- [x] 3.2 Act 2 (COAST): `until` the ignition corridor — Mach band, `q` window, post-fix nav state,
      and the table-sized margin `drift_mean + k·drift_sd`. The commit is a published-command event
      inside the world. **Gate (2) ignition corridor**
- [x] 3.3 Act 3 (BURN): on-axis burn; confirm the plume re-imprint fires from the **second** ignited
      step (it reads the geometry scalars `PlumeObstruction` publishes later in the same step, so it
      lags ignition by exactly one step — expected, not a bug); **gate (3) regime cascade** reads the
      ordered transitions from provenance
- [x] 3.4 The centerpiece: pause mid-burn, `fork`, `branch` the five-world roster (coast, two
      sign-flip straddlers, nominal, engine-degraded) each differing only by its published
      `"commanded_throttle"`, `continue_for`. **On-axis only** — no angle of attack. Drag authority
      in every branch is the A0 correlation; the imprint is state realism
- [x] 3.5 Score branches on trajectory-derived outcomes (terminal miss to a shared aim, propellant
      consumed, peak loads), committed rule pinned as a `const`
- [x] 3.6 **Gates (4a) flow spread**, **(4b) sign-flip found** — its detail line must state that it
      tests the correlation's non-monotonicity **surviving trajectory integration**, not an
      independent flowfield reproducing Jarvinen–Adams — and **(4c) coupling load-bearing** against
      a frozen-drag prediction (same thrust schedule, drag held at the fork value)
- [ ] 3.7 **Gate (4d) fork economics** regressing M1's bands: `shares_fluid_with`/`shares_field_with`
      true; per-branch continuation ratio inside the 2.0× band (M1's committed
      `studies/qtt_rank_plume/output.txt` records 0.67–1.04, setup 42 ns — **not** the 83 ns /
      0.68–1.05 the verdict prose carries; see design Open Questions); post-fork
      peak bond under the cap (M1 measured 16 flat under 32). **Gate (4e) audit trail** reads
      `report.effect_log()` — **not** disk files: the event-fork path has no `save_log` plumbing and
      `run_continued_segment` never flushes
- [x] 3.8 Confirm the roster continues concurrently through the existing `continue_branches` →
      `scoped_map` seam (the example crate already enables `parallel`), and that results are
      bit-identical with the feature on and off. Add no bespoke parallel path

## 4. Act 4: terminal descent and touchdown *(needs M4)*

- [x] 4.1 Cutoff at a leg boundary where the quasi-steady defense is honest; subsonic re-seed under
      its own γ and retuned `S_REF`, each re-seed logged; carrier rebuilds stay under the cap
- [x] 4.2 Terminal leg to the altitude floor under M4's stopping-distance guidance with the throttle
      clamped by the burn envelope; the M = 1 crossing under thrust logged
- [x] 4.3 **Gate (6) touchdown** — descent rate, miss to pad, propellant floor inside limits — and
      **gate (8) bounded rebuilds** counting `"carrier rebuilt at step"` entries in the rendered log

## 5. Belief counterfactual + weather-table loader (closes M2 task 6.4)

- [x] 5.1 Loader glue in the example's `model.rs`: read
      `cfd/plasma_blackout/weather/weather_table.csv` through the typed reader against the recorded
      schema, feed `KeyedTable`, path built from the manifest dir. Bracketing is **by value**, never
      by file order — the recorded rows arrive in run order (0, +20, −25, −40, −5, +5 K). A missing
      column is the reader's named-column error; a malformed cell is a loading error; never a
      default value
- [x] 5.2 Informed vs uninformed worlds on the same measured cold atmosphere, both
      `!!ContextAlternation!!`-marked, differing only in the dispersion row the guidance consumed
- [x] 5.3 The row sizes real inputs: ignition margin `drift_mean + k·drift_sd`, propellant reserve
      from the same dispersion; row, departure, and any clamp stamped into provenance
- [x] 5.4 **Gate (5) table earns its place** — material separation beyond a band earned on the first
      run; a non-separation is a reported finding, not a widened band

## 6. Gate set, capture, and docs

- [x] 6.1 Assemble the full numbered set (0)–(9) as gate sequences merged into one verdict; **every
      gate evaluated before the verdict decides** (no short-circuit); every threshold a documented
      `const` in `constants.rs` reached from a free-function gate
- [x] 6.2 **Gate (9) wall-clock** at the caller against the 600 s budget (the study cannot see the
      wall clock) and **gate (7) compression** re-quantizing the committed branch's final state from
      its report against the cap; an uncomputable witness fails its gate rather than passing
- [x] 6.3 Pin every earned band from the first measured run and record the pins; convert any gate
      the physics cannot satisfy into a reported finding with the conversion recorded
- [x] 6.4 `README.md` in the family form (SPDX header, How to Run, the acts, validation anchors,
      limitations, `## Where Things Live`), **stating the M1 AMBER finding plainly** — the A0
      correlation is the drag authority, the imprint is state realism. Add the crate README row and
      sweep cross-links between the three siblings
- [x] 6.5 Capture `output.txt` from a real passing release run

## 7. Verification and PR preparation (SDD)

- [x] 7.1 `bazel test //deep_causality_cfd/...` green; the corridor and weather examples re-run
      bit-identical
- [x] 7.2 `make format && make fix` — clippy clean without suppressions
- [ ] 7.3 `make test` and `make check` (full-repo SDD pre-PR gate)
- [ ] 7.4 Update `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md` — close the
      ledger, and move the note to the notes archive per the SDD workflow
- [x] 7.5 Prepare the commit message(s) per task group and hand to the user (never commit); draft the
      PR summary referencing this change and roadmap milestone M5
