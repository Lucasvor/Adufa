use std::collections::{BTreeMap, BTreeSet};

use crate::identity::{ApplicationId, OutputId};
use crate::model::{
    ApplicationSnapshot, Capabilities, CapabilityAvailability, Command, CommandError, Notice,
    Observation, RouteFailure, RouteStatus, Snapshot,
};

/// Owns routing intent and reconciles it against complete platform observations.
///
/// Calls are synchronous and must be serialized by the future engine worker. The
/// engine performs no I/O and retains no native handles. Repeating an equivalent
/// observation is idempotent and emits no duplicate failure notice.
pub struct RouterEngine {
    capabilities: Capabilities,
    active_applications: BTreeSet<ApplicationId>,
    outputs: BTreeSet<OutputId>,
    system_default: Option<OutputId>,
    routes: BTreeMap<ApplicationId, OutputId>,
    failure_episodes: BTreeMap<ApplicationId, RouteFailure>,
}

impl RouterEngine {
    /// Creates an engine from the platform's initial complete observation.
    #[must_use]
    pub fn new(observation: Observation) -> Self {
        Self {
            capabilities: observation.capabilities,
            active_applications: observation.applications.into_iter().collect(),
            outputs: observation.outputs.into_iter().collect(),
            system_default: observation.system_default,
            routes: BTreeMap::new(),
            failure_episodes: BTreeMap::new(),
        }
    }

    /// Replaces the previous platform observation and reconciles saved routes.
    ///
    /// The observation must be complete, not a delta. Returned notices are ordered
    /// by application identity and contain only newly started failure episodes.
    pub fn observe(&mut self, observation: Observation) -> Vec<Notice> {
        self.capabilities = observation.capabilities;
        self.active_applications = observation.applications.into_iter().collect();
        self.outputs = observation.outputs.into_iter().collect();
        self.system_default = observation.system_default;
        self.reconcile_failures()
    }

    /// Applies one user intention and returns newly started failure episodes.
    ///
    /// `SetRoute` is rejected before mutation unless application routing is
    /// available. `FollowSystemDefault` always succeeds because it only removes
    /// stored intent; it does not claim a platform routing operation succeeded.
    pub fn dispatch(&mut self, command: Command) -> Result<Vec<Notice>, CommandError> {
        match command {
            Command::SetRoute {
                application,
                output,
            } => {
                if self.capabilities.application_routing != CapabilityAvailability::Available {
                    return Err(CommandError::ApplicationRoutingUnavailable(
                        self.capabilities.application_routing,
                    ));
                }
                self.routes.insert(application, output);
            }
            Command::FollowSystemDefault { application } => {
                self.routes.remove(&application);
                self.failure_episodes.remove(&application);
            }
        }
        Ok(self.reconcile_failures())
    }

    /// Returns an owned snapshot independent of future engine mutations.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        let mut application_ids = self.active_applications.clone();
        application_ids.extend(self.routes.keys().cloned());

        let mut applications: Vec<_> = application_ids
            .into_iter()
            .map(|id| self.application_snapshot(id))
            .collect();
        applications.sort_by(|left, right| {
            right
                .active
                .cmp(&left.active)
                .then_with(|| left.id.cmp(&right.id))
        });

        Snapshot {
            capabilities: self.capabilities,
            applications,
            outputs: self.outputs.iter().cloned().collect(),
            system_default: self.system_default.clone(),
        }
    }

    fn application_snapshot(&self, id: ApplicationId) -> ApplicationSnapshot {
        let active = self.active_applications.contains(&id);
        let desired_output = self.routes.get(&id).cloned();

        let (effective_output, route_status) = if !active {
            (None, RouteStatus::Inactive)
        } else if desired_output.is_none() {
            (
                self.system_default.clone(),
                RouteStatus::FollowingSystemDefault,
            )
        } else {
            let (effective, failure) = self.effective_route(&id);
            let status = failure.map_or(RouteStatus::Applied, RouteStatus::Fallback);
            (effective, status)
        };

        ApplicationSnapshot {
            id,
            active,
            desired_output,
            effective_output,
            route_status,
        }
    }

    fn effective_route(
        &self,
        application: &ApplicationId,
    ) -> (Option<OutputId>, Option<RouteFailure>) {
        let desired = self
            .routes
            .get(application)
            .expect("effective_route is called only for configured applications");

        let failure = match self.capabilities.application_routing {
            CapabilityAvailability::Available if self.outputs.contains(desired) => None,
            CapabilityAvailability::Available => {
                Some(RouteFailure::DesiredOutputUnavailable(desired.clone()))
            }
            CapabilityAvailability::Unavailable(reason) => {
                Some(RouteFailure::RoutingUnavailable(reason))
            }
            CapabilityAvailability::PermissionRequired => Some(RouteFailure::PermissionRequired),
        };

        if failure.is_some() {
            (self.system_default.clone(), failure)
        } else {
            (Some(desired.clone()), None)
        }
    }

    fn reconcile_failures(&mut self) -> Vec<Notice> {
        let current: BTreeMap<_, _> = self
            .active_applications
            .iter()
            .filter_map(|application| {
                self.routes.get(application)?;
                self.effective_route(application)
                    .1
                    .map(|failure| (application.clone(), failure))
            })
            .collect();

        let notices = current
            .iter()
            .filter(|(application, failure)| {
                self.failure_episodes.get(*application) != Some(*failure)
            })
            .map(|(application, reason)| Notice::RouteFailed {
                application: application.clone(),
                reason: reason.clone(),
            })
            .collect();
        self.failure_episodes = current;
        notices
    }
}
