# DeepCausality — Web Design System

Binding reference for every page in this Astro site. If an implementation contradicts this file, fix the implementation. If this file is wrong, update it first, then the implementation.

Authoring constraint: all prose in this file follows `docs/writing_guides/Ai Styleguide.md`.

---

## 1. Brand anchor

The visual system reads from one source of truth: the hero art at `public/img/frontpage-art.webp`. That image fixes the mood — dark blue-black field, cyan particle-network drifting through space, white wordmark, geometric brain glyph rendered as a faceted line drawing. Three things follow from that.

1. **Off-black base, never `#000`.** The hero sits on a deep blue-black. Pure black under it would look washed out and cheap.
2. **One accent, calibrated to the art.** A desaturated cyan around `#5cd4e1`. No second accent. No purple or magenta secondary.
3. **Network motif, not soft-card motif.** Connections, thin lines, sparse dots, geometric edges. The UI should feel like a node-link diagram, not a SaaS dashboard of rounded tiles.

Tone for copy: technical and confident. No marketing fluff. The visitor we want is an engineer who came to read code.

---

## 2. Color system (dark theme, primary)

All values are sRGB hex; contrast ratios are computed against `--bg-0` unless otherwise stated.

| Token | Hex | Role | Contrast |
|---|---|---|---|
| `--bg-0` | `#070b10` | Page background, deepest field | — |
| `--bg-1` | `#0b1118` | Section background, raised surface | — |
| `--bg-2` | `#121a23` | Code surface, input surface | — |
| `--bg-3` | `#1a2430` | Hover surface, selected nav | — |
| `--line-1` | `#1f2a36` | Hairline divider, default border | — |
| `--line-2` | `#2a3a4a` | Stronger border, focused input | — |
| `--fg-0` | `#e6edf3` | Primary text | 14.4:1 AAA |
| `--fg-1` | `#aab3bd` | Secondary text, captions | 7.8:1 AAA |
| `--fg-2` | `#6b7682` | Tertiary, meta, timestamps | 4.6:1 AA |
| `--accent` | `#5cd4e1` | Single accent: links, focus ring, active nav | 10.2:1 AAA |
| `--accent-ink` | `#062029` | Text on accent surface | 11.1:1 AAA |
| `--warn` | `#e3b341` | Inline warnings only | 9.0:1 AAA |
| `--danger` | `#f47174` | Errors only | 6.3:1 AA |
| `--ok` | `#8ed4a8` | Success only | 9.8:1 AAA |

Rules:

- One accent. `--accent` is the only chromatic color allowed on interactive elements. The status colors `--warn / --danger / --ok` appear only on inline feedback (callouts, form errors, build-status badges) and never on CTAs.
- Pure `#000` is banned. Pure `#fff` is also banned for body text; use `--fg-0`.
- No gradient backgrounds on hero, headers, or buttons. A gradient is permitted only inside the hero image itself (which is a raster file). Synthetic CSS gradients on backgrounds are out.
- "AI purple/blue glow" is banned: no purple box-shadow halos, no neon outer-glow on buttons.

### 2.1 Light theme

The site ships both themes from day one. **Dark is the default**, and the theme attribute resolution order is:

1. User's saved preference in `localStorage` (`dc-theme = "dark" | "light"`).
2. `prefers-color-scheme` from the OS, if no saved preference.
3. Fall back to `dark`.

The toggle and the early-blocking script that prevents the flash of wrong theme are specified in §8.9.

Light theme tokens (override `[data-theme="light"]`):

| Token | Hex | Role | Contrast |
|---|---|---|---|
| `--bg-0` | `#ffffff` | Page background | — |
| `--bg-1` | `#f6f8fa` | Section background | — |
| `--bg-2` | `#eef2f6` | Code surface, input surface | — |
| `--bg-3` | `#e2e8ef` | Hover surface, selected nav | — |
| `--line-1` | `#d8dee4` | Hairline divider, default border | — |
| `--line-2` | `#c0c8d1` | Stronger border, focused input | — |
| `--fg-0` | `#0b1118` | Primary text | 16.7:1 AAA |
| `--fg-1` | `#3b4754` | Secondary text, captions | 10.1:1 AAA |
| `--fg-2` | `#5b6772` | Tertiary, meta, timestamps | 6.5:1 AAA |
| `--accent` | `#0a8a98` | Single accent in light mode (deeper teal, AA on white) | 4.9:1 AA |
| `--accent-ink` | `#ffffff` | Text on accent surface | 4.9:1 AA |
| `--warn` | `#a25c00` | Inline warnings only | 5.4:1 AA |
| `--danger` | `#b42318` | Errors only | 6.6:1 AA |
| `--ok` | `#1f7a3a` | Success only | 5.2:1 AA |

