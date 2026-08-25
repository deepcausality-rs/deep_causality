#!/usr/bin/env python3
"""DeepCausality — LF AI & Data Technical Advisory Council, 2026.

Every block is measured against the real TTFs before it is placed, and `check()`
fails the build if a slide reaches past the footer rule. Copy is written to a
vertical budget: header + content + footer must fit 7.5in.
"""

import os
from pptx import Presentation
from pptx.util import Inches

from deckkit import *          # noqa: F401,F403
import deckkit as K
from scene import scene_to_slide, scene_to_png

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.join(HERE, "assets")
OUT_DIR = os.path.abspath(os.path.join(HERE, ".."))
PREVIEW = os.path.join(HERE, "preview")
TOTAL = 15

PAD = 0.26          # panel inner padding
BODY = 12.0         # panel body size
LEAD = 1.40
GAP = 0.28          # gap between panels

SLIDES, NOTES, OK = [], [], [True]


def emit(s, num, title, note):
    if not s.check(f"{num:02d} {title}"):
        OK[0] = False
    s.footer(num, TOTAL)
    s.notes(note)
    SLIDES.append(s)
    NOTES.append((num, title, note))


# ==========================================================================
# 01 — Title
# ==========================================================================
s = S()
art_w = 8.10
art_h = art_w * 630 / 1200
art_x = (W - art_w) / 2
art_y = 0.56
s.panel(art_x - 0.10, art_y - 0.10, art_w + 0.20, art_h + 0.20, fill=BG1,
        border=LINE2, radius=0.16)
s.image(os.path.join(ASSETS, "dc_hero.jpg"), art_x, art_y, art_w, art_h)
s.reticle(art_x - 0.10, art_y - 0.10, art_w + 0.20, art_h + 0.20, arm=0.22)

y = art_y + art_h + 0.44
s.divider(W / 2 - 1.55, y, 3.10)
y = s.title(0, y + 0.26, W,
            "Dynamic Causality: One Stack from Discovery to Deployment",
            size=26, align="c")
y = s.text(0, y + 0.24, W,
           "Marvin Hansen  ·  Director, Center for Dynamic Causality",
           SANS, 14.5, FG1, lead=1.3, align="c")
s.eyebrow(0, y + 0.18, W,
          "LF AI & Data  ·  Technical Advisory Council  ·  2026", align="c")

cl_w = 0.98
s.image(os.path.join(ASSETS, "center_logo_cream.png"), W - ML - cl_w, H - 1.02,
        cl_w, cl_w * 1377 / 1762)
s.text(ML, H - 0.58, 5.2, "LF AI & Data Sandbox project since 2023", MONO, 9,
       FG2, lead=1.3, spc=54)
s.low = 0.0                      # full-bleed title, no content floor
SLIDES.append(s)
s.notes("""
Good morning, and thank you for the slot.

I am Marvin Hansen, Director of the Center for Dynamic Causality, and I am here for
DeepCausality — an LF AI & Data Sandbox project since September 2023.

I presented here in 2023 with a proposal. I am back with a re-introduction, because
almost everything below the surface has changed since then, and because this council
has largely turned over in the meantime.

Twenty minutes, six parts. I will keep the mathematics to one slide and spend the time
on what the project actually ships.
""")
NOTES.append((1, "Title", s.sc.notes))

# ==========================================================================
# 02 — Agenda
# ==========================================================================
s = S()
y = s.header("Agenda", "What this talk covers")
items = [
    ("01", "The premise",
     "Why DeepCausality starts from a different axiom than classical causality"),
    ("02", "The axiom and the primitives",
     "One working definition, three computable primitives, one safety layer"),
    ("03", "One causal stack",
     "Discovery, modeling, action, governance, deployment in a single substrate"),
    ("04", "Two downstream projects",
     "Counterfactual fluid dynamics, and quantum causal models"),
    ("05", "Case study",
     "DeepCausality at United Airlines via the Service Radar integration"),
    ("06", "Project health",
     "Verification posture, governance, and where collaboration would help"),
]
NUM_W, HEAD_W, COLGAP = 0.68, 3.55, 0.30
cy = y + 0.06
for n, head, body in items:
    s.rule(ML, cy, CW, LINE1)
    top = cy + 0.19
    a = s.text(ML, top + 0.02, NUM_W, n, MONO_MED, 13, ACCENT, lead=1.25, spc=90)
    b = s.text(ML + NUM_W, top - 0.02, HEAD_W, head, SANS_MED, 16, FG0,
               lead=1.22, spc=-14)
    c = s.text(ML + NUM_W + HEAD_W + COLGAP, top,
               CW - NUM_W - HEAD_W - COLGAP, body, SANS, 13.5, FG1, lead=1.34)
    cy = max(a, b, c) + 0.19
s.rule(ML, cy, CW, LINE1)

emit(s, 2, "Agenda", """
Six parts.

First the premise — one slide on why we did not simply build another causal-graph
library. Then the axiom and the primitives that make it computable. Then the piece I
most want you to take away: one causal stack running from raw data to a governed
production action.

Then two downstream projects built on that stack, which are in my view the proof that
the substrate generalizes: counterfactual fluid dynamics, and quantum causal models.

Then a case study, and finally project health and where we could use help.

Roughly a minute per slide. Stop me at any point.
""")

# ==========================================================================
# 03 — The premise
# ==========================================================================
s = S()
y = s.header("Premise", "Classical causality assumes a background that moved",
             "Every classical framework inherits a definition of cause that is two thousand "
             "years old, and the static spacetime that came with it.")

pw = (CW - GAP) / 2
inner = pw - 2 * PAD
cols = [
    (ML, LINE1, False, "Classical causality", "Seneca → Pearl", FG2, [
        "Time is a straight line. Cause precedes effect, always.",
        "The causal rules are fixed, and fixed at design time.",
        "Space and time are a static background the model may assume.",
        "Structure is enumerated up front, then held constant.",
    ], "Sound on its own ground. The ground is what moved.", FG2),
    (ML + pw + GAP, LINE2, True, "Dynamic causality", "Whitehead → process",
     ACCENT, [
        "Causality is a process of becoming, not a static snapshot.",
        "Causal rules may evolve while the system is running.",
        "Spacetime is data the model reads, not a frame it assumes.",
        "Structure may emerge at runtime and still be governable.",
    ], "Quantum physics and general relativity broke the fixed background first.",
     FG1),
]
ph = max(PAD + h_of(eb.upper(), MONO_MED, 11, inner, 1.25, 88) + 0.09
         + h_of(hd, SANS_MED, 18, inner, 1.15, -21) + 0.18 + 0.20
         + h_bullets(bl, inner, 12.5, gap=0.16) + 0.22
         + h_of(foot, SANS, 12, inner, 1.34) + PAD
         for _, _, _, eb, hd, _, bl, foot, _ in cols)
for px, border, ret, eb, hd, dotc, bl, foot, footc in cols:
    s.panel(px, y, pw, ph, fill=BG1, border=border, ret=ret)
    cy = s.eyebrow(px + PAD, y + PAD, inner, eb) + 0.09
    cy = s.text(px + PAD, cy, inner, hd, SANS_MED, 18, FG0, lead=1.15,
                spc=-21) + 0.18
    s.rule(px + PAD, cy, inner, border)
    cy = s.bullets(px + PAD, cy + 0.20, inner, bl, size=12.5, gap=0.16,
                   dot_color=dotc) + 0.22
    s.text(px + PAD, cy, inner, foot, SANS, 12, footc, lead=1.34)

