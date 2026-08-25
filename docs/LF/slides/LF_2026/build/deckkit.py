"""Design-system layer over the measured scene graph.

Tokens and idioms are transcribed from website/web_design/. Every text helper
returns the y coordinate *after* the block it drew, so stacking is arithmetic
rather than guesswork.

Deviation on record: 01-foundations.md measures --fg-2 at 4.26:1 on --bg-0 and
calls it "a decorative grey, not a text grey". Nothing a reader must read uses
it here; eyebrows take --accent (11.24:1). --fg-2 is kept for hairline meta.
"""

from scene import Scene, wrap, text_in, text_pt

# ---- palette, tokens.css:10-40 (dark, the default theme) ------------------
BG0 = (0x07, 0x0B, 0x10)
BG1 = (0x0B, 0x11, 0x18)
BG2 = (0x12, 0x1A, 0x23)
BG3 = (0x1A, 0x24, 0x30)
LINE1 = (0x1F, 0x2A, 0x36)
LINE2 = (0x2A, 0x3A, 0x4A)
FG0 = (0xE6, 0xED, 0xF3)
FG1 = (0xAA, 0xB3, 0xBD)
FG2 = (0x6B, 0x76, 0x82)
ACCENT = (0x5C, 0xD4, 0xE1)
WARN = (0xE3, 0xB3, 0x41)

# ---- typography, 02-typography.md ---------------------------------------
# Weight 540 has no static cut; Geist Medium (500) is the nearest face that
# still "reads as medium with presence" against a dark field.
SANS = "Geist"
SANS_MED = "Geist Medium"
SANS_SEMI = "Geist SemiBold"
MONO = "JetBrains Mono"
MONO_MED = "JetBrains Mono Medium"

# ---- layout --------------------------------------------------------------
W, H = 13.3333, 7.5
ML = 0.80
CW = W - 2 * ML
Y_EYEBROW = 0.42
Y_TITLE = 0.68
Y_FOOT_RULE = 6.92
Y_FOOT = 7.02
CONTENT_FLOOR = Y_FOOT_RULE - 0.16


def h_of(text, font, size, w, lead=1.42, spc=0):
    """Height in inches a wrapped block will occupy."""
    return len(wrap(text, font, size, w, spc)) * size * lead / 72.0


