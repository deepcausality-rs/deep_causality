#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Extract Fig. 7 of Gauguet et al., Phys. Rev. A 78, 043615 (2008), arXiv:0809.0149: the phase
difference the two-photon light shift makes between Rabi frequencies Omega' and Omega, against
their ratio Omega'/Omega, at constant pulse area.

The figure is vector graphics, so the data are read from the PDF's paths, not from pixels:

    pdftocairo -svg -f 8 -l 8 "<the paper's PDF>" page8.svg
    python3 -I extract_gauguet2008_fig7.py page8.svg gauguet2008_fig7.csv

The paths share one transform, so the data are read in the paths' own coordinates and calibrated by
the axes' major ticks: the ratio axis from 0.0 to 1.0 in steps of 0.2, the phase axis from -35 to
0 mrad in steps of 5. A point is the centre of its square; its standard error is half the distance
between the far ends of its error bar's two segments, which start at the square's top and bottom
edges. The point at ratio 1 is the reference, sits at zero and has no error bar; it is not written.
Every other square must carry its own two segments, and every vertical segment must belong to a
square; the script exits with the offending square otherwise. It also exits when a calibration
anchor sits on no tick of its axis.
"""

import csv
import re
import sys

# The calibration anchors, in path coordinates: x at ratio 0 and 1, y at 0 and -35 mrad.
X_ZERO, X_ONE = 2315.859209, 4081.067503
Y_ZERO, Y_MINUS_35 = -7184.489424, -5860.038983
BLACK = "rgb(0%, 0%, 0%)"
# The stroke widths of the markers and error bars, and of the axis ticks.
THIN, TICKS = "2.17479", "5.0745"
# How far, in path units, an anchor may sit from its tick.
TICK_TOLERANCE = 0.05


def attributes(path):
    return dict(re.findall(r'(\w[\w-]*)="([^"]*)"', path))


def coordinates(attributes):
    nums = [float(x) for x in re.findall(r"-?\d+\.?\d*", attributes["d"])]
    return nums[0::2], nums[1::2]


def check_anchors(paths):
    """Exit unless each x anchor sits on a vertical tick and each y anchor on a horizontal one:
    black two-vertex segments of the ticks' stroke width."""
    ticks = [coordinates(a) for a in paths
             if a.get("stroke") == BLACK and a.get("stroke-width") == TICKS]
    ticks = [(xs, ys) for xs, ys in ticks if len(xs) == 2]
    for anchor in (X_ZERO, X_ONE):
        if not any(xs[0] == xs[1] and abs(xs[0] - anchor) < TICK_TOLERANCE for xs, _ in ticks):
            sys.exit(f"the anchor x = {anchor} sits on no vertical tick")
    for anchor in (Y_ZERO, Y_MINUS_35):
        if not any(ys[0] == ys[1] and abs(ys[0] - anchor) < TICK_TOLERANCE for _, ys in ticks):
            sys.exit(f"the anchor y = {anchor} sits on no horizontal tick")


def main(svg_path, points_path):
    svg = open(svg_path).read()
    drawn = [attributes(p) for p in re.findall(r"<path[^>]*/>", svg[svg.index("</defs>"):])]
    check_anchors(drawn)
    paths = [a for a in drawn if a.get("stroke") == BLACK and a.get("stroke-width") == THIN]
    squares = [coordinates(a) for a in paths if "Z" in a["d"]]
    segments = [coordinates(a) for a in paths if "Z" not in a["d"]]
    ratio = lambda x: (x - X_ZERO) / (X_ONE - X_ZERO)
    phase = lambda y: -35.0 * (y - Y_ZERO) / (Y_MINUS_35 - Y_ZERO)

    vertical = {i for i, (sx, _) in enumerate(segments) if sx[0] == sx[1]}
    rows, references, used = [], [], set()
    for xs, ys in squares:
        cx, top, bottom = (min(xs) + max(xs)) / 2, min(ys), max(ys)
        bar = [i for i in vertical if abs(segments[i][0][0] - cx) < 0.5
               and (top in segments[i][1] or bottom in segments[i][1])]
        ends = [y for i in bar for y in segments[i][1] if y not in (top, bottom)]
        r, p = ratio(cx), phase((top + bottom) / 2)
        if not bar and abs(r - 1) < 1e-3 and abs(p) < 0.05:
            references.append((r, p))
            continue
        if len(bar) != 2 or len(ends) != 2 or used & set(bar):
            sys.exit(f"the square at ratio {r:.4f}, {p:.3f} mrad, has {len(bar)} error-bar "
                     f"segments with {len(ends)} far ends, not its own two")
        used |= set(bar)
        rows.append((r, p, (phase(min(ends)) - phase(max(ends))) / 2))
    if len(references) != 1 or used != vertical:
        sys.exit(f"{len(references)} reference squares at ratio 1, and "
                 f"{len(vertical - used)} vertical segments belong to no square")

    with open(points_path, "w", newline="") as f:
        out = csv.writer(f, lineterminator="\n")
        out.writerow(["rabi_ratio", "phase_difference_mrad", "standard_error_mrad"])
        for r, p, se in sorted(rows):
            out.writerow([f"{r:.4f}", f"{p:.3f}", f"{abs(se):.3f}"])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
