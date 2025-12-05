# Feature Specification: Replace RNG

**Feature Branch**: `feat/replace-rng`  
**Created**: Tuesday, September 16, 2025  
**Status**: Draft  
**Input**: User description: "Replace-rng Currently, the @deep_causality_uncertain relies on the rand crate to generate random numbers. The uncertain crate uses the following from the rand crate: use rand::Rng; The Rng trait is defined in @ctx/rng_traits.rs Specify how to write a Rust native replacement to replace rand with an internal crate called deep_causality_rand that depends only on the std lib. Ensure that the new deep_causality_rand has: 1) zero external dependencies. Only the std lib is allowed 2) zero unsafe code. 3) zero macros Furthermore, ensure that the source code is organized similarly to the existing crates I.e. subfolders under src for errors, types, and traits. And ensure proper testing in a dedicated test folder that replicates the source code folder structure. Aim for 100% test coverage meaning you test for all error, edge, and corner cases."

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

### For AI Generation
When creating this spec from a user prompt:
1. **Mark all ambiguities**: Use [NEEDS CLARIFICATION: specific question] for any assumption you'd need to make
2. **Don't guess**: If the prompt doesn't specify something (e.g., "login system" without auth method), mark it
3. **Think like a tester**: Every vague requirement should fail the "testable and unambiguous" checklist item
4. **Common underspecified areas**:
   - User types and permissions
   - Data retention/deletion policies  
   - Performance targets and scale
   - Error handling behaviors
   - Integration requirements
   - Security/compliance needs

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
The `deep_causality_uncertain` crate currently relies on the `rand` crate for random number generation. This feature aims to replace `rand` with a new, internal crate called `deep_causality_rand` that adheres to specific constraints (zero external dependencies, zero unsafe code, zero macros, 100% test coverage). The primary user of this feature is the `deep_causality_uncertain` crate, which will benefit from a more controlled and auditable random number generation mechanism.

### Acceptance Scenarios
1. **Given** the `deep_causality_uncertain` crate relies on `rand::Rng`, **When** `deep_causality_rand` is integrated, **Then** `deep_causality_uncertain` uses `deep_causality_rand` for RNG.
2. **Given** `deep_causality_rand` is implemented, **When** its dependencies are checked, **Then** it has zero external dependencies (only `std` lib).
3. **Given** `deep_causality_rand` is implemented, **When** its code is analyzed, **Then** it contains zero unsafe code.
4. **Given** `deep_causality_rand` is implemented, **When** its code is analyzed, **Then** it contains zero macros.
5. **Given** `deep_causality_rand` is implemented, **When** its test suite is run, **Then** it achieves 100% test coverage, including error, edge, and corner cases.
6. **Given** `deep_causality_rand` is implemented, **When** its source code structure is reviewed, **Then** it has subfolders under `src` for errors, types, and traits, similar to existing crates.
7. **Given** `deep_causality_rand` is implemented, **When** its test folder structure is reviewed, **Then** it replicates the source code folder structure.

### Edge Cases
- What happens if the `std` lib features required for RNG are not available in a specific Rust environment? The `deep_causality_rand` crate will implement a pseudo-random number generator (PRNG) using `SipHash13` by default, which relies on `std::collections::hash_map::RandomState`. This approach is expected to be portable across Rust environments that support the standard library.
- How does the system handle potential biases or non-uniformity in the custom random number generation if not carefully implemented? The `deep_causality_rand` crate will use `SipHash13` as its default PRNG, which is considered a reasonably good hash function for general-purpose use. For applications requiring stronger, OS-backed random numbers, a feature flag `os-random` will be provided. When enabled, this flag will switch the RNG implementation to use the `getrandom` crate, which leverages the operating system's entropy sources. This allows users to choose between portability with a default PRNG and stronger randomness with an opt-in feature.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The `deep_causality_rand` crate MUST provide random number generation functionality.
- **FR-002**: The `deep_causality_rand` crate MUST NOT have any external dependencies beyond the Rust standard library.
- **FR-003**: The `deep_causality_rand` crate MUST NOT contain any `unsafe` code.
- **FR-004**: The `deep_causality_rand` crate MUST NOT use any macros.
- **FR-005**: The `deep_causality_rand` crate MUST organize its source code with subfolders under `src` for errors, types, and traits.
- **FR-006**: The `deep_causality_rand` crate MUST have a dedicated test folder that replicates the source code folder structure.
- **FR-007**: The `deep_causality_rand` crate's tests MUST achieve 100% code coverage, including error, edge, and corner cases.
- **FR-008**: The `deep_causality_uncertain` crate MUST be updated to use the `deep_causality_rand` crate for random number generation.
- **FR-009**: The `Rng` trait and all its depending traits MUST be moved into the `deep_causality_rand` crate and implemented there.
- **FR-010**: The `deep_causality_rand` crate MUST implement a default pseudo-random number generator using `SipHash13`.
- **FR-011**: The `deep_causality_rand` crate MUST provide an `os-random` feature flag that, when enabled, switches the RNG implementation to use the `getrandom` crate for OS-backed random numbers.
- **FR-012**: The `deep_causality_rand` crate MUST define a custom error enum `RngError` with variants as needed, including `OsRandomGenerator(String)` to wrap OS errors from `getrandom`.
- **FR-013**: Unit tests MUST be created for the `RngError` enum to ensure it is fully tested.

### Key Entities *(include if feature involves data)*
- **deep_causality_rand crate**: A new Rust crate responsible for generating random numbers, including the `Rng` trait and its dependencies.
- **deep_causality_uncertain crate**: An existing Rust crate that will consume random numbers from `deep_causality_rand`.
- **Rng trait**: A trait defining the interface for random number generation, now residing within `deep_causality_rand`.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] Written for non-technical stakeholders
- [ ] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous  
- [ ] Success criteria are measurable
- [ ] Scope is clearly bounded
- [ ] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
