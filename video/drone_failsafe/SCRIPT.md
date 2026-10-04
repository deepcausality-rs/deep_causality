# Dynamic drone fail-safe: narration and shot list

Draft 1, for review. The narration is written for viewers who build, fly or regulate drones and
have never heard of DeepCausality. It names the library on screen and twice in the voice-over.
Every number comes from the tutorial README, the website pages or `campaign_1000.csv`.

Timings assume about 150 words a minute plus a held outcome card at the end of each run.

## Main cut, about 3:45

### 1. Hook (0:00–0:30)

**Screen.** Black. A schematic of Melbourne's Docklands; a counter climbs to 427 as dots fall into
the harbour. Cut to a schematic of Lake Eola, Orlando; a shifted show box slides 18 m toward the
crowd line. Sources in mono at the bottom: ATSB AO-2023-033, ABC News 16 July 2023, NTSB
DCA25LA065. The operator's words appear in type: *"did exactly what they should've done with any
technical glitch."* The injured boy appears in the narration only.

**Voice.**
> July 2023, Melbourne. Wind pushes five hundred show drones off station. Their fail-safes fire,
> and four hundred and twenty-seven drop into the harbour.
>
> December 2024, Orlando. Setup errors shift a show eighteen metres toward the crowd, and a drone
> strikes a seven-year-old boy.
>
> Each fail-safe fired on one condition: a lost position, a crossed line. Safety engineers call
> this an unsafe control action: a correct rule, in the wrong place.

### 2. The world (0:30–0:50)

**Screen.** The valley cross-section draws in: creek, 30° grass slope, terrace, trees, road, rock,
ravine. The power line and the drone at 40 m. Crew rings appear on the road and at the second
tower pad. The timeline strip draws along the bottom with markers at 55, 65 and 85 s.

**Voice.**
> Now one drone, one night. It inspects a power line forty metres above a steep grass slope. A
> creek runs along the valley floor; a crew works on the road and at a tower.
>
> At fifty-five seconds the drone loses satellite positioning. At sixty-five, its link to the
> pilot. At eighty-five, a battery cell. We fly this night four times.

### 3. Run 1: the textbook ladder (0:50–1:15)

**Screen.** Eyebrow: `DeepCausality · Part 1 · dynamic causality`. The drone flies the line. Fault
markers light as each fault is confirmed; the command label steps `continue → hold → land now`.
Wind arrows; the side view shows the drone sliding downhill as it descends. Freeze on touchdown at
91 s.

**Outcome card.** `Every decision correct. In the creek.`

**Voice.**
> Run one: the fail-safe ladder that drone autopilots document today. The controller confirms
> each fault and steps up the ladder: hold, then land. Every decision is correct. The night wind
> carries the drone seventy-two metres down the slope, into the creek.

### 4. Run 2: context (1:15–1:45)

**Screen.** Eyebrow: `Part 2 · dynamic context`. The patch map fills in under the sensor
footprint; each patch sharpens as it is seen again. The crew patches light as rings. A day/night
switch flips the clock: the creek's 58 water patches turn to `?`, then back. The same drift into
the creek.

**Outcome card.** `It knew the ground. In the creek.`

**Voice.**
> Run two gives the drone eyes. A thermal camera and a LiDAR map every patch of ground below, and
> each look sharpens the map: steep grass, a flat terrace, the creek, the crew.
>
> The clock matters too. At night the creek reads warmer than the land; under the daytime rule,
> all fifty-eight water patches turn to unsure. The drone knows the ground. The wind carries it
> into the creek again.

### 5. Run 3: action (1:45–2:10)

**Screen.** Eyebrow: `Part 3 · dynamic action`. The drone holds its place over the ground. At 69 s
the nearest safe patch highlights, 6 m away on the corner of the tower pad. The drone lands. A
distance ring in `--danger` draws from the touchdown to the nearest crew ring: 3 m. The person's
patch on the drone's own map pulses.

**Outcome card.** `3 m from a person.` (in `--danger`)

**Voice.**
> Run three lets the drone act on its map. It holds its place by camera and LiDAR, picks the
> nearest safe patch and lands there, on the corner of a tower pad. Every test it ran asked about
> the patch itself. A member of the crew stood three metres away, on the drone's own map.

### 6. The twist (2:10–2:30)

**Screen.** Eyebrow: `Effect Ethos · first version`. Norm cards stack beside the map. Every
candidate patch flashes `forbidden`. The battery bar drains; the drone hovers; it drops beside a
crew ring, in `--danger`. On screen: `3.9 % of 1000 nights: every one of them a fall like this`.

**Voice.**
> So we add rules: the published emergency procedures for drone pilots, written as norms in
> DeepCausality's Effect Ethos. The first version could only forbid. In a thousand simulated
> nights, every time it put a person at risk, the same thing happened: it forbade every landing,
> the drone hovered until its battery died, and it fell beside the crew. The safety rule became
> the hazard.

### 7. Run 4: the Effect Ethos (2:30–3:10)

