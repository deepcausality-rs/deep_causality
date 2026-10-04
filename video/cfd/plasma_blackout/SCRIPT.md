# Plasma-blackout corridor: narration and shot list

The cut in `src/` follows this script. Written for engineers who simulate or fly vehicles and have
never heard of DeepCausality. Every number comes from the corridor example's `output.txt`, its trace
files, or its constants; the source table at the end labels each one.

The voice lines below are the captions, with numbers as numerals. `src/script.ts` carries the spoken
form beside each caption that contains a number ("Mach twenty-nine"); its word count at 150 words a
minute sets the phrase length, with a hold before and after each scene. Adjacent scenes overlap by
a 0.6 s fade. The whole cut runs 3:06 (5565 frames at 30 fps).

Every number in a caption is rebuilt from the run in `src/data/captions.ts`; a caption that
disagrees with the run fails the render.

## 1. The problem (0:00–0:17)

**Screen.** 3D. A title card over the approach: `Seventeen counterfactual trajectories`, under it
`A reentry through plasma blackout, forked mid-flight`. Stars, the Earth limb at 90 km, the capsule
entering heatshield first at Mach 29. The shock layer brightens along the silhouette as the air
thickens; the heatshield glows hottest where the flow stagnates. Inset bottom-left, a strip: the
plasma frequency climbing toward a
dashed amber line labelled `GPS L1`. Thin lines from three satellites reach the vehicle. At 13.1 s
the strip crosses the line, the satellite lines fade, the readout top-right turns to `GPS lost`,
and the strip marks `GPS lost · 73.2 km`. The timeline along the top marks `GPS lost · 13.1 s`.

**Voice.**
> A capsule enters the atmosphere at Mach 29. The air in front of it turns to plasma, and 13
> seconds later, at 73 km, the plasma blocks GPS. From here it flies blind.

## 2. The question (0:17–0:29)

**Screen.** 2D, a plan view from above. The descent so far ends at the vehicle; the no-bank path
runs ahead. An aim point appears 20 m to the side, labelled `aim` and `20 m to the side · set by the
example`. A dial shows the bank angle: the roll that tilts the lift sideways. A label at the vehicle
reads `GPS lost · nav error 0.18 m`, the paused value. Four dotted candidate paths and a `?` follow
the last line.

**Voice.**
> Guidance must choose a bank angle now: how far to roll its lift sideways to reach an aim point
> 20 m off its path. Which angle lands closest?

## 3. The pause (0:29–1:02)

**Screen.** 3D, the vehicle at 73.2 km. On "pauses the flight" time stops: the wake freezes, the
frame dims, and the timeline reads `paused`. The camera circles the frozen vehicle. A card titled
`What the pause holds` fills in row by row as the narration names each part, read off the trace:
altitude 73.2 km and Mach 27.2 on "the flow", electron density 3.3 × 10¹⁶ m⁻³ on "the plasma
sheath", navigation error 0.18 m on "the navigation filter". On the last line seventeen branches
leave the heatshield along the flight path, labelled `17 branches, one starting state · schematic`, and
the card adds `Shared by all 17 branches.`

**Voice.**
> DeepCausality pauses the flight the moment GPS is lost, at 13.1 s and 73.2 km. The pause holds
> the whole coupled state: the flow around the vehicle, the plasma sheath at 3.3 × 10¹⁶ electrons
> per m³, and the navigation filter, already 0.18 m off. Every candidate bank angle forks from this
> one state, so all 17 alternatives share the same past.

## 4. The fork (1:02–1:51)

**Screen.** 2D, one step per line. Left panel: sideways distance from the no-bank path against time
since the pause, 0 to 10 s, drawn from `corridor_branch_trace.csv`.

1. The axes, the aim line at 20 m and the `+10 s` mark.
2. Six coarse branches draw in, each labelled with its angle and miss: 0° 20.0 m, 5° 11.6 m,
   10° 3.5 m, 15° 6.0 m, 20° 14.3 m, 40° 28.9 m.
