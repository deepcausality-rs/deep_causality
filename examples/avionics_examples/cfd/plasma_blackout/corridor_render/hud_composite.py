#!/usr/bin/env python3
"""Burn a telemetry HUD (altitude, RF reception, plasma sheath) onto rendered frames.

    python3 hud_composite.py <frames_dir> <out_dir> <telemetry.csv> [first last]

Pure post-process: the render is untouched. Sizes are fractions of the frame height, so the
same script serves the 30% preview and the 1080p final.
"""
import csv
import os
import sys
from concurrent.futures import ProcessPoolExecutor
from PIL import Image, ImageDraw, ImageFont

FONT_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
FONT_REG = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
if not os.path.exists(FONT_BOLD):
    FONT_BOLD = FONT_REG = "/System/Library/Fonts/Supplemental/Arial Bold.ttf"

PLASMA_FULL = 1.35   # keyed level at the 61 km peak -> 100 %

def load_telemetry(path):
    t = {}
    with open(path) as fh:
        for r in csv.DictReader(fh):
            t[int(r["frame"])] = (float(r["alt_km"]), float(r["plasma"]), float(r["rf"]))
    return t

def draw_hud(im, alt, plasma, rf):
    W, H = im.size
    u = H / 324.0                       # 1.0 at the 30 % preview, 3.33 at 1080p
    pad = int(8 * u)
    bw, bh = int(0.30 * W), int(0.26 * H)
    x0, y0 = pad, pad                    # top-left: keeps the vehicle and the ground clear
    x1, y1 = pad + bw, pad + bh
    ov = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    d.rounded_rectangle((x0, y0, x1, y1), radius=int(6 * u), fill=(8, 12, 20, 200), outline=(255, 255, 255, 70), width=max(1, int(1.2 * u)))
    f_lab = ImageFont.truetype(FONT_REG, int(8.5 * u))
    f_val = ImageFont.truetype(FONT_BOLD, int(11.5 * u))
    f_stat = ImageFont.truetype(FONT_BOLD, int(8.5 * u))
    lx = x0 + int(7 * u)
    row_h = (bh - int(10 * u)) / 3.0
    bar_x0 = x0 + int(0.50 * bw)
    bar_x1 = x1 - int(7 * u)
    bar_h = int(6.5 * u)

    def row(i, label, value_txt, bar=None, bar_col=None, status=None, status_col=None):
        cy = y0 + int(6 * u) + row_h * i
        d.text((lx, cy), label, font=f_lab, fill=(190, 200, 215, 255))
        vy = cy + int(10 * u)
        d.text((lx, vy), value_txt, font=f_val, fill=(255, 255, 255, 255))
        if bar is not None:
            by0 = vy + int(4 * u)
            d.rounded_rectangle((bar_x0, by0, bar_x1, by0 + bar_h), radius=int(3 * u), fill=(255, 255, 255, 40))
            bx = bar_x0 + int((bar_x1 - bar_x0) * max(0.0, min(1.0, bar)))
            if bx > bar_x0 + 1:
                d.rounded_rectangle((bar_x0, by0, bx, by0 + bar_h), radius=int(3 * u), fill=bar_col)
            if status:
                sw = d.textlength(status, font=f_stat)
                d.text((bar_x1 - sw, cy - int(1 * u)), status, font=f_stat, fill=status_col)

    row(0, "ALTITUDE", f"{alt:5.1f} km")
    # RF bar colour: green when clear, amber when degrading, red when denied
    if rf > 0.66: rc = (90, 220, 120, 255)
    elif rf > 0.0: rc = (250, 190, 60, 255)
    else: rc = (240, 70, 60, 255)
    status = "GNSS DENIED" if rf <= 0.0 else ("GNSS LINK" if rf >= 0.99 else "DEGRADING")
    scol = (240, 90, 80, 255) if rf <= 0.0 else ((120, 230, 140, 255) if rf >= 0.99 else (250, 190, 60, 255))
    row(1, "RF RECEPTION (L1)", f"{int(round(rf * 100)):3d} %", bar=rf, bar_col=rc, status=status, status_col=scol)
    p = max(0.0, min(1.0, plasma / PLASMA_FULL))
    row(2, "PLASMA SHEATH", f"{int(round(p * 100)):3d} %", bar=p, bar_col=(255, 130, 50, 255))
    return ov, (x0, y1, bw, u)

