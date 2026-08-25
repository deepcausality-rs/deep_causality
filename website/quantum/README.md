# DeepCausality Quantum — project website

Astro static site for the `deep_causality_quantum` crate. Served at
`quantum.deepcausality.com`.

## Run it

```bash
pnpm install
pnpm dev            # local server
pnpm build          # -> dist/
pnpm check          # astro check; needs TypeScript 6.x, see below
pnpm check:tokens   # verify the token mirror has not drifted
```

There is **no Bazel target for this site**. `pnpm build` is the only build path.
`website/quantum/node_modules` is listed in `.bazelignore` so the Rust build
ignores it, and that is the whole of the Bazel involvement. This matches
`website/cfd`, and differs from `website/web` and `website/docs`.

## Deploy

Cloudflare Workers Builds. The dashboard supplies the root directory
(`/website/quantum/`) and the deploy command (`npx wrangler deploy`); the build
lives in `wrangler.toml` as a `[build]` command, so an empty dashboard build
field cannot break the deploy.

The Worker is **`deep-causality-quantum-prod`**, and `wrangler.toml` must carry
that exact name. See [`../README.md`](../README.md) for the two ways this fails
quietly.

## Design

The binding spec is [`../web/DESIGN.md`](../web/DESIGN.md); the descriptive
companion is [`../web_design/`](../web_design/). This site follows both, and it
inherits `website/cfd`'s two deliberate improvements on the marketing site:

1. **Every §12 convention is a shared utility in `global.css`.** The eyebrow,
   panel, reticle, corner-bracket, chip and hairline-list rules are declared
   once. No component redeclares them.
2. **The tokens §12.7 names as missing exist.** `--fw-heading`, `--header-h`,
   `--w-panel`, `--measure-lede`, plus `--stagger` / `--dur-draw` / `--dur-node`
   live in `src/styles/tokens-quantum.css`. Every duration is a token, so the
   reduced-motion contract in §6 holds with no exceptions.

`src/styles/tokens.css` is a **byte-identical mirror** of
`../web/src/styles/tokens.css`. Do not edit values in it. Edit the source, copy
it across, then run `pnpm check:tokens`. Site-local tokens go in
`tokens-quantum.css`.

### Inherited defect

The light-mode accent (`#0a8a98`) fails WCAG AA at 4.12:1, which affects body
links and the primary CTA. This comes from the shared token set and is recorded
in DESIGN.md §2.1 and §10. Fixing it is a colour decision for the whole project,
not something this site should diverge on.

## Content rules

**Every claim on this site traces to something committed in this repository**:
the crate source under `deep_causality_quantum/src/`, a paper under
`deep_causality_quantum/papers/`, the LEAN tree under
`lean/DeepCausalityFormal/Quantum/`, `lean/THEOREM_MAP.md`, or the output of an
example under `examples/quantum_examples/`. That constraint is absolute, and it
is what makes the site citable.

Three consequences shape the pages.

**No roadmap, and no future work.** The site describes the crate as it is today.
When something is not built, the page says what is not built and stops there.
`/formalization/` lists seven targets that carry test witnesses and no LEAN
proof; that list is a statement about today, not a schedule.

**A committed file is not a claim.** Three of the six papers under `papers/` are
not cited from any module. `/papers/` lists them in their own section, so the
presence of a PDF is never read as an implementation.

**Coverage is stated, not implied.** Five of the seven examples in
`quantum_examples` are quantum in subject and do not import this crate.
`/examples/` splits the list on that line and each detail page names the crates
its example actually uses.

Numbers on the site come from a command anyone can re-run:

| Figure | Source |
| --- | --- |
| 197 tests passing | `cargo test -p deep_causality_quantum --all-features` |
| 10 LEAN theorems | the quantum section of `lean/THEOREM_MAP.md` |
| Freeze-check output | `cargo run --release -p quantum_examples --example qcm_freeze_check` |

The test count is the figure the suite reports, not a count of `#[test]`
attributes in the tree; one of those attributes appears inside a doc comment,
so grep says 198 and the runner says 197.

Prose follows `docs/writing_guides/AiStyleguide.md` and
`docs/writing_guides/ClarityTechnicalReporting.pdf`.

Each fact lives in exactly one place, split by shape:

| Content | Home | Why |
| --- | --- | --- |
| Worked examples | `src/content/examples/en/*.mdx` | Prose with a walkthrough. Frontmatter carries the facts a listing needs, so index and detail cannot disagree. |
| API inventory, error variants, theorems, papers | `src/data/*.ts` | Matrices, not prose. Rendered as tables and typed at compile time. |

