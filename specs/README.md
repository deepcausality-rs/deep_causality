# Agentic Software Development Lifecycle

Last updated: Dec/5, 2025

The DeepCausality project adopts an agentic (Ai) augmented Software Development Lifecycle (SDLC) based on prior
experience with spec-kit. In general, the process follows a shift left paradigm that emphasizes more the
spec development revision and improvement before any code will be written, with the understanding that a lot of
implementation problems can be prevented with proper specification and requirement design.

## SDLC Process:

1) Draft specs
2) Review and verify specs
3) Implement specs
4) Testing & QA
5) PR & Merge

Spec document progression

All spec documents are written in standard Markdown.

A) Place draft into folder /sepcs/current
B) Decide feasibility.

- If postponed, move to /sepcs/deferred
- If infeasible, document the reasons, and move to /specs/infeasible
- If feasible, leave it under /sepcs/current until full implementation
  C) When completed, move the spec document to /sepcs/implementated

### Draft specs

In the first stage of the SDLC, please draft the initial specification using the AI agent of your choice. Specifically
follow the three-step approach that has been proven effective over time:

1. Pre-load context. If your feature request is related to an existing crate, let the AI agent read that crate upfront
   to have all the preliminary context required to design the requirement document. If your feature request requires
   certain background knowledge (e.g. a mathematical formula, a physics equation), add a PDF file that the agent can
   read to inform its planning. For instance, if you want to implement a new algorithm based on a publication, give the
   agent all relevant crates it would need to implement the new algorithm and the publication to read through to get the
   details required for the new algorithm. Context pre-loading increases chances of success enormously. The vast
   majority of implemented spec files that were successful all pre-loaded the agent context with all relevant
   information, so that the specification was correct, complete, comprehensive, and in accordance with the existing
   code.

Note: some agents have a hard time sticking to the conventions formulated in the AGENTS.md file. If you observe your
agent deviating from the AGENTS.md file, just add to the prompt to read the AGENTS.md file and stick to the conventions
formulated in it. It does reduce code review and refactoring time after the implementation.

2. State the problem as independent of any particular solution or architecture as possible unless it needs to be
   specifically integrated into an existing architecture. If you want to follow a certain architecture pattern, give the
   AI agent a specific example where that architecture pattern has been used. If you have a similar implementation
   already somewhere in the repository, add the code as an example to the agent. That can be unit tests or real example
   code just to ensure that the agent has the relevant context to draft the spec document as complete as possible.

3. Ask the agent to draft a usage example after the feature has been fully implemented so that you can easily review
   whether the anticipated outcome is what you want it to be. Also include into the agent prompt to add a section about
   comprehensive testing with your quality parameters, for instance, to aim for full test coverage with all code
   branches and exceptions/error cases being fully tested. If your code is performance-sensitive, then add to the prompt
   that the agent adds a section to the spec document with comprehensive details of benchmarking, performance analysis,
   and how to evaluate the benchmark results.

### Review and verify specs

During the spec review, you have to pay attention to integration with the existing code, egonomics of the sample code,
and overall adherence to good design and engineering practices.

Note: it is not uncommon to see at least three to five iterations on a spec document until the proposed API feels right.
Specifically, you could definitely should add performance, security, and other requirements that often get overlooked in
an early design stage. Specifically, if you ask the agent to revise the spec document for, say, stricter performance
requirements, it quite often comes up with remarkable optimizations that are then implemented in the first iteration
instead of requiring significant refactoring later on and that saves a meaningful amount of time.

For complex features, it is sensible to spread out the spec development over a number of days and aggregate external
feedbacks from other engineers or people involved in the project. In general, the revision and improvement process of
the spec document is where the vast majority of effort and brainpower will be applied predominantly to ensure that the
agent has a most complete, comprehensive, and effective blueprint to implement.

