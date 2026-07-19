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
| `--fg-0` | `#e6edf3` | Primary text | 16.70 AAA |
| `--fg-1` | `#aab3bd` | Secondary text, captions | 9.29 AAA |
| `--fg-2` | `#6b7682` | Tertiary, meta, timestamps | **4.26 — fails AA** |
| `--accent` | `#5cd4e1` | Single accent: links, focus ring, active nav | 11.24 AAA |
| `--accent-ink` | `#062029` | Text on accent surface | 9.60 AAA (vs `--accent`) |
| `--warn` | `#e3b341` | Inline warnings only | 10.14 AAA |
| `--danger` | `#f47174` | Errors only | 7.01 AAA |
| `--ok` | `#8ed4a8` | Success only | 11.40 AAA |

Ratios are measured, not estimated: computed from these hex values under WCAG 2.1 relative luminance. Grades are for normal-size text (AA ≥ 4.5, AAA ≥ 7.0).

`--fg-2` fails AA against `--bg-0` and degrades further on raised surfaces (4.10 on `--bg-1`, 3.79 on `--bg-2`, 3.39 on `--bg-3`). **Treat it as a decorative grey, not a text grey.** It is correct for hairline ticks, coordinate indices, and rules. It is not safe for anything a reader must actually read; use `--fg-1` there.

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

The toggle and the early-blocking script that prevents the flash of wrong theme are specified in §8.8.

Light theme tokens (override `[data-theme="light"]`):

| Token | Hex | Role | Contrast |
|---|---|---|---|
| `--bg-0` | `#ffffff` | Page background | — |
| `--bg-1` | `#f6f8fa` | Section background | — |
| `--bg-2` | `#eef2f6` | Code surface, input surface | — |
| `--bg-3` | `#e2e8ef` | Hover surface, selected nav | — |
| `--line-1` | `#d8dee4` | Hairline divider, default border | — |
| `--line-2` | `#c0c8d1` | Stronger border, focused input | — |
| `--fg-0` | `#0b1118` | Primary text | 18.96 AAA |
| `--fg-1` | `#3b4754` | Secondary text, captions | 9.48 AAA |
| `--fg-2` | `#5b6772` | Tertiary, meta, timestamps | 5.79 AA |
| `--accent` | `#0a8a98` | Single accent in light mode (deeper teal) | **4.12 — fails AA** |
| `--accent-ink` | `#ffffff` | Text on accent surface | **4.12 — fails AA** (vs `--accent`) |
| `--warn` | `#a25c00` | Inline warnings only | 5.16 AA |
| `--danger` | `#b42318` | Errors only | 6.57 AA |
| `--ok` | `#1f7a3a` | Success only | 5.38 AA |

Why the accent shifts: the dark-mode cyan (`#5cd4e1`) reads soft and washed-out on white, and fails AA against `--bg-0`. The light-mode accent uses the same hue family pulled deeper into the teal range so it carries weight against white while still echoing the hero art.

**Known defect, unresolved.** `#0a8a98` does not go deep enough. At 4.12:1 it misses AA for normal text, and `global.css` colors every link with it, so every body-size link in light mode is below threshold. The same pairing is the primary CTA (`--accent-ink` on `--accent`), which fails identically. On raised surfaces it is worse: 3.87 on `--bg-1`, 3.66 on `--bg-2`, 3.34 on `--bg-3`.

This contradicts §10. Fixing it is a colour decision, not a cleanup, so it is recorded here rather than silently changed. The cheapest fix that preserves the hue is roughly `#087b88`, which reaches 4.5 on white; it also darkens every accent tick and node dot in light mode, which is why it needs a deliberate call.

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

