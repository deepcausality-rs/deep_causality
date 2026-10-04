# Plasma retropulsion: video plan

One cut that explains the retropulsion example's two counterfactuals, rendered with Remotion from
the example's own run data. It follows the corridor cut in `../plasma_blackout/` in look, structure
and checks, and shows the same capsule. The narration script and shot list live in
[SCRIPT.md](SCRIPT.md).

## Decisions

| Question | Decision |
|---|---|
| Tool | Remotion only |
| Look | Hybrid: the 3D capsule, plume and ground; 2D data graphics for the explanation |
| Format | 3840 × 2160, 30 fps, 3:59 |
| Narration | Captions with numerals, plus the spoken form for timing; an SRT from the same timing |
| Music | The shared theme, `../music/theme.scd`, rendered by SuperCollider at the cut's exact length |
| Vehicle | The corridor cut's capsule, heatshield first, with the central retro nozzle lit for the burn |
| Location | `video/cfd/retropulsion/` |

## The idea that carries the video

The capsule has to land under its own engine, and two questions shape that landing. The first
scene poses both; each is posed again just before its answer, and the close answers both under the
same headings.

1. **How hard to burn at supersonic speed?** Mid-burn, the plume-coupled state is paused and forked
   five ways. At 0.20 throttle the capsule decelerates at 7.47 m/s² against 10.59 m/s² coasting:
   the plume destroys drag about as fast as thrust replaces it.
2. **When to light the landing burn?** The guidance's margin comes from the weather table. Flown
   twice from one state, sized for today and for a standard day, the landing lights its burn
   14.06 m apart, and the extra margin costs 7.28 kg of propellant.

## Data

| File | Rows |
|---|---|
| `retropulsion_trace.csv` | the informed descent, one per 0.1 s step across four legs, with speed, `n_e` and heat flux |
| `retropulsion_branch_trace.csv` | the five throttle branches after the fork, 120 steps each |
| `retropulsion_uninformed_trace.csv` | the landing leg flown with the standard-day margin |
| `retropulsion_branches.csv` | the roster table |
| `output.txt` | the act lines, the roster, the belief table, the 16 gates |
| `../weather/weather_table.csv` | the dispersion table the run reads in flight |

- `pnpm sync` copies them into `public/traces/` (generated, not committed).
- `src/data/retro.ts` parses them and checks every on-screen number against them and against
  `output.txt`; `src/data/captions.ts` checks every caption number. A mismatch fails the render.

## Project layout

```text
video/cfd/retropulsion/
  PLAN.md, SCRIPT.md
  package.json, remotion.config.ts, tsconfig.json
  scripts/sync-traces.mjs   copies the run files and the weather table into public/traces/, the fonts into public/fonts/
  src/
    index.ts, Root.tsx   register Main and one composition per scene
    Main.tsx             the eight scenes joined by fades
    script.ts            narration segments
    timeline.ts          this cut's scene holds, on the shared timing
    data/                the trace loader and the number checks
    views/               Hud
    scenes/              Entry, Coast, Pause, Fork, Plan, Landing, Beliefs, Close
```

The cuts are one pnpm workspace rooted at `video/cfd/`. What they share lives in `video/cfd/shared/`
(the package `@cfd-video/shared`): the design tokens, the stage and its 2D views (backdrop, text,
captions, strips, readout, questions, flight timeline), the capsule the examples fly and the 3D shot
around it, the narration timing, the music player, and the SRT and music scripts. `pnpm install`
runs from `video/cfd/`.


## Commands

| Command | Output |
|---|---|
| `pnpm studio` | the Remotion studio |
| `pnpm music` | `public/music/theme.wav`, the theme at the cut's length |
| `pnpm render` | `out/retropulsion_4k.mp4` |
| `pnpm render:1080` | `out/retropulsion_1080p.mp4`, from the 4K file |
| `pnpm srt` | `out/retropulsion.srt` |
| `pnpm poster` | `out/retropulsion_poster.png` |
