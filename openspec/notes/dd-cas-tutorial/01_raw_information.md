# Raw information: DD-CAS tutorial

Step 0 of `docs/writing_guides/TurnRawMaterialintoWriting.pdf`. This file holds facts and their sources and no prose
for the reader. Every fact carries the letter of its source. `02_outline.md` turns them into the tutorial plan.

Gathered on 2026-10-01, while writing the overview of the dynamic drone fail-safe tutorial. Every regulation, report
and manual below was read in the original unless the row says otherwise.

## Sources

| Letter | Source |
|---|---|
| A | 14 CFR §107.52 and §107.53, https://www.law.cornell.edu/cfr/text/14/107.52 and .../107.53; FAA rationale as summarised by Rupprecht Law, https://jrupprechtlaw.com/section-107-53-automatic-dependent-surveillance-broadcast-ads-b-out-prohibition/ |
| B | 14 CFR §91.225, https://www.ecfr.gov/current/title-14/chapter-I/subchapter-F/part-91/subpart-C/section-91.225; AOPA on non-electrical aircraft, 2017 |
| C | 14 CFR Part 89, §89.305 and §89.310, https://www.ecfr.gov/current/title-14/chapter-I/subchapter-F/part-89 |
| D | FLARM, Atom UAV Manual FTD-088 v1.1, 2022-06-06, https://www.flarm.com/wp-content/uploads/2024/04/FTD-088-Atom-UAV-Manual-1.pdf |
| E | FLARM, Data Port ICD FTD-012 v7.19, https://www.flarm.com/wp-content/uploads/2024/04/FTD-012-Data-Port-Interface-Control-Document-ICD-7.19.pdf |
| F | FLARM, JSON Protocol ICD FTD-092 v0.13, https://www.flarm.com/wp-content/uploads/2024/04/FTD-092-JSON-protocol-definition-v0.13.pdf |
| G | EASA, ADS-L 4 SRD860 technical specification, https://www.easa.europa.eu/sites/default/files/dfu/ads-l_4_srd860_issue_1.pdf (read through EASA's FAQ and unmannedairspace.info summaries) |
| H | DJI, Matrice 4D Series Flight Manual v1.2, 2025-07, https://dl.djicdn.com/downloads/DJI_Dock_3/20260312/DJI_Dock_3_user_manual_en.pdf |
| I | PX4, ADS-B/FLARM/UTM traffic avoidance, https://docs.px4.io/main/en/peripherals/adsb_flarm.html |
| J | ArduPilot, ADS-B receiver, https://ardupilot.org/copter/docs/common-ads-b-receiver.html |
| K | DLR, "A Review of Detect and Avoid Standards for Unmanned Aircraft Systems", Aerospace 12 (4), 2025, https://doi.org/10.3390/aerospace12040344 (read through search summaries) |
| L | FAA Part 108 NPRM, 2025-08-07, read through Skydio and AVweb summaries |
| M | NTSB WPR24LA302, Amazon MK30 midair collision, https://data.ntsb.gov/carol-repgen/api/Aviation/ReportMain/GenerateNewestReport/195091/pdf |
| N | NTSB DCA25LA065, Orlando drone show, final report 2026-09-15 |
| O | ATSB AO-2023-033, Docklands drone show (read through the sUAS News reprint of the full report) |
| P | NTSB DCA17IA202, Black Hawk and DJI Phantom 4, https://data.ntsb.gov/carol-repgen/api/Aviation/ReportMain/GenerateNewestReport/96058/pdf |
| Q | ABC News and KTLA on the Super Scooper collision, Palisades fire, 2025-01-09 |
| R | ASSURE, UAS Airborne Collision Severity Evaluation, https://www.assureuas.org/projects/uas-airborne-collision-severity-evaluation/ (read through the Ohio State summary) |
| S | University of Dayton Research Institute, "Risk in the Sky?", 2018, and DJI's objection |
| T | Reporting rules and records: 49 CFR 830.2 and its 2022 amendment; 14 CFR §107.9; MLIT (Japan) reporting rules 001520661.pdf and accident list 001585162.pdf; EASA Annual Safety Review 2025; NASA ASRS, "ASRS for Unmanned Aviation Systems", 2024 |
| U | FAA, CY 2024 Small UAS Survey Report, `end_to_end_tutorials/dynamic_drone_failsafe/papers/faa_2025_suas_survey_report_cy2024.pdf` |
| V | Iris Automation and Echodyne: DroneLife 2020 and 2023, Inside Unmanned Systems, Aerospace Testing International |
| W | This repository: `examples/avionics_examples/control/geometric_tcas/`, `deep_causality_physics/src/kernels/dynamics/estimation.rs`, `scripts/check_no_std.sh`, `end_to_end_tutorials/dynamic_drone_failsafe/` |
| X | Computed here |

## What drones may transmit and receive

- A small drone under Part 107 may not fly with a transponder on (§107.52) or with ADS-B Out transmitting (§107.53),
  unless the FAA authorises it; effective 2021-03-16. [A]
- The FAA's reason: drone transmitters could saturate the ADS-B frequencies and blind ground receivers and manned
  aircraft. [A]
