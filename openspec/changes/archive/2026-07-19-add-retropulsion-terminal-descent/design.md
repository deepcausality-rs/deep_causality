## Context

M4 is the guidance half of the design note's Stage 3. Its inputs are landed: the three Tier-A
descent kernels shipped in Stage 1, M2 landed the throttle channel and the `BurnEnvelope`, and M3
landed the thrust and plume stages plus the flight-regime axes. What is missing is everything that
*decides* a throttle and everything that makes the envelope real in flight.

The survey found the gap has two halves, and separating them matters for scoping.

**Absences** — things nobody has built yet: a guidance law, a commit predicate, the `q`-window read
site, and production producers for `"q_inf"` and `"descent_rate"` (the gate's own unit tests publish
both, which is why the two axes demonstrably work under test while having no path to work in
flight).

**Defects** — things that exist and are wrong:

- `clamp(commanded, throttle_floor, ceiling)` has no `lo ≤ hi` precondition (`corridor.rs:1031`,
  helper at `:1081-1088`), and it tests the **lower** bound first. When
  `ct_ceiling = max_ct·q∞·s_ref/thrust_ref` falls below `throttle_floor`, the gate emits one of two
  out-of-envelope values chosen by the command rather than by any rule: a command between the bounds
  is clamped **up to the floor**, exceeding the `max_ct` cap, and a command at or above the floor is
  clamped **down to `ct_ceiling`**, below the floor. With the existing test's own numbers (floor 0.1,
  `max_ct` 2.0, `s_ref` 0.785, `thrust_ref` 2000) the window crosses below roughly 127 Pa. The floor
  is a stability constraint for central-nozzle SRP — the physics crate says so at
  `kernels/propulsion/srp.rs:217-220` — so one branch commands an unstable operating point and the
  other violates the cap that exists to prevent it.
- The bank/thermal refusal returns at `corridor.rs:972-980`, before the burn block at `:985`. A heat
  or g breach therefore **masks** a simultaneous propellant-floor or descent-rate breach: neither is
  logged. This is a diagnostic loss rather than a safety hole — the `Err` short-circuits every stage
  after the gate (`coupling.rs:332-334`), so no throttle consumer runs and no out-of-envelope command
  is ever realized — but the step that most needs a complete account reports only one of its causes.
- `self.rebuilds` is incremented and logged and compared against nothing
  (`compressible_march_run.rs:352`), and no accessor exposes it. The carrier is not unbounded: the
  `(1 + rebuild_tol)` trigger against an unconditional `s_ref ← 1.2·s_needed` re-pin is a hysteresis
  ratchet requiring roughly `1.44×` further growth per rebuild at the default tolerance, so the doc
  comment at `:79` describes a real mechanism. What is missing is a bound anyone can state or check
  — the only machine-checkable budget is the corridor example's post-hoc gate counting
  `"carrier rebuilt at step"` substrings in a rendered log, which detects rather than enforces and
  breaks silently if the wording changes.
- A leg boundary discards the marched conserved state — `.from(prev.state())` carries only the
  `CoupledField`, and the new leg re-quantizes the world's uniform `seed_fn` — and writes **no log
  entry**. The fork path, by contrast, both preserves the marched state (`Arc`-resumed) and logs
  `"march resumed at step N for M steps in world 'X'"`.

Two further facts shape the design rather than needing repair. The throttle channel **latches**: it
is an `Option<R>` never cleared between steps and carried by the derived `Clone`, so a forked branch
inherits its parent's last command. And the carrier's `ReferenceScales` are fixed per config and
anchored to the corridor's reentry conditions (`T_REF` 8044 K, `N_REF` 2.645e22, `U_REF` 376 m/s),
documented as "never varied, so the marched numbers stay O(1) across the whole descent" — a premise
that does not survive to sea level, where `n_tot` is 2.5e25 and `ρ̂` runs three decades off O(1).

## Goals / Non-Goals

**Goals.**

- A throttle that is commanded by a cited closed form, clamped by an envelope that actually
  enforces, and felt by the trajectory and the IMU through the force channel M3 already built.
- An ignition commit that is an auditable event with defined edge semantics, not an implicit
  side effect of a throttle becoming positive.
