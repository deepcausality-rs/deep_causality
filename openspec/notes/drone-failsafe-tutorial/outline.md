# Dynamic Drone Fail-Safe System: Tutorial Outline

The tutorial lives in `end_to_end_tutorials/dynamic_drone_failsafe`. Every part flies the same drone through the same
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

A valley cross-section: creek, 30 deg grass slope, flat access road with a maintenance crew, 25 deg rock slope, flat
tower pads, a ravine; a 2 m/s downslope night wind. The drone flies the line 40 m above the ground. Faults on a fixed
timeline: fix degraded at 50 s, lost at 55 s, link lost at 65 s, cell failure at 85 s. `Touchdown::assess` judges where
the drone came down: safe, ditched, into the ravine, among people, or tipped and rolled.

Parts 2 to 4 extend the world with what the sensors need: a ground temperature per surface that depends on the time of
day, the crew's body heat, and LiDAR returns (height, roughness, no return over water).

## Part 1: Dynamic causality (built)

- **Adds:** a causal process with no context. Four fault detectors are causaloids evaluated as a collection (anything
  wrong?) and one by one (what?). The process state confirms a lost fix after 3 s and a lost link after 5 s, and
  latches the fail-safe ladder: continue, hold, return home, land now.
- **Shows:** every fault found in order, the textbook fail-safe at each step.
- **Ends on:** the drone drifts 72 m downslope while holding and descending, touches down on 30 deg grass, tips over
  and tumbles into the creek; a tower pad's centre was 10 m away when the fix dropped. *It lacks context.*

## Part 2: + Dynamic context

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
- **Shows:** the controller knows what lies below it: grass too steep, the creek, the pad, the crew.
- **Ends on:** knowing the ground does not move the drone. *It lacks action.*

## Part 3: + Dynamic action

- **Adds:** the Causal State Machine proposes and flies a descent plan: choose a target patch, approach at reduced
  speed, spiral down, recheck the target during the descent, abort and reselect when the evidence changes. States are
  swapped at runtime (`update_single_state`, versioned) as the context changes.
- **Shows:** the fail-safe adapts to what the fusion reports, and keeps checking that its context still holds.
- **Ends on:** it proposes the best patch it can see even when the battery cannot get it there or wind drift makes the
  touchdown uncertain. *Nobody checks the proposal.*

## Part 4: + Effect Ethos

- **Adds:** each proposal becomes a `ProposedAction` and the Effect Ethos judges it against norms:
  - no person within a clearance radius, widened by wind and position uncertainty;
  - no water, no ravine, no slope above the tip-over angle;
  - enough battery to reach the patch and descend, with reserve;
  - descent speed within limits for the wind.

  An impermissible proposal is rejected with the norm that rejected it, and the controller proposes again until one is
  approved. Only an approved fail-safe executes.
- **Shows:** a safe descent to a safe landing zone, with every proposal, verdict and norm in the decision log.
- **Evaluation:** randomised terrain layouts, times of day, winds and fault points. Three controllers:
  1. fixed "land now" (the Melbourne behaviour);
  2. a competent single-frame version: classify the patch below once and land there or at the best visible patch, with
     no fusion over frames, no context-dependent interpretation and no norm check;
  3. the full stack.

  Metrics: touchdowns among people, in water, into the ravine, on slopes above the tip-over angle, battery exhaustion,
  and false aborts. The result decides the published wording, including a tie.

## Facts to verify before they are published

- Thermal behaviour of water: humboldt.edu "Interpreting Thermal Images" and the University of Lucknow thermal remote
  sensing notes.
- LiDAR over water: no return or specular scatter; 1550 nm is absorbed (ROCK Robotic white paper, Hamamatsu webinar).
- Show-drone sensors: Verge Aero lists RTK GPS, IMUs, magnetometer, barometer, companion computer, and LiDAR capability
  on the X1; thermal cameras are not claimed for show drones.
