# Rendering the corridor scene

File: `render_scratch/plasma_blackout_corridor.blend` (Blender 5.2 LTS, Cycles). It is saved in the state it should open
in: **Fan** view layer, **CAM_fan**, 2560×1440, 200 samples.

---

## The 30-second version

1. Open the `.blend`.
2. `F12` for a still, `Ctrl+F12` for the animation.
3. In the render window: `Alt+S` to save the image.

That gives you the fan shot. Everything below is for the other two.

> **`F12` renders the *current* frame, not frame 1.** The file is parked on frame 270 —
> the end of the hold, where all 17 branches have drawn and all 17 terminals have landed —
> so a single `F12` gives the finished picture. If you scrub to frame 1 and press `F12` you
> get an almost-empty frame showing only the fork ring and the aim point. That is correct,
> not a fault: frames 1-12 are the pre-roll, where `bevel_factor_end` is still 0 and nothing
> has been drawn yet. Scrub past frame 13 to see the fan grow.

---

## Why there are two view layers

The 17 branch ribbons all converge on the fork point — which is exactly where the probe is. For the wide shots that is
the picture. For the close-up the bundle passes **straight through the lens**, and the camera ends up looking down the
inside of a glowing white cone. So:

| View layer  | Contains                    | Use with             |
|-------------|-----------------------------|----------------------|
| **Fan**     | everything                  | `CAM_fan`, `CAM_arc` |
| **Closeup** | branches + markers excluded | `CAM_hero`           |

Switch layers in the top-right header dropdown (next to the scene name). Nothing is hidden by hand — it is collection
exclusion, so it survives saving.

## The cameras

| Camera        | Shot                                                                         | View layer  |
|---------------|------------------------------------------------------------------------------|-------------|
| `CAM_fan`     | the 17-branch counterfactual fan, fork bottom-right to terminal arc top-left | Fan         |
| `CAM_arc`     | looking back up the fan from beyond the aim: the reachable arc, end-on       | Fan         |
| `CAM_hero`    | the probe at 73.2 km — nose cap, raceway, antennas, shock rim                | **Closeup** |
| `CAM_profile` | spare, unframed                                                              | Fan         |

To switch: select the camera in the outliner, then `Ctrl+Numpad 0`. Or Scene properties (printer icon) → Camera
dropdown, which does not require selecting it.

## Rendering

* `F12` renders the still. Output goes to the Render Result window — it is **not written to disk** until you `Alt+S` it,
  or set Output properties → Output path and use
  `Render ▸ Render Image` with a file format set.
* The saved output path is `//renders/frame_`, i.e. a `renders/` folder next to the `.blend`.
* `Esc` cancels a render in progress.

### GPU

Render properties → Device. It is set to GPU but Blender silently falls back to CPU if the backend is not configured.
Check Preferences → System → Cycles Render Devices and pick **Metal**, then tick your GPU. On an M-series Mac this is
roughly a 5–10× difference.

### Sample counts and rough cost

| Purpose  | Samples | Resolution |
|----------|---------|------------|
| Look-see | 24–48   | 960×540    |
| Working  | 64–80   | 1600×900   |
| Final    | 200–256 | 2560×1440  |

The three finals took about 60–110 s each at 2560×1440 / 200 samples on this machine. Denoising is on, which is why 200
is enough; without it you would want 800+.

## The knobs worth touching

Everything below is a single value.

| What                      | Where                                                                                     | Current                     |
|---------------------------|-------------------------------------------------------------------------------------------|-----------------------------|
| **Downrange compression** | rebuild-only — see below                                                                  | 500 : 1                     |
| Sheath brightness         | `Sheath_shocklayer` material → the two Multiply nodes feeding Density / Emission Strength | 1.1 / 2.2                   |
| Wall glow                 | `HeatShield_RadEq` → Mix Shader `Fac`, and the Multiply into Emission Strength            | 0.34 / 2.4                  |
| Sun                       | `Sun` light → Strength                                                                    | 4.0 W/m²                    |
| Earthshine fill           | `Earthshine` area light → Power                                                           | 130 kW                      |
| Overall exposure          | Render properties → Color Management → Exposure                                           | 0, AgX Medium High Contrast |

The compression factor is baked into the ribbon curve geometry, so changing it means re-running the ribbon generation
rather than editing a field. The value and the formula are in `corridor_render_prompt.md` §9 — the paths are
`P(t,φ) = ( −V·t/COMP, −R·s²·(1−cos φ), −R·s²·sin φ )`, `s = t/T`, `R = 96.667 m`.

## Animating it

Nothing is keyframed. The cheapest motion is an orbit: select a camera, `N` panel → add a **Track To** constraint aimed
at an empty at the fork, then keyframe the camera's rotation around it. For the fan shot, keyframing the ribbons'
`Bevel → Geometry → End` factor from 0 to 1 grows all 17 branches out of the fork over the shot — that reads as the fan
actually forking.

## Headless

```bash
blender -b render_scratch/plasma_blackout_corridor.blend \
        --python-expr "import bpy; bpy.context.scene.camera = bpy.data.objects['CAM_arc']" \
        -o //renders/arc_ -f 1
```

`-b` is background, `-f 1` renders frame 1. The `--python-expr` is only needed to pick a camera other than the saved
one. View layers are chosen with `--render-layer Closeup`.

## What is in the scene

