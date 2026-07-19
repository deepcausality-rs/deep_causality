# 01 — Foundations

## Brand anchor

Everything reads from one image: `website/web/public/img/frontpage-art.webp`.

That art fixes the mood. A deep blue-black field. A cyan particle network drifting
through it, dense at the horizon and sparse toward the edges. A white wordmark set
in a geometric sans. A brain glyph drawn as a faceted node-link cage, not as a
filled illustration.

Three consequences follow, and they explain most of the token choices below.

1. **The base is off-black, never `#000`.** The hero sits on a deep blue-black
   field. Pure black underneath it reads as a hole rather than a continuation.
2. **One accent, sampled from the art.** A desaturated cyan. There is no second
   accent and no purple or magenta secondary.
3. **The motif is a network, not a card stack.** Thin lines, sparse dots,
   geometric edges, visible connections. The UI should read as a node-link
   diagram rather than a dashboard of rounded tiles.

Copy register: technical and confident. The reader is an engineer who arrived to
read code.

## Palette — dark (default)

From `website/web/src/styles/tokens.css:10-40`.

| Token | Hex | Role |
|---|---|---|
| `--bg-0` | `#070b10` | Page background, deepest field |
| `--bg-1` | `#0b1118` | Section background, raised surface |
| `--bg-2` | `#121a23` | Code surface, input surface |
| `--bg-3` | `#1a2430` | Hover surface, selected nav, inline code |
| `--line-1` | `#1f2a36` | Hairline divider, default border |
| `--line-2` | `#2a3a4a` | Stronger border, hover border, focused input |
| `--fg-0` | `#e6edf3` | Primary text |
| `--fg-1` | `#aab3bd` | Secondary text, captions |
| `--fg-2` | `#6b7682` | Tertiary, meta, timestamps |
| `--accent` | `#5cd4e1` | Links, focus ring, active nav, ticks and nodes |
| `--accent-ink` | `#062029` | Text on an accent surface |
| `--warn` | `#e3b341` | Inline warnings only |
| `--danger` | `#f47174` | Errors only |
| `--ok` | `#8ed4a8` | Success only |

## Palette — light

From `website/web/src/styles/tokens.css:42-65`. Applied via `[data-theme='light']`.

| Token | Hex | Role |
|---|---|---|
| `--bg-0` | `#ffffff` | Page background |
| `--bg-1` | `#f6f8fa` | Section background |
| `--bg-2` | `#eef2f6` | Code surface, input surface |
| `--bg-3` | `#e2e8ef` | Hover surface, selected nav |
| `--line-1` | `#d8dee4` | Hairline divider, default border |
| `--line-2` | `#c0c8d1` | Stronger border, focused input |
| `--fg-0` | `#0b1118` | Primary text |
| `--fg-1` | `#3b4754` | Secondary text, captions |
| `--fg-2` | `#5b6772` | Tertiary, meta, timestamps |
| `--accent` | `#0a8a98` | Single accent, pulled deeper into teal |
| `--accent-ink` | `#ffffff` | Text on an accent surface |
| `--warn` | `#a25c00` | Inline warnings only |
| `--danger` | `#b42318` | Errors only |
| `--ok` | `#1f7a3a` | Success only |

The accent shifts between themes because the dark-mode cyan washes out on white.
The light value keeps the hue family and pulls it deeper so it carries weight
against `#ffffff`. It does not carry quite enough; see the measurements below.

## Measured contrast

Computed from the shipped hex values, WCAG 2.1 relative luminance. Grades are for
normal-size text: AA needs 4.5, AAA needs 7.0.

### Dark theme, against `--bg-0`

| Token | Ratio | Grade |
|---|---|---|
| `--fg-0` | 16.70 | AAA |
| `--fg-1` | 9.29 | AAA |
| `--fg-2` | **4.26** | **fails AA** |
| `--accent` | 11.24 | AAA |
| `--warn` | 10.14 | AAA |
| `--danger` | 7.01 | AAA |
| `--ok` | 11.40 | AAA |
| `--accent-ink` on `--accent` | 9.60 | AAA |

### Light theme, against `--bg-0`

| Token | Ratio | Grade |
|---|---|---|
| `--fg-0` | 18.96 | AAA |
| `--fg-1` | 9.48 | AAA |
| `--fg-2` | 5.79 | AA |
| `--accent` | **4.12** | **fails AA** |
| `--warn` | 5.16 | AA |
| `--danger` | 6.57 | AA |
| `--ok` | 5.38 | AA |
| `--accent-ink` on `--accent` | **4.12** | **fails AA** |

### Secondary text on raised surfaces

Body text does not always sit on `--bg-0`. These are the ratios that matter on
cards and code panels, and `DESIGN.md` never computes them.

| Pair | Dark | Light |
|---|---|---|
| `--fg-1` on `--bg-1` | 8.93 AAA | 8.90 AAA |
| `--fg-1` on `--bg-2` | 8.26 AAA | 8.43 AAA |
| `--fg-1` on `--bg-3` | 7.39 AAA | 7.68 AAA |
| `--fg-2` on `--bg-1` | 4.10 fails AA | 5.44 AA |
| `--fg-2` on `--bg-2` | 3.79 fails AA | 5.15 AA |
| `--fg-2` on `--bg-3` | 3.39 fails AA | 4.69 AA |
| `--accent` on `--bg-1` | 10.80 AAA | 3.87 fails AA |
| `--accent` on `--bg-2` | 9.99 AAA | 3.66 fails AA |
| `--accent` on `--bg-3` | 8.93 AAA | 3.34 fails AA |

### What these numbers mean in practice

Three findings, in severity order.

**Light-mode links fail AA.** `global.css:33` sets `a { color: var(--accent) }`
site-wide. In light mode that is 4.12:1 on the page background and worse on every
raised surface. Every body-size link in light mode is below the AA threshold.

**Light-mode primary CTA fails AA.** `--accent-ink` on `--accent` is the solid
button pairing, and white on `#0a8a98` is also 4.12:1.

**`--fg-2` is a decorative grey, not a text grey.** In dark mode it fails AA on
every surface including the page background. It is safe for hairline ticks,
coordinate indices, and rules. It is not safe for anything a reader must read.

All three contradict `DESIGN.md` §10, which claims AA everywhere and AAA on body
prose. Remedies are proposed in [08-drift.md](08-drift.md) §1.

## Palette rules

- One accent. `--accent` is the only chromatic colour permitted on interactive
  elements.
- `--warn`, `--danger`, `--ok` appear on inline feedback only. Never on a CTA.
- Pure `#000` is banned. Pure `#fff` is banned for body text; use `--fg-0`.
- No CSS gradient backgrounds. Gradients exist only inside the hero raster.
- No purple or magenta glow. No outer-glow shadow on buttons.

## Theme resolution

1. `localStorage['dc-theme']`, values `dark` or `light`.
2. `prefers-color-scheme`, if nothing is saved.
3. Fall back to `dark`.

A pre-paint inline script in `BaseLayout.astro` sets `data-theme` on the root
element before first paint. It is the sole inline `<head>` script, and it exists
to prevent the flash of wrong theme.

`global.css:24-28` also sets `color-scheme`, so native form controls and
scrollbars follow the theme.