Reduced motion is honored at the **token level**, not per rule. `tokens.css` zeroes the three duration tokens under `prefers-reduced-motion: reduce`:

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --dur-fast: 0ms;
    --dur-med: 0ms;
    --dur-slow: 0ms;
  }
}
```

Any transition whose duration resolves through a token is therefore compliant with no media query of its own. Write `transition: transform var(--dur-med) var(--ease-out)` and you are done.

Two consequences follow, and both are binding.

1. **A hardcoded duration silently opts out of the contract.** Always use a token. The current exceptions are `110ms` and `280ms` in `CausalStack.astro:243-244`, and the `220` and `150` millisecond values in JS at `SiteHeader.astro:110` and `ExampleGrid.astro:128`. The SiteHeader value must be kept in sync with `--dur-med` by hand.
2. **`@keyframes` animations still need an explicit branch.** The token override does not reach an animation's own duration. `global.css:184-188` writes the `reduce` branch for the draw-in idiom; follow that pattern.

Do not gate functionality on motion.

Hardware acceleration: animate only `transform` and `opacity`. Promote sparingly with `will-change: transform` and remove it after the transition ends. Never apply `will-change` to scrolling containers.

Perpetual animation: at most one per page. A single ambient element — for example, a slow `transform: translate3d` drift on a decorative SVG line in the hero — is fine. Two or more makes the page feel restless and burns battery.

---

## 7. Component architecture (Astro)

Static-first. Client islands are the exception, not the default.

### 7.1 File layout

```
src/
  layouts/
    BaseLayout.astro          # shell, meta, JSON-LD, theme guard, footer, IntersectionObserver
  components/
    nav/
      SiteHeader.astro        # header, dropdown, burger + off-canvas sheet
      ThemeToggle.astro       # theme swap, persists to localStorage
    home/
      Hero.astro
      CausalStack.astro       # five-band platform diagram
      ExampleGrid.astro       # tabbed Rust snippet panel
      WhyDeepCausality.astro
      JoinCommunity.astro
      SectionDivider.astro
      examples.ts             # snippet data for ExampleGrid
      ExampleCard.astro       # ORPHAN — unreferenced, see below
      Explainer.astro         # ORPHAN — unreferenced, see below
    examples/
      ExamplesList.astro      # hairline list
      CategoryList.astro      # per-category index
      ExampleDetail.astro     # single example page body
  content/
    blog/en/*.md
    examples/en/*.mdx
  pages/                      # routes; several carry scoped <style>
  styles/
    tokens.css                # the canonical token set; source of truth
    fonts.css                 # @font-face declarations
    global.css                # resets, base typography, shared idiom layer
  consts.ts                   # external URLs
  content.config.ts           # collection schemas
```

There is no `ui/` directory and no `docs/` component directory. The primitives the spec once placed under `ui/` live elsewhere and are documented in §12:

- **Button** — a `.btn` rule scoped inside `Hero.astro`. It is the only button definition on the site.
- **Eyebrow** — the global `.eyebrow` class in `global.css`.
- **Divider** — `SectionDivider.astro`.
- **Tag** — the pill chip pattern, reimplemented in four components.

There is no `prose.css` or `code.css`. Prose rules live in `global.css` under `.static-page .prose`; Shiki theming lives in `global.css` and is configured in `astro.config.mjs`.

Documentation moved to a separate Starlight site at `website/docs/`, which is why no docs layout or docs components exist here. See §9.2.

**Orphans.** `ExampleCard.astro` and `Explainer.astro` are imported by nothing. Both still carry live design vocabulary — `ExampleCard` holds a per-slug monoline SVG glyph map that exists nowhere else, and `Explainer` holds the site's only 600px breakpoint. Neither should be deleted casually; they are candidates for the repo's `reverted/` convention. `ExampleCard.astro:60` also carries a latent bug: it animates `box-shadow` on hover but omits `box-shadow` from its transition list, so the halo would snap rather than fade if the component were ever mounted.

### 7.2 Naming

- PascalCase for `.astro` components, kebab-case for routes and content files.
- One component per file. No barrel files.
- Component names describe the role, not the position (`ExampleCard`, not `LeftCard`).

### 7.3 Islands policy

**There are no client islands.** `grep -rn "client:" src` returns nothing. The site ships zero hydration directives, because it ships zero framework components — there is nothing to hydrate.

Interactivity is plain `<script>` in the eight places that need it:

| File | Kind | Purpose |
|---|---|---|
| `BaseLayout.astro:78` | `is:inline` | Pre-paint theme resolution (FOUC guard) |
| `BaseLayout.astro:93` | bundled module | Site-wide IntersectionObserver → `.in-view` |
| `SiteHeader.astro:94` | bundled module | Mobile sheet open/close, Escape key |
| `ThemeToggle.astro:41` | bundled module | Theme flip, localStorage write |
| `ExampleGrid.astro:63` | bundled module | Tablist ARIA, code-box height equalization |
| `blog/index.astro:118` | bundled module | Force `<details open>` at ≥900px |
| `index.astro:50` | `is:inline set:html` | JSON-LD |
| `blog/[...slug].astro:72` | `is:inline set:html` | JSON-LD |

Only the theme guard is inline, and only because it must run before first paint.

The rule for new work: reach for a plain module script. A client island requires a framework dependency, and §7.4 forbids adding one without a change proposal.

### 7.4 No framework imports

No React, Vue, Svelte, Solid, or Preact in this site. No Tailwind. No Framer Motion. If a future feature genuinely needs one of these, it gets its own change proposal first.

The full runtime dependency set is four Astro integrations and one build-time tool:

| Package | Role |
|---|---|
| `@astrojs/mdx` | `.mdx` example pages |
| `@astrojs/sitemap` | Sitemap with per-route priority |
| `astro-mermaid` + `mermaid` | Diagrams in blog posts; see §8.10 |
| `pagefind` | Build-time search index; see §8.9 |

---

## 8. Component specs

### 8.1 Site header (`SiteHeader.astro`)

Sticky at top, `z-index: 20`, `1px` bottom border `--line-1`. Background is `color-mix(in srgb, var(--bg-0) 92%, transparent)`, so content shows faintly through while scrolling without any blur.

Layout is two-tier:

| | Base | ≥900px |
|---|---|---|
| Grid | `1fr auto` | `auto 1fr auto` |
| Height | `min-height: 56px` | `min-height: 64px` |
| Padding | `0 var(--space-4)` | `0 var(--space-5)` |

Below 900px the primary nav is hidden and a burger reveals an off-canvas sheet at `min(86vw, 320px)`. Utility region holds the theme toggle and the GitHub link. There is no search trigger; see §8.9.

States:

- Default: `border-bottom: 1px solid var(--line-1)`.
- Active nav link: `color: var(--fg-0)`, with a 2px `--accent` underline drawn via `transform: scaleX(0 → 1)` with `transform-origin: left`. Inactive links: `color: var(--fg-1)`.
- Dropdown opens on `:hover` **and** `:focus-within`, with a 6px invisible bridge covering the diagonal-travel gap. Its `visibility` transition carries a delay that resets to `0s` on hover, so the menu hides after the fade rather than during it.

There is **no scroll-state treatment**. The spec previously described a shadow and underline transition past 80px scroll; nothing implements it, and the flat sticky bar works. Do not add one without a reason.

The mobile scrim uses `backdrop-filter: blur(2px)`. That is a deliberate, narrow exception to the glassmorphism ban; see §13 item 9.

Accessibility details worth preserving: the skip-link parks at `left: -10000px` and slides in on focus, and the mobile sheet closes on Escape.

### 8.2 Hero (`Hero.astro`)

Layout: asymmetric split. Left column 7/12 holds wordmark, tagline, two CTAs, and a small "what this is" sub-line. Right column 5/12 holds the hero art (`frontpage-art.webp`) inside a `--radius-lg` container with `border: 1px solid var(--line-1)` and `--shadow-2`. On viewports below 900px the columns stack and the art moves above the text.

Wordmark in the left column is set as text (Geist 540) and accompanied by a small node-link SVG glyph echoing the brain motif from the hero art. The art on the right does the heavy visual lifting; the left text stays restrained.

CTAs: primary is "Read the docs" (solid accent on `--accent-ink`), secondary is "View on GitHub" (text + arrow glyph, no border). Both use `--radius-sm`, 44px tall.

No centered text over a gradient. No oversized H1 stunt. No fade-from-black overlay applied via CSS; the raster image already has the right falloff.

### 8.3 Example panel (`ExampleGrid.astro`)

**This replaced the six-card grid.** The landing page shows one tabbed panel, not a 3×2 card matrix. `ExampleCard.astro` is the orphaned remnant of the old design (§7.1). Snippet data lives in `examples.ts`; every snippet is excerpted from a real crate under `examples/` in the monorepo.

Structure: a HUD gradient panel (§12.4) containing a horizontal tablist above a single Shiki-highlighted code panel.

Tabs:

- `flex: 1 1 0` with `min-width: 140px`, scrolling horizontally on overflow. The scrollbar is styled to 4px.
- Full ARIA tablist: `role="tablist"`, roving tabindex, arrow-key navigation.
- State layers three signals at once — text to `--fg-0`, an accent wash (**5%** hover, **8%** active), and a 2px indicator scaling from `scaleX(0.4)` at 50% opacity on hover to `scaleX(1)` at full on active.
- Focus uses `outline-offset: -3px`, the only inset outline on the site. It is required because the tab sits flush inside a clipped scroll container where a positive offset would be cut off.

Panel:

- Headline reserves two lines via `min-height` to stop layout shift between tabs.
- A JS pass equalizes code-box heights for the same reason.
- Two L-bracket corner accents (§12.5) frame the code surface.

No tilt-on-mouse parallax. No glow halo. No scale > 1.0. No background-color transition on the panel.

### 8.4 Causal stack (`CausalStack.astro`)

**This replaced the pillar row.** The three-pillar concept (Causaloid, Context, Effect Ethos) was superseded by a five-band platform diagram: Discover → Model → Act → Govern → Run. `PillarRow.astro` was never built.

Each band is a `.layer` row, `1fr` at base and `184px 1fr` at ≥720px. The fixed first column holds a mono micro-label; the `1fr` column holds chips and prose.

- A 3px accent tick sits on the left edge of each layer and grows from 18px to 28px on hover, while the row takes a 5% accent wash.
- Chips follow the pill pattern (§12.8) and shift their arrow glyph by `translate3d(2px, -1px, 0)` on hover.
- Reticle corners (§12.3) frame the whole stack, gated to appear on `.in-view`.

Entrance is the site's only staggered animation. An inline custom property carries the index — `style={\`--i:${i}\`}` — consumed as `transition-delay: calc(var(--i) * 110ms)`, gated on `.in-view`, with explicit `no-preference` and `reduce` branches.

The `110ms` and `280ms` values are hardcoded and therefore opt out of the §6 token contract. Tokenize them when touched.

### 8.5 Code

There is no `CodeBlock.astro`. Code styling is configuration plus global CSS.

Shiki is configured in `astro.config.mjs` with dual themes:

```js
shikiConfig: {
  themes: { light: 'github-light', dark: 'github-dark' },
  defaultColor: 'dark',
  wrap: true,
}
```

`global.css` forces every `pre.astro-code` onto `--bg-2` and swaps token colours by `[data-theme]`. Two surfaces produce Shiki HTML and they behave differently, which is why the CSS looks redundant:

- Landing-page `<Code>` uses `defaultColor: false`, so each span carries both `--shiki-light` and `--shiki-dark` and no baked colour.
- Markdown fences use `defaultColor: 'dark'`, so dark is inlined and light must override with `!important`.

Inline code is matched as `:not(pre) > code` and takes `--bg-3`, `--radius-sm`, and `0 0.3em` padding. The `:not(pre)` guard keeps the chip styling off code blocks. Both inline and block code inherit `--font-mono` at `--t-mono-sm`.

There is no copy button and no line-number support. Neither was built; add them only with a stated need.

### 8.6 Callout — not built

No `Callout.astro` exists and no content uses one. The `--warn`, `--danger`, and `--ok` tokens are therefore currently unused by any component; they are reserved, not live.

The one shipped instance of the left-rail pattern is the blog blockquote (`blog/[...slug].astro`), which draws `border-left: 3px solid var(--accent)`.

If callouts are added later, follow that shape: a 3px left border in the matching token colour, a `--bg-1` panel, `--line-1` hairlines on the other three sides, no icon by default. An optional icon slot must be a monoline SVG, never an emoji.

### 8.7 Footer

Lives inside `BaseLayout.astro`; there is no separate `SiteFooter.astro`.

The only three-tier grid on the site:

| Breakpoint | Columns |
|---|---|
| Base | `1fr` |
| ≥720px | `1fr 1fr` |
| ≥1024px | `1fr 1fr 1fr 1fr 2fr` |

The trailing `2fr` holds the contributor paragraph, capped at `52ch`. This is the site's only use of the 1024px breakpoint (§4).

Top edge: a 1px `--line-1` divider, nothing more. Background `--bg-0`, no top shadow, no large brand mark. Links hover to `--fg-0`.

Email capture: none.

### 8.8 Theme toggle (`ThemeToggle.astro`)

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

The toggle's behaviour ships as a plain bundled module script, not a client island (§7.3). It has no `:focus-visible` rule of its own and falls back to the global focus ring; the adjacent `.burger` declares one locally, which is the inconsistency noted in §12.9.

### 8.9 Search — indexed, not wired

**Half-built, and it costs on every deploy.** Pagefind is a dependency and the build script runs it:

```json
"build": "astro build && pagefind --site dist"
```

That emits `dist/pagefind/` — a full search index plus `pagefind-component-ui.js` and `pagefind-component-ui.css`. Nothing in `src/` references Pagefind. The index and UI bundle ship to production on every deploy and no page ever loads them.

There is no `SearchTrigger.astro`, no header input, and no ⌘K binding.

Two ways to resolve this, and it needs a decision:

- **Finish it.** Build the trigger and mount the Pagefind UI. Note this would be the site's first client-side framework mount and interacts with §7.3 and §7.4.
- **Drop it.** Remove `pagefind` from `dependencies` and from the build script, and reclaim the index from the deploy.

Leaving it half-wired is the one option with no upside.

If it is finished, the original spec still holds: a 240px input shell with a `⌘K` hint, opening a centered modal at 640px wide on `--bg-1` with `--shadow-3` and `--radius-md`. No backdrop blur; use a flat `rgba(7,11,16,0.72)` scrim.

### 8.10 Diagrams (`astro-mermaid`)

Mermaid renders diagrams in markdown content via the `astro-mermaid` integration, configured `theme: 'dark', autoTheme: true` so diagrams follow the site theme.

Currently used in three blog posts. It is the heaviest thing the site ships — `mermaid` is a large runtime bundle and it loads only on pages that contain a diagram. Keep it that way: never import it into a layout, and never put a mermaid fence on the landing page.

Diagrams are content, not chrome. A diagram belongs in a post or an example page. Site furniture uses the hand-drawn SVG idioms in §12 instead.

---

## 9. Page-type design rules

### 9.1 Landing page

Composed in `pages/index.astro` with a `SectionDivider` (§12.10) between every section:

1. Hero (§8.2)
2. Causal stack (§8.4)
3. Example panel (§8.3)
4. Why DeepCausality — two-column `<dl>` at ≥900px, no hover states
5. Join community — two cards at ≥720px

The dividers are load-bearing. They are what makes the page read as one continuous diagram rather than as stacked sections; do not add a section without one.

`index.astro` carries zero CSS of its own. It is pure composition plus dual JSON-LD (Organization + WebSite).

### 9.2 Documentation pages — not on this site

Documentation lives on a separate Starlight site at `website/docs/`, served at `docs.deepcausality.com`. This site links out to it via `DOCS_URL` in `consts.ts`.

That site maps this design system onto Starlight's own custom properties in `website/docs/src/styles/theme.css`. **It does so by hand-copying literal hex values**, with the source token named in a trailing comment:

```css
--sl-color-gray-2: #aab3bd; /* fg-1 */
--sl-color-black: #070b10;  /* bg-0 */
```

Twenty-six values across both themes. Changing `tokens.css` does not change them, and nothing fails when they diverge. **When you edit a colour token here, update that file in the same commit.**

The docs theme also defines four accent tints that do not exist in this token set: `#0e3a40` and `#b3ecf2` (dark), `#d7f0f3` and `#063e45` (light). If accent tints become useful here too, promote them into `tokens.css` so both sites read one source.

