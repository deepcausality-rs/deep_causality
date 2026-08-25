# Knowledge Propagation Process

## From Agentic Sympathy to Evolving Knowledge Systems

*Conversation between Marvin Hansen and Claude, April 12, 2026.*
*Context: Building a full macOS enterprise application (Airgap Voice Enterprise) in a single session
using AI-assisted development with Claude Code, OpenSpec, and a structured AGENTS.md project file.*

---

## 1. Agentic Sympathy

In 2016, Martin Thompson popularized **Mechanical Sympathy** — the principle that software performs
best when designed to align with the executing hardware. You don't need to be a mechanical engineer to
drive a race car, but you drive faster when you understand what the engine is doing.

In 2026, the same principle applies to AI-assisted development: **Agentic Sympathy** is the practice
of structuring project metadata so that AI agents can make correct decisions without exploratory
search. The agent's context window is its working memory. The project's metadata is the prefetch
buffer. Every cache miss — a moment where the agent stops, searches, reads, and backtracks — is a
failure of agentic sympathy.

### Observed Correlation

During the construction of the enterprise application, a direct 1:1 correlation emerged between the
quality of the AGENTS.md file and the success rate per change request:

- **Dependency matrix present** → correct SPM wiring on first attempt
- **L10n namespace table missing** → 3 compile-fix cycles guessing string paths
- **Behavioral guidelines present** → no over-engineering, surgical diffs
- **API contract table missing** → TrialManager parameter rename broke 3 apps

A well-tuned AGENTS.md is like a well-tuned database — queries (change requests) execute efficiently because the indexes
(metadata) match the access patterns (what the agent needs to know).

---

## 2. The Limits of Static Knowledge

AGENTS.md, as effective as it is, represents a **snapshot in time**. It describes what exists now. It
does not capture:

- **Why** the architecture looks the way it does
- **What alternatives** were considered and rejected
- **Which constraints** forced specific decisions
- **How** the project evolved from one state to another

Git tracks state transitions (diff A → B) but excludes the reasoning that produced the transition.
Commit messages are free text, not structured reasoning. OpenSpec archives preserve planning artifacts
but are not attached to the code nodes they produced and are not queryable at the point of need.

### The TrialManager Example

A single parameter rename (`keychainKey:` → `storageKeyPrefix:`) broke iOS and Translate apps. The
information needed to prevent this — "TrialManager is consumed by 4 apps" — existed in the dependency
matrix. But the matrix was added after the breakage occurred. If a structured representation had
linked the `TrialManager` API node to all its consumer nodes, the rename would have automatically
flagged all 4 call sites before the change was declared complete.

---

## 3. From Snapshot to Process

### The Representation Inversion

The current paradigm: **code is the artifact, decisions are implicit in the diff.**

The inversion: **decisions are the artifact, code is a materialized consequence.**

This is not a tooling problem. It is a conceptual shift in what a "project" is:

- **Current model**: A project is a directory of source files at a point in time
- **Inverted model**: A project is an evolving decision graph where source code, documentation, and
  history are all views — materialized projections of the underlying causal structure, generated on
  demand for the consumer that needs them

### What the Inversion Enables

**1. Decisions become first-class objects.**
A refactoring isn't "moved file X to package Y." It's a decision node with inputs (constraints),
outputs (chosen approach), and rejected branches. Future agents navigate by *why*, not by *what*.

**2. Alternatives become queryable.**
Rejected approaches are stored with their rejection reasons. The tenth refactoring is informed by the
first nine — not because the same human remembers, but because the decision graph retains the
reasoning.

**3. Temporal causality becomes navigable.**
Not "flow_model was deleted in commit N+3" but "enterprise requirement → exposed Parakeet coupling →
flow_lib_voice extraction → flow_model had no consumers → deletion." The causal chain is the
navigable structure. Time is metadata on causal edges, not the primary axis.

**4. Counterfactual reasoning becomes possible.**
"If we had chosen alternative B instead of alternative C, what downstream decisions would have been
different?" The answer is derivable because causal links are explicit.

**5. Stale documentation is structurally eliminated.**
Documentation is a view over the graph, not a separate artifact. When a node is deleted, every view
that referenced it updates automatically.

---

## 4. The Meta Graph

The knowledge representation needed is a graph where:

- **Nodes** are decisions, constraints, artifacts (packages, types, files), and intents
- **Edges** are causal relationships: "this exists because of that," "this was rejected because of
  that," "this constraint forced that decision"
- **Temporal edges** preserve the sequence of evolution, but causation — not time — is the primary
  organizing principle

### Queryable Convention Paths

```
/intent/architectural/compute-isolation
  → "Qwen3-ASR on MLX/GPU, Canary on CoreML/ANE. No contention."

/decisions/2026-04-12/flow-lib-voice-extraction
  → constraint: "Enterprise must not depend on flow_model_parakeet"
  → alternatives_rejected:
      - "Refactor flow_model" → risk to retail
      - "Duplicate FlowEngine" → maintenance burden
  → chosen: "Extract model-agnostic pipeline into flow_lib_voice"
  → consequences: [flow_lib_ent created, flow_model merged into flow_lib, flow_model deleted]

/contracts/flow_purchase/TrialManager
  → signature: init(storageKeyPrefix:trialDurationDays:)
  → consumers: [flow_app_mac, flow_app_ios, flow_app_mac_enterprise, flow_app_translate_mac]
```

