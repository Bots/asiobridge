use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Manages 8 profile slots (matching ASIO Link Pro behavior)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    pub slots: Vec<Option<Profile>>,
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            slots: vec![None; 8],
        }
    }

    pub fn save(&mut self, slot: usize, profile: Profile) {
        if slot < 8 {
            self.slots[slot] = Some(profile);
        }
    }

    pub fn load(&self, slot: usize) -> Option<&Profile> {
        self.slots.get(slot).and_then(|s| s.as_ref())
    }

    pub fn clear_slot(&mut self, slot: usize) {
        if slot < 8 {
            self.slots[slot] = None;
        }
    }
}
