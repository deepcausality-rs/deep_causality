# Weather Balloon: One Battery Envelope, Three Regimes

A long-duration weather balloon carries a lithium-ion pack that powers its sensors and radio; solar panels recharge it. The expedition climbs at 5 m/s to 20 km, floats through an afternoon and a night, and descends at 07:00 the next morning. A Causal State Machine keeps the pack inside one envelope the whole way:

- charge only between 0 °C and 45 °C, at half rate below 10 °C and above 40 °C;
- cool the pack above 35 °C, heat it below 5 °C.

The envelope never changes. What keeps the pack inside it does.

## How to Run

```bash
cargo run -p csm_examples --example csm_weather_balloon_example
```

or

```bash
bazel run //examples/csm_examples:csm_weather_balloon_example
```

## What the Controller Measures

Once a minute the controller reads four sensors: pressure, air temperature, cell temperature and sunlight. It never reads the altitude or the clock. The simulated world in `model.rs` drives those sensors from a standard atmosphere, the launch-day temperature, the sun's elevation, and one deviation: from 13:30 to 16:00 the balloon floats in air 25 K warmer than the standard atmosphere, above a cloud deck that reflects half again as much sunlight onto the gondola.

## Reasoning: Which Mechanism Cools Better?

A fan's heat transfer falls with air density (to the power 0.8); a radiator's does not depend on air at all. Two causaloids compare the heat each removes at the 35 °C set point, in the air measured now. When the other mechanism removes 10 % more, the regime changes. The crossover depends on air temperature as well as density, so no fixed altitude marks it: on this flight the radiator takes over at 17.7 km on the way up and the fan returns at 16.1 km on the way down.

## Action: The State Machine

Ten states, each a rule paired with an action:

| States | Rule | Action |
|--------|------|--------|
| Cooling on / off | cell at or above 35 °C / at or below 30 °C | fan on / off, or radiator open / closed |
| Heater on / off | cell at or below 5 °C / at or above 15 °C | heater on / off |
| Cell window full / half / stop | the cell's temperature against the charge window | sets the cell charge limit |
| Heat budget full / half / stop | heat coming in at full charge, against what the better cooling mechanism removes with the cell at 45 °C, in the air measured now | sets the budget charge limit |

The charger obeys the stricter of the two limits. The cell window reacts to the temperature the cell has reached; the heat budget looks at where the air and the sun are taking it.

When the regime changes, `update_single_state` replaces the two cooling states while the machine runs. Their rules stay the same, because the envelope does; their actions switch from fan to radiator, and their version rises. The outgoing mechanism is switched off first.

An action is a function with no arguments, so each action writes an actuator register (an atomic), and the simulation reads the registers.

## What the Output Shows

- **Launch pad, 32 °C air, full sun.** The heat budget halves the charge rate while the cell window still allows full; three minutes into the climb the cooler air restores full rate.
- **17.7 km.** The radiator replaces the fan.
- **Float.** In −56.5 °C air the sunlit pack still reaches 35 °C, and the radiator cycles to hold it between 30 °C and 35 °C.
- **Warm air and cloud deck.** The heat budget halves the charge rate at once, with the cell at 31.7 °C; the cell window alone would have waited for 40 °C.
- **Night.** No sun, no charging; the heater holds the cell between 5 °C and 15 °C and the pack falls to 30 %.
- **Dawn and descent.** Charging resumes at full rate; at 16.1 km the fan replaces the radiator.

The cell stays between 4.9 °C and 38.4 °C for the whole flight.

## Model Assumptions

One thermal mass for the pack and electronics; foam insulation in series with outside convection that scales with the square root of density; a fan whose conductance scales with density to the power 0.8; a radiator exchanging heat with surroundings at air temperature; clear-sky sunlight attenuated by the air above. Charging heat (I²R ≈ 0.6 W at full rate) is small, so derating protects the cell chemistry rather than cooling the pack.

## Precision

`FloatType` in `main.rs` sets the working precision. The flight runs unchanged at `f32`, `f64` and `deep_causality_num::Float106`.
