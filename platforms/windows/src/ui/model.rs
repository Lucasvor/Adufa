use router_engine::Snapshot;

use crate::audio::{ScanStats, WindowsOutput};

const MAX_VISIBLE_APPLICATIONS: usize = 7;

/// Owned presentation data consumed by the native popup.
///
/// Native handles deliberately stay out of this model. The UI may be rebuilt
/// after an automatic audio observation without retaining COM or process handles.
pub struct PopupModel {
    pub applications: Vec<ApplicationRow>,
    pub outputs: Vec<OutputChoice>,
}

/// One grouped application row in the popup.
pub struct ApplicationRow {
    /// Stable engine identity used by the routing boundary.
    pub application_id: String,
    pub name: String,
    /// Optional executable path used only to ask Windows for the real app icon.
    pub icon_path: Option<String>,
    /// Every active process represented by this grouped application.
    pub process_ids: Vec<u32>,
    /// `None` means the application follows the system default.
    pub selected_output_id: Option<String>,
    pub volume_percent: u8,
    pub muted: bool,
}

/// A concrete output displayed in an application's native submenu.
pub struct OutputChoice {
    pub id: String,
    pub name: String,
    pub is_system_default: bool,
}

/// Owned request passed from presentation code to the Windows routing adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteRequest {
    pub application_id: String,
    pub process_ids: Vec<u32>,
    /// `None` asks Windows to make the application follow the system default.
    pub output_id: Option<String>,
}

/// Owned request passed from presentation code to the Windows session-volume adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeRequest {
    pub application_id: String,
    pub process_ids: Vec<u32>,
    pub volume_percent: u8,
    pub muted: bool,
}

impl PopupModel {
    pub fn upsert_application(
        &mut self,
        application_id: String,
        name: String,
        icon_path: Option<String>,
        process_ids: Vec<u32>,
    ) -> usize {
        if let Some((index, application)) = self
            .applications
            .iter_mut()
            .enumerate()
            .find(|(_, application)| application.application_id == application_id)
        {
            application.name = name;
            application.icon_path = icon_path;
            application.process_ids = process_ids;
            return index;
        }

        self.applications.push(ApplicationRow {
            application_id,
            name,
            icon_path,
            process_ids,
            selected_output_id: None,
            volume_percent: 100,
            muted: false,
        });
        self.applications.len() - 1
    }

    pub fn from_observation(
        snapshot: &Snapshot,
        stats: &ScanStats,
        outputs: &[WindowsOutput],
    ) -> Self {
        let applications = snapshot
            .applications
            .iter()
            .take(MAX_VISIBLE_APPLICATIONS)
            .map(|application| {
                let volume = stats.volume_for(application.id.as_str());
                ApplicationRow {
                    volume_percent: volume.map_or(100, |volume| volume.percent),
                    muted: volume.is_some_and(|volume| volume.muted),
                    application_id: application.id.as_str().to_owned(),
                    name: display_name(application.id.as_str()),
                    icon_path: stats
                        .executable_path_for(application.id.as_str())
                        .map(str::to_owned)
                        .or_else(|| executable_path_from_identity(application.id.as_str())),
                    process_ids: stats.processes_for(application.id.as_str()),
                    selected_output_id: application
                        .desired_output
                        .as_ref()
                        .map(|output| output.as_str().to_owned()),
                }
            })
            .collect();
        let mut outputs: Vec<_> = outputs
            .iter()
            .map(|output| OutputChoice {
                id: output.id.as_str().to_owned(),
                name: output.name.clone(),
                is_system_default: output.is_default,
            })
            .collect();
        // The synthetic "System default" action is always first in the native
        // menu; immediately after it, surface the device Windows currently uses.
        outputs.sort_by_key(|output| !output.is_system_default);

        Self {
            applications,
            outputs,
        }
    }

    pub fn destination_name(&self, application_index: usize) -> &str {
        let Some(application) = self.applications.get(application_index) else {
            return "System default";
        };
        let Some(selected) = application.selected_output_id.as_deref() else {
            return "System default";
        };
        self.outputs
            .iter()
            .find(|output| output.id == selected)
            .map_or("Unavailable output", |output| output.name.as_str())
    }

    pub fn route_request(
        &self,
        application_index: usize,
        output_id: Option<String>,
    ) -> Option<RouteRequest> {
        let application = self.applications.get(application_index)?;
        Some(RouteRequest {
            application_id: application.application_id.clone(),
            process_ids: application.process_ids.clone(),
            output_id,
        })
    }

    pub fn volume_request(
        &self,
        application_index: usize,
        volume_percent: u8,
        muted: bool,
    ) -> Option<VolumeRequest> {
        let application = self.applications.get(application_index)?;
        Some(VolumeRequest {
            application_id: application.application_id.clone(),
            process_ids: application.process_ids.clone(),
            volume_percent: volume_percent.min(100),
            muted,
        })
    }
}

fn executable_path_from_identity(identity: &str) -> Option<String> {
    identity.strip_prefix("windows:path:").map(str::to_owned)
}

fn display_name(identity: &str) -> String {
    if identity == "windows:system-sounds" {
        return "System sounds".to_owned();
    }

    if let Some(path) = identity.strip_prefix("windows:path:") {
        let file_name = path.rsplit(['\\', '/']).next().unwrap_or(path);
        return file_name
            .strip_suffix(".exe")
            .or_else(|| file_name.strip_suffix(".EXE"))
            .unwrap_or(file_name)
            .to_owned();
    }

    if let Some(aumid) = identity.strip_prefix("windows:aumid:") {
        return aumid
            .rsplit(['!', '.'])
            .find(|part| !part.is_empty())
            .unwrap_or("Application")
            .to_owned();
    }

    "Application".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_identity_becomes_a_human_label() {
        assert_eq!(
            display_name(r"windows:path:C:\Apps\Spotify\Spotify.exe"),
            "Spotify"
        );
    }

    #[test]
    fn executable_identity_also_supplies_an_icon_source() {
        assert_eq!(
            executable_path_from_identity(r"windows:path:C:\Apps\Spotify.exe").as_deref(),
            Some(r"C:\Apps\Spotify.exe")
        );
    }

    #[test]
    fn packaged_identity_uses_its_application_segment() {
        assert_eq!(
            display_name("windows:aumid:Publisher.Product_123!Player"),
            "Player"
        );
    }

    #[test]
    fn system_sounds_has_a_stable_label() {
        assert_eq!(display_name("windows:system-sounds"), "System sounds");
    }

    #[test]
    fn taskbar_application_can_be_added_without_audio_session() {
        let mut model = PopupModel {
            applications: Vec::new(),
            outputs: Vec::new(),
        };

        let index = model.upsert_application(
            r"windows:path:C:\Apps\Player.exe".to_owned(),
            "Player".to_owned(),
            Some(r"C:\Apps\Player.exe".to_owned()),
            vec![42],
        );

        assert_eq!(index, 0);
        assert_eq!(model.applications[0].process_ids, [42]);
        assert_eq!(model.applications[0].volume_percent, 100);
    }
}
