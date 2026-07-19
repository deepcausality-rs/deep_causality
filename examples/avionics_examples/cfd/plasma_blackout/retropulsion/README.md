[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Plasma Retropulsion: From Blackout Exit to Touchdown

The corridor example flies a reentry through plasma blackout and stops at 47 km. The weather
example generates a dispersion table across atmospheric days. This example closes the loop: it
**reads that table in flight**, commits an ignition, and carries the same vehicle to the ground
under a retro burn.

Supersonic retropropulsion is the hard part of landing anything heavy. Firing an engine forward
into a hypersonic freestream does not simply add thrust — the plume displaces the bow shock and
destroys the aerodynamic drag the vehicle was relying on. Jarvinen and Adams measured the
collapse in 1970; past a thrust coefficient near 3 the interaction goes unsteady. A vehicle that
lights its engine too hard arrives faster than one that coasts.

This example flies that trade as a **counterfactual on a marched flow state**. Mid-burn, the
plume-coupled state is forked once per candidate throttle and each branch continues from the same
instant in its own alternated world. The run self-verifies through fourteen gates and exits
nonzero on any regression. Wall-clock is about four minutes.

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

Five acts. Every leg boundary is an event the run *finds* — a flow-resolved crossing or a
guidance commit — not a scripted station switch.

**Act 0 — PLAN.** The measured day is a cold one, 32 K below standard. Its temperature departure
interpolates `weather_table.csv`, the artifact the weather example recorded, and the interpolated
navigation-drift row sizes the ignition margin: 62.9 m of mean drift plus three sigma gives 73.4 m.
A standard-day belief demands 53.0 m from the same table. That 20.4 m separation is the whole
reason the table exists, and gate (5) enforces it.

**Act 1 — CORRIDOR.** The inherited descent, flown with the burn stack already composed and the
throttle commanded to zero. This is deliberate: the thrust and plume stages are strictly inert at
zero throttle, so composing them early costs nothing and keeps the safety envelope live on every
step. Gate (1) checks that the corridor still flies as the corridor.

**Acts 2+3 — COAST, COMMIT, BURN.** One march call, not two. A coupling stack is fixed per call
and the marched fluid tensor is not carried across a leg boundary, so splitting at ignition would
re-seed the flow and the fork below would fork a state with the plume already discarded. The
vehicle coasts from 76 km down through the Jarvinen-Adams band and the ignition corridor commits
at 32.7 km, Mach 2.00, 2235 Pa — inside the Mach band, inside the dynamic-pressure window, on a
post-fix navigation state within the table-sized margin.

**The fork.** The marched, plume-coupled state is forked in O(1) through copy-on-write, once per
candidate throttle: a coast branch, two straddling the drag sign-flip band, a nominal branch, and
an engine-degraded contingency. The preserved-drag fraction collapses 1.000 → 0.217 across the
roster — the cited A0 correlation, carried per branch through a genuinely forked flight. Four
gates read the fork: flow spread (4a), the drag collapse (4b), departure from a frozen-drag
prediction (4c), and the audit trail (4e).

**Act 3b — BURN.** The supersonic retropulsion leg proper, flown under the SRP envelope until the
vehicle drops to Mach 0.6 at 18.9 km — which is where those axes stop describing the physics.

**Act 4 — TERMINAL.** Cutoff, a subsonic re-seed under its own gamma, and the landing. The
guidance coasts to the ignition altitude, lights once at 157 m and 49 m/s, and touches down at
**2.0 m/s with 1372 kg of propellant remaining**.

## The Landing Burn Is Bang-Bang, and That Is Not a Detail

Three physics corrections stand behind that touchdown, each recorded in
[`constants.rs`](constants.rs) beside the band it superseded. They are worth naming because each
one *looked* like a result before it was understood to be an error.

**The SRP envelope does not apply subsonically.** The `C_T ≤ 3` cap and the throttle floor are
bow-shock-interaction statements from a dataset spanning Mach 0.4–2.0. Carried down to the deck
they make the admissible throttle collapse with dynamic pressure and forbid the engine entirely
around 3.7 km — not because the vehicle cannot brake, but because it is not allowed to try. The
terminal leg flies its own subsonic envelope. Landing at 160 m/s was not a finding about
retropropulsion; it was a constraint applied outside its validity envelope.

**Continuous throttle wastes the propellant.** The closed form `a_cmd = v²/2h + g` degenerates to
`a_cmd ≈ g` when `h` is large — thrust balancing weight. A vehicle that commits high nulls its
descent rate at altitude and then *hovers*, spending propellant to hold station. Ours ran dry at
10.5 km. Meditch (1964) showed the fuel-optimal control for the soft-landing problem is
**bang-bang**: null thrust, then maximum thrust. Coast-then-burn is not a trick to save propellant,
it is the optimal structure, and any sustained intermediate throttle is wasteful by construction.
The guidance now holds at zero until the altitude falls to `ignition_altitude_kernel`'s answer.

**A lander arrives moving.** The stopping law must target the gear contact plane, not the
geocenter, and at a commanded contact speed rather than at rest. Falcon 9 touches down near 2 m/s,
and that is a design point: its landing engine at minimum throttle out-thrusts the nearly empty
stage, so it *cannot* hover and must be flown into the deck. This vehicle throttles deep enough
that it could hover, and is commanded into the deck anyway — for firm gear contact, and because a
guidance nulled at exactly the sampled altitude would report its own setpoint back as a
measurement.

## What the Counterfactual Measures, and What It Does Not

The M1 de-risk measurement came back **AMBER** on imprint fidelity: a compressible forcing region
does not reproduce the Jarvinen-Adams drag collapse at this fidelity. So the in-flight drag
authority is the **cited A0 correlation**, not a decrement contracted from the field. Saying
otherwise would be the whole example's credibility.

What M1 measured *green* is the state-fork machinery itself — an O(1) copy-on-write fork whose
branches spread with the intervention. So the fork here is a genuine fork of the marched state,
carrying flow-realism and fork-economics witnesses that a parameter sweep cannot express at all.
Gate (4c) is the one that earns it: the branches depart a frozen-drag prediction by up to
4.23 m/s², so thrust-only kinematics does not predict the outcome.

## Limitations

Every simplification is documented in [`constants.rs`](constants.rs) and in the shared
[`constants.rs`](../../../src/shared/constants.rs).

1) The plume's effect on drag is the cited A0 correlation evaluated at each branch's thrust
   coefficient, not a decrement measured from the marched field (the AMBER verdict above). The
   optional `PlumeImprint` seam gives the marched layer state realism, and nothing more.

2) On-axis magnitudes only. There is no angle of attack in the roster, per the design note's
   discipline pin, so the lateral plume-shock interaction is out of frame.

3) The guidance is the Tier-A closed form. Apollo polynomial guidance (Klumpp 1974) and convex
   powered-descent guidance (Açıkmeşe & Ploen 2007, the lossless-convexification result behind
   modern precision landing) are the named upgrade path, not this example's scope.

4) The corridor limitations carry over unchanged: single-point sheath chemistry, compressed time,
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
- The corridor's own anchors — RAM-C II, NASA RP-1232, Park, Millikan-White, Sutton-Graves — carry
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
