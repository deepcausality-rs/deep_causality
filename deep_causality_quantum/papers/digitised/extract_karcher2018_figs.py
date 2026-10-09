#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Extract Figs. 2 and 4 of Karcher et al., New J. Phys. 20, 113041 (2018), arXiv:1804.04909: the
measured g(T) - g(1.8 µK) with the five-polynomial fit and its 68 % band, and the calculated
gravity shifts of the Zernike polynomials Z_n^0, n = 2 to 10, for 20 nm peak to peak.

The figures are vector graphics, so the data are read from the PDF's paths, not from pixels:

    pdftocairo -svg -f 2 -l 2 "<the paper's PDF>" page2.svg
    pdftocairo -svg -f 4 -l 4 "<the paper's PDF>" page4.svg
    python3 -I extract_karcher2018_figs.py page2.svg page4.svg karcher2018_fig2.csv karcher2018_fig4.csv

Each figure's paths share one transform, so the data are read in the paths' own coordinates and
calibrated by the axes' major ticks. Fig. 2: temperature on a log axis with decades at 0.1, 1 and
10 µK; the shift axis from 0 to 60 in steps of 20. Its axis label reads mm/s²; the values are
nm/s², the unit of the text and of Table I. A point is the centre of its circle; its standard
error is half the distance between the far ends of its error bar. The reference at 1.8 µK, the
red circle, has no error bar and is written with a standard error of zero. The fit is the red
polyline, whose vertices sit at the measured temperatures; the band's half-width is half the
distance between the band's two cyan boundaries there.

Fig. 4: temperature on a linear axis from 0 to 12 µK, the shift axis from -400 to 800 nm/s².
A value is the centre of its marker's bounding box, except a triangle's shift, which is at its
centroid; the segments joining the markers confirm both to 0.4 nm/s².

