# DeepCausality Quantum — project website

Astro static site for the `deep_causality_quantum` crate. Served at
`quantum.deepcausality.com`.

## Run it

```bash
pnpm install
pnpm dev            # local server
pnpm build          # -> dist/
pnpm check          # astro check; needs TypeScript 6.x, see below
pnpm check:tokens   # verify the token mirror has not drifted
pnpm check:prose    # build, then lint prose and internal links (scripts/sitecheck.py)
pnpm check:prose:test   # the lint's own tests
```

There is **no Bazel target for this site**; `pnpm build` is the only build path.
`.bazelignore` lists `website/quantum/node_modules` so the Rust build ignores it.
The same holds for the other three sites.

## Deploy

Cloudflare Workers Builds. The dashboard supplies the root directory
(`/website/quantum/`) and the deploy command (`npx wrangler deploy`); the build
lives in `wrangler.toml` as a `[build]` command, so an empty dashboard build
field cannot break the deploy.

The Worker is **`quantum`**, and `wrangler.toml` must carry that exact name. See [`../README.md`](../README.md) for the two ways this fails
quietly.

## Design

The binding spec is [`../web/DESIGN.md`](../web/DESIGN.md); the descriptive
companion is [`../web_design/`](../web_design/). This site follows both and
shares `website/cfd`'s two deliberate differences from the project website:

1. **Every §12 convention is a shared utility in `global.css`.** The eyebrow,
   panel, reticle, corner-bracket, chip and hairline-list rules are declared
   once. No component redeclares them.
2. **The tokens §12.7 names as missing exist.** `--fw-heading`, `--header-h`,
   `--w-panel`, `--measure-lede`, plus `--stagger` / `--dur-draw` / `--dur-node`
   live in `src/styles/tokens-quantum.css`. Every duration is a token, so the
   reduced-motion contract in §6 holds with no exceptions.

`src/styles/tokens.css` is a **byte-identical mirror** of
`../web/src/styles/tokens.css`. Do not edit values in it. Edit the source, copy
it across, then run `pnpm check:tokens`. Site-local tokens go in
`tokens-quantum.css`.

### Inherited defect

The light-mode accent (`#0a8a98`) fails WCAG AA at 4.12:1, which affects body
links and the primary CTA. The defect comes from the shared token set and is
recorded in DESIGN.md §2.1 and §10. Fixing it is a colour decision for the whole
project; this site does not diverge on it.

## Content rules

**Every claim on this site traces to something committed in this repository**:
the crate source under `deep_causality_quantum/src/`, a paper under
`deep_causality_quantum/papers/`, the LEAN tree under
`lean/DeepCausalityFormal/Quantum/`, `lean/THEOREM_MAP.md`, the output of an
example under `examples/quantum_examples/`, or, for what the library leaves out,
the out-of-scope list of the archived `add-qcl` proposal. That constraint is
absolute and makes the site citable.

**Four questions carry the site.** The crate decides four things, and the
landing page names them in the hero's sub-line and shows one real verdict for
each in the hero: which causal structure explains correlated qubit errors
(`qcl_crosstalk`), which systematic explains a sensor's offset
(`qcl_gravimeter_systematics`), whether an error-correcting code is sound
(`qcl_geometric_qec`), and what stacking codes costs (`qcl_code_switching`,
`qcl_concatenated_code`, `qcl_distillation_round`). The section "What a verdict
carries" states what all four share: the examined count, the margin, the exact
or numeric path, and refusal by name. The source for every statement about the
crate is `openspec/notes/quantum-site/quantum-essence.md`; for the sensing
section, the two sensing examples, the verifications under
`deep_causality_quantum/verification/sensing/` and their README.

**The sensing runs are simulated, and the pages say so.** The instrument values
come from the papers each example cites; the setup times, the tide, the leak and
the dark background are placeholders that `constants.rs` marks; the plans,
predictions, draw counts and biases are computed; every observation is drawn
from the cause the run names. `/sensing/`, the example pages and
`/boundaries/#sensing` state each of these. The verifications read
published values, digitised figures and regenerated data, and `/sensing/` states
every disagreement a verification finds with its paper, with the check that shows it.

**The reader** is a quantum practitioner with basic familiarity who has never
seen this library. After ten seconds the reader should know what the library
does, what it adds, and why that is useful. Structure, order and emphasis serve
a reader who arrives cold from a link.

