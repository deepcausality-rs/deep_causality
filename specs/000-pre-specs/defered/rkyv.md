# Specification: Refactoring Causaloid for Serialization and Auditable Models

## 1. Introduction

This report outlines a comprehensive plan to refactor the `Causaloid` struct within the `deep_causality` crate. The primary motivation is to enable robust serialization and cryptographic signing of causal models, which is crucial for achieving legal defensibility and ensuring data integrity in safety-critical, regulated industries. The current design, which utilizes raw function pointers, prevents direct serialization.

The proposed solution involves:
1.  Replacing function pointers with serializable data representations of causal logic.
2.  Introducing a new dedicated crate, `deep_causality_serialization`, to encapsulate all serialization concerns using the `rkyv` framework and a wrapper type strategy.
3.  Defining a `ModelContainer` structure to package models with their audit trails and cryptographic signatures, ensuring end-to-end verifiability.

## 2. Problem Statement: Function Pointers in `Causaloid`

The `Causaloid` struct, defined in `deep_causality/src/types/causal_types/causaloid/mod.rs`, currently includes fields that are `Option<fn(...)>`:

*   `causal_fn: Option<CausalFn>`
*   `context_causal_fn: Option<ContextualCausalFn<D, S, T, ST, SYM, VS, VT>>`

As identified in `deep_causality/src/alias/alias_function.rs`, `CausalFn` and `ContextualCausalFn` are type aliases for raw function pointers (`fn(...)`).

**Implications of Function Pointers:**
*   **Non-Serializable**: Function pointers are memory addresses. These addresses are not stable across different program runs, machines, or even minor code changes. Attempting to serialize them would store meaningless data.
*   **Non-Portable**: A serialized function pointer would be invalid upon deserialization in a different execution environment.
*   **Non-Verifiable Logic**: Even if a memory address could be serialized, signing it would only attest to the address, not the actual executable logic it points to. This provides no cryptographic guarantee of the function's behavior.

Therefore, to enable serialization and cryptographic signing of `Causaloid` instances, these function pointers must be replaced with a serializable representation of the causal logic they embody.

## 3. Proposed Refactoring of `Causaloid`: Pointerless State

The goal is to transform the `Causaloid` struct so that its internal logic is represented as serializable data. Given that `CausalFn` and `ContextualCausalFn` are `fn(...)` types (static functions), an enum-based approach is suitable for representing a finite, known set of causal behaviors.

### 3.1. Define Serializable Enums for Causal Logic

New enums will be introduced to represent the logic of `CausalFn` and `ContextualCausalFn`. These enums will be `#[derive(Archive, Deserialize, Serialize)]` from `rkyv`.