Querying becomes part of requirement engineering. The agent's first act on any task is to query the
graph for constraints, prior decisions, and rejected alternatives in the affected area. The
requirement doesn't emerge from scratch — it emerges from the intersection of new intent and existing
decision history.

---

## 5. Beyond Systems Thinking

Systems thinking makes feedback loops explicit: A influences B influences C influences A. This is
powerful for understanding system behavior at a given moment. But the loop itself is treated as a
structural fact — discovered, diagrammed, reasoned about.

The meta graph goes further. During the enterprise app session, the feedback loop at 9am was "new app
requirement → discover coupling → extract shared component → enable new app." By 3pm, the act of
traversing that loop had modified the loop itself — the extraction eliminated `flow_model`, which
changed the dependency structure, which meant future requirements face a different coupling landscape.

This is second-order dynamics:

- **First order**: The system changes state (code changes)
- **Second order**: The structure of the system changes (the dependency graph reorganizes)
- **Third order**: The rules by which the structure changes emerge from accumulated decisions (the
  pattern "extract shared component for new product variant" becomes a recognized strategy)

Systems thinking captures the first order. Architecture diagrams capture the second order. Nothing
captures the third order today. We don't have vocabulary for "the pattern by which this project's
architecture evolves in response to new requirements." It's not a methodology. It's emergent from the
accumulated decision history. It's specific to each project. And it's the most valuable knowledge for
predicting what the next change should look like.

---

## 6. Knowledge Propagation Process

The meta graph is not a knowledge base (static store) or a knowledge graph (queryable structure). It
is a **knowledge propagation process** — a running computation whose state is the accumulated
decisions, whose transitions are new decisions informed by history, and whose output at any moment is
a projection (code, documentation, architecture) that agents and humans can act on.

The process of becoming is the record. The artifact is the outcome. Knowledge propagates through time
not as snapshots that get stale, but as a continuous dynamic where each decision modifies the graph,
and the modified graph informs the next decision.

### What Exists Today (Approximations)

| Tool | What it captures | What it misses |
|---|---|---|
| Git | State transitions (diffs) | Reasoning, alternatives, constraints |
| OpenSpec | Planning chain (proposal → design → spec → tasks) | Not attached to code nodes, not queryable |
| AGENTS.md | Current architecture, conventions, dependencies | No temporal evolution, goes stale |
| Conversation context | Full reasoning with alternatives and tradeoffs | Ephemeral, lost between sessions |
| Systems thinking | Feedback loops at time T | Evolution of loops over time |

### What Needs to Exist

A representation where:

1. **Every structural change** to the project creates or modifies decision nodes in the graph
2. **Every decision node** records inputs (constraints, requirements), the chosen approach,
   rejected alternatives with reasons, and downstream consequences
3. **Querying the graph** is the first step of any new task — the query shapes the requirement
4. **A change is complete** only when the graph is synchronized — making staleness structurally
   impossible
5. **The graph co-evolves** with the project, maintained by both humans (intent, constraints) and
   agents (structural facts, API surfaces, dependency edges)
6. **Third-order patterns** — how the project's architecture evolves — emerge from the decision
   history and are themselves queryable

### The Completion Invariant

The most practically impactful property: **a change is not mergeable until the graph is consistent.**
Like "all tests pass" or "no compiler warnings," graph consistency becomes a build gate. The agent
cannot declare a task complete with a stale or inconsistent knowledge state. This eliminates the
entire class of "documentation drift" problems by making knowledge maintenance a structural
requirement, not a discipline problem.

---

## 7. Why This Matters

Software systems that last decades accumulate decisions faster than they accumulate code. The Linux
kernel, PostgreSQL, and similar long-lived systems survive not because their code is perfect but
because institutional knowledge — carried by long-tenured maintainers — informs every change. When
those maintainers leave, the knowledge leaves with them. The code remains but the reasoning is gone.

AI agents face this problem at an extreme: every session is a cold start. There are no long-tenured
maintainers. The reasoning from the previous session is gone unless it was externalized. Current
externalization (documentation, commit messages, specs) is lossy and disconnected.

The knowledge propagation process is the mechanism by which software projects can accumulate and
retain decision intelligence across time, across contributors, and across agent sessions — making
systems that last because the knowledge that built them is preserved, queryable, and alive.

---

*This document summarizes a conversation that emerged from building an enterprise macOS application
in a single session. The application (Airgap Voice Enterprise) combines Qwen3-ASR dictation and
Canary-1B translation, runs entirely offline, and was built from specification to working binary in
approximately 6 hours using Claude Code with OpenSpec-driven development and a structured AGENTS.md
project file.*
