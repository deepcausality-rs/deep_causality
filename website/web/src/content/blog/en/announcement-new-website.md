---
title: DeepCausality got a new website
description: A new Astro-based site replaces the previous Hugo build, with newly authored documentation, a code-first landing page, and a refreshed visual system.
date: 2026-05-17
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

The DeepCausality project got a new website.

The previous site was a Hugo build that lived in a separate repository. It had become difficult to maintain, and its content lagged the actual state of the framework. The new site is built with Astro, lives in this monorepo at `website/web/`, deploys to Cloudflare Pages, and is the source of truth for project documentation going forward.

## What changed

**Landing page.** The front page now leads with six code examples from distinct engineering domains: async event inference on Tokio, Pearl-style counterfactual reasoning, a Causal State Machine wired to sensors, an aerospace flight-envelope monitor, biomedical tumor-treatment optimization, and a Maxwell-field derivation. Each card links to a detail page with file references and run instructions. Every snippet is excerpted from a real, compilable example crate in `examples/`.

**Documentation.** The previous documentation tree was largely retired. The new tree is hand-authored from the current state of the crates, organized into Getting Started, Concepts, and a new overview section. The Concepts section covers the Causal Monad, the Causaloid, the Context, Higher-Kinded Types, the Causal Discovery Language, the Causal State Machine, the Effect Ethos, the Effect Propagation Process, the Uncertain and MaybeUncertain types, and the Uniform Mathematical Foundation.

**Design.** Dark mode is the default; a light theme ships from day one and respects `prefers-color-scheme`. The visual system uses one cyan accent calibrated to the hero artwork rather than the usual SaaS palette. A small futurism layer carries through every section: coordinate-style eyebrows in monospace, network-motif section dividers that draw themselves on first view, reticle corners on key surfaces, and a quiet accent halo on the hero plate and on the docs sidebar's "you are here" item. 

## Where to look

* The site is live at [www.deepcausality.com](https://www.deepcausality.com).
* The Concepts section is at [/docs/overview/why/](https://www.deepcausality.com/docs/overview/why/), and the example detail pages are at [/examples/](https://www.deepcausality.com/examples/).
* The full site source lives in the monorepo at [website/web](https://github.com/deepcausality-rs/deep_causality/tree/main/website/web). Issues and pull requests are welcome.

## About

DeepCausality is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. The project is hosted as a sandbox project at the Linux Foundation for AI and Data. Learn more on [GitHub](https://github.com/deepcausality-rs/deep_causality) and join the [DeepCausality-Announce mailing list](https://lists.lfaidata.foundation/g/DeepCausality-announce).

Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)
