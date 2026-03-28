#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
    Tap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCommand {
    MoveForward(KeyAction),
    MoveBackward(KeyAction),
    StrafeLeft(KeyAction),
    StrafeRight(KeyAction),
    TurnLeftMouse,
    TurnRightMouse,
    Hotbar(u8),
    ZoomIn,
    ZoomOut,
    LeftClick,
    RightClick,
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("input command out of supported range")]
    Unsupported,
    #[error("input backend failed")]
    BackendFailed,
}

pub trait InputBackend {
    fn send(&mut self, command: InputCommand) -> Result<(), InputError>;
}

#[derive(Debug, Default)]
pub struct NullInputBackend {
    pub sent: Vec<InputCommand>,
}

impl InputBackend for NullInputBackend {
    fn send(&mut self, command: InputCommand) -> Result<(), InputError> {
        self.sent.push(command);
        Ok(())
    }
}

pub fn validate_command(command: InputCommand) -> Result<(), InputError> {
    if let InputCommand::Hotbar(slot) = command {
        if !(1..=9).contains(&slot) {
            return Err(InputError::Unsupported);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_command, InputCommand};

    #[test]
    fn hotbar_range_is_enforced() {
        assert!(validate_command(InputCommand::Hotbar(1)).is_ok());
        assert!(validate_command(InputCommand::Hotbar(9)).is_ok());
        assert!(validate_command(InputCommand::Hotbar(0)).is_err());
        assert!(validate_command(InputCommand::Hotbar(10)).is_err());
    }
}
