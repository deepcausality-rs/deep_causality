# Dynamic drone fail-safe: video plan

One main cut for YouTube and short clips for social media, rendered with Remotion from the
tutorial's own run data. The narration script and shot list live in [SCRIPT.md](SCRIPT.md).

## Decisions

| Question | Decision |
|---|---|
| Tool | Remotion only, under its free licence for an individual |
| Narration | A synthetic voice from an open-source voice tool, with captions |
| Colour | The design system's palette, plus `--danger` `#f47174` for people-at-risk moments only |
| Location | `video/drone_failsafe/` |
| Output | One main cut and seven short clips, all in the standard YouTube format, 16:9 |

The mock frame [mockup/run4_85s_mixed.png](mockup/run4_85s_mixed.png) shows the approved layout:
run 4 at 85 s, with its map, Ethos round and prices from the part 4 output in the tutorial README.

## The idea that carries the video

The same drone flies the same night four times. Camera, terrain, wind and fault times stay fixed:
the fix fails at 55 s, the link at 65 s, a cell at 85 s. After the first run the viewer knows when
each fault arrives and watches only what changes: what the drone knows, what it does, and where it
comes down. Each run ends on a frozen outcome card.

Two moments carry the argument:

- **Run 3 puts a person at risk.** The drone acts on its map and lands 3 m from the crew. Across
  the campaign, context and action alone put people at risk on 41 nights, the textbook ladder on 35.
- **The first safety layer became the hazard.** It forbade every landing, and the drone hovered
  until its battery died and fell beside the crew.

## Deliverables

| Composition | Size | Length | Captions | For |
|---|---|---|---|---|
| `Main` | 1920 × 1080, 30 fps | about 3:45 | SRT file | YouTube |
| `Clip-*` | 1920 × 1080, 30 fps | 25–45 s | burned in, word by word | social media posts |

