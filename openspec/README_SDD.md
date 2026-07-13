# SDD: Spec Driven Development

The DeepCausality project makes extensive use of agentic spec-driven development by leaning on the OpenSpec framework
and a handful of conventions. For web design, the taste skills are used. Ensure you have these installed.

## Installation

OpenSpec: https://github.com/Fission-AI/openspec
Taste Skills: https://www.tasteskill.dev

## Convention

### Build context first

The first task is always to build context.
Mandatory context files the agent must read before doing anything are:

@AGENTS.md
@AffectedCrate
@RelevantDocumentation

Affected crate refers to the primary crate you want to work on. The dependency and repo conventions are documented
in AGENTS.md so the agent will find sub-dependencies on its own and use all applicable coding conventions.

Relevant documentation refers to all documents or publications that help the agent to understand the problem.

Note, for scientific publications, please only use publicly available or open access papers e.g. from arxiv.org, and add these
to the paper folder of the crate for future references. However, please stay away from ever committing paywalled or otherwise non-public or inaccessible papers as this would amount to a copyright violation.

### Begin with a note

Start with a note that lays out what you want to do, what the relevant context is, and which constraints apply.
If unsure where to start, use the explore skills from OpenSpec:

```shell
/opsx:explore
```

Add and commit the note to folder 'openspec/notes' where all current notes reside.

Iterate over the note. Inquire the AI agent for:
* Hidden assumptions
* Make or break requirements
* Important gaps

Once you and the agent agree that the note is reasonably complete, proceed to derive the full specification.

### Derive the full specification from the note

Use the propose skill and refer to the note to derive the specification from the note and let OpenSpec do the work.
Ensure to give the specification a meaningful name, for example, add-verification-to-haft-crate.

```shell
/opsx:propose add-verification-to-haft-crate
```

This generates a design document, a proposal, a task list, and a number of specification files.

### Review the specification

Begin the review with the proposal document because it lays out the overall idea and process and then pay attention to the design document. If you find unintended decisions, ask the agent to correct, which usually also updates the affected specs and items on the task list.

It is common to iterate 2 to 3 times over the specification to hammer out the details required for smooth implementation.

### Commit the final specification

For provenance reasons, it is important to commit the final specification before implementing it.

### Implement the specification

Next, ask the agent to implement the specification using the apply skill.

```shell
/opsx:apply add-verification-to-haft-crate
```

In general, Claude ultracode or Codex Xhigh are recommended modalities for the implementation because this usually spawns multiple subagents to implement independent specs in parallel, which accelerates the completion. Furthermore, these modes use adversarial sub-agents that verify the correct implementation, which generally results in faster acceptance during CI.

### Review the implementation

Most frontier models are very good at implementing according to the specification. Therefore, it is advised to focus the review on the most complex part of the code, such as algorithm or multi-layered integration.

After your review concluded that the spec has been implemented, please archive it:

```shell
/opsx:archive add-verification-to-haft-crate
```

This moves the entire folder into the archive. Note, you have to move the corresponding note yourself
into the note archive in 'openspec/notes/archive'. Also, please use the refactoring of your IDE to do the move
to ensure all refernces to the note are updated to the new location.

### Prepare for PR

Before filing a PR, please ensure the following checks are green:

make test
make check
make format && make fix

Then file a PR and tag some of the team for review.
Notice, CI runs a large number of tests and also conducts an AI code review, so it's normal
that multiple code fixes need to be applied before CI turns green.

You can always use the PR review prompt from the prompt folder to prepare a PR review yourself
while waiting for the assigned reviewer to begin the review process. This may catch things the review bot on CI could miss.
