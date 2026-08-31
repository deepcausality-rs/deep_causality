# DeepCausality CFD — project website

Astro static site for the `deep_causality_cfd` crate. Served at
`cfd.deepcausality.com`.

## Run it

```bash
pnpm install
pnpm dev        # local server
pnpm build      # -> dist/
pnpm check      # astro check; needs TypeScript 6.x, see below
pnpm check:tokens   # verify the token mirror has not drifted
```

There is **no Bazel target for this site**. `pnpm build` is the only build path.
`website/cfd/node_modules` is listed in `.bazelignore` so the Rust build ignores
it, and that is the whole of the Bazel involvement: the root `MODULE.bazel`
declares no `npm_*` extension and no `BUILD.bazel` exists anywhere under
`website/`.

## Deploy

Cloudflare Workers Builds. The dashboard supplies the root directory
(`/website/cfd/`) and the deploy command (`npx wrangler deploy`); the build
lives in `wrangler.toml` as a `[build]` command, so an empty dashboard build
field cannot break the deploy.

The Worker is **`deep-causality-cfd-prod`**, spelled with hyphens throughout,
and `wrangler.toml` must carry that exact name. See
[`../README.md`](../README.md) for the two ways this fails quietly.

## Design

The binding spec is [`../web/DESIGN.md`](../web/DESIGN.md); the descriptive
companion is [`../web_design/`](../web_design/). This site follows both. Two
deliberate differences, both of which implement recommendations the spec makes
but the marketing site has not yet applied:

1. **Every §12 convention is a shared utility in `global.css`.** The eyebrow,
   panel, reticle, corner-bracket, chip and hairline-list rules are declared
   once. No component redeclares them, so the nine-copy `.eyebrow` divergence
   documented in §12.1 cannot start here.
2. **The tokens §12.7 names as missing exist.** `--fw-heading`, `--header-h`,
   `--w-panel`, `--measure-lede`, plus `--stagger` / `--dur-draw` / `--dur-node`
   live in `src/styles/tokens-cfd.css`. Every duration is a token, so the
   reduced-motion contract in §6 holds with no exceptions.

`src/styles/tokens.css` is a **byte-identical mirror** of
`../web/src/styles/tokens.css`. Do not edit values in it — edit the source and
re-copy, then run `pnpm check:tokens`. Site-local tokens go in `tokens-cfd.css`.

### Inherited defect

The light-mode accent (`#0a8a98`) fails WCAG AA at 4.12:1, which affects body
links and the primary CTA. This is inherited from the shared token set and is
recorded in DESIGN.md §2.1 and §10. Fixing it is a colour decision for the whole
project, not something this site should diverge on.

## Content rules

From `openspec/notes/archive/cfd-website/cfd-docs-website.md`:

- **Facts are rooted in the crate. Authoring is for the prime audience.**
  Every claim on this site traces to `deep_causality_cfd` in this repo: a
  committed run artifact, a verification baseline, a study, or the crate
  source. That constraint is absolute and is what makes the site citable.

  Structure, order, emphasis and vocabulary are a separate decision, and they
  serve the prime audience: a working CFD engineer who runs Fluent, SU2,
  OpenFOAM or an in-house code and arrives cold from a link. Pages are ordered
  by what that reader needs in order to decide whether to keep reading, which
  is rarely the order the crate README argues in. Evidence may lead. Material
  may be cut from one page and kept on another. A change to the crate README is
  not by itself a reason to change a page.

  Where the two pull apart, the fact holds and the framing moves. Landing-page
  sections stay one-per-component under `src/components/home/`.
- **A toolbox for a named problem class**, with an explicit line between what
  works today and what is aspirational. That line is the `/roadmap/` page, and
  no item moves up a list without a committed artifact.

  Three of its lists are stages of one ladder: *works today*, *building*,
  *open*. The fourth, **not pursued**, is a different kind of entry — a decision
  with its reason, not a gap waiting to be filled, so it carries its own status
  mark and nothing is promoted out of it on effort alone. An unstated non-goal
  reads as an omission; a stated one reads as a choice.
