# Sensing page draft: DeepCausality Quantum

Steps 1 to 4 of `docs/writing_guides/TurnRawMaterialintoWriting.pdf`, applied to
`03_sensing_raw_information.md`. Written for a reader who works with atom interferometers, quantum
navigation or quantum devices, reads English as a second language at about CEFR B2 to C1, knows the
vocabulary of their own field and has never seen this library. Every sentence traces to a fact in the
raw file; the letters after a section name the sources.

The section drafts below carry the headings of the first draft. Two later passes rewrote every
heading, and review moved cards between sections 4 and 5. The closing sections of this note record
each pass; `website/quantum/src/pages/sensing/index.astro` holds the final text.

The page follows the home page's form: a hero with one figure, then sections that each open with a
question as eyebrow, a claim as heading, a short lede, and one figure. Each section ends on the point
the next one picks up.

## Step 1: Prioritize

Nine sections. Each has one main point, its heading; its last sentence returns to that point or hands
over to the next section.

| # | Section | Main point (heading) | Figure | Facts from |
|---|---|---|---|---|
| 1 | Hero | A gravimeter reads 5 µGal low; six experiments find out why. | Seven explanations, one reading | K, K-run |
| 2 | Why name the cause | An accuracy budget is a list of named causes. | Two cited numbers | N1 to N5 |
| 3 | Which change | Six changes to the instrument separate all seven causes. | Which change moves which cause | K, K-run, P, Q |
| 4 | The decision | The campaign stops as soon as one cause remains. | Time each campaign took | K-run, P |
| 5 | Where it ends | Three runs show where the method ends. | Three cards | K-run |
| 6 | A second instrument | The same method separates a harmless correlation from a biased gradient. | What the dark cloud reads | L, L-run |
| 7 | Checked against papers | Eight published attributions, rerun from the papers' numbers. | One row per paper | M, M-run |
| 8 | Limits | Where the sensing examples stop. | List | K, L, M |
| 9 | Run it | Three commands rerun both examples and a verification. | Code | K, L, M |

## Steps 2 and 3: Connect the middle sentences, and transition

### 1. Hero

> Eyebrow: **Quantum sensing**
>
> **Discover the cause of a gravimeter reading error**
>
> A gravimeter reads 5 µGal low, and seven known effects could each explain it. A longer passive run
> fits all seven equally well. DeepCausality Quantum plans the cheapest changes to the instrument that tell the effects
> apart, prices each change in instrument time, runs them until one explanation remains, and names it.
>
> Gloss: The run below uses a published instrument and simulated observations. Eight further runs check
> the method against published results.

Figure: **Seven explanations, one reading.** Seven rows, one per candidate (H1 Coriolis … H7
clipping), each a dot on one shared horizontal scale at −5 µGal, under a single band labelled "the
passive reading". Caption: "Each candidate is sized to produce the offset, so all seven predict the
same passive reading."

Sources: K, K-run.

- Connectors: "and" (addition); the second sentence lists the library's steps in order (chronological).
- The gloss returns to the main point by saying what the reader will see.
- Transition: section 2 opens on why the offset's cause matters to a sensing team.

### 2. Why name the cause

> Eyebrow: **Why name the cause?**
> Heading: **Every budget line names a cause.**
>
> An atom gravimeter measures gravity through the phase of falling atoms, and many effects shift that
> phase: the Earth's rotation, stray magnetic fields, imperfect laser beams, a tilted instrument. An
> accuracy budget lists each effect with its size. The laser wavefront is the largest limit on
> cold-atom gravimeters (Karcher et al.), and in one published budget it carries 4.0 µGal of a
> 5.2 µGal total uncertainty (Louchet-Chauvet et al.).
>
> The instruments now work outside the laboratory. On a 29 m vessel, a quantum gravimeter corrected
> the ship's inertial navigation over 45 nautical miles with GNSS switched off. The corrected position
> ended 2.2 nautical miles from the truth, the uncorrected one 14 (Everitt et al.). At sea, the ship's
> rotation adds errors of its own (Lellouch and Holynski).
>
> Each new effect needs its own line in the budget before anyone can correct it, and each line rests
> on an experiment that moves that effect and leaves the others in place. Choosing those experiments
> is the library's job.

