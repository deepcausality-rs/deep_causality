## ADDED Requirements

### Requirement: Published documentation naming code is machine-verified against it

Documentation published outside the source tree SHALL have its references to code verified
mechanically, and a reference that no longer resolves SHALL fail the build.

The parity requirements this capability already carries govern docstrings and comments, which sit
beside the code they describe and move with it. Published documentation does not: it names symbols
and source locations from another tree, and nothing about editing the code disturbs it. Review does
not catch the resulting drift — unifying the CFD configuration builders shifted one file by five
lines and silently invalidated four line citations on the website, while a fifth had already drifted
by twenty-five lines unnoticed.

Two classes of reference SHALL be verified:

- a **symbol** named as code in published documentation SHALL exist in the public API of the crate it
  is attributed to;
- a **source location** cited as a path and line range SHALL resolve to a file that exists, and where
  the citing passage quotes code, the quoted code SHALL be found within the cited range.

The check SHALL run in CI over the published documentation sources and SHALL fail rather than warn,
because a warning on a documentation job is a warning nobody reads.

#### Scenario: A renamed symbol fails the build

- **WHEN** a public name that published documentation refers to is renamed, retired, or made private
- **THEN** the parity check fails, naming the documentation file and the symbol

#### Scenario: A shifted line citation fails the build

- **WHEN** an edit moves code so that a cited line range no longer contains the quoted code
- **THEN** the parity check fails, naming the citation and the range it should have been

#### Scenario: Prose that is not a code reference does not fail

- **WHEN** published documentation contains inline code that is not a symbol of the crate — a file
  name, a CLI flag, a field-name string literal
- **THEN** the check does not fail on it, so the gate stays enabled rather than being disabled for
  noise
