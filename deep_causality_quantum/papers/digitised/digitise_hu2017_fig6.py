#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Digitise Fig. 6(a) and (b) of Hu et al., Phys. Rev. A 96, 033414 (2017), arXiv:1805.05159: the
magnetic field along the atoms' trajectory with the bias coil at its nominal current, (a), and at
half of it, (b).

The figure is a raster image embedded in the PDF, so the curves are read from its pixels:

    pdfimages -png -f 9 -l 9 "<the paper's PDF>" p9
    python3 -I digitise_hu2017_fig6.py p9-000.png hu2017_fig6.csv

Each panel's height axis runs at 13 pixels per cm with 40 cm at the frame's left edge, the column
`xl`. The field axis is calibrated by the two major ticks inside the frame's left edge, 5700 and
5600 nT in (a), 3000 and 2900 nT in (b). A curve's value in a pixel column is the mean row of the
longest run of the curve's colour in that column, red in (a), blue in (b); in the columns left of
the legend the rows of the legend are dropped. The two panels share their columns, so each row of
the output pairs the nominal and the half-current field at one height.
"""

import csv
import sys

import numpy as np
from PIL import Image

PIXELS_PER_CM = 13.0
LEFT_EDGE_CM = 40.0


def curve(image, xl, rows, cols, mask, ticks):
    dark = (image[..., 0] < 100) & (image[..., 1] < 100) & (image[..., 2] < 100)
    y0, y1 = rows
    x0, x1 = cols
    tick_rows = [y for y in range(y0, y1) if dark[y, x0:x0 + 7].sum() >= 5]
    groups = []
    for y in tick_rows:
        if groups and y - groups[-1][-1] <= 1:
            groups[-1].append(y)
        else:
            groups.append([y])
    (c1, c2), (v1, v2) = [np.mean(g) for g in groups], ticks
    per_row = (v1 - v2) / (c1 - c2)
    points = []
    for x in range(x0 + 8, x1 - 2):
        found = np.nonzero(mask[y0 + 2:y1 - 2, x])[0] + y0 + 2
        if x < xl + 330:
            found = found[found > y0 + 75]
        if len(found) == 0:
            continue
        run = max(np.split(found, np.nonzero(np.diff(found) > 2)[0] + 1), key=len)
        points.append((LEFT_EDGE_CM + (x - xl) / PIXELS_PER_CM, v1 + (run.mean() - c1) * per_row))
    return points


def main(image_path, out_path):
    image = np.asarray(Image.open(image_path).convert("RGB")).astype(int)
    r, g, b = image[..., 0], image[..., 1], image[..., 2]
    nominal = curve(image, 91.5, (16, 391), (93, 805), (r > 180) & (g < 110) & (b < 110), (5700, 5600))
    half = dict(curve(image, 934.5, (16, 391), (936, 1648), (b > 180) & (r < 110) & (g < 110), (3000, 2900)))
    with open(out_path, "w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["height_cm", "B_nominal_nT", "B_half_nT"])
        for z, field in nominal:
            if z in half:
                out.writerow([f"{z:.4f}", f"{field:.3f}", f"{half[z]:.3f}"])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
