## ADDED Requirements

### Requirement: Public framing rebranded to "dynamic causality"
All marketing and conceptual copy on the site (landing page, docs introductory sections, meta tags, navigation labels, OG/Twitter card descriptions) SHALL frame the project as a "dynamic causality" framework. The phrase "hypergeometric computational causality" SHALL NOT appear outside of the `/docs/monograph/` section and a dedicated glossary entry that explicitly addresses the former framing.

#### Scenario: Forbidden phrase absent from marketing pages
- **WHEN** any page outside `/docs/monograph/` and `/docs/concepts/glossary` is rendered
- **THEN** the string "hypergeometric" SHALL NOT appear in the rendered HTML

#### Scenario: Site `<title>` and meta description use new framing
- **WHEN** any page is rendered
- **THEN** its `<title>` and `<meta name="description">` SHALL use "dynamic causality" framing and SHALL NOT contain the string "hypergeometric"

### Requirement: Glossary page codifies terminology
The site SHALL include a glossary page at `/docs/concepts/glossary` that defines the canonical terms: dynamic causality, Causaloid, Context, Effect Ethos, Causal Reasoning, Effect Propagation Process, Causal Monad, HKT (Higher-Kinded Types), and CDL (Causal Discovery Language). The glossary SHALL include a "former framing" entry explaining that earlier material described the project as "hypergeometric computational causality" and clarifying the rebrand.

#### Scenario: Glossary defines core terms
- **WHEN** a visitor opens `/docs/concepts/glossary`
- **THEN** the page SHALL contain definitions for each of: dynamic causality, Causaloid, Context, Effect Ethos, Causal Reasoning, Effect Propagation Process, Causal Monad, HKT, CDL

#### Scenario: Former framing entry present
- **WHEN** a visitor opens `/docs/concepts/glossary`
- **THEN** the page SHALL include an entry acknowledging the prior "hypergeometric computational causality" framing and pointing to current terminology

### Requirement: Brand assets in conventional locations
Brand assets (logo variants, favicons, OG images) SHALL live under `website/web/public/brand/`. The hero art file (originally `ctx/static/img/frontpage-art.webp`) is treated as a content asset and lives under `website/web/public/img/`.

#### Scenario: Logo accessible at conventional path
- **WHEN** a layout references the project logo
- **THEN** the asset SHALL be served from a path under `/brand/`

### Requirement: Build-time check for forbidden phrases
The build SHALL include an automated check that fails if the string "hypergeometric" appears in any rendered HTML outside `/docs/monograph/` and `/docs/concepts/glossary`.

#### Scenario: Stray forbidden phrase fails the build
- **WHEN** a contributor introduces the word "hypergeometric" into a page outside the allowed paths
- **THEN** `pnpm build` (or a post-build verification step) SHALL fail with an error identifying the offending file