- Every envelope axis either enforced or explicitly declared inert — no axis that reads zero and
  looks safe.
- A terminal leg whose re-seed is visible in provenance and whose rebuild budget has teeth.

**Non-Goals.**

- No Apollo polynomial guidance (Klumpp 1974) and no convex powered-descent guidance
  (Açıkmeşe–Ploen 2007). Both are named in the kernels' own docstrings as the upgrade path; Tier A
  is the closed form.
- No example. M4 verifies through a harness test, as M3 did; M5 owns the example folder.
- No re-anchorable or altitude-staged `ReferenceScales` mechanism. M4 supplies a terminal-leg config
  with its own anchors, which the existing per-config field already permits; a staged mechanism that
  re-anchors *within* a leg is a larger change and is recorded as an Open Question.
- No repair of the leg-boundary fluid discontinuity itself. That is the design note's §5
  quasi-steady defense, deliberately accepted; M4 makes it **visible**, not absent.
- No change to the `!!ContextAlternation!!` vocabulary or the existing log message texts the corridor
  example greps.

## Decisions

**D1 — `ThrottleGuidance` is a library stage, and it is the throttle's only producer.** It belongs in
`deep_causality_cfd` beside `RetroThrust` and `PlumeObstruction` by the split the example's own
module docstring states: library stages are generic in `R` and `const D`, parameterized by
construction, with configurable field names; example stages are hardcoded to `PhysicsStage<2,
FloatType>` and fixed literals. The guidance law is built from cited physics kernels and is reusable,
so it is library. The world-specific corridor thresholds stay in the world.

**D2 — The `a_cmd → θ` map is the crate's existing linear convention, not a new kernel.**
`suicide_burn_deceleration_kernel` returns an `Acceleration`, and its docstring names the
clamp-into-envelope step as downstream. There is deliberately no thrust-from-throttle kernel: the
performance module's docstring says "propellant bookkeeping and throttle mapping live in the
CFD-side stages", and `RetroThrust` already hardcodes `T = θ·T_full`. `ThrottleGuidance` inverts the
same relation — `θ = m·a_cmd / T_full`, saturated to `[0, 1]` before the envelope sees it — so the
two stages share one convention rather than each inventing a map.

**D3 — Guidance commands zero from step 0, so the envelope is live before ignition.** The gate's burn
block is guarded on `field.throttle_action()` being `Some`. If guidance wrote the channel only at
commit, then on every pre-ignition step the propellant floor, the descent-rate bound, and the
throttle clamp would all be unenforced. So the stage writes the channel on **every** step, θ = 0
before commit, and relies on `RetroThrust`/`PlumeObstruction` being strictly inert at θ ≤ 0 — the
inertness contract M2 established and M3 preserved. Ignition stays a published-command event; it is
now an event on the *value*, not on the channel's existence.

**D4 — Commit is a rising edge over a latched channel, then a latch of its own.** Because the
throttle channel never clears, "the step on which thrust begins" cannot be recovered from the
channel alone, and there is no edge detector in the crate. The commit therefore carries its own
one-way latch: the conjunction is evaluated each step until it first holds, that step is logged as
the commit event, and thereafter the burn is committed regardless of whether a condition
momentarily lapses. Rationale: an ignition corridor is a decision about when to start, not a
condition to be maintained — re-opening it every step would let a transient nav dropout extinguish a
burn that is already the only survivable option. The latch rides a field scalar so it survives the
leg boundaries that reset carrier-internal state.

**D5 — The commit reads published nav scalars, never `field.nav()`.** Navigation is a one-way sink
in the library today: no stage feeds the nav estimate back into physics or control, and the aero and
control stages steer off the carried truth state and flow scalars. `TrajectoryNav` does take and
return the engine, and does branch on nav-derived state — but only to decide whether to run and
whether to log a mode transition, never to steer anything. So the commit is the first place a
*decision that changes the flight* is taken on navigation quality. It reads `"nav_mode"` and
`"nav_position_variance"`, the scalars published for exactly this purpose and currently consumed by
no production code — the corridor example instead reaches past them into the engine's accessor to
gate on position variance, which is precisely the coupling this decision avoids. The pattern to
follow is `TrajectoryNav`'s own GNSS gate, which reads the published `RegimeClass` rather than
reaching into another subsystem. Two consequences are specified rather than assumed:
`nav_mode = 1.0` means aided by **either** a GNSS or a through-plasma optical fix — the distinction
is not published — and `nav_position_variance` is a covariance **trace in m²**, so a margin stated
in metres compares against its square root.