**The crosstalk run is simulated, and the page says so.** The candidates'
conditional tables, the costs and each candidate's response to each experiment
are typed into the example's `constants.rs` and `model.rs`; the library computes
every predicted read-out from them; the observation is drawn from the Born
sampler at H1's prediction; the cost unit is arbitrary; and the campaign stops
after E1, because H1 separates. The plan table, the decision figure and
`/boundaries/` state each of these. No figure on the site describes a device.

**No roadmap, and no future work.** The site describes the crate as it is today.
When something is not built, the page says what is not built and stops there.
`/proof/` lists seven targets that carry tests and no LEAN proof; the list states
the present and sets no schedule.

**Only cited papers are listed.** `/proof/` lists the works the code cites, each
with what the crate takes from it. `papers/` holds no paper that neither the code
nor the site cites.

**Coverage is stated, not implied.** Five of the sixteen examples in
`quantum_examples` are quantum in subject and do not import this crate.
`/examples/` groups the list by the question each answers, marks that split on
every row, and each detail page names the crates its example uses.

Numbers on the site come from a command anyone can re-run:

| Figure | Source |
| --- | --- |
| Version 0.4.2, released 2026-09-29 | `deep_causality_quantum/CHANGELOG.md` and `Cargo.toml` |
| 14 proved theorems, 7 deferred targets | the quantum section of `lean/THEOREM_MAP.md` |
| The crosstalk run: plan cost 2 of the 5 all four experiments cost, 99.5 bits (survivor and tightest pair), the campaign stops after one experiment | `cargo run --release -p quantum_examples --example qcl_crosstalk`; 99.5 is n·Bhattacharyya distance of 0.40 and 0.10 over 1024 shots |
| The toric code run: four checks accepted, bound 3 rejects with margin 1.333 | `cargo run --release -p quantum_examples --example qcl_geometric_qec` |
| The stacking runs: gadget residual 0.4619 and bound 3.695; distillation 0.1639, 0.1638 and 0.3277 | `qcl_code_switching`, `qcl_distillation_round`, same command form |
| About 4 ms per run | release binary, process start included, Apple M3 Max: median 4.0 ms over 30 runs, and 3.8 ms over 30 runs on a second measurement |
| MSRV 1.98.0 | `rust-version` in the root `Cargo.toml` |
| The gravimeter run: plan 4,956 s, campaigns of 166 s to 4,953 s, tilt refused at 76.43 against 82.54 µGal, 72.41 µGal with no tide, Coriolis named after 1,447 s with clipping off the list | `cargo run --release -p quantum_examples --example qcl_gravimeter_systematics` |
| The gradiometer run: plan 721.8 s, 340 and 60,265 draws, campaigns of 60, 120 and 721.9 s, biases of −355.7 E (quoted as −356) and +5.5 E | `cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk` |
| The verification: 53 checks over 8 verifications, and each finding quoted on `/sensing/` | `cargo run --release -p deep_causality_quantum --features qcm --example <verification>`, verifications listed in `deep_causality_quantum/verification/sensing/README.md` |
| The field's state: 45 nmi, about 6 h, 2.2 against 14 nmi; tilt below 3.3°; the wavefront's 4.0 of 5.2 µGal | Everitt et al. arXiv:2608.25563 (which prints the endpoint as both 4.0 and 4.1 km, so the site quotes nautical miles); Lellouch and Holynski arXiv:2504.11119; Louchet-Chauvet et al. New J. Phys. 13 065025, Table 1 |
| 19 cited works: 10 from `src/`, 9 by the sensing examples and verifications; 17 of them have a PDF under `papers/` | `deep_causality_quantum/papers/`, and grep over `src/`, `examples/quantum_examples/` and `verification/` |

All sixteen examples were run twice on one machine and printed identical
output, which is what the pages say. A wall-clock figure names the machine
(`MACHINE` in `src/consts.ts`).

Each fact lives in exactly one place, split by shape:

| Content | Home | Why |
| --- | --- | --- |
| The crosstalk run: read-outs, costs, observation, plan | `src/data/crosstalk.ts` | The hero figure, the plan table and the prose read it, so they cannot disagree. |
| Counts: tests, theorems, targets, papers | `src/data/evidence.ts` | One place to update after a release. |
| Checks: question, rejection, when vacuous, backing | `src/data/checks.ts` | Rendered on `/checks/`; each row was read against the crate source. |
| Boundaries | `src/data/boundaries.ts` | The landing page shows six, `/boundaries/` shows all, from one list. The sensing limits are a second list, `sensingBoundaries`, at `/boundaries/#sensing`. |
| The sensing runs and the verification | `src/data/sensing.ts` | `/sensing/`, the landing section and the hero's fourth row read it, so they cannot disagree. |
| Theorems, deferred targets, papers | `src/data/formalization.ts`, `src/data/papers.ts` | `theorems` is generated from `lean/THEOREM_MAP.md`. |
| Worked examples | `src/content/examples/en/*.mdx` | Prose with a walkthrough. Frontmatter carries the facts a listing needs, so index and detail cannot disagree. |