# ---------------------------------------------------------------- phase captions (frame ranges follow build_corridor_sequence.py)
F_ONSET, F_CUT2 = 300, 337
F_COARSE0, F_COARSE1 = 360, 540
F_FINE0, F_FINE1 = 570, 730
F_COMMIT, F_PUSH1 = 760, 1000
F_PEAK, F_EXIT, F_END = 1110, 1380, 1440

def phase(f):
    """-> (title, detail, progress or None, colour) for frame f, or None."""
    if F_ONSET <= f < F_CUT2:
        return ("BLACKOUT ONSET  ·  73.2 km", "Electron density crossed the GPS L1 cutoff: the sheath now reflects the link. "
                "Navigation falls back to inertial dead-reckoning.", None, (240, 90, 80))
    if F_CUT2 <= f < F_COARSE0:
        return ("GNSS DENIED  ·  COUNTERFACTUAL GUIDANCE", "The paused state is forked into parallel worlds, one per bank command. "
                "Each is scored by its miss to the aim point.", None, (120, 190, 255))
    if F_COARSE0 <= f < F_FINE0:
        pr = min(1.0, (f - F_COARSE0) / (F_COARSE1 - F_COARSE0))
        det = "6 bank commands: 0°, 5°, 10°, 15°, 20°, 40°.  The 40° command breaches the safety envelope and is clamped (red ghost)."
        if f >= F_COARSE1: det = "Misses 3.5 – 28.9 m.  Coarse winner: 10° at 3.5 m."
        return ("COUNTERFACTUAL SWEEP  ·  ROUND 1 / 2  (coarse)", det, pr, (120, 190, 255))
    if F_FINE0 <= f < F_COMMIT:
        pr = min(1.0, (f - F_FINE0) / (F_FINE1 - F_FINE0))
        det = "11 candidates at 0.5° steps, 7.5° – 12.5°, around the coarse winner."
        if f >= F_FINE1: det = "Minimum at 11.5°: 2.07 m — equal to the INS drift, the vehicle's knowledge floor."
        return ("COUNTERFACTUAL SWEEP  ·  ROUND 2 / 2  (fine)", det, pr, (120, 190, 255))
    if F_COMMIT <= f < F_PUSH1:
        return ("BEST TRAJECTORY SELECTED  ·  BANK 11.5°", "Miss 2.07 m  vs  20.0 m uncorrected: 9.7× better.  "
                "17 worlds flown, one committed.", None, (255, 200, 90))
    if F_PEAK - 40 <= f < F_PEAK + 90:
        return ("PEAK PASSAGE  ·  61 km  ·  Mach 25", "Peak electron density 2.6e19 m⁻³ from uncalibrated finite-rate chemistry, inside the "
                "RAM-C II flight band (1e19).  INS drift has grown to 1.6 m.", None, (255, 150, 70))
    if F_EXIT - 25 <= f <= F_END:
        return ("SHEATH CLEARS  ·  47 km  ·  GNSS REACQUIRED", "Drag slowed the vehicle below the ionization threshold; recombination drains the "
                "sheath.  First folded fix collapses the INS drift from 2.3 m to 0.28 m.", None, (120, 230, 140))
    return None

def fade(f):
    """Soft edges: alpha ramps over 8 frames at each phase boundary."""
    edges = [F_ONSET, F_CUT2, F_COARSE0, F_COARSE1, F_FINE0, F_FINE1, F_COMMIT, F_PUSH1, F_PEAK - 40, F_PEAK + 90, F_EXIT - 25]
    a = 1.0
    for e in edges:
        d = abs(f - e)
        if d < 8: a = min(a, (d + 1) / 8.0)
    return a

def wrap(d, text, font, width):
    words, lines, cur = text.split(), [], ""
    for w in words:
        t = (cur + " " + w).strip()
        if d.textlength(t, font=font) <= width: cur = t
        else:
            lines.append(cur)
            cur = w
    if cur: lines.append(cur)
    return lines