*   **New File**: `deep_causality/src/types/causal_types/causal_logic.rs`

    ```rust
    // Example: deep_causality/src/types/causal_types/causal_logic.rs

    use rkyv::{Archive, Deserialize, Serialize};
    use crate::{PropagatingEffect, CausalityError, Context, Datable, Spatial, Temporal, SpaceTemporal, Symbolic};
    use std::sync::Arc;

    // Enum for stateless causal functions (replaces CausalFn)
    #[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
    #[archive(check_bytes)]
    pub enum CausalLogic {
        Identity,
        Threshold(f64), // Example: a threshold function with a parameter
        SumInputs,
        // ... add variants for all other specific CausalFn implementations
    }

    impl CausalLogic {
        pub fn execute<D, S, T, ST, SYM, VS, VT>(
            &self,
            effect: &PropagatingEffect,
            // Context and generic bounds are added here for consistency with ContextualCausalLogic,
            // but might not be used by all CausalLogic variants.
            _context: &Arc<Context<D, S, T, ST, SYM, VS, VT>>,
        ) -> Result<PropagatingEffect, CausalityError>
        where
            D: Datable + Clone, S: Spatial<VS> + Clone, T: Temporal<VT> + Clone,
            ST: SpaceTemporal<VS, VT> + Clone, SYM: Symbolic + Clone,
            VS: Clone, VT: Clone,
        {
            match self {
                CausalLogic::Identity => Ok(effect.clone()),
                CausalLogic::Threshold(threshold) => {
                    // Placeholder for actual logic
                    if let PropagatingEffect::Numerical(val) = effect {
                        Ok(PropagatingEffect::Deterministic(val >= threshold))
                    } else {
                        Err(CausalityError("Expected Numerical effect for Threshold logic".into()))
                    }
                },
                CausalLogic::SumInputs => { /* ... */ Ok(effect.clone()) },
            }
        }
    }

    // Enum for context-aware causal functions (replaces ContextualCausalFn)
    #[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
    #[archive(check_bytes)]
    pub enum ContextualCausalLogic {
        ContextualFilter(String), // Example: filter based on a context key
        ContextualAggregate,
        // ... add variants for all other specific ContextualCausalFn implementations
    }

    impl ContextualCausalLogic {
        pub fn execute<D, S, T, ST, SYM, VS, VT>(
            &self,
            effect: &PropagatingEffect,
            context: &Arc<Context<D, S, T, ST, SYM, VS, VT>>,
        ) -> Result<PropagatingEffect, CausalityError>
        where
            D: Datable + Clone, S: Spatial<VS> + Clone, T: Temporal<VT> + Clone,
            ST: SpaceTemporal<VS, VT> + Clone, SYM: Symbolic + Clone,
            VS: Clone, VT: Clone,
        {
            match self {
                ContextualCausalLogic::ContextualFilter(key) => {
                    // Placeholder for actual logic using the context
                    // Example: context.get_data_by_key(key)
                    Ok(effect.clone())
                },
                ContextualCausalLogic::ContextualAggregate => { /* ... */ Ok(effect.clone()) },
            }
        }
    }
    ```

### 3.2. Refactor `Causaloid` Struct

The `Causaloid` struct will be updated to store instances of these new enums.

*   **File**: `deep_causality/src/types/causal_types/causaloid/mod.rs`

    ```rust
    // ... existing use statements ...
    use crate::types::causal_types::causal_logic::{CausalLogic, ContextualCausalLogic}; // Import new enums
    // No rkyv derives here, as deep_causality remains dependency-free

    #[allow(clippy::type_complexity)]
    #[derive(Clone, Debug, PartialEq)] // Remove rkyv derives from here
    pub struct Causaloid<D, S, T, ST, SYM, VS, VT>
    where
        D: Datable + Clone, S: Spatial<VS> + Clone, T: Temporal<VT> + Clone,
        ST: SpaceTemporal<VS, VT> + Clone, SYM: Symbolic + Clone,
        VS: Clone, VT: Clone,
        // Remove rkyv bounds from here
    {
        id: IdentificationValue,
        causal_type: CausaloidType,
        causal_logic: Option<CausalLogic>, // Replaces causal_fn
        contextual_causal_logic: Option<ContextualCausalLogic>, // Replaces context_causal_fn
        context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
        effect: ArcRWLock<Option<PropagatingEffect>>,
        causal_coll: Option<Arc<CausalVec<D, S, T, ST, SYM, VS, VT>>>,
        causal_graph: Option<Arc<CausalGraph<D, S, T, ST, SYM, VS, VT>>>,
        description: String,
        ty: PhantomData<(VS, VT)>,
    }

    // Update constructors to accept CausalLogic/ContextualCausalLogic
    impl<D, S, T, ST, SYM, VS, VT> Causaloid<D, S, T, ST, SYM, VS, VT>
    where
        D: Datable + Clone, S: Spatial<VS> + Clone, T: Temporal<VT> + Clone,
        ST: SpaceTemporal<VS, VT> + Clone, SYM: Symbolic + Clone,
        VS: Clone, VT: Clone,
    {
        pub fn new(id: IdentificationValue, causal_logic: CausalLogic, description: &str) -> Self {
            Causaloid {
                id, causal_type: CausaloidType::Singleton, effect: Arc::new(RwLock::new(None)),
                causal_logic: Some(causal_logic), contextual_causal_logic: None, context: None,
                causal_coll: None, causal_graph: None, description: description.to_string(), ty: PhantomData,
            }
        }

        pub fn new_with_context(
            id: IdentificationValue,
            contextual_causal_logic: ContextualCausalLogic,
            context: Arc<Context<D, S, T, ST, SYM, VS, VT>>,
            description: &str,
        ) -> Self {
            Causaloid {
                id, causal_type: CausaloidType::Singleton, effect: Arc::new(RwLock::new(None)),
                causal_logic: None, contextual_causal_logic: Some(contextual_causal_logic), context: Some(context),
                causal_coll: None, causal_graph: None, description: description.to_string(), ty: PhantomData,
            }
        }
        // ... other constructors and methods updated to use causal_logic/contextual_causal_logic
    }
    ```

