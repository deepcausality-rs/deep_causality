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

Or build it hermetically through Bazel:

```bash
bazel build //website/cfd:build   # -> target-bzl/bin/website/cfd/dist
```

The Bazel target is wired: `@npm_cfd` is registered in the root `MODULE.bazel`
alongside `@npm_web` and `@npm_docs`, and `website/cfd/node_modules` is listed
in `.bazelignore`.

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

### Brand anchor

The marketing site reads its mood from a raster (`frontpage-art.webp`). No
equivalent art exists for this crate, so the anchor here is a **drawn schlieren
figure** in `Hero.astro`: a blunt body, its standing bow shock, streamlines
deflecting across it, and sample nodes on the stagnation line. It uses the same
hairline and node vocabulary as the rest of the site, animates with the existing
draw-in idiom, is theme-aware for free, and costs no raster bytes.

### Inherited defect

The light-mode accent (`#0a8a98`) fails WCAG AA at 4.12:1, which affects body
links and the primary CTA. This is inherited from the shared token set and is
recorded in DESIGN.md §2.1 and §10. Fixing it is a colour decision for the whole
project, not something this site should diverge on.

## Content rules

From `openspec/notes/cfd-website/cfd-docs-website.md`:

- **A toolbox for a named problem class**, with an explicit line between what
  works today and what is aspirational. That line is the `/roadmap/` page's
  three-list structure, and no item moves up a list without a committed artifact.
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

## Deliberate omissions

- **No Pagefind.** The marketing site ships an unread search index on every
  deploy (DESIGN.md §8.9). Not repeated here.
- **No mermaid.** Diagrams on this site are hand-drawn SVG in the instrument
  vocabulary, which keeps the heaviest dependency off every route.
- **No client islands.** Zero framework runtime; interactivity is four small
  module scripts.
