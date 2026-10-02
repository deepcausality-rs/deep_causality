# DD-CAS: Dynamic Drone Collision Avoiding Systems — tutorial outline

Status: proposed, nothing built. Facts carry the source letters of `01_raw_information.md`.

DD-CAS is the follow-up to the dynamic drone fail-safe tutorial (`end_to_end_tutorials/dynamic_drone_failsafe`). Its
hallmark is multi-source sensor fusion on the drone: every traffic source the drone can hear or see goes into one
uncertain picture of the airspace, and the drone resolves conflicts from that picture with no help from the ground.

The tutorial has two halves. The first explains why the status quo leaves drones and low-flying aircraft blind to
each other. The second builds the answer into the drone, from its existing sensors, and measures in simulation how
far it can be pushed.

## Thesis

Drone collision avoidance is an emerging problem that nobody has measured. Without measurement, the standards stay
fragmented, the technology stays unapplied, and nothing is coordinated. Avoidance belongs in the drone, built from
the sensors it already carries, and not in a separate product bought on top of it.

## **Open questions for the maintainer**

- Which drone class and sensor stack the tutorial assumes, and whether ADS-B In counts as existing.
- Which broadcast sources to simulate: Remote ID, ADS-B In, FLARM, ADS-L, or a subset.
- The camera detection model: range, field of view, false-alarm rate, latency.
- The motion model of a silent crewed aircraft, and of a drone that does not run DD-CAS.
- The coordination rule between two DD-CAS drones.
- The near mid-air collision distances for drone-drone and drone-aircraft encounters.
- Real data: whether to use the Open Glider Network's FLARM feed for traffic tracks (not checked).
- Where it lives: a new tutorial under `end_to_end_tutorials/`, reusing `geometric_tcas`, or extending that example.

## Half 1: the status quo

Each section states facts; the conclusion is left to the reader.

1. **Who may say where they are.** Small drones may not transmit on the aviation surveillance frequencies (§107.52,
   §107.53), to keep them from saturating ADS-B. [A] Crewed aircraft must broadcast only in certain airspace; below
   10,000 ft away from big airports, and for aircraft without an electrical system, they need not. [B] The aircraft a
   drone meets down low may therefore be silent.
2. **Who may hear.** Receiving is allowed, and some drones do: DJI AirSense receives ADS-B but only warns. [H] Remote
   ID makes every drone broadcast its position once a second, for identification, not avoidance. [C]
3. **What already exists, unconnected.** FLARM broadcasts predicted flight paths twice a second on a licence-free band
   and flies in 50,000 aircraft; its drone module weighs 15 g as a board. [D] Europe specified ADS-L for light
   aircraft on the same band. [G] A drone standard, ACAS sXu, exists on 1,600 pages and in no drone we found. [K]
4. **What autopilots do with traffic.** PX4 and ArduPilot react with one action chosen before flight: warn, Return,
   Land, Hold, climb, move aside, and in PX4 even Terminate. [I] [J] The same pattern as the fail-safes of the first
   tutorial.
5. **What happens.** Two Amazon MK30s, each faithfully running its fail-safe to the same alternate landing pad,
   collided. [M] Show drones without collision avoidance collided in Orlando and Melbourne. [N] [O] Drones have
   dented a Black Hawk's rotor and holed a firefighting plane's wing. [P] [Q]
6. **Why it matters physically.** A drone damages an aircraft more than a bird of the same weight. [R] [S]
7. **Why nobody counts.** Reporting rules catch injuries and certificated aircraft, not drone-on-drone collisions;
   Japan exempts collisions between drones of one owner by rule. [T] Encounters grow roughly with the square of the
   number of drones, and commercial flights already number 16.6 million a year in the US alone. [U] [X]

## Half 2: the system

### Principle

Everything runs on the drone, from the sensors a current drone carries: GNSS and IMU for its own state, the
Wi-Fi or Bluetooth radio for Remote ID, an ADS-B receiver where fitted, and the vision cameras it already uses for
obstacle sensing. A FLARM or ADS-L receiver and a ground radar feed are optional sources, not requirements.

### Layers, mapped onto DeepCausality

| Layer | Does | Built from |
|---|---|---|
| Sense | Reads each source at its own rate and noise: Remote ID, ADS-B In, FLARM, camera detections | simulated sources with realistic rates, latencies and dropouts |
| Fuse | Associates reports with tracks, updates each track's uncertain position and velocity, starts and drops tracks | `deep_causality_context`, `Uncertain` values, the Kalman update in `deep_causality_physics`; **data association is new** |
| Assess | Computes the closest point of approach per track, with its uncertainty, and decides conflict by a sequential probability test | the geometry of `geometric_tcas`, the sequential tests of the fail-safe tutorial |
| Resolve | Proposes maneuvers and rules on each against norms: separation from every track, what lies below, terrain and obstacles, the coordination rule | the Effect Ethos, as in the fail-safe tutorial's part 4 |
| Act | Flies the chosen maneuver and re-assesses each cycle | a causal state machine, as in part 3 |

### Coordination

Two DD-CAS drones in conflict must not both climb. TCAS solves this between airliners by agreeing who climbs. DD-CAS
needs a rule each drone can apply alone from shared data, such as a deterministic choice by geometry and ID, and may
use a broadcast intent where a channel carries one (FLARM's predicted path). [D] The rule is an open design question.

### Proposed parts

1. **Cooperative traffic, one source.** Remote ID or FLARM traffic into geometric TCAS; the conflict geometry against
   a fixed, PX4-style reaction. Ends on: a silent aircraft is invisible.
2. **Multi-source fusion.** Add ADS-B In and the camera, with false alarms and missed detections; data association
   into one track per aircraft. Ends on: the drone sees the conflict and still picks one fixed maneuver.
3. **Silent aircraft.** A helicopter or crop-sprayer without electronic conspicuity, seen only by the camera, late
   and noisy. Ends on: a maneuver that clears the aircraft but descends over people.
4. **Effect Ethos resolution and coordination.** Maneuvers ruled on against the air and the ground; two DD-CAS drones
   resolving the same conflict.
5. **Verification.** A seeded campaign over encounter geometries (head-on, crossing, overtaking, vertical), traffic
   mixes, sensor degradation and a density sweep. Baselines: no avoidance, the fixed reaction, DD-CAS. Metrics: near
   mid-air collisions, conflicts the maneuvers create, ground risk of the maneuvers, false and missed alarms, each
   with a confidence bound and a paired comparison as in the fail-safe tutorial's part 5.

The FLARM samples of [E] and [F] become test vectors: the `PFLAA` sample must give a diverging encounter and no
advisory, as FLARM's alarm level 0 says. [X]

## Out of scope

Certification, flight hardware and real flight; air traffic control; the regulation itself. The tutorial measures
what an on-board system can do in simulation and says so.


