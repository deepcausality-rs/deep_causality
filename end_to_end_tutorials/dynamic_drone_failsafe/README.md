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
| Trees | 64-104 m across, 520-600 m along, canopy 18 m | a drone below the canopy hits them; a person under them is hidden from both sensors |
| Access road | flat, 120-135 m across | four of the crew work on it |
| Rock slope | 25 deg, above the road | steep |
| Tower pads | flat 12 m squares under the line, every 300 m | flat; two of the crew work on the second pad |
| Ravine | 560-580 m along, above the road | a drop |
| Night wind | 2 m/s down the slope | without a satellite fix the drone cannot hold position and drifts with it |

The drone flies the line 60 m across the slope, 40 m above the ground, at 8 m/s. The flight starts at 22:00. The
faults arrive on a fixed timeline: the fix degrades at 50 s and drops out at 55 s, the link drops at 65 s, and a cell
fails at 85 s. The battery dies 25 s after a cell fails, or when the lowest cell runs down to 3.0 V.

The ground has a temperature that depends on the surface and the time of day. At night the creek, at 11 °C, is warmer
than the grass, at 4 °C; by day it is cooler. A person reads 30 °C and stands 1.7 m tall. The LiDAR finds a surface on
95 % of its beams over land and on 10 % over water.

A `Mission` sets the line, the launch hour, the wind, a gust at touchdown, the `FaultTimeline` and the sensor seed; a
`Terrain` sets the line, the terrace, the trees and the crew. Parts 1 to 4 fly the defaults above. Part 5 draws both
at random. A lost fix or link may come back, and a cell need not fail.

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

The ladder follows the fail-safe defaults that drone autopilots document today:

| Fault | Part 1 | ArduPilot Copter | DJI Matrice 4D (manual v1.2, 2025) |
|-------|--------|------------------|------------------------------------|
| Critical battery | land where it is | battery fail-safe: Land, the recommended critical action | lands automatically once the battery lasts only the descent |
| Fix and link lost | land where it is | radio fail-safe: return to launch, or Land without GPS | fail-safe return home; with positioning abnormal, ATTI mode and an automatic landing |
| Fix lost alone | hold, drifting with the wind | EKF fail-safe: Land by default, altitude hold as an option | ATTI mode; the pilot takes control |
| Link lost alone | fly home | radio fail-safe: return to launch by default | fail-safe return home |

Each rule acts on one fault. Newer features extend them: DJI hovers and lands by its vision sensors when GNSS is weak,
and flies to a pre-set alternate landing site when its dock cannot take it; ArduPilot dead-reckons for a limited time
without GPS. None of these documents describes weighing who stands near a landing site.

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

In Orlando in December 2024, show drones entered the audience, and one struck a seven-year-old boy in the face and
chest (NTSB final report, September 2026); his parents say he needed open-heart surgery (FOX 35, September 2026). Losing a drone is acceptable; harming a person is not. Nothing in the flight controller
software checks the machine's proposal against that rule: it lacks a check against safety rules before activating its fail-safe.

## Part 4: Effect Ethos

```bash
cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_4
```

### The safety protocol

Drone pilots do not have to invent a response to a lost fix or a lost link. Aviation authorities and autopilot makers
publish one: the FAA's guidance for Part 107 pilots, CASA's operations-manual requirements, the JARUS SORA risk
method that EASA adopts, and the fail-safes of the ArduPilot autopilot. Part 4 encodes that protocol as norms of the
Effect Ethos. Part 5 then tests how well the encoded protocol works across a thousand randomised scenarios.

The published procedures assume a pilot and a briefed site; the drone here has neither. It has no planned landing
zone and no launch site it may assume safe, only the ground it has sensed. The table gives each step, how Part 4
carries it out, and its source.

