## ADDED Requirements

### Requirement: Informed and uninformed worlds fly the same measured day

The belief counterfactual SHALL fly two whole-world alternations from one baseline against the same
measured cold atmosphere: an **informed** world whose guidance interpolated the recorded dispersion
table at the measured temperature departure, and an **uninformed** world that assumed the
standard-day row. Both MUST be `!!ContextAlternation!!`-marked like every other counterfactual in
the family, naming the baseline they alternate from. The two worlds MUST differ only in the belief
the guidance holds — the flown atmosphere is the same measured day in both — so any separation is
attributable to the table rather than to the weather.

#### Scenario: Only the belief differs

- **WHEN** the informed and uninformed worlds are constructed for a measured cold day
- **THEN** both carry the same atmosphere and differ only in the dispersion row their guidance
  consumed, and both carry the alternation marker

### Requirement: The interpolated row is a load-bearing flight input

The interpolated dispersion row SHALL size real flight quantities rather than being reported for
interest: the ignition-commit margin MUST be `drift_mean + k·drift_sd` from the row, and the
propellant reserve above the envelope floor MUST be sized from the same dispersion. The chosen row,
the measured temperature departure, and any clamp MUST be stamped into the flight provenance log. A
temperature departure outside the tabulated range MUST clamp to the nearest row with the clamp
stamped, and MUST NOT silently extrapolate.

#### Scenario: The margin comes from the table

- **WHEN** the ignition commit gate evaluates the navigated state
- **THEN** the margin it demands is the interpolated row's `drift_mean + k·drift_sd`, and the row and
  departure appear in provenance

#### Scenario: An out-of-range departure clamps and says so

- **WHEN** the measured departure lies outside the tabulated range
- **THEN** the nearest row is used and a clamp entry is stamped into the log

### Requirement: The loader binds the recorded table through the typed reader

The example SHALL load `cfd/plasma_blackout/weather/weather_table.csv` through the existing typed
reader against the recorded schema, feed the parsed rows into the reusable `KeyedTable`, and select
the bracketing pair **by value** — never by file order, because the recorded rows arrive in run
order rather than temperature order. A missing required column MUST surface as the reader's
named-column error and a malformed cell as a loading error, never a default value. This closes the
binding that the `weather-table-consumption` capability specifies and defers to this milestone, and
the path MUST be built from the manifest directory as the sibling examples do.

#### Scenario: Bracketing is by temperature, not by row order

- **WHEN** a departure is interpolated against the recorded table
- **THEN** the two rows bracketing it in temperature are selected, regardless of their positions in
  the file

#### Scenario: A missing column is a named error

- **WHEN** the loader reads a table missing a required column
- **THEN** the reader returns an error naming that column and no partial result is produced

### Requirement: The table earns its place through a material separation

Gate **(5) table earns its place** SHALL require the uninformed world to separate materially from
the informed one on the measured day — measurably breaching its ignition margin or landing worse —
beyond a band earned on the first measured run. A separation that does not clear the band is a
reported finding about the table's value on that day, not a gate to relax: the gate exists to say
whether consuming the table changed the flight, and an honest negative answer is a result.

#### Scenario: The uninformed world lands worse

- **WHEN** both worlds fly the measured cold day and are scored
- **THEN** the uninformed world's margin breach or terminal outcome separates from the informed
  world's beyond the pinned band

#### Scenario: A non-separation is reported, not hidden

- **WHEN** the two worlds do not separate beyond the band
- **THEN** the gate fails with a detail line reporting the measured separation, and the finding is
  recorded rather than the band widened
