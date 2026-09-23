# Specs

Last updated: Dec/5, 2025

## Deprecation Note

The DeepCausality project phased out spec-kit and replaced it with a simpler, leaner and faster process based on Gemini / AntiGravity. 

## OLD README

The DeepCausality project adopted spec-driven development with [spec-kit](https://github.com/github/spec-kit?tab=readme-ov-file#-detailed-process).

AGENTS.md documents the project conventions for the AI agent, so your coding agent uses the right build and test tools by default and understands the repository structure. Alternatively, pre-load the agent's context by typing `read @GEMINI.md`.

Next, install spec-kit. See the [spec-kit](https://github.com/github/spec-kit?tab=readme-ov-file#-detailed-process) repository for details.

If your spec-kit installation is over a week old, update it via:

uvx --from git+https://github.com/github/spec-kit.git specify init .

The trailing dot means "here", assuming you run the command from the project root.

Before you start, ensure

A) You have a sensible AGENTS.md file in place
B) You have a project specific constitution in place.

For A, visit https://agents.md, look at the examples, and build one.
This repo has a sample AGENTS.md in the root folder and a
legacy Gemini.md explaining how to configure Gemini CLI to use
AGENTS.md by default.

For B, start your coding agent and run:

/constitution

Once spec-kit is installed, the basic workflow is:

0) Start your coding CLI agent (e.g., Gemini-CLI, Claude Code, Copilot, or Cursor).
1) Pre-load the agent's context with all relevant crates (e.g., type `read @deep_causality`).
2) Type `/specify "your feature story"`. This creates a new branch and a spec file under `/specs`.
3) Type `/clarify` to improve the `spec.md` file until it is complete.
4) Type `/plan` to derive a plan from the spec document. Let your agent validate the plan.
5) Type `/task` to derive a detailed task plan. Double-check, edit, and adjust.
6) Type `/implement` to start the implementation.
7) Interact with the agent to supervise the implementation.
8) Verify the implementation, test, and conduct a code review.
9) Submit a PR and check CI status.

If you are unsure about a feature or implementation technique, ask the agent to research it. Without a good starting source (e.g., a blueprint, a technical blog post, or sample code), results vary.

Plan validation raises the chance of a quick implementation without the agent running in loops. A sample validation prompt from the spec-kit example:

    Now I want you to go and audit the implementation plan and the implementation detail files.
    Read through it with an eye on determining whether or not there is a sequence of tasks that you need
    to be doing that are obvious from reading this. Because I don't know if there's enough here. For example,
    when I look at the core implementation, it would be useful to reference the appropriate places in the implementation
    details where it can find the information as it walks through each step in the core implementation or in the refinement.

For complex or large features, ask the agent to assess risks, derive a mitigation for each, and update the plan. A sample prompt:

    Please do a comprehensive risk assessment of the implementation plan and the implementation details, identify
    all applicable risks, derive an effective mitigation for each identified risk,
    and then update the plan accordingly.

These practices have proven effective:

*   Define traits upfront whenever possible.
*   Define Error types and Enums upfront for the most common error cases.
*   If possible, define key structs.
*   For complex algorithms, let the agent read a publication or reference document to pre-fill the context. It helps
    most when the publication is detailed.
*   Referencing existing code in the repository via `@path/to/file.rs` gives meaningful context to inform the plan.
*   Adding a sample API and/or mock API usage usually results in an exact replication of the sample API with proper
    implementation.

With these practices, the agent commonly writes 90% to 95% of the code, in a style and standard close to the rest of the project.

Steps 1-7 usually run smoothly, even for complex implementations, especially when the specs and plan are specific and detailed.

Steps 8 and 9, especially code coverage, need follow-up: most agents, even when told to test all methods and branches, miss some on the first run, and most fill the gaps in the follow-up. Since the project sustains code coverage of about 95% to 97%, QA and testing take most of the time for any feature.

Resources:
* https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit
* https://www.seangoedecke.com/ai-agents-and-code-review
* https://youtu.be/xgnIBh25a5A?si=RmvKa7Eq0x5fqDkr
