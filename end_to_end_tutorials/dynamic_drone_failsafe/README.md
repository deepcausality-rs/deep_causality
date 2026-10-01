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

## Part 3: Dynamic Action

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_3
```

A causal state machine, the fail-safe machine, now turns the ladder's command into a maneuver over the ground. The
drone flies these maneuvers by its camera and LiDAR, so it holds its place against the wind without a satellite fix.
The machine has four states, each a causaloid that reads the situation (the ladder's command, the drone's position,
and the controller's judgement of the landing patch) and returns a maneuver:

| State | Active when | Maneuver |
|-------|-------------|----------|
| 1 choose a landing patch | the ladder says land now, and no patch is chosen or the chosen one no longer reads as safe | the controller picks the safe patch nearest the drone |
| 2 hold over the ground | the ladder says hold | hover where the drone is |
| 3 fly home | the ladder says return home | fly to the launch point |
| 4 land on the chosen patch | the ladder says land now, and the chosen patch reads as safe | fly to the patch, then descend onto it |

The landing state's causaloid holds the chosen patch as its context. Each time the controller chooses a patch, it
replaces the landing state with a new version through `update_single_state`. Each second the machine checks the
chosen patch against the context again; if the patch stops reading as safe, state 1 fires and the controller chooses
again.

A new stage, `act`, runs the machine after `decide`.

```text
 time   across   along   agl  confirmed faults                       below      fail-safe  maneuver
   0s      60m      0m   40m  none                                   too steep  continue   none
  18s      60m    144m   40m  none                                   safe       continue   none
  20s      60m    160m   40m  none                                   too steep  continue   none
  50s      60m    400m   40m  fix degraded                           too steep  continue   none
  57s      56m    440m   40m  fix lost                               too steep  hold       hold over the ground
  69s      56m    440m   40m  fix lost, link lost                    too steep  land now   land on the chosen patch
  71s      58m    446m   40m  fix lost, link lost                    safe       land now   land on the chosen patch
  85s      58m    446m   19m  fix lost, link lost, battery critical  safe       land now   land on the chosen patch

Touchdown at 92 s, 446 m along the line, 3 m from the nearest person.
It landed 3 m from a member of the crew.

The ground as the controller judged it, one 4 m patch per cell, downhill to the left:

              . / / / / / / / / / / / /
              . / / / / / / / / / / / /
              . / / / / / / / / / / / /
              . / / / / / . P / / / / /
              . / / / / / P P / / / / /
              . / / / / / X . / / / / /
              . / / / / / / / / / / / /
              . / / / / / / / / / / / /
              . / / / / / / / / / / / /
              . / / / / / / / / / / / /
              . / / / / / / / / / / / /

  . safe   / too steep   ~ water   P person   ? unsure   (blank) not seen
  X where the drone came down

The fail-safe machine chose the safe patch nearest the drone and landed on it. The patch
passed every test the controller ran: flat, dry, no one on it.
The controller's own map showed a person on a patch 4 m away from it.
No test asked how close to a person the drone may land. Part 4 adds that check.

Faults, fail-safes and maneuvers from the controller's log:
  t=50 s: Satellite fix degraded: 7 satellites, HDOP 3.5.
  t=57 s: Satellite fix lost: none for 3 s.
  t=57 s: Fail-safe set to hold: no satellite fix, but the link is up, so wait for the fix or the operator.
  t=57 s: Fail-safe machine runs "hold over the ground", state 2, version 1.
  t=69 s: Command link lost: 95 % of packets dropped for 5 s.
  t=69 s: Fail-safe set to land now: with no satellite fix and no link, the drone can neither navigate nor receive orders.
  t=69 s: Landing patch chosen 6 m away, at 58 m across and 446 m along. Landing state replaced by version 2.
  t=69 s: Fail-safe machine runs "land on the chosen patch", state 4, version 2.
  t=85 s: Battery critical: one cell at 3.25 V.
```

The drone no longer drifts. It holds over the ground at 57 s, and at 69 s the machine chooses the safe patch nearest
the drone: a corner of the second tower pad, 6 m away. The patch read as safe at every recheck, and the drone landed
on it, 3 m from a member of the crew. The controller knew the crew was there: its map shows them on the next patch.
Every test it ran asked about the patch itself. None asked how close to a person the drone may land.

In Orlando in December 2024, show drones fell into the crowd and a 7-year-old needed open-heart surgery (NTSB
preliminary report, January 2025). Losing a drone is acceptable; harming a person is not. Nothing in the flight controller
software checks the machine's proposal against that rule: it lacks a check against safety rules before activating its fail-safe.

## Part 4: Effect Ethos

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_4
```