There is no KaTeX and no math pipeline on either site. The earlier spec for display math was never implemented.

### 9.3 Code-example detail pages

Rendered by `ExampleDetail.astro`. Single column inside `--w-prose` throughout — the earlier two-stage `--w-page` → `--w-prose` narrowing was not built, and the flat single column is simpler.

Related crates appear as a horizontal row of pill tags (§12.8). Four L-bracket corner accents frame the code surface, implemented via SVG data-URI masks.

Content comes from the `examples` collection, schema in `content.config.ts`: `title`, `domain`, `summary`, `crates`, `order`, `category`, `further`. Category is a closed enum — `foundations`, `aerospace`, `physics`, `medicine`, `mathematics` — and drives both the Examples dropdown and the five per-category pages.

There are no callouts (§8.6), so run instructions are plain prose.

### 9.4 Blog posts

Pure prose inside `--w-prose`, rendered by `blog/[...slug].astro` on top of `BaseLayout`. There is no `ProseLayout.astro`.

Byline is a single small line: name, date, no avatar. The "egg avatar" pattern stays banned. A "Back to blog" link closes the post; there is no related-posts grid.

The blog index adds a sticky aside at ≥900px (`minmax(0, 1fr) 220px`) with a collapsible year archive.

Two defects live in `blog/index.astro` and should be fixed when that file is next touched:

