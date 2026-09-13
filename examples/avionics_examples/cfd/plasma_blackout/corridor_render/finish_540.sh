#!/bin/bash
# 540p render -> 2x Lanczos to 1080p -> HUD at native 1080p -> mp4
set -e
cd "$(dirname "$0")"
mkdir -p final_up final_hud
ffmpeg -y -loglevel error -framerate 24 -i final540/c_%04d.png -vf scale=1920:1080:flags=lanczos -start_number 1 final_up/c_%04d.png
python3 hud_composite.py final_up final_hud telemetry.csv
ffmpeg -y -loglevel error -framerate 24 -i final_hud/h_%04d.png -c:v libx264 -crf 17 -pix_fmt yuv420p -movflags +faststart corridor_final_1080p_hud.mp4
ls -la corridor_final_1080p_hud.mp4
