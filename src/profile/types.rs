use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlideProfile {
    pub name: Option<String>,
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Waypoint {
    pub x: f32,
    pub y: f32,
}

impl GlideProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.waypoints.is_empty() {
            return Err("profile has no waypoints".to_string());
        }
        Ok(())
    }
}
