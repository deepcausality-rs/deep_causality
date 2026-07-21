## MODIFIED Requirements

### Requirement: Informed and uninformed worlds fly the same measured day

The belief counterfactual SHALL fly two whole-world alternations from one baseline against the same
measured cold atmosphere: an **informed** world whose guidance interpolated the recorded dispersion
table at the measured temperature departure, and an **uninformed** world that assumed the
standard-day row. The two worlds MUST differ only in the belief the guidance holds — the flown
atmosphere is the same measured day in both — so any separation is attributable to the table rather
than to the weather.

**The `!!ContextAlternation!!` marker requirement is withdrawn for this counterfactual**, on a
measured architectural constraint rather than by preference. That marker is written by the carrier's
fork path, which alternates a **world config**. The belief varies a guidance parameter that lives in
the **coupling**, and a coupling stack is fixed per march call — the same constraint the design note
pins in §5. The two beliefs are therefore two march calls resuming one shared baseline state, which
delivers the substance the marker was asked to evidence (one baseline, one differing input) without
the marker itself. Publishing the margin as a world constant purely to earn a marker was considered
and rejected: it would contort the guidance seam to satisfy an audit token.

#### Scenario: Only the belief differs

- **WHEN** the informed and uninformed worlds are constructed for a measured cold day
- **THEN** both carry the same atmosphere and resume the same baseline state, and differ only in the
  dispersion row their guidance consumed

### Requirement: The table earns its place through a material separation

Gate **(5) table earns its place** SHALL require the two worlds to separate materially on a **flown
outcome**, beyond a band earned on the first measured run. A separation that does not clear the band
is a reported finding about the table's value on that day, not a gate to relax.

**The separation MUST be measured on the flight, not on the table.** Subtracting two interpolations of
one recorded table produces a number that is invariant to the entire descent — it is a property of the
checked-in file and two constants, computed before any march, and it cannot establish that consuming
the table changed anything. The gate MUST read a quantity the two worlds flew: the altitude each lit
its landing burn at, the propellant each spent, or the condition each arrived at.

**Where the margin binds is itself a finding to respect.** The margin reaches the flight through the
stopping-burn ignition altitude, which adds it to the stopping distance — a guidance that believes the
day is more dispersed lights higher. It does **not** bind at the ignition commit, where the navigated
uncertainty sits two orders of magnitude inside either belief's margin and both worlds commit
identically. A gate reading the commit would report the two worlds as indistinguishable while the
flight difference is real elsewhere.

#### Scenario: The two worlds separate on a flown quantity

- **WHEN** both worlds fly the measured cold day from one baseline and are scored
- **THEN** the gate reports the difference in a flown outcome — landing-burn ignition altitude,
  propellant spent, or arrival condition — and requires it to clear the pinned band

#### Scenario: A non-separation is reported, not hidden

- **WHEN** the two worlds do not separate beyond the band
- **THEN** the gate fails with a detail line reporting the measured separation, and the finding is
  recorded rather than the band loosened

## ADDED Requirements

### Requirement: A table clamp is stamped into provenance and gated

An interpolation that clamped to the nearest tabulated row SHALL be recorded in the provenance log and
read by a gate. The table type is pure by design and leaves the stamping to the flight side; the
flight side decorating a console line instead leaves an extrapolated row invisible to every gate and
to the recorded run.

A clamped row means the day flown is outside the tabulated range, so the belief is an extrapolation
rather than an interpolation. The belief gate MUST fail on it, whatever separation it measures.

#### Scenario: A measured departure outside the table is visible in the run

- **WHEN** the measured temperature departure falls outside the tabulated range
- **THEN** the provenance log carries the clamp with the departure and the row it clamped to

#### Scenario: A clamped belief fails the gate

- **WHEN** the day's row clamped
- **THEN** the belief gate fails and names the clamp, even where the two worlds separated

### Requirement: The interpolated row is the single source for the day's belief

Every quantity the day's belief describes SHALL be read from the interpolated row, and no quantity the
belief prints may be duplicated as a hand-set constant. A printed value the coupling does not receive
MUST be removed from the output or passed to the coupling.

Two sources of truth for one physical quantity is how a printed belief and a flown atmosphere drift
apart while a rounded display format hides the difference.

#### Scenario: The flown atmosphere matches the printed belief

- **WHEN** the day's density scale is printed
- **THEN** it is the value the flown atmosphere was built from

#### Scenario: A printed sensor departure is flown

- **WHEN** the accelerometer bias departure is printed as part of the day flown
- **THEN** the coupling's inertial model was constructed with that departure

### Requirement: The ignition margin is sized so it can bind

The margin the commit demands SHALL be sized against the navigated uncertainty the flight actually
achieves, or the run MUST record that it does not bind. A margin two orders of magnitude above the
navigated sigma admits every commit under either belief, so the two worlds commit identically and a
gate reading the commit measures nothing.

#### Scenario: The binding point is stated

- **WHEN** the belief gate reports its verdict
- **THEN** the message names the quantity through which the margin reached the flight, so a reader
  can tell a bound constraint from an inert one
