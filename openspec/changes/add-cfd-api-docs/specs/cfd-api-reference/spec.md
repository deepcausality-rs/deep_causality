## ADDED Requirements

### Requirement: A published API reference covers the crate's full public surface

The project SHALL publish an API reference for `deep_causality_cfd` that covers every name the crate
exports, and SHALL generate the signature-level half of it from the code rather than transcribing it.

The crate is `publish = false`, so no docs.rs page exists for it; the reference is therefore published
with the CFD website. The generated half SHALL be produced by `cargo doc` over the crate, so a name's
signature, generic bounds, and feature gates cannot disagree with the code that defines them.

Coverage SHALL be complete over the names the crate itself defines. Names the crate re-exports from
another crate SHALL be identified as re-exports and linked to their defining crate's reference rather
than duplicated, because the CFD crate does not own their definition.

#### Scenario: Every exported name is reachable

- **WHEN** a reader looks up any name the crate exports
- **THEN** the reference gives its signature, and the signature is the one the code declares

#### Scenario: Re-exported surface is identified, not duplicated

- **WHEN** a reader encounters a type the crate re-exports from another crate
- **THEN** the reference states that it is re-exported for import convenience and links to its
  defining crate, rather than carrying a second copy of its documentation

#### Scenario: The crate's metadata points at its own reference

- **WHEN** a reader follows the crate's `documentation` metadata
- **THEN** it resolves to this crate's reference, not to another crate's page

### Requirement: The reference orients a reader before it enumerates

The reference SHALL provide a curated layer, written in the site's own presentation, that gives a
reader the shape of the API before its contents — because a generated index alone presents several
hundred alphabetised names with no indication of which matter or in what order they are used.

The curated layer SHALL be organized by **surface area** — a coherent piece of the API learned as a
unit — rather than alphabetically or by source module. Each area SHALL state what it is for, name the
entry points a reader starts from, and link the names it introduces into the generated reference.

The curated layer SHALL make the crate's two organizing structures explicit: the single configuration
entry and its per-family methods, and the phase order the workflow and study grammars enforce in the
type system.

The curated layer SHALL NOT publish counts of names per area. A count is a number that goes stale on
every refactor while carrying no information a reader acts on, and it would add a maintenance
liability to the layer whose whole purpose is to stay small enough to keep true.

#### Scenario: A newcomer sees the whole API's shape

- **WHEN** a reader opens the reference index
- **THEN** every surface area is listed with what it covers, on one screen

#### Scenario: The curated layer carries no stale-able tallies

- **WHEN** the reference is read for figures that would have to be revised after a refactor
- **THEN** it states no per-area name counts, and coverage is established by the check that compares
  the documented areas against the crate's public surface rather than by a published number

#### Scenario: An area names its entry points

- **WHEN** a reader opens any surface area page
- **THEN** it names the entry points that area is used through, so the reader knows which of its
  names to reach for first

#### Scenario: Phase order is documented as a type-level guarantee

- **WHEN** a reader consults the workflow or study grammar area
- **THEN** the phase order is stated together with the fact that a mis-ordered program is a compile
  error, not a runtime failure

### Requirement: The curated layer is built from the site's existing design system

The curated layer SHALL conform to `website/web/DESIGN.md`, which `website/cfd/src/styles/global.css`
declares binding on this site, and SHALL reuse the shared page shell and the §12 conventions that
stylesheet already declares once — `.eyebrow`, `.eyebrow-coord`, `.reticle` / `.reticle-host`,
`.panel`, `.corner-brackets`, `.chip`, `.hairline-list`, and the single focus ring.

A convention SHALL NOT be redeclared locally by a reference page (§13.18). The CFD site's stated
advantage over the marketing site is that each convention exists once; a new section that reimplements
one forfeits it. The reference SHALL likewise introduce no new page shell where the existing detail
shell serves, and SHALL respect the §13 anti-patterns.

Conformance is not satisfied by correct tokens alone. The site's character comes from a small number
of repeated visual moves, so a reference page SHALL use the idioms and not merely the token values —
a page that uses every token correctly and none of the idioms reads as foreign.

The shared token file SHALL remain a byte-identical mirror of the marketing site's, as
`pnpm check:tokens` enforces. Any token the reference needs beyond the shared set SHALL be declared in
the site-local token extension instead.

#### Scenario: A reference page reads as native

- **WHEN** a reader moves from a blueprint or tutorial page to a reference page
- **THEN** the page uses the same shell, eyebrow, panel, and corner idioms, and reads as part of the
  same site rather than a bolted-on section

#### Scenario: No convention is redeclared

- **WHEN** the reference's styles are inspected for the §12 conventions
- **THEN** each is used from the shared declaration, and none is reimplemented locally

#### Scenario: The token mirror stays verifiable

- **WHEN** `pnpm check:tokens` runs after the reference is added
- **THEN** it reports the shared token file in sync, any site-local token having gone to the
  site-local extension

### Requirement: Generated output is presented as a distinct destination

The generated reference SHALL be presented as what it is — machine-generated API output — and SHALL
NOT be restyled to imitate the site's design system.

Generated documentation cannot conform to `DESIGN.md`: its markup, navigation, and type scale are
fixed by the generator. Styling it to look native would produce a surface that resembles the site
while behaving differently, which misleads the reader about where they are; leaving it plain and
labelling the transition is honest and costs nothing. The link into it SHALL therefore signal that
the reader is leaving the curated pages for generated output.

#### Scenario: A reader knows which surface they are on

- **WHEN** a reader follows a link from a curated page into the generated reference
- **THEN** the link identifies the destination as the generated API reference, and the reader is not
  presented with a near-copy of the site's design that behaves unlike it

### Requirement: Curated examples are taken from executed code

A code example in the curated layer SHALL be taken from a committed example, verification program, or
test that CI executes, rather than written for the page.

An example written for documentation alone is unverified: it can fail to compile, or compile and
describe a usage the crate does not support, with nothing to detect either. An excerpt from an
executed program carries the guarantee of whatever gate runs it.

#### Scenario: An example on the page is an example that runs

- **WHEN** a code example in the curated layer is traced to its origin
- **THEN** it is an excerpt of a committed program that CI builds or runs, and the page cites that
  program

### Requirement: The reference is versioned with the code it describes

The generated reference SHALL be produced from the same commit as the site that publishes it, and
SHALL NOT be committed to the repository as generated output.

Committing generated documentation creates a second thing that can be stale — a checked-in build
older than the code — and puts a large churning artifact into review diffs.

#### Scenario: The reference matches the deployed commit

- **WHEN** the site is deployed
- **THEN** the generated reference published with it was produced from that deployment's commit

#### Scenario: Generated output is not tracked

- **WHEN** the repository is inspected for the generated reference output
- **THEN** it is absent and ignored, being produced at build time