emit(s, 3, "The premise", """
This is the slide the project rests on, so I will spend a full minute here.

Classical computational causality traces its definition of cause back to Seneca — two
thousand years old — and it inherits the static spacetime that came with it. Pearl's
structural causal models, Granger, Rubin, dynamic Bayesian networks: all excellent, all
well validated, and all assuming time runs straight, that the rules are fixed at design
time, and that the causal structure is enumerated up front.

That is not a flaw. It is a scope. And contemporary science has already moved outside
it. In quantum physics and in general relativity the fixed background does not hold.
Closer to home, any system that crosses a physical or operational regime boundary
changes its own causal rules mid-flight.

So we change the starting point. We root the project in Whitehead's process philosophy —
causality as a process of becoming — and make spacetime data the model reads rather than
a frame the model assumes.

The consequence is on the right: rules may evolve at runtime, structure may emerge, and
— the part that matters for production — it is still governable. I will come back to
that.
""")

# ==========================================================================
# 04 — The axiom
# ==========================================================================
s = S()
y = s.header("The axiom", "One working definition")

quote = ("Dynamic causality is the spacetime-agnostic monadic process in which one propagating "
         "effect is obtained from another by applying a causal function within the monad.")
fh = PAD + 34 * 1.15 / 72.0 + 0.16 + h_of(quote, SANS, 12, CW - 2.4, 1.32) + PAD
s.panel(ML, y, CW, fh, fill=BG2, border=LINE2, ret=True)
cy = s.text(ML, y + PAD, CW, "m₂  =  m₁  >>=  f", MONO_MED, 34, ACCENT,
            lead=1.15, align="c", spc=15) + 0.16
s.text(ML + 1.2, cy, CW - 2.4, quote, SANS, 12, FG1, lead=1.32, align="c")

cy = y + fh + GAP
qw = (CW - 3 * 0.24) / 4
qin = qw - 0.28 - PAD
cards = [
    ("Monadic process",
     "An arity-5 carrier: value, state, context, error, audit log. The monad laws thread "
     "that bookkeeping through every step."),
    ("Functional dependency",
     "Each effect is obtained from the previous one by applying a causal function f. "
     "Chained, those steps propagate effect."),
    ("Spacetime-agnostic",
     "Time and space are not built into the relation. They are inputs the causal function "
     "reads, like any other."),
    ("Explicit context",
     "Anything time-like or space-like lives in a queryable hypergraph — Euclidean, "
     "Minkowski, anything between."),
]
ch = max(PAD + h_of(h, SANS_MED, 14.5, qin, 1.2, -15) + 0.12
         + h_of(b, SANS, BODY, qin, LEAD) + PAD for h, b in cards)
for i, (head, body) in enumerate(cards):
    cx = ML + i * (qw + 0.24)
    s.panel(cx, cy, qw, ch, fill=BG1, border=LINE1)
    s.tick(cx, cy + PAD + 0.02, h=0.30, w=0.038)
    ty = s.text(cx + 0.28, cy + PAD, qin, head, SANS_MED, 14.5, FG0, lead=1.2,
                spc=-15) + 0.12
    s.text(cx + 0.28, ty, qin, body, SANS, BODY, FG1, lead=LEAD)

s.text(ML, cy + ch + 0.26, CW,
       "A ripple in a pond. One ripple is an effect; it propagates and produces the next. "
       "DeepCausality defines how ripples spread, what each carries, and what happens when the "
       "rules for spreading themselves change.", SANS, 12, FG2, lead=1.36)

emit(s, 4, "The axiom", """
Here is the whole premise in one line: m-two equals m-one bind f.

Read it as: dynamic causality is the spacetime-agnostic monadic process in which one
propagating effect is obtained from another by applying a causal function within the
monad.

Four things are packed in there.

Monadic process — the carrier is an arity-five record: value, state, context, error, and
an audit log. Because it obeys the monad laws, that bookkeeping is threaded
automatically. That is where end-to-end explainability comes from; nobody has to remember
to log.

Functional dependency — each effect comes from the previous one by applying a causal
function. Chain those and you get effect propagation.

Spacetime-agnostic — time and space are not in the relation. They are inputs.

And because they are not built in, anything time-like or space-like has to live somewhere
explicit: the Context, a typed hypergraph. That is what lets the same model run in
Euclidean space, in Minkowski spacetime, or in a non-Euclidean context.

The intuition, if you want one: a ripple in a pond. One ripple is an effect, it
propagates, it produces the next one. We define how ripples spread — and what happens
when the rules for spreading change.
""")

# ==========================================================================
# 05 — Consequence
# ==========================================================================
s = S()
y = s.header("Consequence", "Classical frameworks fall out as special cases",
             "The axiom sits low enough in the stack that we did not have to replace anything.")

lw = CW * 0.50
rw = CW - lw - GAP
lin, rin = lw - 2 * PAD, rw - 2 * PAD
left_rows = [("SCM", "Pearl structural causal models"),
             ("DBN", "Dynamic Bayesian networks"),
             ("GRANGER", "Granger causality"),
             ("RUBIN", "Rubin causal model, and CATE")]
right_rows = [("DYNAMIC", "Causal structure changes as the context changes"),
              ("ADAPTIVE", "Causal rules update themselves at runtime"),
              ("EMERGENT", "Structure not enumerable at design time")]
lfoot = "Each implemented under examples/classical_causality_examples."
rfoot = "Three modalities a fixed-structure framework cannot express."
ph = max(PAD + h_of("SPECIALIZATIONS", MONO_MED, 11, lin, 1.25, 88) + 0.14
         + h_rows(left_rows, lin, key_w=1.42, val_size=12, pad=0.13) + 0.18
         + h_of(lfoot, SANS, 11.5, lin, 1.32) + PAD,
         PAD + h_of("WHAT THE AXIOM ADDS", MONO_MED, 11, rin, 1.25, 88) + 0.14
         + h_rows(right_rows, rin, key_w=1.42, val_size=12, pad=0.13) + 0.18
         + h_of(rfoot, SANS, 11.5, rin, 1.32) + PAD)

s.panel(ML, y, lw, ph, fill=BG1, border=LINE1)
cy = s.eyebrow(ML + PAD, y + PAD, lin, "Specializations") + 0.14
cy = s.rows(ML + PAD, cy, lin, left_rows, key_w=1.42, val_size=12,
            pad=0.13) + 0.18
s.text(ML + PAD, cy, lin, lfoot, SANS, 11.5, FG2, lead=1.32)

rx = ML + lw + GAP
s.panel(rx, y, rw, ph, fill=BG1, border=LINE2, ret=True)
cy = s.eyebrow(rx + PAD, y + PAD, rin, "What the axiom adds") + 0.14
cy = s.rows(rx + PAD, cy, rin, right_rows, key_w=1.42, val_size=12,
            pad=0.13) + 0.18
s.text(rx + PAD, cy, rin, rfoot, SANS, 11.5, FG1, lead=1.32)

by = y + ph + GAP
bar_t = "Reasoning is free to be emergent. Actions are not."
bar_b = ("Where structure emerges at runtime, static verification of the reasoning stops being "
         "possible. Verifiability is restored at the action boundary instead — the Effect Ethos, "
         "on the next slide.")
bh = (PAD * 0.72 + h_of(bar_t, SANS_MED, 16.5, CW - 2 * PAD, 1.22, -18) + 0.10
      + h_of(bar_b, SANS, 12, CW - 2 * PAD, 1.34) + PAD * 0.72)
