# Plasma-blackout weather: narration and shot list

The cut in `src/` follows this script. Written for engineers who simulate or fly vehicles and have
never heard of DeepCausality. Every number comes from the weather example's `output.txt`, its table,
draw and trace files, its audit logs, or its constants; the source table at the end labels each one.

The voice lines below are the captions, with numbers as numerals. `src/script.ts` carries the spoken
form beside each caption that contains a number; its word count at 150 words a minute sets the
phrase length, with a hold before and after each scene. Adjacent scenes overlap by a 0.6 s fade. The
whole cut runs 3:08 (5643 frames at 30 fps).

Every number in a caption is rebuilt from the run in `src/data/captions.ts`; a caption that
disagrees with the run fails the render.

The story is two questions, posed in the first scene and answered in turn: does the weather move
the GPS blackout (scene 3), and does it move the navigation error (scenes 4 to 6). The close answers
both under the same two headings.

## 1. Entry (0:00–0:21)

**Screen.** 3D. A title card over the approach: `Six counterfactual atmospheres`, under it `One
descent through six weathers, reduced to one table`. The capsule enters heatshield first on the
standard day's reference trace, from entry to just past the blackout onset; the readout shows the
world, altitude, Mach, GPS and the navigation error, and the flight clock shades the blackout. On
the second line the agenda appears: `Two questions`, `1 Does the weather move the GPS blackout?`,
`2 Does it move the navigation error?` By the last line the sheath glows and GPS reads `lost`.

**Voice.**
> Before this capsule flies, its flight computer needs a table: what the weather does to its
> descent. Two questions: does the weather move the GPS blackout, and does it move the navigation
> error? DeepCausality answers both by flying one validated descent through 6 counterfactual
> atmospheres.

## 2. The worlds (0:21–0:46)

**Screen.** 2D. The baseline card, `standard day`, `the corridor’s validated descent`,
`0 K · density 1.00×`. Five cards fan out from it, one per alternated world, each with its
temperature offset and density scale: hot day +20 K, 0.90×; cold day −25 K, 1.10×; polar winter
−40 K, 1.20×; thin day −5 K, 0.75×; dense day +5 K, 1.30×. On the third line each card shows the
alternation line its reference draw wrote to its audit log, verbatim:
`!!ContextAlternation!!: world 'standard_day' replaced with 'polar_winter'`. On the last line eight
pips, one per draw, fill every card, and `6 worlds × 8 draws = 48 descents, flown in parallel`.

**Voice.**
> The standard day is the baseline: the corridor’s validated descent. From it, DeepCausality
> alternates 5 atmospheres: hot, cold, polar winter, thin and dense, each set by a temperature
> offset and a density scale. Everything else stays the baseline, and each descent’s audit log
> records the swap. Each world flies 8 receiver-noise draws: 48 full descents, flown in parallel.

## 3. Question 1: the blackout window (0:46–1:11)

**Screen.** `Question 1 · Does the weather move the GPS blackout?` set large. One amber bar per
world, from the step GPS is lost to the last step without it, densest air on top. On the second line
the axis zooms onto 11 to 16 s and each onset is labelled, `lost at 12.1 s` for the dense day to
`lost at 14.6 s` for the thin day. On the third it zooms back out to the whole 85 s and each bar
shows its length, 55.0 to 56.1 s. On the last: `onset moves 2.5 s`, `length moves 1.1 s`.

**Voice.**
> The first question: does the weather move the blackout? Denser air ionizes earlier: the dense day
> loses GPS at 12.1 s, the thin day at 14.6 s. But every blackout lasts between 55.0 and 56.1 s.
> The weather shifts the window by 2.5 s, and barely changes its length.

## 4. Question 2: the navigation error (1:11–1:31)

**Screen.** `Question 2 · Does it move the navigation error?` set large. Left: the navigation error
of each world's reference draw (draw 0) against flight time, drawn as the clock runs, with the
blackout shaded; polar winter in cyan, the standard day in white, the others grey. Each grows
through the blackout and drops back under a metre once GPS returns. Right, on the last line: the
largest error while GPS is lost, mean ± 1 sd over 8 draws, one bar per world; `41.60 m`, `58.71 m`,
and a bracket `1.41×`.

**Voice.**
> The second question: does the weather move the navigation error? Without GPS, the error grows
> through the whole blackout. On a standard day it reaches 41.60 m on average; in polar winter,
> 58.71 m, 1.41 times as far.

## 5. Why (1:31–2:07)

**Screen.** `Not the window`: two bars on one scale, `1.04× · longest blackout over shortest,
squared` and `1.41× · polar winter over standard day, measured`. Then two panels. `The instrument`:
the accelerometer bias each world flies against its temperature offset, the V of
`1 + 0.010/K × |dT|` with the six worlds on it, and the dashed `the filter assumes 1.00`; on the
third line polar winter lights, `40 K out · 1.40×`. `The prediction`: predicted against measured
drift, mean ± 1 sd, for every world, around the diagonal; under it
`41.60 m × departure × (dwell / 55.7 s)²` and `every world within 3.4%`.

**Voice.**
> Why? Not the window: the longest blackout is 2% longer than the shortest, which moves the drift by
> about 4%. The instrument: the accelerometer bias departs from its calibration by 1% per kelvin,
> while the navigation filter assumes a standard day. Polar winter sits 40 K from the calibration
> point and flies 1.40 times the bias. Scaled by that factor and the square of the blackout’s
> length, the standard day’s drift predicts every world within 3.4%.