### 3.3. Update `Causaloid::evaluate` Method

The `evaluate` method will dispatch to the `.execute()` method of the stored `CausalLogic` or `ContextualCausalLogic` enum variant.

*   **File**: `deep_causality/src/types/causal_types/causaloid/causable.rs`

    ```rust
    // ... existing use statements ...
    impl<D, S, T, ST, SYM, VS, VT> Causable for Causaloid<D, S, T, ST, SYM, VS, VT>
    where
        D: Datable + Clone, S: Spatial<VS> + Clone, T: Temporal<VT> + Clone,
        ST: SpaceTemporal<VS, VT> + Clone, SYM: Symbolic + Clone,
        VS: Clone, VT: Clone,
    {
        fn evaluate(&self, effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
            let effect = match self.causal_type {
                CausaloidType::Singleton => {
                    if let Some(contextual_logic) = &self.contextual_causal_logic {
                        let context = self.context.as_ref().ok_or_else(|| {
                            CausalityError(format!("Causaloid {} has contextual_causal_logic but is missing a context", self.id))
                        })?;
                        contextual_logic.execute(effect, context)?
                    } else if let Some(causal_logic) = &self.causal_logic {
                        // Pass a dummy context if the CausalLogic::execute requires it,
                        // or refactor CausalLogic::execute to not require context if it's stateless.
                        // For simplicity, assuming CausalLogic::execute can take a dummy context or be adapted.
                        let dummy_context = Arc::new(Context::with_capacity(0, "dummy", 0)); // Placeholder
                        causal_logic.execute(effect, &dummy_context)?
                    } else {
                        return Err(CausalityError(format!("Causaloid {} is missing causal logic", self.id)));
                    }
                }
                // ... CausaloidType::Collection and CausaloidType::Graph remain largely the same ...
                _ => { /* ... existing logic for Collection/Graph ... */ effect.clone() } // Placeholder
            };

            // Store the resulting effect for later inspection.
            let mut effect_guard = self.effect.write().unwrap();
            *effect_guard = Some(effect.clone());
            Ok(effect)
        }
        // ... other methods ...
    }
    ```

### 3.4. Impact on `deep_causality` Crate

*   **Internal Refactoring**: The changes are primarily internal to the `Causaloid` struct and its associated implementation blocks.
*   **Public API Changes**:
    *   The signatures of `Causaloid::new` and `Causaloid::new_with_context` will change, requiring updates at their call sites.
    *   The `CausalFn` and `ContextualCausalFn` type aliases will be removed or deprecated.
*   **No Direct `rkyv` Dependency**: The `deep_causality` crate itself will remain free of direct `rkyv` dependencies.

## 4. Proposed Serialization via `deep_causality_serialization` Crate

A new crate, `deep_causality_serialization`, will be introduced to handle all serialization and deserialization concerns using `rkyv`. This crate will depend on `deep_causality` and `rkyv`.

### 4.1. Wrapper Type Strategy

To maintain strict dependency separation and comply with Rust's orphan rule, `deep_causality_serialization` will use newtype wrappers for `deep_causality`'s types.

