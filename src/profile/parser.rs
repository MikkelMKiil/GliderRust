use quick_xml::de::from_str;
use serde::Deserialize;
use thiserror::Error;

use crate::profile::types::{GlideProfile, Waypoint};

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("failed to read profile file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse XML: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("profile validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename = "GlideProfile")]
struct RawGlideProfile {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "MinLevel")]
    min_level: Option<String>,
    #[serde(rename = "MaxLevel")]
    max_level: Option<String>,
    #[serde(rename = "Waypoint", default)]
    waypoints: Vec<String>,
}

pub fn load_profile(path: &str) -> Result<GlideProfile, ProfileError> {
    let xml = std::fs::read_to_string(path)?;
    let raw: RawGlideProfile = from_str(&xml)?;

    let waypoints = raw
        .waypoints
        .iter()
        .filter_map(|line| parse_waypoint_line(line))
        .collect::<Vec<_>>();

    let profile = GlideProfile {
        name: raw.name,
        min_level: raw.min_level.as_deref().and_then(parse_u8),
        max_level: raw.max_level.as_deref().and_then(parse_u8),
        waypoints,
    };

    profile
        .validate()
        .map_err(ProfileError::Validation)
        .map(|_| profile)
}

fn parse_u8(input: &str) -> Option<u8> {
    input.trim().parse::<u8>().ok()
}

fn parse_waypoint_line(input: &str) -> Option<Waypoint> {
    let normalized = input.replace(',', " ");
    let mut parts = normalized.split_whitespace();

    let x = parts.next()?.parse::<f32>().ok()?;
    let y = parts.next()?.parse::<f32>().ok()?;

    Some(Waypoint { x, y })
}
