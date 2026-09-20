//! Platform-independent routing state and policy.
//!
//! Platform adapters provide stable identities and complete observations. The
//! [`RouterEngine`] owns persistent route intent, fallback semantics, and
//! failure-episode deduplication; it never receives native handles or performs
//! I/O. Callers serialize mutations and consume owned [`Snapshot`] values.

#![forbid(unsafe_code)]

mod engine;
mod identity;
mod model;

pub use engine::RouterEngine;
pub use identity::{ApplicationId, IdentifierError, OutputId};
pub use model::{
    ApplicationSnapshot, Capabilities, CapabilityAvailability, Command, CommandError, Notice,
    Observation, RouteFailure, RouteStatus, Snapshot, UnavailableReason,
};