s.panel(ML, by, CW, bh, fill=BG2, border=LINE2)
s.tick(ML, by + PAD * 0.72 + 0.02, h=0.34, w=0.045)
cy = s.text(ML + PAD, by + PAD * 0.72, CW - 2 * PAD, bar_t, SANS_MED, 16.5, FG0,
            lead=1.22, spc=-18) + 0.10
s.text(ML + PAD, cy, CW - 2 * PAD, bar_b, SANS, 12, FG1, lead=1.34)

emit(s, 5, "Consequence", """
Two consequences, and the second is the one I would defend hardest.

First: because the axiom sits low enough in the stack, we did not have to replace
anything. Pearl's structural causal models, dynamic Bayesian networks, Granger, Rubin,
conditional average treatment effects — each drops out as a parametric specialization of
the same relation. We do not argue that; we implement each one directly in the
classical-causality examples folder, so you can read the code.

Second: it adds three modalities a fixed-structure framework cannot express. Dynamic —
structure changes with context. Adaptive — the rules update at runtime. Emergent —
structure that was not enumerable at design time.

Now the honest objection: if structure can emerge at runtime, you cannot statically
verify the reasoning any more. That is true, and we do not pretend otherwise.

Our answer is the line at the bottom. Reasoning is free to be emergent; actions are not.
We move verifiability from the reasoning boundary to the action boundary. That is a
deliberate architectural trade, and it is what makes emergent causality safe enough to
deploy. The next slide shows the layer that does it.
""")

# ==========================================================================
# 06 — Primitives
# ==========================================================================
s = S()
y = s.header("Primitives", "Three primitives, and a safety layer",
             "Both primitives emit the same carrier, so structural and sequential reasoning "
             "compose in one path.")


def dbox(x, yy, w, h, title, sub=None, eb=None, border=LINE2, size=13.5,
         eb_color=ACCENT):
    """A diagram node: optional mono eyebrow, title, optional caption."""
    s.panel(x, yy, w, h, fill=BG1, border=border, radius=0.08)
    eh = h_of(eb.upper(), MONO_MED, 8, w - 0.16, 1.25, 64) if eb else 0
    th = h_of(title, SANS_MED, size, w - 0.16, 1.2, -12)
    sh = h_of(sub, SANS, 10.5, w - 0.18, 1.26) if sub else 0
    block = eh + (0.07 if eb else 0) + th + (0.05 if sub else 0) + sh
    cy = yy + (h - block) / 2
    if eb:
        cy = s.text(x + 0.08, cy, w - 0.16, eb.upper(), MONO_MED, 8, eb_color,
                    lead=1.25, align="c", spc=64) + 0.07
    cy = s.text(x + 0.08, cy, w - 0.16, title, SANS_MED, size, FG0, lead=1.2,
                align="c", spc=-12)
    if sub:
        s.text(x + 0.09, cy + 0.05, w - 0.18, sub, SANS, 10.5, FG1, lead=1.26,
               align="c")


MIDY = y + 1.70
BOXH = 0.90
ROW1, ROW2 = MIDY - 1.15, MIDY + 0.25
dbox(0.70, ROW1, 2.08, BOXH, "Causaloid", "structural reasoning")
dbox(0.70, ROW2, 2.08, BOXH, "Causal Monad", "sequential reasoning")

s.panel(3.16, MIDY - 0.40, 2.32, 0.80, fill=ACCENT, fill_alpha=18,
        border=ACCENT, radius=0.40)
s.text(3.16, MIDY - 0.113, 2.32, "PropagatingEffect", MONO_MED, 12.5, ACCENT,
       lead=1.3, align="c", spc=20)

dbox(5.86, MIDY - BOXH / 2, 2.26, BOXH, "Causal State Machine",
     "infers active states at runtime", size=12.5)

s.sc.diamond(8.46, MIDY - 0.72, 1.68, 1.44, fill=BG1, line=LINE2)
dty = s.text(8.46, MIDY - 0.26, 1.68, "Effect Ethos", SANS_MED, 12.5, FG0,
             lead=1.24, align="c")
s.text(8.46, dty + 0.02, 1.68, "verdict?", SANS, 10.5, FG1, lead=1.24,
       align="c")

# The verdict rides inside the outcome node, so no label sits on an arrow.
dbox(10.55, MIDY - 1.32, 2.08, 1.02, "Action fires", "+ audit entry",
     eb="Obligatory / Optional", border=ACCENT, size=12.5)
dbox(10.55, MIDY + 0.30, 2.08, 1.02, "Action stopped", "+ reason locked to log",
     eb="Impermissible", size=12.5, eb_color=FG2)

s.arrow(2.80, ROW1 + BOXH / 2, 3.12, MIDY - 0.11)
s.arrow(2.80, ROW2 + BOXH / 2, 3.12, MIDY + 0.11)
s.arrow(5.50, MIDY, 5.82, MIDY)
s.arrow(8.16, MIDY, 8.42, MIDY)
s.arrow(10.16, MIDY - 0.22, 10.51, MIDY - 0.72, ACCENT, alpha=80)
s.arrow(10.16, MIDY + 0.22, 10.51, MIDY + 0.72)

ctx_y = ROW2 + BOXH + 0.52
dbox(0.70, ctx_y, 2.08, 0.68, "Context", border=LINE1, size=12.5)
s.arrow(1.74, ctx_y - 0.02, 1.74, ROW2 + BOXH + 0.06, LINE1, 1.0)
s.text(3.02, ctx_y - 0.02, 9.60,
       "A typed weighted hypergraph of contextoids carrying data, space, time, spacetime and "
       "symbolic payloads, mutated in place across a run. Counterfactuals run as parallel "
       "extra-contexts against the same Causaloid, without disturbing the primary one.",
       SANS, 12, FG1, lead=1.42)

emit(s, 6, "Primitives", """
Three primitives operationalize the axiom, plus an optional fourth for safety.

On the left, two reasoning primitives. The Causaloid carries causal structure, and it is
isomorphic-recursive: a singleton, a collection, and a hypergraph all implement the same
trait surface, so they nest into each other to any depth. The Causal Monad carries
sequencing — pure, bind, and intervene, where intervene is Pearl's do-operator applied
mid-chain.

The important part is the middle. Both emit the same carrier. So you can take a
Causaloid's verdict and bind directly onto it, or run a bind chain and feed the result
into a Causaloid. The boundary between structural and sequential reasoning moves as the
problem moves. You do not pick a framework and then glue.

The Causal State Machine is the bridge to the outside world. Its active state space is
inferred at runtime from the propagating effect, not enumerated at design time — that is
how it avoids the classical finite-state-machine limitation.

Then the Effect Ethos. Every action the CSM proposes gets intercepted and evaluated
against an immutable graph of computable norms, under a defeasible deontic calculus —
Olson and Forbus. The verdict is obligatory, impermissible, or optional with a cost. If
it is stopped, the reason is locked to the audit log alongside the line of reasoning that
produced it. That is what an auditor reads afterwards.

Underneath everything: the Context. Because spacetime is not built into the relation, it
has to live here — and counterfactuals run as parallel extra-contexts without disturbing
the primary one.
""")

# ==========================================================================
# 07 — Uniform mathematics (deliberately one page)
# ==========================================================================
s = S()
y = s.header("Substrate", "One categorical interface across every math layer",
             "Most scientific stacks force a bridge between domain libraries. Everything here "
             "implements one surface.")