3. The 40° branch brightens and a label reads `40° commanded · gate flies 28.6°`.
4. The 10° branch turns white.
5. Eleven fine branches draw in around it.
6. Right panel: the end plane at one scale on both axes. The seventeen end points land one by one
   on a faint arc, the arc draws through them, and the aim sits just off it.
7. The committed 11.5° branch draws in accent, a line from the aim to its end reads
   `11.5° · 2.07 m`, and below: `11.5° lands 2.07 m from the aim`,
   `no bank: 20.0 m · 9.7× closer · 17 branches, one past`.

**Voice.**
> Each branch flies the same coupled simulation 10 s ahead with its own bank angle. The first round
> flies 6 angles in parallel, from 0° to 40°. 40° exceeds the safety envelope, so the gate clamps it
> to 28.6°. 10° lands 3.5 m from the aim, the best of the round. A second round forks the same
> pause again: 11 angles around 10°, 0.5° apart. Every end lands on one arc set by the bank angle,
> and the aim sits just off it. 11.5° comes closest: 2.07 m from the aim, against 20 m with no bank.

## 5. Commit and fly through (1:50–2:37)

**Screen.** Back to 3D, the committed branch from the pause to the end of the run. The sheath
brightens through the peak passage. Inset strip: electron density with a marker reading
`60.9 km: 2.6 × 10¹⁹`, under a dashed line labelled `RAM-C II flight, 1970: 10¹⁹`. A second strip:
navigation error rising through the dark to `42.4 m in the dark`. At 69.0 s and 46.8 km the readout
turns to `linked`, the satellite lines return, the error drops to `first fix: 2.29 m`, and
`at the end: 0.28 m` appears below the readout.

**Voice.**
> DeepCausality commits 11.5° and resumes the paused flight in that world. The plasma thickens: at
> 61 km the sheath holds 2.6 × 10¹⁹ electrons per m³, against 10¹⁹ measured on the RAM-C II flight
> in 1970. With no GPS, the vehicle dead-reckons on its inertial sensors. Over 56 s of blackout, its
> navigation error grows from 0.18 m to 42.4 m. At 46.8 km the sheath clears and GPS returns; the
> first fix cuts the error to 2.29 m, and 3 s later to 0.28 m.

## 6. Close (2:37–3:06)

**Screen.** First the result, in the end plane at one scale on both axes, 10 s after the pause:
`Without GPS: 20.0 m`, the no-bank end labelled `0° · flying uncorrected` 20.0 m from the aim.
Then `→ 2.07 m`: the committed end, labelled `11.5° · 2.07 m`, and around it a ring of radius
5.6 m labelled `navigation filter, 1σ: 5.6 m`, the vehicle's own position uncertainty when the
branches are scored. The aim lies inside the ring.

Then the run condensed into one line: entry at 90 km, the pause at 73.2 km with seventeen branches
leaving it (`17 branches, one committed: 11.5°`), GPS back at 46.8 km. The thirteen gate lines from
`output.txt` appear one by one, each `PASS`. End card: `DeepCausality CFD`, `Seventeen
counterfactual trajectories, one shared past.`, `cfd.deepcausality.com/tutorial/stage-1-corridor`,
`cargo run --release -p avionics_examples --example plasma_blackout_corridor`, and `the whole run:
44.4 s on an Apple M3 Max laptop · open source, in Rust`.

**Voice.**
> With GPS gone, flying uncorrected would miss the aim by 20 m. Counterfactuals from the last known
> state cut that to 2.07 m, inside the 5.6 m the navigation filter reports as its own uncertainty.
> The whole run takes 44 s on a laptop, and the corridor is open source, in Rust.

## Sources for every number

