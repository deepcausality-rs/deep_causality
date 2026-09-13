# Plasma-blackout corridor — Blender render

Animation of `examples/avionics_examples/cfd/plasma_blackout/corridor`: entry and sheath formation, the counterfactual
bank-angle sweep, commit, peak passage and reacquisition. 60 s, 24 fps, 1440 frames.

## Files

| File                             | What                                                                                                                                                                      |
|----------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `corridor_sequence.blend`        | The scene (Blender 5.2, Cycles). Fully keyframed; parked at 50 % / 48 samples, output `//final540/c_`                                                                     |
| `plasma_blackout_corridor.blend` | Asset source (probe, sheath volumes, Earth, the 17 branch curves). The build script appends from it; keep it                                                              |
| `build_corridor_sequence.py`     | Regenerates `corridor_sequence.blend` from the asset file in a few seconds. Run from Blender's Text editor, over MCP, or `blender -b --python build_corridor_sequence.py` |
| `telemetry.csv`                  | Per-frame altitude / plasma level / RF reception, read off the scene's animation curves                                                                                   |
| `hud_composite.py`               | Post-process: telemetry box + phase captions burned onto rendered frames, sized relative to frame height                                                                  |
| `finish_540.sh`                  | 540p frames → Lanczos 2× to 1080p → HUD → `corridor_final_1080p_hud.mp4`                                                                                                  |
| `final540/`                      | The 1440 rendered master frames at 960×540 (≈30 min of GPU time; everything downstream is reproducible from these in minutes)                                             |
| `corridor_final_1080p_hud.mp4`   | The final                                                                                                                                                                 |
| `corridor_preview.mp4`           | 30 % preview with HUD                                                                                                                                                     |
| `corridor_render_prompt.md`      | Physics brief the scene geometry was built from                                                                                                                           |
| `HOW_TO_RENDER.md`               | Notes from the first attempt (partly superseded; still useful for camera/material knobs)                                                                                  |

## Pipeline

```bash
# 1. (optional) rebuild the scene after editing the build script
blender -b --python build_corridor_sequence.py

# 2. render the master frames (≈30 min on the M3 Max)
blender -b corridor_sequence.blend -a          # writes final540/c_0001.png …

# 3. upscale + HUD + encode (a few minutes, run natively — needs ffmpeg and pip3 install pillow)
./finish_540.sh
```

Rendering at 100 % instead of 50 % costs ~100 min for a marginally sharper star field; the volumes, fan and hardware are
visually identical after the Lanczos upscale (tested: 49–52 dB PSNR against a native 1080p frame). Never AI-upscale
frames that already carry the HUD — composite the HUD after any upscaling so the text stays exact.

## Where the numbers come from

Every figure in the captions and telemetry is from the committed `output.txt` of the example:
onset 73.2 km, coarse misses 20.0/11.6/3.5/6.0/14.3/28.9 m, committed 11.5° at 2.07 m vs the 20.0 m ballistic miss, peak
n_e 2.6e19 m⁻³ at 61 km, exit at 47 km, INS drift 0.18 → 1.56 → 2.29 → 0.28 m. Frame constants for the phases are at the
top of both scripts and must agree.
