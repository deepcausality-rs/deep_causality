#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Extract Fig. 9(a) and (c) of Louchet-Chauvet et al., New J. Phys. 13, 065025 (2011): the
measured g(T_at) - g(2 µK) and the Zernike-polynomial fits.

The figure is vector graphics, so the data are read from the PDF's paths, not from pixels:

    pdftocairo -svg -f 16 -l 16 "<the paper's PDF>" page16.svg
    python3 -I extract_louchet2011_fig9.py page16.svg louchet2011_fig9_points.csv louchet2011_fig9_fits.csv

Both panels share one transform. The temperature axis runs from the frame's left edge, 0 µK, to
its right edge, 7 µK, with major ticks every 1 µK; the gravity axis has major ticks every 2 µGal,
and the point at 2 µK, the reference, sits at zero. A data point is the centre of its circle; its
standard error is half the distance between its error bar's caps. A fit is a polyline; the plot
clips the polylines to the frame, so a fit that leaves the frame has no value at zero temperature.

Panel (a): the measured points, the linear fit (thick grey), and the fits with Zernike polynomials
up to n_max = 2 (green), 4 (red), 6 (grey), 8 (blue) and 10 (black). Panel (c): the n_max = 10 fit
with one added fake point below 2 µK each; the fake points are the stars of the fit's colour.
"""

import csv
import re
import sys

X0 = 19.9516
X_PER_UK = (174.479158 - 19.9516) / 7.0
Y_PER_UGAL = 23.665 / 2.0
ZERO_A = 229.70
ZERO_C = ZERO_A - 153.2

# Path indices in the page's drawing order, as pdftocairo writes them; each is checked against the
# path's shape and colour before it is read.
CIRCLES_A = [49, 41, 39, 43, 47, 45]
ERROR_BARS_A = 50
BLACK, GREEN, BLUE = "rgb(0%, 0%, 0%)", "rgb(0%, 50.19989%, 0%)", "rgb(0%, 0%, 100%)"
RED, GREY, DARK_GREY = "rgb(100%, 0%, 0%)", "rgb(50%, 50%, 50%)", "rgb(25%, 25%, 25%)"
LIGHT_GREY = "rgb(75.39978%, 75.39978%, 75.39978%)"
FITS = [
    ("linear", 32, ZERO_A, LIGHT_GREY), ("n_max 2", 36, ZERO_A, GREEN),
    ("n_max 4", 35, ZERO_A, RED), ("n_max 6", 34, ZERO_A, GREY),
    ("n_max 8", 37, ZERO_A, BLUE), ("n_max 10", 33, ZERO_A, BLACK),
    ("fake black", 206, ZERO_C, BLACK), ("fake light grey", 207, ZERO_C, LIGHT_GREY),
    ("fake red", 208, ZERO_C, RED), ("fake blue", 209, ZERO_C, BLUE),
    ("fake dark grey", 210, ZERO_C, DARK_GREY),
]
# An error bar's vertices lie within this distance of its point's temperature, in µK: its caps
# reach 0.055 µK to each side, and the closest two points are 0.45 µK apart.
BAR_WINDOW_UK = 0.08


def attributes(path):
    return dict(re.findall(r'(\w[\w-]*)="([^"]*)"', path))


def coordinates(path):
    nums = [float(x) for x in re.findall(r"-?\d+\.?\d*", attributes(path)["d"])]
    return nums[0::2], nums[1::2]


def main(svg_path, points_path, fits_path):
    svg = open(svg_path).read()
    paths = re.findall(r"<path[^>]*/>", svg[svg.index("</defs>"):])
    temperature = lambda x: (x - X0) / X_PER_UK
    gravity = lambda y, zero: (y - zero) / Y_PER_UGAL

    centres, diameters = [], []
    for i in CIRCLES_A:
        if "Z" not in attributes(paths[i])["d"]:
            sys.exit(f"path {i} is not a closed marker")
        xs, ys = coordinates(paths[i])
        diameters += [max(xs) - min(xs), max(ys) - min(ys)]
        centres.append((temperature((min(xs) + max(xs)) / 2), gravity((min(ys) + max(ys)) / 2, ZERO_A)))
    if max(diameters) > 1.05 * min(diameters):
        sys.exit(f"the markers are not circles of one size: {diameters}")
    xs, ys = coordinates(paths[ERROR_BARS_A])
    bars = [(temperature(x), gravity(y, ZERO_A)) for x, y in zip(xs, ys)]
    points = []
    for t, g in sorted(centres):
        bar = [y for x, y in bars if abs(x - t) < BAR_WINDOW_UK]
        if not bar or not min(bar) < g < max(bar):
            sys.exit(f"the point at {t:.3f} µK, {g:.3f} µGal, does not sit inside an error bar")
        points.append((t, g, (max(bar) - min(bar)) / 2))
    claimed = sum(1 for x, _ in bars if any(abs(x - t) < BAR_WINDOW_UK for t, _, _ in points))
    if claimed != len(bars):
        sys.exit(f"{len(bars) - claimed} error-bar vertices belong to no point")
    t, g, _ = points[0]
    if abs(t - 2.0) > 0.01 or abs(g) > 0.02:
        sys.exit(f"the reference point reads {t:.3f} µK, {g:.3f} µGal, not 2 µK at zero")
    for label, i, _, colour in FITS:
        path = attributes(paths[i])
        if path.get("stroke") != colour or "Z" in path["d"]:
            sys.exit(f"path {i} is not the open {colour} polyline of the fit {label}")

    with open(points_path, "w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["temperature_uK", "delta_g_uGal", "standard_error_uGal"])
        for t, g, se in points:
            out.writerow([f"{t:.3f}", f"{g:.3f}", f"{se:.3f}"])

    with open(fits_path, "w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["fit", "temperature_uK", "delta_g_uGal"])
        for label, i, zero, _ in FITS:
            xs, ys = coordinates(paths[i])
            for x, y in sorted(zip(xs, ys)):
                out.writerow([label, f"{temperature(x):.4f}", f"{gravity(y, zero):.4f}"])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3])
