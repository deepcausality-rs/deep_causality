# Context Types: Space and Time Representations

This module provides a robust and extensible system of spatial
and temporal context types for causal modeling across symbolic, physical, and hybrid domains.


---

## ⚖️ Why So Many Context Types?

Not all causal systems operate in the same regime.

Some are **relativistic**, others **discrete**.  
Some use **geographic coordinates**, others rely on **symbolic logic**.

Supporting multiple space and time types allows you to reason natively in each domain
without coercing your models into a single format or losing meaning through abstraction.

---

## 🧠 When to Use Which Context Type

| **Domain**                           | **Time Type**    | **Space Type**                              | **Use This When...**                                       |
|--------------------------------------|------------------|---------------------------------------------|------------------------------------------------------------|
| Relativistic physics, MagNav         | `LorentzianTime` | `MinkowskiSpacetime`, `LorentzianSpacetime` | You model causality, velocity, or lightcones               |
| Quantum/statistical models           | `EuclideanTime`  | `EuclideanSpacetime`                        | You run simulations, QFT, or use Wick rotation             |
| Symbolic AI, planning, rules         | `SymbolicTime`   | `SymbolicContext` or `SymbolicSpacetime`    | You reason in logical or qualitative steps                 |
| Embedded / step-based systems        | `DiscreteTime`   | `EuclideanSpace`, `NedSpace`                | Your systems run on ticks or control loops                 |
| Real-world navigation (MagNav, GNSS) | `LorentzianTime` | `GeoSpace`, `EcefSpace`, `NedSpace`         | You use real sensors, location data, or earth-fixed frames |
| Robotics, attitude control           | `LorentzianTime` | `QuaternionSpace`                           | You track orientation in 3D space                          |
| Simulation or animation engines      | `DiscreteTime`   | `EuclideanSpace`                            | You simulate systems frame-by-frame                        |
| Emergent/thermodynamic systems       | `EntropicTime`   | any                                         | You care about time direction or entropy                   |
| Human-interpretable traces           | `SymbolicTime`   | any                                         | You want readable timelines or explainability              |

---

## 🔩 How It's Designed

- All space types implement `Spatial<V>` and `Coordinate<V>`
- All time types implement `Temporal<VT>`
- `SpaceKind` and `TimeKind` enums allow polymorphic usage in core systems
- Contexts are statically typed, but composable

This design supports:

- Physical models (real-valued, time-aware)
- Symbolic models (label-driven, logic-first)
- Hybrid models (e.g., sensor fusion with symbolic constraints)

---

## 🧪 Example

```rust
let t = TimeKind::Lorentzian(LorentzianTime::new(1, TimeScale::Seconds, 1.23));
let s = SpaceKind::Geo(GeoSpace::new(1, 48.85, 2.35, 35.0));

println!("Time ID = {}, value = {}", t.id(), t.time_unit());
println!("Space dimension = {}", s.dimension());
