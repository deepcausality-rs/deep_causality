<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality — LF AI & Data TAC, 2026

Deck for the Technical Advisory Council slot. A general overview and
re-introduction: the council has largely turned over since the 2023 Sandbox
proposal in [`../LF_2023/`](../LF_2023/), and the project is no longer the same
shape.

| File | What it is |
|---|---|
| `DeepCausality_TAC_2026.pptx` | The deck. 15 slides, 16:9, notes on every slide. |
| `SPEAKER_NOTES.md` | The same notes as a standalone document, mirroring the 2023 `DC_Notes.pdf`. |
| `contact_sheet.png` | All 15 slides at a glance, without opening PowerPoint. |
| `build/` | The generator. Re-run it to rebuild the deck. |

Budgeted at 15–20 minutes, roughly a minute per slide. Slide 11 is a deliberate
placeholder: the United Airlines / Service Radar case study, content to follow.

## Fonts

The deck is set in **Geist** and **JetBrains Mono**, the two families the project
website uses. Both are installed in `~/Library/Fonts` on the authoring machine.
Any machine that presents this file needs them too, or PowerPoint will
substitute and the layout will shift. Both are free:

- Geist — <https://github.com/vercel/geist-font/releases>
- JetBrains Mono — <https://www.jetbrains.com/lp/mono/>

## Design

Palette, type scale and idioms are transcribed from
[`website/web_design/`](../../../website/web_design/): one accent sampled from
the hero art, hairline borders rather than soft-card shadows, mono uppercase
tracked eyebrows, and the reticle corners that are the site's signature.

One deliberate deviation: `01-foundations.md` measures `--fg-2` at 4.26:1 on
`--bg-0` and calls it "a decorative grey, not a text grey". Nothing a reader has
to read uses it here — eyebrows take `--accent` (11.24:1) instead. `--fg-2`
survives only on hairline meta.

## Rebuilding

```bash
pip install python-pptx pillow fonttools
cd build && python3 build2.py
```

The build writes the `.pptx` and `SPEAKER_NOTES.md` back into this directory,
plus per-slide PNG previews into `build/preview/`.

Three properties are worth knowing before editing the source:

**Text is measured, never estimated.** `scene.py` measures every string against
the real TTF, pre-wraps it to a hard line list, and sets exact point line
spacing. A block's height is `len(lines) * leading`, so stacking is arithmetic.
Every text helper in `deckkit.py` returns the y coordinate *after* the block it
drew.

**The build fails loudly on overflow.** `S.check()` compares the lowest point a
slide reaches against the footer rule and prints `!! OVERFLOW` with the
measurement. If you lengthen copy, the build tells you which slide broke and by
how much.

**The preview cannot drift from the deck.** Both the `.pptx` and the PNGs render
from the same scene graph, so what the preview shows is what the file contains.

`glyphcheck.py` reports any character a slide asks for that its font does not
carry — worth running after adding mathematical notation, since JetBrains Mono
has no U+1D62 (subscript i).