Why the accent shifts: the dark-mode cyan (`#5cd4e1`) reads soft and washed-out on white, and fails AA against `--bg-0`. The light-mode accent uses the same hue family pulled deeper into the teal range so it carries weight against white while still echoing the hero art.

The hero raster `frontpage-art.webp` stays the same file in both themes; in light mode it sits inside a `--bg-2` plate with `--line-1` border so its dark field reads as an intentional artifact rather than a hole punched in the page.

Shadows are re-tinted in light mode (see §5).

---

## 3. Typography

Two type families, both self-hosted (no Google Fonts CDN at runtime).

- **Sans**: `Geist` — UI, headings, prose
- **Mono**: `JetBrains Mono` — code, technical inline values, numeric tabular data

Serif is banned. Inter is banned.

### 3.1 Scale (rem, line-height, letter-spacing)

| Token | rem | px @16 | line-height | letter-spacing | Use |
|---|---|---|---|---|---|
| `--t-display` | 3.75 | 60 | 1.02 | `-0.022em` | Hero only |
| `--t-h1` | 2.5 | 40 | 1.08 | `-0.018em` | Page H1 |
| `--t-h2` | 1.875 | 30 | 1.15 | `-0.012em` | Section heading |
| `--t-h3` | 1.375 | 22 | 1.25 | `-0.005em` | Subsection |
| `--t-h4` | 1.125 | 18 | 1.35 | 0 | Card title |
| `--t-body` | 1.0 | 16 | 1.6 | 0 | Default body |
| `--t-body-sm` | 0.9375 | 15 | 1.55 | 0 | Caption, meta |
| `--t-mono-sm` | 0.875 | 14 | 1.55 | 0 | Inline code |
| `--t-mono-block` | 0.9375 | 15 | 1.6 | 0 | Code block |
| `--t-eyebrow` | 0.75 | 12 | 1.2 | `0.08em` | UPPERCASE eyebrow labels |

Weights:

- Headings: 540 (Geist supports variable; treat 540 as "medium with a touch of presence", not 600+ which looks heavy in dark mode)
- Body: 400
- Strong: 600
- Mono: 450

### 3.2 Prose measure

Long-form prose containers cap line length at `--measure` = `68ch`. Code blocks ignore this and breathe up to the column width. No prose section gets full-bleed text at desktop widths.

### 3.3 Hero typography

The hero display uses `--t-display` at weight 540, color `--fg-0`. The tagline ("Dynamic causality for advanced systems.") sits underneath at `--t-h3`, color `--fg-1`. No drop shadow. No text-fill gradient. No oversized H1 stunt. The wordmark glyph from the hero art carries the visual weight; the typography supports it.

---

## 4. Spacing & rhythm

Base unit is **4px**. Scale follows a near-doubling cadence so vertical rhythm stays predictable.

```
--space-1:  0.25rem   /*  4 */
--space-2:  0.5rem    /*  8 */
--space-3:  0.75rem   /* 12 */
--space-4:  1rem      /* 16 */
--space-5:  1.5rem    /* 24 */
--space-6:  2rem      /* 32 */
--space-7:  3rem      /* 48 */
--space-8:  4rem      /* 64 */
--space-9:  6rem      /* 96 */
--space-10: 8rem      /* 128 */
```

Section vertical rhythm at desktop: `--space-9` between major page sections, `--space-7` between subsections inside a section. Mobile collapses these to `--space-7` / `--space-6`.

Container widths:

- `--w-prose`: 720px (single-column prose pages)
- `--w-doc`: 1080px (docs with sidebar)
- `--w-page`: 1280px (landing, listing pages)
- `--w-wide`: 1440px (hero only)

Density target: 4 on the 1–10 dial. Daily-app feel, not packed cockpit, not airy gallery.