*   **New Crate**: `deep_causality_serialization`
*   **Dependencies**: `deep_causality = { path = "../deep_causality" }`, `rkyv = { version = "0.8", features = ["derive", "validation"] }`

    ```rust
    // deep_causality_serialization/src/lib.rs

    use deep_causality::{Causaloid, CausalLogic, ContextualCausalLogic, /* ... other types ... */};
    use rkyv::{Archive, Deserialize, Serialize, AlignedVec, Infallible};
    use rkyv::validation::ValidationError;

    // --- Wrapper for Causaloid ---
    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
    #[archive(check_bytes)]
    pub struct SerializableCausaloid<D, S, T, ST, SYM, VS, VT>(
        #[omit_bounds] // rkyv specific: allows inner type to not have rkyv bounds directly
        pub Causaloid<D, S, T, ST, SYM, VS, VT>
    )
    where
        // These bounds ensure the inner Causaloid's components are rkyv-compatible.
        // All types used within Causaloid (IdentificationValue, CausaloidType, etc.)
        // must also implement rkyv's traits.
        D: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<D, Infallible>,
        S: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<S, Infallible>,
        T: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<T, Infallible>,
        ST: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<ST, Infallible>,
        SYM: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<SYM, Infallible>,
        VS: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VS, Infallible>,
        VT: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VT, Infallible>,
    ;

    // --- Wrapper for CausalLogic ---
    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
    #[archive(check_bytes)]
    pub struct SerializableCausalLogic(pub CausalLogic);

    // --- Wrapper for ContextualCausalLogic ---
    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
    #[archive(check_bytes)]
    pub struct SerializableContextualCausalLogic(pub ContextualCausalLogic);

    // --- Serialization/Deserialization Functions ---

    pub fn to_bytes_causaloid<D, S, T, ST, SYM, VS, VT>(
        causaloid: &Causaloid<D, S, T, ST, SYM, VS, VT>
    ) -> Result<AlignedVec, rkyv::ser::serializers::AllocSerializerError>
    where
        D: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<D, Infallible>,
        S: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<S, Infallible>,
        T: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<T, Infallible>,
        ST: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<ST, Infallible>,
        SYM: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<SYM, Infallible>,
        VS: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VS, Infallible>,
        VT: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VT, Infallible>,
    {
        let wrapper = SerializableCausaloid(causaloid.clone());
        rkyv::to_bytes::<_, 256>(&wrapper)
    }

    pub fn from_bytes_causaloid<D, S, T, ST, SYM, VS, VT>(
        bytes: &[u8]
    ) -> Result<Causaloid<D, S, T, ST, SYM, VS, VT>, ValidationError>
    where
        D: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<D, Infallible>,
        S: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<S, Infallible>,
        T: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<T, Infallible>,
        ST: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<ST, Infallible>,
        SYM: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<SYM, Infallible>,
        VS: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VS, Infallible>,
        VT: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VT, Infallible>,
    {
        rkyv::check_archived_value::<SerializableCausaloid<D, S, T, ST, SYM, VS, VT>>(bytes)
            .map(|archived_wrapper| archived_wrapper.deserialize(&mut Infallible).unwrap().0)
    }
    ```

### 4.2. Benefits of the Wrapper Type Strategy

*   **Strict Zero Dependencies in Core**: The `deep_causality` crate's `Cargo.toml` remains completely free of `rkyv` (or any other serialization library).
*   **Orphan Rule Compliance**: This is the standard way to implement foreign traits (`rkyv`'s traits) for foreign types (`deep_causality`'s types).
*   **Clear Separation of Concerns**: Serialization logic and dependencies are entirely encapsulated within `deep_causality_serialization`.
*   **Centralized Validation Gateway**: The wrapper types provide the ideal place to integrate comprehensive validation logic (checksums, cryptographic signatures, semantic checks) during deserialization, acting as a certified data assurance layer.

## 5. Comprehensive Data Integrity and Auditing with `ModelContainer`

To ensure end-to-end verifiability, a `ModelContainer` struct will be defined within `deep_causality_serialization` to package the model, its audit trail, and cryptographic signatures.

### 5.1. `ModelContainer` Structure

