## ADDED Requirements

### Requirement: One coordinate spacetime serves both sign conventions

The spacetime family SHALL carry one Cartesian coordinate spacetime rather than two that differ
only in a sign convention, and the surviving name SHALL be the Lorentzian one.

The two types are field-identical, each holding an id, three spatial coordinates, a time
coordinate and a time scale. The sole difference between them is the metric signature, which the
type's name carried and which the frame now supplies as a constant.

Lorentzian is the name that survives because it names the signature class, where Minkowski names
one flat member of that class. A merge keeps the general name and lets the frame's constant say
which convention a given world uses.

#### Scenario: The merged type serves a mostly-plus frame

- **WHEN** a frame declaring the mostly-plus signature names the coordinate spacetime
- **THEN** it compiles and the signature comes from the frame

#### Scenario: The merged type serves a mostly-minus frame

- **WHEN** a frame declaring the mostly-minus signature names the same coordinate spacetime
- **THEN** it compiles, and the two frames differ only in the constant they declare

#### Scenario: The narrower name is gone from the variant enum

- **WHEN** the spacetime variant enum is read
- **THEN** it carries no arm for the narrower of the two merged names

### Requirement: Orientation is not a space, and leaves the spatial family

The crate SHALL NOT carry a quaternion spatial type, and the spatial variant enum SHALL carry no
arm for one.

A quaternion is a rotation operator acting on a space rather than a space of its own. An
orientation sits beside a position in a pose; a context locates things, and a rotation locates
none, so it was misplaced in the spatial family rather than merely over-implemented.

Removing only the spatial trait is insufficient. The variant enum is the ordinary frame member
under `context-frame`, so an arm carrying the type leaves it reachable as a space exactly where
most frames will meet it.

An orientation belongs in a deployment's own vocabulary, attached to a position rather than
standing beside one.

#### Scenario: No frame can name an orientation as its space

- **WHEN** a frame is written naming a quaternion type as its spatial member
- **THEN** it does not compile, because the crate declares no such type

#### Scenario: The variant enum has no orientation arm

- **WHEN** the spatial variant enum is matched exhaustively
- **THEN** no arm carries an orientation, so a frame over the variant enum cannot reach one

### Requirement: A geodetic altitude states the datum it is measured from

The geodetic spatial type SHALL carry the vertical datum its altitude is measured from, as a field
beside the coordinate rather than as a statement in documentation.

An altitude is a number against a reference surface, and the surfaces differ by tens of metres. The
same number means different heights against an ellipsoid, a geoid model and a barometric standard
atmosphere, so a type that carries the number without the reference carries an ambiguity that reads
as a fact. The type records the reference today in a doc comment on the field, which fixes one
datum for every value the type will ever hold.

The datum is a field rather than a constant of the frame. A frame-level constant would be the
better model of what a datum is — it is a property of the reference frame, like the metric
signature — but it cannot vary, and two positions differing only in datum are two positions in one
context, not two contexts.

#### Scenario: Two altitudes against different references are distinguishable

- **WHEN** two geodetic positions hold the same numeric altitude against different vertical datums
- **THEN** the two values are distinguishable from the positions alone, without external knowledge
  of which datum was intended

#### Scenario: The datum survives a round trip

- **WHEN** a geodetic position is constructed with a datum, stored by a consumer and rebuilt
- **THEN** the datum is recoverable, so the altitude means on the way out what it meant on the way
  in

### Requirement: The spatial and spacetime families are not in correspondence, and this is recorded

The crate SHALL document that its spatial types have no matching spacetime for every member, so
that a frame whose space is geodetic or navigation-local has no spacetime to name.

The spacetime types are Cartesian. A geodetic or navigation-local frame must convert to a Cartesian
spatial type before it can name a spacetime. Recording the restriction is what this requirement
asks for; adding the missing spacetime types is a separate modelling decision.

The geodetic type is the furthest from correspondence: beyond having no spacetime counterpart, it
carries a datum the Cartesian types do not, so a conversion out of it discards a reference the
others never held.

#### Scenario: The restriction is discoverable from the documentation

- **WHEN** a reader looks for which spatial types can be paired with a spacetime
- **THEN** the crate states which pairings exist and which require a conversion first
