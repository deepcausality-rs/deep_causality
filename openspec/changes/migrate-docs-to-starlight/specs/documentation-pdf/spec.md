## ADDED Requirements

### Requirement: Local single-PDF build script

The documentation app SHALL provide a script that renders the entire documentation into a single PDF, intended to run locally (it relies on a headless browser) rather than in the Cloudflare build environment.

#### Scenario: Script renders the whole docs tree

- **WHEN** a maintainer runs the PDF build script locally
- **THEN** it produces a single PDF containing the documentation pages

#### Scenario: Cloudflare build launches no browser

- **WHEN** the documentation is built on Cloudflare
- **THEN** the build does not launch a headless browser and does not run the PDF script

### Requirement: Committed PDF served as a static asset

The generated PDF SHALL be committed to the repository and served by the documentation site as a static asset.

#### Scenario: PDF is downloadable from the docs site

- **WHEN** a reader requests the documentation PDF URL on the docs origin
- **THEN** the committed PDF is served as a static file

#### Scenario: PDF is part of the repository

- **WHEN** the repository is inspected
- **THEN** the generated PDF is present as a committed artifact alongside the documentation source