| Step | Procedure | In part 4 | Source |
|------|-----------|-----------|--------|
| 0. Before flight | A navigation fault triggers altitude hold, not a landing; recovery time limits are set in advance. | A lost fix or link never triggers a descent by itself; the recovery window is 20 s. | ArduPilot GPS fail-safe (land or altitude hold); CASA AC 101-01 |
| 1. Stabilise | Stop the descent and hold. | The drone holds over the ground by camera and LiDAR. | ArduPilot; attitude-mode practice |
| 2. Assess | Battery and control healthy, positioning or link lost: a contingency, recoverable. Energy or control failing: an emergency. | The `assess` stage sets the urgency: routine, contingency, emergency. | JARUS SORA: contingency and emergency procedures |
| 3. Wait within limits | Hold and wait for the fix or link to recover, for the time set in advance. | During the window the drone waits, chooses a site and flies over it to look. If the fix and link return, the mission resumes. | FAA lost-link guidance; CASA |
| 4. Move to known ground | Return toward a planned landing zone, clear of people. | The drone has no planned zone. It moves to the best ground it has sensed and ruled clear of people, and explores unseen ground when there is none. | SORA flight geography; pilot practice |
| 5. Land as soon as practical | Land at a suitable site, not the nearest spot; avoid water, trees and power lines. | Each round the Ethos judges every candidate, and the drone lands on the cheapest permitted one, sooner first among equal costs. | FAA: "land as soon as practical"; pilot practice |
| 6. Emergency, only if unrecoverable | Terminate the flight in the ground risk buffer, free of people. People first, property second, the aircraft last. | Once the battery is critical, the drone may be ditched. If nothing is permitted, the person bans become costs, so the drone comes down where it endangers the fewest, farthest from people. | SORA ground risk buffer; Pilot Institute |

### The norms

The norms read the same context the controller fills. Besides the ground, it holds the drone's own state: where it
is, its height, how accurately it lands, its endurance, its descent rate and the land temperature.

| Norm | Applies | Forbids a proposal, or prices it, when | Priority |
|------|---------|----------------------------------------|----------|
| person | always | a person is more likely than not within the clearance | 4 |
| not ruled out | always | a person is not ruled out, at 99 %, everywhere within the clearance; unseen ground and ground under canopy rule out no one | 4 |
| path | always | the flight there at the drone's height would cross the canopy or unseen ground | 3 |
| battery | always | the battery cannot get the drone there and down with 5 s to spare | 3 |
| drone | always | the drone would not survive the touchdown: steep, no surface, trees or unseen | 2 |
| edge | always | a gust could put the drone on seen steep ground beside the patch | 2 |
| sacrifice | ditching | never; it prices the drone's loss at 10³ and defeats the drone, edge and battery norms | 4 |
| emergency | emergency | never; it defeats "not ruled out" | 5 |
| near the unknown | emergency, last resort | a person is not ruled out within the clearance: 10⁴ | 1 |
| last resort | last resort | never; it defeats both person norms | 6 |
| within 20, 10, 5 m | last resort | a person more likely than not lies that close: 10⁵, 10⁶, 10⁷ | 1 |

The clearance is 10 m, plus 1 m for landing accuracy and 2 m for a gust. The harm costs follow the protocol's order:
each step is ten times the one before. Losing the drone costs least; a person not ruled out near the patch costs
more; a likely person within 20, 10 or 5 m costs most. Only the last-resort norm can lift the person bans, and it
puts these prices in their place. So a tree always beats a person, and a farther person beats a nearer one.

A failed cell shows as a one-second sag in the lowest cell's voltage. It leaves 20 s of flight, and the drone then
descends at the emergency rate. Without one, the battery lasts as long as the lowest cell takes to drain to empty. The
emergency begins with 60 s or less left; a low battery alone is a contingency.

