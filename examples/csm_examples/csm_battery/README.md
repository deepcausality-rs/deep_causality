# CSM Example: Charging a Battery After Its Coolant Fails

A lithium-ion cell charges from 20 % to 80 %. After ten minutes its coolant pump stops. A Causal State Machine sets the
charge current from the cell temperature once a minute, and the same session runs twice:

1. The controller keeps the charge law written for a cooled cell.
2. A causaloid detects the loss of cooling from the temperature trend, and the controller replaces its charge states
   with a derated law while it runs.

The cell model is the one from the project README's battery example: 50 Ah, 25 °C ambient, the current heats the cell
by `0.00048 · I²` per minute, and the coolant removes `0.08 · (T − ambient)`. Without the pump the cell sheds only
`0.01 · (T − ambient)`.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_battery_example
```

## Output

```text
A lithium-ion cell charges from 20 % to 80 %. Its coolant pump stops after 10 minutes.

The controller keeps the charge law for a cooled cell:
 minute  cell °C  charge  current
      1     25.0  20.0 %    50 A
     22     45.2  55.0 %    25 A
Reached 80.8 % after 52 minutes. The cell peaked at 47.8 °C and spent 32 minutes above 45 °C.

The controller replaces its charge states when it detects the loss of cooling:
 minute  cell °C  charge  current
      1     25.0  20.0 %    50 A
     12  >> the cell rose 1.12 K in the last minute; the pumped coolant allows 0.52 K.
         >> charge states replaced by the derated law, version 2: 20 A below 40 °C, 10 A to 45 °C.
     12     34.6  38.3 %    20 A
Reached 80.3 % after 74 minutes. The cell peaked at 39.1 °C and spent 0 minutes above 45 °C.

Replacing the charge states cost 22 minutes and kept the cell 8.7 K cooler at its peak.
```

## How It Works

1. **Three charge states.** States 1, 2 and 3 each pair a rule on the cell temperature with an action that sets the
   charger. The law for a cooled cell gives 50 A below 45 °C, 25 A up to 55 °C, and none above. Each minute the
   controller evaluates all three states on the same reading; the active one sets the current.
2. **A cooling-loss detector.** One more causaloid compares the last minute's temperature rise with the rise the
   pumped coolant allows for the current that flowed. When the cell rose more than 0.3 K above that, the cooling is
   lost.
3. **Replacing the states while running.** On detection, `update_single_state` replaces states 1, 2 and 3 with the
   derated law, version 2: 20 A below 40 °C, 10 A up to 45 °C, none above. The identifiers stay; the rules and actions
   behind them change. Without the pump, 20 A holds the cell near 44 °C.
4. **The trade.** The law for a cooled cell reaches 80 % in 52 minutes but holds the cell above 45 °C, where charging
   ages a lithium-ion cell faster, for 32 of them. The derated law takes 74 minutes and keeps the cell below 40 °C.

## Files

| File | Contents |
|------|----------|
| `main.rs` | The two charge sessions and the minute loop |
| `model_config.rs` | The charge states, the two charge laws, and the cooling-loss detector |
| `model.rs` | The cell on charge, the charger the actions set, and the heat balance |
| `model_types.rs` | `Reading` and `Summary` |
| `constants.rs` | The cell, the coolant, the session, and the thresholds of both laws |
| `utils_print.rs` | Console output |
