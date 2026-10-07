# compressible-qtt-validation Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: Sod shock-tube verification

`deep_causality_cfd/verification/` SHALL gain a self-verifying Sod shock-tube example that gates the
`compressible-qtt-flux` against the **exact Riemann solution** (density, velocity, pressure; shock / contact /
expansion speeds), exiting non-zero on break, in the established house style.

#### Scenario: Sod matches exact Riemann
- **WHEN** the Sod example is run
- **THEN** the profiles match the exact Riemann solution within the recorded tolerance and the run exits zero

### Requirement: RAM-C stagnation-line verification (the buildable milestone)

A self-verifying **RAM-C stagnation-line** example SHALL march a 1-D fitted normal shock at the cited
RAM-C II 71 km freestream (Mach 25.9, T∞ = 217.9 K, dynamic pressure 2.28 kPa; Parent, Thoguluva
Rajendran & Omprakas, arXiv:2111.09432), with the velocity and number density derived from those three
values and never stated independently of them. It SHALL apply the exact Rankine–Hugoniot post-shock
state, run the **reused Tier-A reacting/ionization LER stack** in the post-shock relaxation zone, and
gate the peak **electron density** against the RAM-C II station-1 Ka-band datum: Grantham (1970,
p. 18) records the Ka-band critical-density crossing at 71.93 km, and Grantham's relation
N_e,pk = 0.63 × 1.287e-8 f² cm⁻³, with f in Hz, at f = 3.5e10 Hz (35 GHz) gives 9.93e12 cm⁻³ =
9.93e18 m⁻³. The harness SHALL state the 0.93 km between
the 71 km condition and the 71.93 km crossing. It SHALL also run the cited RAM-C II 61 km freestream
(Mach 23.9, T∞ = 255.9 K, dynamic pressure 8 kPa) and gate, as a `[reference]` bound with no allowance,
that the network's peak `n_e` there is at least the anchor value, because the flight's station 1 is
overdense at Ka-band below 71.93 km.

The network's ±0.70-decade allowance SHALL be a chemistry-model spread, independent of `μ_sr`, and
its gate SHALL be a `[tripwire]`; the Park-2T controller's regression pin SHALL be re-derived from the
corrected vibrational-relaxation closure at the cited freestream. Neither SHALL be widened to re-admit
a prediction the previous, incorrect `μ_sr` produced, nor to re-admit the agreement the previous,
hybrid freestream produced. The bands in force before this change were earned
under `μ_sr = 7.0`, a value with no valid collision pair, and later at a freestream that paired a 72 km
density with a 61 km temperature; a band restored to keep the former headline agreement would assert
an accuracy the physics does not support.

Because `μ` sits inside the Millikan–White exponential, the correction moves the prediction
materially, and the corrected freestream lowers the post-shock temperature further. If the corrected
prediction no longer supports an order-of-magnitude claim against the RAM-C II anchor, the harness
SHALL report that outcome rather than presenting a re-tuned band as agreement, and the gate's evidence
class SHALL reflect what the bound actually encodes.

#### Scenario: RAM-C peak electron density reproduced
- **WHEN** the stagnation-line example is run at the cited RAM-C II 71 km freestream
- **THEN** the peak electron density is compared with the station-1 Ka-band datum within the recorded
  tolerance, with the fitted shock at `O(1)` rank

#### Scenario: The 61 km station meets the flight's lower bound
- **WHEN** the stagnation-line example runs the cited 61 km freestream
- **THEN** the network's peak `n_e` is at least the station-1 Ka-band anchor value, and the gate is
  labelled `[reference]`

#### Scenario: The flight condition is derived, not stated
- **WHEN** the harness configuration is inspected
- **THEN** it states Mach number, freestream temperature and dynamic pressure with their citation, and
  computes velocity and number density from them

#### Scenario: The acceptance band is traceable to the corrected physics
- **WHEN** the recorded tolerance is inspected
- **THEN** the network allowance states its evidence class and the closure and freestream it applies
  under, the Park-2T pin states the closure and freestream it was derived from, and neither is a
  reinstatement of a band earned under the superseded `μ_sr` or the superseded freestream

#### Scenario: A prediction outside the anchor band is reported, not absorbed
- **WHEN** the corrected prediction falls outside a band that could honestly be called agreement with
  the RAM-C II anchor
- **THEN** the harness reports the measured offset as its result, and the documentation states the
  offset rather than describing the comparison as validation

### Requirement: 2-D bow-shock bounded-rank verification

A self-verifying 2-D blunt-body example SHALL march a bow shock in the body-fitted coordinate and gate that
the bond dimension stays **bounded and resolution-stable** (the measured `χ ~ O(10)` fitted regime), and that
captured-Cartesian control runs reproduce the `χ ~ √side` growth — confirming the coordinate is the lever.

#### Scenario: Fitted bow shock stays bounded; Cartesian grows
- **WHEN** the 2-D bow shock is run in the fitted coordinate and, as a control, captured on a Cartesian grid
- **THEN** the fitted run's χ is bounded and roughly resolution-independent while the Cartesian control's χ
  grows with resolution

### Requirement: 3-D forebody verification (wake out of scope)

A 3-D example SHALL march and validate the **forebody sheath** (the blackout-driving region) in the
body-fitted coordinate and gate that the forebody bond dimension stays **bounded**. The 3-D **wake** is
explicitly **out of scope** — it requires turbulence (a non-goal) and is downstream of the sheath; the
example SHALL report a forebody result and SHALL NOT gate or assert any wake-rank bound. If a wake bond
dimension is incidentally observed it is reported as an out-of-scope datapoint for the standing
`qtt_rank_3d` research question, not a claim.

#### Scenario: 3-D forebody is bounded; wake is not claimed
- **WHEN** the 3-D forebody example is run
- **THEN** the forebody bond dimension is bounded (consistent with the body-fitted `χ ~ O(10)` result) and the
  run exits zero, while any wake observation is labelled out-of-scope and never gated

### Requirement: Cross-references and honesty labels

The verification SHALL report cross-references — Sod analytic, RAM-C II electron density, Apollo blackout
dwell — with explicit labels for model scope (scalar vs full system where applicable, fitted vs captured, and
any reduced geometry), claiming no absolute match where the configuration differs.

#### Scenario: References reported with scope labels
- **WHEN** the suite reports results
- **THEN** each reference is shown with its tolerance and a scope/disclaimer label