The fail-safe machine now flies only what the Effect Ethos approves. Each landing it proposes becomes a
`ProposedAction` naming a patch, and the Ethos judges it against norms. The norms read the same context the
controller fills, which now also holds the drone's own state: where it is, its height, its position error, its
battery endurance and the land temperature.

| Norm | Forbids a proposal when | Priority |
|------|-------------------------|----------|
| person | a person is judged present, at 95 %, within the clearance of the patch | 4 |
| not ruled out | a person is not ruled out, at 99 %, everywhere within the clearance; unseen ground rules out no one | 4 |
| battery | the battery cannot get the drone to the patch and down with 5 s to spare | 3 |
| drone | the drone would not survive the touchdown: steep ground, no surface, or unseen | 2 |
| sacrifice | never; it permits ditching the drone and defeats the drone norm | 3 |
| landing | never; it permits every proposal no other norm forbids | 1 |

The clearance is 10 m, widened by the drone's position error, which grows 0.1 m for each second without a
satellite fix, and by 2 m for a gust. Losing the drone is acceptable; harming a person is not. The sacrifice norm
therefore overrides the drone norm, and nothing overrides the two person norms. The judge names a person at 95 %
confidence; the Ethos needs 99 % that there is none.

The machine proposes landings on the safe patches nearest the drone, then ditchings on the nearest water, until the
Ethos approves one. A proposal forbidden only because a person is not yet ruled out may stand while the drone flies
over the patch and looks; the machine gains a look state for that. Each second the Ethos reviews the plan in force
again. When nothing is approved, the drone holds over the ground instead of descending where it is.

```text
 time   across   along   agl  confirmed faults                       below      fail-safe  maneuver
   0s      60m      0m   40m  none                                   too steep  continue   none
  18s      60m    144m   40m  none                                   safe       continue   none
  20s      60m    160m   40m  none                                   too steep  continue   none
  50s      60m    400m   40m  fix degraded                           too steep  continue   none
  57s      56m    440m   40m  fix lost                               too steep  hold       hold over the ground
  69s      56m    440m   40m  fix lost, link lost                    too steep  land now   fly over the patch and look
  74s      41m    439m   40m  fix lost, link lost                    too steep  land now   land on the approved patch
  76s      35m    438m   40m  fix lost, link lost                    safe       land now   land on the approved patch
  85s      34m    438m   26m  fix lost, link lost, battery critical  safe       land now   land on the approved patch

Touchdown at 94 s, 438 m along the line, 26 m from the nearest person.
It landed upright and can be recovered.

The ground as the controller judged it, one 4 m patch per cell, downhill to the left:

              ~ / / / . . . / / / / / / / / / / / /
              ~ / / / . . . / / / / / . P / / / / /
              ~ / / / . . . / / / / / P P / / / / /
              ~ / / / . . . / / / / / . . / / / / /
              ~ / / / . . . / / / / / / / / / / / /
              ~ / / / . . X / / / / / / / / / / / /
              ~ / / / . . . / / / / / / / / / / / /
              ? / / / . . . / / / / / / / / / / / /
              ~ / / / . . . / / / / / / / / / / / /
              ~ / / / . . . / / / / / / / / / / / /
              ~ / / / / / / / / / / / / / / / / / /

  . safe   / too steep   ~ water   P person   ? unsure   (blank) not seen
  X where the drone came down

The Effect Ethos rejected proposals for these reasons:
    3  a person is within the clearance
    3  a person is not yet ruled out within the clearance
    0  the battery cannot get the drone there and down
    0  the drone would not survive the touchdown
It approved the patch at 34 m across and 438 m along once the drone had seen the ground around it.
The controller's own map put the nearest person 27 m from that patch.

The same Effect Ethos on three other proposals, against the context at touchdown, with the battery nearly spent:
  Land next to the crew, at 58 m across and 446 m along: rejected: a person is within 16 m of it; a person is not yet ruled out within 16 m of it; the battery cannot get the drone there and down with 5 s to spare.
  Land on the steep grass, at 38 m across and 438 m along: rejected: the drone would not survive the touchdown.
  Ditch the drone in the creek, at 10 m across and 438 m along: rejected: a person is not yet ruled out within 16 m of it; the battery cannot get the drone there and down with 5 s to spare.
    The drone would be lost there, and the sacrifice norm overrides the drone norm. Nothing overrides the person norms.

Faults, fail-safes and maneuvers from the controller's log:
  t=50 s: Satellite fix degraded: 7 satellites, HDOP 3.5.
  t=57 s: Satellite fix lost: none for 3 s.
  t=57 s: Fail-safe set to hold: no satellite fix, but the link is up, so wait for the fix or the operator.
  t=57 s: Fail-safe machine runs "hold over the ground", state 2, version 1.
  t=69 s: Command link lost: 95 % of packets dropped for 5 s.
  t=69 s: Fail-safe set to land now: with no satellite fix and no link, the drone can neither navigate nor receive orders.
  t=69 s: Proposal 1: land on the patch 6 m away, at 58 m across and 446 m along. Rejected: a person is within 13 m of it; a person is not yet ruled out within 13 m of it.
  t=69 s: Proposal 2: land on the patch 8 m away, at 62 m across and 446 m along. Rejected: a person is within 13 m of it; a person is not yet ruled out within 13 m of it.
  t=69 s: Proposal 3: land on the patch 14 m away, at 58 m across and 454 m along. Rejected: a person is within 13 m of it; a person is not yet ruled out within 13 m of it.
  t=69 s: Proposal 4: land on the patch 22 m away, at 34 m across and 438 m along. Not yet: a person is not yet ruled out within 13 m of it, so the drone flies over it to look.
  t=69 s: Land and look states replaced by version 2.
  t=69 s: Fail-safe machine runs "fly over the patch and look", state 5, version 2.
  t=74 s: The Effect Ethos approves the patch at 34 m across and 438 m along: a person is now ruled out within 14 m of it, and no other norm forbids it.
  t=74 s: Fail-safe machine runs "land on the approved patch", state 4, version 2.
  t=85 s: Battery critical: one cell at 3.25 V.
```

