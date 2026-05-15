## ADDED Requirements

### Requirement: Documentation tree structure
The site SHALL provide a documentation section under `/docs/` organized into the following top-level sections: `getting-started`, `concepts`, `guides`, `reference`, and `monograph`. The `concepts` section SHALL include pages for **Causal Monad** (sourced from `deep_causality_core::types::causal_monad`), **HKT / Higher-Kinded Types** (sourced from `deep_causality_haft` and `deep_causality_core::types::causal_effect_propagation_process::hkt`), and **CDL (Causal Discovery Language)** (sourced from `deep_causality_discovery`) in addition to dynamic-causality, Causaloid, Context, Effect Ethos, and Effect Propagation Process. The `guides` section SHALL include a dedicated **CDL pipeline guide** that walks through the typestate-builder workflow end-to-end using the example in `deep_causality_discovery/examples/main.rs`.

#### Scenario: Top-level docs sections exist
- **WHEN** the site is built
- **THEN** routes SHALL exist for `/docs/getting-started/`, `/docs/concepts/`, `/docs/guides/`, `/docs/reference/`, and `/docs/monograph/`, each with at least one published page

### Requirement: Newly authored documentation, not migrated from legacy Hugo content
All content under `/docs/concepts/`, `/docs/guides/`, `/docs/reference/`, and `/docs/getting-started/` SHALL be newly authored using the monograph in `papers/` and the current state of the Rust crates as source material. Legacy Hugo documentation from the external repo SHALL NOT be migrated into these sections.

#### Scenario: Concept pages cite the monograph
- **WHEN** a `/docs/concepts/` page covers a topic also covered in the monograph
- **THEN** the page SHALL link to the corresponding `/docs/monograph/` chapter

### Requirement: Per-crate reference overview pages
The site SHALL provide a one-page-per-crate overview under `/docs/reference/` covering each of the project's published crates, summarizing what the crate does and when a developer should reach for it. Deep API reference remains the responsibility of docs.rs and SHALL NOT be reproduced on this site.

#### Scenario: Reference page exists per crate
- **WHEN** a published crate exists in the monorepo
- **THEN** `/docs/reference/<crate-name>/` SHALL render with at least a summary, primary types/traits, and a link to the crate's docs.rs page

### Requirement: Monograph section
The site SHALL publish the monograph as a section under `/docs/monograph/` covering the six EPP preprint volumes in `papers/src/EPP/`: `Preprint_EPP`, `Preprint_EPP_Metaphysics`, `Preprint_EPP_Formalization`, `Preprint_EPP_Ontology`, `Preprint_EPP_Epistemology`, and `Preprint_EPP_Teleology`. The LaTeX source SHALL remain the canonical source of truth; the site SHALL publish at minimum a downloadable PDF per preprint plus an MDX overview page per preprint suitable for in-site reading and SEO.

#### Scenario: All six monograph volumes published
- **WHEN** a visitor navigates to `/docs/monograph/`
- **THEN** the index SHALL list all six EPP preprint volumes, each with an MDX overview page and a link to the canonical PDF

#### Scenario: PDF downloadable
- **WHEN** a visitor clicks the PDF link on a monograph volume page
- **THEN** the browser SHALL download a PDF rendered from the corresponding `papers/src/EPP/Preprint_EPP*` LaTeX source

### Requirement: Build-time search index (Pagefind)
The site SHALL include a client-side search index generated at build time by Pagefind, covering all docs, examples, monograph overview pages, and blog posts.

#### Scenario: Search returns results
- **WHEN** a user types a query into the site search box
- **THEN** results SHALL be returned client-side from the Pagefind index without contacting any external service

#### Scenario: Index regenerated each build
- **WHEN** `pnpm build` runs
- **THEN** Pagefind SHALL run as part of the build and write its index into the deployed `dist/` output

### Requirement: Getting-started covers install and a first Causaloid
The `/docs/getting-started/` section SHALL include an installation page and at least one runnable end-to-end example that constructs a Causaloid and evaluates it.

#### Scenario: Hello-Causaloid example runnable
- **WHEN** a developer follows the getting-started example verbatim
- **THEN** the example SHALL compile and run against the current crate versions in this monorepo