**Screen.** Eyebrow: `Part 4 · Effect Ethos`. Urgency label: `routine → contingency → emergency`.
Candidate patches flash with their verdicts: beside the crew `forbidden: person within 13 m`;
terrace edge `forbidden: a gust could tip it`; terrace middle `approved, harm cost 0`. The drone
flies over, descends to 25 m and waits; the recovery window counts down from 20 s. The cell
fails; the drone lands at 90 s. Ring to the nearest person: 34 m, in accent. Then three price
tags over the map: beside the crew `11,111,000`, the creek `11,000`, empty steep grass `1,000`.

**Outcome card.** `Upright. 34 m from the nearest person.`

**Voice.**
> The fix: rules that relax as the danger grows, people first and the drone last.
>
> Run four. The drone proposes landing sites, and the Ethos rules on each and names its reasons.
> Beside the crew: forbidden. The terrace edge: a gust could tip it. The middle of the terrace:
> approved. The drone hovers over it until the recovery window closes. When the cell fails, it is
> already thirteen metres above the approved patch. It lands upright, thirty-four metres from the
> nearest person.
>
> Had every landing been forbidden, the last-resort rules would have priced each place to come
> down: beside the crew, over eleven million; the creek, eleven thousand; empty steep grass, one
> thousand. The drone goes before a person.

### 8. A thousand nights (3:10–3:35)

**Screen.** Three grids of 1000 cells, one per controller, fill in outcome by outcome. A filter
lights the people-at-risk cells in `--danger`: 35, 41, 1. Rates in mono beneath: `3.5 %`,
`4.1 %`, `0.1 % (at most 0.47 % at 95 % confidence)`. The single cell of part 4 zooms open to
scenario 810: the drone at 40 m, the reachable patches each beside unseen ground.

**Voice.**
> One night proves little. So all three controllers flew the same thousand randomised nights,
> with the crew placed where the trouble starts. The textbook ladder put a person at risk on
> thirty-five of them. Context and action, on forty-one. The Effect Ethos, on one. That night,
> number 810, the cell failed with the drone still forty metres up, and every patch its battery
> could reach lay beside ground where someone might be standing.

### 9. Close (3:35–3:45)

**Screen.** A verdict line from the part 4 log types out. End card: `DeepCausality · Dynamic drone
fail-safe`, `deepcausality.com/tutorials/dynamic-drone-failsafe/`, and
`cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_1`.

**Voice.**
> Every verdict names the rules behind it, so an engineer can trace a failure to one rule and fix
> it there, before the drone flies. The tutorial is open source, in Rust, in five parts you can
> run.

## Clips

Each clip opens on its strongest frame with the hook line, plays the scene, and closes on the end
card with its closing line. Captions are burned in.

| Clip | Length | Hook line | Scenes | Closing line |
|---|---|---|---|---|
| 0 Teaser | 30 s | Same drone. Same night. Four fail-safes. | the four outcome cards, then the campaign counts 35, 41, 1 | Five parts, in Rust. |
| 1 Correct, and in the creek | 30 s | Every decision this fail-safe made was correct. | 2 (fault times only), 3 | Next: give it eyes. |
| 2 It knew the ground | 35 s | This drone mapped the creek. Then it landed in it. | 4 | Next: let it act. |
| 3 Three metres | 30 s | This drone chose the safest patch it could see. | 5, plus the counts 35 and 41 from 8 | Next: rules. |
| 4 The rule became the hazard | 25 s | We added safety rules. The rules became the hazard. | 6 | Next: rules that relax. |
| 5 Permission to land | 45 s | This drone asks permission before it lands. | 7 | Next: a thousand nights. |
| 6 A thousand nights | 40 s | Three fail-safes. The same thousand nights. | 8, 9 | Five parts, in Rust. |

## Sources for every number

| On screen or spoken | Value | Kind | Source |
|---|---|---|---|
| Melbourne drones, in the harbour | 500, 427 | external | ATSB AO-2023-033, via the tutorial index page |
| Orlando shift toward the crowd | about 18 m | external | NTSB DCA25LA065, via the tutorial index page |
| Height, fault times | 40 m; 55, 65, 85 s | typed | `src/constants.rs` |
| Run 1 drift | 72 m | computed | README, part 1 |
| Water patches under the daytime rule | 58 to 0 | computed | README, part 2 |
| Run 3 patch distance, touchdown to person | 6 m, 3 m | computed | README, part 3 |
| First version's people-at-risk rate | 3.9 % | sampled | README, "How the campaign changed part 4" |
| Clearance | 13 m | typed | part 4 constants: 10 + 1 + 2 m |
| Run 4 look height, window, height at cell failure, touchdown | 25 m, 20 s, 13 m, 34 m | typed and computed | README, part 4 |
| Last-resort prices | 11,111,000; 11,000; 1,000 | computed | README, part 4 |
| People at risk in 1000 nights | 35, 41, 1; 3.5 %, 4.1 %, 0.1 % (at most 0.47 %) | sampled | README, part 5 |
| Scenario 810 height at cell failure | 40 m | sampled | README, part 5 |