Program output in a page is copied from a run, with each elision marked by an
ellipsis. Regenerate an excerpt with the command the page shows and compare it
line by line.

## Voice and vocabulary

Prose follows `docs/writing_guides/AiStyleguide.md`, `ElementsOfStyle.md` and
`ParamedicEditing.md`. These rules decide most edits.

**The reader** works with quantum devices or codes, knows the field's vocabulary,
has never seen this library, and leaves if the page does not say what it does.

**Lead with the outcome.** A page opens with the answer to the question its title
asks, in one or two sentences, before any mechanism. A section opens with its
finding. Show the result (a figure, a table, real output) before the code that
produced it.

**One word per concept.** Define each term once, where the reader first needs it.

| Say | Meaning | Instead of |
|---|---|---|
| candidate | one proposed causal structure for the system | hypothesis (the type is `Hypothesis`), structural candidate |
| structure | who influences whom, as a directed graph over the parts of the system | causal graph, DAG |
| screen | the candidates that pass every validate check (`Screened`) | filter |
| check | a decision that reports what it measured, its threshold, its margin and how many items it examined | test (a unit test is a test), gate |
| experiment | an intervention or a measurement, with a cost and a predicted read-out under each candidate | probe (the builder method is `probes`) |
| plan | the cheapest set of experiments that separates each coverable pair of candidates | design (the stage is `design`) |
| read-out | the number a measurement returns | outcome, result |
| survivor | the candidate the observation leaves standing | winner |
| bits | the separation between two candidates' predicted read-outs at an experiment's shot count | information, entropy |

**Never call a check a gate.** A gate is a quantum gate in this crate, and the
CFD site's use of the word for a pass-or-fail check does not carry over.

Internal terms stay off the landing page and the first paragraph of any page:
Choi–Jamiołkowski, orthomodular, Kraus, causaloid, monad. Where one is needed,
say the plain thing first and give the term in parentheses. The linter's
allowlist names the pages that define each one.

**Sentences.** Prefer concrete nouns and active verbs. Vary length: a four-word
sentence after a thirty-word one earns its emphasis. Put the new or heavy word
last. No more than one em dash per 250 words. Never open a paragraph with
*Additionally*, *Furthermore* or *Moreover*. Cut hedges, *very*, *really*, and
the phrases *delve into*, *shed light on*, *game-changer*, *unlock*, *seamless*,
*robust*, *powerful*, *leverage* and *not only … but also*. State a fact; do not
argue against a rival, and make no uniqueness claims. Do not begin a sentence
with a numeral.

**Checking.** `pnpm check:prose` builds the site and runs `scripts/sitecheck.py`
over every page. It lints the text a reader sees: the content of `<main>`, plus
each page's `<title>` and meta description. Errors fail the run (exit 1): a banned
phrase, *very* or *really*, a paragraph that opens with a transition word, a
duplicate id, a broken link or anchor, a redirect whose target does not exist,
and a page that yields no prose at all. Warnings print and fail only under
`--strict`: jargon outside the pages that define it, more than one em dash per 250
words, "not A, but B", a sentence over 45 words, uniform sentence length, words
glued across markup. The site passes `--strict` with no warnings. Redirect stubs
are not linted for prose. Code blocks and `aria-hidden` figure text are not linted.

**Whitespace in `.astro` templates.** `compressHTML` is off in `astro.config.mjs`.
With it on, Astro drops the line break between a line of text and a following
inline element or `{expression}`, and words join ("over1,024 shots"). Leave it off.

## Pages

