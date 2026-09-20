//! Application service coordinating discovery, domain state and Windows I/O.
//!
//! The UI never owns routing intent. This runtime keeps one `RouterEngine`
//! alive across observations and persists accepted commands before a later
//! refresh can rebuild presentation data.

use std::collections::{BTreeMap, BTreeSet};

use router_engine::{ApplicationId, Command, OutputId, RouterEngine};

use crate::audio::WindowsObservation;
use crate::route_store::RouteStore;
use crate::routing;
use crate::ui::{PopupModel, RouteRequest, VolumeRequest};
use crate::volume;

pub struct ApplicationRuntime {
    engine: RouterEngine,
    routes: RouteStore,
    observed_processes: BTreeMap<String, BTreeSet<u32>>,
}

impl ApplicationRuntime {
    pub fn start(observed: WindowsObservation) -> Result<(Self, PopupModel), String> {
        let engine = RouterEngine::new(observed.observation.clone());
        let routes = RouteStore::load_default()?;
        let mut runtime = Self {
            engine,
            routes,
            observed_processes: BTreeMap::new(),
        };
        runtime.restore_and_reapply(&observed, true);
        let model = runtime.model(&observed);
        Ok((runtime, model))
    }

    pub fn refresh(&mut self, observed: WindowsObservation) -> PopupModel {
        self.engine.observe(observed.observation.clone());
        self.restore_and_reapply(&observed, false);
        self.model(&observed)
    }

    pub fn route(&mut self, request: RouteRequest) -> Result<(), String> {
        routing::route_processes(&request.process_ids, request.output_id.as_deref())
            .map_err(|error| error.to_string())?;

        let application = ApplicationId::new(request.application_id.clone())
            .map_err(|error| error.to_string())?;
        let command = match request.output_id.as_deref() {
            Some(output_id) => Command::SetRoute {
                application,
                output: OutputId::new(output_id.to_owned()).map_err(|error| error.to_string())?,
            },
            None => Command::FollowSystemDefault { application },
        };
        self.engine
            .dispatch(command)
            .map_err(|error| error.to_string())?;
        self.routes
            .record(&request.application_id, request.output_id.as_deref())?;

        self.observed_processes.insert(
            request.application_id,
            request.process_ids.into_iter().collect(),
        );
        Ok(())
    }

    pub fn set_volume(&mut self, request: VolumeRequest) -> Result<(), String> {
        volume::set_for_processes(&request.process_ids, request.volume_percent, request.muted)
    }

    fn restore_and_reapply(&mut self, observed: &WindowsObservation, initial: bool) {
        let mut current_processes = BTreeMap::new();

        for application in &observed.observation.applications {
            let application_id = application.as_str();
            let process_ids = observed.stats.processes_for(application_id);
            let process_set: BTreeSet<_> = process_ids.iter().copied().collect();
            let first_observation = !self.observed_processes.contains_key(application_id);

            // Import an existing Windows selection only when Adufa has no saved
            // intent. The private getter may temporarily report no value, so it
            // must never erase a route that Adufa already persisted.
            if self.routes.output_for(application_id).is_none()
                && let Ok(Some(output_id)) = routing::persisted_output_for_processes(&process_ids)
                && let Err(error) = self.routes.record(application_id, Some(&output_id))
            {
                eprintln!("Could not import the Windows audio route: {error}");
            }

            let saved_output = self.routes.output_for(application_id).map(str::to_owned);
            if let Some(output_id) = saved_output.as_deref() {
                let command = OutputId::new(output_id.to_owned()).map(|output| Command::SetRoute {
                    application: application.clone(),
                    output,
                });
                if let Ok(command) = command {
                    let _ = self.engine.dispatch(command);
                }

                let previous = self.observed_processes.get(application_id);
                let new_processes: Vec<_> = process_ids
                    .iter()
                    .copied()
                    .filter(|process_id| {
                        initial
                            || first_observation
                            || previous.is_none_or(|known| !known.contains(process_id))
                    })
                    .collect();
                if !new_processes.is_empty()
                    && let Err(error) = routing::route_processes(&new_processes, Some(output_id))
                {
                    eprintln!("Could not restore a saved audio route: {error}");
                }
            }

            current_processes.insert(application_id.to_owned(), process_set);
        }

        self.observed_processes = current_processes;
    }

    fn model(&self, observed: &WindowsObservation) -> PopupModel {
        PopupModel::from_observation(&self.engine.snapshot(), &observed.stats, &observed.outputs)
    }
}
