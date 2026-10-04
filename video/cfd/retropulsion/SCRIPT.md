# Plasma retropulsion: narration and shot list

The cut in `src/` follows this script. Written for engineers who simulate or fly vehicles and have
never heard of DeepCausality. Every number comes from the retropulsion example's `output.txt`, its
trace files, its constants, or the weather example's `weather_table.csv`, which the run reads in
flight; the source table at the end labels each one.

The voice lines below are the captions, with numbers as numerals. `src/script.ts` carries the spoken
form beside each caption that contains a number; its word count at 150 words a minute sets the
phrase length, with a hold before and after each scene. Adjacent scenes overlap by a 0.6 s fade. The
whole cut runs 3:59 (7167 frames at 30 fps).

Every number in a caption is rebuilt from the run in `src/data/captions.ts`; a caption that
disagrees with the run fails the render.

The story is two questions, posed in the first scene and answered in turn: how hard to burn at
supersonic speed (scenes 3 and 4), and when to light the landing burn (scenes 5 to 7). The close
answers both under the same two headings.

## 1. Entry (0:00–0:23)

**Screen.** 3D. A title card over the approach: `Five counterfactual burns`, under it `From plasma
blackout to touchdown, under a retro burn`. The capsule enters heatshield first, its heatshield
glowing. On the second line the agenda appears beside it: `Two questions`, `1 How hard to burn at
supersonic speed?`, `2 When to light the final burn?`

**Voice.**
> The corridor’s capsule flies again, and this time it has to land, braking with its own engine.
> Two questions shape that landing: how hard to burn at supersonic speed, and when to light the
> final burn. DeepCausality answers both by flying the alternatives from one shared state.

## 2. Blackout and the coast (0:22–0:44)

**Screen.** 3D. The flight clock runs through the blackout (the plasma sheath glowing, `GPS lost`),
the coast after it (the heatshield cooling, the altitude strip showing the lift loft), and the
commit. Strips: Mach with the line `ignition band: Mach 0.4 to 2.0`, and altitude with `GPS back` and
`32.7 km` marked. On the last line the engine lights: the plume fires forward out of the heatshield,
the capsule levels to fly on axis, and the camera pans to make room for the plume.

**Voice.**
> It enters at Mach 32 on a day 32 K colder than standard, and loses GPS for 56 s. After blackout it
> coasts, engine off, for another 156 s. At 32.7 km and Mach 2.00 the engine lights, firing forward
> through the heatshield, into the oncoming air.

## 3. Question 1: the pause, mid-burn (0:44–1:10)

**Screen.** 3D, the lit capsule at the fork step. `Question 1 · How hard to burn?` set large. Labels
on the picture: `retro plume, firing forward` and `bow shock, held off by the plume`. On "pauses"
time stops: the plume and wake freeze, the frame dims, the timeline reads `paused`, and the camera
circles the capsule. A card titled `What the pause holds`: altitude 32.68 km, Mach 1.99, dynamic
pressure 2225 Pa, throttle 0.72, propellant 2198.2 kg, and `The marched flow with the plume on it,
shared by all 5 branches.` On the last line five branches leave the capsule, labelled `coast`,
`0.20`, `0.40`, `0.60`, `0.85`, with `5 branches, one starting state · schematic`.

**Voice.**
> The first question: how hard to burn. The plume pushes the bow shock away and destroys the drag
> the capsule relies on, so more thrust is not simply more braking. DeepCausality pauses the lit
> flight and forks it 5 ways: coasting, and throttles of 0.20, 0.40, 0.60 and 0.85.

## 4. Answer 1: the fork (1:09–1:51)

**Screen.** 2D, one step per line.

1. Left: the velocity each branch sheds over its 12 s, drawn from `retropulsion_branch_trace.csv`,
   labelled `coast 139.5`, `0.20 88.4`, `0.40 120.3`, `0.60 152.0`, `0.85 212.1`.
2. Right: deceleration at the end of the branch against the throttle flown; the coasting point and
   the dashed line `coasting: 10.59`.
3. The 0.20 point below the line: `0.20: 7.47, less than coasting`.
4. The row `drag kept` under the axis: 0.25, 0.12, −0.02, −0.06.
5. The harder points and the line through them; `0.85 commanded · envelope flies 0.79`.
6. The footnote brightens: `drag kept: the Jarvinen and Adams (1970) correlation at each branch's
   thrust coefficient, not the simulated plume`.

