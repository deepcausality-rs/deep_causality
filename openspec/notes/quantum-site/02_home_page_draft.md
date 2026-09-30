# Home page draft: DeepCausality Quantum

Steps 1 to 4 of `docs/writing_guides/TurnRawMaterialintoWriting.pdf`, applied to
`01_raw_information.md`. Written for a reader who works with quantum devices or quantum codes, knows
that field's vocabulary and has never seen this library. Every sentence traces to a fact in the raw
file; the letters in brackets after a paragraph name the sources.

Scope: the hero and the seven sections of the home page. The inner pages (`/how-it-works/`,
`/checks/`, `/examples/`, `/proof/`, `/boundaries/`, `/start/`) follow the same four steps from the
same raw file and are not drafted here.

## Step 1: Prioritize

Eight paragraphs. Each has one main point, which is its topic sentence, and its last sentence returns
to that point.

| # | Paragraph | Main point (topic sentence) | Facts from |
|---|---|---|---|
| 1 | Hero | The library plans the experiments that tell rival causes apart. | A, B, D, H |
| 2 | The problem | Passive observation cannot separate a direct cause from a shared one. | H |
| 3 | The four calls | One program carries the example from candidates to verdict in four calls. | A, B, C, D |
| 4 | The plan | Two experiments, at a total cost of 2, separate all three candidates. | B, H, H-run |
| 5 | The result | One observation, of E1, leaves one candidate standing. | H, H-run |
| 6 | Other subjects | The same builder takes a model, a code and an abstraction, and each offers the checks that suit it. | F, I, I-run |
| 7 | Limits | Six limits mark where the library stops today. | A, B, E, F, G |
| 8 | Run it | Three shell lines run the crosstalk example. | H, E |

## Steps 2 and 3: Connect the middle sentences, and transition

Each paragraph is followed by its connectors and its transition into the next paragraph. Connector
types follow the guide's list: chronological, causal, exemplifying, addition, contrast,
reformulation, qualification.

### 1. Hero

> **Three rival causes for correlated qubit errors.**
> **Two experiments tell them apart.**
>
> DeepCausality Quantum is a Rust library that plans the experiments which tell rival causes of a
> quantum system's behaviour apart. You declare the candidate causes, each a causal structure over
> your qubits, and you give every available experiment a cost and the read-out each candidate
> predicts. The library then checks each candidate for consistency, selects the cheapest set of
> experiments that separates the candidate pairs, and names the candidate your results leave standing.
> The example below applies that sequence to two qubits with correlated errors: three candidates,
> five available experiments, and a plan of two.

- Connectors: "then" (chronological); "below" (points to the example the closing sentence names).
- Closing sentence returns to the main point by stating the plan the library produced.
- Transition: paragraph 2 opens on the two qubits the last sentence introduced.

### 2. The problem

> Passive observation cannot separate a direct cause of correlated errors from a shared one. Qubit 1
> may drive qubit 2, qubit 2 may drive qubit 1, or a shared source of noise may drive both. The three
> structures are Markov equivalent, so they fit the passive data equally well: a passive read-out of
> both qubits predicts 0.04 under each of them. For a direct cause the example concludes that a
> scheduling or echo fix applies, so the choice among the three changes the repair. Telling the three
> apart therefore takes an intervention that the causes answer differently.

Sources: H (`main.rs`, `model.rs`, `constants.rs`).

- Connectors: "so" and "therefore" (causal); "for a direct cause" (exemplifying).
- Closing sentence returns to the main point and names what the next paragraph supplies.
- Transition: paragraph 3 opens with the program that supplies the intervention.

### 3. The four calls

> One Rust program carries the example from candidates to verdict in four calls. The first,
> `config`, names the plant, the candidates and the experiments, and `build()` refuses any candidate
> whose structure contains a cycle. The second, `validate`, screens each candidate: factors that
> share a Hilbert leg must pairwise commute, and the structure must admit a circuit decomposition
> with the same connectivity. The third, `control`, forks one world per admitted candidate and asks
> `design` for the cheapest set of experiments that separates the candidate pairs, listing any pair
> that no experiment separates. The fourth, `adjudicate`, compares each world's prediction with the
> observation and returns the survivor or names why none survives. Each call reports what it
> measured, the threshold it used and how many items it examined, so a reader can see how close a
> candidate came to rejection.

Sources: A, B, C, D.