```text
 time   across   along   agl  confirmed faults                       below      urgency      maneuver
   0s      60m      0m   40m  none                                   too steep  routine      fly the mission
  18s      60m    144m   40m  none                                   safe       routine      fly the mission
  20s      60m    160m   40m  none                                   too steep  routine      fly the mission
  50s      60m    400m   40m  fix degraded                           too steep  routine      fly the mission
  57s      56m    440m   40m  fix lost                               too steep  contingency  fly over the patch and look
  58s      53m    440m   40m  fix lost                               too steep  contingency  fly over the patch and look
  64s      36m    435m   40m  fix lost                               safe       contingency  fly over the patch and look
  69s      30m    430m   36m  fix lost, link lost                    safe       contingency  fly over the patch and look
  77s      30m    430m   25m  fix lost, link lost                    safe       contingency  land on the approved patch
  85s      30m    430m   13m  fix lost, link lost, battery critical  safe       emergency    land on the approved patch

Touchdown at 90 s, 430 m along the line, 34 m from the nearest person.
It landed upright and can be recovered.


              ~ ~ / / / . . . / / / / / P P / / / /
              ~ ~ / / / . . . / / / / / . . / / / /
              ~ ~ / / / . . . / / / / / / / / / / /
              ~ ~ / / / . . . / / / / / / / / / / /
              ~ ~ / / / . . . / / / / / / / / / / /
              ~ ~ / / / . X . / / / / / / / / / / /
              ~ ~ / / / . . . / / / / / / / / / / /
              ~ ~ / / / . . . / / / / / / / / / / /
              ~ ~ / / / / / / / / / / / / / / / / /
              ~ ~ / / / / / / / / / / / / / / / / /
              ~ ~ / / / / / / / / / / / / / / / / /

  . safe   / too steep   ~ water   T trees   P person   ? unsure   (blank) not seen
  X where the drone came down

The last-resort norms on three ditchings, against the context at 85 s, when the emergency began.
Under them a person near a patch no longer forbids it but prices it, so the cheapest endangers the fewest:
  Next to the crew, at 58 m across and 446 m along: permitted at harm cost 11111000.
  On the steep grass, at 22 m across and 430 m along: permitted at harm cost 1000.
  In the creek, at 10 m across and 430 m along: permitted at harm cost 11000.
```

When the fix drops at 57 s the drone does not descend. It treats the loss as a contingency and, while it waits, puts
the candidates to the Effect Ethos. The Ethos rejects the pad beside the crew, and the drone flies over the terrace to
look. The first two terrace patches it tries lie on the terrace's edge, where a gust could put it on the steep step.
At 64 s the Ethos permits a patch in the terrace's middle. The drone descends to 25 m over it and waits out the window,
which ends at 77 s with no recovery, so it lands as soon as practicable. When the cell fails at 85 s the drone is 13 m
up over the permitted patch. The emergency round confirms that patch, and the drone lands upright at 90 s, 34 m from
the nearest person.

The closing rulings show the last resort's prices at the moment of the emergency. Ditching next to the crew costs
over eleven million. Ditching in the creek costs eleven thousand, because unseen ground lies within its clearance and
a person there is not ruled out. Ditching on empty steep grass costs a thousand.

## Part 5: Verification

```bash
cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_5
cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_5 -- scenario 810
```

A regulator asks whether a safety measure works across the conditions an operation can meet. Part
5 flies the controllers of parts 1, 3 and 4, unchanged, through the same 1000 scenarios. Each controller's stages are
wired the way that part's own `main` wires them. Each scenario draws, uniformly:

- the launch time, any hour of the day or night;
- a wind of 0 to 6 m/s from any direction, and a gust at touchdown of up to 2 m;
- the line anywhere from 24 to 108 m across the slope;
- the fix lost 20 to 90 s after launch, and the link lost from 20 s before to 30 s after;
- in a third of scenarios the fix comes back 10 to 60 s later, and so, independently, does the link;
- in half of them a cell fails up to 60 s after the later loss;
- the terrace anywhere on the slope, and a stand of 18 m trees near where the fix drops out;
- up to 8 people within 40 m across and 60 m along of that point, some perhaps under the trees.

