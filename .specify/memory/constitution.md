<!--
    Sync Impact Report
    - Version: v1.0.0 (Initial establishment of the project constitution)
    - Added Principles:
        - I. Convention and Structure
        - II. Rigorous Testing
        - III. Performance via Zero-Cost Abstractions
        - IV. Uncompromising Safety
        - V. Clear and Private APIs
    - Added Sections:
        - Development Workflow
        - Code Style and Formatting
    - Templates Requiring Updates:
        - ✅ .specify/templates/plan-template.md
    - Follow-up TODOs:
        - TODO(RATIFICATION_DATE): Set initial adoption date.
-->

# deep_causality Constitution

## Core Principles

### I. Convention and Structure
All contributions MUST adhere to the established conventions, style, and structure of the existing codebase. Each crate MUST follow the established `src/{errors, traits, types}` layout. Each file within these modules MUST contain only a single, corresponding definition (one type per file, one trait per file, etc.).
Rationale: This ensures consistency, maintainability, and predictability across the monorepo, making it easier for contributors to navigate and understand the code.

### II. Rigorous Testing
All new features, bug fixes, or substantial changes MUST be accompanied by corresponding tests. Crate-specific tests MUST be run via `cargo test -p <crate_name>`. Before submitting a contribution with multi-crate impact, the entire project test suite MUST pass via `make test`.
Rationale: Guarantees code quality, prevents regressions, and ensures that individual components and the system as a whole behave as expected.

### III. Performance via Zero-Cost Abstractions
Code MUST prefer static dispatch over dynamic dispatch (e.g., `dyn Trait`). All abstractions should strive to be 'zero-cost,' meaning they have no runtime overhead. A functional style (e.g., `map`, `filter`) is preferred for its clarity and potential for optimization.
Rationale: Aligns with Rust's core philosophy and is critical for a high-performance computational causality library. It ensures that high-level code does not compromise low-level performance.

### IV. Uncompromising Safety
The use of `unsafe` code is strictly prohibited in all library crates (`/src`). External dependencies should be minimized and vetted for security. All code MUST be checked with `make check` before submission.
Rationale: Ensures memory safety, robustness, and security, which are non-negotiable for a foundational library intended for production-grade systems.

### V. Clear and Private APIs
All fields in public-facing types MUST be private. Access MUST be provided through explicit constructors, getters, and setters. Internal modules MUST remain private at the crate root, and all public APIs MUST be explicitly exported from `src/lib.rs`.
Rationale: Enforces strong encapsulation, creating stable and predictable public APIs while allowing internal implementation details to evolve without breaking downstream users.

## Development Workflow

All development follows a crate-centric model. Changes should be confined to a single crate where possible. Build and test commands (`cargo build -p <name>`, `cargo test -p <name>`) MUST be run on the affected crate. For changes spanning multiple crates, `make format && make fix` MUST be run before submission to ensure repository-wide consistency.

## Code Style and Formatting

All code MUST be formatted using `cargo fmt` according to the `rustfmt.toml` configuration. Lints are enforced by `clippy`, and all warnings MUST be resolved by running `make fix` or addressing them manually.

## Governance

This Constitution is the single source of truth for development standards and supersedes all other practices. All contributions and reviews MUST verify compliance with these principles. Any deviation requires a formal amendment to this document. The `AGENTS.md` file provides runtime development guidance for AI agents and MUST remain in sync with these principles.

**Version**: v1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): Set initial adoption date. | **Last Amended**: 2025-09-30