**D6 — A crossed clamp window is a breach, and the fix cannot be one-sided.** When
`ct_ceiling < throttle_floor` there is no admissible throttle: the dynamic pressure cannot support
even the minimum stable command. The gate refuses — logs and returns `Err`. The two bounds encode
different physics (a stability floor and a bow-shock-instability ceiling), so their crossing is a
real operating-point failure and reporting it is the honest action. Both nearby repairs are
rejected, and for the same reason: holding at the floor emits a command the `max_ct` axis exists to
forbid, and reordering the clamp helper so the upper bound wins would satisfy a "never below the
floor" reading while silently violating the cap instead. That is why the requirement is written
against the crossed window rather than against either bound — a one-sided obligation here is
gameable by construction.

**D7 — All axes are sensed and decided before any refusal.** The gate senses every axis, collects
every breach, logs each one, and then returns the refusal. This removes the masking at
`corridor.rs:972-980` and matches the "evaluate all, then decide" discipline the study gate sequences
already give by construction. The motivation is diagnostic, not corrective: the current ordering is
already safe, because the refusal short-circuits every later stage, so what is lost is the record of
*why* a step failed rather than any enforcement. The returned error names the first breach in a
fixed axis order so the error string stays deterministic.

**D8 — A gate that cannot see the throttle refuses, and the existing requirement is amended to say
so.** The gate keeps sensing the channel only. Widening it to the `"commanded_throttle"` scalar is
rejected on a mechanism, not a preference: the gate's clamp writes the bounded value **back into the
channel**, and once a channel exists it outranks the published scalar in the crate's
channel-or-scalar precedence — so a counterfactual branch driven by a published constant would be
frozen at its first clamped value on every later step, silently destroying the intervention seam
M5's fork study rides on. Instead the gate detects the one genuinely dangerous configuration — burn
axes attached **and** a positive `"commanded_throttle"` scalar **and** no channel — and refuses.

This refusal fires *inside* the configuration `powered-descent-envelope`'s "Inactive axes change
nothing" requirement appeared to protect, so that requirement is **modified** rather than merely
extended. Its original wording keys invisibility to "no throttle channel was written", while its own
closing sentence keys it to a throttle command existing — and the crate defines a commanded throttle
as channel-or-scalar (`retro-thrust-stage`, `retropulsion.rs:86-92`). The two readings disagree
exactly on the scalar-only case. The amendment re-keys invisibility to **both seams being quiet**,
which preserves the corridor guarantee the requirement exists for — a corridor world publishes no
`commanded_throttle` and drives no channel — while removing the ambiguity that would otherwise make
this change silently non-conforming.

**D9 — The sensor producers are one library stage, taking the mean molecular mass by construction.**
The carrier could compute `descent_rate` unaided (it holds `r` and `v` at `pre_step`) but cannot
compute `q∞`: it has `n_tot` in m⁻³ and no mean molecular mass, and `deep_causality_physics` carries
no air `m̄` either. Rather than push a species constant into the carrier or the physics crate, a
stage takes `m̄` as a constructor parameter and publishes both scalars — following the example's
`FreestreamFeeds`, which already does exactly this `n·m̄` conversion. Both field names are
configurable, matching the gate's own sensing configuration.

**D10 — The leg re-seed is logged; the fluid discontinuity itself is not repaired.** The carrier
writes an entry at the start of a leg that names what the boundary discarded and what it re-seeded
from, mirroring the fork path's resume line. This makes the quasi-steady defense auditable — a
reader of provenance sees the re-seed rather than inferring it — without changing the semantics the
corridor already flies and gates against.