For complex features, prompt the agent to conduct risk analysis over the entire spec document, identify all relevant
risks, and write a comprehensive risk mitigation strategy for each identified risk. During the review, it is important
go re-rank the risks based on actual priority relative to the project scope. Furthermore, the risk mitigation strategy
needs particularly careful review and might receive some revisions to ensure that non-obvious risks are carefully
handled by the agent during the implementation stage. There is a very high correlation between implementation success
and risk mitigation in the specification document.

It is also a good best practice for complex features to ask the agent to add a list of all
affected files and how they're affected, for instance newly added, edited, updated or deleted and combine that with the
actual path of the real file. That way, agents tend to struggle less with finding files they are supposed to edit,
create, or delete.

By experience, most of the coding agents do a significant better job with a detailed, specific, comprehensive and
de-risked spec document.

### Implement specs

For the implementation, it is often helpful to close the agent and restart a new session to clear out the previous
context. It is by practical observations that most agents struggle increasingly as the context pollutes from previous
conversations. The context clearance prior to implementing a specification does help to start with a clean context.

For small to medium-complex spec documents, usually the agent can implement them in one or two shots in Autonomous Mode.
There's a one-to-one correlation between better specs and better/faster implementation by the agent.

For large or very complex specification documents, it is usually preferred to spread out the implementation over
multiple stages. When you see the specs that have been implemented and moved to the implemented folder, there are a
handful of larger refactoring specification documents that were spread out up to five stages where each stage was
conditioned on the completion of the previous one. That is actually necessary to ensure that if the agent derails at any
point, you can resume from the previous step.

### Testing & QA

The most common follow-up after fully autonomous agent implementations usually boils down to three things:

1. crate imports. For some reason, even the most advanced agents do struggle with understanding the import convention
   spelled out in the agents.md file. Expect to fix incorrect, missing, or malformatted imports.
2. Quite often, the source code organization is not quite in line with the repo convention.
3. Logical and especially complex formulas do need to be verified in dedicated non-unit tests. Specifically, that means
   you have to design corner cases and testing properties of the formula or equation. And quite often that does expose
   non-trivial implementation bugs. So be advised to allocate additional time and effort to property test or otherwise
   verify complex formulas.

When the previous spec document was drafted comprehensively with clear testing instructions, in general, the agent hits
an 80-90% test coverage after the first iteration. And again, the quality and comprehensiveness of the spec document
correlates one-to-one with the anticipated test coverage after the implementation.

Before filling a PR, ensure the following checks pass without any warnings:

* make test | alternatively: bazel test //... - runs faster
* make check
* make format
* make fix

When test format or fix reports a lot of lint or test failures, just let the agent run the command and task it to fix
whatever Clippy reports or whatever tests are failing.

### PR & Merge

When drafting the PR, use the agent to query the current Git status and draft a PR message. In general, the agent does a
reasonable job to articulate a fairly understandable PR message. However, do expect to edit the message in terms of
cutting the fluff and reducing the amount of details. As most agents tend to go a little bit too much into the
nitty-gritty details.

The CI pipeline runs the same series of commands as listed before, plus some additional security scans. So if your
previous run of tests checked format and fix reports zero issues, do expect your PR to get green relatively soon.
However, the only meaningful area that requires attention is the test coverage, which in this project is relatively
high, with an average at or above 95%, and the meanwhile useful code improvements suggested by the GitHub AI PR agent.

In general, expect to allocate some time to hit the test coverage quota at the CI, and allocate some time to at least
review as the code improvement suggestions, because some of them are actually really helpful to catch things like
idiomatic Rust code or even not so obvious issues i.e. unnecessary range checks or logic issues.

## Closing thoughts

The DeepCausality adopted the agent augmented SDLC as a response to the known difficulty to acquire and keep OSS
maintainers for a complex code base. The overall experience with the process is overwhelmingly positive and in many ways
has become a driving force for the project to advance further. Specifically, the implementation of complex physics
formulas has been proven to be particularly effective when done and reviewed by agents. 