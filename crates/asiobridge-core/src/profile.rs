use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

/// A saved profile — collection of racks, connections, and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub racks: HashMap<String, serde_json::Value>,
    pub connections: Vec<serde_json::Value>,
    pub global_settings: HashMap<String, serde_json::Value>,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            racks: HashMap::new(),
            connections: Vec::new(),
            global_settings: HashMap::new(),
        }
    }
}

/// Manages 8 profile slots persisted to disk as JSON files
pub struct ProfileManager {
    dir: PathBuf,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        let dir = PathBuf::from("profiles");
        fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    fn slot_path(&self, slot: usize) -> PathBuf {
        self.dir.join(format!("slot_{}.json", slot))
    }

    pub fn save(&mut self, slot: usize, profile: Profile) -> bool {
        if slot >= 8 {
            error!("Invalid profile slot: {}", slot);
            return false;
        }

        let path = self.slot_path(slot);
        match fs::write(
            &path,
            serde_json::to_string_pretty(&profile).unwrap_or_default(),
        ) {
            Ok(_) => {
                info!(
                    "Saved profile '{}' to slot {} ({})",
                    profile.name,
                    slot,
                    path.display()
                );
                true
            }
            Err(e) => {
                error!("Failed to save profile: {}", e);
                false
            }
        }
    }

    pub fn load(&self, slot: usize) -> Option<Profile> {
        if slot >= 8 {
            return None;
        }

        let path = self.slot_path(slot);
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Profile>(&content) {
                Ok(profile) => {
                    info!(
                        "Loaded profile '{}' from slot {} ({})",
                        profile.name,
                        slot,
                        path.display()
                    );
                    Some(profile)
                }
                Err(e) => {
                    error!("Failed to deserialize profile from slot {}: {}", slot, e);
                    None
                }
            },
            Err(_) => {
                // No profile in this slot
                None
            }
        }
    }

    pub fn delete_slot(&mut self, slot: usize) -> bool {
        if slot >= 8 {
            return false;
        }

        let path = self.slot_path(slot);
        match fs::remove_file(&path) {
            Ok(_) => {
                info!("Deleted profile from slot {}", slot);
                true
            }
            Err(e) => {
                error!("Failed to delete profile slot: {}", e);
                false
            }
        }
    }

    pub fn has_profile(&self, slot: usize) -> bool {
        if slot >= 8 {
            return false;
        }
        self.slot_path(slot).exists()
    }

    pub fn list_profiles(&self) -> Vec<(usize, String)> {
        let mut profiles = Vec::new();
        for i in 0..8 {
            let path = self.slot_path(i);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        profiles.push((i, profile.name));
                    }
                }
            }
        }
        profiles
    }
}