class S:
    """One slide."""

    def __init__(self, bg=BG0):
        self.sc = Scene(bg)
        self.low = 0.0

    # -- text ----------------------------------------------------------
    def text(self, x, y, w, body, font=SANS, size=13.5, color=FG1, lead=1.42,
             align="l", spc=0):
        lines = wrap(body, font, size, w, spc)
        leading = size * lead
        self.sc.lines(x, y, w, lines, font, size, color, leading, align, spc)
        end = y + len(lines) * leading / 72.0
        self.low = max(self.low, end)
        return end

    def eyebrow(self, x, y, w, body, color=ACCENT, size=11, align="l"):
        return self.text(x, y, w, body.upper(), MONO_MED, size, color,
                         lead=1.25, align=align, spc=int(size * 8))

    def title(self, x, y, w, body, size=30, color=FG0, align="l"):
        return self.text(x, y, w, body, SANS_MED, size, color, lead=1.13,
                         align=align, spc=int(-size * 1.2))

    # -- primitives ----------------------------------------------------
    def rule(self, x, y, w, color=LINE1, pt=0.75, alpha=None):
        self.sc.seg(x, y, x + w, y, color, pt, alpha)

    def dot(self, cx, cy, d=0.055, color=ACCENT, alpha=None):
        self.sc.oval(cx, cy, d, color, alpha)

    def divider(self, x, y, w, nodes=3):
        self.rule(x, y, w, LINE2, 0.75)
        for i in range(nodes):
            self.dot(x + w * (i + 1) / (nodes + 1), y, 0.05, ACCENT, 75)

    def reticle(self, x, y, w, h, arm=0.18, color=ACCENT, alpha=55, pt=1.0):
        """global.css:131-145 — 12x12 L-marks, accent at 0.55 opacity."""
        for cx, cy, dx, dy in ((x, y, 1, 1), (x + w, y, -1, 1),
                               (x, y + h, 1, -1), (x + w, y + h, -1, -1)):
            self.sc.seg(cx, cy, cx + dx * arm, cy, color, pt, alpha)
            self.sc.seg(cx, cy, cx, cy + dy * arm, color, pt, alpha)

    def panel(self, x, y, w, h, fill=BG1, border=LINE1, radius=0.09,
              ret=False, gradient=None, fill_alpha=None):
        self.sc.rect(x, y, w, h, fill=None if gradient else fill,
                     fill_alpha=fill_alpha, line=border, line_w=0.75,
                     radius=radius, gradient=gradient)
        if ret:
            self.reticle(x, y, w, h)
        self.low = max(self.low, y + h)

    def tick(self, x, y, h=0.32, w=0.042, color=ACCENT):
        """CausalStack.astro:149-160 — the left-edge accent tick."""
        self.sc.rect(x, y, w, h, fill=color, line=None)

    def arrow(self, x1, y1, x2, y2, color=LINE2, pt=1.1, alpha=None):
        self.sc.seg(x1, y1, x2, y2, color, pt, alpha, arrow=True)

    def image(self, path, x, y, w, h):
        self.sc.image(path, x, y, w, h)
        self.low = max(self.low, y + h)

    # -- composites ----------------------------------------------------
    def bullets(self, x, y, w, items, size=13, gap=0.16, dot_color=ACCENT,
                lead=1.42, body_color=FG1, indent=0.26):
        """Dot-marked list. The marker is a network node, not a bullet glyph."""
        cy = y
        for it in items:
            head, body = it if isinstance(it, tuple) else (None, it)
            self.dot(x + 0.055, cy + size * lead / 72.0 * 0.48, 0.055, dot_color)
            if head:
                cy = self.text(x + indent, cy, w - indent, head, SANS_MED, size,
                               FG0, lead=lead)
            if body:
                cy = self.text(x + indent, cy, w - indent, body, SANS, size,
                               body_color, lead=lead)
            cy += gap
        return cy - gap

    def rows(self, x, y, w, rows, key_w=1.6, key_size=10, key_color=ACCENT,
             val_size=12.5, val_color=FG1, pad=0.15, key_font=MONO_MED,
             val_font=SANS):
        """Hairline list — DESIGN.md §12.12."""
        cy = y
        for k, v in rows:
            self.rule(x, cy, w, LINE1, 0.75)
            top = cy + pad
            kb = self.text(x, top + 0.015, key_w, k, key_font, key_size,
                           key_color, lead=1.3,
                           spc=int(key_size * 8) if key_font in (MONO, MONO_MED) else 0)
            vb = self.text(x + key_w, top, w - key_w, v, val_font, val_size,
                           val_color, lead=1.34)
            cy = max(kb, vb) + pad
        self.rule(x, cy, w, LINE1, 0.75)
        return cy

    def chip(self, x, y, body, size=10, pad=0.17, h=0.30, color=ACCENT,
             border=LINE2, fill=BG2):
        spc = int(size * 5)
        w = pad * 2 + text_in(body, MONO_MED, size, spc)
        self.sc.rect(x, y, w, h, fill=fill, line=border, line_w=0.75,
                     radius=h / 2)
        self.text(x, y + (h - size * 1.3 / 72.0) / 2, w, body, MONO_MED, size,
                  color, lead=1.3, align="c", spc=spc)
        return x + w

    # -- scaffolding ---------------------------------------------------
    def header(self, eb, ttl, lede=None, title_size=30):
        self.eyebrow(ML, Y_EYEBROW, CW, eb)
        y = self.title(ML, Y_TITLE, CW, ttl, size=title_size)
        if lede:
            y = self.text(ML, y + 0.13, CW * 0.88, lede, SANS, 14, FG1,
                          lead=1.44)
        self.rule(ML, y + 0.19, CW, LINE1, 0.75)
        return y + 0.19 + 0.30

    def footer(self, num=None, total=None):
        self.rule(ML, Y_FOOT_RULE, CW, LINE1, 0.75)
        self.text(ML, Y_FOOT, 6.0, "DEEPCAUSALITY  ·  LF AI & DATA  ·  TAC 2026",
                  MONO, 8.5, FG2, lead=1.25, spc=68)
        if num is not None:
            self.text(W - ML - 2.0, Y_FOOT, 2.0, f"{num:02d} / {total:02d}",
                      MONO_MED, 8.5, ACCENT, lead=1.25, align="r", spc=68)

    def notes(self, text):
        self.sc.notes = text.strip()

    def check(self, name):
        if self.low > CONTENT_FLOOR + 0.001:
            print(f"  !! OVERFLOW {name}: content reaches {self.low:.3f}in, "
                  f"floor is {CONTENT_FLOOR:.3f}in")
            return False
        return True


# ---- measuring twins, so a panel can be sized before its content is drawn --
def h_bullets(items, w, size=13, gap=0.16, lead=1.42, indent=0.26):
    tot = 0.0
    for it in items:
        head, body = it if isinstance(it, tuple) else (None, it)
        if head:
            tot += h_of(head, SANS_MED, size, w - indent, lead)
        if body:
            tot += h_of(body, SANS, size, w - indent, lead)
        tot += gap
    return tot - gap


def h_rows(rows, w, key_w=1.6, key_size=10, val_size=12.5, pad=0.15,
           key_font=MONO_MED, val_font=SANS):
    cy = 0.0
    for k, v in rows:
        kb = h_of(k, key_font, key_size, key_w, 1.3,
                  int(key_size * 8) if key_font in (MONO, MONO_MED) else 0)
        vb = h_of(v, val_font, val_size, w - key_w, 1.34)
        cy += pad + max(kb + 0.015, vb) + pad
    return cy
