# Customizing context

The contextuable protocols are a set of traits wihtout default implementations that specify a context and
its node types. Each node type has a default type implementation in the type folder. However, these types are fairly
basic and whenever you want a custom context, you have to do some customizations;

## Customizing context:

### Extend the node type trait i.e. Temporal

Example:

```rust
pub trait Temporable: Temporal {
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> u32;
}
```

### Define a custom node type i.e. Tempoid

```rust
pub struct Tempoid{
    id: u64,
    time_scale: TimeScale,
    time_unit: u32,
}
```

### Implement all super traits and the new custom trait

Full code: [Tempoid](../types/context_types/node_types/tempoid.rs)

```rust
// A bunch of required super traits
impl Temporal for Tempoid {}
impl Adjustable for Tempoid {}
impl Identifiable for Tempoid {
    fn id(&self) -> u64 { self.id }
}

// Custom trait 
impl Temporable for Tempoid
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> u32 {
        self.time_unit
    }
}
```

### Import super traits and the new custom trait to build a custom context

Also, you have to import the custom trait wherever you want to access the
custom functionality of your custom type.

See the ctx example for
an [end to end code example](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/examples/ctx)
