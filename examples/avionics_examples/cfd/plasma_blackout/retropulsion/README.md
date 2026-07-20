[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Plasma Retropulsion: From Blackout Exit to Touchdown

The corridor example flies a reentry through plasma blackout and stops at 47 km. The weather
example generates a dispersion table across atmospheric days. This example closes the loop: it
**reads that table in flight**, commits an ignition, and carries the same vehicle to the ground
under a retro burn.

Supersonic retropropulsion is the hard part of landing anything heavy. Firing an engine forward
into a hypersonic freestream does not simply add thrust; the plume displaces the bow shock and
destroys the aerodynamic drag the vehicle was relying on. Jarvinen and Adams measured the
collapse in 1970; past a thrust coefficient near 3 the interaction goes unsteady. A vehicle that
lights its engine too hard arrives faster than one that coasts.

This example flies that trade as a **counterfactual on a marched flow state**. Mid-burn, the
plume-coupled state is forked once per candidate throttle and each branch continues from the same
instant in its own alternated world. The run self-verifies through sixteen gates and exits
nonzero on any regression. Wall-clock is about six minutes.

## How to Run

From the repository root:

```bash
cargo run --release -p avionics_examples --example plasma_blackout_retropulsion
```

Precision is a parameter, as in the sibling examples. `main.rs` carries one alias and the whole
descent is generic over it:

```rust
pub type FloatType = f64; // or deep_causality_num::Float106
```

## What Happens When You Run It

Five acts. Every leg boundary is an event the run *finds*: a flow-resolved crossing or a
guidance commit, never a scripted station switch.

**Act 0, PLAN.** The measured day is a cold one, 32 K below standard. Its temperature departure
interpolates `weather_table.csv`, the artifact the weather example recorded. The interpolated row
sizes the ignition margin (62.9 m of mean drift plus three sigma gives 73.4 m), supplies the density
scale the atmosphere is flown at, and scales the accelerometer bias the inertial model carries. A
clamp (a measured departure outside the tabulated range) is stamped into provenance and gated.

**Act 1, CORRIDOR.** The inherited descent, flown with the burn stack already composed and the
throttle commanded to zero. This is deliberate: the thrust and plume stages are strictly inert at
zero throttle, so composing them early costs nothing and keeps the safety envelope live on every
step. Gate (1) checks that the corridor still flies as the corridor.

**Acts 2+3, COAST, COMMIT, BURN.** One march call, not two. A coupling stack is fixed per call
and the marched fluid tensor is not carried across a leg boundary, so splitting at ignition would
re-seed the flow and the fork below would fork a state with the plume already discarded. The
vehicle coasts from 76 km down through the Jarvinen-Adams band and the ignition corridor commits
at 32.7 km, Mach 2.00, 2235 Pa: inside the Mach band, inside the dynamic-pressure window, on a
post-fix navigation state within the table-sized margin.

**The fork.** The marched, plume-coupled state is forked in O(1) through copy-on-write, once per
candidate throttle: a coast branch, two straddling the drag sign-flip band, a nominal branch, and
an engine-degraded contingency. The preserved-drag fraction collapses 1.000 → 0.217 across the
roster. That is the cited A0 correlation, carried per branch through a genuinely forked flight. Five
gates read the fork: flow spread (4a), the drag collapse (4b), departure from a frozen-drag
prediction (4c), fork economics (4d), and the audit trail (4e).

**Act 3b, BURN.** The supersonic retropulsion leg proper, flown under the SRP envelope until the
vehicle drops to Mach 0.6 at 18.9 km, which is where those axes stop describing the physics.

**Act 4, TERMINAL.** Cutoff, a subsonic re-seed under its own gamma, and the landing, flown
**twice**. Both descents start from the same baseline on the same measured atmosphere and differ in
exactly one input: the margin their guidance was sized with.

| world | margin | lights landing burn | contact | propellant |
|---|---:|---:|---:|---:|
| informed | 73.41 m | 153.19 m | 1.78 m/s | +13.45 kg |
| uninformed | 52.98 m | 129.75 m | 1.85 m/s | — |

The margin reaches the flight through `ignition_altitude_kernel`, which adds it to the stopping
distance; a guidance that believes the day is more dispersed lights its landing burn higher. The
23.4 m between those two decisions is what carrying the table into the cockpit buys, and the 13.5 kg
the informed world spends to buy it is the dispersion-sized reserve, measured rather than configured.

The flown separation is 23.4 m against an arithmetic margin difference of 20.4 m. They differ because
the kernel solves a stopping distance rather than applying an offset: the extra margin also changes
the mass and speed the burn starts from. One number is arithmetic; the other is a flight, and gate (5)
reads the flight.

Note where the margin does **not** bind. At the ignition commit the navigated sigma is 0.38 m against
margins of 73.4 m and 53.0 m, two orders of magnitude inside either. Both beliefs commit on the same
step. A gate reading the commit would have reported the two worlds as identical; a gate subtracting
the two table lookups would have reported a separation neither world flew.

## The Landing Burn Is Bang-Bang by design

Three physics corrections stand behind that touchdown, each recorded in
[`constants.rs`](constants.rs) beside the band it superseded. They are worth naming because each
one *looked* like a result before it was understood to be an error.

**The SRP envelope does not apply subsonically.** The `C_T ≤ 3` cap and the throttle floor are
bow-shock-interaction statements from a dataset spanning Mach 0.4–2.0. Carried down to the deck
they make the admissible throttle collapse with dynamic pressure and forbid the engine entirely
around 3.7 km. The vehicle can brake there; it is simply not allowed to try. The
terminal leg flies its own subsonic envelope. Landing at 160 m/s was not a finding about
retropropulsion; it was a constraint applied outside its validity envelope.

**Continuous throttle wastes the propellant.** The closed form `a_cmd = v²/2h + g` degenerates to
`a_cmd ≈ g` when `h` is large: thrust balancing weight. A vehicle that commits high nulls its
descent rate at altitude and then *hovers*, spending propellant to hold station. Ours ran dry at
10.5 km. Meditch (1964) showed the fuel-optimal control for the soft-landing problem is
**bang-bang**: null thrust, then maximum thrust. Coast-then-burn is not a trick to save propellant,
it is the optimal structure, and any sustained intermediate throttle is wasteful by construction.
The guidance now holds at zero until the altitude falls to `ignition_altitude_kernel`'s answer.

**A lander arrives moving.** The stopping law must target the gear contact plane, not the
geocenter, and at a commanded contact speed rather than at rest. Falcon 9 touches down near 2 m/s,
and that is a design point: its landing engine at minimum throttle out-thrusts the nearly empty
stage, so it *cannot* hover and must be flown into the deck. This vehicle throttles deep enough
that it could hover, and is commanded into the deck anyway, for firm gear contact, and because a
guidance nulled at exactly the sampled altitude would report its own setpoint back as a
measurement.

## What the Counterfactual contributes

The M1 de-risk measurement came back **AMBER** on imprint fidelity: a compressible forcing region
does not reproduce the Jarvinen-Adams drag collapse at this fidelity. So the in-flight drag
authority is the **cited A0 correlation**, not a decrement contracted from the field. Saying
otherwise would be the whole example's credibility.

What M1 measured *green* is the state-fork machinery itself: an O(1) copy-on-write fork. So the
fork here is a genuine fork of the marched state, carrying flow-realism and fork-economics witnesses
that a parameter sweep cannot express at all. Gate (4c) is the one that earns it: over the same
continuation, each branch's realized velocity increment departs a frozen-drag prediction (its own
thrust schedule with the drag closure held at the fork's value) by up to 139 m/s. Thrust-only
kinematics does not predict the outcome.

**The two SRP models barely overlap, and that bounds what the fork can show.** Jarvinen-Adams
measured drag preservation over Mach 0.4-2.0; Cordell-Braun validated the plume boundary over Mach
2-4. They meet at a single point. The burn flies the correlation's band, so the plume geometry that
drives the marched-layer imprint is outside its own model's envelope for essentially all of it. Both
bands are declared at the call site and each stands down where it does not apply. The consequence is
that gate (4a)'s flow spread measures throttle → trajectory → post-shock density rather than a plume
footprint. It is still a flow observable the interventions move, and it still clears the corridor's
branch-invariance. It is not the imprint witness the design note imagined, and this example does
not claim it is.

Gate (4d) regresses the O(1) claim rather than trusting it, with one honesty correction. The two
sharing flags compare a clone against the `Arc` it was cloned from, so no *input* can falsify them;
they guard against a future edit that materializes the state instead of sharing it, and the README
previously overclaimed a reference count as "positive evidence" when that count was read on the line
after the clone and was above one by construction. What genuinely varies with the run is the
post-fork bond growth, which the gate now bands: measured 0, so a state that forks cheaply also stays
cheap through the continuation.

**Gate (4b) finds the sign flip.** Ordered by the throttle each branch actually flew, net
deceleration is non-monotone: 10.60 m/s² coasting, falling to 7.47 m/s² at 0.20 throttle before
rising again. In the low thrust-coefficient band the plume destroys preserved drag about as fast as
thrust replaces it, so lighting the engine buys *less* deceleration than coasting. Preserved drag
collapses 0.251 → −0.061 across the burning branches, reaching the correlation's negative, wake-type
branch at the harder throttles.

**The vehicle's reference area is derived, not stated.** `PLUME_S_REF_M2` follows from the flown
ballistic bundle: `CDA_OVER_M · VEHICLE_MASS_KG / C_d` = 14.09 m², a 4.23 m aeroshell at 172 kg/m²,
the `β ≈ 170` the bundle is documented as. Stated independently the two disagreed, implying a
drag coefficient of 4.29. That mattered: the thrust coefficient sets both the preserved-drag fraction
and the envelope's dynamic throttle ceiling, so the roster had been pinned to a ceiling that was an
artefact of the error.

## Limitations

Every simplification is documented in [`constants.rs`](constants.rs) and in the shared
[`constants.rs`](../../../src/shared/constants.rs).

1) The plume's effect on drag is the cited A0 correlation evaluated at each branch's thrust
   coefficient, not a decrement measured from the marched field (the AMBER verdict above). The
   optional `PlumeImprint` seam gives the marched layer state realism, and nothing more.

