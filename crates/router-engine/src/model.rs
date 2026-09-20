use std::error::Error;
use std::fmt;

use crate::identity::{ApplicationId, OutputId};

/// A stable reason why a capability is not currently available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnavailableReason {
    /// The platform cannot provide the operation reliably.
    UnsupportedByPlatform,
    /// The platform backend is temporarily unavailable.
    BackendUnavailable,
}

/// Runtime availability of one platform operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityAvailability {
    /// The operation may be requested.
    Available,
    /// The operation cannot currently be provided.
    Unavailable(UnavailableReason),
    /// The operation requires user-granted operating-system permission.
    PermissionRequired,
}

/// Platform operations understood by this engine slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capabilities {
    /// Whether persistent application-level output routing can be requested.
    pub application_routing: CapabilityAvailability,
}

/// A complete point-in-time observation from a platform audio adapter.
///
/// Applications are already grouped into stable application identities. Duplicate
/// identities are harmless and removed. The engine takes ownership of all values;
/// observations contain no borrowed or native handles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    /// Capabilities that may change after permissions or backend recovery.
    pub capabilities: Capabilities,
    /// Applications that currently own observable audio streams.
    pub applications: Vec<ApplicationId>,
    /// Output devices currently reported by the operating system.
    pub outputs: Vec<OutputId>,
    /// The current system default, if the platform reports one.
    pub system_default: Option<OutputId>,
}

/// A user intention accepted by the routing engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    /// Persistently route the application's current and future streams.
    SetRoute {
        /// The stable grouped application identity.
        application: ApplicationId,
        /// The exact output identity to retain through temporary disconnection.
        output: OutputId,
    },
    /// Remove the persistent route so the application tracks the system default.
    FollowSystemDefault {
        /// The stable grouped application identity.
        application: ApplicationId,
    },
}

/// A command rejected before any routing state is mutated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
    /// Persistent application routing is not currently available.
    ApplicationRoutingUnavailable(CapabilityAvailability),
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApplicationRoutingUnavailable(availability) => {
                write!(
                    formatter,
                    "application routing is unavailable: {availability:?}"
                )
            }
        }
    }
}

impl Error for CommandError {}

/// Why an active application is temporarily falling back to the system default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteFailure {
    /// The exact saved output is not present in the latest observation.
    DesiredOutputUnavailable(OutputId),
    /// The platform reports that application routing is unsupported.
    RoutingUnavailable(UnavailableReason),
    /// The platform requires user permission before it can route applications.
    PermissionRequired,
}

/// A quiet user-facing event emitted once per distinct failure episode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Notice {
    /// A saved route cannot currently be applied. The desired route remains stored.
    RouteFailed {
        /// The active application whose route entered a failure episode.
        application: ApplicationId,
        /// The structured reason to localize and present outside the engine.
        reason: RouteFailure,
    },
}

/// Current routing status for one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteStatus {
    /// The active application has no saved route and follows the system default.
    FollowingSystemDefault,
    /// The exact desired output is present and routing is available.
    Applied,
    /// The desired route remains stored while the active application uses fallback.
    Fallback(RouteFailure),
    /// The configured application currently has no observed audio streams.
    Inactive,
}

/// Immutable application state owned by a [`Snapshot`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSnapshot {
    /// The stable grouped application identity.
    pub id: ApplicationId,
    /// Whether the latest observation contains audio streams for this application.
    pub active: bool,
    /// The stored exact output, or `None` when following the system default.
    pub desired_output: Option<OutputId>,
    /// The output currently selected by policy, or `None` while inactive or unknown.
    pub effective_output: Option<OutputId>,
    /// Whether the saved route is applied, following default, inactive, or failing.
    pub route_status: RouteStatus,
}

/// An owned, immutable view for shells and adapters.
///
/// Active applications come first, followed by configured inactive applications;
/// each section is ordered by stable identity. Outputs are ordered by stable
/// identity. A snapshot remains valid after later engine mutations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Runtime platform capabilities used to validate new commands.
    pub capabilities: Capabilities,
    /// Active-first application state with configured inactive applications retained.
    pub applications: Vec<ApplicationSnapshot>,
    /// Currently observed outputs, ordered by stable identity.
    pub outputs: Vec<OutputId>,
    /// The latest platform-reported system default, if known.
    pub system_default: Option<OutputId>,
}
