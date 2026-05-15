## ADDED Requirements

### Requirement: Migration source is the `ctx/` snapshot only
Content migration SHALL draw exclusively from the `ctx/` directory in this monorepo, which is a one-time snapshot of Markdown and images from the external Hugo repository. The external Hugo repo SHALL NOT be modified by this change.

#### Scenario: Migration touches only `ctx/`
- **WHEN** a developer migrates content
- **THEN** the source files SHALL be read from `ctx/` within this monorepo, and no operation SHALL be performed against the external Hugo repository

### Requirement: Blog posts migrate from `ctx/content/`
All blog posts present in `ctx/content/` SHALL be migrated into the Astro `blog` content collection under `website/web/src/content/blog/`. Frontmatter SHALL be normalized to match the Astro collection schema (title, date, author, tags, summary).

#### Scenario: Blog post round-trips
- **WHEN** an existing Hugo blog post is migrated
- **THEN** the resulting Astro page SHALL render the same title, body, and publication date as the source, with valid frontmatter accepted by the collection schema

#### Scenario: Frontmatter validation
- **WHEN** a migrated blog post is missing a required field
- **THEN** `pnpm build` SHALL fail with an error identifying the missing field and source file

### Requirement: Static assets migrate to Astro `public/`
Brand and content imagery from `ctx/static/` SHALL be migrated into `website/web/public/`. The front-page hero art currently at `ctx/static/img/frontpage-art.webp` SHALL be placed at `website/web/public/img/frontpage-art.webp` and referenced by the landing page.

#### Scenario: Front-page hero art available
- **WHEN** the landing page is rendered
- **THEN** the hero art SHALL be served from `/img/frontpage-art.webp`

#### Scenario: Migrated image paths resolve
- **WHEN** a migrated blog post references an image originally at `ctx/static/img/<name>`
- **THEN** the migrated page SHALL reference the image at the corresponding path under `/img/` and the image SHALL load with HTTP 200

### Requirement: Curated allowlist for non-blog evergreen pages
A small allowlist of evergreen non-blog pages from `ctx/content/` (about, license, accessibility, community, and similar static pages) SHALL be migrated. A contact page SHALL NOT be migrated; the new site has no contact form. The legacy long-form documentation pages on the live deepcausality.com site SHALL NOT be migrated verbatim and SHALL be considered superseded by the newly authored documentation.

#### Scenario: About / community / accessibility routes resolve
- **WHEN** the site is built
- **THEN** static routes at `/about/`, `/community/`, and `/accessibility/` SHALL exist with content rebranded to "dynamic causality" and with all outbound links pointing at the current monorepo

#### Scenario: No `/contact/` route is generated
- **WHEN** the site is built
- **THEN** no `/contact/` route SHALL exist; the contact page from the legacy site is intentionally excluded

### Requirement: `ctx/docs/` intro material is used as a source, not migrated verbatim
The high-level intro files in `ctx/docs/` (`INTRO.md`, `CORE.md`, `DEEP_DIVE.md`, `HAFT.md`, `ETHOS.md`, `DISCOVERY.md`, `PHYSICS.md`, `TOPOLOGY.md`, `TENSOR.md`, `UNIFORM_MATH.md`) SHALL be treated as raw source material for the newly authored `/docs/concepts/` and `/docs/reference/` pages. They SHALL be rewritten to apply the "dynamic causality" rebrand and the AI Styleguide; their original file structure SHALL NOT be preserved on the new site.

#### Scenario: Intro file content reflected in new pages
- **WHEN** an intro file in `ctx/docs/` covers a topic that has a corresponding new concept or reference page
- **THEN** the new page SHALL incorporate the relevant content from the intro file, rewritten under the rebrand and styleguide

#### Scenario: Legacy docs not migrated
- **WHEN** the migration is complete
- **THEN** no page under `/docs/` SHALL have been produced by migration from `ctx/content/`; all `/docs/` content SHALL be newly authored

### Requirement: `ctx/` retirement requires user approval
The `ctx/` directory SHALL NOT be deleted as part of this change. Its removal is a separate, user-approved action once migration is verified complete.

#### Scenario: `ctx/` remains after migration
- **WHEN** the migration tasks are complete
- **THEN** the `ctx/` directory SHALL still exist in the monorepo and SHALL be untouched apart from being read