**Voice.**
> Each branch flies 12 s from the same state. Coasting slows the capsule at 10.59 m/s². Burning at
> 0.20 slows it at only 7.47: the plume removes drag about as fast as thrust replaces it. Only harder
> burns win the deceleration back: 12.89 at 0.60, and 17.20 at 0.85, which the envelope trims to
> 0.79. The drag comes from the Jarvinen and Adams correlation of 1970, carried through each forked
> flight.

## 5. Question 2: the landing margin (1:50–2:22)

**Screen.** 2D. First `Question 2 · When to light the landing burn?` alone. Then the weather table:
navigation drift through blackout against the day's temperature departure, one point per row with
±1σ. A dashed line at −32 K and the interpolated point, `today, −32 K · 54.43 ± 2.62 m`. Right, as
the lines name them: `guidance sized for today, −32 K`, `54.43 + 3 × 2.62 = 62.30 m`, then
`guidance sized for a standard day, 0 K`, `41.60 + 3 × 1.97 = 47.52 m`.

**Voice.**
> The second question: when to light the landing burn. The guidance adds a margin for its
> navigation error, sized from the weather table it read before entry. For today’s cold, the table
> gives 54.43 m of drift, plus or minus 2.62; the margin is the mean plus 3 standard deviations:
> 62.30 m. A guidance that assumed a standard day would use 47.52 m.

## 6. Burn-out and the landing (2:21–2:52)

**Screen.** 3D. The flight resumes from the fork step: the burn to Mach 0.6, cutoff, the fall with
the engine off, the stopping burn. The strips (`throttle flown` with `cutoff · Mach 0.6` and
`stopping burn · 139.12 m`; `descent rate, m/s` with `touchdown · 1.81 m/s`) appear after cutoff, so
they do not cover the plume. On the last line the view cuts to a camera that rides down with the
capsule and stops at the ground: the plume lights the ground and raises dust, and the capsule
settles.

**Voice.**
> The flight resumes from the pause on its own guidance and burns down to Mach 0.6 at 18.5 km. Then
> the engine cuts off and the capsule falls. With today’s margin it lights the stopping burn at
> 139.12 m, and touches down at 1.81 m/s, with 1192.5 kg of propellant left.

## 7. Answer 2: two landings (2:51–3:26)

**Screen.** 2D. Two cards: `sized for today, −32 K` (ignition margin 62.30 m) and `sized for a
standard day` (47.52 m). Height above the touchdown plane against flight time for both landings,
from `retropulsion_trace.csv` and `retropulsion_uninformed_trace.csv`. The burn-light points
(139.12 m and 125.06 m) and `14.06 m higher`; the contact speeds (1.81 and 1.72 m/s); then `The extra
14.06 m of margin costs 7.28 kg of propellant.`

**Voice.**
> To see what the margin buys, the landing is flown twice from the same state, once with each
> margin. Sized for today, it lights at 139.12 m; sized for a standard day, at 125.06 m. On this day
> both land softly, at 1.81 and 1.72 m/s. The extra 14 m of margin costs 7.28 kg of propellant.

## 8. Close (3:26–3:59)

