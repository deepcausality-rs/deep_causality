[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Plasma-Blackout Family

A reentry vehicle enters at Mach 25. Its own shock layer ionizes into a plasma sheath, and the
sheath cuts every GNSS link. The vehicle navigates through the dark, then comes back down under a
retro burn.

Three examples fly that story. They are not three variations on a theme: each one consumes what
the last produced, and the third closes the loop.

| | Example | What it does | Consumes | Produces |
|---|---|---|---|---|
| 1 | [corridor](corridor/README.md) | Flies one continuous Mach-25 descent through blackout, forking bank-angle counterfactuals at the onset | — | the validated baseline descent |
| 2 | [weather](weather/README.md) | Alternates that baseline into six weather worlds, flown concurrently, reduced to a dispersion table | the corridor's baseline | `weather_table.csv` |
| 3 | [retropulsion](retropulsion/README.md) | Reads that table **in flight**, commits an ignition, forks the marched plume-coupled state, and lands | both of the above | a landing at 2.0 m/s |

```bash
cargo run --release -p avionics_examples --example plasma_blackout_corridor
cargo run --release -p avionics_examples --example plasma_blackout_weather
cargo run --release -p avionics_examples --example plasma_blackout_retropulsion
```

Run them in that order the first time. The weather example writes the table the retropulsion
example reads.

## Why Blackout

Above roughly Mach 12 the shock layer ionizes. Past a critical electron density the sheath
reflects every GNSS frequency and the vehicle goes dark. NASA's RAM-C II flight measured this in
1970, and the family gates against that flight anchor.

Blackout is an honest hard case. It forces four things to be true at once:

* **The physics has to be real.** The blackout window is *predicted* from evolved chemistry, never
  scheduled. No onset constant exists anywhere in the corridor.
* **The navigation has to degrade.** GNSS denial is flow-resolved. When the computed electron
  density crosses the L1 cutoff, the classifier flips the link and a 17-state ESKF dead-reckons
  through the outage.
* **The counterfactuals have to be cheap.** Deciding a bank angle means flying candidate worlds
  from the same paused state, in O(1). Re-flying the descent per candidate does not scale.
* **Everything has to be auditable.** One provenance log carries regime transitions, nav-mode
  changes, bounded corrections, and alternation markers. Each run gates itself against that log.

## What Each One Adds

**The corridor** establishes the physics and the machinery: a tensor-train compressed compressible
carrier with a shock-fitted Rankine-Hugoniot inflow strip, Park two-temperature ionization on the
evolved state, flow-resolved GNSS denial, and a cybernetic gate whose clamped command actually
steers the 3-DOF lift. Its counterfactual is a **parameter** fork. Seventeen bank-angle branches
across two rounds resolve the optimum to half a degree.

**The weather example** turns that one descent into a distribution. Six atmospheric conditions
become six alternated worlds off the same validated baseline, at eight Monte Carlo receiver-noise
draws each. Forty-eight descents fly concurrently and reduce to a table of navigation drift
against weather. Its output is an artifact, not just a verdict.

**The retropulsion example** is where the counterfactual becomes a **state** fork. Supersonic
retropropulsion is the hard part of landing anything heavy: firing an engine into a hypersonic
freestream displaces the bow shock and destroys the aerodynamic drag the vehicle was relying on
(Jarvinen & Adams, 1970). So choosing a throttle means flying candidate throttles from the same
marched, plume-coupled instant. A parameter sweep cannot express that at all, because the thing
being forked is the flow state itself.

## Shared Ground

The vehicle, the atmosphere, the carrier anchors, the example-local physics stages, and the
coupling stacks live once, in `avionics_examples::shared` (under
[`examples/avionics_examples/src/`](../../src/)). That module also carries the precision notes.

Precision is a parameter throughout. Each example carries its own alias, deliberately per-example,
so each makes the choice for itself:

```rust
pub type FloatType = f64; // or deep_causality_num::Float106
```

The counterfactual fan-outs run on scoped threads through the workspace `parallel` feature
(`deep_causality_par::scoped_map`; no external dependency). Results are bit-identical to the
sequential run.

## The Library Machinery Underneath

| Crate | What the family uses it for |
|---|---|
| `deep_causality_cfd` | The compressible carrier, the coupled-loop seam, the corridor and propulsion stages, the burn envelope, the study grammar, the state fork |
| `deep_causality_physics` | Park two-temperature chemistry, Sutton-Graves heating, the descent-guidance kernels, the A0 correlation |
| `deep_causality_tensor` | The tensor-train compressed state the carrier marches |
| `deep_causality_par` | The scoped fork-join behind every fan-out |
| `deep_causality_core` | The alternation vocabulary (`!!ContextAlternation!!`) and the provenance log |

## Reading Order

Start with the [corridor](corridor/README.md). It explains the physics, the anchors, and the
machinery the other two inherit. Then [weather](weather/README.md), for the fan-out and the table.
Then [retropulsion](retropulsion/README.md), which assumes both.

Every example is self-verifying. Each ends in a numbered gate set merged into one verdict and
exits nonzero on any regression, so a stale claim in these READMEs fails a run rather than
surviving in prose.
