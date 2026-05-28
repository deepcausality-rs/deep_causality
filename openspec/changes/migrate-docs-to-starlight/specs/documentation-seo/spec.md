## ADDED Requirements

### Requirement: Per-origin sitemap and robots

The documentation origin SHALL serve its own `sitemap-index.xml` covering only documentation URLs, and its own `robots.txt` referencing that sitemap.

#### Scenario: Docs sitemap scoped to the docs origin

- **WHEN** `https://docs.deepcausality.com/sitemap-index.xml` is fetched
- **THEN** it lists documentation URLs under the `docs.deepcausality.com` origin

#### Scenario: Robots references the docs sitemap

- **WHEN** `https://docs.deepcausality.com/robots.txt` is fetched
- **THEN** it references the documentation sitemap

### Requirement: Canonical URLs on the docs origin

Each documentation page SHALL declare a canonical URL on the `docs.deepcausality.com` origin.

#### Scenario: Canonical points to the docs origin

- **WHEN** a documentation page is rendered
- **THEN** its canonical link references the corresponding `https://docs.deepcausality.com/...` URL

### Requirement: Redirects from the former docs paths

Every documentation URL that previously existed under `www.deepcausality.com/docs/*` SHALL issue a permanent (301) redirect to its corresponding `https://docs.deepcausality.com/*` URL.

#### Scenario: Old docs URL redirects

- **WHEN** a request is made to a former `https://www.deepcausality.com/docs/<path>` URL
- **THEN** it responds with a 301 redirect to the corresponding `https://docs.deepcausality.com/<path>` URL

### Requirement: Marketing sitemap excludes migrated docs

The marketing site's sitemap SHALL NOT list any `/docs/*` URLs once the documentation has migrated.

#### Scenario: No docs entries in the www sitemap

- **WHEN** the marketing site's sitemap is generated
- **THEN** it contains no `/docs/*` entries

### Requirement: Search Console coverage of both origins

The project SHALL register a DNS-verified Domain property for `deepcausality.com` in Google Search Console and submit the documentation sitemap to it.

#### Scenario: Domain property covers subdomains

- **WHEN** the `deepcausality.com` Domain property is configured
- **THEN** it reports coverage for both the `www` and `docs` origins, and the documentation sitemap is submitted to it

### Requirement: Cross-origin linking

The marketing site and the documentation site SHALL link to each other so the two origins are discoverable from one another.

#### Scenario: Marketing links to docs

- **WHEN** a visitor is on the marketing site
- **THEN** a link to `docs.deepcausality.com` is reachable from the site navigation or a documentation landing page

#### Scenario: Docs links back to marketing

- **WHEN** a reader is on the documentation site
- **THEN** a link back to `www.deepcausality.com` is reachable