tw = CW * 0.60
tin = tw - 2 * PAD
math_rows = [("Mechanics", "CausalTensor<T>", "Functor — map over field data"),
             ("Algebra", "CausalMultiVector<T>", "Monad — chain operations"),
             ("Topology", "Manifold<T>", "Comonad — neighbourhood analysis"),
             ("Causality", "PropagatingEffect<T>", "Monad — sequencing plus log")]
tfoot = ("fmap, bind, extend and extract mean the same thing on every container. Arity-5 HKTs "
         "via the witness pattern, on stable Rust — monomorphized, no boxing.")
ROW_H = 0.48
th = (PAD + h_of("DOMAIN · TYPE · CATEGORICAL ROLE", MONO_MED, 11, tin, 1.25, 88)
      + 0.16 + len(math_rows) * ROW_H + 0.18
      + h_of(tfoot, SANS, 11.5, tin, 1.36) + PAD)
s.panel(ML, y, tw, th, fill=BG1, border=LINE1)
cy = s.eyebrow(ML + PAD, y + PAD, tin, "Domain  ·  Type  ·  Categorical role") + 0.16
C1, C2 = 1.58, 2.66
for dom, typ, role in math_rows:
    s.rule(ML + PAD, cy, tin, LINE1)
    s.text(ML + PAD, cy + 0.14, C1, dom, SANS_MED, 13, FG0, lead=1.3)
    s.text(ML + PAD + C1, cy + 0.15, C2, typ, MONO_MED, 11.5, ACCENT, lead=1.3)
    s.text(ML + PAD + C1 + C2, cy + 0.14, tin - C1 - C2, role, SANS, 12, FG1,
           lead=1.3)
    cy += ROW_H
s.rule(ML + PAD, cy, tin, LINE1)
s.text(ML + PAD, cy + 0.18, tin, tfoot, SANS, 11.5, FG2, lead=1.36)

rx = ML + tw + GAP
rw = CW - tw - GAP
rin = rw - 2 * PAD
b1 = "Tensor → MultiVector → Manifold → PropagatingEffect"
b1s = "Relativity, geometric algebra, topology, causal logic — one chain, no adapters."
h1 = (PAD + h_of("ONE BIND CHAIN", MONO_MED, 11, rin, 1.25, 88) + 0.12
      + h_of(b1, MONO_MED, 11, rin, 1.42, 8) + 0.10
      + h_of(b1s, SANS, 11.5, rin, 1.34) + PAD)
s.panel(rx, y, rw, h1, fill=BG2, border=LINE2, ret=True)
cy = s.eyebrow(rx + PAD, y + PAD, rin, "One bind chain") + 0.12
cy = s.text(rx + PAD, cy, rin, b1, MONO_MED, 11, ACCENT, lead=1.42, spc=8) + 0.10
s.text(rx + PAD, cy, rin, b1s, SANS, 11.5, FG1, lead=1.34)

b2 = "pub type FloatType = Float106;"
b2s = ("One alias flows through every contraction, rotation, manifold extension and monadic step. "
       "f32, f64, or 106-bit — switched in one line.")
h2 = (PAD + h_of("PRECISION AS A PARAMETER", MONO_MED, 11, rin, 1.25, 88) + 0.12
      + h_of(b2, MONO_MED, 11, rin, 1.4) + 0.10
      + h_of(b2s, SANS, 11.5, rin, 1.34) + PAD)
y2 = y + h1 + 0.24
s.panel(rx, y2, rw, h2, fill=BG2, border=LINE1)
cy = s.eyebrow(rx + PAD, y2 + PAD, rin, "Precision as a parameter") + 0.12
cy = s.text(rx + PAD, cy, rin, b2, MONO_MED, 11, ACCENT, lead=1.4) + 0.10
s.text(rx + PAD, cy, rin, b2s, SANS, 11.5, FG1, lead=1.34)

s.text(ML, max(y + th, y2 + h2) + 0.24, CW,
       "The GRMHD example runs exactly this: curvature feeds metric selection, which feeds a "
       "multivector Lorentz force, which feeds causal stability analysis — in one chain.",
       SANS, 12, FG1, lead=1.36)

emit(s, 7, "Uniform mathematics", """
One slide on the mathematics, as promised, then straight back to the product.

Most scientific computing stacks make you bridge silos. One library for tensors, another
for geometric algebra, a third for topology, and glue between them. You spend more time
on adapters than on the problem.

Here every layer implements the same categorical surface. A tensor is a functor. A
multivector is a monad. A manifold is a comonad. The propagating effect is a monad. fmap,
bind, extend, extract mean the same thing everywhere. That is done with arity-five
higher-kinded types on stable Rust using a witness pattern — so it monomorphizes, with no
boxing and no virtual calls.

Two payoffs. First, a single bind chain can step from general relativity through geometric
algebra onto topology and finish in causal logic. The GRMHD example does exactly that.

Second, precision becomes one type alias for the entire pipeline. Change one line and the
whole computation runs at f32, f64, or 106-bit — about thirty-two decimal digits, several
times faster than IEEE binary128.

That is the mathematics. Moving on.
""")

# ==========================================================================
# 08 — One causal stack
# ==========================================================================
s = S()
y = s.header("The stack", "One Causal Stack: From Discovery to Deployment",
             "One uniform substrate for the whole causal lifecycle — not five tools joined by a "
             "pipeline.")

layers = [
    ("Discover", "Causal Discovery Language",
     "Surface causal structure straight from raw observational data.",
     "SURD · MRMR · BRCD"),
    ("Model", "Causaloid & Context",
     "Encode that structure as composable causal logic with an explicit context.",
     "Singleton · collection · graph"),
    ("Act", "Causal State Machine",
     "Turn a causal verdict into an action and interact with the outside world.",
     "State inferred at runtime"),
    ("Govern", "Effect Ethos",
     "Verify deterministically what is allowed to happen.",
     "Defeasible deontic calculus"),
    ("Run", "Async Runtime",
     "Run the model thread-safe and asynchronously at scale.",
     "Embeds in any Tokio handler"),
]
px, pw = 0.98, W - 2 * 0.98
CA, CB, CC = 0.50, 4.10, 9.10
WA, WB, WC = 3.40, 4.80, pw - 9.10 - 0.32
band = max(max(h_of(st.upper(), MONO_MED, 11, WA, 1.25, 88) + 0.04
               + h_of(t, SANS_MED, 15.5, WA, 1.18, -19),
               h_of(b, SANS, 12.5, WB, 1.36),
               h_of(m, MONO, 9.5, WC, 1.4, 25)) + 0.30
           for st, t, b, m in layers)
ph = band * len(layers)
s.panel(px, y, pw, ph, gradient=(BG0, BG1), border=LINE1, radius=0.10)
s.reticle(px, y, pw, ph, arm=0.22)
for i, (stage, title, body, meta) in enumerate(layers):
    by = y + i * band
    if i:
        s.rule(px, by, pw, LINE1)
    s.tick(px + 0.22, by + band / 2 - 0.15, h=0.30, w=0.042)
    ty = s.text(px + CA, by + 0.15, WA, stage.upper(), MONO_MED, 11, ACCENT,
                lead=1.25, spc=88) + 0.04
    s.text(px + CA, ty, WA, title, SANS_MED, 15.5, FG0, lead=1.18, spc=-19)
    s.text(px + CB, by + 0.17, WB, body, SANS, 12.5, FG1, lead=1.36)
    s.text(px + CC, by + 0.19, WC, meta, MONO, 9.5, FG2, lead=1.4, spc=25)