**D11 — The rebuild budget gains an explicit bound and a reader; the ratchet stays.** The carrier
already bounds rebuilds by hysteresis — the `1.2×` re-pin against a `(1 + tol)` trigger — and that
mechanism is kept. What is added is a config bound checked in-loop, like the plume imprint's
`max_refreshes`, plus an accessor for the count. Where the imprint cap simply stops refreshing — the
imprint is state realism, so degrading it is harmless — the rebuild bound returns `Err`: a leg
needing that many re-pins is not converging on an envelope, and its numbers should not be reported
as results. The accessor matters as much as the bound: today the only machine-checkable budget is an
example gate matching `"carrier rebuilt at step"` in a rendered log, which silently stops working if
the message is ever reworded, and which counts one leg's rendered log rather than the run.

## Risks / Trade-offs

- [New `Err` paths in a gate M2 froze as bit-identical] → every new refusal requires `burn` axes to
  be attached, and no world that omits `with_burn` reaches any of them. The corridor's inheritance
  guard re-runs unchanged, and the existing inactive-axes requirement keeps its exact meaning.
- [The commit latch could hide a genuinely lost corridor] → the latch is one-way by design (D4), so
  the failure mode is committing to a burn whose preconditions later lapse. Mitigated by the
  envelope, which keeps enforcing every step after commit: a burn that becomes unsurvivable breaches
  the propellant floor or the descent-rate bound and refuses there, where the breach is real, rather
  than being pre-empted by a transient sensing dropout.
- [`nav_mode` conflates GNSS and optical fixes] → specified as a known limitation rather than
  papered over. If an ignition corridor must require GNSS reacquisition specifically, publishing the
  fix source is a follow-on, and this design names it in Open Questions instead of silently treating
  optical as equivalent.
- [Terminal reference scales are supplied, not derived] → M4 pins a terminal-leg config with its own
  anchors, which is honest for the leg it configures but does not solve the general problem that a
  single `ReferenceScales` cannot span 90 km to sea level. Recorded as an Open Question; the risk is
  that the terminal leg's own anchors are themselves a guess until the first measured run pins them.
- [The rebuild path is not taken on the corridor's own trajectory] → the corridor's committed run
  records zero rebuilds despite `S_REF` being documented as deliberately snug so the mechanism
  "fires where the descent steepens" — a doc comment the committed output contradicts and which
  should be corrected or the constant re-pinned. The path is not untested (four integration tests
  drive it with an undersized `s_ref`, and the audit-sink suite depends on it firing), but it is
  unexercised on a real descent leg, so M4's bound lands with harness coverage on a trajectory
  rather than inheriting the assumption that it works in flight.
- [The one-step actuation lag] → inherited from the bank channel, not introduced here, and specified
  rather than discovered. It means a commit at step *k* first produces thrust at step *k+1*.

## Migration Plan

Additive for every world that does not attach burn axes: `SafetyEnvelope::new` still yields
`burn: None`, and the corridor composes no burn axes, so the corridor and weather examples re-run
bit-identically. Worlds that *do* attach burn axes gain the new refusals — which is the point, since
today those worlds fly unenforced whenever they drive the scalar rather than the channel. The leg
re-seed entry adds one provenance line per leg boundary; the corridor's rebuild gate counts a
distinct substring and is unaffected, but its recorded `output.txt` provenance block gains the new
lines and is regenerated as part of this change.

## Open Questions

- **Where the terminal leg's reference scales come from.** M4 supplies them as pinned constants for
  the terminal configuration. Whether they should instead be derived from the previous leg's
  published projections, or whether the carrier should support staged re-anchoring within a leg, is
  a larger change this milestone deliberately does not take. The first measured terminal run is what
  would tell us whether pinned anchors suffice.
- **Whether the ignition corridor must require a GNSS fix specifically.** `nav_mode` does not
  distinguish a GNSS fix from a through-plasma optical fix. If the corridor's intent is
  "post-reacquisition", that distinction is load-bearing and a fix-source scalar must be published;
  if the intent is "the navigated state is aided at all", the current scalar suffices. M4 assumes
  the latter and states the assumption in the spec.
- **Whether the rebuild cap should be per-leg or per-descent.** The carrier's counter resets on every
  `build()`, so a cap enforced in-carrier is necessarily per-leg, while the corridor's existing
  post-hoc gate counts across the whole descent by grepping the accumulated field log. M4 enforces
  per-leg in the carrier and leaves the descent-wide count to the example gate; whether the
  descent-wide budget should also be a carried field scalar is unresolved.