2) **Day-of-entry targeting is not implemented, and the measurement is why.** The design note asks
   the interpolated row to shift the deorbit aim point, on the premise that a colder day ionizes
   earlier and dwells longer. Both halves hold, and they cancel: across the whole tabulated range
   onset swings 4.2 s and dwell swings 3.1 s, while blackout **exit lands within 1.1 s on every day**.
   The gap that shift would protect runs from exit at ~70 s to the commit at 224 s: 154 s, perturbed
   by 1.1 s. The table's information is in the drift column (which swings 32.7 m), and the ignition
   margin already consumes it. A vehicle with a heavier ballistic bundle would reach the ignition band
   lower and later and squeeze that gap; this one does not.

3) On-axis magnitudes only. There is no angle of attack in the roster, per the design note's
   discipline pin, so the lateral plume-shock interaction is out of frame.

4) The guidance is the Tier-A closed form. Apollo polynomial guidance (Klumpp 1974) and convex
   powered-descent guidance (Açıkmeşe & Ploen 2007, the lossless-convexification result behind
   modern precision landing) are the named upgrade path, not this example's scope.

5) The corridor limitations carry over unchanged: single-point sheath chemistry, compressed time,
   a 2-D marched layer, and a deterministic 3-DOF flight world with no winds or aero dispersions.

