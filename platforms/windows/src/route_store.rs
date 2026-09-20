//! Versioned per-user configuration for persistent application routes.
//!
//! Windows' private audio-policy getter can lag behind process creation. The
//! stable application identity stored here is therefore Adufa's source of truth;
//! the native getter is used to migrate preferences created outside Adufa.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};
use windows::core::HSTRING;

const CONFIG_VERSION: u32 = 1;

#[derive(Default, Deserialize, Serialize)]
struct UserConfiguration {
    version: u32,
    #[serde(default)]
    routes: BTreeMap<String, String>,
}

pub struct RouteStore {
    path: PathBuf,
    routes: BTreeMap<String, String>,
}

impl RouteStore {
    pub fn load_default() -> Result<Self, String> {
        let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
            "Windows did not provide the local application-data folder.".to_owned()
        })?;
        Self::load(
            PathBuf::from(local_app_data)
                .join("Adufa")
                .join("config.json"),
        )
    }

    fn load(path: PathBuf) -> Result<Self, String> {
        let backup = backup_path(&path);
        let configuration = match read_configuration(&path) {
            Ok(configuration) => configuration,
            Err(primary_error) => match read_configuration(&backup) {
                Ok(configuration) => configuration,
                Err(_) if primary_error.kind() == std::io::ErrorKind::NotFound => {
                    UserConfiguration::default()
                }
                Err(_) => {
                    return Err(format!(
                        "Adufa could not read a valid configuration: {primary_error}"
                    ));
                }
            },
        };

        if configuration.version != 0 && configuration.version != CONFIG_VERSION {
            return Err(format!(
                "Adufa configuration version {} is not supported.",
                configuration.version
            ));
        }

        Ok(Self {
            path,
            routes: configuration.routes,
        })
    }

    pub fn output_for(&self, application_id: &str) -> Option<&str> {
        self.routes.get(application_id).map(String::as_str)
    }

    /// Records a successful routing request. `None` means follow system default.
    pub fn record(&mut self, application_id: &str, output_id: Option<&str>) -> Result<(), String> {
        match output_id {
            Some(output_id) => {
                self.routes
                    .insert(application_id.to_owned(), output_id.to_owned());
            }
            None => {
                self.routes.remove(application_id);
            }
        }
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "The configuration path has no parent folder.".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Adufa could not create its settings folder: {error}"))?;

        let next = next_path(&self.path);
        let configuration = UserConfiguration {
            version: CONFIG_VERSION,
            routes: self.routes.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&configuration)
            .map_err(|error| format!("Adufa could not encode its configuration: {error}"))?;
        let mut file = File::create(&next)
            .map_err(|error| format!("Adufa could not stage its configuration: {error}"))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("Adufa could not flush its configuration: {error}"))?;
        drop(file);

        if self.path.exists() {
            replace_with_backup(&self.path, &next, &backup_path(&self.path))
        } else {
            fs::rename(&next, &self.path)
                .map_err(|error| format!("Adufa could not publish its configuration: {error}"))
        }
    }
}

fn read_configuration(path: &Path) -> std::io::Result<UserConfiguration> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn next_path(path: &Path) -> PathBuf {
    path.with_extension("next.json")
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("previous.json")
}

fn replace_with_backup(current: &Path, next: &Path, backup: &Path) -> Result<(), String> {
    fs::copy(current, backup)
        .and_then(|_| File::options().write(true).open(backup)?.sync_all())
        .map_err(|error| format!("Adufa could not retain its previous configuration: {error}"))?;
    let current = HSTRING::from(current);
    let next = HSTRING::from(next);
    unsafe {
        MoveFileExW(
            &next,
            &current,
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|error| format!("Adufa could not atomically replace its configuration: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_store_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("adufa-routes-{}-{nonce}.json", std::process::id()))
    }

    fn clean(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(next_path(path));
        let _ = fs::remove_file(backup_path(path));
    }

    #[test]
    fn route_survives_loading_a_new_store_instance() {
        let path = temporary_store_path();
        let mut first = RouteStore::load(path.clone()).expect("temporary store should load");
        first
            .record("windows:path:C:\\Apps\\Player.exe", Some("device-id"))
            .expect("route should save");

        let second = RouteStore::load(path.clone()).expect("saved store should reload");
        assert_eq!(
            second.output_for("windows:path:C:\\Apps\\Player.exe"),
            Some("device-id")
        );
        clean(&path);
    }

    #[test]
    fn following_system_default_removes_the_saved_route() {
        let path = temporary_store_path();
        let mut store = RouteStore::load(path.clone()).expect("temporary store should load");
        store
            .record("windows:aumid:Example!Player", Some("headset"))
            .expect("route should save");
        store
            .record("windows:aumid:Example!Player", None)
            .expect("route should clear");

        let reloaded = RouteStore::load(path.clone()).expect("saved store should reload");
        assert_eq!(reloaded.output_for("windows:aumid:Example!Player"), None);
        clean(&path);
    }

    #[test]
    fn corrupt_primary_falls_back_to_the_previous_valid_copy() {
        let path = temporary_store_path();
        let mut store = RouteStore::load(path.clone()).expect("temporary store should load");
        store
            .record("app", Some("first"))
            .expect("first route should save");
        store
            .record("app", Some("second"))
            .expect("second route should save with backup");
        fs::write(&path, b"not json").expect("primary should be corruptible");

        let recovered = RouteStore::load(path.clone()).expect("backup should recover");
        assert_eq!(recovered.output_for("app"), Some("first"));
        clean(&path);
    }
}