---

## 5. Elevation, borders, shadow language

No soft-card stack. The default container for grouped content is a `1px` hairline using `--line-1`, sometimes paired with a tinted inset highlight. Drop shadows are reserved for elements that genuinely float (popovers, the search panel, the mobile menu sheet).

Shadow tokens are theme-aware. Each theme defines its own tints so shadows always sit in the same color family as the page background.

Dark theme:

```
--shadow-1: 0 1px 0 0 rgba(255,255,255,0.04) inset;          /* hairline lift */
--shadow-2: 0 12px 24px -16px rgba(8,16,24,0.70);            /* floating panel */
--shadow-3: 0 24px 48px -24px rgba(8,16,24,0.80);            /* modal */
```

Light theme:

```
--shadow-1: 0 1px 0 0 rgba(255,255,255,0.6) inset;           /* hairline lift */
--shadow-2: 0 10px 24px -16px rgba(15,23,32,0.18);           /* floating panel */
--shadow-3: 0 24px 48px -24px rgba(15,23,32,0.22);           /* modal */
```

`rounded-2xl + shadow-md` soft-card stack: **banned**. Cards on this site use `--radius-sm` (4px) or `--radius-md` (10px) and rely on `border: 1px solid var(--line-1)` plus `--shadow-1` for separation.

Radii:

```
--radius-sm: 4px    /* default control */
--radius-md: 10px   /* cards, code blocks */
--radius-lg: 16px   /* hero plate, only one allowed per page */
--radius-pill: 999px
```

Glassmorphism is banned. Frosted backdrops break against the hero art and read as decorative noise.

---

## 6. Motion

Motion intensity: 6 on the 1–10 dial. Fluid CSS, no scroll-jacking, no library dependency at launch.

Allowed properties: `transform`, `opacity`, `filter` (only `blur` on the search overlay), and `clip-path` (only on staged reveals).

Banned properties for animation: `width`, `height`, `top`, `left`, `margin`, `padding`, `background-color` (on large surfaces), `box-shadow` (on large surfaces). Animating any of these triggers layout or paint storms.

Durations:

```
--dur-fast:  120ms    /* hover, focus */
--dur-med:   220ms    /* card hover, nav reveal */
--dur-slow:  420ms    /* page-section entrance */
```

Easing:

```
--ease-out:    cubic-bezier(0.16, 1, 0.3, 1)
--ease-in-out: cubic-bezier(0.65, 0, 0.35, 1)
```

Reduced motion is honored: every transition or animation rule must sit inside `@media (prefers-reduced-motion: no-preference)`, with a static fallback by default. Do not gate functionality on motion.

Hardware acceleration: animate only `transform` and `opacity`. Promote sparingly with `will-change: transform` and remove it after the transition ends. Never apply `will-change` to scrolling containers.

Perpetual animation: at most one per page. A single ambient element — for example, a slow `transform: translate3d` drift on a decorative SVG line in the hero — is fine. Two or more makes the page feel restless and burns battery.

---

## 7. Component architecture (Astro)

Static-first. Client islands are the exception, not the default.

### 7.1 File layout

```
src/
  layouts/
    BaseLayout.astro          # global shell, meta, header, footer
    DocsLayout.astro          # sidebar + article
    ProseLayout.astro         # blog, monograph overview
  components/
    nav/
      SiteHeader.astro        # static
      MobileMenu.astro        # client:load
      SearchTrigger.astro     # client:idle (Pagefind UI)
      ThemeToggle.astro       # client:load (theme swap, persists to localStorage)
    home/
      Hero.astro              # static
      ExampleGrid.astro       # static
      ExampleCard.astro       # static
      PillarRow.astro         # static
    docs/
      Sidebar.astro           # static, server-rendered from collection
      Toc.astro               # static
      Callout.astro           # static
      CodeBlock.astro         # static; Shiki via Astro
    ui/
      Tag.astro
      Eyebrow.astro
      Divider.astro
      Button.astro
  styles/
    tokens.css                # the canonical token set; source of truth
    global.css                # element resets and base typography
    prose.css                 # long-form content rules
    code.css                  # Shiki theme overrides
```

### 7.2 Naming

- PascalCase for `.astro` components, kebab-case for routes and content files.
- One component per file. No barrel files.
- Component names describe the role, not the position (`ExampleCard`, not `LeftCard`).

