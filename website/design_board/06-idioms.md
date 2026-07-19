# 06 — Idioms

Read this section before building anything. The site's character comes from
roughly nine repeated moves, and none of them are in the token table. A component
that uses every token correctly and none of these idioms will still look foreign.

The unifying theme is instrumentation. Ticks, brackets, coordinate indices,
node dots, hairline rules. The page should read like a technical instrument
panel rather than a marketing surface. `global.css` calls this the "futurism
layer". `DESIGN.md` does not mention it at all, which is the single largest gap
between the spec and the site.

## 1. The eyebrow tick

The most-used idiom on the site. A small cyan dash preceding a mono uppercase
label.

Base rule, `global.css:104-116`:

```css
.eyebrow {
  font-family: var(--font-mono);
  display: flex;
  align-items: center;
  gap: 0.55em;
}
.eyebrow::before {
  content: '';
  flex: 0 0 12px;
  height: 1px;
  background: var(--accent);
  opacity: 0.7;
}
```

The tick is what turns a small heading into an instrument annotation. Without it
the mono uppercase label reads as an ordinary overline.

Use the global `.eyebrow` class. Do not redeclare it locally; nine files
currently do, and they disagree on `font-family` and `letter-spacing`. See
[08-drift.md](08-drift.md) §5.

## 2. The coordinate eyebrow

A variant that splits the label into a dim index and a brighter name, so a
section reads as telemetry. `global.css:121-129`:

```css
.eyebrow-coord { display: inline-flex; gap: 0.6em; align-items: baseline; letter-spacing: 0.06em; }
.eyebrow-coord .ix { color: var(--fg-2); }
.eyebrow-coord .lb { color: var(--fg-1); }
```

Worn alongside `.eyebrow`. Renders as `— 01 / 04  MODEL`.

## 3. Reticle corners

Four L-shaped brackets pinned to the corners of a panel, drawn in accent at 55%
opacity, rising to full on hover. `global.css:131-145`.

```html
<div class="reticle-host">
  <span class="reticle reticle-tl"></span>
  <span class="reticle reticle-tr"></span>
  <span class="reticle reticle-bl"></span>
  <span class="reticle reticle-br"></span>
</div>
```

Each corner is a 12px square with two borders removed and one radius set, so it
traces the panel's own corner rather than sitting inside it. Offsets are `-1px`
so the bracket overlaps the host's 1px border exactly.

Used by six components: Hero, CausalStack, Explainer, ExampleGrid, ExampleCard,
JoinCommunity. This is the site's signature move.

The radius must match the host. Hero overrides all four to `--radius-lg`
(`Hero.astro:125-128`) because its frame is the one `--radius-lg` element on the
page.

## 4. The HUD gradient panel

The standard framed container. Four components copy it verbatim:

```css
background-image: linear-gradient(180deg,
  color-mix(in srgb, var(--bg-1) 70%, transparent) 0%,
  var(--bg-1) 100%);
border: 1px solid var(--line-1);
border-radius: var(--radius-md);
box-shadow: var(--shadow-1);
```

The gradient runs from partly-transparent `--bg-1` at the top to solid at the
bottom, so a panel appears to catch light along its upper edge. It is subtle by
design and does not contradict the ban on gradient *backgrounds*; this is a
surface treatment on a bounded panel, not a page background.

`ExampleDetail.astro:177-184` uses the same shape with `--bg-2` at 80%.

## 5. L-bracket corner accents

Distinct from reticles. Two or four corner marks drawn in
`color-mix(in srgb, var(--accent) 60%, var(--line-2))` at 10×10, used to frame a
code surface rather than a whole panel.

Three components implement this, in three different ways: pseudo-elements at 10px
(`ExampleGrid.astro:330-347`, `overview/index.astro:407-418`) and SVG data-URI
masks at 14px (`ExampleDetail.astro:202-221`). Same motif, three code paths.
Consolidation candidate; see [08-drift.md](08-drift.md) §6.

## 6. Hairline list with slide-on-hover

The standard list treatment. No bullets, no cards. Rows separated by hairlines,
and the link slides right on hover.

```css
li + li { border-top: 1px solid var(--line-1); }
a { transition: padding-left var(--dur-med) var(--ease-out); }
a:hover { padding-left: var(--space-3); }
```

Three verbatim copies: `ExamplesList.astro:33-48`, `blog/index.astro:191-199`,
`blog/[...slug].astro:187-196`.

Note this animates `padding-left`, which [05-motion.md](05-motion.md) lists as
banned. The exception is deliberate and narrow: a single text link in a
hairline row is cheap to reflow. Do not generalize it to a card or a grid item.

## 7. Accent wash on hover

Interactive surfaces tint toward the accent rather than changing to a new
background colour:

```css
background: color-mix(in srgb, var(--accent) 5%, transparent);
```

Six percentages are in use across the site: 5%, 8%, 12%, 32%, 38%, 55%. Prefer
**5%** for a row or tab hover and **8%** for a selected or active state. Those two
cover most cases and are already the most common.

## 8. Text arrows, never icons

Directional affordances are literal glyphs with `aria-hidden="true"`, never SVG
icons and never emoji.

| Glyph | Meaning |
|---|---|
| `→` | Forward, open, continue |
| `↗` | External link, leaves the site |
| `←` | Back |
| `▼` | Expand, flow-downward in diagrams |
| `·` | Separator inside a meta line |

## 9. Scroll-triggered draw-in

SVG hairlines draw themselves once on entry, then node dots pop in staggered.

The mechanism: `BaseLayout.astro:97-114` runs one site-wide IntersectionObserver
that adds `.in-view`; `global.css:163-189` animates `stroke-dashoffset` from 100
to 0 on any element marked `data-anim-draw`. `<line>` and `<path>` carry
`pathLength="100"` so the dash math is resolution-independent.

Both reduced-motion branches are written. Under `reduce` the line is drawn and
the dots are visible with no transition.

Used by `SectionDivider.astro` and `CausalStack.astro`.

## 10. Section divider

The connective tissue of the landing page: a full-width hairline with three
accent node dots at 25%, 50%, and 75%.

```html
<svg viewBox="0 0 100 4" preserveAspectRatio="none" data-anim-draw>
```

It is the network motif reduced to its minimum, and it is what makes the page
read as one continuous diagram rather than as stacked sections. Twelve lines with
no scoped CSS; everything lives in `global.css:148-161`.

## Applying the idioms

A new panel component should almost always take: the HUD gradient panel (§4),
reticle corners (§3), and an eyebrow (§1). That combination is the house style.

A new list should take the hairline list (§6) and nothing else.

A new interactive row should take the accent wash (§7) at 5%, and promote its
border from `--line-1` to `--line-2`.