| On screen or spoken | Value | Kind | Source |
|---|---|---|---|
| Entry altitude | 90 km | typed | `TRUTH_ALTITUDE_0`, `shared/constants.rs` |
| Entry Mach | 29.1 | computed | `corridor_trace.csv`, first row |
| GPS lost: time, altitude, Mach | 13.1 s, 73.2 km, 27.2 | computed | `output.txt` leg 1; `corridor_trace.csv` |
| Electron density and nav error at the pause | 3.27 × 10¹⁶ m⁻³, 0.1823 m | computed | `output.txt` leg 1 |
| Aim offset, and so the no-bank miss | 20 m, 20.0 m | typed | `AIM_CROSS_RANGE_M`, `corridor/constants.rs`; `aim_point` in `model.rs` |
| Candidate angles | 0, 5, 10, 15, 20, 40°; then 7.5–12.5° by 0.5° | typed | `BANK_ANGLES_DEG`, `FINE_SPAN_STEPS`, `FINE_STEP_DEG` |
| Branch length | 10 s | typed | `BRANCH_STEPS` × `DT_FLIGHT` |
| 40° flown at | 28.6° | typed, computed | `MAX_BANK_RAD` = 0.5 rad; `corridor_branch_trace.csv` bank column |
| Coarse misses | 20.0, 11.6, 3.5, 6.0, 14.3, 28.9 m | computed | `output.txt` coarse table |
| Committed angle and miss | 11.5°, 2.07 m | computed | `output.txt` fine table |
| Improvement over no bank | 9.7× | computed | gate (4e), `output.txt` |
| The end arc | radius 96.67 m, every end within 17 mm | derived | one-parameter fit to the 17 end positions in `corridor_branch_trace.csv` |
| Navigation filter 1σ when the branches are scored | 5.6 m | derived | √ of `nav_var` (trace of the position covariance), committed branch's last row, `corridor_branch_trace.csv` |
| Peak passage | 60.9 km, 2.6 × 10¹⁹ m⁻³ | computed | `output.txt` leg 2 |
| RAM-C II reference | 10¹⁹ m⁻³, 1970 | typed (flight data) | `RAMC_NE_REFERENCE`, `shared/constants.rs` |
| Blackout duration | 56 s | computed | gate (1), `output.txt` (55.9 s) |
| Largest navigation error in the dark | 42.4 m | computed | `corridor_trace.csv`, last denied step |
| GPS back | 69.0 s, 46.8 km | computed | `corridor_trace.csv`; `output.txt` leg 3 |
| Error after the first fix, and at the end | 2.29 m, 0.28 m | computed | `output.txt` legs 3 and 4 |
| Reacquisition | 3 s | typed | `REACQ_STEPS` × `DT_FLIGHT` |
| Gates | 13, all passing | computed | `output.txt` |
| Run time | 44.4 s | measured | gate (5b), Apple M3 Max |

## Notes on what is not physical

- The fan of branches in scene 3 is schematic: its spread carries no data. Scene 4 draws the real
  branches.
- The 3D vehicle is the capsule the examples fly: a 4.23 m aeroshell, the diameter the ballistic
  bundle implies (`CDA_OVER_M · VEHICLE_MASS_KG / VEHICLE_CD` = 14.09 m²), with the central retro
  nozzle of `NOZZLE_EXIT_R` = 0.42 m in its heatshield. Its proportions follow the Apollo command
  module, the lineage `VEHICLE_CD` cites. The 20° tilt to the flow is drawn: the examples fly lift
  as a point-mass L/D of 0.3 and carry no attitude. The roll is the bank angle the trace records.
- The wall temperature uses the run's stagnation heat flux, which the examples compute with the
  RAM-C II nose radius of 0.1524 m. A 4.23 m capsule's broader heatshield sees less flux, so the
  glow overstates it. The backshell's heating is an assumed 2 % of the stagnation value.
- The 3D register is shaded in real time. Emission strengths are tone-mapped; absolute brightness
  is not physical.
- The bow shock's shape is drawn, not computed.
- The glow around the stagnation point is a drawn bloom sprite, not a radiative quantity.
- In scene 2, downrange distance is compressed so the 20 m aim offset is visible next to kilometres
  of flight; the panels of scenes 4 and 6 use true metres on both axes.
- The branches are scored against the simulation's true positions, which a flight computer does
  not see; it would steer from its navigated estimate. The corridor README makes the same point.