Figure: **Two cited numbers**, as two stat cards: "4.0 of 5.2 µGal: the wavefront's share of one
gravimeter's uncertainty budget" and "2.2 against 14 nautical miles: position error after 45 nautical
miles at sea, with and without the quantum gravimeter". Each card names its paper.

Sources: N1, N2, N3, N5.

- Connectors: a colon introduces the examples (exemplifying); "now" (chronological, from the
  laboratory to the field); "At sea" (addition); "Each … and each" (causal chain to the main point).
- The last paragraph returns to the main point, the named line, and hands over to section 3.
- Transition: section 3 opens on the experiments the library chooses.

### 3. Which change

> Eyebrow: **Which change?**
> Heading: **Six changes separate seven causes.**
>
> Each effect answers a different change to the instrument. Turning the sensor head 180° flips the
> Coriolis and clipping shifts and leaves the rest in place. Reversing the laser's wave vector flips
> the quadratic Zeeman shift. Halving the Rabi frequency halves the light shift. You write these
> responses once, in a response model, and the library computes what every change predicts under
> every candidate.
>
> Eight changes are on offer. The library prices each as its setup time plus the time its
> measurements take, and picks the cheapest set that separates every pair of candidates by at least
> 5 bits. That margin keeps the chance of mistaking one candidate for another below 1 in 64. Six
> changes do it, in 4,956 s.
>
> The plan skips the wavefront's own test, a temperature scan with 20,000 s of setup. Once the other
> six candidates each have a change that moves them, the wavefront is the explanation left when all
> six fail, so the plan names it by elimination.

Figure: **Which change moves which cause.** The existing matrix: candidates as rows, the eight changes
as columns, a dot where a change moves a candidate, the six planned columns highlighted, setup time
under each column, and "by elimination" beside the wavefront.

Sources: K, K-run, P, Q (the 1 in 64).

- Connectors: "Each … different" (contrast across the list); "and" (causal, from the model to the
  predictions); "That margin" (reformulating); "Once … so" (causal).
- The last sentence returns to the main point: six changes, and the seventh cause by elimination.
- Transition: section 4 opens on running the plan.

### 4. The decision

> Eyebrow: **The decision**
> Heading: **The campaign stops when one cause remains.**
>
> The library runs the plan as a campaign. It starts with the cheapest change that can separate the
> candidates still standing, compares every candidate's prediction with the result, and stops as soon as one
> candidate fits and stands clear of the rest. When the quadratic Zeeman shift caused the offset, the
> first change settled it in 166 s. When the wavefront caused it, all six other candidates had to fail
> first, which took six changes and 4,953 s.
>
> The tide moves gravity by far more than 5 µGal during a campaign; one published budget corrects
> −18.8 µGal for it (Louchet-Chauvet et al.). Each change therefore carries the tide at the time it
> runs, and the campaign plans again whenever the tide moves a prediction. In every run, the campaign
> names the cause the observations came from.

