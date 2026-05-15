## ADDED Requirements

### Requirement: Astro project lives at `website/web/`
The website SHALL be implemented as an Astro project rooted at `website/web/` within this monorepo. The project SHALL be self-contained: its own `package.json`, lockfile, and `tsconfig.json`, independent of the Rust Cargo workspace.

#### Scenario: Astro project bootstrap
- **WHEN** a developer runs `pnpm install && pnpm build` inside `website/web/`
- **THEN** the build SHALL succeed and produce a static site under `website/web/dist/`

#### Scenario: Isolation from Rust workspace
- **WHEN** any `cargo` command is run from the repo root
- **THEN** the Astro project's files MUST NOT be picked up as a Cargo workspace member

### Requirement: Cloudflare Pages deployment configuration
The site SHALL build with a configuration compatible with the existing Cloudflare Pages auto-deploy setup (fork branch → beta domain, `main` → production). No GitHub Actions workflow shall be added by this change.

#### Scenario: Cloudflare build settings
- **WHEN** the Cloudflare Pages project is configured against this monorepo
- **THEN** the build command SHALL be `pnpm install && pnpm build`, the output directory SHALL be `dist`, and the project root SHALL be `website/web/`

#### Scenario: Fork branch beta deploy
- **WHEN** a developer pushes to a fork branch
- **THEN** Cloudflare SHALL auto-deploy the build to the configured beta domain without any additional workflow files in this repo

### Requirement: Astro i18n scaffolding
The site SHALL be configured with Astro's built-in i18n routing using `en` as the default locale. The default locale URL prefix SHALL NOT be emitted (English URLs remain unprefixed). Content collections SHALL be structured so additional locales can be added without restructuring existing routes.

#### Scenario: English URLs are unprefixed
- **WHEN** a user visits `/docs/getting-started`
- **THEN** the English version SHALL be served at that path with no `/en/` prefix

#### Scenario: Locale-keyed content collections
- **WHEN** a new locale `de` is added in the future
- **THEN** German content SHALL be added under a `de/` segment in the relevant content collection, and German routes SHALL be served under `/de/...`, without changes to English routes

### Requirement: UI design driven by the design-taste skill
All UI design decisions (layout, typography scale, color system, spacing scale, shadow language, motion, and component architecture) SHALL be produced through the `design-taste-frontend` skill. Generic AI-default patterns — centered hero with gradient background, rounded-2xl shadow-md card stacks, uniform Tailwind-default grids — SHALL NOT be shipped. Component architecture and CSS performance rules from the skill (hardware-accelerated transforms, no layout-thrashing animations) SHALL be enforced in review.

#### Scenario: Skill applied before UI work begins
- **WHEN** a developer starts implementation of any page or reusable component listed in this change
- **THEN** the `design-taste-frontend` skill SHALL have been invoked to produce the design direction, and the resulting tokens / component spec SHALL be the reference the implementation builds against

#### Scenario: Generic-default patterns rejected
- **WHEN** a PR for this change is reviewed
- **THEN** any component matching a flagged generic-default pattern (per the skill's rules) SHALL be sent back for revision rather than merged

### Requirement: Shared layout and brand tokens
The site SHALL provide a shared base layout used by all pages, a brand-token module (colors, typography, spacing) consumed via CSS variables, and a global header/footer.

#### Scenario: Header and footer on every page
- **WHEN** any page is rendered
- **THEN** the page SHALL include the shared site header and footer

#### Scenario: Brand tokens centralized
- **WHEN** a brand color is changed in the central token module
- **THEN** the change SHALL propagate to all pages without per-page edits

### Requirement: All new prose adheres to the AI Styleguide
All prose authored or edited under this change SHALL comply with `docs/writing_guides/Ai Styleguide.md`. Compliance means: em dashes ≤ 4 per 1,000 words; semicolons used where natural; sentence-length variance spanning roughly 3–35 words; no occurrences of the banned phrases "delve into", "shed light on", "game-changer", "unlock the potential", or "not only … but also"; "Additionally" / "Furthermore" paragraph-opener ratio below 0.4; filler words ("very", "really") below 2% of word count; no rotational use of "crucial / vital / essential / significant" as substitutes for "important". The monograph LaTeX source under `papers/src/EPP/` is exempt.

#### Scenario: Banned phrase absent from site content
- **WHEN** the rendered `dist/` is searched for any of the banned phrases
- **THEN** no banned phrase SHALL appear in any page authored under this change

#### Scenario: Em dash density within budget
- **WHEN** any single content page is checked
- **THEN** its em-dash count divided by its word count multiplied by 1,000 SHALL be 4 or less

### Requirement: Content authored as Markdown / MDX via content collections
Documentation, blog, and code-example detail pages SHALL be authored as Markdown or MDX within Astro content collections, with typed frontmatter schemas.

#### Scenario: Invalid frontmatter fails the build
- **WHEN** a content file is added with frontmatter that violates the collection schema
- **THEN** `pnpm build` SHALL fail with a clear error pointing at the offending file
