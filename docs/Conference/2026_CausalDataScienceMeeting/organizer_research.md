# Causal Data Science Meeting 2026: submission format and organizer interests

Working notes behind [proposal.md](proposal.md). Compiled September 2, 2026 from the meeting
website, the organizers' publication records, and the keynote speaker's recent work.

## 1. Submission facts

Taken from the call for papers at [causalscience.org](https://www.causalscience.org/#Call-for-Papers).

| Item | Value |
|---|---|
| Meeting | Two-day virtual workshop, November 4 to 5, 2026 |
| Submission deadline | September 30, 2026 |
| Acceptance notification | October 7, 2026 |
| What to submit | "your presentation proposal, extended abstract or full paper" |
| Length limit | None stated |
| How | Email to submission@causalscience.org |
| Subject line | The site's mail link still uses `Submission: CDSM2025`; use `Submission: CDSM2026` |
| Proceedings | None. "The meeting is organized as a workshop for the purpose of facilitating discussion and disseminating ideas." |
| Program | "Invited talks and presentations of accepted proposals" |
| Keynote | Prof. Teppo Felin, University of Utah |
| Sponsor | xplain-data.de |

The 2022 call asked for abstracts. The 2023 edition ran two days with a keynote by Dominik Janzing
(Amazon Science), a special session on LLMs and causality run by Microsoft Research, an industry
round table, 26 speakers, and 950 registered participants. Talk lengths are not published anywhere
I could reach; with 26 speakers over two virtual days the slots are short, so plan for 20 to 25
minutes plus questions and say the talk can be cut to 15.

**Format decision.** Submit a presentation proposal in the form of a two-page extended abstract:
title, a 200-word abstract, a 1,000-word description, a timed outline, the fit with the listed
topics, format and duration, a short speaker biography, and links. Send it as a PDF attachment
with the abstract repeated in the email body. Nothing is archival, so the text can be reused.

## 2. The organizers

### Paul Hünermund, Professor, TUM School of Management, Campus Heilbronn

Stated interests: causal AI, innovation economics, technology management, policy evaluation.
Associate editor of the *Journal of Causal Inference*. Current project: "Optimizing Human-AI
Interaction: Integrating Domain Knowledge into Causal AI Systems."

Publications that matter for this proposal:

- Hünermund and Bareinboim, "Causal Inference and Data Fusion in Econometrics," *The Econometrics
  Journal*, 2025 ([arXiv:1912.09104](https://arxiv.org/abs/1912.09104)). Combining imperfect data
  sources across heterogeneous populations; Haavelmo's structural approach joined with Pearl's
  graphs.
- Hünermund, Kaminski, Schmitt, "Causal Machine Learning and Business Decision Making," 2021
  ([CBS portal](https://research.cbs.dk/en/publications/causal-machine-learning-and-business-decision-making/)).
  Interviews and survey data on how firms use causal ML. "It highlights the crucial role of theory
  in causal inference and offers a new perspective on human-machine interaction for data-augmented
  decision making."
- Hünermund, Louw, Caspi, "Double Machine Learning and Automated Confounder Selection: A Cautionary
  Tale," *Journal of Causal Inference*, 2023. Automated variable selection admits bad controls when
  structure is left implicit.
- Hünermund, Louw, Rönkkö, "The Choice of Control Variables in Empirical Management Research: How
  Causal Diagrams Can Inform the Decision," *The Leadership Quarterly*, 2025.
- Hünermund and Louw, "On the Nuisance of Control Variables in Causal Regression Analysis,"
  *Organizational Research Methods*, 2025.
- Rohrer, Hünermund, Arslan, Elson, "That's a Lot to Process! Pitfalls of Popular Path Models,"
  2021. Causal assumptions must be stated; model fit cannot pick the causal direction.
- Hünermund, Bammens, Kaminski, "Causal Discovery in Strategic Management Research," DRUID 2023
  ([CBS portal](https://research.cbs.dk/en/publications/causal-discovery-in-strategic-management-research)).
  Discovery as a route to causal models for theory building, with its strengths and limits.
- Schmitt, Kaminski, Hünermund, "Structural Causal Models in Strategy: Opportunities and
  Boundaries," Academy of Management Proceedings, 2023.

Reading: domain knowledge and explicit structure are non-negotiable for him; automation earns
trust only when its assumptions are visible and checkable. Data fusion across populations is the
econometric cousin of reasoning across regimes.

Sources: [TUM profile](https://www.mgt.tum.de/professors-1/info/paul-huenermund),
[SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3867326),
[DataCamp podcast](https://www.datacamp.com/podcast/causal-ai-in-business-with-paul-hunermund).

### Jermain Kaminski, Assistant Professor, Maastricht University School of Business and Economics

Co-founder and co-chair of the meeting. Entrepreneurship and innovation. Methods from machine
learning and natural language processing on large text, audio, and video data, and "strategic
decision-making with causal machine learning."

Publications that matter for this proposal:

- The two joint papers with Hünermund above (causal ML and decision making; causal discovery in
  strategic management).
- Horck, Steens, Kaminski, "Synergizing Human Insight and Machine Learning: A Dual-Lens Approach,"
  *International Journal of Information Management Data Insights*, 2024.
- Pöhlmann, Santos, Kaminski, "Simulating the Entrepreneurial Self: A Cognitive Tool for Fostering
  Resilience and Confidence," Academy of Management Proceedings, 2025.
- Juhász, Wachs, Kaminski, Hidalgo, "The Software Complexity of Nations," *Research Policy*, 2026.
- Kaminski and Hopp, "Predicting Outcomes in Crowdfunding Campaigns with Textual, Visual, and
  Linguistic Signals," *Small Business Economics*, 2020.

Reading: human plus machine as a pairing, simulation as a decision aid, and an interest in software
ecosystems. He is the organizer most likely to ask what a practitioner can run tomorrow.

Sources: [Google Scholar](https://scholar.google.com/citations?user=b-HX-AYAAAAJ&hl=en),
[Maastricht profile](https://www.maastrichtuniversity.nl/jc-kaminski),
[Poets&Quants](https://poetsandquants.com/2024/05/18/2024-best-40-under-40-mba-professors-jermain-kaminski-maastricht-university-school-of-business-and-economics/).

### Beyers Louw, Assistant Professor, Rotterdam School of Management

Strategic Management and Entrepreneurship, ERIM member since 2024. His profile names "how people
make decisions under uncertainty" and "finding causality in complex data." Publications in
*Organizational Research Methods*, *The Leadership Quarterly*, and the *Journal of Causal
Inference*, all three co-authored with Hünermund and listed above.

Reading: the methodologist of the trio. He will weigh what a system lets a user interpret causally,
and how uncertainty about structure is carried rather than hidden.

Sources: [ERIM profile](https://www.erim.eur.nl/people/beyers-louw/),
[Google Scholar](https://scholar.google.com/citations?user=hEdt0nsAAAAJ&hl=en),
[RSM profile](https://www.rsm.nl/people/beyers-louw/).

### The keynote: Teppo Felin

Felin and Holweg, "Theory Is All You Need: AI, Human Cognition, and Causal Reasoning," *Strategy
Science*, 2024 ([INFORMS](https://pubsonline.informs.org/doi/10.1287/stsc.2024.0189)). Human
cognition as theory-based causal reasoning, set against data-based prediction; progress comes from
theory, experimentation, and intervention. The call describes him as challenging "purely predictive
approaches to decision-making."

## 3. What the three have in common

1. Decisions are the unit of analysis. Every paper above ends in a managerial or strategic
   decision, not in an estimate.
2. Theory and domain knowledge must be explicit. The control-variable papers, the path-model paper,
   and the DML paper all show what goes wrong when structure stays implicit.
3. Automation is welcome inside guardrails. Causal discovery and causal ML are endorsed, with
   their assumptions on the table.
4. Humans and machines share the loop. "Human-machine interaction for data-augmented decision
   making" is their phrase.
5. Practice counts. The meeting exists to connect industry and academia, and the topic list names
   root-cause analysis, open-source software, and organizational adoption.

## 4. How the proposal maps onto those interests

The hook: **from dynamic decisions to safeguarded actions in a dynamic world.**

| Their interest | What the talk shows | DeepCausality piece |
|---|---|---|
| Decisions under uncertainty and regime change (Louw, Hünermund's data fusion) | The regime is a declared input the model reads, so one model serves several regimes and reports which one it is in | Explicit context hypergraph; regime classification |
| Theory made explicit (all three, Felin) | Causal structure and causal rules are code that a reviewer can read, and both may change at runtime | Causaloid graph; adaptive rules |
| Classical methods stay valid (all three) | SCM, DBN, Granger, and the Rubin model run as special cases of the same definition | `examples/classical_causality_examples` |
| Discovery with visible assumptions (DRUID 2023, DML paper) | Discovery proposes structure with its equivalence-class uncertainty; the analyst encodes it | Causal Discovery Language: SURD, MRMR, BRCD with bootstrap CPDAG uncertainty |
| Simulation before commitment (Kaminski) | A running model forks into counterfactual branches; each branch carries its own log; the branches reduce to a verdict | Counterfactual forks on the causal monad |
| Human-machine interaction, governance (Hünermund's current project, topic list) | A deontic safety layer decides whether an action is obligatory, optional, or impermissible, and writes why | Causal State Machine plus Effect Ethos with audit log |
| Industry adoption (meeting purpose) | A network operations product deployed at United Airlines | ServiceRadar case study |
| Open-source software (topic list) | 29 crates, 113 runnable examples, MIT license, Linux Foundation project | Project health |

Topics from the call the proposal answers directly: causal discovery and root-cause analysis;
open-source software for causal inference; causal ML/AI for business decision-making;
organizational challenges and best practice for implementing causal inference; interplay between
causality and generative AI (the safety layer governs actions regardless of whether a person, a
model, or an agent proposed them).

## 5. What to keep out

- Category theory. Say "a composition rule that threads five things through every step" once and
  move on. The audience is econometrics and business analytics.
- Physics. The plasma-blackout example is the deepest demonstration of regime change in the
  repository and the wrong opener for this room. One sentence, if any.
- Scrith. The meeting is about DeepCausality and its use; the joint project stays on the LF deck.
- Claims about the United Airlines deployment beyond what the LF deck states: real-time anomaly
  detection at about one million operations per second, built on the hypergraph, the core engine,
  and dynamic context.
- Talk length. It is unpublished; offer 25 minutes and flexibility.
