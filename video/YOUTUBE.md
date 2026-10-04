# YouTube uploads

Title, description and chapters for each video, ready to paste. Chapter times are each scene's first
frame, from the cut's own timeline; every chapter lasts at least the 10 s YouTube requires. Every
number in a description is one the video shows.

For each upload: Category *Science & Technology*, language *English*. Captions: upload the `.srt`
as an English subtitle track where one exists. Thumbnail: the `_thumbnail.jpg` in the same `out/`
folder, 1280 × 720.

## 1. Corridor

**File:** `cfd/plasma_blackout/out/plasma_blackout_corridor_4k.mp4` · captions
`plasma_blackout_corridor.srt` · thumbnail `plasma_blackout_corridor_thumbnail.jpg`

**Title:** Steering a capsule through GPS blackout: 17 counterfactual trajectories | DeepCausality CFD

**Description:**

```text
A capsule enters the atmosphere at Mach 29. 13 seconds later, at 73 km, the plasma around it blocks GPS, and guidance must still choose a bank angle.

DeepCausality pauses the simulated flight the moment GPS is lost and forks it 17 times in 2 rounds, one bank angle per branch, all from the same paused state. Each branch flies the same coupled simulation 10 s ahead; 11.5° comes closest to the aim, and the paused flight resumes with it. Flying uncorrected would miss the aim by 20 m; the counterfactual choice cuts that to 2.07 m, inside the 5.6 m the navigation filter reports as its own uncertainty.

Every number on screen comes from the example's own run and is checked against it when the video renders. The whole run takes 44.4 s on an Apple M3 Max laptop.

Tutorial: https://cfd.deepcausality.com/tutorial/stage-1-corridor/
Run it: cargo run --release -p avionics_examples --example plasma_blackout_corridor
Code: https://github.com/deepcausality/deep_causality.rs
DeepCausality: https://deepcausality.com
The Center for Dynamic Causality: https://www.causalcenter.com

Music: an original theme, synthesized in SuperCollider.

Chapters
0:00 A capsule enters the GPS blackout
0:16 The question: which bank angle?
0:29 Pausing the flight when GPS is lost
1:01 17 forked trajectories
1:50 Committing 11.5° and flying on
2:36 The result: 20 m down to 2.07 m
```

## 2. Weather

**File:** `cfd/weather/out/weather_4k.mp4` · captions `weather.srt` · thumbnail
`weather_thumbnail.jpg`

**Title:** Does the weather move the GPS blackout? 6 counterfactual atmospheres | DeepCausality CFD

**Description:**

```text
Before a capsule flies, its flight computer needs a table of what the weather does to its descent. DeepCausality flies one validated descent through 6 counterfactual atmospheres, 8 receiver-noise draws each: 48 descents, flown in parallel.

Does the weather move the GPS blackout? It shifts it by 2.5 s and changes its length by 1.1 s.
Does it move the navigation error? Polar winter drifts 58.71 m against the standard day's 41.60 m, 1.41 times as far and 5.7 times the receiver noise. The accelerometer bias departs from its calibration by 1% per kelvin, and polar winter flies 40 K from it.

Every number on screen comes from the example's own run and is checked against it when the video renders. The whole campaign takes 184.0 s on an Apple M3 Max laptop.

Tutorial: https://cfd.deepcausality.com/tutorial/stage-2-weather/
Run it: cargo run --release -p avionics_examples --example plasma_blackout_weather
Code: https://github.com/deepcausality/deep_causality.rs
DeepCausality: https://deepcausality.com
The Center for Dynamic Causality: https://www.causalcenter.com

Music: an original theme, synthesized in SuperCollider.

Chapters
0:00 A table for the flight computer
0:21 One baseline, 5 alternated atmospheres
0:46 Question 1: does the weather move the blackout?
1:11 Question 2: does it move the navigation error?
1:31 Why polar winter drifts further
2:06 Real, or receiver noise?
2:36 The answers
```

## 3. Retropropulsion

**File:** `cfd/retropropulsion/out/retropropulsion_4k.mp4` · captions `retropropulsion.srt` ·
thumbnail `retropropulsion_thumbnail.jpg`

**Title:** Supersonic retropropulsion: 5 counterfactual burns, 2 counterfactual landings | DeepCausality CFD

**Description:**

```text
The corridor's capsule flies again and lands under its own engine, firing forward through the heatshield into the oncoming air. 2 questions shape that landing.

How hard to burn at supersonic speed? DeepCausality pauses the lit flight and forks it 5 ways. Burning at 0.20 throttle slows the capsule at 7.47 m/s², less than coasting at 10.59 m/s²: the plume removes drag about as fast as thrust replaces it.

When to light the final burn? The landing is flown twice from the same state, with the margin sized for today's cold and for a standard day. Knowing the day lights the burn 14 m higher, for 7.28 kg of propellant.

Every number on screen comes from the example's own run and is checked against it when the video renders. The whole descent, 5491 coupled steps, takes 337.5 s on an Apple M3 Max laptop.

Tutorial: https://cfd.deepcausality.com/tutorial/stage-3-retropropulsion/
Run it: cargo run --release -p avionics_examples --example plasma_blackout_retropropulsion
Code: https://github.com/deepcausality/deep_causality.rs
DeepCausality: https://deepcausality.com
The Center for Dynamic Causality: https://www.causalcenter.com

Music: an original theme, synthesized in SuperCollider.

Chapters
0:00 2 questions for a powered landing
0:22 Blackout and the coast
0:43 Question 1: how hard to burn?
1:09 5 forked burns
1:50 Question 2: when to light the final burn?
2:21 The landing
2:51 2 counterfactual landings
3:25 The answers
```

## 4. Drone fail-safe

**File:** `drone_failsafe/out/Main.mp4` (1920 × 1080; the narration is on-screen text, so there is
no caption file) · thumbnail `drone_failsafe_thumbnail.jpg`

**Title:** Where should a failing drone land? A dynamic fail-safe in Rust | DeepCausality

**Description:**

```text
A drone inspects a power line at night while a repair crew works below. 3 faults strike, and a fail-safe takes over.

Tutorial part 1 flies the textbook fail-safe, the defaults drone autopilots document today: each fault triggers a fixed action, whatever lies below, and the fail-safe loses the drone. Part 4 makes the fail-safe dynamic with DeepCausality's Effect Ethos, which checks each landing site against rules from published pilot procedures and the context the drone senses, people first. It saves the drone.

The tutorial flew both fail-safes through 1,000 simulated nights, each changing the launch time, wind, faults, terrain and where up to 8 people stood. The textbook fail-safe landed within 10 m of a person 35 times (3.5%); the dynamic fail-safe once (0.1%, at most 0.47% at 95% confidence).

Tutorial: https://deepcausality.com/tutorials/dynamic-drone-failsafe/
Run it: cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_4
Code: https://github.com/deepcausality/deep_causality.rs
DeepCausality: https://deepcausality.com
The Center for Dynamic Causality: https://www.causalcenter.com

Chapters
0:00 Where should a failing drone land?
0:15 Part 1: the textbook fail-safe
0:46 A static rule
0:56 Part 4: the dynamic fail-safe
1:38 1,000 simulated nights
1:52 How DeepCausality builds the dynamic fail-safe
2:04 Build it yourself
```