At 69 s the machine proposes the pad corner it landed on in part 3. The Ethos rejects it and the two pad patches after
it: the crew stands within the clearance. It lets the terrace stand pending a look, because the camera has not yet
seen enough of the ground around it to rule a person out. The drone flies over the terrace, and at 74 s the Ethos
approves it. When the cell fails at 85 s, the Ethos reviews the plan again; the drone, 26 m up, can still descend
with time to spare. It lands upright, 26 m from the nearest person.

The rulings at the end show the priorities at work. Ditching the drone in the creek would lose it, and the sacrifice
norm overrides the drone norm that forbids that; the Ethos still rejects the ditching, because it cannot rule out a
person near the creek and the battery cannot get there.

## Precision

`FloatType` in `src/lib.rs` sets the working precision of the whole tutorial: `f32`. Every part produces the same
output at `f64`. At `deep_causality_num::Float106`, part 1 is the same; in parts 2 to 4 one borderline patch on the
edge of the first tower pad reads unsure for one second, at 19 s, and in part 4 the Effect Ethos approves the landing
patch one second later, at 75 s. The drone lands at the same time and place.

`deep_causality_num::BFloat16` runs too, but resolves positions only to 2 m at 440 m along the line, which moves a
member of the crew into the next patch.

## Where Things Live

| Path | Contents |
|------|----------|
| [`src/types/terrain/`](src/types/terrain) | `Terrain`: elevation, slope, surface, ground temperature, the crew |
| [`src/types/drone/`](src/types/drone) | `Drone`: its mission, fault timeline, `Telemetry`, sensor frames, and response to a `Command` or `Guidance` |
| [`src/types/guidance.rs`](src/types/guidance.rs) | `Guidance`: hover over or land on a ground point, flown by the camera and LiDAR |
| [`src/types/patch_reading/`](src/types/patch_reading) | `PatchReading`: one frame's reading of one ground patch, by `Quantity` |
| [`src/types/touchdown/`](src/types/touchdown) | `Touchdown`: the ground truth of where the drone came down, and its `Outcome` |
| [`src/constants.rs`](src/constants.rs) | The world's dimensions and temperatures, the drone, its sensors, the fault timeline |
| [`part_1_dynamic_causality/`](part_1_dynamic_causality) | The fail-safe controller with dynamic causality alone |
| [`part_2_dynamic_context/`](part_2_dynamic_context) | The controller with the ground as its context |
| [`part_3_dynamic_action/`](part_3_dynamic_action) | The controller with a fail-safe machine that acts on the context |
| [`part_4_effect_ethos/`](part_4_effect_ethos) | The controller whose fail-safe machine flies only what the Effect Ethos approves |

## References

- ABC News, [Hundreds of drones plunge into Melbourne's Yarra River](https://www.abc.net.au/news/2023-07-16/hundreds-of-drones-plunge-into-yarra-river/102607576), 16 July 2023.
- N. G. Leveson and J. P. Thomas, [STPA Handbook](https://www.flighttestsafety.org/images/STPA_Handbook.pdf), MIT, 2018.