- `scroll-margin-top: 80px` appears four times, but the header is 56px or 64px (§8.1). Anchor links land 16px low. Use the `--header-h` token from §12.7.
- The 900px breakpoint is duplicated as a JS string in a `matchMedia` call, so changing the CSS breakpoint silently desynchronizes the archive behaviour.

`blog/[...slug].astro` uses `<style is:global>`, which leaks its `.post` rules site-wide. Scope it when convenient.

### 9.5 Static pages

`about`, `community`, `accessibility`, and `overview` carry **zero scoped CSS**. They rely entirely on the `.static-page` and `.prose` rules in `global.css`. That is the correct default for a prose page; add scoped CSS only when a page genuinely needs a component.

`overview/index.astro` is the exception and the warning. At 479 lines it introduces a private `.dc-*` diagram vocabulary — bands with a 2px accent left rail, chips, `▼` flow arrows — used on no other page. It is internally consistent, but it is a parallel design system, and it breaks one convention outright: `.dc-diagram` uses `--radius-sm` where every other framed panel uses `--radius-md`.

Do not extend `.dc-*`. If those diagram primitives are needed elsewhere, promote them into `global.css` as shared idioms first.

There are no monograph pages. The earlier spec for an academic register with a PDF download band and a BibTeX citation block was never built and is removed.

