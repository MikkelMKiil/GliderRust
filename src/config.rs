use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindConfig {
    pub move_forward: String,
    pub move_backward: String,
    pub strafe_left: String,
    pub strafe_right: String,
    pub ascend: String,
    pub descend: String,
    pub turn_left: String,
    pub turn_right: String,
    pub interact: String,
    pub assist_target: String,
    pub rotation_slots: [String; 9],
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            move_forward: "W".to_string(),
            move_backward: "S".to_string(),
            strafe_left: "A".to_string(),
            strafe_right: "D".to_string(),
            ascend: "E".to_string(),
            descend: "Q".to_string(),
            turn_left: "MOUSE_LEFT".to_string(),
            turn_right: "MOUSE_RIGHT".to_string(),
            interact: "F".to_string(),
            assist_target: "T".to_string(),
            rotation_slots: [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
            ],
        }
    }
}

impl KeybindConfig {
    pub fn set_binding(&mut self, action: &str, key: &str) -> Result<(), String> {
        let action = action.trim().to_ascii_lowercase();
        let key = normalize_binding_key(key)?;

        match action.as_str() {
            "move_forward" => self.move_forward = key,
            "move_backward" => self.move_backward = key,
            "strafe_left" => self.strafe_left = key,
            "strafe_right" => self.strafe_right = key,
            "ascend" => self.ascend = key,
            "descend" => self.descend = key,
            "turn_left" => self.turn_left = key,
            "turn_right" => self.turn_right = key,
            "interact" => self.interact = key,
            "assist_target" => self.assist_target = key,
            _ => {
                return Err(format!(
                    "Unknown keybind action '{action}'. Expected movement, turn, interact, or assist action name"
                ));
            }
        }

        Ok(())
    }

    pub fn set_rotation_slot(&mut self, slot: u8, key: &str) -> Result<(), String> {
        if !(1..=9).contains(&slot) {
            return Err("Rotation slot must be in range 1..=9".to_string());
        }

        let key = normalize_binding_key(key)?;
        self.rotation_slots[(slot - 1) as usize] = key;
        Ok(())
    }
}

fn normalize_binding_key(key: &str) -> Result<String, String> {
    let normalized = key.trim().to_ascii_uppercase();

    if normalized.is_empty() {
        return Err("Key binding cannot be empty".to_string());
    }

    if normalized.len() > 24 {
        return Err("Key binding is too long".to_string());
    }

    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err("Key binding can only contain letters, numbers, or '_'".to_string());
    }

    Ok(normalized)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub telemetry_enabled: bool,
    pub memory_poll_ms: u64,
    pub keybinds: KeybindConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            telemetry_enabled: true,
            memory_poll_ms: 2500,
            keybinds: KeybindConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeybindConfig;

    #[test]
    fn keybind_action_update_is_normalized() {
        let mut config = KeybindConfig::default();

        config
            .set_binding("move_forward", " w ")
            .expect("binding update should work");

        assert_eq!(config.move_forward, "W");
    }

    #[test]
    fn keybind_rejects_unknown_action() {
        let mut config = KeybindConfig::default();
        assert!(config.set_binding("fly", "F").is_err());
    }

    #[test]
    fn rotation_slot_update_enforces_range() {
        let mut config = KeybindConfig::default();

        assert!(config.set_rotation_slot(0, "1").is_err());
        assert!(config.set_rotation_slot(10, "1").is_err());
        assert!(config.set_rotation_slot(5, "G").is_ok());
        assert_eq!(config.rotation_slots[4], "G");
    }
}