The script exits before writing a figure when one of its calibration anchors sits on no tick of
its axis; for Fig. 2, also when a circle other than the red reference lacks its two error-bar
segments, when the reference does not read zero near 1.8 µK, when an error-bar segment belongs to
no circle, and when the fit has no vertex at a circle's temperature.
"""

import csv
import re
import sys

FIG2 = "matrix(0.089322"
FIG4 = "matrix(0.0866005"
BLACK, RED, CYAN = "rgb(0%, 0%, 0%)", "rgb(100%, 0%, 0%)", "rgb(0%, 100%, 100%)"
COLOURS = {BLACK: 2, RED: 4, "rgb(0%, 0%, 100%)": 6, "rgb(100%, 0%, 100%)": 8,
           "rgb(0%, 100%, 0%)": 10}
# The calibration anchors, in path coordinates: Fig. 2's x at 0.1 and 10 µK and y at 0 and
# 60 nm/s²; Fig. 4's x at 0 and 12 µK and y at -400 and 800 nm/s².
FIG2_X, FIG2_Y = (745.0, 2481.719566), (-801.61146, -1508.185902)
FIG4_X, FIG4_Y = (517.493016, 2407.95364), (-311.987438, -1983.529116)
# The stroke widths of each figure's axis ticks.
FIG2_TICKS, FIG4_TICKS = "5.76018", "6.72067"
# How far, in path units, an anchor may sit from its tick, and an error bar or a fit vertex from
# its circle's centre.
TICK_TOLERANCE = 0.05
ALIGNED = 0.5


def paths(svg_path, transform):
    svg = open(svg_path).read()
    out = []
    for path in re.findall(r"<path[^>]*/>", svg[svg.index("</defs>"):]):
        attributes = dict(re.findall(r'(\w[\w-]*)="([^"]*)"', path))
        if attributes.get("transform", "").startswith(transform):
            nums = [float(x) for x in re.findall(r"-?\d+\.?\d*", attributes["d"])]
            out.append((attributes.get("stroke"), attributes.get("stroke-width"),
                        "Z" in attributes["d"], nums[0::2], nums[1::2]))
    return out


def check_anchors(drawn, width, xs, ys):
    """Exit unless each anchor in `xs` sits on a vertical tick and each in `ys` on a horizontal
    one: black two-vertex segments of stroke width `width`."""
    ticks = [p for p in drawn if p[0] == BLACK and p[1] == width and len(p[3]) == 2]
    for anchor in xs:
        if not any(p[3][0] == p[3][1] and abs(p[3][0] - anchor) < TICK_TOLERANCE for p in ticks):
            sys.exit(f"the anchor x = {anchor} sits on no vertical tick")
    for anchor in ys:
        if not any(p[4][0] == p[4][1] and abs(p[4][0] - anchor) < TICK_TOLERANCE for p in ticks):
            sys.exit(f"the anchor y = {anchor} sits on no horizontal tick")


def fig2(svg_path, out_path):
    temperature = lambda x: 10 ** (-1 + (x - FIG2_X[0]) / (FIG2_X[1] - FIG2_X[0]) * 2)
    shift = lambda y: (FIG2_Y[0] - y) / (FIG2_Y[0] - FIG2_Y[1]) * 60
    drawn = paths(svg_path, FIG2)
    check_anchors(drawn, FIG2_TICKS, FIG2_X, FIG2_Y)
    circles = [p for p in drawn if p[1] == "2.88009" and p[2]]
    bars = [p for p in drawn if p[0] == BLACK and p[1] == "1.92006" and p[3][0] == p[3][1]]
    fit = next(p for p in drawn if p[0] == RED and p[1] == "15.3605")
    upper, lower = [p for p in drawn if p[0] == CYAN and len(p[3]) == 18]
    rows, references, claimed = [], [], set()
    for colour, _, _, xs, ys in sorted(circles, key=lambda p: min(p[3])):
        cx, cy = (min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2
        bar = [k for k, b in enumerate(bars) if abs(b[3][0] - cx) < ALIGNED]
        i = min(range(18), key=lambda k: abs(fit[3][k] - cx))
        if abs(fit[3][i] - cx) > ALIGNED:
            sys.exit(f"the fit has no vertex at the circle at {temperature(cx):.4f} µK")
        if bar:
            if len(bar) != 2:
                sys.exit(f"the circle at {temperature(cx):.4f} µK has {len(bar)} error-bar "
                         f"segments, not two")
            ends = [y for k in bar for y in bars[k][4]]
            se = (shift(min(ends)) - shift(max(ends))) / 2
        elif colour == RED:
            references.append((temperature(cx), shift(cy)))
            se = 0.0
        else:
            sys.exit(f"the circle at {temperature(cx):.4f} µK has no error bar and is not the "
                     f"red reference")
        claimed |= set(bar)
        half = abs(shift(upper[4][i]) - shift(lower[4][i])) / 2
        rows.append([f"{temperature(cx):.4f}", f"{round(shift(cy), 2) + 0.0:.2f}", f"{se:.2f}",
                     f"{shift(fit[4][i]):.2f}", f"{half:.2f}"])
    if len(references) != 1 or abs(references[0][0] - 1.8) > 0.1 or abs(references[0][1]) > 0.005:
        sys.exit(f"the barless circles {references} are not one reference at 1.8 µK reading zero")
    if len(claimed) != len(bars):
        sys.exit(f"{len(bars) - len(claimed)} error-bar segments belong to no circle")
    with open(out_path, "w", newline="") as f:
        out = csv.writer(f, lineterminator="\n")
        out.writerow(["temperature_uK", "delta_g_nm_s2", "standard_error_nm_s2", "fit_nm_s2",
                      "fit_half_band_nm_s2"])
        out.writerows(rows)


def fig4(svg_path, out_path):
    temperature = lambda x: (x - FIG4_X[0]) / (FIG4_X[1] - FIG4_X[0]) * 12
    shift = lambda y: -400 + (FIG4_Y[0] - y) / (FIG4_Y[0] - FIG4_Y[1]) * 1200

    def centre(xs, ys):
        y = sum(ys[:3]) / 3 if len(ys) == 4 else (min(ys) + max(ys)) / 2
        return (min(xs) + max(xs)) / 2, y

    drawn = paths(svg_path, FIG4)
    check_anchors(drawn, FIG4_TICKS, FIG4_X, FIG4_Y)
    with open(out_path, "w", newline="") as f:
        out = csv.writer(f, lineterminator="\n")
        out.writerow(["n", "temperature_uK", "delta_g_nm_s2"])
        for colour, n in COLOURS.items():
            markers = [centre(p[3], p[4]) for p in drawn if p[0] == colour and p[1] == "2.88029" and p[2]]
            for x, y in sorted(markers):
                out.writerow([n, f"{temperature(x):.4f}", f"{shift(y):.2f}"])


if __name__ == "__main__":
    fig2(sys.argv[1], sys.argv[3])
    fig4(sys.argv[2], sys.argv[4])