The seven clips are listed in [SCRIPT.md](SCRIPT.md#clips). Each clip opens on its strongest
frame with a one-line hook, plays one scene of the main cut, and ends on the tutorial URL. Feeds
autoplay muted, so every clip reads with the sound off.

## Visual system

- **3D on the left: the world, cut open at the drone.** A block of the valley, sliced across the
  slope at the drone's position along the line. The cut face is the cross-section: creek, 30°
  slope, terrace, the step behind it, in true profile. The top surface carries the drone with its
  shadow, sensor cone and height above ground, the tower and power line, the crew with a 13 m
  keep-out ring, and wind arrows downslope. The cut moves with the drone, and the camera tracks it.
- **2D on the right: what the drone knows and decides.** A top view of the 4 m patch map with the
  sensor footprint, and below it the panel of the part: telemetry and confirmed faults in run 1,
  the Effect Ethos round and the last-resort prices in run 4. In run 1 the map stays empty: the
  drone knows nothing about the ground.
- **Drawing.** Both views are SVG. The 3D view projects the terrain with a pinhole camera and
  shades it with one light, as in the mock; it needs no WebGL and renders the same frame every
  time. The drone and the crew figures are drawn at about three times scale so they read; the
  tower height is set dressing, as the simulation models neither.
- **Truth and belief.** Sensed ground carries the 4 m patch grid and the tone of its judgement.
  Unseen ground stays dark with a sparse wireframe, so the map grows as the drone looks.
- **The program's glyphs.** Patch kinds in the top view use the glyphs the terminal map prints:
  `.` safe, `/` steep, `~` water, `T` trees, `P` person, `?` unsure. They are drawn as dots,
  hatching, waves and rings, so kind reads by shape as well as tone.
- **A shared timeline** along the bottom, with fault markers at 50, 55, 65 and 85 s.
- **Verdicts on the map** in run 4. Each candidate patch flashes with the norm that rules on it,
  and the chosen patch draws in.
- **Tokens** from `website/web_design/01-foundations.md`: background `#070b10`, raised surface
  `#0b1118`, hairline `#1f2a36`, text `#e6edf3` and `#aab3bd`, accent `#5cd4e1`. `--danger`
  `#f47174` marks a person at risk and appears nowhere else. Geist for text, JetBrains Mono for
  eyebrows and numbers, both from `website/web/public/fonts/`.
- **Sound.** A low wind bed, a tick when the cell fails, silence on each outcome card. Music, if
  any, comes from a CC0 source.

## Data

Every frame draws from traces the Rust programs record. This follows the plasma-blackout pipeline
(`examples/avionics_examples/src/shared/trace.rs`).

| File | Source | Rows |
|---|---|---|
| `world.csv`, `crew.csv` | part 1, from `Terrain::new()` | one per 2 m grid point, −24 to 140 m across and 0 to 520 m along: elevation, slope, surface, canopy; and the six crew positions |
| `part_N_trace.csv`, N = 1–4 | each part's `main` | one per second: position, height, confirmed faults, command or urgency, judgement below, maneuver, target patch, plan status |
| `part_N_patches.csv`, N = 2–4 | each part's `main` | one per patch in each second's sensor frame: judgement and fused slope uncertainty |
| `part_N_touchdown.csv`, N = 1–4 | each part's `main` | where and when the drone came down, its outcome and the nearest person |
| `part_2_daytime.csv` | part 2's `main` | the water patches, and how many the daytime rule still reads as water |
| `part_4_rulings.csv` | part 4's `main` | one per proposal of each Effect Ethos round: patch, proposal, verdict, norms, harm cost |
| `part_4_last_resort.csv` | part 4's `main` | the three last-resort ditchings priced at the emergency |
| `campaign_1000.csv` | part 5, committed | the 1000 scenarios and each controller's ending |

- Recording runs behind a `trace <dir>` argument, so a plain run prints and writes exactly what
  it does without one.
- The twist scene states the campaign's first finding as the README records it: every person at
  risk in that run, 3.9 % of 1000 nights, was a fall after the norms forbade every landing. The
  first part 4 commit, `e701f0f58`, predates today's world: it imports a constant the library no
  longer exports, flies the older drone API, and judges an 18 m canopy as a person. Its flight
  through today's scenarios would be a different experiment from the one that found the 3.9 %, so
  the scene shows the finding and draws no flight for it.
- Each number on screen carries its source: **typed** (a constant), **computed** (one run's
  output) or **sampled** (the campaign). The Remotion data module checks every on-screen number
  against the traces and the README, and the render fails on a mismatch.

## Voice-over and captions

- `narration.json` holds the narration as segments, one per scene and one per clip hook line,
  each a list of sentences. It is the single source for the audio file names, the captions and
  the scene lengths.
- `scripts/voice.mjs` synthesizes every sentence into its own WAV file in `public/vo/` and records
  each file's duration. Any voice tool that writes one WAV per sentence fits; the script calls it
  through one function. Until you pick the open-source tool, it uses the macOS `say` voice as a
  placeholder.
- One file per sentence gives each caption its exact start and end, so the burned-in captions of
  the clips and the SRT of the main cut come from the same timings.
- Scene lengths follow the audio: a re-synthesized sentence re-times its scene.
- The clips reuse the main cut's sentences plus their own hook and closing lines, so the voice
  stays consistent.

## Project layout

```text
video/drone_failsafe/
  package.json, pnpm-workspace.yaml, remotion.config.ts, tsconfig.json
  narration.json    narration segments, one sentence per audio file
  scripts/
    data.mjs        reads the traces, checks every on-screen number, writes src/data/*.json
    voice.mjs       synthesizes each sentence and records its duration
  public/
    fonts/          Geist and JetBrains Mono, copied from website/web/public/fonts/
    traces/         the CSV files above
    vo/             one WAV per sentence
  src/
    Root.tsx        registers Main and the Clip-* compositions
    tokens.ts       palette, type scale
    data/           the generated JSON and its types
    world/          projection, terrain, the cut block, drone, tower, crew
    views/          WorldView, TopView, panels, Timeline, OutcomeCard, ScenarioGrid, Captions
    scenes/         Hook, World, Run (one per part), Twist, Campaign, Close
```

All pnpm settings go in `pnpm-workspace.yaml`. The project sits outside Cargo and Bazel.

## Steps

| Step | Work | Check |
|---|---|---|
| 1 | Review [SCRIPT.md](SCRIPT.md) | You approve the narration and the shot list |
| 2 | Trace recording in parts 1–5 behind `trace` | Parts 1–4 print byte-identical output with and without the recorder; `make check_examples` passes |
| 3 | The twist scene from the README's recorded finding | The 3.9 % and its wording match the README |
| 4 | Remotion scaffold, tokens, the cut-block world view | It matches the approved mock frame |
| 5 | All scenes of the main cut, with the placeholder voice | Every on-screen number passes the data check |
| 6 | Voice-over from your open-source tool, one WAV per sentence | Scene lengths and captions follow the audio files |
| 7 | Clip compositions | You review one clip |
| 8 | Final renders, SRT, thumbnails | Each render's numbers trace to the README or a trace file |
| 9 | Links from the tutorial pages | A plain link with a static poster image; `web/DESIGN.md` §13 bans auto-playing video |

You upload to YouTube and the social platforms; the tutorial pages then link the published video.

## Build

```bash
# 1. Record the four flights (from the workspace root).
for n in 1 2 3 4; do
  cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_$n -- trace video/drone_failsafe/public/traces
done

# 2. Build the video (from video/drone_failsafe).
pnpm install
pnpm data        # traces to JSON; stops on any number that disagrees with the README
pnpm voice       # one WAV per sentence; VOICE_CMD='<tool> ... {out}' selects your voice tool
pnpm timeline    # scene lengths from the audio, and out/main.srt
pnpm studio      # preview
node scripts/render.mjs all                    # out/Main.mp4 and out/Clip*.mp4
node scripts/render.mjs stills Main 5275 5510  # single frames into out/stills/
```

`public/traces/`, `public/vo/` and `src/data/*.json` are generated and ignored by git, as the
repository's `video/**` rules ignore traces for every video project: step 1 rebuilds the traces,
step 2 everything else.
