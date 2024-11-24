//! Producer module for the ring buffer implementation.
//!
//! This module provides two types of producers for the ring buffer:
//! - Single Producer: Optimized for scenarios where only one thread produces events
//! - Multi Producer: Designed for concurrent event production from multiple threads
//!
//! The producers manage sequence generation and coordinate with consumers through
//! gating sequences to ensure proper event processing order.

pub(crate) mod multi_producer;
pub(crate) mod single_producer;
