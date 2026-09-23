# DeepCausality — Design Board

Extracted from the running implementation at `website/web/`.

## What this is

A design board: the visual system as it exists in code today.

`DESIGN.md` is a *specification* and states what the site should be. This board
is a *description* and states what the site is, with measurements. Confusing the
two is how design systems rot.

## Division of authority

`DESIGN.md` agrees with the implementation except where
[08-drift.md](08-drift.md) records otherwise. Each document owns different ground:

| Question | Authority |
|---|---|
| What *should* a component do? | `DESIGN.md` |
| Which convention is canonical? | `DESIGN.md` §12 |
| Is something banned? | `DESIGN.md` §13 |
| What does the code *actually* do today? | This board |
| What are the measured contrast ratios? | [01-foundations.md](01-foundations.md) |
| Where is each idiom used, and how often? | [06-idioms.md](06-idioms.md) |
| What is still unreconciled? | [08-drift.md](08-drift.md) |

The board does not restate the conventions. `DESIGN.md` §12 declares one form
each; [06-idioms.md](06-idioms.md) points there and adds only a usage census.

## Sections

| File | Covers |
|---|---|
| [01-foundations.md](01-foundations.md) | Brand anchor, both palettes, measured contrast |
| [02-typography.md](02-typography.md) | Families, scale, weights, measure |
| [03-space-layout.md](03-space-layout.md) | Spacing scale, containers, breakpoints |
| [04-surface.md](04-surface.md) | Borders, radii, shadows, elevation |
| [05-motion.md](05-motion.md) | Durations, easing, reduced-motion contract |
| [06-idioms.md](06-idioms.md) | The recurring visual moves that carry the style |
| [07-components.md](07-components.md) | Per-component anatomy as built |
| [08-drift.md](08-drift.md) | Every point where code and spec diverge |

## Source of truth, in order

1. `website/web/src/styles/tokens.css` — the token values themselves.
2. `website/web/src/styles/global.css` — base elements plus the shared idiom layer.
3. Per-component `<style>` blocks in `website/web/src/`.
4. `website/web/DESIGN.md` — intent, and the arbiter when code looks accidental.

## Using this board

When you add a component, read [06-idioms.md](06-idioms.md) first. Four or five
repeated moves carry the site's character; a component that uses every token
correctly and none of the idioms still looks foreign.

When you change a token, check [08-drift.md](08-drift.md). Several values are
hand-copied into `website/docs/src/styles/theme.css`, and that copy does not
update itself.

## Verification

Contrast ratios in [01-foundations.md](01-foundations.md) are computed from the
shipped hex values with the WCAG 2.1 relative-luminance formula, by a calculator
checked against known reference pairs (`#767676` on white = 4.54, `#1976d2` on
white = 4.60). `DESIGN.md` §2 and §2.1 carry the measured ratios and flag both AA
failures; see [08-drift.md](08-drift.md) §1.

Prose in this board follows `docs/writing_guides/`.
