# 03 — Space and layout

## Spacing scale

4px base, near-doubling cadence. From `tokens.css:100-110`.

| Token | rem | px |
|---|---|---|
| `--space-1` | 0.25 | 4 |
| `--space-2` | 0.5 | 8 |
| `--space-3` | 0.75 | 12 |
| `--space-4` | 1 | 16 |
| `--space-5` | 1.5 | 24 |
| `--space-6` | 2 | 32 |
| `--space-7` | 3 | 48 |
| `--space-8` | 4 | 64 |
| `--space-9` | 6 | 96 |
| `--space-10` | 8 | 128 |

The cadence is not a pure doubling. It breaks at `--space-5` (24, not 32) and
again at `--space-7` (48, not 64). Those two half-steps are where most component
padding lands, and they are the reason the scale can stay this short.

Density target: 4 on a 1–10 dial. Daily-application feel. Not a packed cockpit,
not an airy gallery.

## Section rhythm

| Context | Desktop | Mobile |
|---|---|---|
| Between major page sections | `--space-9` (96) | `--space-7` (48) |
| Between subsections | `--space-7` (48) | `--space-6` (32) |

The shipped pattern for a page shell, seen in `ExampleDetail.astro:61-71` and
`global.css:192-199`:

```css
padding: var(--space-7) var(--space-4);
@media (min-width: 720px) { padding: var(--space-9) var(--space-5); }
```

Both axes step up together at 720px. Vertical goes 48 → 96, horizontal 16 → 24.

## Container widths

From `tokens.css:112-116`.

| Token | Width | Use |
|---|---|---|
| `--w-prose` | 720px | Single-column prose, example detail pages |
| `--w-doc` | 1080px | Docs with sidebar |
| `--w-page` | 1280px | Landing, listing pages |
| `--w-wide` | 1440px | Hero only |

Four home components ignore these tokens and hardcode their own frame width. See
[08-drift.md](08-drift.md) §2.

## Measure

`--measure: 68ch` caps prose line length. In practice components reach for
literal `ch` values instead: `44ch`, `52ch`, `56ch`, and `60ch` all appear. The
token is used only by `.static-page`. Recorded in [08-drift.md](08-drift.md) §3.

## Breakpoints

The system is nominally two-breakpoint. Five values ship.

| Value | Uses | Role |
|---|---|---|
| **720px** | 19 | The default. Stack-to-side-by-side, padding ramp |
| **900px** | 9 | Hero split, header nav, blog sidebar |
| 600px | 1 | `Explainer.astro:100` only, and that component is orphaned |
| 480px | 1 | `blog/index.astro:140` only |
| 1024px | 1 | `BaseLayout.astro:164`, footer five-column |

Write new components against 720px and 900px. The other three are accidents, not
tiers.

720px carries the stack-to-columns transition and the padding ramp. 900px carries
the layouts that need real horizontal room: the 7fr/5fr hero, the header's
three-region grid, the blog's content-plus-sidebar split.

Mobile-first throughout. Every query is `min-width`; there is no `max-width`
query on the site.

## Grid patterns in use

| Component | Base | Breakpoint | Columns |
|---|---|---|---|
| `Hero` | `1fr` | ≥900px | `7fr 5fr` |
| `WhyDeepCausality` | `1fr` | ≥900px | `1fr 1fr` |
| `JoinCommunity` | `1fr` | ≥720px | `repeat(2, 1fr)` |
| `CausalStack` layer | `1fr` | ≥720px | `184px 1fr` |
| `Explainer` row | `1fr` | ≥600px | `200px 1fr` |
| `blog/index` | `1fr` | ≥900px | `minmax(0, 1fr) 220px` |
| `BaseLayout` footer | `1fr` | ≥720px / ≥1024px | `1fr 1fr` / `1fr 1fr 1fr 1fr 2fr` |
| `SiteHeader` | `1fr auto` | ≥900px | `auto 1fr auto` |

Two things generalize.

**Label-plus-content rows use a fixed first column.** `184px` in CausalStack,
`200px` in Explainer, `220px` in the blog aside. The fixed column holds a mono
micro-label; the `1fr` column holds the content.

**`minmax(0, 1fr)` guards the content column** in `blog/index.astro:164`. A plain
`1fr` cannot shrink below its content's min-content width, so a long code block
or unbroken URL blows out the grid. Use `minmax(0, 1fr)` whenever a grid column
holds code.

## Overflow safety

`global.css:18-21` sets `overflow-x: clip` on `html, body`:

```css
/* Mobile-first safety net: stop any rogue child from forcing a horizontal scroll.
   `clip` is preferred over `hidden` because it does not establish a new
   scroll container and leaves `position: sticky` intact. */
overflow-x: clip;
```

`clip` rather than `hidden` matters. `hidden` creates a scroll container, which
breaks `position: sticky` on descendants, and the header and blog aside both
depend on sticky.

This is a net, not a licence. A component that overflows is still a bug; the net
stops it from taking the whole page's horizontal scroll with it.