s.text(ML, y + ph + 0.24, CW,
       "Every stage exchanges work through the same propagating effect, and the audit log "
       "accumulates across all five. You can enter at any layer and leave at any layer.",
       SANS, 12, FG1, lead=1.36)

emit(s, 8, "One causal stack", """
This is the slide I would ask you to remember.

Read it top to bottom. Discover: the Causal Discovery Language is a typestate builder DSL
that takes raw observational data and finds the structure worth modelling. SURD tells you
which variables are uniquely or synergistically causal; MRMR does feature selection; BRCD
ranks the root cause of a regime shift.

Model: that structure becomes a Causaloid with an explicit Context.

Act: the Causal State Machine turns a verdict into a proposed action.

Govern: the Effect Ethos decides whether that action is permissible.

Run: the whole thing is thread-safe by construction, so deployment is embedding the model
in an ordinary async request handler.

Two things make this one stack rather than five tools with a pipeline between them. First,
every stage exchanges work through the same propagating effect — there is no serialization
boundary anywhere in that column. Second, the audit log accumulates across all five, so the
explanation you hand a regulator spans discovery through deployment, not just the inference
step.

And you are not obliged to use all of it. You can enter at any layer and leave at any layer.
Plenty of users only ever touch Model and Act.
""")

# ==========================================================================
# 09 — Downstream: CFD
# ==========================================================================
s = S()
y = s.header("Downstream 01", "DeepCausality CFD — counterfactual fluid dynamics",
             "Released August 2026. Four of the six NASA CFD Vision 2030 grand challenges "
             "informed its design.")

lw = CW * 0.545
rw = CW - lw - GAP
lin, rin = lw - 2 * PAD, rw - 2 * PAD
cfd_bul = [
    ("Couples disciplines.",
     "Flow, plasma chemistry, navigation and control march in one typed process — not four "
     "codes exchanging files."),
    ("Forks the running simulation.",
     "March until a predicate fires, then fork the paused state in O(1) and fly every branch "
     "concurrently."),
    ("Carries its uncertainty.",
     "Ensembles and gates reduce a family of alternate worlds to one scored verdict."),
]
cfd_rows = [("SOD", "L1 ≤ 0.027 vs the exact Riemann solution"),
            ("CAVITY", "Ghia lid-driven cavity; graded MMS at order 2.00"),
            ("RAM-C II", "Peak electron density within ~2× of the flight anchor"),
            ("FORK", "Mid-burn fork departs frozen-drag by 139.4 m/s")]
lfoot = "cfd.deepcausality.com documents every solver, boundary and gate."
rfoot = "Every roadmap item ships with its measurement or its open question."
ph = max(PAD + h_of("WHAT IT DOES", MONO_MED, 11, lin, 1.25, 88) + 0.16
         + h_bullets(cfd_bul, lin, BODY, gap=0.16, lead=LEAD) + 0.18
         + h_of(lfoot, SANS, 11.5, lin, 1.32) + PAD,
         PAD + h_of("MEASURED, NOT ASSERTED", MONO_MED, 11, rin, 1.25, 88) + 0.14
         + h_rows(cfd_rows, rin, key_w=1.32, val_size=11.5, pad=0.12) + 0.16
         + h_of(rfoot, SANS, 11.5, rin, 1.32) + PAD)

s.panel(ML, y, lw, ph, fill=BG1, border=LINE2, ret=True)
cy = s.eyebrow(ML + PAD, y + PAD, lin, "What it does") + 0.16
cy = s.bullets(ML + PAD, cy, lin, cfd_bul, size=BODY, gap=0.16,
               lead=LEAD) + 0.18
s.text(ML + PAD, cy, lin, lfoot, SANS, 11.5, FG2, lead=1.32)

rx = ML + lw + GAP
s.panel(rx, y, rw, ph, fill=BG1, border=LINE1)
cy = s.eyebrow(rx + PAD, y + PAD, rin, "Measured, not asserted") + 0.14
cy = s.rows(rx + PAD, cy, rin, cfd_rows, key_w=1.32, val_size=11.5,
            pad=0.12) + 0.16
s.text(rx + PAD, cy, rin, rfoot, SANS, 11.5, FG2, lead=1.32)

by = y + ph + 0.26
s.text(ML, by, CW,
       "cfd.deepcausality.com  ·  every roadmap item states its measurement or its open question  "
       "·  two non-goals stated with their reasons", MONO, 10, FG2, lead=1.4, spc=30)

emit(s, 9, "Downstream: CFD", """
Two downstream projects, and I show them because they are the evidence that the substrate
generalizes beyond toy causality.

The first is counterfactual fluid dynamics. It shipped to crates.io this August and has its
own documentation site.

Three things it does. It couples disciplines — compressible flow, plasma chemistry,
navigation and control march inside one typed process rather than four codes exchanging
files. It forks the running simulation: march until a predicate fires, then fork the paused
state in constant time and fly every counterfactual branch concurrently. That is the causal
monad's intervene operator applied to a fluid solver. And it carries uncertainty through to
a scored verdict.

On the right is why I am comfortable saying that out loud. Sod shock tube to an L1 of 0.027
against the exact Riemann solution. Ghia cavity. RAM-C II peak electron density within about
a factor of two of the flight anchor, from an uncalibrated finite-rate network. And the
number that shows what forking buys: a mid-burn fork of the marched, plume-coupled state
departs a frozen-drag prediction by 139 metres per second.

The framing at the bottom: Teschner's survey of the six biggest unsolved problems in CFD,
drawn from NASA's CFD Vision 2030 study. Four of the six informed this design.

One thing I want to flag for this council specifically: the roadmap page also lists non-goals
with reasons. Distributed execution and GPU acceleration are both deferred, and we say why.
An unstated decision reads as a gap.
""")

# ==========================================================================
# 10 — Downstream: Quantum
# ==========================================================================
s = S()
y = s.header("Downstream 02", "DeepCausality Quantum — quantum causal models",
             "The same axiom, with quantum channels in place of conditional-probability tables.")

qq = ("A classical causal model factorizes a joint distribution over its graph. A quantum causal "
      "model factorizes a process operator into per-node Choi–Jamiołkowski operators.")
fh = PAD + 22 * 1.2 / 72.0 + 0.14 + h_of(qq, SANS, 11.5, CW - 2.6, 1.32) + PAD
s.panel(ML, y, CW, fh, fill=BG2, border=LINE2, ret=True)
cy = s.text(ML, y + PAD, CW, "σ  =  ∏_i  ρ_{A_i | Pa(A_i)}", MONO_MED, 22, ACCENT,
            lead=1.2, align="c", spc=15) + 0.14
s.text(ML + 1.3, cy, CW - 2.6, qq, SANS, 11.5, FG1, lead=1.32, align="c")

cy = y + fh + GAP
cw = (CW - 2 * 0.26) / 3
cin = cw - 0.28 - PAD
qcards = [
    ("Freeze-time validation",
     "Not every product of operators is a legal model. The quantum Markov condition — factors "
     "with intersecting Hilbert support must pairwise commute — is a gate at the graph freeze "
     "boundary. It never accepts a non-commuting model."),
    ("Orthomodular verdict",
     "The Birkhoff–von Neumann projection lattice carries the Verdict, with Born-rule read-out "
     "to a probability. Quantum logic is not bolted on beside the causal engine; it is a verdict "
     "type inside it."),
    ("Two modalities, kept apart",
     "A verifiable default path backed by Lean proofs, and a physical-QPU seam behind a feature "
     "flag. Nesting is deliberately not offered: it is unestablished, and we would rather ship "
     "the flat model honestly."),
]
ch = max(PAD + h_of(h, SANS_MED, 14.5, cin, 1.2, -15) + 0.12
         + h_of(b, SANS, 11.5, cin, LEAD) + PAD for h, b in qcards)
