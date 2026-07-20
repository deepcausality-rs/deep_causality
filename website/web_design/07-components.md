# 07 — Components

Anatomy as built. Line references point at `website/web/src/`.

## Architecture

Static-first, and more so than the spec assumed. `grep -rn "client:" src` returns
nothing: there are no framework islands, no hydration directives, and no React,
Vue, Svelte, or Solid anywhere. Interactivity is plain `<script>` in eight files.

| File | Kind | Purpose |
|---|---|---|
| `BaseLayout.astro:78` | `is:inline` | Pre-paint theme resolution |
| `BaseLayout.astro:93` | module | Site-wide IntersectionObserver → `.in-view` |
| `SiteHeader.astro:94` | module | Mobile sheet, Escape key |
| `ThemeToggle.astro:41` | module | Theme flip, localStorage |
| `ExampleGrid.astro:63` | module | Tablist ARIA, code-box height equalization |
| `blog/index.astro:118` | module | Force `<details open>` at ≥900px |
| `index.astro:50` | `is:inline set:html` | JSON-LD |
| `blog/[...slug].astro:72` | `is:inline set:html` | JSON-LD |

Only the first is inline, and only because it must run before first paint.

## BaseLayout

The shell: meta, Open Graph, JSON-LD, theme guard, footer.

Footer is the only three-tier grid on the site: `1fr` → `1fr 1fr` at 720px →
`1fr 1fr 1fr 1fr 2fr` at 1024px (`:158-167`). The trailing `2fr` holds the
contributor paragraph, capped at `52ch`.

It also owns the shared IntersectionObserver (`:97-114`), tuned
`rootMargin: '0px 0px -10% 0px', threshold: 0.15`. One observer for the whole
site; components opt in with `data-anim-draw` and read `.in-view`.

## SiteHeader

Sticky header, and the most complex component in the codebase.

Grid goes `1fr auto` at `min-height: 56px` base, then `auto 1fr auto` at 64px
from 900px up (`:138-155`). Below 900px the primary nav hides and a burger
reveals an off-canvas sheet at `min(86vw, 320px)`.

Background is `color-mix(in srgb, var(--bg-0) 92%, transparent)` (`:134`), so
content shows faintly through while scrolling without a blur.

Active nav link draws a 2px accent underline via `transform: scaleX(0→1)` with
`transform-origin: left` (`:181-187`). Transform, not width, so it is composited.

The dropdown handles the classic diagonal-travel problem with a 6px invisible
bridge (`:194`) and opens on `:hover` **and** `:focus-within` (`:221-227`). Its
`visibility` transition carries a delay that resets to `0s` on hover, so it hides
after the fade rather than during it.

Two accessibility details worth preserving: the skip-link parks at
`left: -10000px` and slides in on focus (`:125-130`), and the sheet closes on
Escape.

The mobile scrim uses `backdrop-filter: blur(2px)` (`:298-299`). That contradicts
the glassmorphism ban; see [08-drift.md](08-drift.md) §8.

## ThemeToggle

32×32 `inline-grid`, `place-items: center`. Flips `data-theme` on the root and
writes `dc-theme` to localStorage. `aria-label` swaps with state.

Glyph swap uses `:global([data-theme='dark']) .ico-sun` (`:37-38`) to escape
Astro's scope hash. The inactive glyph is `display: none` rather than stacked.

No `:focus-visible` rule, unlike the adjacent `.burger`. Inconsistency, not a
failure: the global ring still applies.

## Hero

Asymmetric split, `7fr 5fr` at ≥900px (`:45-53`). Below that it stacks and the
art moves above the text via `order: -1` (`:132`).

The display type is the site's only fluid clamp:

```css
font-size: clamp(2.25rem, 6vw + 0.5rem, 3.75rem);   /* :61 */
```

Floor 36px, ceiling 60px matching `--t-display`. The `6vw + 0.5rem` form keeps it
zoom-responsive; a bare `vw` value would not scale with user text settings.

`.btn` is 44px tall (`:75`), which is the accessible touch-target minimum, and it
is the only button height definition on the site. Primary hover shifts the accent
90% toward white; `:active` presses `translateY(1px)` (`:87`) — the only press
state anywhere.

The art frame carries a wide, soft accent halo layered over `--shadow-2`:

```css
box-shadow: 0 0 80px -28px color-mix(in srgb, var(--accent) 38%, transparent), var(--shadow-2);  /* :120 */
```

`.btn` has no `:focus-visible` rule; it inherits the global ring.

## CausalStack

Five-band platform diagram: Discover → Model → Act → Govern → Run. The most
elaborate animation on the site.

