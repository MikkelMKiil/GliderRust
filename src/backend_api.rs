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
        service.start_bot();
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

pub fn memory_attach_pid(
    state: &SharedAppService,
    pid: u32,
) -> Result<AppStateSnapshot, String> {
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
    use super::{bot_pause, bot_start, bot_stop, new_state};
    use crate::bot::BotState;

    #[test]
    fn command_style_bot_controls_work() {
        let state = new_state();

        let running = bot_start(&state).expect("start should work");
        assert_eq!(running.bot_state, BotState::Running);

        let paused = bot_pause(&state).expect("pause should work");
        assert_eq!(paused.bot_state, BotState::Paused);

        let stopped = bot_stop(&state).expect("stop should work");
        assert_eq!(stopped.bot_state, BotState::Stopped);
    }
}