---

## 10. Accessibility

Target: WCAG AA everywhere, AAA on body prose.

**The target is currently missed in two places**, both recorded with measurements in §2:

- Light-mode `--accent` is 4.12:1 on `--bg-0`. Every body-size link in light mode is below AA, as is the primary CTA (`--accent-ink` on `--accent`).
- `--fg-2` is 4.26:1 in dark mode and fails on every surface. Use `--fg-1` for text; keep `--fg-2` for ticks and rules.

Do not restate the AA claim as satisfied until those two are fixed.

The rest holds:

- All interactive elements have a visible focus ring: `outline: 2px solid var(--accent); outline-offset: 2px`, declared once globally on `:focus-visible`. Never remove the outline without providing an equivalent. See §12.9 for the one justified deviation.
- `:focus-visible`, not `:focus`, so the ring appears for keyboard traversal and not for mouse clicks.
- The accent ring is the focus ring; do not mix with `box-shadow` glows.
- Skip-link in the header that jumps to `#main`, visible only on focus.
- Heading levels are strictly hierarchical inside each page; no jumping from `<h1>` to `<h3>`.
- All non-decorative images carry meaningful `alt`. Decorative SVGs use `aria-hidden="true"`.
- Text arrows and separator glyphs (§12.6) are always `aria-hidden="true"`; they are decoration, and a screen reader announcing "rightwards arrow" after every link is noise.
- `prefers-reduced-motion: reduce` zeroes the duration tokens (§6).
- Interactive controls meet the 44px touch-target minimum. `.btn` is 44px; the header's 32px and 36px icon controls are below it and are a known exception, mitigated by generous surrounding padding.

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
- JS on the landing page: the theme guard, the IntersectionObserver, the header script, and the ExampleGrid tablist. All are small hand-written modules; there is no framework runtime (§7.3).
- Self-hosted fonts: `Geist` and `JetBrains Mono` only, variable woff2, subset to Latin and Latin Extended, `font-display: swap`.