People stand around the point where the fix drops out, the worst place for a fail-safe that lands where it is. Every
scenario is a function of its number alone, so any one can be flown again with each controller's log. Each controller
flies each scenario in the same world, with the same sensor noise, so the three outcomes of a scenario answer what
would have happened there under each controller. The campaign writes every scenario and its outcomes to
`part_5_verification/campaign_1000.csv`; a campaign of another size writes its own file.

```text
How the flight ended                                 part 1         part 3         part 4
                                                   textbook    context and       with the
                                                  fail-safe         action   Effect Ethos
resumed its mission                              0    0.0 %     0    0.0 %    37    3.7 %
landed upright, clear of people                 51    5.1 %   691   69.1 %   599   59.9 %
came down within 10 m of a person               35    3.5 %    41    4.1 %     1    0.1 %
dropped into the creek                         234   23.4 %     3    0.3 %    72    7.2 %
fell into the ravine                             8    0.8 %     0    0.0 %     0    0.0 %
tipped over on steep ground                    631   63.1 %    79    7.9 %   272   27.2 %
flew into the trees                             41    4.1 %     1    0.1 %    19    1.9 %
fell when its battery died                       0    0.0 %   185   18.5 %     0    0.0 %

Rates, each with its one-sided 95 % upper confidence bound:
  all scenarios, 1000 scenarios:
    Part 1, textbook fail-safe     near a person   3.5 % (at most  4.61 %)   drone lost  91.4 % (at most 92.82 %)
    Part 3, context and action     near a person   4.1 % (at most  5.29 %)   drone lost  26.8 % (at most 29.20 %)
    Part 4, with the Effect Ethos  near a person   0.1 % (at most  0.47 %)   drone lost  36.3 % (at most 38.88 %)
  battery healthy, 495 scenarios:
    Part 1, textbook fail-safe     near a person   3.0 % (at most  4.63 %)   drone lost  91.9 % (at most 93.84 %)
    Part 3, context and action     near a person   3.8 % (at most  5.58 %)   drone lost   9.9 % (at most 12.39 %)
    Part 4, with the Effect Ethos  near a person   0.0 % (at most  0.60 %)   drone lost  10.9 % (at most 13.49 %)
  a cell fails, 505 scenarios:
    Part 1, textbook fail-safe     near a person   4.0 % (at most  5.70 %)   drone lost  90.9 % (at most 92.91 %)
    Part 3, context and action     near a person   4.4 % (at most  6.16 %)   drone lost  43.4 % (at most 47.11 %)
    Part 4, with the Effect Ethos  near a person   0.2 % (at most  0.94 %)   drone lost  61.2 % (at most 64.80 %)

Paired on the same scenarios, part 4 against each other controller, all 1000 scenarios:
  Part 1, textbook fail-safe: only it came down near a person in 35 scenarios, only part 4 in 1, both in 0.
    Part 4 came down near a person 35.0 times less often; at least 6.6 times less with 95 % confidence.
    Were the two equally safe, a split this uneven would arise with probability 5.4e-10.
  Part 3, context and action: only it came down near a person in 41 scenarios, only part 4 in 1, both in 0.
    Part 4 came down near a person 41.0 times less often; at least 7.8 times less with 95 % confidence.
    Were the two equally safe, a split this uneven would arise with probability 9.8e-12.
```

The upper bounds are one-sided Clopper-Pearson bounds at 95 %: zero events in 495 scenarios bounds the rate at 0.60 %.
The campaign takes 3 to 4 minutes on an M3 Max with 16 cores.

Every controller flies the same scenarios, so the campaign pairs them. In 35 scenarios only part 1 came down near a
person, in one only part 4 did, and in none both. Part 4 came down near a person 35 times less often than part 1, and
at least 6.6 times less with 95 % confidence; against part 3 the figures are 41 and 7.8. The bound takes the
matched-pairs variance of the ratio's logarithm. Were part 1 and part 4 equally safe, the exact sign test puts the
chance of a split of 35 to 1 at 5.4e-10.