for i, (head, body) in enumerate(qcards):
    cx = ML + i * (cw + 0.26)
    s.panel(cx, cy, cw, ch, fill=BG1, border=LINE1)
    s.tick(cx, cy + PAD + 0.02, h=0.30, w=0.038)
    ty = s.text(cx + 0.28, cy + PAD, cin, head, SANS_MED, 14.5, FG0, lead=1.2,
                spc=-15) + 0.12
    s.text(cx + 0.28, ty, cin, body, SANS, 11.5, FG1, lead=LEAD)

s.text(ML, cy + ch + 0.24, CW,
       "Gate kernels — Born probability, expectation value, commutator, fidelity — each wrap into "
       "a PropagatingEffect, so quantum mechanics enters the same causal chain as everything else. "
       "R. Lorenz (2022), Definition 3.3.", SANS, 11.5, FG2, lead=1.36)

emit(s, 10, "Downstream: quantum", """
The second downstream project, and this one tests the claim that the axiom is genuinely
spacetime-agnostic.

A classical causal model factorizes a joint distribution over its graph. A quantum causal
model — Lorenz, 2022 — does the same for a process operator, factorized into per-node
Choi–Jamiołkowski operators. The conditional-probability tables become quantum channels,
and the model is their product.

The interesting engineering is the first card. Not every product of operators is a legal
quantum causal model: factors whose Hilbert supports intersect have to pairwise commute. We
make that a freeze-time gate. When you freeze the graph, we embed each intersecting pair
onto its common support, form the commutator, and compare against a depth-aware tolerance.
The check is sound — it never accepts a non-commuting model — and it may be incomplete. A
failure names the exact offending pair and rolls the graph back.

Second card: the projection lattice carries the Verdict, with Born-rule read-out. So quantum
logic is not sitting beside the causal engine; it is a verdict type inside it.

Third card, and this is a governance point as much as a technical one. There are two
modalities and we keep them apart: a verifiable path backed by Lean proofs, and a
physical-QPU seam behind a feature flag. We also deliberately do not offer nested quantum
causal models, because that is unestablished in the literature. We would rather ship the flat
model honestly than ship something that looks more general than it is.
""")

# ==========================================================================
# 11 — Case study (content deliberately blank)
# ==========================================================================
s = S()
y = s.header("Case study", "DeepCausality at United Airlines",
             "via the Service Radar integration")
ph = CONTENT_FLOOR - y - 0.04
s.panel(ML, y, CW, ph, fill=BG1, border=LINE1, ret=True)
s.text(ML, y + ph / 2 - 0.30, CW, "[  content to follow  ]", MONO, 13, FG2,
       lead=1.3, align="c", spc=100)
s.divider(W / 2 - 1.30, y + ph / 2 + 0.20, 2.60)

emit(s, 11, "Case study", """
PLACEHOLDER — content to be written.

Suggested shape when you fill it in, so the slide matches the rest of the deck: one line on
what Service Radar is and where DeepCausality sits inside it; one line on what the deployment
does that a correlational monitor cannot; and one measured number. Keep it to three claims.

Timing: budget roughly 90 seconds here, the longest single block in the deck. If it runs
short on the day, spend the recovered time on the causal-stack slide.
""")

# ==========================================================================
# 12 — Verification
# ==========================================================================
s = S()
y = s.header("Verification", "Laws proved in Lean, checked again in Rust",
             "No tool turns a Lean proof into a Rust test, so each property is stated twice and "
             "bound by a shared id.")

sw = (CW - 2 * 0.26) / 3
sin = sw - 2 * PAD
stats = [("183", "THEOREMS PROVED", "Lean 4 against Mathlib, pinned at v4.32.0"),
         ("0", "SORRY", "No unproved statement in the gated tree"),
         ("11", "KANI HARNESSES", "Bounded model checking on the core carrier")]
sh = max(PAD * 0.8 + 34 / 72.0 + 0.12 + h_of(l, MONO_MED, 9.5, sin, 1.3, 76)
         + 0.07 + h_of(sub, SANS, 11.5, sin, 1.3) + PAD * 0.8
         for _, l, sub in stats)
for i, (big, lab, sub) in enumerate(stats):
    cx = ML + i * (sw + 0.26)
    s.panel(cx, y, sw, sh, fill=BG1, border=LINE2)
    cy = s.text(cx + PAD, y + PAD * 0.8, sin, big, SANS_MED, 34, ACCENT,
                lead=1.0, spc=-40) + 0.12
    cy = s.text(cx + PAD, cy, sin, lab, MONO_MED, 9.5, FG0, lead=1.3,
                spc=76) + 0.07
    s.text(cx + PAD, cy, sin, sub, SANS, 11.5, FG1, lead=1.3)

cy = y + sh + GAP
lw = CW * 0.52
rw = CW - lw - GAP
lin, rin = lw - 2 * PAD, rw - 2 * PAD
layers_txt = "Num · Algebra · Haft · Core · Complex & Dual · Rational · Topology · Quantum"
lbody = ("Lean proves the statement — deductive, unbounded, higher-order. A Rust witness checks it "
         "independently: a law-test for the algebraic layers, a Kani harness for the core carrier. "
         "The monad laws, the Kleisli category laws and the causaloid fixpoint sit in here.")
edges = [
    "Float106 bit-exact error bounds are open. Lean proves the real-field laws; the empirical "
    "bounds are Rust tests only.",
    "Octonions sit outside the proved layer — Mathlib does not carry them.",
    "The unconditional quantum partial-trace preservation is proved FALSE, with a witnessed "
    "counterexample. Only the conditional version holds.",
]
ph = max(PAD + h_of("THE LAYERS UNDER PROOF", MONO_MED, 11, lin, 1.25, 88) + 0.14
         + h_of(layers_txt, MONO_MED, 11, lin, 1.5, 12) + 0.18 + 0.18
         + h_of(lbody, SANS, 11.5, lin, LEAD) + PAD,
         PAD + h_of("THE EDGES WE PUBLISH", MONO_MED, 11, rin, 1.25, 88) + 0.16
         + h_bullets(edges, rin, 11.5, gap=0.14, lead=1.38) + PAD)

s.panel(ML, cy, lw, ph, fill=BG1, border=LINE1)
ly = s.eyebrow(ML + PAD, cy + PAD, lin, "The layers under proof") + 0.14
ly = s.text(ML + PAD, ly, lin, layers_txt, MONO_MED, 11, ACCENT, lead=1.5,
            spc=12) + 0.18
s.rule(ML + PAD, ly, lin, LINE1)
s.text(ML + PAD, ly + 0.18, lin, lbody, SANS, 11.5, FG1, lead=LEAD)

rx = ML + lw + GAP
s.panel(rx, cy, rw, ph, fill=BG2, border=LINE2, ret=True)
ry = s.eyebrow(rx + PAD, cy + PAD, rin, "The edges we publish", color=WARN) + 0.16
s.bullets(rx + PAD, ry, rin, edges, size=11.5, gap=0.14, dot_color=WARN,
          lead=1.38)

