pub mod state;

use serde::{Deserialize, Serialize};

use crate::memory::types::MemorySnapshot;
use crate::profile::GlideProfile;
use crate::input::InputCommand;

pub use state::{BotState, RuntimeStatus};

const WAYPOINT_RADIUS: f32 = 2.0;
const HEADING_TURN_THRESHOLD_RAD: f32 = 0.25;
const MOVE_FORWARD_DISTANCE_MIN: f32 = 1.5;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationOutput {
    pub desired_heading_rad: Option<f32>,
    pub heading_error_rad: Option<f32>,
    pub distance_to_waypoint: Option<f32>,
    pub waypoint_reached: bool,
}

#[derive(Debug)]
pub struct BotRuntime {
    state: BotState,
    status: RuntimeStatus,
    active_profile: Option<GlideProfile>,
    current_waypoint_index: usize,
    nav_output: NavigationOutput,
}

impl Default for BotRuntime {
    fn default() -> Self {
        Self {
            state: BotState::Stopped,
            status: RuntimeStatus::Idle,
            active_profile: None,
            current_waypoint_index: 0,
            nav_output: NavigationOutput::default(),
        }
    }
}

impl BotRuntime {
    pub fn state(&self) -> BotState {
        self.state
    }

    pub fn status(&self) -> RuntimeStatus {
        self.status
    }

    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile
            .as_ref()
            .and_then(|profile| profile.name.as_deref())
    }

    pub fn waypoint_progress(&self) -> Option<(usize, usize)> {
        self.active_profile
            .as_ref()
            .map(|profile| (self.current_waypoint_index + 1, profile.waypoints.len()))
    }

    pub fn nav_output(&self) -> &NavigationOutput {
        &self.nav_output
    }

    pub fn suggested_inputs(&self) -> Vec<InputCommand> {
        let mut commands = Vec::new();

        if self.state != BotState::Running || self.status != RuntimeStatus::RunningProfile {
            return commands;
        }

        if let Some(heading_error) = self.nav_output.heading_error_rad {
            if heading_error > HEADING_TURN_THRESHOLD_RAD {
                commands.push(InputCommand::TurnLeftMouse);
            } else if heading_error < -HEADING_TURN_THRESHOLD_RAD {
                commands.push(InputCommand::TurnRightMouse);
            }
        }

        if let Some(distance) = self.nav_output.distance_to_waypoint {
            if distance > MOVE_FORWARD_DISTANCE_MIN {
                commands.push(InputCommand::MoveForward(crate::input::KeyAction::Press));
            } else {
                commands.push(InputCommand::MoveForward(crate::input::KeyAction::Release));
            }
        }

        commands
    }

    pub fn set_profile(&mut self, profile: GlideProfile) {
        self.active_profile = Some(profile);
        self.current_waypoint_index = 0;
        self.nav_output = NavigationOutput::default();
    }

    pub fn clear_profile(&mut self) {
        self.active_profile = None;
        self.current_waypoint_index = 0;
        self.nav_output = NavigationOutput::default();
    }

    pub fn start(&mut self) {
        self.state = BotState::Running;
        self.status = RuntimeStatus::PollingMemory;
    }

    pub fn pause(&mut self) {
        self.state = BotState::Paused;
        self.status = RuntimeStatus::Paused;
    }

    pub fn stop(&mut self) {
        self.state = BotState::Stopped;
        self.status = RuntimeStatus::Idle;
        self.nav_output = NavigationOutput::default();
    }

    pub fn tick(&mut self, snapshot: Option<&MemorySnapshot>) {
        if self.state != BotState::Running {
            return;
        }

        let Some(snapshot) = snapshot else {
            self.status = RuntimeStatus::PollingMemory;
            self.nav_output = NavigationOutput::default();
            return;
        };

        let Some(profile) = self.active_profile.as_ref() else {
            self.status = RuntimeStatus::PollingMemory;
            self.nav_output = NavigationOutput::default();
            return;
        };

        let Some((px, py, _pz)) = snapshot.position else {
            self.status = RuntimeStatus::PollingMemory;
            self.nav_output = NavigationOutput::default();
            return;
        };

        if profile.waypoints.is_empty() {
            self.status = RuntimeStatus::PollingMemory;
            self.nav_output = NavigationOutput::default();
            return;
        }

        if self.current_waypoint_index >= profile.waypoints.len() {
            self.current_waypoint_index = 0;
        }

        let waypoint = &profile.waypoints[self.current_waypoint_index];
        let dx = waypoint.x - px;
        let dy = waypoint.y - py;

        let distance = (dx * dx + dy * dy).sqrt();
        let desired_heading = dy.atan2(dx);
        let heading_error = snapshot
            .heading_rad
            .map(|current_heading| normalize_angle(desired_heading - current_heading));

        let waypoint_reached = distance <= WAYPOINT_RADIUS;
        if waypoint_reached {
            self.current_waypoint_index = (self.current_waypoint_index + 1) % profile.waypoints.len();
        }

        self.nav_output = NavigationOutput {
            desired_heading_rad: Some(desired_heading),
            heading_error_rad: heading_error,
            distance_to_waypoint: Some(distance),
            waypoint_reached,
        };

        self.status = RuntimeStatus::RunningProfile;
    }
}

fn normalize_angle(mut angle_rad: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;
    const TWO_PI: f32 = std::f32::consts::TAU;

    while angle_rad > PI {
        angle_rad -= TWO_PI;
    }

    while angle_rad < -PI {
        angle_rad += TWO_PI;
    }

    angle_rad
}

#[cfg(test)]
mod tests {
    use super::{normalize_angle, BotRuntime};
    use crate::memory::MemorySnapshot;
    use crate::profile::{GlideProfile, Waypoint};

    #[test]
    fn normalizes_heading_delta_range() {
        let wrapped_positive = normalize_angle(4.5);
        assert!(wrapped_positive <= std::f32::consts::PI);

        let wrapped_negative = normalize_angle(-4.5);
        assert!(wrapped_negative >= -std::f32::consts::PI);
    }

    #[test]
    fn advances_waypoint_when_inside_radius() {
        let profile = GlideProfile {
            name: Some("Test".to_string()),
            min_level: None,
            max_level: None,
            waypoints: vec![
                Waypoint { x: 10.0, y: 10.0 },
                Waypoint { x: 20.0, y: 20.0 },
            ],
        };

        let mut runtime = BotRuntime::default();
        runtime.set_profile(profile);
        runtime.start();

        let snapshot = MemorySnapshot {
            player_name: "TestPlayer".to_string(),
            player_guid: Some(1),
            player_object_type: None,
            player_current_health: Some(100),
            player_max_health: Some(100),
            player_race: Some(1),
            player_level: Some(1),
            player_faction: Some(1),
            player_unit_flags: None,
            player_monster_definition_ptr: None,
            player_display_id: None,
            player_native_display_id: None,
            position: Some((10.5, 10.5, 0.0)),
            heading_rad: Some(0.0),
            target_guid: None,
            target_object_type: None,
            target_name: None,
            target_current_health: None,
            target_max_health: None,
            target_race: None,
            target_level: None,
            target_faction: None,
            target_unit_flags: None,
            target_monster_definition_ptr: None,
            target_display_id: None,
            target_native_display_id: None,
            target_hostility: None,
            target_distance: None,
            diagnostics: Vec::new(),
        };

        runtime.tick(Some(&snapshot));

        assert_eq!(runtime.waypoint_progress(), Some((2, 2)));
        assert!(runtime.nav_output().waypoint_reached);
    }
}