### 7.3 Islands policy

A component becomes a client island only if it meets one of these conditions:

1. It needs DOM events the user initiates (search input, mobile menu toggle, copy-code button).
2. It depends on `window` or browser-only APIs.
3. It hydrates UI state from `localStorage` (theme toggle when added).

Everything else is static. The landing page ships with zero client JS by default; the only exception is `SearchTrigger.astro` (Pagefind) loaded with `client:idle`.

### 7.4 No framework imports

No React, Vue, Svelte, Solid, or Preact in this site. No Tailwind. No Framer Motion. If a future feature genuinely needs one of these, it gets its own change proposal first.

---

## 8. Component specs

### 8.1 Site header (`SiteHeader.astro`)

Layout: full-bleed bar, 64px tall, `--bg-0` with a `1px` bottom border `--line-1`. Sticky at top. Three regions in a grid: brand (left), primary nav (center-left), utility (right: search trigger, GitHub link).

States:

- Default: `border-bottom: 1px solid var(--line-1)`.
- Scrolled past 80px: same border, plus `--shadow-1` and a 200ms opacity transition on a `::after` underline under the active nav item.
- Active nav link: `color: var(--fg-0)`, with a 2px `--accent` underline drawn via `transform: scaleX(1)` on a pseudo-element. Inactive links: `color: var(--fg-1)`.

No drop shadow on the header at rest. No translucent backdrop blur.

### 8.2 Hero (`Hero.astro`)

Layout: asymmetric split. Left column 7/12 holds wordmark, tagline, two CTAs, and a small "what this is" sub-line. Right column 5/12 holds the hero art (`frontpage-art.webp`) inside a `--radius-lg` container with `border: 1px solid var(--line-1)` and `--shadow-2`. On viewports below 900px the columns stack and the art moves above the text.

Wordmark in the left column is set as text (Geist 540) and accompanied by a small node-link SVG glyph echoing the brain motif from the hero art. The art on the right does the heavy visual lifting; the left text stays restrained.

CTAs: primary is "Read the docs" (solid accent on `--accent-ink`), secondary is "View on GitHub" (text + arrow glyph, no border). Both use `--radius-sm`, 44px tall.

No centered text over a gradient. No oversized H1 stunt. No fade-from-black overlay applied via CSS; the raster image already has the right falloff.

### 8.3 Example grid (`ExampleGrid.astro` + `ExampleCard.astro`)

Six code-example cards in a 3×2 grid at desktop. The "3 equal cards horizontally" cliché is mitigated by intentional internal variance:

- Each card carries a 16px square domain glyph (custom monoline SVGs, never emoji, never the Lucide user icon).
- Each card's snippet height varies by 1–3 lines on purpose; cards are not forced to equal height. Use `align-items: start` on the grid.
- The middle card of each row is offset by 16px on the y-axis at viewports above 1200px. Below that, all six align flat.

Card structure (top to bottom):

1. Eyebrow row: domain label (`--t-eyebrow`, `--fg-1`, uppercase, letter-spaced) on the left, monoline glyph on the right.
2. One-line problem statement (`--t-h4`, `--fg-0`).
3. Code snippet, 10–18 lines, Shiki-highlighted, `--bg-2` background, no border, `--radius-md`.
4. Foot row: a hairline divider (`--line-1`) and a single "Open example →" link in `--accent`.

Hover state on the entire card:

- `border-color: var(--line-2)` over 220ms.
- `transform: translate3d(0, -2px, 0)` over 220ms ease-out.
- A 1px inner top highlight via `box-shadow: inset 0 1px 0 rgba(255,255,255,0.06)`.

No tilt-on-mouse parallax. No glow halo. No scale > 1.0. No background-color transition.

Mobile (≤ 768px): single column, no offset, cards full-width with 16px page gutter.

### 8.4 Pillar row (`PillarRow.astro`)

