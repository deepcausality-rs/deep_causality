## Context

M5 is the terminal milestone of the plasma-retropulsion roadmap. Its inputs are all landed or
scheduled: M1 measured the coupling depth and wrote the verdict, M2 landed the contracts, M3 landed
the burn physics stages, M4 lands the guidance law and the terminal leg. What is missing is the
example that flies them, and the survey that preceded this design found that the missing piece is
not "wiring" in the trivial sense — six of the seams fail *silently* when composed naively.

The load-bearing survey findings, each with its anchor:

- `CompressibleCarrier::pre_step` re-publishes **every** world constant with `set_scalar` at the
  head of every step (`compressible_march_run.rs:300-302`), while `RetroThrust` decrements `"mass"`
  and `"propellant"` inside the coupling (`retropulsion.rs:233-242`). A world that seeds mass via
  `publish_constant` never depletes, so the propellant floor can never trip and the mass-aware
  thrust normalization is frozen.
- The throttle is read from **three** seams by three consumers: the propulsion stages prefer
  `throttle_action()` and fall back to the `"commanded_throttle"` scalar (`retropulsion.rs:86-92`);
  `CyberneticCorrect`'s burn gate reads **only** the channel (`corridor.rs:985-986`);
  `refresh_plume_imprint` reads **only** the scalar (`compressible_march_run.rs:155-158`).
- `refresh_plume_imprint` consumes `"plume_max_radius"`/`"plume_penetration"` published by the
  *previous* step's `PlumeObstruction`, because `pre_step` runs before the coupling
  (`carrier.rs:109-123`). On the first ignited step the geometry scalars are absent and the refresh
  returns `Ok` silently — the imprint lags ignition by exactly one step.
- `RegimeClassify`'s Mach/thrust/touchdown axes are opt-in behind `with_flight_axes`
  (`corridor.rs:183-188`), and `CyberneticCorrect`'s dynamic C_T cap is inert at the default
  `thrust_ref = 0` until `with_burn_sensing` is called (`corridor.rs:857-907`).
- Nothing in `deep_causality_cfd` produces `"q_inf"` or `"descent_rate"`, yet `CyberneticCorrect`
  reads both. `peak` of an absent scalar is 0, so the descent-rate breach can never fire and the
  dynamic C_T cap degenerates to the static ceiling.
- `S_REF = 1.8` in the shared constants is the **reference wave speed of the implicit acoustic
  envelope** (`constants.rs:48-50`), not an area. `PlumeObstruction::new(thrust, q_inf, s_ref)`
  wants an area in m². The two are same-named and same-typed.

The centerpiece also carries an open design choice the M1 verdict explicitly left to this milestone:
the verdict pivoted M5 to "the parameter-fork design", then recorded the nuance that the state-fork
*mechanics* measured green and that a hybrid "is measurement-consistent and may be proposed to the
user at M5 design time; this verdict does not preselect it" (`derisk-verdict.md:46-52`). D1 below is
that proposal.

## Goals / Non-Goals

**Goals.**

- A three-act-plus-two example that flies the corridor unchanged, commits to an ignition, burns
  on-axis with the plume coupled, and lands — inside the 600 s budget with every gate green.
- Both counterfactuals present and material: the mid-burn state fork and the belief counterfactual.
- Every silent seam above turned into a normative requirement, so the failure mode is a spec
  violation rather than a plausible-looking run with a frozen mass or an unclamped throttle.
- M2's carried-forward task 6.4 (the weather CSV binding) closed against the existing
  `weather-table-consumption` requirement.

**Non-Goals.**

- No new library physics. Every stage, kernel, envelope axis, and classifier axis this example
  composes already exists (or lands in M4). If a requirement here needs a library change, that is a
  finding to raise, not to implement inside M5.
- No angle of attack, no thrust vectoring, no off-axis SRP — the design note's §6 discipline pin.
- No second bank study: the corridor owns that, and this example flies the committed 13.5° profile.
- No upgrade to the field-channel drag authority. That path is the roadmap's unscheduled
  measurement-gated M-later, not M5.
- No example-crate tests. Examples are example code; anything general-purpose belongs in a library
  crate and is tested there.

## Decisions

