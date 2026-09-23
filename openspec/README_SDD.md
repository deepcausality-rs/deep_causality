# SDD: Spec Driven Development

DeepCausality develops with AI agents from specifications, using the OpenSpec framework and a handful of
conventions. Web design uses the taste skills. Install both before you start.

## Installation
 
* OpenSpec: https://github.com/Fission-AI/openspec
* Taste Skills: https://www.tasteskill.dev

## Convention

### Build context first

The first task is always to build context.
Before doing anything, the agent must read:

@AGENTS.md
@docs/writing_guides/AiStyleguide.md
@AffectedCrate
@RelevantDocumentation

AGENTS.md documents all crate dependencies and repo conventions, so the agent finds sub-dependencies
on its own and applies the coding conventions. Load AGENTS.md at the start of every AI coding session.

The AiStyleguide tells the agent how to write readable prose.

The affected crate is the primary crate you work on.

Relevant documentation means every document or publication that helps the agent understand the problem.

Use only publicly available or open-access papers, e.g. from arxiv.org, and add them to the crate's
papers folder for future reference. Never commit paywalled or otherwise non-public papers; doing so violates copyright.

### Begin with a note

Start with a note that states what you want to do, the relevant context, and the constraints.
If unsure where to start, use the OpenSpec explore skill:

```shell
/opsx:explore
```

Add and commit the note to 'openspec/notes', which holds all current notes.

Iterate over the note. Ask the agent for:
* Hidden assumptions
* Make or break requirements
* Important gaps

Once you and the agent agree the note is reasonably complete, derive the full specification. Do not delete notes used to derive a specification. Track all notes, specs and relevant information in Git within the openspec folder.

### Derive the full specification from the note

Run the propose skill with a reference to the note, and OpenSpec derives the specification.
Give the specification a meaningful name, for example add-verification-to-haft-crate.

```shell
/opsx:propose add-verification-to-haft-crate
```

This generates a design document, a proposal, a task list, and a number of specification files.

### Review the specification

Begin with the proposal, which lays out the overall idea and process, then read the design document. If you find unintended decisions, ask the agent to correct them; the correction usually updates the affected specs and task-list items too.

Expect 2 to 3 iterations over the specification to settle the details implementation needs.

### Commit the final specification

Commit the final specification before implementing it, for provenance.

### Defer the implemention when deemeded temporary infeasible 

When a specification cannot be implemented in full, move the derived specs into:

openspec/changes/deferred

and document why the implementation is deferred, e.g. a missing feature in the Rust compiler. Document
the exact requirement that would unblock the deferred specification.


### Implement the specification

Ask the agent to implement the specification with the apply skill.

```shell
/opsx:apply add-verification-to-haft-crate
```

Claude ultracode or Codex Xhigh are the recommended modes for implementation. They spawn subagents that implement independent specs in parallel, and adversarial subagents that verify the implementation, which usually speeds acceptance in CI.

### Review the implementation

Frontier models implement specifications well, so focus the review on the most complex code, such as algorithms or multi-layered integration.

Once the review confirms the spec is implemented, archive it:

```shell
/opsx:archive add-verification-to-haft-crate
```

This moves the change folder into the archive. Move the corresponding note yourself
into 'openspec/notes/archive', using your IDE's refactoring so that every reference to the note follows it.

### Prepare for PR

Before filing a PR, make sure these checks pass:

* make test
* make check
* make format && make fix

Then file a PR and tag team members for review.

CI runs a large test suite and an AI code review, so expect several rounds of fixes before CI
turns green.

While waiting for the assigned reviewer, run the PR review prompt from 'openspec/prompts' yourself.
It may catch things the CI review bot misses.

### Reverting an implementation 

To revert a fully implemented specification, draft a specification for the removal, complete the refactoring, and move both the reverted specs AND the removal specs into:

openspec/changes/reverted

Document why the revert was necessary and what would trigger a re-evaluation. If the approach is a fundamental dead end, document why.

### Apply improvements

When you learn a lesson worth keeping, update this document and related process documents so later implementations benefit. 
