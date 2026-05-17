---
title: DeepCausality has a new website
description: A new Astro-based site replaces the previous Hugo build, with newly authored documentation, a code-first landing page, and a refreshed visual system.
date: 2026-05-17
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

The DeepCausality project has a new website.

The previous site was a Hugo build that lived in a separate repository. It had become difficult to maintain, and its content lagged the actual state of the framework. The new site is built with Astro, lives in this monorepo at `website/web/`, deploys to Cloudflare Pages, and is the source of truth for project documentation going forward.

## What changed

**Framing.** The public description moves from "hyper-geometric computational causality" to "dynamic causality." The hyper-geometric framing was an early formal scaffold; it never landed with the engineers who are the project's primary audience. The new framing is closer to what the framework actually does for someone reading the code.

**Landing page.** The front page now leads with six code examples from distinct engineering domains: async event inference on Tokio, Pearl-style counterfactual reasoning, a Causal State Machine wired to sensors, an aerospace flight-envelope monitor, biomedical tumor-treatment optimization, and a Maxwell-field derivation. Each card links to a detail page with file references and run instructions. Every snippet is excerpted from a real, compilable example crate in `examples/`; nothing is invented for the marketing surface.

**Documentation.** The previous documentation tree was largely retired. The new tree is hand-authored from the current state of the crates and the EPP monograph, organized into Getting Started, Concepts, Guides, Reference, and Monograph sections. The Concepts section covers the Causal Monad, the Causaloid, the Context, Higher-Kinded Types, the Causal Discovery Language, the Causal State Machine, the Effect Ethos, the Effect Propagation Process, the Uncertain and MaybeUncertain types, and the Uniform Mathematical Foundation. Each page is grounded in the actual source, with struct definitions and method names that match `git HEAD`.

**Design.** Dark mode is the default; a light theme ships from day one and respects `prefers-color-scheme`. Typography is self-hosted (`Geist Variable` for sans, `JetBrains Mono Variable` for monospace), with no CDN at runtime. The visual system uses one cyan accent calibrated to the hero artwork rather than the usual SaaS palette. A small futurism layer carries through every section: coordinate-style eyebrows in monospace, network-motif section dividers that draw themselves on first view, reticle corners on key surfaces, and a quiet accent halo on the hero plate and on the docs sidebar's "you are here" item. Motion runs under `prefers-reduced-motion: no-preference` and is skipped otherwise.

**Infrastructure.** Cloudflare Pages auto-deploys every push: fork branches go to a preview domain; merges to `main` go to production. Search is provided by Pagefind at build time. There are no analytics, no cookies, and no tracking pixels.

## What is still in flight

The monograph section is scaffolded but its content is not yet ported from the LaTeX sources in `papers/src/EPP/`. The per-crate Reference pages and the deeper end-to-end Guides are deferred to follow-on work. A handful of legacy URLs from the Hugo site do not yet have redirect entries; if you find a broken inbound link, please open an issue.

## Where to look

* The site is live at [www.deepcausality.com](https://www.deepcausality.com).
* The Concepts section is at [/docs/concepts/](https://www.deepcausality.com/docs/concepts/), and the example detail pages are at [/examples/](https://www.deepcausality.com/examples/).
* The full site source lives in the monorepo at [website/web](https://github.com/deepcausality-rs/deep_causality/tree/main/website/web). Issues and pull requests are welcome.

## About

DeepCausality is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. The project is hosted as a sandbox project at the Linux Foundation for AI and Data. Learn more on [GitHub](https://github.com/deepcausality-rs/deep_causality) and join the [DeepCausality-Announce mailing list](https://lists.lfaidata.foundation/g/DeepCausality-announce).

Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)