| Collection    | Contents                                                                                       |
|---------------|------------------------------------------------------------------------------------------------|
| `probe`       | RAM-C II body, nose cap, raceway, 5 stringer bands, 4 flush antennas, Langmuir rake, bolt ring |
| `sheath`      | shock-layer shell (5.66 mm standoff), ionized wake — both volumes                              |
| `branches`    | 17 ribbons + 17 terminal markers + the refused 40° ghost                                       |
| `markers`     | fork ring, aim point                                                                           |
| `environment` | Earth (6371 km sphere, 73.2 km below), Sun, Earthshine                                         |
| `cameras`     | the four cameras                                                                               |

Camera clip end is 2×10⁷ m because the Earth is in the scene at true size. If you add a camera, copy that or the planet
vanishes.

---

# The animation

The scene is now keyframed. `Ctrl+F12` renders the sequence.

## Timing — it is 1:1 with the physics

|          | Frames  | Seconds  | What                                                      |
|----------|---------|----------|-----------------------------------------------------------|
| Pre-roll | 1–12    | 0.5      | the probe at the fork, nothing drawn yet                  |
| Draw     | 13–253  | **10.0** | the ribbons grow — **exactly `BRANCH_STEPS × DT_FLIGHT`** |
| Hold     | 254–277 | 1.0      | terminals pop in, camera settles                          |

24 fps, 277 frames, 11.54 s. The draw phase is real time: one second of screen is one second of branch dwell, so the
rate the fan opens at is the rate the vehicle actually separates the counterfactuals. Do not change `frame_end` without
changing the draw span to match, or that stops being true.

## What is animated

* **`bevel_factor_end` 0 → 1** on all 18 curves (17 branches + the refused 40° ghost), **linear** interpolation. Linear
  because time is linear; easing the draw would imply an acceleration the branches do not have.
* **`bevel_factor_mapping_end = 'RESOLUTION'`**, not `'SPLINE'`. RESOLUTION maps the factor along the spline's evaluated
  points, which are uniform in the path parameter `s`. SPLINE maps by arc length, and arc length is not linear in `s`
  here — the outer branches are longer — so the branches would draw at subtly wrong *relative* rates and the fan would
  open crooked.
* **Terminal markers** scale in at frames 249 → 269 with a small overshoot. All 17 arrive together, because all 17 fly
  the same 10.0 s dwell from the same paused state.
* **`CAM_fan`** pushes slowly from 90 m to 67 m, bezier ease-in-out, aimed at a fixed point.

## What is deliberately NOT animated

The vehicle does not fly down the committed branch. At the fork there is no committed branch — that is the whole point
of the sweep. All 17 worlds fly the *same* paused state, and the commitment is the reduction that happens after they
land. The vehicle sits at the fork and the 17 growing tips are the 17 counterfactual vehicles. Putting one vehicle on
the 11.5° path would assert the answer in frame 1.

## Cost — and how to preview without waiting

Use **Output properties ▸ Format ▸ Resolution %**, not the X/Y fields. It scales the render without touching framing,
aspect, or the camera, so the preview frames the same as the final.

All measured on this machine, Metal GPU, on frame 130 (mid-draw, the heaviest typical frame):

| Resolution %    | Pixels      | Samples | s/frame | All 277      | Every 3rd frame |
|-----------------|-------------|---------|---------|--------------|-----------------|
| 100 % *(saved)* | 1920×1080   | 96      | 5.9     | **27.2 min** | 9.1 min         |
| 50 %            | 960×540     | 48      | 1.9     | 8.8 min      | 2.9 min         |
| **50 %**        | **960×540** | **32**  | **1.6** | **7.4 min**  | 2.5 min         |
| 25 %            | 480×270     | 24      | 1.02    | 4.7 min      | 1.6 min         |
| 25 %            | 480×270     | 16      | 1.02    | 4.7 min      | 1.6 min         |

**50 % / 32 samples is the one to use.** Full motion, under 8 minutes, and at 32 samples the denoiser still leaves the
ribbons clean — there is no visible noise to mistake for a problem.

Note the floor: 25 % at 16 samples is no faster than 25 % at 24. Around 1 s/frame the cost stops being sampling and
becomes fixed per-frame overhead — scene sync, building the volume grids, the denoiser. So dropping samples below ~32
buys nothing, and going below 50 % resolution saves under 3 minutes while making the fine sweep's eleven branches
illegible.

For a pure timing check, add **Output ▸ Frame Step = 3**. It renders every third frame, so the result plays at 8 fps but
the motion and the camera move are all there in 2.5 minutes.

### One-liner

```bash
blender -b render_scratch/plasma_blackout_corridor.blend \
  --python-expr "import bpy; s=bpy.context.scene; s.render.resolution_percentage=50; s.cycles.samples=32" \
  -o //preview/p_ -a
```

`preview_fan.mp4` in `render_scratch/` is exactly this pass, already assembled.

> This Blender build has **no FFMPEG output** — Output ▸ File Format offers only image
> formats. So video has to be assembled from the PNG sequence afterwards (see below);
> `ffmpeg` is already on this machine at `/opt/homebrew/bin/ffmpeg`.

## Getting a video out

Frames land in `render_scratch/renders/fan_0001.png` …. PNG sequence rather than direct video on purpose: a crashed or
interrupted render resumes instead of starting over.

```bash
cd render_scratch/renders
ffmpeg -framerate 24 -i fan_%04d.png \
       -c:v libx264 -crf 16 -pix_fmt yuv420p -movflags +faststart fan.mp4
```

For a talk slide that should loop, append a reversed copy:

```bash
ffmpeg -i fan.mp4 -filter_complex "[0]reverse[r];[0][r]concat=n=2:v=1[v]" -map "[v]" fan_loop.mp4
```

## Headless

```bash
blender -b render_scratch/plasma_blackout_corridor.blend -o //renders/fan_ -a
```

`-a` renders the whole frame range. Add `-s 13 -e 253` to render only the draw phase.