| Route | Answers |
| --- | --- |
| `/` | What does it do? Ten sections: the hero, what a verdict carries, which cause, the plan and the decision, the program, which systematic, is the code sound, what does stacking cost, limits, run it |
| `/how-it-works/` | What are the stages of a QCL program, what does a check report, what does an abstraction add? |
| `/sensing/` | What caused a gravimeter's reading error? Nine sections in the home page's form: the hero, why name the cause, which change, the decision, where it ends, a second instrument, checked against papers, limits, run it. Story and sentences: `openspec/notes/quantum-site/04_sensing_page_draft.md` |
| `/checks/` | What does each check decide, and what does a failure name? |
| `/examples/` | Sixteen runnable examples, grouped by the question each answers |
| `/proof/` | What is proved in Lean, what is tested, what is stated and not proved, which papers does it implement? |
| `/boundaries/` | What does it not do, what do the examples assume, where do the sensing examples stop, how do the two senses of quantum stay apart? |
| `/start/` | How do I run it, add it, and choose features? |

The layer-by-layer pages of the earlier site redirect: `redirects` in
`astro.config.mjs` maps `/qcm/`, `/operators/`, `/verdicts/`, `/gates/`,
`/modalities/`, `/formalization/`, `/papers/` and `/errors/` to the page that now
holds their content. The pages themselves are in `superseded/`, moved with
`git mv` and excluded from the build. `superseded/README.md` records what replaced
each one.

## Figures

The figures are HTML and CSS with small inline SVG glyphs, in the site's
instrument vocabulary: hairline strokes, accent dots, no gradients, no raster.

- **Geometry in CSS, words in HTML.** Every label is text a reader can select and
  a screen reader can read. `DecisionFigure.astro` sets the convention.
- **Each row draws its own segment of a shared band.** Rows have no gaps, so on a
  wide screen the segments join, and on a narrow screen, where rows stack, each
  track still reads on its own.
- **Tables collapse into labelled cards on narrow screens.** Put `.table-stack` on
  the table and `data-label` on every `td`. `PlanTable.astro` overrides the shared
  rule to keep the three predictions on one row.
- **Animate through the shared contract.** Put `data-rise` on the figure and
  `.rise` with a `--i` index on the parts that enter in turn. The site-wide
  observer adds `.in-view`, and a `<noscript>` style makes the parts visible
  without it.
- **Data comes from `src/data/`.** No figure hard-codes a number.

The Open Graph card at `public/img/social-share.jpg` is a miniature of the hero
figure. Its source is committed beside it as `social-share.source.svg`;
re-render with `rsvg-convert -w 1200 -h 630`, then convert to JPEG. The card uses
system faces rather than the vendored woff2 files, which rsvg cannot embed.

## Logo

`public/img/deepcausality-quantum-on-{dark,light}.svg` are copies of
`img/project-logos/quantum/` at the repository root. Each variant is drawn in its
own theme's tokens, dark carrying
`#5cd4e1` on `#e6edf3` and light carrying `#0a8a98` on `#0b1118`. The header
ships both and shows one, switched on `[data-theme]` the same way `ThemeToggle`
swaps its glyphs.

The lockup is also inlined verbatim into `social-share.source.svg`, because rsvg
cannot reliably resolve an external SVG reference and the card has to stay a
single self-contained file. If the logo is redrawn, re-copy both variants and
re-inline the dark one into the card.

## Toolchain note

`astro check` requires **TypeScript 6.x**. TypeScript 7.0 dropped the
programmatic API the checker uses (withastro/roadmap#1321), so `typescript` is
pinned to `^6.0.3` across all four sites. Do not let a routine upgrade move it
to 7.x.

`@astrojs/markdown-satteri` is also pinned, in `pnpm-workspace.yaml`, so pnpm
resolves a single copy. See
[`../README.md`](../README.md) for both constraints.

`shiki` is pinned in the same file. `shiki-rust-themes.mjs` derives its themes
from the `bundledThemes` of this project's own copy, while astro highlights with
the copy its `^4.0.2` dependency resolves; a single override keeps those the
same shiki.

pnpm 11 does not read the `pnpm` field from `package.json`, so
`overrides` and `onlyBuiltDependencies` must live in `pnpm-workspace.yaml`.

## Deliberate omissions

- **No Pagefind.** The project website ships an unread search index on every
  deploy (DESIGN.md §8.9); this site does not.
- **No mermaid.** The figures are HTML and CSS, which keeps the heaviest
  dependency off every route.
- **No client islands.** Zero framework runtime; interactivity is three small
  module scripts (the observer, the mobile sheet, the theme toggle).
- **No API reference.** docs.rs builds it from the source with all features, so a
  copy here would age. The earlier site's inventory had no entry for the QCL
  builder.

## License

All software source code is licensed under the
[MIT License](https://opensource.org/license/mit/).

All documentation is distributed under the
[Creative Commons Attribution 4.0 International Licence](https://creativecommons.org/licenses/by/4.0/).