Two build-level items that do not show up in a per-page budget:

- **Mermaid** is the heaviest dependency and loads only on the three blog posts that contain a diagram (§8.10). Keep it off every other route.
- **Pagefind** emits an index and a UI bundle into `dist/` on every build that no page loads (§8.9). It costs deploy size and nothing else until search is either finished or removed.

DOM cost guard: a noise/grain overlay is allowed only as a fixed, `pointer-events: none` pseudo-element on the page background. It must never sit inside a scrolling container.

---

## 12. Conventions — one form each

The site's character comes from a handful of repeated moves, not from the token table. A component that uses every token correctly and none of these will still look foreign.

`global.css` calls this the "futurism layer". The unifying theme is instrumentation: ticks, brackets, coordinate indices, node dots, hairline rules. The page should read like a technical instrument panel.

**Each convention below has exactly one canonical form.** Where the code currently carries several, the canonical one is marked and the variants are listed as debt. Do not add a new variant.

### 12.1 The eyebrow

Canonical: **the global `.eyebrow` class in `global.css`.** Mono, uppercase, tracked, preceded by a 12px accent tick at 0.7 opacity, flex gap `0.55em`.

```css
.eyebrow { font-family: var(--font-mono); display: flex; align-items: center; gap: 0.55em; }
.eyebrow::before { content: ''; flex: 0 0 12px; height: 1px; background: var(--accent); opacity: 0.7; }
```

The tick is what turns a small heading into an instrument annotation. Without it the label is just an overline.

**Debt.** Nine files redeclare `.eyebrow` locally — Hero, WhyDeepCausality, ExampleGrid, ExampleDetail, CategoryList, `blog/index`, `blog/[...slug]`, `examples/index`, `404`. Only three re-apply `font-mono`, so the other six silently lose the treatment. `404.astro` is the correct version; `SiteHeader.astro` sets eyebrow size without the mono family and is the clearest instance of the bug.

The tick itself has three implementations at two sizes: 12px in `global.css`, 10px in `ExampleCard` and `blog/index`, and an absolutely-positioned 8px variant in `Explainer`. The gap is `0.55em` in two places and `0.5em` in a third.

Fix: delete all nine local redeclarations and let the global rule win. Three of them also declare dead `.eyebrow` CSS with no matching element in the markup (Hero, ExampleGrid, WhyDeepCausality).

### 12.2 The coordinate eyebrow

A variant that splits the label into a dim index and a brighter name, so a section reads as telemetry. Renders as `— 01 / 04  MODEL`.

```css
.eyebrow-coord { display: inline-flex; gap: 0.6em; align-items: baseline; letter-spacing: 0.06em; }
.eyebrow-coord .ix { color: var(--fg-2); }
.eyebrow-coord .lb { color: var(--fg-1); }
```

Worn alongside `.eyebrow`. This is the one correct use of `--fg-2` on something text-like: the index is decoration, not content.

### 12.3 Reticle corners

The site's signature move. Four L-shaped brackets pinned to a panel's corners, accent at 55% opacity, rising to full on hover.

```html
<div class="reticle-host">
  <span class="reticle reticle-tl"></span>
  <span class="reticle reticle-tr"></span>
  <span class="reticle reticle-bl"></span>
  <span class="reticle reticle-br"></span>
</div>
```

Each corner is a 12px square with two borders removed and one radius set, offset `-1px` so it overlaps the host's own 1px border exactly. Base rules live in `global.css`; used by six components.

**The radius must match the host.** Hero overrides all four to `--radius-lg` because its frame is the one `--radius-lg` element on the page.

### 12.4 HUD gradient panel

Canonical framed container:

```css
background-image: linear-gradient(180deg,
  color-mix(in srgb, var(--bg-1) 70%, transparent) 0%,
  var(--bg-1) 100%);
border: 1px solid var(--line-1);
border-radius: var(--radius-md);
box-shadow: var(--shadow-1);
```

