# Specs

The DeepCausality projected adopted spec driven development
with [spec-kit](https://github.com/github/spec-kit?tab=readme-ov-file#-detailed-process).

Important, the project conventions for the Ai agent are written down in the [Gemini.md](../GEMINI.md) file. For agents other then Gemini, you have to make a copy and rename the file to something your agent reads by default i.e. AGENT.md or similar. That way, your coding agent will use the right build and test tools by default and knows the structure of the repo. Alternatively, you can pre-load the agents context by simply typing `read @GEMINI.md`

Next it is important that you have spec-kit installed on your machine. See the [spec-kit](https://github.com/github/spec-kit?tab=readme-ov-file#-detailed-process) repository for details.

Once spec-kit is installed, the basic workflow is as following.

0) Start your coding cli agent i.e. Gemini-CLI, Claude Code, copilot, or cursor.
1) Pre-load the agent's context with all relevant crates i.e. type `read @dee_causality`
2) type `/specify `"your feature story". This crates a new branch and a spec file under /specs
3) Interact with the agent to edit the new spec.md file until it is complete, and correct
4) Type `/plan `to derive a plan from the spec document. Let your agent validate the plan.
5) Type `/task `to derive a detailed task plan. Double check, edit and adjust.
6) Type `implement specs/00x-my-feature/plan.md`
7) Interact with the agent to supervise the implementation.
8) Verify and the implementation, test, and do code review
9) Fill a PR and check CI status

If you are unsure about a feature or implementation technique, you can ask the agent to do research for you. However,
without a good starting source i.e. a blueprint, a technical blog post, or sample code, your millage and luck may vary.

Plan validation significantly increases chances of a speedy implementation without the agent running in random loops.
A sample prompt to ask the agent to validate the plan, form the spec-kit example:

    Now I want you to go and audit the implementation plan and the implementation detail files.
    Read through it with an eye on determining whether or not there is a sequence of tasks that you need
    to be doing that are obvious from reading this. Because I don't know if there's enough here. For example,
    when I look at the core implementation, it would be useful to reference the appropriate places in the implementation
    details where it can find the information as it walks through each step in the core implementation or in the refinement.

For complex, large, or advanced features, it is recommended to ask the agent to do a risk assessment, derive a
mitigation for each identified risk, and update the plan accordingly. A sample prompt would be:

    Plz do a comprehensive risk assment of the mplementation plan and the implementation details, identify 
    all applicable risks,derive an effective mitigation for each identified risk, 
    and then update the plan accordintly. 

A handful of best practiced have been proven as effective:

* Define traits upfront whenever possible
* Define Error types and Enums upfront for the most common error cased
* If possible, define key structs.
* For complex algorithms, let the agent read a publication or reference document to pre-fill the context. It does make a
  meaningful difference especially if the publication is detailed.
* Referencing existing code in the repo via @path/to/file.rs gives meaningful context to inform the plan.
* Adding a sample API and / or mock API usage usually results in an exact replication of the sample API with proper
  implementation.

When these best practices are applied, it is very common that the agent writes up 90% to 95% of the code while
maintaining a code style and standard that is similar to the overall code quality of the project.

By experience, steps 1-7 usually run fairly straightforward even for complex implementations especially when the specs
and plan document are very specific and detailed. As a guiding principle, the following best practices usually ensure a
fast and efficient implementation by the agent:

Step 8 and 9, especially code coverage, usually requires some follow up interventions because most agents, even if told
specifically to test all methods and code branches, often skip many parts of the code under test. Nevertheless, most
coding agents do correct missing code coverage during the follow up. Since the DeepCausality project maintains a
sustained code coverage rate of about 95% to 97%, QA and testing requirers the majority of time for any given feature.