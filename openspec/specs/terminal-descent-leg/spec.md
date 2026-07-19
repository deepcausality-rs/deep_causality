# terminal-descent-leg Specification

## Purpose
TBD - created by archiving change add-retropulsion-terminal-descent. Update Purpose after archive.
## Requirements
### Requirement: A leg boundary logs what it discarded and re-seeded

The compressible carrier SHALL append one provenance entry at the start of a leg resumed from a
prior march state, naming the world it re-seeded into and recording that the marched conserved state
was re-seeded rather than carried. A leg boundary carries only the coupled field: the conserved
state is re-quantized from the world's uniform seed, and the carrier's inflow, acoustic-envelope
drift, rebuild count, and plume-imprint budget are all reset. Today that boundary writes **no** entry
at all, so the single most consequential event at a leg seam is invisible in provenance, while the
fork path — which genuinely does preserve the marched state — logs its resume. The entry MUST NOT
change the existing rebuild or pause message texts, which downstream gates match on.

#### Scenario: The re-seed is visible in provenance

- **WHEN** a leg resumes from a prior march state
- **THEN** one entry naming the re-seed and the incoming world appears in the log before that leg's
  first step entry

#### Scenario: Existing message texts are unchanged

- **WHEN** a multi-leg descent is run
- **THEN** the rebuild and pause entries carry exactly their existing texts, and gates matching on
  them are unaffected

### Requirement: The rebuild budget gains an explicit bound and a programmatic reader

The compressible carrier SHALL carry a configured numeric rebuild bound, enforce it in-loop, and
expose the rebuild count through an accessor. The carrier is **not** unbounded today: the trigger
`s_needed > s_ref·(1 + rebuild_tol)` combined with the unconditional re-pin `s_ref ← 1.2·s_needed`
is a hysteresis ratchet, so each successive rebuild requires roughly `1.44×` further wave-speed
growth at the default tolerance, and the carrier's "a re-pin gate caps it" documentation describes
that real mechanism. What is missing is a bound anyone can state or check: the counter is written,
formatted into the log entry, and compared against nothing, and no accessor exposes it, so the only
machine-checkable budget in the workspace is an example gate that tallies `"carrier rebuilt at
step"` substrings in a rendered provenance string — an assertion that detects rather than enforces,
and one that breaks silently if the message wording ever changes. Exceeding the explicit bound MUST
return `Err(PhysicalInvariantBroken)`, because a leg needing that many re-pins is not converging on
an envelope and its numbers should not be reported as results. This differs deliberately from the
plume imprint's cap, which stops refreshing without erroring because the imprint is state realism.

#### Scenario: Exceeding the explicit bound refuses

- **WHEN** a leg's acoustic envelope drifts often enough to require one rebuild past the configured
  bound
- **THEN** the carrier logs the exceedance and returns an error

#### Scenario: The rebuild count is readable without parsing the log

- **WHEN** a harness or gate needs the rebuild count
- **THEN** it reads the accessor, rather than counting substrings in a rendered provenance string

#### Scenario: The imprint cap keeps its softer behavior

- **WHEN** the plume-imprint refresh budget is exhausted
- **THEN** the carrier stops refreshing the imprint and continues, as it does today

### Requirement: The rebuild trigger's scope and blind spot are specified

The rebuild mechanism SHALL be documented as wave-speed-keyed and **per-carrier**, not per-descent.
It fires on `s_needed = û + √(γ·t̂)` growing past the tolerance; the nondimensional **density does
not enter the trigger at all**, so a configuration whose density anchor is wrong is never corrected
by it. The acoustic envelope ratchets upward within one carrier instance and resets to the world's
configured value at every leg boundary, because each leg builds a fresh carrier — so the trigger is
re-armed at the baseline on each new leg rather than inheriting the previous leg's earned envelope.
A terminal leg MUST NOT rely on rebuild-on-drift to rescue an ill-anchored configuration: the
mechanism corrects an undersized *wave-speed* envelope within a leg and nothing else. It is not
true that the trigger cannot fire while decelerating — the post-shock temperature enters `s_needed`,
and the atmosphere warms from roughly 217 K in the 15–20 km band to 288 K at sea level, so `t̂` can
rise as the vehicle descends.

#### Scenario: A density-anchor error is not corrected by a rebuild

- **WHEN** a leg's nondimensional density runs far from unity because its reference scales are
  anchored to another regime
- **THEN** no rebuild fires on that account, because density does not enter the trigger

#### Scenario: The envelope resets at a leg boundary

- **WHEN** a leg that earned a larger acoustic envelope hands off to the next leg
- **THEN** the new leg starts from the world's configured value with the trigger re-armed at that
  baseline

### Requirement: The terminal leg is configured for subsonic flight, not inherited from reentry

A terminal descent leg SHALL be configured with its own reference scales, acoustic reference, and
seed, and MUST NOT inherit the corridor's. The carrier keeps **two** distinct gammas — the
schedule's effective gamma, which is used only to construct the Rankine–Hugoniot jump, and the
marcher's gamma, which appears in the energy relation and the rebuild criterion — and a terminal leg
SHALL set both for cool low-Mach air rather than for a reacting shock. The reference scales are
fixed per configuration and the corridor's are anchored near a 90 km, Mach-24 post-shock state, so a
terminal leg re-seeded from them starts its fluid layer at reentry conditions at low altitude and
carries a nondimensional density orders of magnitude away from unity.

#### Scenario: Both gammas are set for the terminal regime

- **WHEN** a terminal leg is configured
- **THEN** its schedule gamma and its marcher gamma are each set for cool low-Mach air, not
  inherited from the reacting-shock recipe

#### Scenario: The terminal seed is not the reentry seed

- **WHEN** a terminal leg re-seeds its fluid layer
- **THEN** it seeds from a terminal-regime state rather than from the corridor's post-shock values

### Requirement: The terminal leg reaches touchdown inside bounded witnesses

A terminal descent SHALL reach the configured altitude floor with the descent rate, the miss
distance to the aim point, and the remaining propellant each inside its pinned bound, and the
transonic crossing under thrust MUST appear in the provenance log as a regime transition. The
transonic passage degrades gracefully by construction — the carrier gates its Rankine–Hugoniot inflow
jump above a Mach threshold and enforces the raw freestream below it — and the crossing MUST be
logged rather than passing silently, since a terminal leg crosses it by design.

#### Scenario: Touchdown witnesses are bounded

- **WHEN** the terminal leg reaches the altitude floor
- **THEN** its descent rate, miss distance, and remaining propellant are each inside their pinned
  bounds

#### Scenario: The transonic crossing is logged

- **WHEN** the flight Mach crosses the threshold under thrust
- **THEN** a regime transition naming the new Mach regime and the thrust state appears in provenance