Three pillars: Causaloid, Context, Effect Ethos. **Not** rendered as three rounded cards in a row (that's the banned 3-equal-cards layout). Render as a horizontal sequence linked by a thin SVG path drawn left-to-right at the top of the row, with each pillar's name and one-paragraph description hanging beneath an anchor node on that path. The path echoes the network/graph motif from the hero art.

At ≤ 900px the row collapses to a vertical stack and the connector becomes a vertical line.

### 8.5 Code block (`CodeBlock.astro`)

Powered by Shiki. Background `--bg-2`, padding `var(--space-5)`, `--radius-md`, no border.

Optional header row with file path on the left and a copy button on the right. The copy button is the only client-side island a code block may include; it loads with `client:visible`. Inline code uses `--bg-3`, `--radius-sm`, padding `0 0.3em`, font-size 0.9em.

Line numbers off by default; on when an example detail page explicitly requests them via frontmatter.

### 8.6 Callout (`Callout.astro`)

Four variants: `note`, `warn`, `danger`, `ok`. Each variant draws a 3px left border in the matching token color and uses the same `--bg-1` panel with `--line-1` hairlines on the other three sides. No icon by default. An optional icon slot is allowed but must be a monoline SVG, never an emoji.

### 8.7 Footer (`SiteFooter.astro`)

Three columns at desktop, single column at mobile. Top edge: a 1px `--line-1` divider, nothing more. Columns: project (links to docs, blog, monograph, GitHub), community (issues, discussions, license), and a single short paragraph crediting contributors. Background `--bg-0`, no top shadow, no large brand mark.

Email capture: none at launch.

### 8.9 Theme toggle (`ThemeToggle.astro`)

Two-state control: sun glyph and moon glyph, both monoline SVGs. Sits in the header utility region, immediately left of the GitHub link, at 32×32 hit area. The visible glyph is 16×16. The current theme's glyph is `--fg-0`; the inactive glyph is hidden via `display: none` to avoid the "two icons stacked" cliché.

Behavior:

- Click flips the theme by toggling `data-theme` on the `<html>` root and writing `dc-theme` to `localStorage`. No page reload.
- Keyboard: native button; Enter and Space activate.
- Aria-label updates to "Switch to light theme" / "Switch to dark theme" based on the current state.
- The transition between themes runs for `--dur-med` on `color` and `background-color` of `:root` only. No theme-wide cross-fade animation; that costs paint without earning anything.

A small inline `<script>` runs in the document `<head>` before the first paint to set `data-theme` from `localStorage` or `prefers-color-scheme`. This script is the one exception to the "no inline JS in `<head>`" rule because it prevents the flash of wrong theme; it stays under 600 bytes minified.

Implementation sketch (informative, not normative):

```html
<script is:inline>
  const saved = localStorage.getItem('dc-theme');
  const sysDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const theme = saved ?? (sysDark ? 'dark' : 'dark'); /* default dark */
  document.documentElement.setAttribute('data-theme', theme);
</script>
```

The toggle itself is the only client island this component needs and loads with `client:load` so it works the moment the header is interactive.

### 8.8 Search input (`SearchTrigger.astro` + Pagefind UI)

Header-side trigger renders a 240px-wide input shell with the placeholder "Search docs" and a `⌘K` hint glyph on the right. The shell itself is static. Clicking it (or hitting ⌘K) hydrates the Pagefind UI client island, which mounts a centered modal at 640px wide, `--bg-1` background, `--shadow-3`, `--radius-md`, with results rendered inline below the input. No backdrop blur; use a flat `rgba(7,11,16,0.72)` scrim over the page.

---

## 9. Page-type design rules

### 9.1 Landing page

- Hero (§8.2)
- Example grid (§8.3)
- "What is dynamic causality?" — three short paragraphs in `--w-prose`, left-aligned, no decorative imagery.
- Pillar row (§8.4)
- Closing band: one sentence and a CTA to docs. No newsletter form.

Above-the-fold rule: only the hero and the first row of example cards are visible at 1440×900. The "What is dynamic causality?" copy lives below the fold by construction.

### 9.2 Documentation pages

Two-column layout inside `--w-doc`. Left sidebar 240px, article column fills the rest, right TOC 200px on screens above 1200px. Sidebar background `--bg-0`, hairline right border. Sticky header preserved.

Prose lives inside `--w-prose` constraint within the article column. Headings use `--t-h1` / `--t-h2` / `--t-h3`. Code blocks ignore the prose measure.

Math via KaTeX. Inline math uses Geist for the surrounding text and KaTeX defaults for the formula; display math gets a `--bg-1` panel with `--line-1` border and `--radius-sm`.

### 9.3 Code-example detail pages

Single-column inside `--w-page` for the first two sections (intro + headline code), then collapses to `--w-prose` for the walkthrough. Run instructions appear in a `Callout` with `note` variant. Related crates listed as a horizontal row of tags using `--radius-pill`.

### 9.4 Blog posts

Pure prose layout. `ProseLayout.astro`, `--w-prose`. Author byline as a single small line at the top: name, date, no avatar (the "Jane Doe egg avatar" pattern is banned). No related-posts grid at the bottom — a simple "Back to blog" link is enough.

### 9.5 Monograph overview pages

Academic register. Title, volume number, one-paragraph abstract. A persistent download band sits at the top of the article column: a `Button` linking to the canonical PDF, plus the LaTeX-source link to `papers/src/EPP/`. The body below is the MDX overview. Citation block at the bottom in BibTeX.

---

## 10. Accessibility

- Color contrast meets WCAG AA everywhere and AAA on body prose.
- All interactive elements have a visible focus ring: `outline: 2px solid var(--accent); outline-offset: 2px`. Never remove the outline without providing an equivalent.
- The accent ring is the focus ring; do not mix with `box-shadow` glows.
- Skip-link in the header that jumps to `#main`, visible only on focus.
- Heading levels are strictly hierarchical inside each page; no jumping from `<h1>` to `<h3>`.
- All non-decorative images carry meaningful `alt`. Decorative SVGs use `aria-hidden="true"`.
- `prefers-reduced-motion: reduce` disables every transition except `opacity` ≤ 100ms.

---

## 11. Performance budgets

Lighthouse targets at launch, measured on the landing page on a mid-tier mobile profile:

- Performance ≥ 95
- Accessibility ≥ 95
- Best practices ≥ 95
- SEO ≥ 95

Asset budgets per page:

- Total transferred: ≤ 250 KB for the landing page (excluding the hero raster, which is its own line item).
- Hero raster: ≤ 180 KB; serve as WebP.
- JS shipped on the landing page: 0 KB except the Pagefind UI bundle loaded with `client:idle` after the page is interactive.
- Self-hosted fonts: `Geist` and `JetBrains Mono` only, woff2, subset to Latin Extended.

DOM cost guard: a noise/grain overlay is allowed only as a fixed, `pointer-events: none` pseudo-element on the page background. It must never sit inside a scrolling container.

---

## 12. Anti-patterns (banned)

These are forbidden in this codebase. PR reviewers reject them on sight.

1. Centered hero text over a CSS gradient.
2. `rounded-2xl` + `shadow-md` soft-card stack on rows of three.
3. Generic 3-equal-cards horizontal feature row without internal asymmetry.
4. Tailwind. Any utility-class framework.
5. Inter font. Serif on technical UI surfaces.
6. Emojis as icons. Egg/Lucide-user-icon avatars. Generic stock avatars.
7. Purple or magenta accent glows.
8. Outer-glow box-shadows on buttons.
9. Glassmorphism panels with `backdrop-filter: blur(...)` over the hero or the docs sidebar.
10. Animating `width`, `height`, `top`, `left`, `margin`, `padding`, large `background-color`, or large `box-shadow`.
11. Custom mouse cursors.
12. Auto-playing video or audio anywhere.
13. Scroll-jacking. Scroll-hijack horizontal panels on the landing page.
14. Marketing filler verbs: "elevate", "unleash", "seamless", "next-gen", plus the styleguide ban list ("delve into", "shed light on", "game-changer", "unlock the potential", "not only … but also").
15. `h-screen` for the hero. Use `min-height: 100dvh` if a viewport-bound height is ever needed.
16. Every-paragraph-leading-icon-bullet patterns.
17. Three-column footer with a newsletter form. Email capture is out at launch.

---

## 13. Token file

The canonical token set lives in `src/styles/tokens.css`. That file is the source of truth referenced by everything else. The bootstrap placeholder originally written there has been replaced with the full token set defined in this document.

---

## 14. Open follow-ups

- A second language. Astro i18n is scaffolded; the design system does not change per locale.
- A second accent for editorial content (blog illustrations, monograph diagrams). Reserved, not chosen.

End of document.
