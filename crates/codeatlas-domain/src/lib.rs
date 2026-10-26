//! CodeAtlas Pure Domain Layer
//!
//! Enforces Clean Architecture & Domain-Driven Design (DDD).
//! Contains pure business entities, value objects, domain errors, and abstract port traits.
//! Zero dependencies on external transport, network, or low-level storage frameworks.

pub mod entities;
pub mod ports;
pub mod value_objects;

pub use entities::*;
pub use ports::*;
pub use value_objects::*;