emit(s, 12, "Verification", """
This is where I would spend your scepticism, so let me be specific.

183 theorems machine-checked in Lean 4 against Mathlib, on a pinned toolchain. Zero sorry —
no unproved statement anywhere in the gated tree. Eleven Kani harnesses doing bounded model
checking on the core carrier.

The bridge matters more than the count. There is no tool that turns a Lean proof into a Rust
test. So each property is stated twice — once in Lean, once as a Rust witness — and both
carry the same identifier. CI fails if an identifier is missing either side. That means the
proofs cannot silently drift away from the code, which is the usual failure mode of
formalization efforts.

Now the right-hand panel, which is the part I actually want you to notice.

We publish the edges. The Float106 bit-exact error bounds are open — Lean proves the
real-field laws, the empirical bounds are Rust tests only. Octonions are outside the proved
layer because Mathlib does not carry them.

And the third one: in the quantum layer, the unconditional partial-trace preservation property
is proved false. Not unproven — false, with a witnessed counterexample. Only a conditional
boundary version holds. We found that by trying to prove it, and we publish it as a headline
result rather than quietly narrowing the claim.

That is the standard I would like the project judged against.
""")

# ==========================================================================
# 13 — By the numbers
# ==========================================================================
s = S()
y = s.header("Project health", "Where the project stands in 2026",
             "In September 2023, at the point of the Sandbox proposal, the repository held four "
             "crates.")

tiles = [("29", "CRATES", "Four in 2023"),
         ("438k", "LINES OF RUST", "20.7k in 2023"),
         ("10,967", "TESTS", "534 in 2023"),
         ("113", "RUNNABLE EXAMPLES", "Across 14 domains"),
         ("183", "LEAN THEOREMS", "Bound to Rust witnesses"),
         ("23 / 29", "ZERO-DEPENDENCY CRATES", "No external runtime deps")]
tw = (CW - 2 * 0.24) / 3
tin = tw - 0.28 - PAD
th = max(PAD * 0.72 + 32 / 72.0 + 0.10 + h_of(l, MONO_MED, 9.5, tin, 1.3, 76)
         + 0.06 + h_of(sub, SANS, 11, tin, 1.3) + PAD * 0.72
         for _, l, sub in tiles)
