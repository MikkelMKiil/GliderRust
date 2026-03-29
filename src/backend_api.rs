use crate::service::{new_shared_app_service, AppService, AppStateSnapshot, SharedAppService};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachWowResponse {
    pub pid: u32,
    pub state: AppStateSnapshot,
}

fn with_service<T>(
    state: &SharedAppService,
    f: impl FnOnce(&AppService) -> Result<T, String>,
) -> Result<T, String> {
    let guard = state
        .lock()
        .map_err(|_| "App service lock poisoned".to_string())?;
    f(&guard)
}

fn with_service_mut<T>(
    state: &SharedAppService,
    f: impl FnOnce(&mut AppService) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = state
        .lock()
        .map_err(|_| "App service lock poisoned".to_string())?;
    f(&mut guard)
}

pub fn new_state() -> SharedAppService {
    new_shared_app_service()
}

pub fn get_state_snapshot(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service(state, |service| Ok(service.state_snapshot()))
}

pub fn bot_start(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.start_bot()?;
        Ok(service.state_snapshot())
    })
}

pub fn bot_pause(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.pause_bot();
        Ok(service.state_snapshot())
    })
}

pub fn bot_stop(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.stop_bot();
        Ok(service.state_snapshot())
    })
}

pub fn profile_load(state: &SharedAppService, path: &str) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.load_profile_from_path(path)?;
        Ok(service.state_snapshot())
    })
}

pub fn profile_clear(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.clear_profile();
        Ok(service.state_snapshot())
    })
}

pub fn memory_attach_pid(state: &SharedAppService, pid: u32) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.attach_pid(pid)?;
        Ok(service.state_snapshot())
    })
}

pub fn memory_attach_wow(state: &SharedAppService) -> Result<AttachWowResponse, String> {
    with_service_mut(state, |service| {
        let pid = service.attach_wow()?;
        Ok(AttachWowResponse {
            pid,
            state: service.state_snapshot(),
        })
    })
}

pub fn memory_detach(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.detach();
        Ok(service.state_snapshot())
    })
}

pub fn settings_set_poll_interval(
    state: &SharedAppService,
    interval_ms: u64,
) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.set_memory_poll_ms(interval_ms);
        Ok(service.state_snapshot())
    })
}

pub fn settings_set_telemetry(
    state: &SharedAppService,
    telemetry_enabled: bool,
) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.set_telemetry_enabled(telemetry_enabled);
        Ok(service.state_snapshot())
    })
}

pub fn settings_set_keybind(
    state: &SharedAppService,
    action: &str,
    key: &str,
) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.set_keybind(action, key)?;
        Ok(service.state_snapshot())
    })
}

pub fn settings_set_rotation_slot(
    state: &SharedAppService,
    slot: u8,
    key: &str,
) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.set_rotation_slot(slot, key)?;
        Ok(service.state_snapshot())
    })
}

pub fn run_cycle_now(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.run_cycle_now();
        Ok(service.state_snapshot())
    })
}

pub fn run_scheduled_cycle(state: &SharedAppService) -> Result<AppStateSnapshot, String> {
    with_service_mut(state, |service| {
        service.run_scheduled_cycle();
        Ok(service.state_snapshot())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        bot_pause, bot_start, bot_stop, memory_attach_pid, new_state, settings_set_keybind,
        settings_set_rotation_slot,
    };
    use crate::bot::BotState;

    #[test]
    fn command_style_bot_controls_work() {
        let state = new_state();
        let pid = std::process::id();

        memory_attach_pid(&state, pid).expect("attach should work for current process");

        let running = bot_start(&state).expect("start should work");
        assert_eq!(running.bot_state, BotState::Running);

        let paused = bot_pause(&state).expect("pause should work");
        assert_eq!(paused.bot_state, BotState::Paused);

        let stopped = bot_stop(&state).expect("stop should work");
        assert_eq!(stopped.bot_state, BotState::Stopped);
    }

    #[test]
    fn command_start_requires_attachment() {
        let state = new_state();

        let error = bot_start(&state).expect_err("start should fail when detached");
        assert_eq!(error, "Cannot start bot: attach to a process first");
    }

    #[test]
    fn keybind_commands_update_config() {
        let state = new_state();

        let after_action = settings_set_keybind(&state, "interact", "g")
            .expect("keybind action update should work");
        assert_eq!(after_action.config.keybinds.interact, "G");

        let after_rotation =
            settings_set_rotation_slot(&state, 3, "r").expect("rotation update should work");
        assert_eq!(after_rotation.config.keybinds.rotation_slots[2], "R");
    }
}