- Receiving (ADS-B In) is not prohibited. DJI AirSense receives 1090ES and UAT ADS-B from crewed aircraft within
  10 km. [H] FLARM's Atom UAV carries a 1090 MHz ADS-B receiver. [D]
- Remote ID: a standard drone broadcasts its ID, latitude, longitude, geometric altitude, velocity, the control
  station's position and a UTC time mark (§89.305), at least once per second, at most 1.0 s from measurement to
  broadcast (§89.310). It was written for identification. [C]
- RTCA DO-396, the ACAS sXu standard for drones under 55 lb, dates from 2022-12-15 and runs to over 1,600 pages; it
  keeps the ban on transponder and ADS-B Out for small drones. [K]

## What crewed aircraft must broadcast

- ADS-B Out is required in Class A, B and C airspace, within 30 NM of the largest airports, in Class E above
  10,000 ft MSL, and a few other areas (§91.225). Below that, away from big airports, it is not required. [B]
- Aircraft "not originally certificated with an electrical system … including balloons and gliders" are exempt
  (§91.225(e)). [B]
- The FAA ran a rebate programme for owners retrofitting ADS-B Out for the 2020 mandate. [B]
- Part 108 (proposed 2025-08-07) would require drones flying beyond visual line of sight to receive ADS-B on 1090 MHz
  and UAT, detect aircraft, and yield to those broadcasting ADS-B Out or other electronic conspicuity. It would give
  drones right of way over crewed aircraft that do not broadcast. The FAA reopened comment on 2026-01-27 with seven
  questions on ADS-B Out, electronic conspicuity and detect-and-avoid. Final rule status not checked. [L]

## Broadcast systems that already exist

- FLARM broadcasts each aircraft's predicted 3D flight path twice per second, encrypted, on the 868 MHz (SRD860) or
  915 MHz band; in service since 2004; over 50,000 manned aircraft, nearly 100 % of gliders in Europe. [D]
- Atom UAV: 43 g (15 g as OEM board), 1.4 W typical, Cortex-M4F core, FLARM transceiver, 1090 MHz ADS-B receiver,
  Remote ID broadcast and Wi-Fi NAN Remote ID reception; outputs a unified traffic stream as JSON or MAVLink
  `ADSB_VEHICLE`; verified with PX4 1.13.2 and ArduCopter 4.3.3. [D]
- EASA ADS-L transmits position and identity on licence-free SRD860: 868.2 and 868.4 MHz at 25 mW ERP, 869.525 MHz at
  500 mW. [G]

## FLARM data formats and samples

- `PFLAA` (FTD-012): alarm level (0 none; 1, 2, 3 = 15–20, 10–15, 0–10 s to impact), relative north, east, vertical
  in m, ID type, ID, track (deg), turn rate (empty), ground speed (m/s), climb rate (m/s), aircraft type (D = UAV).
  [E]
- `PFLAA` is best effort: targets may be skipped or irregular, values may be extrapolated; FLARM says to use `PFLAU`
  as the primary alarm source. Stealth mode blanks track, speed and climb and adds altitude noise until an alarm. [E]
- Sample, verbatim: `$PFLAA,0,-1234,1234,220,2,DD8F12,180,,30,-1.4,1*` [E]
- JSON traffic (FTD-092): `pos` lat, lon (deg), alt above the ellipsoid, baro (m); `mov` speed (m/s), gnd, climb
  (m/s), turn (deg/s), track (deg); `time`; one traffic message per target between heartbeats, best source first.
  Sample, verbatim: `{"traffic":{"id":{"random":123},"mov":{"climb":5.2,"speed":4.2,"track":90.2,"turn":-2.4},
  "pos":{"alt":491,"baro":481,"lat":47.214272,"lon":8.4666112},"rec":[{"rad":{"dBm":-64.9}}]}}` [F]
- The Atom UAV manual's sample stream holds `heartbeat`, `navigation` (own position, speed, ground flag, time) and
  `traffic` messages of ground targets without speed or track. [D]
- The `PFLAA` sample as geometric TCAS input, own drone hovering: relative position (E, N, U) = (1234, −1234, 220) m,
  relative velocity = (0, −30, −1.4) m/s, time to closest approach = −36712 / 901.96 = −40.7 s: diverging, no
  advisory, matching FLARM's alarm level 0. [X]

## What autopilots do with traffic

- PX4 consumes ADS-B, FLARM and UTM traffic. `NAV_TRAFF_AVOID`: 0 disabled, 1 warn, 2 Return, 3 Land, 4 Hold,
  5 Terminate. A conflict needs all of `NAV_TRAFF_A_HOR`, `NAV_TRAFF_A_VER` and `NAV_TRAFF_COLL_T`. An ASTM F3442 mode
  maps alert levels to actions through `DAA_LVL_{LOW,MED,HIGH,CRIT}_ACT`. [I]
- ArduPilot's `AVOID_ADSB` mode: `AVD_F_ACTION` 2 climb or descend, 3 move horizontally, 4 move perpendicularly in
  3D, 5 RTL, 6 hover; thresholds `AVD_F_DIST_XY`, `AVD_F_DIST_Z`, `AVD_F_TIME`. Its documentation names ADS-B only.
  [J]
