"""Report any character a slide asks for that its font does not carry."""
import os, sys
from fontTools.ttLib import TTFont
from scene import FONT_DIR, FONT_FILES

_cmaps = {}
def cmap(name):
    if name not in _cmaps:
        f = TTFont(os.path.join(FONT_DIR, FONT_FILES[name]), lazy=True)
        s = set()
        for t in f["cmap"].tables:
            s |= set(t.cmap.keys())
        _cmaps[name] = s
    return _cmaps[name]

def check(slides):
    bad = {}
    for i, sl in enumerate(slides, 1):
        for kind, o in sl.sc.ops:
            if kind != "text":
                continue
            runs = o["runs"] or [None] * len(o["lines"])
            for j, ln in enumerate(o["lines"]):
                spec = runs[j] if j < len(runs) else None
                for txt, fname, _ in (spec or [(ln, o["name"], None)]):
                    cm = cmap(fname)
                    for ch in txt:
                        if ch != " " and ord(ch) not in cm:
                            bad.setdefault((i, fname, ch), txt.strip()[:60])
    return bad
