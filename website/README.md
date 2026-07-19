[//]: # (SPDX-License-Identifier: CC-BY-4.0)

# Project websites

This folder holds **two independent Astro projects**, each built and deployed on
its own. They are deliberately decoupled: the main project site changes rarely,
while the documentation evolves with the project, so each ships on its own
cadence to its own Cloudflare Worker and hostname.

| Directory | Purpose                                                                             | Framework | Cloudflare Worker | Domain |
| --- |-------------------------------------------------------------------------------------| --- | --- | --- |
| [`web/`](./web) | website (home, blog, examples, short getting-started/overview)                      | Astro (custom) | `deepcausality-prod` | https://www.deepcausality.com |
| [`docs/`](./docs) | Reference documentation (concepts, guides, overview, single-PDF export) | [Starlight](https://starlight.astro.build) on Astro | `deepcausality-docs` | https://docs.deepcausality.com |

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

Each project is standalone: its own `package.json`, lockfile, Astro version,
and `wrangler.toml`. A change under `web/` rebuilds and deploys only the
website Worker; a change under `docs/` rebuilds and deploys only the docs
Worker.

## Local development

```bash
# Project website
cd web  && pnpm install && pnpm dev      # http://localhost:4321

# Documentation
cd docs && pnpm install && pnpm dev      # http://localhost:4321
```

Each project builds to its own `dist/` (`pnpm build`).

Both sites also build hermetically under Bazel — `bazel build //website/...`
builds `web` and `docs` together (see the repo `MODULE.bazel` Astro/Node
toolchain section and each project's `BUILD.bazel`).

## Deployment

Both sites are fully static and deployed as Cloudflare Workers Static Assets during CI.

Custom domains are bound in the Cloudflare dashboard. Per-origin caching and
security headers are configured via each project's `public/_headers` file.

## License

All software source code is licensed under the [MIT License](https://opensource.org/license/mit/).

All documentation is distributed under the [Creative Commons Attribution 4.0 International Licence](https://creativecommons.org/licenses/by/4.0/).

The documentation site is built with [Starlight](https://github.com/withastro/starlight), licensed under the [MIT License](https://opensource.org/license/mit/).