- Connectors: "first, second, third, fourth" (chronological); "so" (causal).
- Closing sentence returns to the main point: the four calls share one report form.
- Transition: paragraph 4 opens on the third call, the one that decides how many experiments the work
  needs.

### 4. The plan

> Two experiments, at a total cost of 2, separate all three candidates. Experiment E1 holds qubit 1
> excited and reads qubit 2; it predicts 0.40 if qubit 1 drives qubit 2 and 0.10 under each of the
> other two candidates. Experiment E2 holds qubit 2 excited and reads qubit 1; it predicts 0.40 if
> qubit 2 drives qubit 1 and 0.10 under the others. E1 therefore separates the first candidate from
> both rivals, and E2 separates the second candidate from the shared bath. `design` measures the
> separation of two predicted read-outs as their Bhattacharyya distance in bits, added up over the
> shots, and it counts a pair as separated at 5 bits or more over 1024 shots. The example supplies
> the costs and the predictions: 1 for each intervention, 2 for an echo and 200 for process
> tomography, in one arbitrary unit. Within those inputs, the two interventions cover all three
> pairs at cost 2, and tomography would cover them at cost 200.

Sources: B, H, H-run. The run output lists E1 as resolving (Q1→Q2, Q2→Q1) and (Q1→Q2, bath), and E2 as
resolving (Q1→Q2, Q2→Q1) and (Q2→Q1, bath).

- Connectors: "therefore" (causal); "within those inputs" (qualification).
- Closing sentence returns to the main point and states the cost as a consequence of the inputs.
- Transition: paragraph 5 opens on what the run observed, since the plan is only a plan until an
  experiment runs.

### 5. The result

> One observation, of experiment E1, leaves one candidate standing. The example draws 1024 shots from
> a simulated sampler seeded at 20260821 and reads 0.377 with a standard error of 0.015. It accepts a
> candidate whose prediction lies within three standard errors of that read-out: the first candidate
> predicts 0.40 and is accepted, and the other two predict 0.10 and are rejected. The adjudication
> returns the first candidate as the survivor, 100.1 bits from its nearest rival. The sampler draws
> from the first candidate's own prediction, so the run also checks that the library recovers the
> structure that generated the data, and it does. Had either rival generated the data, E1 would read
> near 0.10 and leave both rivals standing, and E2 would then separate them. This run needed one
> observation, and the plan of two guarantees that any of the three causes needs at most two.

Sources: H, H-run. The sentence beginning "Had either rival" is derived from the prediction tables in
`constants.rs` and was not run.

- Connectors: "so" (causal); "also" (addition); "had ... would" (contrast); "then" (chronological).
- Closing sentence returns to the main point by counting the observations the run needed.
- Transition: paragraph 6 opens by widening from the crosstalk subject to the other subjects.

### 6. Other subjects

> The same builder takes a quantum causal model, a code and an abstraction between models as its
> subject, and each subject offers the checks that suit it. A model subject runs the two checks that
> freeze a causal graph: commuting factors let the graph freeze, and non-commuting factors abort the
> freeze, name the offending pair and roll a dynamic graph back. A code subject reads a chain complex
> as a CSS code and decides it with exact 𝔽₂ arithmetic. On the 4 × 4 toric code, [[32, 2]], four
> checks accept, and a weight bound of 3 rejects at the first Z check with margin 4/3. That code has
> 32 qubits and the in-process simulator stops at 24, so the exact checks decide a code the simulator
> cannot run. An abstraction links a physical circuit to a logical one and records a composition law,
> ε ≤ ‖τ₂‖post · ε₁ + ‖τ₁‖pre · ε₂. In the code-switching example, depolarising noise of 1/10 on one
> logical wire gives a measured residual of 0.4619 under a recorded bound of 3.695. Every subject
> reports in the form the crosstalk run used: a measured value, a threshold and a count of items
> examined.

Sources: A, F, I, I-run (`qcl_qcm_freeze`, `qcl_geometric_qec`, `qcl_code_switching`).

- Connectors: "so" (causal); a colon and a semicolon carry the exemplifying links.
- Closing sentence returns to the main point: each subject, one report form.
- Transition: paragraph 7 opens on where these subjects and checks stop.

### 7. Limits

