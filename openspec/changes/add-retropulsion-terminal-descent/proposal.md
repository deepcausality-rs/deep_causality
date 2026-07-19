## Why

The plasma-retropulsion roadmap
(`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md`, milestone M4 /
design-note Stage 3, guidance half) guides the burn and lands it. M3 landed the physics — thrust in
the force channel, the A0 drag decrement, the regime axes — but nothing commands a throttle and
nothing enforces the burn envelope in flight. The survey behind this change found the gap is wider
than "add a guidance law", and in two places it is a defect rather than an absence:

- **Nothing outside tests writes the throttle channel.** `set_throttle_action` is called only by
  the gate echoing its own clamp back and by snapshot resume. The commanded throttle is a two-source
  seam — the propulsion stages read the channel **or** the published `"commanded_throttle"` scalar,
  and a study drives the scalar today — but the gate's burn block reads the channel alone, so a
  scalar-driven world runs its full propulsion path unenforced. No composed world attaches a
  `BurnEnvelope` at all, so the powered-descent envelope is unreachable outside tests for that
  reason first.
- **Two of the five sensed axes have no production producer.** `"q_inf"` and `"descent_rate"` are
  read by the gate and bounded by `BurnEnvelope`, and written only by the gate's own unit tests —
  which is why both axes demonstrably work under test while having no path to work in flight.
  Because each read is `peak(...).unwrap_or(zero)`, a missing producer reads as **0**, so the first
  world to attach burn axes without publishing them **fails open**: the descent-rate bound cannot
  fire and the dynamic `C_T` cap collapses to the static ceiling, with nothing in the run saying so.
- **The ignition `q` window is stored and never read.** `BurnEnvelope::q_min`/`q_max` exist as
  fields, constructor parameters, and doc comments saying "stored for M4, not gated here" — with
  zero read sites and zero tests.
- **A crossed clamp window emits an out-of-envelope throttle in either direction.** The interval is
  `[throttle_floor, min(ceiling, ct_ceiling)]` with no `lo ≤ hi` precondition, and the helper tests
  the lower bound first. So when dynamic pressure is low enough that the dynamic `C_T` ceiling falls
  below the throttle floor, a command between the two bounds is clamped **up** past the `C_T` cap,
  while a command at or above the floor is clamped **down** past the floor — and that floor is a
  stability constraint for central-nozzle SRP. Neither branch is admissible, so the fix cannot be a
  one-sided bound.
- **A heat or g breach masks a simultaneous burn breach.** The bank refusal returns before the burn
  block is reached, so a propellant-floor or descent-rate breach on the same step is neither logged
  nor raised. This costs diagnosis rather than safety — the refusal short-circuits every later
  stage, so no throttle consumer runs — but it means the step that most needs a complete account
  reports only one of its causes.
- **The rebuild budget has no statable bound and no reader.** The carrier is not unbounded — the
  `1.2×` re-pin against a `(1 + tol)` trigger is a hysteresis ratchet requiring roughly `1.44×`
  further growth per rebuild — but the counter is compared against nothing and exposed by no
  accessor, so the only machine-checkable budget in the workspace is an example gate that tallies
  log substrings in a rendered provenance string.
- **A leg boundary discards the marched fluid state, unlogged.** `.from(prev.state())` carries only
  the `CoupledField`; the conserved state is re-quantized from the world's uniform seed, and the
  carrier's `inflow`, `s_ref` drift, rebuild count, and plume-imprint budget all reset. The fork
  path logs its resume; the leg path logs nothing at all, so the single most consequential event at
  a leg seam is invisible in provenance.

## What Changes

- A **`ThrottleGuidance`** stage in `deep_causality_cfd` — the first producer of the throttle
  channel. It forms the commanded deceleration from the Tier-A closed form
  `a_cmd = v²/2h + g` (`suicide_burn_deceleration_kernel`), maps it to a throttle through the
  crate's existing linear convention `T = θ·T_full`, and writes `set_throttle_action`. It composes
  after the navigation stage (so it reads the current step's nav quality) and before the gate (so
  the gate clamps what it wrote), which gives the same **one-step actuation lag** the bank channel
  already has: the thrust stages, composed earlier, fly the previous step's clamped command.
- An **ignition-corridor commit** with explicit event semantics: the conjunction of Mach band,
  `q∞ ∈ [q_min, q_max]`, a post-fix navigation state, and the table-sized margin, evaluated as a
  **rising edge** and then **latched**, so a momentary condition loss does not extinguish a burn.
  The commit is logged as a transition entry, the pattern the nav-mode flip already uses.
