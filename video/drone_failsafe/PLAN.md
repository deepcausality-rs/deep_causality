# Dynamic drone fail-safe: video plan

A 135.5-second explainer with on-screen text and no voice, rendered with Remotion from the tutorial's
own run data, plus two shorter cuts for social posts. It shows two fail-safes on the same drone, one
after the other: the textbook fail-safe of tutorial part 1, then the dynamic fail-safe of part 4,
built on DeepCausality's Effect Ethos. The on-screen text lives in `src/script.json`, and
[SCRIPT.md](SCRIPT.md) mirrors it with the source of every number.

## Decisions

| Question | Decision |
|---|---|
| Tool | Remotion only, under its free licence for an individual |
| Voice | None; on-screen text carries the story |
| Scenarios | Two, one after the other: the textbook fail-safe (part 1), then the dynamic fail-safe (part 4) |
| Story | The problem (the textbook run), the issue (its static rule), the answer (the dynamic run), the evidence (1,000 simulations), how it is built |
| Events | A running log: one line per event, each below the one before |
| World | Static: the power line, the tower and the crew stand still; the drone flies through |
| Colour | The design system's palette, plus `--danger` `#f47174` for people at risk only |
| Format | 1920 × 1080, 30 fps, the standard YouTube format |
| Location | `video/drone_failsafe/` |

## The story

| Scene | Length | What it shows |
|---|---|---|
| Title | 5.5 s | The DeepCausality logo, the title and the question: where should a failing drone land? |
| Setting | 10 s | The valley in 3D with the power line, the tower, the repair crew and the drone on its line. The text names the setting and the three faults ahead. |
| Part 1 card | 5.5 s | Tutorial part 1, the textbook fail-safe: the defaults drone autopilots document today. |
| Textbook | 25 s | The textbook fail-safe flies the night. The log adds a line at each event its trace records. The wind carries the holding drone downhill, and it drops into the creek. |
| The issue | 10 s | A static rule: each fault triggers a fixed action, whatever lies below. |
| Part 4 card | 8.5 s | Tutorial part 4, the dynamic fail-safe: the Effect Ethos checks each landing site against published pilot procedures and the context the drone senses every second. |
| Effect Ethos | 34 s | The dynamic fail-safe flies the same night. Its sensor pyramid, the ground it has judged, the 13 m ring around each person, and the verdicts of each round on the ground; the log names the rule behind each line. It lands upright, 34 m from the crew. |
| 1,000 simulations | 14 s | Part 5 simulated the scenario 1,000 different ways; both fail-safes flew all of them: 35 unsafe landings within 10 m of a person against 1. |
| Summary | 12 s | How DeepCausality builds the dynamic fail-safe: dynamic context, dynamic reasoning, dynamic Effect Ethos. |
| End | 11 s | Build the dynamic fail-safe in Rust, step by step. The DeepCausality logo with the tutorial URL and the command that runs part 4; the Center for Dynamic Causality logo with causalcenter.com. |

| Composition | Scenes | Length |
|---|---|---|
| `Main` | all ten | 135.5 s |
| `Short-Part1` | title, setting, part 1 card, textbook, the issue, end | 67 s |
| `Short-Part2` | title, part 4 card, Effect Ethos, 1,000 simulations, summary, end | 85 s |

## Visual system

- **The world in 3D, standing still.** A block of the valley, cut across the slope at 420 m along
  the line, through the terrace, the tower pad and the crew. The cut face is the cross-section:
  creek, 30° slope, terrace, the step behind it. The block and the camera stay still; the drone flies
  into the block and through it, with its shadow and height.
- **The crew** stand as workers in hard hats and vests. They and the drone are drawn at three times
  their size so they read at this distance; the tower height is set dressing, as the simulation
  models neither.
- **The log** sits at the top left of both runs. A line fades in at its flight second; the newest
  line is bright, the earlier ones dimmed. In the Effect Ethos run a rule line in the accent sits
  under each event. The outcome appears in a box below the log at touchdown.
- **The Effect Ethos run** shows its sensor pyramid down to the footprint, the ground it has judged,
  a 13 m ring around each person, and the verdicts of each round on the patches they judged: ✕
  forbidden, a dashed outline for a patch worth a look, a faint outline for a permitted one, and the
  chosen patch in the accent.
- **The textbook run** shows the wind and the drone's ground track: it holds no map of the ground.
- **Logos.** The title and end screens carry `img/logo_background.jpg`; the end screen also carries
  `img/causal_center_logo_dark.svg`. Both are copied into `public/logos/`.
- **Drawing.** All views are SVG. The 3D view projects the terrain with a pinhole camera and shades it
  with one light; it needs no WebGL and renders the same frame every time.
- **Tokens** from `website/web_design/01-foundations.md`: background `#070b10`, raised surface
  `#0b1118`, hairline `#1f2a36`, text `#e6edf3` and `#aab3bd`, accent `#5cd4e1`, and `--danger`
  `#f47174` for the unsafe landings. Geist for text, JetBrains Mono for eyebrows and numbers, both
  from `website/web/public/fonts/`.

## Data

Every frame draws from traces the Rust programs record (`end_to_end_tutorials/dynamic_drone_failsafe`,
`src/trace.rs`), and from the campaign record of part 5.

| File | Source | Rows |
|---|---|---|
| `world.csv`, `crew.csv` | part 1, from `Terrain::new()` | one per 2 m grid point, −24 to 140 m across and 0 to 520 m along: elevation, slope, surface, canopy; and the six crew positions |
| `part_N_trace.csv` | each part's `main` | one per second: position, height, confirmed faults, command or urgency, judgement below, maneuver, target patch, plan status |
| `part_N_patches.csv` | parts 2 to 4 | one per patch in each second's sensor frame: judgement and fused slope uncertainty |
| `part_N_touchdown.csv` | each part's `main` | where and when the drone came down, its outcome and the nearest person |
| `part_4_rulings.csv` | part 4's `main` | one per proposal of each Effect Ethos round: patch, proposal, verdict, norms, harm cost |
| `campaign_1000.csv` | part 5, committed | the 1000 scenarios and each controller's ending |

`scripts/data.mjs` stops the build when

- a number the video shows disagrees with the tutorial README or its web pages,
- a log line in `src/script.json` sits at a second where its flight's trace records no event,
- any text in `src/script.json` states a number outside the list of numbers the tutorial states.

The video uses the traces of parts 1 and 4; the data step checks all four.

## Build

```bash
# 1. Record the flights (from the workspace root).
for n in 1 2 3 4; do
  cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_$n -- trace video/drone_failsafe/public/traces
done

# 2. Build the video (from video/drone_failsafe).
pnpm install
pnpm data                                         # traces to JSON, checked against the tutorial
pnpm studio                                       # preview
node scripts/render.mjs all                       # out/Main.mp4 and the two shorts
node scripts/render.mjs stills Main 450 2410      # single frames into out/stills/
```

`public/traces/` and `src/data/*.json` are generated and ignored by git, as the repository's
`video/**` rules ignore traces for every video project.
