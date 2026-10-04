# Plasma-blackout weather: video plan

One cut that explains the weather example's dispersion table, rendered with Remotion from the
example's own run data. It follows the corridor cut in `../plasma_blackout/` and the retropropulsion
cut in `../retropropulsion/` in look, structure and checks, and shows the same capsule. The narration
script and shot list live in [SCRIPT.md](SCRIPT.md).

## Decisions

| Question | Decision |
|---|---|
| Tool | Remotion only |
| Look | Hybrid: the 3D capsule for the entry; 2D data graphics for the explanation |
| Format | 3840 × 2160, 30 fps, 3:08 |
| Narration | Captions with numerals, plus the spoken form for timing; an SRT from the same timing |
| Music | The shared theme, `../music/theme.scd`, rendered by SuperCollider at the cut's exact length |
| Vehicle | The corridor cut's capsule, heatshield first |
| Location | `video/cfd/weather/` |

## The idea that carries the video

A flight computer needs to know what the weather does to its descent. The first scene poses two
questions; each is posed again just before its answer, and the close answers both under the same
headings.

1. **Does the weather move the GPS blackout?** It shifts the onset by 2.5 s, densest air first, and
   moves the length by 1.1 s.
2. **Does it move the navigation error?** Polar winter drifts 41% further than the standard day,
   5.7 combined standard deviations apart. The window explains at most 4% of it; the accelerometer
   bias, flown 1.40× its calibration while the filter assumes 1.00, explains the rest, and the
   prediction from those two factors reproduces every world within 3.4%.

## Data

| File | Rows |
|---|---|
| `weather_table.csv` | one per world: density, IMU departure, window, peak flow, drift and terminal statistics |
| `weather_draws.csv` | one per draw: window, largest drift while GPS is lost, terminal error |
| `weather_trace.csv` | each world's reference draw, one per 0.1 s step, with Mach, speed, `n_e` and heat flux |
| `audit/*.draw-0.log` | each world's reference-draw audit log, with its alternation line |
| `output.txt` | the printed table, the 8 gates |

- `pnpm sync` copies them into `public/traces/` (generated, not committed).
- `src/data/weather.ts` parses them and checks every on-screen number against them and against
  `output.txt`; `src/data/captions.ts` checks every caption number. A mismatch fails the render.

## Project layout

```text
video/cfd/weather/
  PLAN.md, SCRIPT.md
  package.json, remotion.config.ts, tsconfig.json
  scripts/sync-traces.mjs   copies the run files and audit logs into public/traces/, the fonts into public/fonts/
  src/
    index.ts, Root.tsx   register Main and one composition per scene
    Main.tsx             the seven scenes joined by fades
    script.ts            narration segments
    timeline.ts          this cut's scene holds, on the shared timing
    data/                the run loader and the number checks
    views/               Hud
    scenes/              Entry, Worlds, Window, Drift, Why, Resolved, Close
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
| `pnpm render` | `out/weather_4k.mp4` |
| `pnpm render:1080` | `out/weather_1080p.mp4`, from the 4K file |
| `pnpm srt` | `out/weather.srt` |
| `pnpm poster` | `out/weather_poster.png` |