- **Live envelope enforcement** closes the defects above: the `q` window becomes an enforced axis; a
  crossed clamp window becomes a refusal rather than a choice between two out-of-envelope bounds;
  burn axes are sensed and decided alongside the bank axes so no breach goes unlogged; and a burn
  world that commands thrust the gate cannot see — axes attached, a positive `"commanded_throttle"`
  scalar, no channel — is refused loudly instead of flying unenforced. The gate's sensing is
  deliberately **not** widened to the scalar: its clamp writes back into the channel, and a channel
  written from a scalar source would then outrank that world's published constant on every later
  step, freezing a counterfactual branch at its first clamped value.
- **The two missing sensor producers**, as a library stage: `"q_inf"` from the freestream number
  density, the flight speed, and a configured mean molecular mass, and `"descent_rate"` from the
  truth state as `ḣ = −(r·v)/|r|`, with the sign convention pinned so that descent is positive (the
  gate tests `descent_rate > bound` after a `peak` reduction).
- **The terminal leg**: a logged re-seed entry at every leg boundary that names what was discarded,
  an explicit rebuild bound with a programmatic reader rather than a log-substring tally, and a
  terminal-leg configuration that separates the two gammas the carrier already keeps apart — the
  schedule's `gamma_eff` (the Rankine–Hugoniot jump) and the marcher's `gamma` — with a retuned
  acoustic reference and terminal reference scales, since the corridor's are anchored to a 90 km
  Mach-24 post-shock state and the rebuild trigger, which is keyed on wave speed alone, never
  corrects a density anchor.

**BREAKING**: `CyberneticCorrect` gains enforcement paths that can return `Err` where it previously
returned `Ok` (the `q` window, the crossed clamp window, the blind-gate case). All three require
burn axes to be attached, so every world that does not call `with_burn` is unaffected and the
corridor stays bit-identical. The existing `powered-descent-envelope` requirement "Inactive axes
change nothing" is **modified** rather than merely extended: its invisibility guarantee is re-keyed
from "no throttle channel was written" to "no throttle commanded on either seam", because the
crate's own definition of a commanded throttle is channel-or-scalar and the blind-gate refusal fires
inside the configuration the original wording appeared to protect.

## Capabilities

### New Capabilities

- `throttle-guidance-stage`: the stopping-distance guidance law, the `a_cmd → θ` map, the
  composition position and its one-step actuation lag, and the nav-quality read discipline.
- `ignition-corridor-commit`: the commit conjunction, rising-edge detection over a latching channel,
  the margin-sizing policy, and the logged commit event.
- `flight-sensor-scalars`: the `"q_inf"` and `"descent_rate"` producers, their units and sign
  conventions, and the reduction semantics the gate applies to them.
- `terminal-descent-leg`: the logged leg re-seed, the enforced rebuild cap, and the terminal-leg
  configuration (the gamma split, the retuned acoustic reference, terminal reference scales).

### Modified Capabilities

- `powered-descent-envelope`: the "Inactive axes change nothing" requirement is re-keyed to both
  throttle seams, resolving a conflict with the crate's channel-or-scalar definition of a commanded
  throttle; and the ignition `q` window, the crossed clamp window, the breach precedence, the
  blind-gate refusal, the silent `max_ct` no-op, and the per-axis reduction semantics all become
  specified behavior.

## Impact

- `deep_causality_cfd`: new `ThrottleGuidance` and flight-sensor stages; `types/flow/corridor.rs`
  gate enforcement extended; `types/flow/compressible_march_run.rs` gains the leg re-seed log entry,
  the enforced rebuild bound, and a rebuild-count accessor;
  `types/flow_config/compressible_march_config.rs` gains the rebuild-bound knob. Mirrored tests plus
  `tests/BUILD.bazel` registration, and a terminal-leg harness test — the first anywhere to compose
  the gate **with** burn axes into a marching stack.
- `deep_causality_physics`: consumed only (`suicide_burn_deceleration_kernel`,
  `stopping_distance_kernel`, `ignition_altitude_kernel`, `propellant_mass_flow_kernel`); no changes.
- Downstream: M5 (`wire-plasma-retropulsion-example`) composes all of this. Two M5 requirements are
  reconciled by this change — the `"q_inf"`/`"descent_rate"` producers move from example-local to
  library, and M5's open question about where the ignition `q` window is enforced is answered here.