**D1 — The centerpiece is the hybrid: A0 drag authority over a real state fork.** The M1 verdict
named three options and preselected none. Flying the **parameter fork** would discard a measurement
that came back green: the fork is O(1) (setup 42 ns, `shares_fluid_with`/`shares_field_with`
witnessed), per-branch continuation costs 0.67–1.04× the trunk, post-fork bond stays flat at 16
under the 32 cap, and the branch flow observables spread monotonically with throttle (L2 vs coast
0.18 / 0.28 / 0.35 / 0.61) — the corridor's branch-invariant foil is already broken. What M1
measured as failing is narrower: reading a Jarvinen–Adams-faithful drag decrement *out of the
field*. So the drag authority is the cited A0 correlation (M3's `PlumeObstruction`, unchanged), and
the fork is a genuine fork of the marched, plume-imprinted state, carrying gates (4a) flow spread
and (4d) fork economics — both of which the parameter fork cannot express at all. Rationale: the
example's job is to demonstrate what the stack can actually do, and this is the depth the
measurements support in both directions. Alternative — parameter fork — rejected because it throws
away a green measurement and reduces the centerpiece to the weather example's shape.

**D2 — Gate (4b) states its authority rather than overclaiming.** Under the A0 depth the trajectory
outcome is downstream of the correlation, so "the sign-flip band agrees with the correlation" is
close to tautological if left unqualified. The gate is therefore specified as what it actually
tests: that the correlation's non-monotonicity **survives trajectory integration** into
decision-relevant outcomes (net deceleration, terminal miss, propellant consumed) rather than being
washed out by the thrust term and the mass depletion. That is a real property with a real failure
mode, and it is not the claim that an independent flowfield reproduced Jarvinen–Adams. The gate's
detail string says so, so a reader of `output.txt` cannot mistake one for the other.

**D3 — Mass and propellant are seeded on the initial field; the throttle is driven on both seams.**
`world::powered_initial_field()` seeds `"mass"` and `"propellant"` with `set_scalar` exactly once,
and they are never published as world constants. The throttle guidance writes **both**
`set_throttle_action` and the world-published `"commanded_throttle"`, because the envelope gate
reads only the former and the plume re-imprint reads only the latter. Alternative — unify the seam
in the library — rejected for M5: it is a `deep_causality_cfd` change, so it belongs to a change
that owns that crate, and M5's job is to fly what exists. The asymmetry is recorded as a finding in
this design and as a normative "drive both" requirement so the example cannot half-wire it.

**D4 — The burn stack is carried from step 0, in one march call.** Acts 2 and 3 (coast, commit,
burn, and the mid-burn fork) share **one** march call with the propulsion stages composed from the
start and inert while `commanded_throttle` is zero. This is forced, not stylistic: a coupling stack
is fixed per march call and `MarchState` carries the coupled field but not the marched fluid
tensor, so making ignition a leg boundary would re-seed the flow and the fork would fork a field
that had thrown away the plume it exists to measure. Leg boundaries with re-seeds are placed only
where the quasi-steady defense is honest — the Act-1/Act-2 blackout exit (which the corridor
already does today) and cutoff before the terminal leg — and each re-seed is logged.

**D5 — The stack composes `RetroThrust` + `PlumeObstruction`, never alongside `PropulsionStub`.**
The M2 stub bundles thrust, depletion, and the A0 decrement in one contract stub. Composing it
beside the M3 production stages applies the thrust term and the propellant/mass depletion twice, and
*compounds* the drag decrement rather than doubling it: the decrement is written as
`add_aero_force((f − 1)·x)`, which rescales the axial channel multiplicatively (`x ← f·x`), so two
applications give `f²·D`. Nothing in the type system prevents the combination, so the example
composes the production pair and the spec forbids the pairing explicitly.

**D6 — `"q_inf"` and `"descent_rate"` come from M4's library stage, not an example-local one.** They
are read by `CyberneticCorrect` but produced nowhere, and their absence is silent (a `peak` over an
absent scalar is 0), so a safety axis that cannot fire reports as enforcing. M4
(`add-retropulsion-terminal-descent`, capability `flight-sensor-scalars`) lands the producer in
`deep_causality_cfd`, taking the mean molecular mass by construction, because the envelope axes that
consume the scalars are library and a decorative safety axis is a correctness problem rather than a
wiring preference. M5 composes that stage; it does not author a second producer.

**D7 — Propulsion constants are named for what they are.** The reference **area** the plume stage
consumes is `PLUME_S_REF_M2`, never the shared `S_REF` (an acoustic wave speed). The new shared
propulsion section carries thrust, Isp, the reference area, initial mass and propellant, and the
ignition-corridor bands, each with the model label the house convention requires.

**D8 — Pinned bands are `const`; runtime bands ride the row.** `GateFn<Row>` is a plain `fn` pointer
(`gate_seq.rs:20`), so a gate cannot **capture** anything — but that does not force every band to be
a `const`, and a spec that said so would contradict its own wall-clock gate. Two paths exist and the
example uses both: pinned bands are `const`s in `constants.rs` (one documented edit per re-pin), and
values known only at run time ride the `Row` type into the view, which is exactly how the weather
example gates elapsed seconds through `GateSeq<FloatType>` over `StudyView::of(&[elapsed])`. Gates
are written as named free functions in `model.rs` by convention, not by necessity — a non-capturing
closure coerces to the alias just as well. All gates are evaluated and then the verdict decides,
which `GateSeq::check` gives by construction (`gate_seq.rs:51-61` maps over every gate and collects);
the verification binaries' `&&` chain does not, and it suppresses later FAIL lines exactly when a
reader most needs them.

**D9 — Wall-clock and branch audit are read where they actually live.** The wall clock is gated at
the caller through `StudyView::of(&[elapsed])` because the DSL cannot see it. Gate (4e) reads
`report.effect_log()` and not disk files, because the event-fork path has no `save_log` plumbing at
all: `run_continued_segment` never flushes to a sink, so `fork → branch → continue_for` branches
produce no on-disk log today.

## Risks / Trade-offs

- [The hybrid centerpiece could be read as claiming a field-contracted drag decrement] → every
  requirement naming the fork also names the A0 correlation as the drag authority, gate (4b) states
  its own authority in its detail string, and the README repeats the AMBER finding rather than
  burying it. The imprint is state realism, and the spec says so in the same sentence that
  introduces it.
- [The imprint lags ignition by one step] → specified as known behavior with the reason (`pre_step`
  precedes the coupling), so a reader of the provenance log sees an expected one-step lag rather
  than a suspected bug. If the lag turns out to matter for the fork witnesses, that is a measured
  finding for M-later, not a silent artifact.
- [M4 has not landed] → Acts 2 and 4 and gates (2) and (6) consume M4's guidance, live envelope
  enforcement, and terminal re-seed. This change is derived ahead of M4 and its tasks are ordered so
  the M4-dependent groups come after the M4-independent ones, but it MUST NOT be applied before M4
  archives. Stated in the proposal's Impact and in task group 0.
- [Bands cannot be pinned before the first run] → the earned-then-regressed convention, unchanged
  from the corridor and from `srp-derisk-verdict`: the first measured run pins, later runs gate
  regressions, and a re-pin is recorded with its reason rather than quietly edited.
- [The 600 s budget] → the corridor spends 42 s; this example adds roughly 1200–1800 further coupled
  steps plus one branch fan-out that runs concurrently through the existing `scoped_map` seam. The
  estimate is low minutes, but it is an estimate: gate (9) is the one that decides, and if it fails
  the finding is a resolution or roster decision, not a band relaxation.

## Migration Plan

Additive throughout. The corridor and weather examples are untouched; their `output.txt` captures
must reproduce bit-identically, which gate (1) asserts for Acts 0–1 with the full burn stack present
and the throttle at zero. The new `[[example]]` stanza appends to `Cargo.toml` (the house
append-at-end convention). `src/shared/` gains a coupling assembler, a burn-seeded initial field, a
propulsion constants section, and two example-local stages; nothing existing in `shared` changes
signature, so both existing examples keep compiling unchanged.

## Open Questions

- **Two M1 figures in `derisk-verdict.md` are not reproducible from the artifact the note itself
  cites.** The verdict's prose records fork setup **83 ns** and a per-branch continuation ratio of
  **0.68–1.05×** (`derisk-verdict.md:32`, repeated in the pinned-bands table at `:60` and in
  `studies/qtt_rank_plume/main.rs:89`), but the committed evidence
  `deep_causality_cfd/studies/qtt_rank_plume/output.txt:23-30` records **42 ns** and **0.67–1.04×**,
  and git history shows that file has only ever said 42 ns. The bond figures (16, flat, under 32)
  and the L2 spread (0.18 / 0.28 / 0.35 / 0.61) do reproduce exactly. This change cites the
  artifact. Under the note's "[measured] throughout" convention the prose should be re-run and
  re-pinned against a committed artifact, or corrected as a transcription error — a decision for the
  note's owner, not for M5 to make silently, since `derisk-verdict.md` is the authority M3 already
  archived against.
- ~~Where the ignition-window `q` bounds are enforced.~~ **Resolved by M4**
  (`add-retropulsion-terminal-descent`): `CyberneticCorrect` enforces `[q_min, q_max]` as a refusal
  on a throttle rising from zero outside the window, and does not re-apply it as a running
  constraint once the burn is under way. Gate (2) reads the commit event M4's
  `ignition-corridor-commit` capability logs.
- The branch roster size is pinned at five (coast, two sign-flip straddlers, nominal,
  engine-degraded) on the design note's guidance. Whether a refinement round around the committed
  throttle is warranted is a first-measured-landscape decision, exactly as the corridor's
  coarse/fine pattern was.
- Whether the terminal leg's subsonic re-seed wants its own `γ → 1.4` and retuned `S_REF` as module
  constants or as a second `DescentSchedule` is an M4 shape question this example consumes rather
  than decides.
