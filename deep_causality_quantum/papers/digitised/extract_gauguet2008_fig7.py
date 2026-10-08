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
"""

import csv
import re
import sys

X_ZERO, X_ONE = 2315.859209, 4081.067503
Y_ZERO, Y_MINUS_35 = -7184.489424, -5860.038983
BLACK_THIN = ('stroke="rgb(0%, 0%, 0%)"', 'stroke-width="2.17479"')


def coordinates(path):
    d = re.search(r'd="([^"]*)"', path).group(1)
    nums = [float(x) for x in re.findall(r"-?\d+\.?\d*", d)]
    return nums[0::2], nums[1::2]


def main(svg_path, points_path):
    svg = open(svg_path).read()
    paths = [p for p in re.findall(r"<path[^>]*/>", svg[svg.index("</defs>"):])
             if all(attribute in p for attribute in BLACK_THIN)]
    squares = [coordinates(p) for p in paths if "Z" in p]
    segments = [coordinates(p) for p in paths if "Z" not in p]
    ratio = lambda x: (x - X_ZERO) / (X_ONE - X_ZERO)
    phase = lambda y: -35.0 * (y - Y_ZERO) / (Y_MINUS_35 - Y_ZERO)

    rows = []
    for xs, ys in squares:
        cx, top, bottom = (min(xs) + max(xs)) / 2, min(ys), max(ys)
        ends = [y for sx, sy in segments if sx[0] == sx[1] and abs(sx[0] - cx) < 0.5
                and (top in sy or bottom in sy) for y in sy if y not in (top, bottom)]
        if not ends:
            continue
        rows.append((ratio(cx), phase((top + bottom) / 2), (phase(min(ends)) - phase(max(ends))) / 2))

    with open(points_path, "w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["rabi_ratio", "phase_difference_mrad", "standard_error_mrad"])
        for r, p, se in sorted(rows):
            out.writerow([f"{r:.4f}", f"{p:.3f}", f"{abs(se):.3f}"])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
