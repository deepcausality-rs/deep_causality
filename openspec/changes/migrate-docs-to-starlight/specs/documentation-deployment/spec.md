## ADDED Requirements

### Requirement: Standalone documentation build

The documentation app SHALL build independently of the marketing site, with its own dependency manifest, lockfile, and Astro version, and SHALL NOT be a member of the `website/web` pnpm workspace.

#### Scenario: Independent dependency resolution

- **WHEN** the documentation app's dependencies are installed
- **THEN** they resolve from the docs app's own manifest and lockfile, independent of `website/web`

#### Scenario: Independent Astro version

- **WHEN** the docs app pins an Astro version required by its Starlight release
- **THEN** that version is independent of the Astro version used by `website/web`

#### Scenario: pnpm overrides location

- **WHEN** the docs app needs a dependency override
- **THEN** the override is declared in the docs app's `pnpm-workspace.yaml`, not in its `package.json`

### Requirement: Dedicated documentation origin

The documentation app SHALL be served from `docs.deepcausality.com` by a dedicated Cloudflare Worker distinct from the marketing site's Worker.

#### Scenario: Docs served from the subdomain

- **WHEN** a request is made to `https://docs.deepcausality.com/`
- **THEN** it is served by the dedicated documentation Worker, not by the marketing site's Worker

#### Scenario: Site URL configured for the subdomain

- **WHEN** the docs app is built
- **THEN** its configured site URL is `https://docs.deepcausality.com`, so generated absolute URLs and the sitemap use that origin

### Requirement: Independent deployment

A change to the documentation SHALL rebuild and redeploy only the documentation, leaving the marketing site untouched, and a change to the marketing site SHALL rebuild and redeploy only the marketing site.

#### Scenario: Docs-only change

- **WHEN** a change touches only files under `website/docs/`
- **THEN** the deployment pipeline rebuilds and redeploys only the documentation Worker

#### Scenario: Marketing-only change

- **WHEN** a change touches only files under `website/web/`
- **THEN** the deployment pipeline rebuilds and redeploys only the marketing site Worker