for i, (big, lab, sub) in enumerate(tiles):
    cx = ML + (i % 3) * (tw + 0.24)
    ty = y + (i // 3) * (th + 0.24)
    s.panel(cx, ty, tw, th, fill=BG1, border=LINE1)
    s.tick(cx, ty + PAD * 0.72 + 0.05, h=0.30, w=0.038)
    cy = s.text(cx + 0.28, ty + PAD * 0.72, tin, big, SANS_MED, 32, FG0,
                lead=1.0, spc=-38) + 0.10
    cy = s.text(cx + 0.28, cy, tin, lab, MONO_MED, 9.5, ACCENT, lead=1.3,
                spc=76) + 0.06
    s.text(cx + 0.28, cy, tin, sub, SANS, 11, FG1, lead=1.3)

by = y + 2 * (th + 0.24) + 0.02
bh_t = "Two build systems, one workspace"
bh_b = ("Cargo and Bazel both build and test the full tree, and CI checks that no Cargo example is "
        "missing its Bazel target. unsafe_code = \"forbid\" is set repo-wide, with three exemptions, "
        "each carrying a written justification.")
bh = (PAD * 0.72 + h_of(bh_t, SANS_MED, 15, CW - 2 * PAD, 1.22, -17) + 0.09
      + h_of(bh_b, SANS, 11.5, CW - 2 * PAD, 1.32) + PAD * 0.72)
s.panel(ML, by, CW, bh, fill=BG2, border=LINE2)
s.tick(ML, by + PAD * 0.72 + 0.02, h=0.32, w=0.045)
cy = s.text(ML + PAD, by + PAD * 0.72, CW - 2 * PAD, bh_t, SANS_MED, 15, FG0,
            lead=1.22, spc=-17) + 0.09
s.text(ML + PAD, cy, CW - 2 * PAD, bh_b, SANS, 11.5, FG1, lead=1.32)

emit(s, 13, "By the numbers", """
Quickly, because numbers are only context.

When I stood here in 2023, the repository held four crates, about twenty thousand lines, and
five hundred tests. Today: twenty-nine crates, four hundred and thirty-eight thousand lines,
and just under eleven thousand tests. Roughly a twenty-one-fold increase on both lines and
tests — so test density held while the codebase grew. That was deliberate.

A hundred and thirteen runnable examples across fourteen domains: avionics, physics, medicine,
materials, quantum, classical causality, and so on. Every example is a build target in both
build systems, and CI fails if a Cargo example is missing its Bazel target, so the examples
cannot rot.

Twenty-three of twenty-nine crates have no external runtime dependency at all. The six that do
are narrow and mostly optional. For a project aimed at regulated domains, that supply-chain
surface is a feature, not an accident.

And unsafe is forbidden repo-wide by a workspace lint. Three crates are exempt; each exemption
carries a written, irreducible justification, and two of them are marked for removal when the
compiler limitation behind them is fixed.
""")

# ==========================================================================
# 14 — Governance, community, sponsors
# ==========================================================================
s = S()
y = s.header("Project", "How the project is run and supported")

lw = CW * 0.615
rw = CW - lw - GAP
lin, rin = lw - 2 * PAD, rw - 2 * PAD
gov_rows = [
    ("FOUNDATION", "LF AI & Data Sandbox project since September 2023"),
    ("LICENSE", "MIT, across every crate in the workspace"),
    ("SECURITY", "OpenSSF Best Practices badge, security policy, Miri in CI"),
    ("SUPPLY CHAIN", "OSV-Scanner on every pull request, merge group and push"),
    ("QUALITY", "Codecov, CodeFactor, clippy and rustfmt gates on every PR"),
    ("RELEASES", "release-plz publishes from CI and writes the changelog"),
    ("PROOFS", "CI fails if a Lean theorem id has lost its Rust witness"),
]
gfoot = "Contributing guide, code of conduct, and a written policy for AI coding assistants."
lh = (PAD + h_of("GOVERNANCE AND POSTURE", MONO_MED, 11, lin, 1.25, 88) + 0.16
      + h_rows(gov_rows, lin, key_w=1.86, val_size=11.5, pad=0.13) + 0.18
      + h_of(gfoot, SANS, 11.5, lin, 1.32) + PAD)
s.panel(ML, y, lw, lh, fill=BG1, border=LINE1)
cy = s.eyebrow(ML + PAD, y + PAD, lin, "Governance and posture") + 0.16
cy = s.rows(ML + PAD, cy, lin, gov_rows, key_w=1.86, val_size=11.5,
            pad=0.13) + 0.18
s.text(ML + PAD, cy, lin, gfoot, SANS, 11.5, FG2, lead=1.32)

rx = ML + lw + GAP
c1 = "Discord  ·  GitHub Discussions  ·  LF lists"
c2 = "deepcausality.com  ·  cfd.deepcausality.com"
h1 = (PAD + h_of("COMMUNITY", MONO_MED, 11, rin, 1.25, 88) + 0.16
      + h_of(c1, SANS, 12.5, rin, 1.34) + 0.12
      + h_of(c2, MONO_MED, 10.5, rin, 1.34, 15) + PAD)
s.panel(rx, y, rw, h1, fill=BG1, border=LINE1)
cy = s.eyebrow(rx + PAD, y + PAD, rin, "Community") + 0.16
cy = s.text(rx + PAD, cy, rin, c1, SANS, 12.5, FG0, lead=1.34) + 0.12
s.text(rx + PAD, cy, rin, c2, MONO_MED, 10.5, ACCENT, lead=1.34, spc=15)

# Sponsors — the logo column is fixed so both marks share one optical axis.
sy = y + h1 + GAP
LOGO_COL = 1.22
sponsors = [
    ("jetbrains.png", 0.72, 801 / 800, "JetBrains",
     "All-product licences for core maintainers"),
    ("center_emblem_cream.png", 0.86, 1090 / 1294, "Center for Dynamic Causality",
     "Ongoing research and resources"),
]
srow = []
for _, lw_i, ar, nm, sub in sponsors:
    srow.append(max(lw_i * ar + 0.10,
                    h_of(nm, SANS_MED, 12.5, rin - LOGO_COL, 1.24, -13)
                    + 0.04 + h_of(sub, SANS, 11, rin - LOGO_COL, 1.3)))
h2 = (PAD + h_of("SPONSORS", MONO_MED, 11, rin, 1.25, 88) + 0.18
      + srow[0] + 0.22 + srow[1] + PAD)
s.panel(rx, sy, rw, h2, fill=BG1, border=LINE2, ret=True)
cy = s.eyebrow(rx + PAD, sy + PAD, rin, "Sponsors") + 0.18
for (fn, lwi, ar, nm, sub), rh in zip(sponsors, srow):
    s.image(os.path.join(ASSETS, fn), rx + PAD + (LOGO_COL - 0.28 - lwi) / 2,
            cy + (rh - lwi * ar) / 2, lwi, lwi * ar)
    blk = (h_of(nm, SANS_MED, 12.5, rin - LOGO_COL, 1.24, -13) + 0.04
           + h_of(sub, SANS, 11, rin - LOGO_COL, 1.3))
    ty = s.text(rx + PAD + LOGO_COL, cy + (rh - blk) / 2, rin - LOGO_COL, nm,
                SANS_MED, 12.5, FG0, lead=1.24, spc=-13) + 0.04
    s.text(rx + PAD + LOGO_COL, ty, rin - LOGO_COL, sub, SANS, 11, FG1,
           lead=1.3)
    cy += rh + 0.22

emit(s, 14, "Governance and support", """
Briefly, on how the project is actually run.

Sandbox project at LF AI & Data since September 2023. MIT licensed across every crate — no
dual licensing, no contributor licence agreement friction. OpenSSF Best Practices badge, a
published security policy, OSV-Scanner on every pull request and push, Miri in CI where it is
meaningful, and coverage and lint gates on every pull request.

The bar at the bottom is the part I would point at if you are assessing maturity. Releases are
not manual: release-plz opens the release pull request and publishes every unpublished crate
from CI, and writes the changelog. And the formalization workflow fails the build if a Lean
theorem id has lost its Rust witness, or the reverse — so the proof layer cannot quietly rot.

There is also a written policy for AI coding assistants in the repository, which is not yet
common and which I am happy to discuss separately — several of you have raised that question
in other projects.

Community runs through Discord, GitHub Discussions, and the LF mailing lists.

Two sponsors. JetBrains provides all-product licences to core maintainers, and has renewed
that. And the Center for Dynamic Causality — the organization I direct — contributes ongoing
research and resources. I want to be transparent about that relationship rather than have it
discovered: the Center funds research that lands in the project under the project's own MIT
licence and open governance.
""")

# ==========================================================================
# 15 — Close
# ==========================================================================
s = S()
s.eyebrow(ML, Y_EYEBROW, CW, "Collaborate")
y = s.title(ML, Y_TITLE, CW, "Where the project would value help")
s.rule(ML, y + 0.20, CW, LINE1)
y = y + 0.20 + 0.32

cw = (CW - 2 * 0.26) / 3
cin = cw - 0.28 - PAD
asks = [
    ("Domain partners",
     "The stack is strongest where a regulated domain needs an audit trail: avionics, medicine, "
     "industrial control, finance. A partner with a real decision boundary is worth more than "
     "another benchmark."),
    ("Adjacent LF projects",
     "The propagating effect is a natural seam for anything that already produces a verdict and "
     "then has to explain it. If your project has that shape, the integration surface is small."),
    ("Reviewers and provers",
     "The Lean layer publishes its open edges. Extending the proved surface — particularly the "
     "quantum foundation — is well-scoped, self-contained work."),
]
ch = max(PAD + h_of(h, SANS_MED, 15.5, cin, 1.2, -17) + 0.14
         + h_of(b, SANS, 12, cin, 1.44) + PAD for h, b in asks)
for i, (head, body) in enumerate(asks):
    cx = ML + i * (cw + 0.26)
    s.panel(cx, y, cw, ch, fill=BG1, border=LINE1)
    s.tick(cx, y + PAD + 0.02, h=0.30, w=0.038)
    ty = s.text(cx + 0.28, y + PAD, cin, head, SANS_MED, 15.5, FG0, lead=1.2,
                spc=-17) + 0.14
    s.text(cx + 0.28, ty, cin, body, SANS, 12, FG1, lead=1.44)

cy = y + ch + 0.62
s.divider(ML, cy, CW)
cy = s.title(0, cy + 0.34, W, "Thank you", size=28, align="c") + 0.24
cy = s.text(0, cy, W, "Marvin Hansen  ·  Director, Center for Dynamic Causality",
            SANS, 12.5, FG1, lead=1.3, align="c") + 0.08
s.text(0, cy, W, "github.com/deepcausality-rs/deep_causality   ·   deepcausality.com",
       MONO_MED, 10.5, ACCENT, lead=1.3, align="c", spc=25)

emit(s, 15, "Close", """
Three places where help would compound, and then I will stop.

First, domain partners. The stack is strongest where a regulated domain needs an audit trail —
avionics, medicine, industrial control, finance. One partner with a real decision boundary
teaches us more than another benchmark does.

Second, adjacent LF projects. The propagating effect is a natural seam for anything that already
produces a verdict and then has to explain it. If your project has that shape, the integration
surface is genuinely small, and I would rather find that out in a fifteen-minute call than in a
design document.

Third, reviewers and provers. The Lean layer publishes its open edges deliberately. Extending
the proved surface — especially the quantum foundation, which we built from first principles
because Mathlib does not carry it — is well-scoped, self-contained work for anyone who enjoys
that.

Thank you. Happy to take questions, and happy to go deeper on any slide.
""")

# ==========================================================================
prs = Presentation()
prs.slide_width = Inches(W)
prs.slide_height = Inches(H)
for sl in SLIDES:
    scene_to_slide(sl.sc, prs)

os.makedirs(OUT_DIR, exist_ok=True)
prs.save(os.path.join(OUT_DIR, "DeepCausality_TAC_2026.pptx"))

os.makedirs(PREVIEW, exist_ok=True)
for i, sl in enumerate(SLIDES, 1):
    scene_to_png(sl.sc, os.path.join(PREVIEW, f"{i:02d}.png"), W, H, ppi=110)

md = ["# DeepCausality — Speaker Notes", "",
      "LF AI & Data Technical Advisory Council, 2026.  ",
      "Marvin Hansen · Director, Center for Dynamic Causality.", "",
      "Deck: `DeepCausality_TAC_2026.pptx` · 15 slides · budgeted 15–20 minutes.", "",
      "The same text is embedded in the PowerPoint notes pane, so Presenter View shows it too.",
      "", "---", ""]
for n, title, body in NOTES:
    md += [f"## Slide {n:02d} — {title}", "", body.strip(), "", "---", ""]
with open(os.path.join(OUT_DIR, "SPEAKER_NOTES.md"), "w") as fh:
    fh.write("\n".join(md).rstrip() + "\n")

print("deck + previews + notes written." if OK[0] else "!! OVERFLOWS REMAIN")
