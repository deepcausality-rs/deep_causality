---
title: DeepCausality v.0.6 supports multiple contexts
description: This post summarizes the new multiple context feature of DeepCausality v.0.6 
date: 2023-09-08
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

DeepCausality v.0.6 now supports multiple contexts. The previous context API remains the same, meaning all
existing code should compile as before. However, new API functionality was added to interact with additional contexts to
enable more advanced use cases.

## Problem

To deal with problems in the financial industry when modelling synthetics such as future spreads, multiple contexts
apply to the instrument. Specifically, when modelling a classic long-short-term spread, in total three contexts apply,
one for the long-term future contract, a second one for the short-term contract, and a final one for the resulting
spread. In previous versions of DeepCausality, all three context could only be stored in one context, which becomes
cumbersome to maintain over time.

Similarly, in the IoT industry, multiple contexts may arise from different sensor networks and therefore require more
than one context.

## Concepts

DeepCausality enables contextual causal reasoning across data that change over time, space, and spacetime through an
adjustable context. DeepCausality comes with four default node types that can be stored in a context:

* Data – T
* Space - T
* Time - T
* SpaceTime - T

For the data node, the generic type T refers to the exact embedded data, which might be a primitive (i32, u64, etc) type
or a struct. For time and spacetime, T refers to the time unit. When time is represented as low resolution, say years
and weeks, then a U8 integer should suffice, whereas for higher resolution time, say minutes, it would require a larger
number type, say an u32. For space, T represents the coordinates, and depending on the requirements, one might choose an
i32 for whole numbers or a f64 for double-precision floats.

More comprehensible customization of time, spacetime, or space nodes requires the implementation of the corresponding
traits in a custom type.

Also, for each of the four default node types, adjustable equivalents exist in the library:

* AdjustableData – T
* AdjustableSpace - T
* AdjustableTime - T
* AdjustableSpaceTime - T

The difference is that the node types are immutable by default, whereas the adjustable node types can be modified (
adjusted) by implementing the adjustable trait. Please refer to the tests for details.

DeepCausality comes with a type alias, "BaseContext” that refers to a context that uses only the default node types
provided by the library for smaller tasks or testing. When the default node types are insufficient, the context type
allows more flexibility to mix default and custom types as needed. For the code examples below, the BaseCotext is used
for simplicity.

## Create a new context

You create a new context by calling the constructor with capacity. The capacity expands automatically once the initial
capacity has been reached, but for performance reasons, it's best to set a reasonable initial capacity with some
headroom.

```rust
use deep_causality::prelude::*;

// Util function 
fn get_context<'l>() -> BaseContext<'l> {
    let id = 1;
    let name = "base context";
    let capacity = 10; // adjust as needed
    Context::with_capacity(id, name, capacity)
}
``` 

### Add nodes & edges to the default context

Once a context has been created, you can freely add nodes and edges to the default context. The API for the default
context is largely self-explanatory with the usual CRUD operations.

```rust
use deep_causality::prelude::*;

fn main() {
    let mut context = get_context();
    
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let idx = context.add_node(contextoid);

    assert_eq!(idx, 0); // index of the added node
}
``` 

For more examples, please see
the [unit tests](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/tests).

## Create an additional context

In a scenario that requires more than one context, DeepCausality supports the addition of an arbitrary number of
additional contexts.

You add an additional context using the extra_ctx_add_new method, and this creates a new additional context and returns
the generated context ID. The Boolean argument determines if the newly created context is set as default for all
operations. Please be aware of the following:

1) You can only create an additional context but not delete one.
2) Because of 1), context id’s are strictly sequential
3) Before using an additional context, you have to ensure it has been set

The API is designed so that you have to set a context to use all API functions prefixed with extra_ctx. In practice,
this entails the following workflow:

1) Set context ID to the correct additional context
2) Operate on the additional context
3) Unset context ID or switch to another context

It is important to track context IDs in the actual application, for example, in a HashMap, to prevent accidentally
modifying the wrong context. The API provides functionality to retrieve the current context ID so you can test
programmatically whether the right context is already set and, if not, set it correctly.

```rust
use deep_causality::prelude::*;

fn main() {
    let mut context = get_context();
    
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    
    // Operation on the main context. 
    let idx = context.add_node(contextoid);
    
    // Capacity of the new additional context
    let capacity = 100;
    // Flag to set the additional context
    let default = true;

    // Create a new additional context
    let ctx_id = context.extra_ctx_add_new(capacity, default);

    // Context ID of the new additional context
    assert_eq!(ctx_id, 1);
    
    // Check if the additional context exists 
    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);
    
    // Get the currently set context ID
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);
    
    // Unset the additional context
    let res = context.extra_ctx_unset_current_id();
    assert!(res.is_ok());

    // Zero is the default value for the if nothing else is set
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, 0);
}
``` 

## Modify an additional context

Once the correct context has been set, all API functions prefixed with extra_ctx operate on the set context. The extra
context API mirrors the regular context API in functionality. If you accidentally call an API function without the
extra_ctx prefix, you operate against the default index and may encounter unexpected results.

```rust
use deep_causality::prelude::*;

fn main() {
    let mut context = get_context();
    
    // Capacity of the new additional context
    let capacity = 100;
    // Flag to set the additional context
    let default = true;

    // Create a new additional context
    let ctx_id = context.extra_ctx_add_new(capacity, default);

    // Create a new root node 
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    
    // Add new root node to the additional context
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_ok());
    let node_id = res.unwrap();
    assert_eq!(node_id, 0);
    
    // Check if the root node exists
    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);
    
    // Create new time node
    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12; // 12 = December
    let tempoid = Time::new(t_id, t_time_scale, t_time_unit);

    // Add time node
    let id = 2; 
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());
    
    // Add a relation between root node and the time node 
    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());
}
```

## Accessing an additional context

Accessing an additional context from within a causal model works the exact same way as accessing the default context by
calling the corresponding get method. For the main context, it's get_node, and for an additional context, its
extra_ctx_get_node. specifically, when constructing a causaloid, you can specify either a context-free causaloid or a
contextual causaloid. The example below shows how to build a contextualized causaloid that accesses both, the default
context and an additional context.

```rust
use deep_causality::prelude::*;

fn main() {
 let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let context = get_context();

    fn contextual_causal_fn<'l>(obs: NumericalValue,ctx: &'l BaseContext<'l>) -> Result<bool, CausalityError> {
       
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into());
        }

        // get contextoid by ID from the default context 
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");

        // extract data from the contextoid
        let val = contextoid.id() as f64;
        
        // check whether a node exists in the additional context
        let node_id = 3; // Assuming fixed node id
         let exists = context.extra_ctx_contains_node(node_id);
        assert!(exists);

        // get contextoid from the additional context
        let node = context.extra_ctx_get_node(node_id);
        assert!(node.is_ok());
    
        let exta_contextoid = node.unwrap();
        assert_eq!(exta_contextoid.id(), 1);

        // run any arithmetic with the data from the contextois
        if val == 1.0 {
            Ok(true)
        } else {
            // relate the observation (obs) to the data (val) from the contextoid
            if !obs.ge(&val) {
                Ok(false)
            } else {
                Ok(true)
            }
        }
    }

    // Build & returnt the causalouid 
    let causaloid: BaseCausaloid =
        Causaloid::new_with_context(id, contextual_causal_fn, Some(&context), description);

}
``` 

Because context is passed as an immutable reference into the causal model, the model cannot modify its context, which
means updating the context must happen before evaluating a causal model using the context. For more details on the
context design, see the the architecture documentation.

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware
causal reasoning in Rust. Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)
