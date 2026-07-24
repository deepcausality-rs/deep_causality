[//]: # (SPDX-License-Identifier: CC-BY-4.0)

# Project websites

This folder holds **three independent Astro projects**, each built and deployed
on its own. They are deliberately decoupled: the main project site changes
rarely, the documentation evolves with the library, and the CFD site tracks a
single crate, so each ships on its own cadence to its own Cloudflare Worker and
hostname.

| Directory | Purpose | Framework | Cloudflare Worker | Domain |
| --- | --- | --- | --- | --- |
| [`web/`](./web) | Website (home, blog, examples, short getting-started/overview) | Astro (custom) | `deepcausality-prod` | https://www.deepcausality.com |
| [`docs/`](./docs) | Reference documentation (concepts, guides, overview, single-PDF export) | [Starlight](https://starlight.astro.build) on Astro | `deepcausality-docs` | https://docs.deepcausality.com |
| [`cfd/`](./cfd) | `deep_causality_cfd`: blueprints, validation status, worked examples, capability boundaries | Astro (custom) | `deepcausality-cfd-prod` | https://cfd.deepcausality.com |

A fourth directory, [`web_design/`](./web_design), is documentation rather
than a site: it describes the shipped visual system as implemented. The binding
specification is [`web/DESIGN.md`](./web/DESIGN.md), which all three sites
follow.

The Rust API reference is generated separately and hosted on
[docs.rs/deep_causality](https://docs.rs/deep_causality).

## Separation of concerns

- **`web/` — the project website.** First-touch content: the landing page, the
  blog, the examples gallery, and short getting-started / overview summaries
  that link out to the full docs.
- **`docs/` — the documentation.** The long-form documentation (concepts,
  getting-started walkthroughs, the in-depth overview) on Starlight, with
  full-text search, code highlighting, and a build-time single-PDF export.
  See [`docs/README.md`](./docs/README.md) for its commands.
- **`cfd/` — the CFD crate site.** Task-oriented blueprints, one citable
  validation-status page, worked examples with committed run output, and a
  measured capability-boundaries page. Every figure is quoted from a committed
  artifact under `deep_causality_cfd/` or an example's `output.txt`.
  See [`cfd/README.md`](./cfd/README.md).

Each project is standalone: its own `package.json`, lockfile, Astro version,
and `wrangler.toml`. A change under one directory rebuilds and deploys only
that Worker.

`cfd/` mirrors `web/`'s design tokens byte-for-byte in
`cfd/src/styles/tokens.css`; `pnpm check:tokens` fails if the two drift. Any
site-local token lives in `cfd/src/styles/tokens-cfd.css` instead.

## Local development

```bash
# Project website
cd web  && pnpm install && pnpm dev      # http://localhost:4321

# Documentation
cd docs && pnpm install && pnpm dev      # http://localhost:4321

# CFD crate site
cd cfd  && pnpm install && pnpm dev      # http://localhost:4321
```

Each project builds to its own `dist/` (`pnpm build`).

All three also build hermetically under Bazel. `bazel build //website/...`
builds `web`, `docs` and `cfd` together; each has its own npm repository
(`@npm_web`, `@npm_docs`, `@npm_cfd`) declared in the repo `MODULE.bazel`, and
its `node_modules` path is listed in `.bazelignore`.

## Toolchain constraints

Two pins are deliberate and should not be "fixed" by a routine upgrade.

**TypeScript stays on the 6 line.** TypeScript 7.0 dropped the programmatic API
that `@astrojs/check` uses, so `pnpm check` fails on 7.x
([withastro/roadmap#1321](https://github.com/withastro/roadmap/discussions/1321)).
All three projects pin `typescript` to `^6.0.3`.

**`@astrojs/markdown-satteri` is deduplicated by an override.**
`@astrojs/markdown-remark` peers `^0.3.1` while `astro` pins an exact patch, so
pnpm resolves two copies. Bazel's `public_hoist_packages` cannot hoist an
ambiguous name and fails the build. Each project therefore forces one version in
its `pnpm-workspace.yaml`.

Note that pnpm 11 no longer reads the `pnpm` field from `package.json`, so
`overrides` and `onlyBuiltDependencies` must live in `pnpm-workspace.yaml`. An
override placed in `package.json` is silently ignored, and a missing
`onlyBuiltDependencies` entry aborts the Cloudflare install with
`ERR_PNPM_IGNORED_BUILDS`.

## Deployment

All three sites are fully static and deployed as Cloudflare Workers Static Assets during CI.

Custom domains are bound in the Cloudflare dashboard. Per-origin caching and
security headers are configured via each project's `public/_headers` file.

## License

All software source code is licensed under the [MIT License](https://opensource.org/license/mit/).

All documentation is distributed under the [Creative Commons Attribution 4.0 International Licence](https://creativecommons.org/licenses/by/4.0/).

The documentation site is built with [Starlight](https://github.com/withastro/starlight), licensed under the [MIT License](https://opensource.org/license/mit/).