The panel appears to catch light along its upper edge. This does not contradict the gradient ban in §2; that rule targets page and hero backgrounds, and this is a surface treatment on a bounded panel.

**Debt.** Four verbatim copies (CausalStack, Explainer, ExampleGrid, JoinCommunity) plus a `--bg-2` 80% variant in ExampleDetail. This should be one `.panel` utility in `global.css`.

`--radius-md` is part of the convention. `overview/index.astro` uses `--radius-sm` for `.dc-diagram` and is wrong.

### 12.5 L-bracket corner accents

Distinct from reticles. Corner marks framing a **code surface** rather than a whole panel, drawn in `color-mix(in srgb, var(--accent) 60%, var(--line-2))`.

Canonical: **10×10 pseudo-elements**, `border-width: 1px 0 0 1px` and `0 1px 1px 0`.

**Debt.** Three implementations of one motif: pseudo-elements at 10px in `ExampleGrid` and `overview`, and SVG data-URI masks at 14px in `ExampleDetail`. The first two share an identical colour expression, so they were copied. Collapse all three into a `.corner-brackets` utility beside `.reticle`.

### 12.6 Text arrows and separators

Directional affordances are literal glyphs with `aria-hidden="true"`. Never SVG icons, never emoji.

| Glyph | Meaning |
|---|---|
| `→` | Forward, open, continue |
| `↗` | External link, leaves the site |
| `←` | Back |
| `▼` | Expand; flow-downward in diagrams |
| `·` | Separator inside a meta line |

Write the character directly. Do not mix `→` and `&rarr;` for the same meaning; both currently appear.

### 12.7 Fixed measurements

Three magic numbers recur and each needs one token.

**Header height.** The header is 56px base, 64px at ≥900px. `blog/index.astro` compensates for it with `80px` in four places, a value that matches nothing on the site, so anchor links land 16px low. Introduce `--header-h`, set it at both breakpoints, and consume it in every `scroll-margin-top` and sticky `top`.

**Frame width.** Four home components hardcode a panel width and none agree: 880px (JoinCommunity), 900px (CausalStack), 980px (Explainer and ExampleGrid). The landing page's left and right edges therefore do not line up between sections. Add one `--w-panel` token and point all four at it.

**Measure.** `--measure` is 68ch and only `.static-page` uses it. Components write literal `ch` values instead: `44ch`, `52ch`, `56ch`, `60ch`. Consolidate to two — a prose measure and a shorter lede measure — and tokenize both.

Also missing: `font-weight: 540` is the deliberate heading weight and appears as a literal six times. Add `--fw-heading`.

### 12.8 Pill chip

`--radius-pill`, `1px solid var(--line-1)`, `--t-body-sm`. Hover promotes the border to `--line-2` and the text to `--fg-0`.

Used for crate tags, category rows, and stack chips. Four components implement it separately; it should be one class.

### 12.9 Focus ring

Canonical: the global `:focus-visible` rule in `global.css`. **Components should not declare their own.**

```css
:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; border-radius: var(--radius-sm); }
```

**Debt.** Four offsets are in use: `2px` (global and SiteHeader), `-3px` (ExampleGrid), `4px` (ExampleCard), and none. Only the ExampleGrid inset is justified — its tab sits inside a clipped scroll container where a positive offset is cut off. Keep that one, document it inline, and delete the others.

`.btn`, `.chip`, `.community-card`, and `.theme-toggle` declare no focus rule and correctly fall back to the global. That is the intended pattern, not an omission.

### 12.10 Section divider

A full-width hairline with three accent node dots at 25%, 50%, 75%:

```html
<svg viewBox="0 0 100 4" preserveAspectRatio="none" data-anim-draw>
```

`<line>` and `<path>` carry `pathLength="100"` so the dash math is resolution-independent. It is the network motif reduced to its minimum, and it is the connective tissue of the landing page.

Draw-in is driven by one site-wide IntersectionObserver in `BaseLayout` that adds `.in-view`; `global.css` animates `stroke-dashoffset` from 100 to 0 on any `data-anim-draw` element. Both reduced-motion branches are written.

Scroll-triggered draw-in runs once per element and does not count against the one-perpetual-animation budget in §6.

### 12.11 Hover vocabulary

Interactive feedback uses three moves, in this order of preference.

1. **Border promotion.** `--line-1` → `--line-2`. Cheap, and it carries most of the site's feedback.
2. **Accent wash.** `background: color-mix(in srgb, var(--accent) 5%, transparent)` for hover, **8%** for selected or active. Six percentages currently exist (5, 8, 12, 32, 38, 55); use 5 and 8 for interaction and treat the rest as one-offs.
3. **Lift.** Canonical: `transform: translate3d(0, -2px, 0)` at `--dur-med`. Canonical press: `translateY(1px)` at `--dur-fast`.

