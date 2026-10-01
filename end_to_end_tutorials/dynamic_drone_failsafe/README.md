[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Dynamic Drone Fail-Safe System

An inspection drone flies a high-voltage line at night along a mountain slope. Its satellite fix fails, its command
link goes behind the ridge, and a cell fails. A fail-safe has to bring it down somewhere it can stay: not on a slope
it will tumble down, not in the creek, not into the ravine, not near the crew working on the line.

Fail-safes usually fail in one particular way. The system does exactly what it was designed to do, and that is the
wrong thing to do where it is. In July 2023, hundreds of light-show drones over Melbourne's Docklands broke off their
routine; per the operator they "did exactly what they should've done with any technical glitch", and landed in the
Yarra River. Safety engineering calls this an unsafe control action: one that becomes unsafe "in a particular context
and worst-case environment" (Leveson and Thomas, STPA Handbook, 2018).

Each part of this tutorial adds one layer of DeepCausality to the controller and flies the same drone through the same
night.

## The World

`src/` holds the world every part shares. The controller sees it only through the drone's telemetry and, from part 2
on, its thermal camera and LiDAR.

| Feature | Where | Why it matters |
|---------|-------|----------------|
| Creek | valley floor, below 12 m across | water |
| Grass slope | 30 deg, 12-120 m across | a drone on skids tips over above 15 deg |
| Terrace | flat, 24-36 m across, 420-470 m along | flat grass in the middle of the slope |
| Access road | flat, 120-135 m across | four of the crew work on it |
| Rock slope | 25 deg, above the road | steep |
| Tower pads | flat 12 m squares under the line, every 300 m | flat; two of the crew work on the second pad |
| Ravine | 560-580 m along, above the road | a drop |
| Night wind | 2 m/s down the slope | without a satellite fix the drone cannot hold position and drifts with it |

The drone flies the line 60 m across the slope, 40 m above the ground, at 8 m/s. The flight starts at 22:00. The
faults arrive on a fixed timeline: the fix degrades at 50 s and drops out at 55 s, the link drops at 65 s, and a cell
fails at 85 s.

The ground has a temperature that depends on the surface and the time of day. At night the creek, at 11 °C, is warmer
than the grass, at 4 °C; by day it is cooler. A person reads 30 °C and stands 1.7 m tall. The LiDAR finds a surface on
95 % of its beams over land and on 10 % over water.

## Part 1: Dynamic Causality

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_1
```

The controller is one causal process with three stages each second:

1. `sense` takes the telemetry into the process and logs it.
2. `detect` evaluates four fault detectors, each a causaloid, as a collection. The collection says whether anything
   is wrong; only then does each detector say what. The process state confirms a lost fix after 3 s and a lost link
   after 5 s, so a dropped packet does not trigger a fail-safe.
3. `decide` applies the standard fail-safe ladder: a critical battery, or a lost fix with a lost link, lands the drone
   where it is; a lost fix alone holds it for the operator; a lost link alone flies it home. Once an emergency begins,
   the fail-safe never steps back down in flight.

The process has no context: the controller knows only what the drone reports about itself.

```text
 time   across   along   agl  sats  hdop  loss   cell  confirmed faults                       fail-safe
   0s      60m      0m   40m    14   0.8    2%  4.05V  none                                   continue
  50s      60m    400m   40m     7   3.5    2%  3.95V  fix degraded                           continue
  57s      56m    440m   40m     3  99.0    2%  3.94V  fix lost                               hold
  69s      32m    440m   40m     3  99.0   95%  3.91V  fix lost, link lost                    land now
  85s       0m    440m   16m     3  99.0   95%  3.25V  fix lost, link lost, battery critical  land now

Touchdown at 91 s, 440 m along the line, 71 m from the nearest person.
It dropped into the creek and was lost.

Faults and fail-safes from the controller's log:
  t=50 s: Satellite fix degraded: 7 satellites, HDOP 3.5.
  t=57 s: Satellite fix lost: none for 3 s.
  t=57 s: Fail-safe set to hold: no satellite fix, but the link is up, so wait for the fix or the operator.
  t=69 s: Command link lost: 95 % of packets dropped for 5 s.
  t=69 s: Fail-safe set to land now: with no satellite fix and no link, the drone can neither navigate nor receive orders.
  t=85 s: Battery critical: one cell at 3.25 V.
```

The controller confirmed every fault in order and took the textbook fail-safe each time. While it held and then
descended, the night wind carried it 72 m down the slope and into the creek. It never knew what lay below it: it
lacks context.

## Part 2: Dynamic Context

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_2
```

A thermal camera and a LiDAR look straight down. Each second they read every 4 m patch of ground in their footprint,
which shrinks as the drone descends: its mean temperature, its hottest spot, its slope, the fraction of LiDAR beams
that return, and how far anything sticks up from it. Each reading is noisy, and noisier the higher the drone flies.

The controller fuses these readings into a context, a `deep_causality_context::Context`:

- Each patch is a space node at its centre, linked to five data nodes, one per quantity. A data node holds an
  `Uncertain` value: the inverse-variance fusion of every frame that has seen the patch, so it sharpens with each look.
- A clock node holds the local time of day.

The causal process gains two stages between `sense` and `detect`:

1. `perceive` sets the clock and fuses the frame into the context.
2. `judge` reads each patch back from the context and judges it:
   - **too steep**: the slope exceeds 15 deg;
   - **water**: the LiDAR finds no surface, and the patch is warmer than the land at night or cooler by day; the clock
     node says which;
   - **a person**: a hot spot 15 °C above the land, or a bump over 1 m where the LiDAR finds a surface;
   - **safe**: none of these, and the LiDAR finds a surface.

   Each test is a comparison on uncertain values, decided by a sequential probability test at 95 % confidence. A patch
   neither safe nor a hazard at that confidence is unsure.

The ladder in `decide` is the one from part 1.

```text
 time   across   along   agl  sats  hdop  loss   cell  confirmed faults                       below        fail-safe
   0s      60m      0m   40m    14   0.8    2%  4.05V  none                                   too steep    continue
  18s      60m    144m   40m    14   0.8    2%  4.01V  none                                   safe         continue
  20s      60m    160m   40m    14   0.8    2%  4.01V  none                                   too steep    continue
  50s      60m    400m   40m     7   3.5    2%  3.95V  fix degraded                           too steep    continue
  57s      56m    440m   40m     3  99.0    2%  3.94V  fix lost                               too steep    hold
  68s      34m    440m   40m     3  99.0   95%  3.91V  fix lost                               safe         hold
  69s      32m    440m   40m     3  99.0   95%  3.91V  fix lost, link lost                    safe         land now
  74s      22m    440m   32m     3  99.0   95%  3.90V  fix lost, link lost                    too steep    land now
  80s      10m    440m   24m     3  99.0   95%  3.89V  fix lost, link lost                    water        land now
  85s       0m    440m   16m     3  99.0   95%  3.25V  fix lost, link lost, battery critical  water        land now

Touchdown at 91 s, 440 m along the line, 71 m from the nearest person.
It dropped into the creek and was lost.

The ground as the controller judged it, one 4 m patch per cell, downhill to the left:

              ~ ~ / / / . . . / / / / / / / / /
          ~ ~ ~ ~ / / / . . . / / / / / / / / /
        ~ ~ ~ ~ ~ / / / . . . / / / / / . P / /
      ~ ~ ~ ~ ~ ~ / / / . . . / / / / / P P / /
      ~ ~ ~ ~ ~ ~ / / / . . . / / / / / . . / /
      X ~ ~ ~ ~ ~ / / / . . L / / / / / / / / /
      ~ ~ ~ ~ ~ ~ / / / . . . / / / / / / / / /
      ~ ~ ~ ~ ~ ~ / / / . . . / / / / / / / / /
      ~ ~ ~ ~ ~ ~ / / / . . . / / / / / / / / /
        ~ ~ ~ ~ ~ / / / . . . / / / / / / / / /
          ~ ~ ~ ~ / / / . . . / / / / / / / / /

  . safe   / too steep   ~ water   P person   ? unsure   (blank) not seen
  L where the fail-safe chose to land now   X where the drone came down

At night the creek is warmer than the land. Read with the daytime rule, where water is the
cooler surface, 0 of the 58 patches judged to be water would still read as water.

When the fail-safe chose to land, at 69 s, the ground below was safe. The descent took 22 s,
and the night wind carried the drone 44 m downhill before it touched down.
The controller knew the ground, and nothing in it acted on what it knew. Part 3 adds action.
```

The controller now knows the ground. It found the first tower pad at 18 s, the crew on the second pad, the flat
terrace below it when it chose to land, and the creek it drifted into. The map shows the terrace as a strip of safe
patches between the steep grass, and the drone's track from the terrace to the creek.

The time of day matters. The same context, read with the daytime rule, finds no water in the creek: at night the
creek is warmer than the land, not cooler. Its patches would read as unsure.

The fail-safe still lands the drone in the creek. Knowing the ground does not move the drone: it lacks action.

The simulation registers every frame in the world frame, so a patch keeps its place while the drone drifts without a
fix. A real system registers frames against each other with visual or LiDAR odometry.

## Precision

`FloatType` in `src/lib.rs` sets the working precision of the whole tutorial. Part 1 produces the same output at
`f32`, `f64` and `deep_causality_num::Float106`. Part 2 produces the same output at `f32` and `f64`. At `Float106` one
borderline patch on the edge of the first tower pad reads unsure for one second, at 19 s; every other line is the
same.

## Where Things Live

| Path | Contents |
|------|----------|
| [`src/types/terrain/`](src/types/terrain) | `Terrain`: elevation, slope, surface, ground temperature, the crew |
| [`src/types/drone/`](src/types/drone) | `Drone`: its mission, fault timeline, `Telemetry`, sensor frames and response to a `Command` |
| [`src/types/patch_reading/`](src/types/patch_reading) | `PatchReading`: one frame's reading of one ground patch, by `Quantity` |
| [`src/types/touchdown/`](src/types/touchdown) | `Touchdown`: the ground truth of where the drone came down, and its `Outcome` |
| [`src/constants.rs`](src/constants.rs) | The world's dimensions and temperatures, the drone, its sensors, the fault timeline |
| [`part_1_dynamic_causality/`](part_1_dynamic_causality) | The fail-safe controller with dynamic causality alone |
| [`part_2_dynamic_context/`](part_2_dynamic_context) | The controller with the ground as its context |

## References

- ABC News, [Hundreds of drones plunge into Melbourne's Yarra River](https://www.abc.net.au/news/2023-07-16/hundreds-of-drones-plunge-into-yarra-river/102607576), 16 July 2023.
- N. G. Leveson and J. P. Thomas, [STPA Handbook](https://www.flighttestsafety.org/images/STPA_Handbook.pdf), MIT, 2018.
