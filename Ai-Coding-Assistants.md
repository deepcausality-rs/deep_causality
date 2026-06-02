[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# AI Coding Assistants

This document provides guidance for AI tools and developers using AI assistance when contributing to DeepCausality.

## Conventions

All contributors are asked to adhere to the project's convention laid out in the following document:
* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)

## Development Process

Regardless of what tools are used, please adhere to the following standard development process during your contribution.

1) Propose a change or patch as a GitHub issue. Feel free to discuss the issue in the community Discord or open a GitHub discussion.
2) Break larger changes into multiple issues as appropriate to structure the work in a way that it can be reviewed.
3) Please ensure that after completing your work, that the entire code base builds, all tests are passing, and neither Clippy nor Rustfmt reporting any issues on the latest stable Rust tool chain. If you added new test, make sure that Bazel test still executes without errors. And in case it does not, ensure that the required test dependencies are declared in Bazel.
4) When you fill up ER, please make a concise summary of the change set and a link to the related issue. Also, please mention any preceding discussion. 
5) Test coverage is expected to stay at the preceding level. If not, please close gaps in the test coverage. The code review can only complete when all PR checks, including test coverage, are green.

## Licensing and Legal Requirements

All contributions must comply with the DeepCausality licensing requirements:

* All code must be compatible with MIT Licence
* Use appropriate SPDX license identifiers in all files: SPDX-License-Identifier: MIT

## Signed-off-by and Developer Certificate of Origin

AI agents MUST NOT add Signed-off-by tags. Only humans can legally
certify the Developer Certificate of Origin (DCO). The human submitter
is responsible for:

* Reviewing all AI-generated code
* Ensuring compliance with licensing requirements
* Adding their own Signed-off-by tag to certify the DCO
* Taking full responsibility for the contribution

## Ai Attribution

When AI tools contribute to the DeepCausality development, proper attribution
helps track the evolving role of AI in the development process.

Contributions should include an Assisted-by tag in the following format:

Assisted-by: AGENT_NAME:MODEL_VERSION 

Where:

* ``AGENT_NAME`` is the name of the AI tool or framework
* ``MODEL_VERSION`` is the specific model version used

Example::

Assisted-by: Claude:claude-4.8