```rust
// deep_causality_serialization/src/model_container.rs

use rkyv::{Archive, Deserialize, Serialize, AlignedVec};
// Assuming SerializableCausaloid and SerializableAuditTrail are defined as wrapper types
use crate::{SerializableCausaloid, SerializableAuditTrail};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct ModelMetadata {
    pub model_version: String,
    pub creation_timestamp: u64,
    pub author_id: String,
    pub intended_use: String,
    pub combined_model_audit_hash: [u8; 32], // Hash of (model_hash + audit_trail_hash)
    pub public_key_id: String, // Identifier for the public key used to sign this container
    // Add any other relevant metadata
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct ModelContainer<D, S, T, ST, SYM, VS, VT>
where
    // Generic bounds for SerializableCausaloid's generic parameters
    D: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<D, Infallible>,
    S: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<S, Infallible>,
    T: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<T, Infallible>,
    ST: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<ST, Infallible>,
    SYM: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<SYM, Infallible>,
    VS: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VS, Infallible>,
    VT: Archive + Serialize<rkyv::ser::serializers::AllocSerializer<256>> + Deserialize<VT, Infallible>,
{
    pub model_data: AlignedVec, // The rkyv-archived bytes of the SerializableCausaloid
    pub audit_trail_data: AlignedVec, // The rkyv-archived bytes of the SerializableAuditTrail
    pub metadata: ModelMetadata,
    pub metadata_signature: Vec<u8>, // Digital signature over the serialized metadata
}
```

### 5.2. Verification Process (Unpacking)

The `deep_causality_serialization` crate would provide functions to unpack and verify a `ModelContainer`:

1.  **Initial `rkyv` Structural Validation**: Validate the `ModelContainer`'s structure using `rkyv::check_archived_value`.
2.  **Deserialize `ModelContainer`**: Obtain the `model_data`, `audit_trail_data`, `metadata`, and `metadata_signature`.
3.  **Verify `metadata_signature`**:
    *   Serialize `metadata` to bytes and compute its cryptographic hash.
    *   Verify `metadata_signature` against this hash using the appropriate public key.
    *   **Failure indicates metadata tampering or unauthorized origin.**
4.  **Verify `model_data` and `audit_trail_data` Hashes**:
    *   Compute cryptographic hashes of `model_data` and `audit_trail_data`.
    *   Compare these with the hashes stored in `metadata.combined_model_audit_hash`.
    *   **Failure indicates corruption or tampering of the model or audit trail data.**
5.  **`rkyv` Structural Validation of Inner Data**:
    *   Perform `rkyv::check_archived_value` on `model_data` and `audit_trail_data` to ensure their internal structure is valid.
6.  **Deserialize and Semantic Validation (DIVC)**:
    *   Deserialize `model_data` into `deep_causality::Causaloid` (via `SerializableCausaloid`).
    *   Perform comprehensive semantic validation (Data Integrity Verification Component - DIVC) on the deserialized `Causaloid` to ensure logical consistency and adherence to safety rules.
    *   Similarly, deserialize and validate the audit trail.

### 5.3. Benefits for Bitwise Integrity and Legal Defensibility

This comprehensive approach directly addresses the concern of "rotten bytes" and accidental/malicious alteration:

*   **Bitwise Integrity**: Cryptographic hashes (SHA-256) are highly sensitive to any bit flip, ensuring detection of even the smallest corruption.
*   **Tamper Detection**: Digital signatures, combined with hashing, provide strong protection against intentional alteration and prove authenticity.
*   **Structural Assurance**: `rkyv`'s `check_bytes` ensures the archived data conforms to its expected memory layout, preventing memory unsafety.
*   **Centralized Control**: All critical integrity checks are performed within the certified `deep_causality_serialization` crate, acting as a trusted gateway.
*   **End-to-End Verifiability**: The `ModelContainer` provides a single, self-contained, cryptographically verifiable artifact that links the model, its provenance, and its integrity checks, forming an unimpeachable record for legal and regulatory scrutiny.

## 6. Conclusion

By refactoring `Causaloid` to a pointerless, enum-based representation and implementing serialization through a dedicated `deep_causality_serialization` crate using `rkyv` and wrapper types, the Deep Causality project can achieve:

*   **Strict Zero Dependencies** in its core crates.
*   **Robust, High-Performance Binary Serialization** for causal models and audit trails.
*   **Comprehensive Bitwise Integrity Verification** against accidental corruption and malicious tampering.
*   **Enhanced Legal Defensibility** through cryptographic signing and a formalized `ModelContainer` structure.

This strategy ensures that Deep Causality can meet the stringent requirements of safety-critical and regulated industries, providing a foundation for auditable, certifiable, and trustworthy causal reasoning systems.