> Six limits mark where the library stops today.
>
> 1. **Evidence is simulated.** Shots come from a seeded sampler on a density matrix, or from an
>    in-process simulator of at most 24 qubits under the `qpu` feature, and no vendor adapter ships.
> 2. **The crate contains no decoder.** The decoder abstraction validates a detector error model
>    against a circuit and takes the decoder's reading of the record as data.
> 3. **Cyclic structures are refused.** `build()` rejects them by decision, before any check runs.
> 4. **You supply the predicted read-outs and the costs.** The crosstalk example types them in;
>    `Hypothesis::evaluate` computes a prediction from a factorization for callers who want one.
> 5. **Scale is bounded.** `design` covers at most 7 candidates by default, the ideal recovery of a
>    code stops at 10 qubits, and the numeric semantics refuses Choi operators above 2²⁴ entries.
> 6. **The pipeline needs `std`.** It sits behind the `qcm` feature, which implies `std`.
>
> Each limit is a fact about the present release, and the boundaries page names the file that shows it.

Sources: A, B, E, F, G, H.

- Connectors: an enumeration; the numbered list is the addition link.
- Closing sentence returns to the main point and points to the page that holds the evidence.
- Transition: paragraph 8 opens with the action that lets the reader check the page.

### 8. Run it

> Three shell lines run the crosstalk example from a clean clone.
>
> ```bash
> git clone https://github.com/deepcausality-rs/deep_causality
> cd deep_causality
> cargo run --release -p quantum_examples --example qcl_crosstalk
> ```
>
> The run prints each stage in the order of this page, from the refused cycle to the survivor. The
> seed is fixed, so a second run prints the same output.

Sources: H, H-run, E (`qpu/prng.rs`).

- Connectors: "so" (causal).
- Closing sentence returns to the main point: the run is repeatable from a clean clone.

## Step 4: Check the verbs

Asked of each paragraph: strong or weak, whether the chain tells the whole story, whether it returns to
the main point, whether it repeats, whether tense and agreement hold.

| # | Verb chain | Finding |
|---|---|---|
| 1 | is, plans, tell apart; declare, give; checks, selects, names; applies | Strong after the definition. "Is" appears once, in the definition. The chain reads declare, check, select, name, which is the order the library works in. |
| 2 | cannot separate; may drive; are, fit, predicts; concludes, changes; takes | "Cannot separate" is the topic sentence's claim, and the rest supports it. "Takes" recurs in paragraph 6 with a different subject and sense. |
| 3 | carries; names, refuses; screens, must commute, must admit; forks, asks, listing; compares, returns, names; reports | Each call gets one subject and one verb group. "Names" appears three times, for `config`, `build()` and `adjudicate`; the first two could read "declares" if the repetition grates. |
| 4 | separate; holds, reads, predicts; separates; measures, counts; supplies; cover | "Separate" repeats by design: it is the paragraph's word. "Supplies" marks the inputs. All present tense. |
| 5 | leaves standing; draws, reads; accepts, predicts, rejected; returns; draws, checks, recovers; would read, would separate; needed, guarantees | The conditional "had ... would" sentence is the only non-present tense and it is meant. "Draws" appears twice, for the sampler in two roles. |
| 6 | takes, offers; runs, let ... freeze, abort, roll back; reads, decides, accept, rejects; stops, decide; links, records, gives; reports | The longest paragraph. It holds four subjects and could split into a model-and-code paragraph and an abstraction paragraph if it reads long aloud. |
| 7 | mark; are, come, ship; contains, validates, takes; refuses; types, computes; covers, stops, refuses; needs, implies | Verbs are plain and factual. The negative forms ("no decoder", "no vendor adapter") state limits, as the maintainer's site rules allow. |
| 8 | run; prints; is fixed, prints | Short and concrete. |

## Read-aloud notes

- The draft holds 62 sentences with a mean of about 17 words. The longest are 34 words: paragraph 4,
  sentence 5 (`design` measures the separation ...) and paragraph 6, sentence 2 (the model subject).
  None exceeds the guide's 35-word ceiling.
- No sentence opens with "Additionally", "Furthermore" or "Moreover".
- Em dashes: none.

## Questions for the maintainer

1. Headline. This draft uses "Three rival causes for correlated qubit errors. Two experiments tell
   them apart." Alternatives: "Choose between rival causes with the cheapest experiments that
   separate them." (general) or "Correlated qubit errors, three candidate causes, two experiments."
   (compact).
2. Repair line. Paragraph 2 repeats the example's closing line about a scheduling or echo fix.
   Removing it costs nothing else in the draft.
3. Timing. The draft states no run time, because this session did not measure one.
4. Paragraph 6 length. Split now, or after a read-aloud?
