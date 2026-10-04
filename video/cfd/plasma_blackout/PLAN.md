# Plasma-blackout corridor: video plan

One cut that explains a counterfactual fork, rendered with Remotion from the corridor example's own
run data. It replaces the Blender render in `../renders/plasma_blackout/corridor_render/`, which
looked basic, rendered at low fidelity and explained nothing. The narration script and shot list
live in [SCRIPT.md](SCRIPT.md).

## Decisions

| Question | Decision |
|---|---|
| Tool | Remotion only |
| Look | Hybrid: a 3D vehicle, sheath and Earth limb for the descent; 2D data graphics for the explanation |
| Format | 3840 × 2160, 30 fps, 3:06 |
| Narration | On-screen text plus a voice-over script; captions from the same segments |
| Music | The shared theme, `../music/theme.scd`, rendered by SuperCollider at the cut's exact length |
| Colour | The design system's palette; `--warn` `#e3b341` marks the GPS L1 band and the blackout only |
| Location | `video/cfd/plasma_blackout/` |
| Scope | The corridor first; weather and retropulsion follow the same template once the look is approved |

## The idea that carries the video

The Blender cut showed seventeen ribbons and never said what they shared. This cut states it, and
closes on one sentence:

> With GPS gone, flying uncorrected would miss the aim by 20 m. Counterfactuals from the last known
> state cut that to 2.07 m, inside the 5.6 m the navigation filter reports as its own uncertainty.

The fork pauses the one real descent the moment GPS is lost, when the state is best known, and
continues seventeen counterfactual trajectories from it: the same flow, the same plasma, the same
navigation error. Everything after that answers one question: which bank angle is best from where
the vehicle actually is.

Three moments carry it:

- **The pause.** Time visibly stops at 13.1 s and 73.2 km. The state the branches inherit is
  listed on screen, read off the trace.
- **The arc.** The seventeen end points fall on one arc set by the bank angle, the aim sits just
  off it, and 11.5° lands 2.07 m away against 20.0 m with no bank.
- **The result.** At one scale: 20.0 m uncorrected, 2.07 m committed, inside the filter's 5.6 m.

## Visual system

- **Two registers.** 3D (React Three Fiber through `@remotion/three`) for the vehicle in flight:
  the examples' capsule, heatshield first and tilted for lift, the heatshield glowing hottest where
  the flow stagnates, a shock layer bright at the silhouette and nearly clear across the face, the
  Earth limb below. 2D vector graphics for everything that is measured: traces, branches, the arc,
  the gates.
- **One vehicle across the three parts.** `video/cfd/shared/src/three/Capsule.tsx` is the vehicle the corridor, the
  weather campaign and the retropulsion descent fly: a 4.23 m aeroshell, from the examples'
  ballistic bundle, with the central retro nozzle the retropulsion part lights.
- **Data, not keyframes.** Every line in the 2D register is a trace row. The 3D register is driven
  by the same rows: altitude sets the camera's Earth limb, electron density sets the sheath glow.
- **A shared timeline** along the top from entry to the end of the run, with GPS loss and the
  pause marked. It stays on screen through the flight scenes so the viewer always knows where in
  the descent they are.
- **Tokens** from `website/web_design/01-foundations.md`: background `#070b10`, raised surface
  `#0b1118`, hairline `#1f2a36`, text `#e6edf3` and `#aab3bd`, accent `#5cd4e1` for the committed
  branch and the shared past. Geist for text, JetBrains Mono for numbers, both from
  `website/web/public/fonts/`.
- **Physics in the 3D register.** The wall temperature follows radiative equilibrium from the
  run's stagnation heat flux, with Lees' `cos θ` distribution over the heatshield; the sheath glow
  follows the electron density; the shock layer is a skin and not a halo. WebGL renders it in real
  time; it is shaded, not path-traced.

## Data

Every frame draws from files the corridor example commits:

| File | Rows |
|---|---|
| `corridor_trace.csv` | one per 0.1 s step of the flown descent, 720 rows, labelled by leg |
| `corridor_branch_trace.csv` | the 17 branches after the pause, 100 steps each |
| `corridor_branches.csv` | the fine round's scored table |
| `output.txt` | leg snapshots, the provenance log, the 13 gates |

- `pnpm sync` copies them into `public/traces/` (generated, not committed).
- `src/data/corridor.ts` parses them and checks every on-screen number against them and against
  `output.txt`. A mismatch fails the render, as on the website.
- Each number on screen carries its source: **typed** (a constant), **computed** (the run's output)
  or **derived** (computed here from the run's output, such as the arc fit). The SCRIPT.md source
  table lists them.

## Voice-over and captions

- `src/script.ts` holds the narration as segments, one per scene: the single source for the audio
  file names, the captions and the scene lengths.
- `src/timeline.ts` sizes each scene from its segment's word count at 150 words a minute. Once a
  voice-over exists, each segment is recorded as its own file in `public/vo/` and the timing reads
  the file durations instead, so a re-recorded line re-times the cut.
- Captions come from the same timing: `pnpm srt` writes `out/plasma_blackout_corridor.srt`.

## Project layout

```text
video/cfd/plasma_blackout/
  PLAN.md, SCRIPT.md
  package.json, remotion.config.ts, tsconfig.json
  scripts/sync-traces.mjs   copies the committed run files into public/traces/, the fonts into public/fonts/
  public/
    fonts/          generated by pnpm sync, from website/web/public/fonts/
    traces/         generated by pnpm sync
    music/          generated by pnpm music, from ../music/theme.scd
    vo/             one audio file per narration segment, once recorded
  src/
    index.ts, Root.tsx   register Main and one composition per scene
    Main.tsx             the six scenes joined by fades
    script.ts            narration segments
    timeline.ts          this cut's scene holds, on the shared timing
    data/                the trace loader and the number checks, on the graphics and the captions
    views/               Timeline, BranchFan, EndArc
    scenes/              Problem, Question, Pause, Fork, FlyThrough, Close
```

The cuts are one pnpm workspace rooted at `video/cfd/`. What they share lives in `video/cfd/shared/`
(the package `@cfd-video/shared`): the design tokens, the stage and its 2D views (backdrop, text,
captions, strips, readout, questions, flight timeline), the capsule the examples fly and the 3D shot
around it, the narration timing, the music player, and the SRT and music scripts. `pnpm install`
runs from `video/cfd/`.

All pnpm settings go in `video/cfd/pnpm-workspace.yaml`. The project sits outside Cargo and Bazel;
`.bazelignore` and `.gitignore` carry its `node_modules`.

## Steps

| Step | Work | Check |
|---|---|---|
| 1 | Review [SCRIPT.md](SCRIPT.md) | You approve the narration and the shot list |
| 2 | Scaffold, tokens, data checks, and the fork scene as a look test | You review the look before the other scenes are built |
| 3 | All scenes of the main cut, with timing from word counts | Every on-screen number passes the data check |
| 4 | Voice-over recorded per segment | Scene lengths follow the recordings |
| 5 | Final 4K render, 1080p derivative, SRT, poster frame | Each number traces to a trace file or `output.txt` |
| 6 | A link from walk 1 | A plain link with a poster image; the site does not auto-play video |

## Open questions

1. **Voice.** Your own recording, or a synthetic voice?
2. **License.** Remotion is free for individuals and companies of up to three people; a larger
   organisation needs a company license. Confirm which applies to the account that renders it.