## Anchors and Citations

- **Jarvinen & Adams (1970)**, *The aerodynamic characteristics of large angled cones with retro-
  rockets*: the A0 central-nozzle correlation, the preserved-drag collapse, and the `C_T ≈ 3`
  bow-shock instability bound.
- **Cordell & Braun**: the plume model, including the `P_exit/P_inf ≥ 7` blunt-flow transition the
  nozzle here is sized to stay above.
- **Meditch (1964)**, *On the problem of optimal thrust programming for a lunar soft landing*,
  IEEE Trans. Automatic Control: the fuel-optimal soft-landing control is bang-bang.
- **Açıkmeşe & Ploen (2007)**, *Convex programming approach to powered descent guidance for Mars
  landing*, JGCD: lossless convexification of the minimum-fuel problem under a thrust lower bound.
- The corridor's own anchors (RAM-C II, NASA RP-1232, Park, Millikan-White, Sutton-Graves) carry
  over through the inherited stack.

## Where Things Live

| File | Contents |
|---|---|
| [`main.rs`](main.rs) | The descent: five acts, the fork, provenance, the gates |
| [`model.rs`](model.rs) | The worlds, the table loader, branch scoring, every gate function |
| [`constants.rs`](constants.rs) | This example's knobs: horizons, the roster, the earned bands |
| [`utils_print.rs`](utils_print.rs) | Console rendering |
| [`output.txt`](output.txt) | A captured release run |

The vehicle, the propulsion constants, the atmosphere, the carrier anchors, and the powered-descent
coupling stack are shared with the [corridor](../corridor/README.md) and
[weather](../weather/README.md) examples through `avionics_examples::shared` (under
`examples/avionics_examples/src/`).

The library machinery this example exercises lives in `deep_causality_cfd` (the compressible
carrier, the coupled-loop seam, `ThrottleGuidance`, `RetroThrust`, `PlumeObstruction`, the burn
envelope, the study grammar), `deep_causality_physics` (the descent kernels and the A0
correlation), and `deep_causality_par` (the scoped fork-join).