Figure: **Time each campaign took.** Seven horizontal bars, one per cause, length proportional to the
seconds spent (166 to 4,953), each bar labelled with the changes run in order (for example "E1 → E8 →
E3 → E2") and the number of re-plans. A tick at 4,956 s marks the full plan. Caption: "Every
observation is drawn from the named cause. Setup times and the tide are placeholders for a lab's own
values."

Sources: K, K-run, N5, P.

- Connectors: "When … When" (contrast); "therefore" (causal); "In every run" (qualification).
- The last sentence returns to the main point: the campaign stops at the right cause.
- Transition: section 5 opens on runs where the answer is wrong or absent.

### 5. Where it ends

> Eyebrow: **Where it ends**
> Heading: **Three runs mark its limits.**
>
> The answer is only as good as the model and the list of candidates. Three runs show what happens at
> their edges.

Figure: **Three cards.**

1. **A reading too high for one cause.** The light shift pushes the reading 5 µGal high. A tilted
   instrument only reads low, so the library refuses tilt before planning, and the campaign names the
   light shift.
2. **A record without the tide.** The instrument's records leave out the tide, so every candidate
   predicts −5.00 µGal against a reading of 72.41 µGal. The library refuses all seven.
3. **A cause missing from the list.** Clipping causes the offset, but the list leaves it out. The
   library names Coriolis, which a 180° turn flips the same way. Farah et al. traced a wrong-sign
   Coriolis result to clipping in a real gravimeter.

Sources: K-run, M (Farah et al.).

- Connectors: "so" (causal) inside each card; the cards are an enumeration (addition).
- The heading states the point; the cards return to it, one edge each.
- Transition: section 6 opens with a second instrument, to show the method is not tied to one.

### 6. A second instrument

> Eyebrow: **Gradiometer**
> Heading: **One experiment exposes a gradiometer leak.**
>
> A gravity gradiometer compares two atom clouds that share one laser, so their readings rise and fall
> together. If the shared laser alone causes that, the gradient stays correct. If light from one
> cloud's detection leaks into the other's, or the platform rotates, the gradient carries a bias. All
> four explanations fit the passive data.
>
> One 60 s change exposes a leak: light up one cloud, keep the other dark, and read the dark one. A
> leak lifts the dark cloud's signal from 0.020 to 0.079. A 6 % leak biases the gradient by −356 E,
> about 3.6 × 10⁻⁷ s⁻². Rotation needs a deliberate turn of the platform, and the full plan takes
> 722 s.

Figure: **What the dark cloud reads.** Four tracks on a 0 to 0.1 scale, one per candidate: the leak
from A to B at 0.079, the other three at 0.020. Caption: "Light up cloud A, keep cloud B dark, read B.
Placeholders: a 6 % leak and a 2 % dark background."

Sources: L, L-run.

- Connectors: "so" (causal); "If … If" (contrast); "All four" (summary).
- The last sentence returns to the main point with the bias and the plan's cost.
- Transition: section 7 asks whether the method reaches the answers published papers reached.

### 7. Checked against papers

> Eyebrow: **Published results**
> Heading: **Eight papers, same answers.**
>
> Simulated runs show the method works on its own terms. To test it against real data, eight
> verifications feed published numbers into the same planner and judge, and check whether they reach the
> papers' conclusions. All 53 checks pass. Where a paper's own numbers disagree with each other, the
> verification records it.

Figure: **One row per paper.** Columns: paper; the question it answered; same answer?; what the
rerun also found.

| Paper | Question | Same answer? | Also found |
|---|---|---|---|
| Karcher et al. 2018 | Which effect shifts gravity as the atoms cool? | Yes, the wavefront | The paper's bound on atom interactions holds at one standard error; at three, interactions remain possible. |
| Farah et al. 2014 | Why did a Coriolis test show the wrong sign? | Yes, detection clipping | The text uses two conversion factors and one wrong unit. |
| Louchet-Chauvet et al. 2011 | Which change separates Coriolis from the wavefront? | Yes, a 180° turn | One uncertainty total and one equation disagree with their own inputs. |
| Sarovar et al. 2020 | Which qubits cross-talk, and how? | Yes, in all four cases | The paper's own detection method, rerun on the same data, misses the published answer in three. |
| Gauguet et al. 2008 | Is the shift a two-photon light shift? | Yes, 32 against 33 mrad | Repeat measurements spread ±16 %, against a stated ±10 %. |
| Ménoret et al. 2018 | How far does averaging help? | Yes, to one hour | Day-long averages show noise beyond white noise. |
| Sorrentino et al. 2014 | What drives each drift of a gradiometer? | Yes, three of three | The text's list of main drifts leaves out the third largest. |
| Hu et al. 2017 | How large is the quadratic Zeeman bias? | Yes, 2.01 against 2.04 µGal | One coefficient's unit is printed wrong. |

Closing line: "The verifications' README lists each check, its numbers and the command that reruns it."

Sources: M, M-run.

- Connectors: "To test it" (contrast with simulation); "Where" (qualification).
- The closing line returns to the main point by pointing to the evidence.
- Transition: section 8 states the limits.

### 8. Limits

> Eyebrow: **Limits**
> Heading: **Where the examples stop.**
>
> - The examples simulate their observations; the verifications read published values and data regenerated from published models.
> - A lab supplies the setup times, the tide, the leak and the dark background.
> - The library never names a missing cause; a change that rules out every listed cause shows the list is incomplete.
> - Measurement time follows white noise, within the instrument's white-noise range.
>
> Every limit, with the file that shows it, is on the boundaries page.

Sources: K, L, M.

### 9. Run it

> Eyebrow: **Run it**
> Heading: **Rerun it in three commands.**
>
> ```bash
> cargo run --release -p quantum_examples --example qcl_gravimeter_systematics
> cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk
> cargo run --release -p deep_causality_quantum --features qcm --example verification_v1_wavefront
> ```
>
> Both examples are seeded, so a second run prints the same output.

Sources: K, L, M.

## Step 4: Check the verbs

| # | Verb chain | Finding |
|---|---|---|
| 1 | reads, find out; could explain, fits; plans, tell apart, prices, runs, remains, names | The chain is the library's order of work. "Fits" carries the problem. |
| 2 | measures, shift; lists; is, carries; work; corrected, ended; adds; needs, rests, moves, leaves; is | "Is" twice: the largest limit, and the job. Both define. |
| 3 | answers; flips, leaves; flips; halves; write, computes; are on offer; prices, picks, separates; keeps; do; skips; have, moves, is left, fail, names | "Flips" repeats by design: the reader learns what a change does. |
| 4 | runs; starts, rule out, compares, stops, fits, stands clear; caused, settled; caused, had to fail, took; moves, corrects; carries, runs, plans, moves; names | Strong and concrete; the two "caused" sentences make the contrast. |
| 5 | is; show; pushes, reads, refuses, names; leave out, predicts, refuses; causes, leaves out, names, flips, met | Three short causal chains. |
| 6 | compares, share, rise and fall; causes, stays; leaks, rotates, carries; fit; exposes, light up, keep, read; lifts; biases; needs, takes | Plain verbs a B2 reader knows. |
| 7 | show, works; feed, judge, check, reach; pass; disagree, records | The table carries the detail; the lede stays short. |

## Read-aloud notes

- Sentences average about 15 words; the longest is about 30 words (section 3, second paragraph,
  "The library prices each …").
- One idea per sentence and at most two numbers in a prose sentence; the tables and figures carry the
  rest.
- Defined where first used: "candidate" (section 1, implicit: one possible explanation), "response
  model" (section 3), "5 bits" (section 3, with its meaning), "campaign" (section 4), "by elimination"
  (section 3).
- No paper-internal references (figure, table or equation numbers) on the page; they stay in the
  verifications' README.
- No sentence opens with "Additionally", "Furthermore" or "Moreover". No em dashes.

## Questions for the maintainer

1. **Headline.** Decided: "Discover the cause of a gravimeter reading error"; the 5 µGal puzzle moves
   into the hero's first sentence.
2. **The 1 in 64.** It follows from the Bhattacharyya bound under equal priors (Kailath 1967), and the
   crate does not state it. Keep it on the page with the citation, or drop it and say only "5 bits"?
3. **Paper findings in plain words.** Section 7 states each paper's slips in one plain line and moves
   the detail to the README. Is that the right level for the public page?
4. **The gradiometer.** Keep it as section 6, or move it to its example page with a link only?

## Headline edit (Paramedic pass)

Every heading rewritten for the claim alone: prepositional chains cut ("changes to the instrument"),
"is" and passive forms turned into active verbs with the doer first ("Tilt is refused" became "The
baseline refuses tilt"), and description dropped where the section's lede already describes. Figure
titles and table headers follow: "Time to name the cause", "B's read-out, A lit, B dark", "Question",
"Also found". The h1 stays as the maintainer chose it.

## Headline edit, second pass: research and rewrite

The first pass made the headings short and kept them as statements of mechanism ("The campaign
stops when one cause remains"), which a reader already assumes. Research on headlines agrees on five
points:

1. State the takeaway, not the topic: a sentence headline that gives the main message, which
   audiences understand and remember better (Alley, assertion-evidence structure,
   https://writing.engr.psu.edu/research.html).
2. Promise a benefit and be specific; benefit headlines are read far more often (Ogilvy, *Ogilvy on
   Advertising*), and the "4 U's" ask for useful and ultra-specific, with numbers
   (https://unbounce.com/copywriting/gut-check-your-landing-page-headlines/).
3. Short, front-loaded and understandable out of context; the BBC's best average five words
   (Nielsen, https://www.nngroup.com/articles/worlds-best-headlines-bbc-news/).
4. No label subheads and no spoilers; curiosity or surprise, without turning cryptic
   (https://smartblogger.com/subhead-blunders/).
5. Wit sparingly: puns and idioms cost readers whose English is a second language, so plain
   contrast and concrete numbers carry the surprise here.

Each heading now states the section's non-obvious fact: a number, a contrast or a consequence. "Name"
leaves the page's prose; "find", "blame" and "report" take its place, and redundant uses go.

| Section | Heading |
|---|---|
| 2 | One effect carries most of the error. |
| 3 | Skip the 5.6-hour scan and still find the wavefront. |
| 4 | Under three minutes for one cause. |
| 5 | Leave out a cause, and another takes the blame. |
| 6 | A one-minute test exposes a light leak. |
| 7 | Published numbers in, real answers out. |
| 8 | What your lab still supplies. |
| 9 | Verify the numbers. |

Cards: "Too high for tilt.", "Forget the tide, and all seven fail.", "Coriolis takes the blame."
Figure title: "Time to an answer". The numbers: 20,000 s is 5.6 h; 166 s is under three minutes;
4,953 s is 83 minutes; E1 runs 60 s.

## Sections 4 and 5: tilt card moved

The tilt card shows the library working, so it sits in section 4, below the campaign bars, where the
passive reading drops candidates before the plan. Section 5 keeps the two runs that leave an input
out.

Section 4, first paragraph, opens:

> Every run starts with the passive reading. The library drops any candidate the reading
> contradicts, plans for the rest, and runs the plan as a campaign. The campaign starts with the
> cheapest change that can separate the candidates still standing, …

Section 5:

> Eyebrow: **Missing inputs**
> Heading: **Leave out a cause, and another takes the blame.**
>
> The library weighs only the causes on your list, against the records you give it. When the records
> leave out the tide, no cause fits, and the library refuses all seven. When the true cause is missing
> from the list, the plan drops the one change that would expose it, and the campaign blames a cause
> that behaves the same way.

Cards: "No tide on record" and "Clipping off the list". The change clipping needs is E8, cloud
displaced: the model gives Coriolis `size × heading` and clipping `(size + slope × x) × heading`, so
only a displaced cloud separates them (K-run, `model.rs`).

## Sections 4 and 5: one omission per section

Section 5 held two omissions with opposite outcomes: a missing tide gets no answer, a missing cause
gets a wrong one. Its heading covered only the second. The tide card shows the passive-reading check,
the same mechanism as the tilt card, so both sit in section 4. Section 5 keeps the missing cause.

> Eyebrow: **Missing cause**
> Heading: **Leave out a cause, and another takes the blame.**
>
> The library weighs only the causes on your list, and its plan leaves out any change that only a
> missing cause would answer. If the missing cause answers the planned changes the way a listed cause
> does, the listed cause takes the blame, and the output reads like a correct run.

Card: "Clipping causes the offset, and the list leaves it out. A displaced cloud is the one change
that separates clipping from Coriolis, and with clipping unlisted the plan drops it. The library
blames Coriolis. Farah et al. traced a wrong-sign Coriolis result to clipping in a real gravimeter."

Sources: K-run lines 69 and 71 (plan without E8; "H1 Coriolis survives", the same form as the
correct Coriolis run on line 13); `model.rs` (Coriolis `size × heading`, clipping
`(size + slope × x) × heading`).