**Debt.** The lift is written three ways — `translate3d(0,-2px,0)` at `--dur-med` in ExampleCard, `translateY(-1px)` at `--dur-fast` in JoinCommunity, and the Hero press. Distance, direction, and duration all vary, and `translate3d` forces GPU promotion while `translateY` may not, so they do not even composite alike.

One exception is sanctioned: the hairline list (§12.12) animates `padding-left`, which §6 otherwise bans.

### 12.12 Hairline list

The standard list. No bullets, no cards.

```css
li + li { border-top: 1px solid var(--line-1); }
a { transition: padding-left var(--dur-med) var(--ease-out); }
a:hover { padding-left: var(--space-3); }
```

Three verbatim copies (`ExamplesList`, `blog/index`, `blog/[...slug]`). Animating `padding-left` is a deliberate, narrow exception: a single text link in a hairline row is cheap to reflow. **Do not generalize it to a card or a grid item.**

### 12.13 Breakpoints

Two tiers. Write new components against these and nothing else.

| Value | Role |
|---|---|
| **720px** | Stack → columns; the padding ramp |
| **900px** | Layouts needing real horizontal room: hero split, header nav, blog sidebar |

Mobile-first: every query is `min-width`. There is no `max-width` query on the site.

**Debt.** Three one-off breakpoints exist: 600px (only in the orphaned `Explainer`), 480px (only in `blog/index`), and 1024px (only in the `BaseLayout` footer). The 900px value is also duplicated as a JS string in `blog/index`.

### 12.14 Applying the conventions

- A new **panel** takes the HUD gradient panel (§12.4), reticle corners (§12.3), and an eyebrow (§12.1). That combination is the house style.
- A new **list** takes the hairline list (§12.12) and nothing else.
- A new **interactive row** promotes its border and takes a 5% accent wash.

---

## 13. Anti-patterns (banned)

These are forbidden in this codebase. PR reviewers reject them on sight.

1. Centered hero text over a CSS gradient.
2. `rounded-2xl` + `shadow-md` soft-card stack on rows of three.
3. Generic 3-equal-cards horizontal feature row without internal asymmetry.
4. Tailwind. Any utility-class framework.
5. Inter font. Serif on technical UI surfaces.
6. Emojis as icons. Egg/Lucide-user-icon avatars. Generic stock avatars.
7. Purple or magenta accent glows.
8. Outer-glow box-shadows on buttons.
9. Glassmorphism panels with `backdrop-filter: blur(...)` over the hero or a content surface. **One sanctioned exception:** a 2px blur on the mobile menu scrim in `SiteHeader.astro`. A scrim is a dismissible overlay, not a content panel; the ban targets frosted panels over readable content.
10. Animating `width`, `height`, `top`, `left`, `margin`, `padding`, large `background-color`, or large `box-shadow`.
11. Custom mouse cursors.
12. Auto-playing video or audio anywhere.
13. Scroll-jacking. Scroll-hijack horizontal panels on the landing page.
14. Marketing filler verbs: "elevate", "unleash", "seamless", "next-gen", plus the styleguide ban list ("delve into", "shed light on", "game-changer", "unlock the potential", "not only … but also").
15. `h-screen` for the hero. Use `min-height: 100dvh` if a viewport-bound height is ever needed.
16. Every-paragraph-leading-icon-bullet patterns.
17. Three-column footer with a newsletter form. Email capture is out.
18. Redeclaring a convention from §12 locally instead of using the shared rule.
19. A new breakpoint outside 720px and 900px without a stated reason.
20. A hardcoded transition duration. It silently opts out of the reduced-motion contract (§6).

---

## 14. Token file

The canonical token set lives in `src/styles/tokens.css`. That file is the source of truth referenced by everything else.

Two consumers sit outside it and do not update themselves:

- `website/docs/src/styles/theme.css` hand-copies 26 colour values (§9.2).
- The duration values duplicated in JS at `SiteHeader.astro` and `ExampleGrid.astro` (§6).

Tokens named in §12.7 as missing — `--header-h`, `--w-panel`, `--fw-heading`, and a tokenized measure pair — are not yet in the file.

---

## 15. Open follow-ups

- **Search.** Pagefind indexes on every build and nothing consumes it (§8.9). Finish it or remove it.
- **Light-mode accent contrast.** `#0a8a98` fails AA for body links and the primary CTA (§2.1). Needs a colour decision.
- **Convention consolidation.** The debt listed under §12 — one eyebrow, one bracket utility, one panel class, one lift, one focus ring.
- **Orphaned components.** `ExampleCard.astro` and `Explainer.astro` are unreferenced but hold live vocabulary (§7.1). Decide: revive, move to `reverted/`, or extract the glyph map first.
- A second language. Astro i18n is scaffolded with `en` only; the design system does not change per locale.
- A second accent for editorial content. Reserved, not chosen.

End of document.
