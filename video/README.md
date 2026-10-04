# Videos

Explainer videos for the tutorials, rendered with [Remotion](https://www.remotion.dev) from the
examples' own run data. Each project reads the files a Rust example writes, checks every number. The projects sit outside Cargo and Bazel.

## Contents

| Folder | Contents |
|---|---|
| `cfd/` | A pnpm workspace with the three plasma-blackout cuts and the code they share |
| `cfd/shared/` | `@cfd-video/shared`: design tokens, the 2D stage and its views, the end card with both logos, the 3D capsule and shot, narration timing, the music player and check, the SRT and music scripts |
| `cfd/music/` | `theme.scd`, the theme under the three CFD cuts, written in SuperCollider |
| `cfd/plasma_blackout/` | Part 1, the corridor: seventeen counterfactual trajectories through the GPS blackout. 3:06 |
| `cfd/weather/` | Part 2, the weather table: six counterfactual atmospheres. 3:08 |
| `cfd/retropropulsion/` | Part 3, the landing: five counterfactual burns and two counterfactual landings. 3:59 |
| `drone_failsafe/` | The dynamic drone fail-safe tutorial: one main cut and seven clips, narrated in on-screen text |

Each project holds a `PLAN.md` (decisions, data, layout) and a `SCRIPT.md` (narration and shot
list).

## Requirements

- Node.js and pnpm 11.9.0, the version each `package.json` pins in `packageManager`.
- ffmpeg, for the 1080p downscale of the CFD cuts and the music's loudness normalization.
- SuperCollider, for the CFD cuts' music: `brew install --cask supercollider`. `SCLANG` overrides
  the path to its interpreter, `/Applications/SuperCollider.app/Contents/MacOS/sclang`.
- Remotion downloads its own headless Chrome on the first render.

## CFD cuts

Install once, from `video/cfd/`:

```bash
cd video/cfd
pnpm install
```

Then run each command from the cut's own folder, for example `video/cfd/weather/`:

| Command | Output |
|---|---|
| `pnpm music` | `public/music/theme.wav`, the theme at the cut's exact length; run it before the first render and after any change to the cut's timing or to `theme.scd` |
| `pnpm render` | `out/<name>_4k.mp4`, 3840 × 2160 at 30 fps |
| `pnpm render:1080` | `out/<name>_1080p.mp4`, downscaled from the 4K file, so run `pnpm render` first |
| `pnpm srt` | `out/<name>.srt`, the captions |
| `pnpm poster` | `out/<name>_poster.png`, one frame at 1920 × 1080 |
| `pnpm studio` | the Remotion studio, for preview |
| `pnpm check` | a typecheck |
| `pnpm still <composition> out/<file>.png --frame=<n>` | one frame; the compositions are `Main` and `Scene-<id>` |

| Cut | Folder | `<name>` |
|---|---|---|
| Part 1, corridor | `video/cfd/plasma_blackout/` | `plasma_blackout_corridor` |
| Part 2, weather | `video/cfd/weather/` | `weather` |
| Part 3, retropropulsion | `video/cfd/retropropulsion/` | `retropropulsion` |

So the finished weather video lands in `video/cfd/weather/out/weather_4k.mp4` and
`video/cfd/weather/out/weather_1080p.mp4`.

Every command that renders first runs `pnpm sync`, which copies the example's committed run files
into `public/traces/`, the fonts from `website/web/public/fonts/` into `public/fonts/`, and the end
card's logos from `img/` into `public/logos/`. The retropropulsion cut also copies the weather
example's `weather_table.csv`, which that run reads in flight. The main composition fails to render
when `public/music/theme.wav` is missing or does not last exactly as long as the cut. To render from
fresh data, re-run the example from the workspace root first:

```bash
cargo run --release -p avionics_examples --example plasma_blackout_corridor
cargo run --release -p avionics_examples --example plasma_blackout_weather
cargo run --release -p avionics_examples --example plasma_blackout_retropropulsion
```

## Drone fail-safe

The traces come from the tutorial's four flights, recorded behind a `trace <dir>` argument. From
the workspace root:

```bash
for n in 1 2 3 4; do
  cargo run --release -p dynamic_drone_failsafe --example drone_failsafe_part_$n -- trace video/drone_failsafe/public/traces
done
```

Then from `video/drone_failsafe/`:

| Command | Output |
|---|---|
| `pnpm install` | the dependencies |
| `pnpm data` | `src/data/*.json` from the traces; stops on any number that disagrees with the tutorial README or its web pages |
| `pnpm timeline` | the scene lengths from the narration text, and `out/main.srt` |
| `pnpm prepare-video` | `data` and `timeline` in one step |
| `pnpm studio` | the Remotion studio, for preview |
| `pnpm render all` | `out/Main.mp4` and the clips `out/Clip0-Teaser.mp4` to `out/Clip6-Campaign.mp4`, 1920 × 1080 |
| `pnpm render video <composition>` | `out/<composition>.mp4` |
| `pnpm render stills <composition> <frame> ...` | `out/stills/<composition>_<frame>.png` |

The main cut and every clip show the narration as on-screen text, without a voice. `out/main.srt`
carries the main cut's text as a caption file.

## Generated files

Git ignores every `out/` and `public/traces/` folder, the CFD cuts' `public/fonts/`, `public/logos/`
and `public/music/`, and the drone project's `src/data/*.json`. Each is rebuilt by the commands
above.