With a healthy battery, the encoded protocol came down near no one in 495 scenarios, and lost about as many drones as
part 3, 10.9 % against 9.9 %. Part 3 came down within 10 m of a person in 3.8 % of the same scenarios. When a cell
fails, part 4 gives up more drones than part 3, 61.2 % against 43.4 %, and comes down near a person in 0.2 % of
scenarios against 4.4 %. With 15 s of battery left, a landing beside ground where a person is not ruled out costs more
than the drone, so the drone goes down on empty ground instead. That is the protocol's order: people first, the
aircraft last.

One scenario still ends near a person: 810. The cell fails 6 s after the controller confirms the lost link, while the
drone is still at 40 m. Every patch the battery can still reach lies beside ground where a person cannot be ruled out,
and the drone comes down 2 m from someone.

### How the campaign changed part 4

Part 4 reached this form through the campaign. Each run found a failure that the single night had not shown. The
failure was traced in a flown-again scenario and fixed in the norms or the machine, and the campaign was run again.

| Finding | Fix |
|---------|-----|
| With nothing permitted, the drone held until its battery died and fell where it hovered. All its near-person outcomes, 3.9 %, were such falls beside the crew: the protocol's veto was itself the unsafe control action. | In an emergency the drone may be ditched; a ditching may fly away from people even if the battery dies first. |
| A patch the drone was still looking at was never cleared, because the clearance grew with the drone's absolute position drift. | The clearance grows with landing accuracy instead: the drone sees the patch and the people near it in the same frames. A look has a time limit. |
| The controller read a battery running down normally as a failed cell, and gave up approved landings. | A failed cell shows as a sudden sag; otherwise the battery lasts until the lowest cell is empty. |
| A patch judged flat at its centre lay on a terrace edge, and the gust tipped the drone. | The edge norm forbids a patch beside seen steep ground. |
| With all seen ground set aside, the drone hovered with a healthy battery until the battery ran down. | The drone explores unseen ground. |
| In an emergency the drone ditched beside people it had not ruled out. | A person not ruled out costs ten times the drone, and a person more likely than not forbids the patch. |

## Precision

`FloatType` in `src/lib.rs` sets the working precision of the whole tutorial: `f32`. Parts 1 to 4 produce the same
output at `f64`. At `deep_causality_num::Float106`, part 1 is the same. In parts 2 to 4 one borderline patch on the
edge of the first tower pad reads unsure for one second, at 19 s. In part 4 the drone also lands on a neighbouring
patch of the same terrace, at the same time, 30 m from the nearest person instead of 34 m. Part 5 was run at `f32`.

`deep_causality_num::BFloat16` runs too, but resolves positions only to 2 m at 440 m along the line, which moves a
member of the crew into the next patch.

## Where Things Live

| Path | Contents |
|------|----------|
| [`src/types/terrain/`](src/types/terrain) | `Terrain`: elevation, slope, surface, canopy, ground temperature, the crew |
| [`src/types/drone/`](src/types/drone) | `Drone`: its mission, fault timeline, `Telemetry`, sensor frames, and response to a `Command` or `Guidance` |
| [`src/types/guidance.rs`](src/types/guidance.rs) | `Guidance`: hover over, descend over or land on a ground point, flown by the camera and LiDAR |
| [`src/types/mission/`](src/types/mission) | `Mission`: the line, the launch hour, the wind and gust, the faults and the sensor seed |
| [`src/types/fault_timeline/`](src/types/fault_timeline) | `FaultTimeline`: when the fix and the link drop and come back, and when a cell fails |
| [`src/types/patch_reading/`](src/types/patch_reading) | `PatchReading`: one frame's reading of one ground patch, by `Quantity` |
| [`src/types/touchdown/`](src/types/touchdown) | `Touchdown`: the ground truth of where the drone came down, and its `Outcome` |
| [`src/constants.rs`](src/constants.rs) | The world's dimensions and temperatures, the drone, its sensors, the fault timeline |
| [`src/trace.rs`](src/trace.rs) | The CSV tables parts 1 to 4 record when run with `-- trace <dir>`, for the animation in [`video/drone_failsafe`](../../video/drone_failsafe) |
| [`part_1_dynamic_causality/`](part_1_dynamic_causality) | The fail-safe controller with dynamic causality alone |
| [`part_2_dynamic_context/`](part_2_dynamic_context) | The controller with the ground as its context |
| [`part_3_dynamic_action/`](part_3_dynamic_action) | The controller with a fail-safe machine that acts on the context |
| [`part_4_effect_ethos/`](part_4_effect_ethos) | The controller under the safety protocol, encoded as norms of the Effect Ethos |
| [`part_5_verification/`](part_5_verification) | The campaign: parts 1, 3 and 4 over 1000 randomised scenarios, and its record |
| [`papers/`](papers) | Source documents the tutorial cites: the FAA's Small UAS Survey Report 2024 |

