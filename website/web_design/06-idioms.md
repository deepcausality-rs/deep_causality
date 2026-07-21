# 06 — Idioms

**The canonical statement of every convention now lives in `website/web/DESIGN.md` §12.** That section is normative: it declares one form per convention and lists the variants as debt. Read it before building anything.

This page does not restate those rules. It records what the board can measure and the spec does not: where each idiom is actually used, and how often.

## Usage census

Counted across `website/web/src/`.

| Idiom | DESIGN.md | Components using it |
|---|---|---|
| Eyebrow | §12.1 | 9 files redeclare it locally on top of the global rule |
| Coordinate eyebrow | §12.2 | Global rule only |
| Reticle corners | §12.3 | 6 — Hero, CausalStack, Explainer, ExampleGrid, ExampleCard, JoinCommunity |
| HUD gradient panel | §12.4 | 4 verbatim + 1 variant (ExampleDetail, `--bg-2` at 80%) |
| L-bracket corners | §12.5 | 3, in 3 implementations at 2 sizes |
| Text arrows | §12.6 | 6 glyph forms across ~15 sites |
| Pill chip | §12.8 | 4 — CausalStack, ExampleDetail, `examples/index`, `overview` |
| Focus ring | §12.9 | 1 global + 3 local overrides at 3 offsets |
| Section divider | §12.10 | 2 — SectionDivider, CausalStack (via `data-anim-draw`) |
| Hairline list | §12.12 | 3 verbatim copies |
| Accent wash | §12.11 | 6 distinct percentages: 5, 8, 12, 32, 38, 55 |
| `font-weight: 540` | §12.7 | 6 literal occurrences, no token |

Reticle corners are the most-used idiom and the site's signature. The eyebrow is the most-diverged.

## What the census implies

Three idioms account for most of the visual identity: reticle corners, the HUD gradient panel, and the section divider. A new panel that takes all three reads as native immediately.

The idioms with the most drift are the cheapest to fix. The eyebrow needs nine deletions and no new code. The focus ring needs three. Neither changes a pixel of intended output; both remove the chance of future divergence.

The accent wash is the opposite case. Six percentages exist and only two are conventional (5% hover, 8% active). The other four are one-off decorative halos in Hero and ExampleCard, documented in [04-surface.md](04-surface.md). Those are legitimate, and collapsing them to the interaction values would be wrong.

## Mechanism notes

Two implementation details worth knowing before touching the idiom layer.

**One observer, site-wide.** `BaseLayout.astro:97-114` runs a single IntersectionObserver tuned `rootMargin: '0px 0px -10% 0px', threshold: 0.15` that adds `.in-view`. Components opt in with `data-anim-draw` and read `.in-view`. Do not add a second observer.

**`pathLength="100"` is what makes the draw-in work.** Setting it on `<line>` and `<path>` normalizes the dash math so `stroke-dasharray: 100` behaves identically regardless of the element's real geometry. Omit it and the animation breaks on any element whose path length isn't 100.
