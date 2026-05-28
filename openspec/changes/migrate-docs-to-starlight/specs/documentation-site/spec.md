## ADDED Requirements

### Requirement: Starlight documentation application

The project SHALL provide a Starlight-based documentation application located in `website/docs`, built on Astro and producing a fully static site.

#### Scenario: Docs app builds to static output

- **WHEN** the documentation app is built
- **THEN** it produces a static site (no SSR) under its own `dist/` directory, using Starlight as the documentation framework

#### Scenario: Docs app is self-contained

- **WHEN** a developer inspects `website/docs`
- **THEN** the directory contains its own Astro/Starlight configuration and content, independent of `website/web`

### Requirement: Long-form documentation content is migrated

The documentation app SHALL contain the migrated long-form documentation: the `concepts` pages, the deep `getting-started` walkthroughs, and the deep `overview` pages (including Literature, Innovations, and Problem).

#### Scenario: Concepts pages present

- **WHEN** the docs site is served
- **THEN** every concepts page that previously existed under `website/web/src/content/docs/concepts` is reachable on the docs site

#### Scenario: Frontmatter mapped to Starlight conventions

- **WHEN** a migrated page is built
- **THEN** its title, description, and sidebar position are expressed using Starlight's frontmatter conventions, and the page renders without frontmatter-schema errors

#### Scenario: Internal links resolve

- **WHEN** the docs site is built
- **THEN** intra-docs links resolve to docs-origin paths, and links to examples resolve to absolute `www.deepcausality.com/examples/*` URLs

### Requirement: Code highlighting

The documentation app SHALL render fenced code blocks with syntax highlighting that supports both a light and a dark theme.

#### Scenario: Code block highlighted in both themes

- **WHEN** a page containing a fenced code block is viewed in light mode and in dark mode
- **THEN** the code is syntax-highlighted appropriately for the active theme

### Requirement: Backlinks graph view

The documentation app SHALL provide a graph view of the documentation via the Starlight Obsidian theme.

#### Scenario: Graph view available

- **WHEN** a reader opens the docs site
- **THEN** a backlinks/graph view of the documentation pages is available

### Requirement: Shared visual identity with the marketing site

The documentation app SHALL adopt the marketing site's visual identity, including its color tokens, fonts, and logo.

#### Scenario: Consistent look across origins

- **WHEN** a visitor moves from `www.deepcausality.com` to `docs.deepcausality.com`
- **THEN** the documentation site presents the same colors, fonts, and logo as the marketing site, reading as one product

### Requirement: Documentation search

The documentation app SHALL provide full-text search over the documentation content.

#### Scenario: Search returns docs results

- **WHEN** a reader searches a term present in the documentation
- **THEN** matching documentation pages are returned
