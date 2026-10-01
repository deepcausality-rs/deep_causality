[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Dynamic Drone Fail-Safe System

An inspection drone flies a high-voltage line at night along a mountain slope. Its satellite fix fails, its command
link goes behind the ridge, and a cell fails. A fail-safe has to bring it down somewhere it can stay: not on a slope
it will tumble down, not in the creek, not into the ravine, not near the crew parked on the road.

Fail-safes usually fail in one particular way. The system does exactly what it was designed to do, and that is the
wrong thing to do where it is. In July 2023, hundreds of light-show drones over Melbourne's Docklands broke off their
routine; per the operator they "did exactly what they should've done with any technical glitch", and landed in the
Yarra River. Safety engineering calls this an unsafe control action: one that becomes unsafe "in a particular context
and worst-case environment" (Leveson and Thomas, STPA Handbook, 2018).

Each part of this tutorial adds one layer of DeepCausality to the controller and flies the same drone through the same
night.

## The World

`src/` holds the world every part shares. The controller sees it only through the drone's telemetry.

| Feature | Where | Why it matters |
|---------|-------|----------------|
| Creek | valley floor, 0-12 m across | water |
| Grass slope | 30 deg, 12-120 m across | a drone on skids tips over above 15 deg |
| Access road | flat, 120-135 m across | flat ground, but a maintenance crew is parked on it |
| Rock slope | 25 deg, above the road | steep |
| Tower pads | flat 12 m squares under the line | the safe places to land |
| Ravine | 560-580 m along, above the road | a drop |
| Night wind | 2 m/s down the slope | without a satellite fix the drone cannot hold position and drifts with it |

The drone flies the line 100 m across the slope, 40 m above the ground, at 8 m/s. The faults arrive on a fixed
timeline: the fix degrades at 50 s and drops out at 55 s, the link drops at 65 s, and a cell fails at 85 s.

## Part 1: Dynamic Causality

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_1
```

The controller is one causal process with three stages each second:

1. `sense` takes the telemetry into the process.
2. `detect` evaluates four fault detectors, each a causaloid, as a collection: the collection says whether anything
   is wrong, each detector says what. The process state confirms a lost fix after 3 s and a lost link after 5 s, so a
   dropped packet does not trigger a fail-safe.
3. `decide` applies the standard fail-safe ladder: a critical battery, or a lost fix with a lost link, lands the drone
   where it is; a lost fix alone holds it for the operator; a lost link alone flies it home. Once an emergency begins,
   the fail-safe never steps back down in flight.

The process has no context: the controller knows only what the drone reports about itself.

```text
 time   across   along   agl  sats  hdop  loss   cell  confirmed faults                       fail-safe
   0s     100m      0m   40m    14   0.8    2%  4.05V  none                                   continue
  50s     100m    400m   40m     7   3.5    2%  3.95V  GNSS degraded                          continue
  57s      96m    440m   40m     3  99.0    2%  3.94V  GNSS lost                              hold
  69s      72m    440m   40m     3  99.0   95%  3.91V  GNSS lost, link lost                   land now
  85s      40m    440m   16m     3  99.0   95%  3.25V  GNSS lost, link lost, battery critical land now

Touchdown at 91 s, 28 m across and 440 m along: grass at 30 deg, nearest person 152 m away.
The ground was too steep: the drone tipped over and tumbled 16 m downhill, coming to rest on water. It is lost.
```

The controller confirmed every fault in order and took the textbook fail-safe each time. While it held and then
descended, the night wind carried it 72 m down the slope; it touched down on 30 deg grass, tipped over and tumbled into
the creek. When the fix dropped, the centre of a flat tower pad lay 10 m away. The controller had no way to know: it lacks context.

## Precision

`FloatType` in `src/lib.rs` sets the working precision of the whole tutorial. Part 1 produces the same output at
`f32`, `f64` and `deep_causality_num::Float106`.

## Where Things Live

| Path | Contents |
|------|----------|
| [`src/types/terrain/`](src/types/terrain) | `Terrain`: elevation, slope, surface, the crew |
| [`src/types/drone/`](src/types/drone) | `Drone`: its mission, fault timeline, `Telemetry` and response to a `Command` |
| [`src/types/touchdown/`](src/types/touchdown) | `Touchdown`: the ground truth of where the drone came down, and its `Outcome` |
| [`src/constants.rs`](src/constants.rs) | The world's dimensions, the drone, the fault timeline |
| [`part_1_dynamic_causality/`](part_1_dynamic_causality) | The fail-safe controller with dynamic causality alone |

## References

- ABC News, [Hundreds of drones plunge into Melbourne's Yarra River](https://www.abc.net.au/news/2023-07-16/hundreds-of-drones-plunge-into-yarra-river/102607576), 16 July 2023.
- N. G. Leveson and J. P. Thomas, [STPA Handbook](https://www.flighttestsafety.org/images/STPA_Handbook.pdf), MIT, 2018.