- DJI AirSense "only issues warning messages" and is "not able to actively control or take over the DJI aircraft to
  avoid collisions"; it cannot receive aircraft without working ADS-B Out. [H]
- DJI's vision obstacle sensing: "The aircraft cannot bypass obstacles if the flight speed exceeds the effective
  sensing speed"; near power lines DJI recommends an extra obstacle-sensing module. [H]
- Non-cooperative detection on the market: Iris Automation Casia 360 (on-board cameras and computer vision) and
  Casia G (ground nodes); Echodyne ground radar. Kansas DOT received an FAA waiver to fly BVLOS on on-board detection
  alone; the University of Alaska flew a 3.87-mile pipeline inspection with Casia and eight Echodyne radars. [V]

## Incidents

- Amazon MK30, Pendleton, Oregon, 2024-09-06: "N213PA was intentionally faulted by the UAS operator to simulate a
  'motor out recovery' and diverted to an alternate landing pad as the system was designed to do. The same fault was
  applied to N282PA as it was departing, which then diverted the UAS to the same alternate landing pad and into the
  flight path of N213PA, resulting in a mid-air collision." Probable cause: "the operator's failure to maintain
  separation between two drones during dual simulated engine out recoveries." Both held experimental airworthiness
  certificates. [M]
- Orlando, 2024-12-21: setup errors crossed the drones' paths; the UVify IFO "is not equipped with collision avoidance
  technologies"; the accident drone approached the soft geofence and crossed the hard geofence 0.1 s later at
  29.9 m/s. [N]
- Docklands, 2023-07-14: the Damoda Newton V2.2 drones "were not fitted with sensors to allow independent collision
  avoidance"; outside the geofence "the motors were shut down". [O]
- Black Hawk and DJI Phantom 4, 2017, about 300 ft: a 1½-inch dent in a main-rotor blade's leading edge, cracks in the
  composite fairing and window frame, debris in the engine oil-cooler fan; minor damage, normal landing. [P]
- Super Scooper (CL-415), Palisades fire, 2025-01-09: a drone holed the left wing about 3 by 6 in; out of service
  about five days; the operator pleaded guilty, over $65,000 restitution. [Q]

## Damage physics

- Drones damage aircraft more than birds of the same weight: the motor, battery and payload are rigid. Tested: 2.7 lb
  and 4 lb quadcopters, 4 lb and 8 lb fixed-wing drones, against a single-aisle transport and a business jet;
  horizontal stabilisers worst, windshields least. [R]
- A 2.1 lb DJI Phantom 2 at 238 mph tore open a Mooney M20 wing's leading edge and damaged the main spar; a gel bird
  of the same weight did not reach the spar. DJI called the speed unrealistic. [S]

## Why there are no collision rates

- The NTSB counts a drone event as an accident only with death or serious injury, or when the aircraft holds an
  airworthiness certificate and is substantially damaged (49 CFR 830.2, 2022 amendment). [T]
- Part 107 operators report serious injury or damage over $500 to property other than the drone (§107.9). The FAA
  publishes no counts by event type that we found. [T]
- Japan's MLIT rules, section 6(1): a collision between two drones counts as property damage only if the other drone
  belongs to a third party; with the same owner, a report depends on other conditions. MLIT's list holds 195 reports
  from 2022-12-06 to 2025-03-18 and no drone-on-drone collision. [T]
- EASA recorded 21 drone occurrences in 2024 (accidents and serious incidents), mostly loss of control and airprox.
  [T]
- NASA ASRS received 692 drone-related voluntary reports from April 2021 to June 2024. [T]
- US commercial operators flew over 16.6 million drone flights in 2024. [U]
- Among n drones flying independently in a shared volume, the number of pairs is n(n−1)/2, so encounters grow roughly
  with the square of n. [X]

## What the repository already has

- `geometric_tcas`: `AircraftState` holds position and velocity as east-north-up multivectors; the closest point of
  approach comes from the wedge product of relative position and velocity; thresholds 500 m horizontal, 100 m
  vertical, 45 s look-ahead, a resolution advisory below 20 s; the resolution is a vertical heuristic (descend if the
  intruder is above); an automatic intervention fires when a descend advisory goes unanswered for 2.5 s. No
  trigonometry in `model.rs` or `main.rs`. [W]
- `deep_causality_core` and `deep_causality_multivector` declare `no-std`; `scripts/check_no_std.sh` builds every such
  crate for `thumbv7em-none-eabihf` (Cortex-M4F). [W]
- `deep_causality_physics` provides a linear Kalman filter update in Joseph form. [W]
- The drone fail-safe tutorial: part 2 fuses uncertain readings into a context by inverse variance and decides by
  sequential probability tests; part 4 rules on every candidate maneuver with the Effect Ethos; part 5 runs a seeded
  1,000-scenario campaign with paired comparisons and Clopper–Pearson bounds. [W]
- Not in the repository: data association across sensors, track management, coordination between two avoiding
  aircraft. [W]
