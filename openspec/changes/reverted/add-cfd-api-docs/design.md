## Context

The measured public surface of `deep_causality_cfd`, taken from `lib.rs` with the three glob
re-exports expanded:

| Kind | Count | | Module | Count |
|---|---|---|---|---|
| struct | 149 | | `types/flow` | 112 |
| fn | 73 | | `solvers/dec` | 37 |
| re-export (other crate) | 21 | | `types/flow_config` | 30 |
| enum | 15 | | `solvers/qtt` | 29 |
| const | 14 | | `tensor_bridge` | 25 |
| type | 13 | | `theories` | 13 |
| trait | 12 | | `navigation` | 9 |
| **total** | **297** | | `coordinate` | 7 |
| | | | `traits` | 7 |
| | | | `types` (other) | 5 |

Plus `deep_causality_physics::quantities::*`, ~105 names the crate re-exports but does not define.

Constraints that shape the design:

- The crate is `publish = false`, so **docs.rs will not host this**. Whatever is published, this repo
  publishes.
- The site is Astro 7 with MDX and `glob`-loader content collections (`blueprints`, `examples`,
  `tutorial`), each with a Zod-validated frontmatter schema and a shared `DocDetail.astro` shell.
  A fourth collection fits the existing pattern exactly.
- Documentation that names source drifts. Five citations on the site are or were stale; four of them
  broke during a single refactor in this repo's recent history.

## Goals / Non-Goals

**Goals:**

- A reader can find any public name and its signature, always current.
- A reader new to the crate can see the shape of ~400 names on one screen and knows which ~10 to
  start with.
- Curated prose stays small enough that keeping it true is realistic.
- Drift fails the build.

**Non-Goals:**

- Re-documenting the physics quantity types the crate re-exports but does not own.
- Replacing the tutorial, blueprints, or examples. This is the "what is there" layer, not "how do I".
- Publishing to docs.rs, or changing `publish = false`.
- Restyling rustdoc to imitate the site. Generated output is presented as generated (D7).

## Decisions

### D1 — Hybrid: generated signatures, curated orientation

Rustdoc is deployed for the full surface; hand-written pages carry only orientation and link into it.

The split follows what each side is good at. Rustdoc knows every signature, generic bound and
feature gate, and is regenerated from the code so it cannot be wrong. It cannot tell a reader that of
297 names, the seven `CfdConfigBuilder` entries are the door, or that `Cases → Configured → Marched →
Swept → Judged` is a phase order enforced in the type system. That is what the guide is for.

*Alternatives considered.* **Rustdoc only** — complete and free, but a newcomer meets 400
alphabetised names with no path through them, which is the problem this change exists to solve.
**Curated only** — best-looking, but 400 hand-written entries is an authoring and maintenance load
this repo has already shown it cannot carry without drift.

### D2 — The curated guide is organized by surface area, not alphabetically

Nine pages, each a coherent piece of the API a reader would learn as a unit:

| Page | Covers |
|---|---|
| Configuration | `CfdConfigBuilder` and its 7 entries, the config containers and builders, the scenario value types (`Mesh`, `Body`, `Seed`, `Observe`) |
| Workflow DSL | `CfdFlow`, the march pipelines, runs, pauses, forks, `Report` |
| Study grammar | the campaign phases and their typestates, `GateSeq`, `Verdict`, `StudyEffect` |
| Coupling and stages | `Coupling`, `PhysicsStage` and the shipped stages |
| Solvers | the 5 families (DEC, QTT incompressible/immersed, compressible, acoustic) and their states |
| Boundary zones | `BoundaryZone` and the zone types |
| Trait seams | the 12 traits, and which to implement to extend the crate |
| Tensor bridge | codec, operators, masks, projection |
| Navigation | the GNSS-denial estimation layer |

The measured sizes above informed this split but are **not published**: a per-area count goes stale on
every refactor and a reader acts on none of it. Coverage is established by check D4.3 instead.

Each page states what the area is for, shows one real compiling example taken from a committed
example or verification program, and links every name it introduces into rustdoc.

### D3 — Re-exported physics quantities are named and linked, not duplicated

The Configuration and Solvers pages state that the crate re-exports the physics quantity newtypes so
a CFD program imports from one crate, list the categories, and link to the physics crate's reference.
Duplicating 105 entries the crate does not define would double the drift surface for no gain.

### D4 — The parity gate is three checks, all cheap

1. **Symbol existence.** Extract every identifier in site content that is written as inline code and
   matches a Rust-symbol shape, intersect with the crate's public name set (derived the same way
   this design's inventory was), and fail on any name that is not public and not an allow-listed
   word.
2. **Line-citation accuracy.** For every `path.rs:N` or `path.rs:N-M` in site content, verify the
   file exists and — where the citing page quotes code in an adjacent fence — that the quoted first
   line appears within the cited range.
3. **Coverage.** Every public name is attributable to a documented surface area or to the documented
   re-export set. This replaces the published per-area counts: the invariant a count stood in for is
   "nothing is undocumented", and a check states that directly without putting a stale-able number on
   the page.

All three run over `website/cfd/src/content/**` in CI. Check 2 is what would have caught all five known
drifts. Check 1 is what catches a renamed or retired symbol, which is what this session's config
unification would have triggered had the site named those builders.

*Alternative considered:* trusting review. Rejected on evidence — four citations broke in one
refactor and nobody noticed until asked.