The MDX collection is declared in `src/content.config.ts`, same `glob` plus
locale-stripping pattern as `website/cfd` and `website/web`.

## Pages

| Route | Carries |
| --- | --- |
| `/` | Five sections: the model, the freeze gate, the five-band stack, the evidence, the fit |
| `/qcm/` | The factorization, the Markov condition, the Q-TOL threshold, C₃-exclusion faithfulness |
| `/operators/` | Density matrices, the Choi–Jamiołkowski isomorphism, the CPTP checks, the dense kernels |
| `/gates/` | The five kernels and their monad wrappers, the signature-dependent adjoint, the Haruna gates |
| `/verdicts/` | The orthomodular projection lattice and Born read-out to `Prob` |
| `/modalities/` | The verifiable and emergent split, and what the `qpu` feature contains |
| `/formalization/` | Ten proved theorems with Rust witnesses, seven deferred targets |
| `/examples/` | Seven runnable examples, split by whether they import this crate |
| `/papers/` | Six committed papers, split by whether a module cites them |
| `/errors/` | Twelve `QuantumErrorEnum` variants and the modules that raise them |
| `/start/` | The git dependency, the feature flags, the no-std story, the MSRV |

## Diagrams

`src/components/home/SupportsDiagram.astro` is the site's figure and sets the
convention, following `website/cfd`'s `ForkTree.astro`.

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
  `circle:nth-of-type(n)` in `global.css`, declared to eight.
- **Colour is scoped to the component**, never global. `global.css` carries dash
  and node *timing* only.
- **Never combine `vector-effect: non-scaling-stroke` with the draw-in.** It
  resolves `stroke-dasharray` in device pixels, which fights the
  `pathLength="100"` normalization and renders every stroke as a dashed line
  instead of a drawn one.

The Open Graph card at `public/img/social-share.jpg` is built from the same
figure. Its source is committed beside it as `social-share.source.svg`;
re-render with `rsvg-convert -w 1200 -h 630`, then convert to JPEG. The card
uses system faces rather than the vendored woff2 files, which rsvg cannot embed.

## Logo

`public/img/deepcausality-quantum-on-{dark,light}.svg` are copies of
`img/project-logos/quantum/` at the repository root. They are not two renderings
of one file: each variant is drawn in its own theme's tokens, dark carrying
`#5cd4e1` on `#e6edf3` and light carrying `#0a8a98` on `#0b1118`. The header
ships both and shows one, switched on `[data-theme]` the same way `ThemeToggle`
swaps its glyphs.

The lockup is also inlined verbatim into `social-share.source.svg`, because rsvg
cannot reliably resolve an external SVG reference and the card has to stay a
single self-contained file. If the logo is redrawn, re-copy both variants and
re-inline the dark one into the card.

## Toolchain note

`astro check` requires **TypeScript 6.x**. TypeScript 7.0 dropped the
programmatic API the checker uses (withastro/roadmap#1321), so `typescript` is
pinned to `^6.0.3` across all four sites. Do not let a routine upgrade move it
to 7.x.

`@astrojs/markdown-satteri` is also pinned, in `pnpm-workspace.yaml`, because
two resolved copies break Bazel's `public_hoist_packages`. See
[`../README.md`](../README.md) for both constraints.

`shiki` is pinned in the same file. `shiki-rust-themes.mjs` derives its themes
from the `bundledThemes` of this project's own copy, while astro highlights with
the copy its `^4.0.2` dependency resolves; a single override keeps those the
same shiki.

Note that pnpm 11 no longer reads the `pnpm` field from `package.json`, so
`overrides` and `onlyBuiltDependencies` must live in `pnpm-workspace.yaml`.

## Deliberate omissions

- **No Pagefind.** The marketing site ships an unread search index on every
  deploy (DESIGN.md §8.9). Not repeated here.
- **No mermaid.** The one diagram is hand-drawn SVG, which keeps the heaviest
  dependency off every route.
- **No client islands.** Zero framework runtime; interactivity is three small
  module scripts (the observer, the mobile sheet, the theme toggle).

## License

All software source code is licensed under the
[MIT License](https://opensource.org/license/mit/).

All documentation is distributed under the
[Creative Commons Attribution 4.0 International Licence](https://creativecommons.org/licenses/by/4.0/).
