## Why

`deep_causality_cfd` exposes **297 public names** it defines plus **~105 physics quantity types** it
re-exports, and none of them have a published API reference. The crate is `publish = false`, so
docs.rs has no page for it; no CI job builds rustdoc; and the crate's `Cargo.toml` points
`documentation` at `https://docs.rs/deep_causality` — a **different crate**. A reader who wants to
know what `CfdConfigBuilder::qtt_march` takes, or which of the 12 traits they must implement to add a
solver, has to read the source.

The CFD website already carries a tutorial, blueprints, and worked examples — the "how do I" layer.
It has no "what is there" layer, and its own prose is where readers currently go looking. That gap is
what this change fills.

There is also direct evidence that hand-written references to source **rot**: unifying the config
builders shifted `shared/world.rs` by five lines and silently broke four line citations in the
blueprints, and a fifth citation (`compressible_march_run.rs:441-444`) had already drifted by 25
lines before that. Any API documentation added here has to be built so this cannot happen quietly.

## What Changes

- **Publish rustdoc.** A `cargo doc --no-deps` build of `deep_causality_cfd` is deployed with the
  site at `/api/`, giving complete, always-correct signatures for the full public surface. This is
  generated, never hand-maintained.
- **Add a curated API guide.** A new `reference` content collection carries hand-written orientation
  pages in the site's own design, one per surface area, each linking into rustdoc for signatures:
  configuration entries, the workflow DSL and its phase typestates, the study grammar, solver
  families, coupling stages, trait seams, the tensor bridge, navigation, and IO/snapshot.
- **Add an API index page** at `/reference/` listing every surface area, so a reader sees the whole
  API's shape on one screen. No per-area name counts: a count goes stale on every refactor and a
  reader acts on none of it, so coverage is a CI check instead of a published number.
- **Build on the site's existing design system.** `website/web/DESIGN.md` is binding on this site and
  its §12 conventions are already declared once in `website/cfd/src/styles/global.css`; the reference
  pages reuse those and the existing detail shell, adding no convention of their own.
- **Gate all of it against the code.** A CI check verifies that every Rust symbol named in site
  content still exists in the crate's public API, that every `path:line` citation still points at the
  quoted code, and that every public name is attributable to a documented area. The check fails the
  build rather than letting documentation drift silently.
- **Fix the `documentation` field** in `deep_causality_cfd/Cargo.toml`, which currently points at
  another crate's docs.rs page.
- The ~105 re-exported physics quantity types are **named and linked out** to the physics crate
  rather than duplicated: the CFD crate re-exports them for import convenience, it does not define
  them.

Non-breaking: this change adds documentation and CI, and touches one metadata field. No library code
changes.

## Capabilities

### New Capabilities

- `cfd-api-reference`: the published API reference contract — that a reference exists and is
  reachable, that generated signatures cover the full public surface, what the curated guide must
  orient the reader to, how re-exported surface is treated, and that the reference is versioned with
  the code it describes.

### Modified Capabilities

- `documentation-code-parity`: extend the parity principle from in-code docstrings to **published**
  documentation. Adds a requirement that a symbol or source location named in published docs is
  machine-verified against the code, so a refactor that moves or renames it fails the build instead
  of leaving a stale reference on the site.

## Impact

**Website** (`website/cfd/`) — a new `reference` content collection in `src/content.config.ts`, a
new `src/pages/reference/` index and `[...slug]` detail route reusing the existing `DocDetail` shell,
a nav entry in `SiteHeader.astro`, and the curated MDX pages under `src/content/reference/en/`. The
pages add no styles of their own — the §12 conventions they need are already shared utilities in
`global.css`, and `tokens.css` stays a byte-identical mirror (`pnpm check:tokens`).

**Build** — a rustdoc build step producing `website/cfd/public/api/`, wired so the site deploy
carries it. `.gitignore` for the generated output; it is a build artifact, not committed source.

**CI** — a new documentation-parity job (symbol existence + line-citation verification) covering
`website/cfd/src/content/**`.

**Crate** — one line in `deep_causality_cfd/Cargo.toml` (`documentation`).

**Existing drift to fix as part of this** — the `compressible_march_run.rs:441-444` citation in
`blueprints/en/couple-multiphysics.mdx`, which the new gate would immediately flag.
