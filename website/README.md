[//]: # (SPDX-License-Identifier: CC-BY-4.0)

# Project websites

This folder holds **four independent Astro projects**, each built and deployed
on its own. The project website changes rarely, the documentation evolves with
the library, and each crate site tracks one crate, so each ships on its own
cadence to its own Cloudflare Worker and hostname.

| Directory | Purpose | Framework | Cloudflare Worker | Domain |
| --- | --- | --- | --- | --- |
| [`web/`](./web) | Website (home, blog, examples, short getting-started/overview) | Astro (custom) | `deepcausality-prod` | https://www.deepcausality.com |
| [`docs/`](./docs) | Reference documentation (concepts, guides, overview, single-PDF export) | [Starlight](https://starlight.astro.build) on Astro | `deepcausality-docs` | https://docs.deepcausality.com |
| [`cfd/`](./cfd) | `deep_causality_cfd`: blueprints, validation status, worked examples, capability boundaries | Astro (custom) | `deep-causality-cfd-prod` | https://cfd.deepcausality.com |
| [`quantum/`](./quantum) | `deep_causality_quantum`: the quantum causal model, operator layer, verdicts, formalization status | Astro (custom) | `quantum` | https://quantum.deepcausality.com |

A fifth directory, [`web_design/`](./web_design), documents the shipped
visual system as implemented. All four sites follow the binding specification,
[`web/DESIGN.md`](./web/DESIGN.md).

The Rust API reference is hosted on
[docs.rs/deep_causality](https://docs.rs/deep_causality).

## Separation of concerns

- **`web/`, the project website.** The landing page, the blog, the examples
  gallery, and short getting-started and overview summaries that link to the
  full docs.
- **`docs/`, the documentation.** Long-form concepts, getting-started
  walkthroughs and the in-depth overview on Starlight, with full-text search,
  code highlighting, and a build-time single-PDF export.
  See [`docs/README.md`](./docs/README.md) for its commands.
- **`cfd/`, the CFD crate site.** Task-oriented blueprints, one citable
  validation-status page, worked examples with committed run output, and a
  measured capability-boundaries page. Every figure is quoted from a committed
  artifact under `deep_causality_cfd/` or an example's `output.txt`.
  See [`cfd/README.md`](./cfd/README.md).
- **`quantum/`, the quantum crate site.** One page per layer of
  `deep_causality_quantum`, a citable formalization-status page pairing each Lean
  theorem with its Rust witness, and a typed API inventory. Every claim traces to
  the crate source, the committed papers, or `lean/THEOREM_MAP.md`.
  See [`quantum/README.md`](./quantum/README.md).

Each project is standalone: its own `package.json`, lockfile, Astro version,
and `wrangler.toml`. A change under one directory rebuilds and deploys only
that Worker.

`cfd/` and `quantum/` each mirror `web/`'s design tokens byte-for-byte in their
own `src/styles/tokens.css`; `pnpm check:tokens` fails if a copy drifts. Any
site-local token lives beside it, in `tokens-cfd.css` or `tokens-quantum.css`.

## Local development

```bash
# Project website
cd web     && pnpm install && pnpm dev   # http://localhost:4321

# Documentation
cd docs    && pnpm install && pnpm dev   # http://localhost:4321

# CFD crate site
cd cfd     && pnpm install && pnpm dev   # http://localhost:4321

# Quantum crate site
cd quantum && pnpm install && pnpm dev   # http://localhost:4321
```

Each project builds to its own `dist/` (`pnpm build`).

All four sites build with `pnpm` only: none declares an npm repository in
`MODULE.bazel` or carries a `BUILD.bazel`. Each site's `node_modules` path is
listed in `.bazelignore`.

## Toolchain constraints

Three pins are deliberate; a routine upgrade must not "fix" them.

**TypeScript stays on the 6 line.** TypeScript 7.0 dropped the programmatic API
that `@astrojs/check` uses, so `pnpm check` fails on 7.x
([withastro/roadmap#1321](https://github.com/withastro/roadmap/discussions/1321)).
All four projects pin `typescript` to `^6.0.3`.

**`@astrojs/markdown-satteri` is deduplicated by an override.**
`@astrojs/markdown-remark` peers `^0.3.1` while `astro` pins an exact patch, so
pnpm resolves two copies. Each project forces one version in its
`pnpm-workspace.yaml`.

**`shiki` is pinned in `cfd/` and `quantum/`.** Each carries a
`shiki-rust-themes.mjs` that derives the site's Rust themes from the
`bundledThemes` of the project's own `shiki`, while astro highlights with
whatever its `^4.0.2` dependency resolves to. Left free, pnpm keeps two copies
and the derived themes cross a version boundary, so each project's
`pnpm-workspace.yaml` forces one.

**`mermaid` in `web/` sits outside its peer range.** `web/` depends on
`mermaid` `^12.0.0`, while `astro-mermaid` 2.1.0 peers
`mermaid: ^10.0.0 || ^11.0.0`.

pnpm 11 does not read the `pnpm` field from `package.json`, so
`overrides` and `onlyBuiltDependencies` must live in `pnpm-workspace.yaml`. An
override placed in `package.json` is silently ignored, and a missing
`onlyBuiltDependencies` entry aborts the Cloudflare install with
`ERR_PNPM_IGNORED_BUILDS`.

## Deployment

All four sites are fully static and deployed as Cloudflare Workers Static Assets during CI.

Custom domains are bound in the Cloudflare dashboard. Each project's
`public/_headers` file sets per-origin caching and security headers.

Each Worker's build configuration lives in the Cloudflare dashboard:

| Setting | Value |
| --- | --- |
| Root directory | `/website/<project>/` |
| Build command | `pnpm run build` |
| Deploy command | `npx wrangler deploy` |

Two misconfigurations fail quietly.

**An empty build command does not fail the build.** Cloudflare installs
dependencies by itself, so the build step reports success, and the run dies
later in the deploy with `The directory specified by the "assets.directory"
field in your configuration file does not exist`, because nothing produced
`dist/`. `cfd/wrangler.toml` and `quantum/wrangler.toml` therefore carry their
own `[build]` command, which wrangler runs before reading `assets.directory`;
there the dashboard field is a fallback. `web/` and `docs/` rely on the
dashboard field alone.

**A wrong Worker name does not fail either.** `wrangler deploy` takes the name
from `wrangler.toml`, so a name that does not match the Worker the build is
attached to creates a second Worker with no custom domain bound, then reports
success while the live site stays unchanged. The four names follow no single
convention: `deepcausality-prod`, `deepcausality-docs`,
`deep-causality-cfd-prod` and `quantum`. Match the dashboard, not the pattern.

## License

All software source code is licensed under the [MIT License](https://opensource.org/license/mit/).

All documentation is distributed under the [Creative Commons Attribution 4.0 International Licence](https://creativecommons.org/licenses/by/4.0/).

The documentation site is built with [Starlight](https://github.com/withastro/starlight), licensed under the [MIT License](https://opensource.org/license/mit/).