- **Blueprints, not rustdoc.** `/blueprints/` is task-shaped: sweep a parameter,
  gate against a placard, fork a running simulation, pick a solver.
- **One citable validation page.** `/validation/` is the adoption document —
  per target: what was validated, against which reference, to what number.
- **Honest boundaries, stated where they will be hit.** `/boundaries/` leads
  with the four hypotheses the project refuted by running them.

Every number on the site is copied from a committed run artifact under
`deep_causality_cfd/verification/`, `deep_causality_cfd/studies/`, or an
example's `output.txt`. Where no artifact exists, the page says so on the row.

Each figure lives in exactly one place, split by shape:

| Content | Home | Why |
|---|---|---|
| Blueprints, worked examples | `src/content/**/en/*.mdx` | Prose with a walkthrough. Frontmatter carries the facts a listing needs, so index and detail cannot disagree. |
| Validation records, capability boundaries | `src/data/*.ts` | Matrices, not prose. Rendered as tables and typed at compile time. |

The MDX collections are declared in `src/content.config.ts`, same `glob` +
locale-stripping pattern as `website/web`.

Prose follows `docs/writing_guides/AiStyleguide.md` and `ElementsOfStyle.md`.

## Toolchain note

`astro check` requires **TypeScript 6.x**. TypeScript 7.0 dropped the
programmatic API the checker uses (withastro/roadmap#1321), so `typescript` is
pinned to `^6.0.3` across all three sites. Do not let a routine upgrade move it
to 7.x.

`@astrojs/markdown-satteri` is also pinned, in `pnpm-workspace.yaml`, because
two resolved copies break Bazel's `public_hoist_packages`. See
[`../README.md`](../README.md) for both constraints.

`shiki` is pinned in the same file. `shiki-rust-themes.mjs` derives its themes
from the `bundledThemes` of this project's own copy, while astro highlights with
the copy its `^4.0.2` dependency resolves; a single override keeps those the
same shiki.

## Diagrams

The fork tree in `src/components/home/ForkTree.astro` is the site's first figure
and sets the convention. Follow it rather than inventing a second mechanism.

- **Hand-drawn inline SVG**, in the instrument vocabulary: hairline strokes,
  accent node circles, no fills, no gradients, no raster.
- **Geometry in the SVG, words in HTML.** SVG text scales with the viewBox, so a
  label that reads well on a desktop lands at eight or nine pixels on a phone.
  Labels go in an HTML row beneath, using the site's own type tokens.
- **Animate through the shared contract.** Put `data-anim-draw` on the `<svg>`,
  give every `<line>` and `<path>` `pathLength="100"`, and let the site-wide
  observer add `.in-view`. No per-diagram script; the reduced-motion contract in
  §6 then holds for free.
- **Order circles the way the figure should be read.** The node stagger is
  `circle:nth-of-type(n)` in `global.css`, currently declared to eight.
- **Colour is scoped to the component**, never global. `global.css` carries dash
  and node *timing* only.
- **Never combine `vector-effect: non-scaling-stroke` with the draw-in.** It
  resolves `stroke-dasharray` in device pixels, which fights the
  `pathLength="100"` normalization and renders every stroke as a dashed line
  instead of a drawn one. Set `stroke-width` in user units instead. Note that
  `SectionDivider.astro` still pairs the two, which is why the dividers render
  dashed rather than drawing in.

The Open Graph card at `public/img/social-share.jpg` is built from the same
figure. Its source is committed beside it as `social-share.source.svg`; re-render
with `rsvg-convert -w 1200 -h 630`, then convert to JPEG.

## Deliberate omissions

- **No Pagefind.** The marketing site ships an unread search index on every
  deploy (DESIGN.md §8.9). Not repeated here.
- **No mermaid.** Diagrams on this site are hand-drawn SVG in the instrument
  vocabulary, which keeps the heaviest dependency off every route.
- **No client islands.** Zero framework runtime; interactivity is four small
  module scripts.