## 6. Real, or receiver noise? (2:06–2:37)

**Screen.** `Real, or receiver noise?` Left: every draw's largest error while GPS is lost, one
column per world, with the mean and a ±1 sd band. On the third line the standard day and polar
winter stay lit and a bracket joins their means: `17.11 m apart = 5.7 × 3.01 m combined sigma`.
Right, on the last line: every draw's error after GPS returns, all 48, under the dashed
`gate (5): under 1.0 m`, with `worst 0.22 m`.

**Voice.**
> Is the difference real, or receiver noise? Each dot is one draw: receiver noise scatters a world’s
> drift by 1.71 to 2.93 m, one standard deviation. Polar winter and the standard day sit 17.11 m
> apart: 5.7 times their combined scatter. And once GPS returns, every draw of every world recovers
> to within 0.22 m.

## 7. Close (2:36–3:08)

**Screen.** First the two answers under the agenda's headings: `1 · Does the weather move the
blackout?`, `it shifts by 2.5 s; its length moves 1.1 s`, the six windows on one clock; then
`2 · Does it move the navigation error?`, `polar winter drifts 41% further: its bias flies at
1.40×`, bars for the standard day (41.60 m) and polar winter (58.71 m), `5.7 sigma apart`. Then
`6 worlds × 8 draws = 48 counterfactual descents, one table`, the eight gate lines from
`output.txt`, each `PASS`, and the end card: `DeepCausality CFD`, `Six counterfactual atmospheres,
one table.`, `cfd.deepcausality.com/tutorial/stage-2-weather`, `cargo run --release -p
avionics_examples --example plasma_blackout_weather`, and `the whole campaign: 184.0 s on an Apple
M3 Max laptop · open source, in Rust`.

**Voice.**
> Does the weather move the blackout? It shifts it by 2.5 s and barely changes its length. Does it
> move the navigation error? Polar winter drifts 41% further, because its instrument flies furthest
> from calibration. 48 counterfactual descents, one table, which the landing reads in flight. The
> campaign runs in about 3 minutes on a laptop, open source, in Rust.

## Sources for every number

| On screen or spoken | Value | Kind | Source |
|---|---|---|---|
| Worlds | six; dT and density per world | typed | `WEATHER`, `weather/constants.rs` |
| Draws per world, descents | 8; 48 | typed, computed | `MC_DRAWS`, `weather/constants.rs`; the Monte Carlo line of `output.txt` |
| Flight per world | 850 steps, 85 s | typed | `STEPS` × `DT_FLIGHT`; the baseline line of `output.txt` |
| Alternation lines | one per alternated world | computed | the reference draws' audit logs, `weather/audit/*.draw-0.log` |
| Entry readout | altitude, Mach, GPS, navigation error | computed | `weather_trace.csv`, the standard day's rows |
| Onsets | 12.1 s (dense) to 14.6 s (thin) | computed | `weather_table.csv`, reproduced by `weather_trace.csv` |
| Blackout lengths | 55.0 to 56.1 s | computed | `weather_table.csv` |
| Spreads | onset 2.5 s, length 1.1 s | computed | gate (3), `output.txt` |
| Drift | 41.60 m, 58.71 m, 1.41× | computed | `weather_table.csv`, reproduced by `weather_draws.csv`; gate (4) |
| Window factor | 2% longer, about 4% (1.04×) | computed | longest over shortest blackout, `weather_table.csv` |
| Bias departure | 1% per kelvin; 1.40× at 40 K | typed, computed | `IMU_THERMAL_COEFF_PER_K`, `weather/constants.rs`; `weather_table.csv` |
| Prediction | within 3.4% (worst: cold day, 3.34%) | computed | 41.60 m × departure × (dwell / 55.7 s)², over `weather_table.csv` |
| Draw scatter | 1.71 to 2.93 m, one sd | computed | `weather_table.csv` |
| Separation | 17.11 m, 3.01 m combined sigma, 5.7 | computed | gate (4b), `output.txt` |
| Recovery | worst 0.22 m (0.219 m); ceiling 1.0 m | computed, typed | `weather_draws.csv`; `REACQ_ERR_MAX_M`, gate (5) |
| Gates | 8, all passing | computed | `output.txt` |
| Run time | 184.0 s, about 3 minutes | measured | gate (6), Apple M3 Max |

## Notes on what is not physical

- The vehicle is the shared capsule (`video/cfd/shared/src/three/Capsule.tsx`), the same in every
  cut: a 4.23 m aeroshell from the examples' ballistic bundle.
- The attitude: the flight-path angle is read off the trace, `asin(descent rate / speed)`, with the
  descent rate taken from the altitude lost over each 0.1 s step. The angle of attack is drawn at
  20°, because the examples fly lift as a point-mass L/D.
- The wall temperature uses the run's stagnation heat flux, which the examples compute with the
  RAM-C II nose radius of 0.1524 m, so the glow overstates a 4.23 m heatshield's.
- The curves in scene 4 are each world's reference draw. The bars, the draw columns and every number
  are over all eight draws.
- The connectors from the baseline card in scene 2 are schematic.
