# Dynamic Drone Fail-Safe System: Tutorial Outline

The tutorial lives in `../../../../end_to_end_tutorials/dynamic_drone_failsafe`. Every part flies the same drone through the same
night in the same world (`src/`), and each part adds one layer of DeepCausality to the fail-safe controller, in the
order of the Overview pages. Each part ends on what the controller still lacks.

## The problem

A fail-safe often does exactly what it was designed to do, and that is the wrong thing to do where it is. STPA calls
this an unsafe control action: "a control action that, in a particular context and worst-case environment, will lead
to a hazard" (Leveson and Thomas, STPA Handbook, 2018).

- Melbourne Docklands, July 2023: as many as 350 of 500 show drones broke off their routine and, per the operator,
  "did exactly what they should've done with any technical glitch and they auto-rotated and landed", into the Yarra
  River. The cause is not officially determined.
- Orlando, December 2024 (NTSB preliminary report, January 2025): a launch parameter file was never sent and the show
  was rotated 7 degrees, which put the geo hard fence too close to the crowd; drones fell among spectators and a
  7-year-old needed open-heart surgery. A configuration check against the venue before launch addresses this.

The scenario is an inspection drone on a high-voltage line in mountain terrain, where landing in a random place means
the drone tumbles downhill and is destroyed.

## The world (shared, built)

A valley cross-section: creek, 30 deg grass slope with a flat terrace, flat access road, 25 deg rock slope, flat tower
pads every 300 m under the line, a ravine; a 2 m/s downslope night wind. Six crew: four on the road, two on the second
pad. The drone flies the line 60 m across the slope, 40 m above the ground, from 22:00. Faults on a fixed timeline: fix
degraded at 50 s, lost at 55 s, link lost at 65 s, cell failure at 85 s. `Touchdown::assess` judges where the drone
came down: safe, ditched, into the ravine, among people, or tipped and rolled.

The sensors (built): `Drone::scan` reads every 4 m patch in the footprint, which shrinks with altitude: mean
temperature, hot spot, slope, LiDAR return fraction, protrusion. Ground temperature depends on surface and time of day
(water warmer than land at night, cooler by day); people read 30 °C and 1.7 m. Noise grows with altitude and is seeded.

## Part 1: Dynamic causality (built)

- **Adds:** a causal process with no context. Four fault detectors are causaloids evaluated as a collection (anything
  wrong?) and one by one (what?). The process state confirms a lost fix after 3 s and a lost link after 5 s, and
  latches the fail-safe ladder: continue, hold, return home, land now.
- **Shows:** every fault found in order, the textbook fail-safe at each step.
- **Ends on:** the drone drifts 72 m downslope while holding and descending and drops into the creek. *It lacks
  context.*

## Part 2: + Dynamic context (built)

- **Adds:** `deep_causality_context`. Under the drone, a grid of ground patches; each patch is a context node holding
  the fused thermal and LiDAR reading as an uncertain value (`UncertainData`). A time node carries the time of day.
  Each second, new frames update the nodes.
- **Reasoning over the context:**
  - Thermal finds heat sources: people, animals, engines.
  - LiDAR reads geometry: slope, roughness, steps and holes, a person as a 1.5-1.9 m bump, water as missing or
    unnaturally flat returns.
  - The time of day changes what a thermal reading means. Water is cooler than land by day and warmer at night, and
    the two cross over near dawn and dusk. A "cold means water" rule lands in the river at night.
  - Successive frames sharpen each patch's classification as the drone descends and looks again.
- **Shows:** the controller knows what lies below it: grass too steep, the creek, the pad, the crew. When it chooses
  to land, at 69 s, it is over the safe terrace; the wind carries it 44 m into the creek during the descent. Read with
  the daytime rule, none of the 58 water patches reads as water.
- **Built as:** each patch is a space node linked to five `UncertainData` nodes (inverse-variance fusion over frames);
  a `DiscreteTime` clock node. Judgements are uncertain comparisons decided by `probability_exceeds` at 95 %.