Each `.layer` is `1fr` base, `184px 1fr` at ≥720px (`:163-170`), with a 3px accent
tick on the left edge that grows from 18px to 28px on hover (`:149-160`). The row
tints with a 5% accent wash.

Entrance is staggered by an inline custom property: `style={\`--i:${i}\`}` at
`:71`, consumed as `transition-delay: calc(var(--i) * 110ms)` at `:243`. Gated on
`.in-view`, with explicit `no-preference` and `reduce` branches.

`max-width: 900px` is hardcoded rather than tokenized (`:101`).

## ExampleGrid

Tabbed Rust snippet panel. The landing page's main content surface.

Full ARIA tablist: roving tabindex, arrow-key navigation, and a JS pass that
equalizes code-box heights so switching tabs does not jump the page. Tabs are
`flex: 1 1 0` with `min-width: 140px` and scroll horizontally when they overflow;
the scrollbar is styled to 4px.

Tab states layer three signals at once: text to `--fg-0`, an accent wash (5%
hover, 8% active), and a 2px indicator scaling `scaleX(0.4)` at 50% opacity on
hover to `scaleX(1)` at full on active.

Focus uses a negative inset offset, the only one on the site:

```css
outline: 2px solid var(--accent); outline-offset: -3px;   /* :252 */
```

Necessary because the tab sits flush inside a clipped scroll container, where a
positive offset would be cut off.

`:286` reserves two lines of headline height with
`min-height: calc(var(--t-h3) * 1.25 * 2)` to stop layout shift between tabs.
Effective, though it hardcodes the line-height rather than using `--lh-h3`.

## WhyDeepCausality

Two-column feature list at ≥900px. Semantic `<dl>`/`<dt>`/`<dd>`, with
`border-top: 1px solid var(--line-1)` as the row rule. No hover states, no
transitions. Correct for a read-only surface.

## JoinCommunity

Two cards at ≥720px, `max-width: 880px`. Hover promotes the border to `--line-2`,
tints the panel gradient's bottom stop toward the accent, lifts by
`translateY(-1px)`, and turns the card's `h3` to accent.

The lift is 1px at `--dur-fast`, whereas ExampleCard uses 2px at `--dur-med` for
the same gesture. Pick one; see [08-drift.md](08-drift.md) §4.

## SectionDivider

Twelve lines, zero scoped CSS. Documented in [06-idioms.md](06-idioms.md) §10.

## ExamplesList

The cleanest file in the codebase. No hardcoded values at all. Hairline rows plus
slide-on-hover, and nothing else.

## ExampleDetail

Single column at `--w-prose`. Four-corner L-brackets via inline SVG data-URI
masks at 14px — a third implementation of a motif that already exists twice
elsewhere. No hover states beyond link colour, no transitions.

## overview/index.astro

At 479 lines the longest page, and it carries a private `.dc-*` diagram
vocabulary used nowhere else: bands with a 2px accent left rail, chips, and `▼`
flow arrows. Self-consistent, but it is a parallel design system.

It also breaks the panel radius convention: `.dc-diagram` uses `--radius-sm`
(`:402`) where every other framed panel uses `--radius-md`.

## blog/index.astro

The only four-step padding ramp: base → 480px → 720px, and the only use of
`--space-6` as page side-padding. Content grid is `minmax(0, 1fr) 220px` at
≥900px with a sticky aside.

The 900px breakpoint is duplicated as a string in JS (`:123`,
`matchMedia('(min-width: 900px)')`), so a change to the CSS breakpoint silently
desynchronizes the archive's open-state behaviour.

`scroll-margin-top: 80px` appears four times to clear the sticky header. The
header is 56px or 64px, never 80. See [08-drift.md](08-drift.md) §7.

## Orphans

Two components are unreferenced. Verified: nothing imports either.

**`ExampleCard.astro`** — superseded by ExampleGrid's tab panels. Still carries
live design vocabulary: a per-slug monoline SVG glyph map (six 16×16 paths at
`stroke-width="1.4"`, plus a node-link fallback) that exists nowhere else. It
also has a real bug: `:65` animates `box-shadow` on hover, but `:60` omits
`box-shadow` from the transition list, so the halo snaps instead of fading.

**`Explainer.astro`** — the site's only 600px breakpoint lives here and nowhere
else.

Do not delete either without a decision. The glyph map in particular is work that
would have to be redone. Both are candidates for the `reverted/` convention used
elsewhere in this repo rather than for removal.

## Static pages

`about`, `community`, and `accessibility` carry zero scoped CSS and rely entirely
on `.static-page` / `.prose` in `global.css:192-215`. The five example category
pages are identical 14-line files differing only in frontmatter, with eyebrow
strings always formatted `"Examples · <Category>"`.
