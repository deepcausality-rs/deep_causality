## ADDED Requirements

### Requirement: The descent runs a numbered gate set evaluated in full

The example SHALL gate the descent with the numbered set (0) integrity, (1) corridor inheritance,
(2) ignition corridor, (3) regime cascade, (4a) flow spread, (4b) sign-flip found, (4c) coupling
load-bearing, (4d) fork economics, (4e) audit trail, (5) table earns its place, (6) touchdown,
(7) compression, (8) bounded rebuilds, and (9) wall-clock budget, expressed through the study
grammar's gate sequences and rendered as one merged verdict. **Every gate MUST be evaluated before
the verdict decides** — the sequence-based path evaluates all gates and collects their outcomes,
unlike the verification binaries' short-circuiting conjunction, which suppresses exactly the later
FAIL lines a reader needs when diagnosing. Each gate MUST carry a human-readable detail line
carrying its measured numbers, since the outcome itself is a bare boolean.

#### Scenario: A failing early gate does not hide the later ones

- **WHEN** gate (1) fails on a run
- **THEN** gates (2) through (9) are still evaluated and their PASS/FAIL lines still render

#### Scenario: The verdict merges the campaign and trajectory sequences

- **WHEN** the study verdict and the per-act trajectory verdict are combined
- **THEN** one merged verdict renders every outcome and the example's exit code follows its
  aggregate pass state

### Requirement: Gate thresholds reach their gate without being captured

Every gated band SHALL reach its gate through one of the two paths the gate type permits: a `const`
in the example's `constants.rs`, or a field on the `Row` type the view carries. The gate type is a
plain function pointer, so a gate **cannot capture** a threshold, a reference value, or a config
object from its environment — a band held anywhere else is unreachable from the gate that enforces
it. Pinned bands SHOULD take the `const` form, each carrying a doc comment naming the model or
measurement it came from, so a re-pin is a single documented edit; a band that is only known at
runtime — the elapsed wall clock, a baseline drawn from the study's own prior round — MUST instead
ride the `Row` type into the view.

#### Scenario: A pinned band is changed in one place

- **WHEN** a pinned band is re-tuned
- **THEN** the edit is to a single documented `const` in `constants.rs`, and the gate function that
  reads it is unchanged

#### Scenario: A runtime band reaches its gate through the row

- **WHEN** a gate must compare against a value that only exists at run time
- **THEN** that value rides the row type into the view rather than being captured by the gate

### Requirement: Bands are earned on the first measured run, then regressed

Every band marked as pinned SHALL be earned from the first measured run and MUST NOT be invented
ahead of it. Once earned, bands gate regressions. A band MUST NOT be re-pinned without recording the
re-pin and its reason, and a gate that cannot pass because the physics does not produce the effect
it asserts MUST be converted to a reported finding with the conversion recorded — a permanently
failing gate regresses nothing.

#### Scenario: A first run pins rather than gates

- **WHEN** the example runs for the first time and a band has no recorded value
- **THEN** the measured value is recorded as the pin and the reason is documented

#### Scenario: A re-pin is recorded

- **WHEN** an earned band is changed
- **THEN** the change, its measured basis, and its reason are recorded rather than silently edited

### Requirement: Acts 0 and 1 reproduce the corridor bit-identically

Gate **(1) corridor inheritance** SHALL require that, with the full burn stack composed but the
commanded throttle at zero, Acts 0 and 1 reproduce the corridor's blackout window, RAM-C II anchor
band, and drift and reacquisition witnesses bit-for-bit against the corridor example's recorded
output. This holds on two preconditions the example MUST maintain: the burn stages are strictly
inert at zero throttle, and the atmosphere rows below the corridor's sampled altitudes are appended
data only. The corridor and weather examples themselves MUST re-run bit-identically after this
change.

#### Scenario: The inert burn stack changes nothing upstream

- **WHEN** Acts 0 and 1 fly with the propulsion stages composed and the throttle at zero
- **THEN** their witnesses match the corridor's recorded output bit-for-bit

#### Scenario: The sibling examples are untouched

- **WHEN** the corridor and weather examples are re-run after this change
- **THEN** each reproduces its committed output

### Requirement: Wall-clock is gated at the caller and compression at the report

Gate **(9) wall-clock budget** SHALL be evaluated at the caller against a budget sized for the
flight actually flown, because the study grammar cannot observe the wall clock — it times the whole
program, which the study is only part of. The budget is **1800 s**, not the corridor's 600 s: that
figure was sized for a descent that stops at 47 km after roughly 476 coupled steps, and this example
carries the same vehicle to the ground under a burn and forks a roster mid-way, which is several
thousand steps more. A budget inherited from a shorter flight measures nothing about this one. Gate **(7) compression** SHALL read the committed branch's final evolved state from
its report and re-quantize it under the run's truncation to observe the peak bond against the cap,
and gate **(8) bounded rebuilds** SHALL count the carrier's rebuild entries in the rendered
provenance log against the cap. A witness that cannot be computed MUST fail its gate rather than
pass by default.

#### Scenario: The whole program is timed

- **WHEN** the example finishes
- **THEN** the elapsed wall clock measured across the whole program is gated against the budget and
  rendered with its measured value

#### Scenario: An uncomputable witness fails

- **WHEN** the compression witness cannot be formed from the report
- **THEN** gate (7) fails rather than passing on a missing value

### Requirement: The example ships a passing capture and a family README

The example SHALL ship an `output.txt` that is a verbatim capture of a real passing release run, and
a `README.md` in the family form carrying the SPDX comment header, a `## How to Run` section, the
narrative of the acts, the validation anchors, the limitations pointing at `constants.rs`, and a
`## Where Things Live` table. The README MUST state the M1 AMBER finding plainly — that the in-flight
drag decrement comes from the cited correlation and that the marched imprint is state realism — and
MUST NOT present the state fork as a field-contracted drag measurement. Cross-links between the
three family siblings and the crate's examples table MUST be swept so every reference resolves.

#### Scenario: The capture matches a real run

- **WHEN** the example is run in release and its console output compared to `output.txt`
- **THEN** they agree apart from the wall-clock line

#### Scenario: The README states the measured limitation

- **WHEN** a reader reaches the README's description of the counterfactual centerpiece
- **THEN** it names the A0 correlation as the drag authority and the imprint as state realism
