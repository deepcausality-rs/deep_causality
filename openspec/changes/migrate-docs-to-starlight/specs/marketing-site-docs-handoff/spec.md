## ADDED Requirements

### Requirement: Marketing site retains short getting-started and overview

The marketing site SHALL keep a short getting-started and a short overview after the long-form documentation migrates to the documentation site.

#### Scenario: Short summaries remain on www

- **WHEN** a visitor browses the marketing site after the migration
- **THEN** a short getting-started and a short overview are present on `www.deepcausality.com`

### Requirement: Documentation landing page on the marketing site

The marketing site SHALL provide a Documentation landing page that links to the documentation site and to the Rust API reference on docs.rs.

#### Scenario: Landing page links outward

- **WHEN** a visitor opens the Documentation landing page on the marketing site
- **THEN** it links to `https://docs.deepcausality.com` and to the project's docs.rs API reference

### Requirement: Migrated long-form pages removed from the marketing site

The marketing site SHALL no longer serve the long-form documentation pages that have migrated to the documentation site (the concepts pages, the deep getting-started walkthroughs, and the deep overview pages), nor the bespoke docs rendering components for them.

#### Scenario: Migrated pages no longer on www

- **WHEN** a former long-form documentation URL on `www.deepcausality.com/docs/*` is requested
- **THEN** the marketing site no longer serves that page directly (it is redirected to the documentation origin)

### Requirement: Examples remain on the marketing site

The marketing site SHALL continue to host the examples content and the `/examples/*` routes.

#### Scenario: Examples still served by www

- **WHEN** an `https://www.deepcausality.com/examples/<path>` URL is requested
- **THEN** the marketing site serves the example as before