**Screen.** First the two answers under the agenda's headings: `1 · How hard to burn?`, `a light
burn slows the capsule less than coasting`, bars for coasting (10.59 m/s²) and burning at 0.20
(7.47 m/s²); then `2 · When to light the final burn?`, `knowing the day lights it 14.06 m higher`,
the two burn-light altitudes and `14.06 m apart, for 7.28 kg`. Then the descent in one line (entry
at 90 km, ignition at 32.7 km with five branches leaving it, cutoff at 18.5 km, touchdown at
1.81 m/s), the sixteen gate lines from `output.txt`, each `PASS`, and the end card: `DeepCausality
CFD`, `Five counterfactual burns, two counterfactual landings.`,
`cfd.deepcausality.com/tutorial/stage-3-retropulsion`, `cargo run --release -p avionics_examples
--example plasma_blackout_retropulsion`, and `the whole descent: 337.5 s on an Apple M3 Max laptop ·
open source, in Rust`.

**Voice.**
> How hard to burn? The 5 forked burns show that a light burn slows the capsule less than coasting.
> When to light the final burn? The 2 forked landings show that knowing the day costs 7.28 kg for
> 14 m of extra margin. The whole descent, 5491 coupled steps, runs in under 6 minutes on a laptop,
> open source, in Rust.

## Sources for every number

| On screen or spoken | Value | Kind | Source |
|---|---|---|---|
| Measured day | −32 K | typed | `MEASURED_D_TEMP`, `retropulsion/constants.rs` |
| Drift for the day, and the margins | 54.43 ± 2.62 m; 62.30 m; 47.52 m; k = 3 | computed | interpolation of `weather_table.csv`, checked against the Act 0 lines of `output.txt` |
| Table rows | drift mean and σ per row | computed | `weather_table.csv` |
| Entry Mach | 32 (31.94) | computed | `retropulsion_trace.csv`, first row |
| Blackout | from 12.6 s, 56 s (56.10 s) | computed | `retropulsion_trace.csv`; gate (1), `output.txt` |
| Coast after blackout | 156 s (155.6 s) | computed | `retropulsion_trace.csv`, blackout exit to the commit |
| Ignition commit | 32.7 km, Mach 2.00 | computed | `retropulsion_trace.csv`; the commit line of `output.txt` |
| Ignition band | Mach 0.4 to 2.0 | typed | `IGNITION_MACH_MIN`, `IGNITION_MACH_MAX`, `shared/constants.rs` |
| The pause | 32.68 km, Mach 1.99, 2225 Pa, throttle 0.72, 2198.2 kg | computed | the COAST + BURN act of `output.txt`, checked against the trace |
| Roster | coast, 0.20, 0.40, 0.60, 0.85 | typed | `ROSTER`, `retropulsion/constants.rs` |
| Branch length | 12 s | typed | `BRANCH_STEPS` × `DT_FLIGHT` |
| Velocity shed | 139.5, 88.4, 120.3, 152.0, 212.1 m/s | computed | `retropulsion_branches.csv` |
| Deceleration | 10.59, 7.47, 9.97, 12.89, 17.20 m/s² | computed | `retropulsion_branches.csv`, checked against the roster in `output.txt` |
| Drag kept | 0.25, 0.12, −0.02, −0.06 | computed (cited correlation) | `retropulsion_branches.csv` |
| Envelope trims 0.85 to | 0.79 | computed | `retropulsion_branches.csv` |
| Subsonic handover | Mach 0.6 at 18.5 km | typed, computed | `SUBSONIC_HANDOVER_MACH`; the BURN act of `output.txt` |
| Burn-light altitudes | 139.12 m, 125.06 m | computed | belief table, `output.txt`; reproduced by the two traces |
| Contact speeds | 1.81, 1.72 m/s | computed | belief table, `output.txt`; the traces' last rows |
| Propellant | 7.28 kg more; 1192.5 kg left | computed | belief table; gate (6), `output.txt` |
| Steps | 5491 | computed | sum of the act lines, `output.txt`; rows of `retropulsion_trace.csv` |
| Gates | 16, all passing | computed | `output.txt` |
| Run time | 337.5 s | measured | gate (9), Apple M3 Max |

## Notes on what is not physical

- The vehicle is the shared capsule (`video/cfd/shared/src/three/Capsule.tsx`), the same in every
  cut: a 4.23 m aeroshell from the examples' ballistic bundle, with the central retro nozzle. This
  cut uses its plume, its shock edge and its variable angle of attack.
- The attitude: the flight-path angle is read off the trace, `asin(descent rate / speed)`. The angle
  of attack is drawn: 20° before ignition, then 0°, because the examples fly lift as a point-mass
  L/D and the retro burn on axis.
- The plume's length and shape are drawn from the throttle flown. Below the plasma regime the bow
  shock is drawn as a faint edge, the way a schlieren photograph shows it, standing off ahead of the
  plume; its distance is drawn too.
- The wall temperature uses the run's stagnation heat flux, which the examples compute with the
  RAM-C II nose radius of 0.1524 m, so the glow overstates a 4.23 m heatshield's.
- The ground, its lighting by the plume and the dust are drawn. The touchdown plane is the altitude
  at the last recorded step.
- The fan of branches in scene 4 is schematic. Scene 5 draws the real branches.
- The roster is a measurement: the flight after the fork continues on its own guidance, not on a
  roster branch.
