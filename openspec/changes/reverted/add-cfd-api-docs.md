# Reverted: add-cfd-api-docs — the hosting premise does not hold

## Status

**Reverted during implementation, at 3 of 35 tasks. No living-spec impact.** The change set was
authored and validated (`--strict`), and its artifacts were committed (`e7d69b947`), but the
deployment premise underneath it was falsified while implementing phase 1. Neither delta was synced
into `openspec/specs/`: `cfd-api-reference` was never created as a capability, and the
`documentation-code-parity` addition rode on it.

Date reverted: 2026-08-11 (same day it was created).

## What the change set assumed

That `deep_causality_cfd`'s ~297 public names have no published API reference — true, and the reason
is that the crate is `publish = false`, so docs.rs builds no page for it. The plan was a hybrid: ship
generated rustdoc at `/api/` on the CFD site for complete signatures, plus a curated MDX guide in the
site's design for orientation, with a CI parity gate against drift.

## What falsified it

Two facts collided, and forcing past them was not worth the cost.

**Rust is not in the Cloudflare build image.** The CFD site is deployed by Cloudflare Workers Builds
on push, running `pnpm run build` from `wrangler.toml`. That image ships Go, Node, Bun, Python, Ruby,
PHP, Java, Elixir, Erlang, Clojure, Swift and .NET — and no Rust, with no version override for a
toolchain that is absent (verified against Cloudflare's build-image documentation). `cargo doc`
therefore cannot run where the site is built.

**Every way around that was worse than the problem.** Committing the generated output means 1,262
files and 28 MB of churning HTML in review diffs, plus a second way for the reference to be stale.
Moving the deploy into a GitHub Actions workflow works technically but replaces a working
auto-deploy-on-push with a hand-rolled one, and adds an operational trap: two enabled deploy paths
race, and the Cloudflare one publishes a site with no `/api/`, which presents as the reference
intermittently 404ing rather than as a misconfiguration. Hosting rustdoc on a second origin splits
the reference away from the site that is supposed to carry it.

Each of these fights the two systems that already work — rustdoc's own automation, and the existing
Cloudflare deploy — to substitute for a page that a normal crate release produces for free.

## What happens instead

**The crate gets published, and docs.rs hosts its API reference.** That is the mechanism this change
set was reimplementing by hand. Once `publish = false` is lifted, docs.rs builds and hosts the
rustdoc per version, and the CFD site links to it — a link, not a build step, a workflow, or a
generated artifact in the repo.

The curated-guide half of the proposal is not refuted by any of this: rustdoc will never tell a
reader which of ~297 names to start from, and docs.rs does not change that. If that layer is wanted
later, it should be proposed on its own, without the generated half dragging a deploy rewrite behind
it.

## What was rolled back

Implementation (phase 1, tasks 1.1–1.6, all reverted):

- `build/scripts/docs.sh` and the `make docs` target — deleted.
- `website/cfd/public/api/` (1,262 generated files) — deleted; it was git-ignored and never committed.
- `.gitignore` entry for that directory — reverted.
- The `API` nav entry in `website/cfd/src/components/nav/SiteHeader.astro` — reverted.
- `deep_causality_cfd/Cargo.toml`'s `documentation` field — reverted to its prior value.
- A `cfd_website_deploy.yml` workflow, written and rejected before it ran — deleted.

## One finding worth keeping

The inventory stands regardless of hosting, and is recorded in this change's `design.md`: 297 public
names the crate defines (149 struct, 73 fn, 15 enum, 14 const, 13 type, 12 trait, 21 re-exported from
other crates), concentrated in `types/flow` (112), `solvers/dec` (37), `types/flow_config` (30) and
`solvers/qtt` (29), plus ~105 physics quantity types re-exported through a glob.

Two defects it surfaced are real and outlive this revert:

1. `deep_causality_cfd/Cargo.toml` sets `documentation = "https://docs.rs/deep_causality"` — a
   **different crate**. Publishing the crate fixes this properly; until then the field misdirects.
2. `blueprints/en/couple-multiphysics.mdx` cites `compressible_march_run.rs:441-444` for a block that
   sits at 466. Pre-existing drift, unrelated to this change, still unfixed.