## References

- ABC News, [Hundreds of drones plunge into Melbourne's Yarra River](https://www.abc.net.au/news/2023-07-16/hundreds-of-drones-plunge-into-yarra-river/102607576), 16 July 2023.
- N. G. Leveson and J. P. Thomas, [STPA Handbook](https://www.flighttestsafety.org/images/STPA_Handbook.pdf), MIT, 2018.
- NTSB, [Aviation investigation final report DCA25LA065](https://data.ntsb.gov/carol-repgen/api/Aviation/ReportMain/GenerateNewestReport/199458/pdf), Orlando drone show, September 2026.
- FOX 35 Orlando, [Orlando drone show errors led to Lake Eola crash that seriously injured 7-year-old: NTSB](https://www.fox35orlando.com/news/orlando-drone-show-errors-led-lake-eola-crash-seriously-injured-7-year-old-ntsb), 16 September 2026.
- FAA, [Advisory Circular 107-2A, Small Unmanned Aircraft Systems](https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_107-2A.pdf).
- Flightpath107, [Part 107 emergency procedures](https://www.flightpath107.com/curriculum-home/section-three/emergency-procedures/): "If GPS signal cannot be reacquired, land as soon as practical."
- CASA, [AC 101-01, Remotely piloted aircraft systems: licensing and operations](https://www.casa.gov.au/remotely-piloted-aircraft-systems-licensing-and-operations).
- JARUS, [SORA v2.5, main body](http://jarus-rpas.org/wp-content/uploads/2024/06/SORA-v2.5-Main-Body-Release-JAR_doc_25.pdf) and [Annex A](http://jarus-rpas.org/wp-content/uploads/2024/06/SORA-v2.5-Annex-A-Release.JAR_doc_26-pdf.pdf): flight geography, contingency volume, ground risk buffer, contingency and emergency procedures.
- ArduPilot, [Radio Failsafe](https://ardupilot.org/copter/docs/radio-failsafe.html), [Battery Failsafe](https://ardupilot.org/copter/docs/failsafe-battery.html) and [EKF Failsafe](https://ardupilot.org/copter/docs/common-ekf-inav-failsafe.html).
- DJI, [DJI Matrice 4D Series Unmanned Aircraft Flight Manual v1.2](https://dl.djicdn.com/downloads/DJI_Dock_3/20260312/DJI_Dock_3_user_manual_en.pdf), July 2025: sections 3.9 Return to Home, 4.2 Loss of C2 Link and 4.3 Loss of Navigation Systems.
- ArduPilot, [GPS failsafe and glitch protection](https://github.com/ArduPilot/ardupilot_wiki/blob/master/copter/source/docs/gps-failsafe-glitch-protection.rst) and [Dead Reckoning Failsafe](https://ardupilot.org/copter/docs/deadreckoning-failsafe.html).
- Pilot Institute, [Drone in-flight emergencies](https://pilotinstitute.com/drone-in-flight-emergency/): "protect people first, property second, and the aircraft last."
- C. J. Clopper and E. S. Pearson, "The use of confidence or fiducial limits illustrated in the case of the binomial", Biometrika 26 (4), 1934.
