> Five phases. Phases 1 and 2 deliver value alone and can ship without 3–5.
> Each phase ends with `pnpm -C website/cfd build` clean and the parity gate green.

## 1. Generated reference

- [x] 1.1 Add a `docs` target to the makefile: `cargo doc --no-deps -p deep_causality_cfd --target-dir <tmp>` copying `doc/` into `website/cfd/public/api/`
- [x] 1.2 Git-ignore `website/cfd/public/api/`
- [ ] 1.3 **BLOCKED** — Cloudflare's build image has no Rust toolchain, so `cargo doc` cannot run in the deploy build. Needs a hosting decision (see below)
- [x] 1.4 Verify `cargo doc` emits no new warnings for the crate; the 9 pre-existing intra-doc warnings are recorded, not introduced
- [ ] 1.5 Add the nav entry in `src/components/nav/SiteHeader.astro` and confirm `/api/` resolves in a local build
- [ ] 1.6 Fix `documentation` in `deep_causality_cfd/Cargo.toml` — it points at `docs.rs/deep_causality`, a different crate

## 2. Parity gate

- [ ] 2.1 Script: derive the crate's public name set by expanding `lib.rs` re-exports including the three globs (`theories::*`, `solvers::dec::*`, `physics::quantities::*`)
- [ ] 2.2 Script: symbol check over `website/cfd/src/content/**` — inline code matching a Rust-symbol shape (`UpperCamelCase`, or ending `()`) must be in the public set; allow-list file for the rest
- [ ] 2.3 Script: line-citation check — every `path.rs:N[-M]` resolves, and where the citing passage quotes code in an adjacent fence, the quoted first line falls inside the range
- [ ] 2.4 Run the gate against current content; it must flag `compressible_march_run.rs:441-444` in `blueprints/en/couple-multiphysics.mdx` (quoted block is at 466)
- [ ] 2.5 Fix that citation
- [ ] 2.6 Tune the allow-list until the gate is clean on existing content with no false positives
- [ ] 2.7 Script: coverage check — every public name is attributable to a documented area or the documented re-export set
- [ ] 2.8 Add the gate as a blocking CI job over `website/cfd/src/content/**` and the crate

## 3. Reference collection and routes

- [ ] 3.1 Add a `reference` collection to `src/content.config.ts`: `title`, `area`, `summary`, `order`, `entryPoints` (string[]), `rustdocHref`. No count field — per-area tallies are not published
- [ ] 3.2 Add `src/pages/reference/index.astro` — every area with its summary and entry points on one screen, using the shared `.eyebrow` / `.panel` / `.reticle` utilities from `global.css`
- [ ] 3.3 Add `src/pages/reference/[...slug].astro` reusing `DocDetail.astro`, with a meta rail carrying entry points and a rustdoc link labelled as generated output
- [ ] 3.4 Prove the shape with two pages: **Configuration** (30 names, the 7 `CfdConfigBuilder` entries) and **Trait seams** (12 traits, which to implement to extend the crate)
- [ ] 3.5 Verify both pages pass the parity gate
- [ ] 3.6 Design conformance: no §12 convention redeclared locally (§13.18), no new page shell, no §13 anti-pattern; `pnpm check:tokens` still reports the mirror in sync and any site-local token went to `tokens-cfd.css`

## 4. Remaining curated pages

- [ ] 4.1 **Workflow DSL** — `CfdFlow`, march pipelines, runs, pauses, forks, `Report`; state the phase order and that mis-ordering is a compile error
- [ ] 4.2 **Study grammar** — campaign phase typestates (`Cases → Configured → Marched → Swept → Judged`), `GateSeq`, `Verdict`, `StudyEffect`
- [ ] 4.3 **Coupling and stages** — `Coupling`, `PhysicsStage`, the shipped stages
- [ ] 4.4 **Solvers** — the 5 families and their state types
- [ ] 4.5 **Boundary zones** — `BoundaryZone` and the zone types
- [ ] 4.6 **Tensor bridge** — codec, operators, masks, projection
- [ ] 4.7 **Navigation** — the GNSS-denial estimation layer
- [ ] 4.8 Every code example is an excerpt of a committed, CI-executed program, and each page cites that program
- [ ] 4.9 Re-exported physics quantities named and linked out on the Configuration and Solvers pages, not duplicated

## 5. Acceptance

- [ ] 5.1 Coverage check (D4.3, CI not page): every public name is attributable to a documented area or the documented re-export set; report holes. The result is a check, not a published number
- [ ] 5.2 `pnpm -C website/cfd build` clean; `/reference/` and `/api/` both resolve; every rustdoc link from a curated page resolves
- [ ] 5.3 Parity gate green over all content
- [ ] 5.4 Read each page against the code once, for prose the gate cannot check
- [ ] 5.5 Read the reference next to a blueprint and a tutorial page: same shell, same idioms, reads as one site
- [ ] 5.6 Prepare the commit message
