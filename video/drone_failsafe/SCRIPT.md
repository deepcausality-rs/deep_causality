# Dynamic drone fail-safe: on-screen text

The words on screen, scene by scene, and where each number comes from. The video has no voice.
`src/script.json` holds this text, and the scenes read it from there; this file mirrors it.

## Title (5.5 s)

> [DeepCausality logo]
>
> DEEPCAUSALITY TUTORIAL
>
> **Dynamic Drone Fail-Safe**
>
> Where should a failing drone land?

## Setting (10 s)

Beside the 3D valley, one line after another:

> A drone inspects a power line at night.
>
> A repair crew works on the access road and a tower pad.
>
> A fail-safe takes over as 3 faults strike:
> `55 s GPS lost · 65 s link lost · 85 s cell fails`

Labels in the scene: creek, terrace, repair crew, power line.

## Tutorial part 1 card (5.5 s)

> TUTORIAL PART 1
>
> **The textbook fail-safe**
>
> It follows the defaults that drone autopilots document today. Watch it face the 3 faults.

## Textbook (25 s)

> TUTORIAL PART 1 · TEXTBOOK FAIL-SAFE

A running log: each line appears at its flight second, below the one before.

| Flight second | Log line |
|---|---|
| 50 | GPS signal degrades |
| 57 | GPS loss confirmed → rule says hold for the pilot |
| 58 | The night wind pushes the drone downhill |
| 69 | Link loss confirmed → rule says land; drone descends |
| 85 | Cell fails → drone keeps descending |
| 91 | Drone drops into the creek |

At touchdown: **The fail-safe loses the drone.**

## The issue (10 s)

> TUTORIAL PART 1 · THE ISSUE
>
> **A static rule**
>
> Each fault triggers a fixed action, whatever lies below.
>
> So the drone can come down in a creek, on steep grass or beside people.

## Tutorial part 4 card (8.5 s)

> TUTORIAL PART 4
>
> **The dynamic fail-safe**
>
> DeepCausality's Effect Ethos makes the fail-safe dynamic.
>
> It checks each landing site against rules from published pilot procedures and the context the
> drone senses every second.

## Effect Ethos (34 s)

> TUTORIAL PART 4 · DYNAMIC FAIL-SAFE

The same running log, with the rule behind each line beneath it.

| Flight second | Log line | Rule line |
|---|---|---|
| 50 | GPS signal degrades | |
| 57 | GPS loss confirmed → drone steers by camera and LiDAR | contingency: wait 20 s for recovery |
| 57 | Effect Ethos forbids landing beside the crew | rule: keep 13 m clear of people |
| 64 | Effect Ethos permits the middle of the terrace | every rule passes |
| 69 | Link loss confirmed → drone waits over the terrace | contingency: battery healthy, keep waiting |
| 77 | Recovery window closes → drone lands on the terrace | contingency: land as soon as practicable |
| 85 | Cell fails → Effect Ethos confirms the terrace | emergency: land as soon as possible |
| 90 | Drone lands upright, 34 m from the crew | |

At touchdown: **The fail-safe saves the drone.**

## 1,000 simulations (14 s)

> **The tutorial simulated this scenario 1,000 different ways**
>
> Each simulation changed the launch time, wind, faults, terrain and where up to 8 people stood.
> Both fail-safes flew all 1,000.
>
> TEXTBOOK FAIL-SAFE: **35** unsafe landings within 10 m of a person · 3.5 %
>
> DYNAMIC FAIL-SAFE: **1** unsafe landing within 10 m of a person · 0.1 % · at most 0.47 % at 95 %
> confidence

## How DeepCausality builds the dynamic fail-safe (12 s)

| Step | Text |
|---|---|
| Dynamic context | A thermal camera and LiDAR update a context of the ground every second. |
| Dynamic reasoning | Every second, a causal process confirms each fault and declares contingency or emergency. |
| Dynamic Effect Ethos | Rules from published pilot procedures check each landing site against the context, people first. |

## End (11 s)

> Build the dynamic fail-safe in Rust, step by step.
>
> [DeepCausality logo] deepcausality.com/tutorials/dynamic-drone-failsafe
> `cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_4`
>
> [Center for Dynamic Causality logo] causalcenter.com

## Sources for every number

| On screen | Value | Kind | Source |
|---|---|---|---|
| Fault times | 50, 55, 65, 85 s | typed | `src/constants.rs` |
| Log seconds | 57, 58, 64, 69, 77, 90, 91 s | computed | `part_1_trace.csv`, `part_4_trace.csv`, `part_4_rulings.csv`, `part_N_touchdown.csv` |
| Recovery window | 20 s | typed | part 4 constants |
| Clearance ring | 13 m | typed | part 4 constants: 10 + 1 + 2 m |
| Ethos touchdown to the nearest person | 34 m | computed | `part_4_touchdown.csv`; README, part 4 |
| People per simulation | up to 8 | typed | README, part 5 |
| Unsafe landings | 35 and 1; 3.5 %, 0.1 %, at most 0.47 % | sampled | `campaign_1000.csv`; README, part 5 |

`pnpm data` stops the build when a log line sits at a second its trace records no event for, or when
any text in `src/script.json` states a number outside the list of numbers the tutorial states.