def draw_caption(ov, anchor, f):
    ph = phase(f)
    if ph is None: return
    title, detail, prog, col = ph
    x0, y_top, bw_tele, u = anchor
    W = ov.size[0]
    alpha = fade(f)
    bw = int(0.40 * W) if F_CUT2 <= f < F_PUSH1 else bw_tele   # wide only in the far fan cut; same column as telemetry otherwise
    pad_in = int(7 * u)
    f_d = ImageFont.truetype(FONT_REG, int(8.5 * u))
    d = ImageDraw.Draw(ov)
    # title: shrink until it fits the box (font metrics differ between sizes)
    ts = 9.5
    while True:
        f_t = ImageFont.truetype(FONT_BOLD, int(ts * u))
        if d.textlength(title, font=f_t) <= bw - 2 * pad_in or ts <= 6.5: break
        ts -= 0.25
    lines = wrap(d, detail, f_d, bw - 2 * pad_in)
    lh = int(11 * u)
    bh = pad_in + int(13 * u) + len(lines) * lh + (int(14 * u) if prog is not None else 0) + pad_in
    y0 = y_top + int(6 * u)
    layer = Image.new("RGBA", ov.size, (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.rounded_rectangle((x0, y0, x0 + bw, y0 + bh), radius=int(6 * u), fill=(8, 12, 20, 200), outline=(*col, 110), width=max(1, int(1.2 * u)))
    ld.text((x0 + pad_in, y0 + pad_in), title, font=f_t, fill=(*col, 255))
    y = y0 + pad_in + int(13 * u)
    for ln in lines:
        ld.text((x0 + pad_in, y), ln, font=f_d, fill=(225, 230, 240, 255))
        y += lh
    if prog is not None:
        pct = f"{int(prog * 100)} %"
        pw = ld.textlength(pct, font=f_d)
        # bar, then the % to its right
        by = y + int(5 * u)
        bx0 = x0 + pad_in
        bx1 = x0 + bw - pad_in - int(pw) - int(5 * u)
        ld.rounded_rectangle((bx0, by, bx1, by + int(6 * u)), radius=int(3 * u), fill=(255, 255, 255, 40))
        bx = bx0 + int((bx1 - bx0) * prog)
        if bx > bx0 + 1: ld.rounded_rectangle((bx0, by, bx, by + int(6 * u)), radius=int(3 * u), fill=(*col, 255))
        ld.text((bx1 + int(5 * u), by - int(3 * u)), pct, font=f_d, fill=(*col, 255))
    if alpha < 1.0:
        a = layer.split()[3].point(lambda v: int(v * alpha))
        layer.putalpha(a)
    ov.alpha_composite(layer)

def render_frame(im, alt, plasma, rf, f):
    ov, anchor = draw_hud(im, alt, plasma, rf)
    draw_caption(ov, anchor, f)
    return Image.alpha_composite(im.convert("RGBA"), ov).convert("RGB")

def one(args):
    src, dst, vals, f = args
    im = Image.open(src)
    render_frame(im, *vals, f).save(dst, compress_level=1)
    return dst

if __name__ == "__main__":
    frames_dir, out_dir, csv_path = sys.argv[1:4]
    first = int(sys.argv[4]) if len(sys.argv) > 4 else 1
    last = int(sys.argv[5]) if len(sys.argv) > 5 else 10 ** 6
    os.makedirs(out_dir, exist_ok=True)
    tel = load_telemetry(csv_path)
    jobs = []
    for f, vals in sorted(tel.items()):
        if f < first or f > last: continue
        src = os.path.join(frames_dir, f"c_{f:04d}.png")
        dst = os.path.join(out_dir, f"h_{f:04d}.png")
        if os.path.exists(src) and not os.path.exists(dst):
            jobs.append((src, dst, vals, f))
    with ProcessPoolExecutor() as ex:
        n = sum(1 for _ in ex.map(one, jobs, chunksize=8))
    print("composited", n, "frames ->", out_dir)
