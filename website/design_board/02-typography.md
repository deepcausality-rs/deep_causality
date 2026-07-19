# 02 — Typography

## Families

Two families, both self-hosted. No CDN request at runtime.

| Token | Stack | Use |
|---|---|---|
| `--font-sans` | `'Geist Variable'`, `'Geist'`, `ui-sans-serif`, `system-ui`, `-apple-system`, `'Segoe UI'`, `Roboto`, `sans-serif` | UI, headings, prose |
| `--font-mono` | `'JetBrains Mono Variable'`, `'JetBrains Mono'`, `ui-monospace`, `SFMono-Regular`, `Menlo`, `monospace` | Code, technical values, eyebrows, tabular numbers |

Both ship as variable woff2 with a `100 900` weight axis, split into Latin and
Latin-Extended subsets, `font-display: swap`. Files live in
`website/web/public/fonts/`; the faces are declared in
`website/web/src/styles/fonts.css`.

Serif is banned. Inter is banned.

## Scale

From `tokens.css:72-98`. Line-height and letter-spacing ship as separate tokens
(`--lh-*`, `--ls-*`) rather than baked into the size.

| Token | rem | px @16 | Line height | Letter spacing | Use |
|---|---|---|---|---|---|
| `--t-display` | 3.75 | 60 | `--lh-display` 1.02 | `--ls-display` -0.022em | Hero only |
| `--t-h1` | 2.5 | 40 | `--lh-h1` 1.08 | `--ls-h1` -0.018em | Page H1 |
| `--t-h2` | 1.875 | 30 | `--lh-h2` 1.15 | `--ls-h2` -0.012em | Section heading |
| `--t-h3` | 1.375 | 22 | `--lh-h3` 1.25 | `--ls-h3` -0.005em | Subsection |
| `--t-h4` | 1.125 | 18 | `--lh-h4` 1.35 | 0 | Card title |
| `--t-body` | 1.0 | 16 | `--lh-body` 1.6 | 0 | Default body |
| `--t-body-sm` | 0.9375 | 15 | 1.55 | 0 | Caption, meta |
| `--t-mono-sm` | 0.875 | 14 | 1.55 | 0 | Inline code |
| `--t-mono-block` | 0.9375 | 15 | 1.6 | 0 | Code block |
| `--t-eyebrow` | 0.75 | 12 | `--lh-eyebrow` 1.2 | `--ls-eyebrow` 0.08em | Uppercase eyebrow labels |

Negative tracking scales with size. Display gets -0.022em; by `--t-h4` it is
zero. Large text set at default tracking looks loose, and small text set tight
looks cramped, so the two move against each other.

## Weights

| Role | Weight | Note |
|---|---|---|
| Headings | 540 | Set on `h1`–`h4` in `global.css:46` |
| Body | 400 | |
| Strong | 600 | |
| Mono | 450 | |

540 is the deliberate choice and the reason the variable font is worth its
bytes. It reads as medium with presence. 600 and above looks heavy against a
dark field, because light-on-dark type already gains apparent weight from halation.

## Base element rules

`global.css:46-62` sets the floor:

- `h1`–`h4` carry weight 540, `color: var(--fg-0)`, `margin: 0`. Vertical rhythm
  is the container's job, never the heading's.
- `p` gets `margin: 0 0 var(--space-4) 0`. Bottom margin only, so no collapse
  arithmetic.
- `code, pre` get `--font-mono` at `--t-mono-sm`.
- Inline code, matched as `:not(pre) > code`, gets `--bg-3`, `--radius-sm`, and
  `0 0.3em` padding. The `:not(pre)` guard keeps the chip styling off code blocks.

## Measure

`--measure: 68ch`. Long-form prose caps line length there.

Code blocks ignore the measure and breathe to the column width. No prose section
runs full-bleed at desktop width.

`.static-page` applies the measure to `p` and `li` but not to headings, so a long
`h2` can outrun the paragraph beneath it. That is intentional; headings are
scanned, not read.

## Hierarchy in practice

The site separates levels using three channels at once: size, colour, and family.

| Level | Size | Colour | Family |
|---|---|---|---|
| Eyebrow | `--t-eyebrow` | `--fg-2` | mono, uppercase, tracked |
| Heading | `--t-h1`…`--t-h4` | `--fg-0` | sans, 540 |
| Body | `--t-body` | `--fg-1` | sans, 400 |
| Meta | `--t-body-sm` | `--fg-2` | mono or sans |

Body prose sits at `--fg-1`, not `--fg-0`. Headings take `--fg-0`. The result is
that a page scans as a heading ladder before any word is read, without any
heading needing to grow.

The eyebrow switching to mono is the load-bearing move. It is what makes a
section label read as an instrument annotation rather than as a small heading.
See [06-idioms.md](06-idioms.md).
