## ADDED Requirements

### Requirement: Code-example-first hero
The landing page SHALL lead, above the fold, with a hero section followed immediately by six code-example cards. Philosophical or conceptual long-form content SHALL NOT appear above the fold.

#### Scenario: Above-the-fold composition
- **WHEN** a visitor loads the landing page at a 1440×900 viewport
- **THEN** the visible region SHALL contain the hero (logo, tagline, primary CTAs) and at least the first row of code-example cards, and SHALL NOT contain any of the philosophical explainer copy

#### Scenario: Primary CTAs present
- **WHEN** the hero is rendered
- **THEN** it SHALL include a "Read the docs" CTA linking to `/docs/getting-started` and a "View on GitHub" CTA linking to the project's GitHub repository

### Requirement: Six code-example cards spanning distinct engineering domains
The landing page SHALL display exactly six code-example cards, each representing a distinct engineering domain. The initial slate SHALL cover: quant finance / trading, robotics / control, observability / SRE, bioinformatics / signal processing, physics simulation, and policy / compliance.

#### Scenario: Six cards present
- **WHEN** the landing page is rendered
- **THEN** exactly six code-example cards SHALL be visible, each labeled with its domain

#### Scenario: Each card is clickable to a detail page
- **WHEN** a visitor clicks any code-example card
- **THEN** the browser SHALL navigate to a dedicated detail page for that example under `/examples/<slug>`

#### Scenario: Card contents
- **WHEN** a code-example card is rendered
- **THEN** it SHALL display a domain label, a one-line problem statement, a syntax-highlighted Rust snippet of approximately 10–20 lines, and a visible "Open example" affordance

### Requirement: Six dedicated example detail pages
The site SHALL provide one detail page per landing-page code example at `/examples/<slug>`, each containing an expanded code listing, a written walkthrough, run instructions, and links to the relevant crate(s) and monograph chapter(s).

#### Scenario: Detail page renders
- **WHEN** a visitor navigates to `/examples/<slug>` for any of the six examples
- **THEN** the page SHALL render with: full code listing, narrative walkthrough, run instructions, and a "Related crates" and "Further reading" section

### Requirement: Rebranded hero copy
The landing-page hero SHALL use the "dynamic causality" framing. The phrase "hypergeometric computational causality" SHALL NOT appear on the landing page.

#### Scenario: Forbidden phrase absent
- **WHEN** the landing-page HTML is inspected
- **THEN** the string "hypergeometric" SHALL NOT appear anywhere on the page

#### Scenario: Tagline uses new framing
- **WHEN** the hero is rendered
- **THEN** the tagline SHALL describe the framework in terms of "dynamic causality"

### Requirement: Below-the-fold conceptual content
Below the code-example grid, the landing page SHALL provide a concise plain-language "What is dynamic causality?" explainer and a pillars section linking to docs pages for Causaloid, Context, and Effect Ethos.

#### Scenario: Pillars link to docs
- **WHEN** a visitor clicks any of the three pillar cards
- **THEN** the browser SHALL navigate to the corresponding `/docs/concepts/<pillar>` page