- **Ends on:** knowing the ground does not move the drone. *It lacks action.*

## Part 3: + Dynamic action (built)

- **Adds:** a Causal State Machine with four states (choose a landing patch, hold over the ground, fly home, land on
  the chosen patch). Each state's causaloid reads a `Situation` and returns a `Maneuver`. The landing state's causaloid
  holds the chosen patch as its context; choosing a patch replaces the state with a new version
  (`update_single_state`). The drone flies maneuvers by camera and LiDAR (`Drone::guide`), so it no longer drifts.
- **Choice rule:** the safe patch nearest the drone. The target is rechecked each second; state 1 fires again if it
  stops reading as safe (it did not in this run).
- **Shows:** at 69 s the machine picks a corner of the second tower pad 6 m away and lands on it.
- **Ends on:** touchdown 3 m from a crew member, a person the controller's own map showed 4 m from the patch. Every test
  asked about the patch itself; none about distance to people. *Nobody checks the proposal.*

## Part 4: + Effect Ethos (built: the safety protocol)

- **Adds:** the published contingency and emergency procedures (FAA Part 107 guidance, CASA AC 101-01, JARUS SORA,
  ArduPilot fail-safes, pilot practice) encoded as Effect Ethos norms. No briefed knowledge (user ruling 2026-10-01):
  no planned landing zones, no launch site assumed safe; only sensed ground.
- **Urgency stage** replaces the ladder: routine; contingency (lost fix or link, or a low battery): hold, wait 20 s for
  recovery while choosing and looking, then land as soon as practicable; emergency (60 s or less of battery, or a
  failed cell detected as a one-second sag): land as soon as possible, ditching allowed; last resort when nothing is
  permitted: the person bans become banded costs.
- **Ranking:** every candidate put to the Ethos; cheapest permitted flown, sooner first; looks compete with the look's
  time added; with nothing permitted or worth a look, explore unseen ground.
- **Norms:** person (more likely than not within the clearance), not ruled out (99 %; unseen and canopy ground rule out
  no one), path (canopy or unseen ground at the drone's height), battery, drone, edge (seen steep neighbour), sacrifice
  (10³, defeats drone, edge, battery), emergency (defeats not ruled out), near the unknown (10⁴, emergency and last
  resort), last resort (defeats both person norms), bands within 20/10/5 m (10⁵/10⁶/10⁷).
- **Shows:** the default night lands upright on the terrace's middle at 90 s, 34 m from the nearest person.

## Part 5: Verification (built)

- Parts 1, 3 and 4 flown unchanged over 1000 seeded scenarios: any hour, wind 0–6 m/s, gust ≤ 2 m, line anywhere on the
  slope, fault times and order random, a third of fixes and links recover, half the batteries lose a cell, random
  terrace and trees, up to 8 people around the fault point. One-sided 95 % Clopper–Pearson bounds; per-scenario record
  in `campaign_<count>.csv`; any scenario replayable.
- **Result (2026-10-01):** part 4 near a person 0.1 % (≤ 0.47 %), 0 of 495 with a healthy battery (≤ 0.60 %); drone lost
  10.9 % with a healthy battery (part 3: 9.9 %), 61.2 % when a cell fails (part 3: 43.4 %, at 4.4 % near people).
  Residual: scenario 810 (cell fails 6 s into the contingency; every reachable patch beside ground not ruled out).
- The campaign found six failures in earlier part 4 versions; the README lists each with its fix.

## Facts to verify before they are published

- Thermal behaviour of water: humboldt.edu "Interpreting Thermal Images" and the University of Lucknow thermal remote
  sensing notes.
- LiDAR over water: no return or specular scatter; 1550 nm is absorbed (ROCK Robotic white paper, Hamamatsu webinar).
- Show-drone sensors: Verge Aero lists RTK GPS, IMUs, magnetometer, barometer, companion computer, and LiDAR capability
  on the X1; thermal cameras are not claimed for show drones.