### D5 — Rustdoc output is a build artifact, and the deploy moves to CI to produce it

`cargo doc --no-deps -p deep_causality_cfd` writes into `website/cfd/public/api/`, produced at deploy
time and git-ignored. Committing generated HTML — measured at **1,262 files, 28 MB** — would put a
churning artifact into every review diff and create a second way for the reference to be stale (a
checked-in build older than the code).

That forces the deploy path. The site is deployed by Cloudflare Workers Builds today, and
**Cloudflare's build image has no Rust toolchain** (Go, Node, Bun, Python, Ruby, PHP, Java, Elixir,
Erlang, Clojure, Swift, .NET — no Rust, and no version override for a toolchain that is absent). A
Cloudflare-side build cannot generate the reference, so the deploy moves to a GitHub Actions workflow
that has `cargo doc`, builds the site, and calls `wrangler deploy`.

The build itself stays defined in `wrangler.toml`: `wrangler deploy` runs its `[build] command`, so
the site is built once by the same definition a local deploy uses, and CI adds no second place the
build is described.

*Alternatives considered.* **Commit the output** — works with no build change, but reverses the
invariant above. **CI commits it** — automates the committing without removing the churn or the
staleness path. **Host rustdoc separately** (GitHub Pages) — leaves the CFD deploy untouched but
splits hosting across two origins for a reference that belongs to this site.

*Operational consequence, not optional:* the Cloudflare-side build must be disabled when this lands.
Two enabled paths race, and the Cloudflare one publishes a site with no `/api/` — which presents as
the reference intermittently 404ing rather than as a misconfiguration.

### D6 — Versioning is by co-deployment, and this is a stand-in for docs.rs

The reference describes the code at the commit it was built from, because both ship together from
this repo. No version selector, no historical builds.

This whole generated half exists only because the crate is `publish = false`. A published crate gets
its rustdoc built and hosted by docs.rs automatically, per version, for free. When
`deep_causality_cfd` publishes, docs.rs takes over: the `make docs` step and the `public/api/`
deployment are dropped, and the reference links point at docs.rs instead. The curated pages are
unaffected — they are this repo's own prose, and they are the half docs.rs never provides.

### D7 — The curated layer is built from the existing design system; generated output is not

`website/cfd/src/styles/global.css` names `website/web/DESIGN.md` as binding on this site, with
`website/web_design/` as its descriptive companion. The CFD site's improvement over the marketing site
is that each §12 convention is declared **once** there as a shared utility — `.eyebrow`,
`.eyebrow-coord`, `.reticle`/`.reticle-host`, `.panel`, `.corner-brackets`, `.chip`, `.hairline-list`,
and one focus ring — where the marketing site accumulated nine `.eyebrow` redeclarations and three
L-bracket implementations.

The reference pages therefore add no shell and no convention: they reuse `DocDetail.astro` and those
utilities. §13.18 bans redeclaring a convention locally, and the board's own guidance is that the
site's character comes from a few repeated moves rather than the token table — a page using every
token correctly and none of the idioms still reads as foreign. `tokens.css` stays a byte-identical
mirror (`pnpm check:tokens`); anything site-local goes to `tokens-cfd.css`.

Generated rustdoc is the deliberate exception. It cannot conform — its markup and type scale belong to
the generator — so it is left plain and the link into it says so, rather than restyled into a near-copy
of the site that behaves differently. Imitation would mislead about which surface the reader is on;
honest labelling costs nothing.

## Risks / Trade-offs

**Rustdoc's look differs from the site** → Accepted and made explicit (D7). It is reached from a link
that names it as generated output. Restyling would be a maintenance burden that also misleads, and
`DESIGN.md` §13 governs pages this repo authors, not a generator's output.

**The curated pages can still drift in prose** → The gate catches renamed and moved symbols, not
prose that is merely out of date. Mitigated by keeping each page short and by having every code
example lifted verbatim from a committed, CI-run example rather than written for the page.

**Symbol check false positives** → Site prose contains inline code that is not a Rust symbol (file
names, CLI flags, field strings like `"truth_state"`). The check needs an allow-list, and an
over-eager version would be turned off rather than fixed. Start narrow: only identifiers in
`UpperCamelCase` or ending in `()`, with an explicit allow-list file.

**Deploy-time doc build adds minutes** → `cargo doc --no-deps` on one crate is cheap relative to the
existing CI. If it becomes an issue it can be cached on the crate's fingerprint.

## Migration Plan

1. Rustdoc build (`make docs`), the CI deploy workflow, `/api/` link. Verifiable immediately: the
   full surface is browsable. Requires disabling the Cloudflare-side build in the same window.
2. The parity gate, run against existing content — it should immediately flag the known
   `compressible_march_run.rs:441-444` drift. Fix that, then turn the gate on as blocking.
3. The `reference` collection, index page, and detail route, with two pages (Configuration, Trait
   seams) to prove the shape.
4. The remaining seven pages.
5. `Cargo.toml` `documentation` field.

Steps 1 and 2 deliver value independently of 3-5 and can ship alone.

## Open Questions

- Does `website/web` (the project site) want the same treatment for the other 26 crates? Out of scope
  here; the pattern this change establishes would be the template.

**Resolved:** per-area name counts are not published (see D2). They would go stale on every refactor
and add a maintenance liability to the layer whose value depends on staying true.
