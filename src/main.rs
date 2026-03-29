use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::WindowBuilder;
use wry::http::Request;
use wry::WebViewBuilder;

use glider_rust::backend_api;
use glider_rust::service::SharedAppService;

const TAURI_BRIDGE_SCRIPT: &str = r#"
(() => {
  const pending = new Map();
  let nextId = 1;

  window.__gliderResolve = (id, ok, payload) => {
    const entry = pending.get(id);
    if (!entry) {
      return;
    }

    pending.delete(id);
    if (ok) {
      entry.resolve(payload);
    } else {
      entry.reject(payload);
    }
  };

  window.__TAURI__ = window.__TAURI__ || {};
  window.__TAURI__.core = window.__TAURI__.core || {};
  window.__TAURI__.core.invoke = (cmd, args = {}) => new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    window.ipc.postMessage(JSON.stringify({ id, cmd, args }));
  });
})();
"#;

#[derive(Debug, Clone)]
enum UserEvent {
    EvalScript(String),
}

#[derive(Debug, Deserialize)]
struct IpcInvokeMessage {
    id: u64,
    cmd: String,
    #[serde(default)]
    args: Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    glider_rust::init_logging();
    tracing::info!("Starting GliderRust — embedded HTML runtime");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("GliderRust")
        .with_inner_size(LogicalSize::new(1100.0, 860.0))
        .with_min_inner_size(LogicalSize::new(960.0, 720.0))
        .build(&event_loop)?;

    let shared_state = backend_api::new_state();

    let proxy = event_loop.create_proxy();
    let ipc_state = shared_state.clone();

    let webview = WebViewBuilder::new()
        .with_initialization_script(TAURI_BRIDGE_SCRIPT)
        .with_url(frontend_index_url())
        .with_ipc_handler(move |request: Request<String>| {
            handle_ipc_message(request.body().as_str(), &ipc_state, &proxy);
        })
        .build(&window)?;

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(25));

        match event {
            Event::NewEvents(_) => {
                if let Err(err) = backend_api::run_scheduled_cycle(&shared_state) {
                    tracing::warn!("Scheduled cycle error: {err}");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::EvalScript(script)) => {
                if let Err(err) = webview.evaluate_script(&script) {
                    tracing::warn!("Failed to evaluate response script: {err}");
                }
            }
            _ => {}
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}

fn frontend_index_url() -> String {
    let index_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("frontend")
        .join("index.html");
    let canonical = index_path.canonicalize().unwrap_or(index_path);

    url::Url::from_file_path(&canonical)
        .expect("frontend/index.html must exist and be a valid file path")
        .to_string()
}

fn handle_ipc_message(raw_message: &str, state: &SharedAppService, proxy: &EventLoopProxy<UserEvent>) {
    let message: IpcInvokeMessage = match serde_json::from_str(raw_message) {
        Ok(message) => message,
        Err(err) => {
            tracing::warn!("Invalid IPC payload: {err}. payload={raw_message}");
            return;
        }
    };

    let response_script = match dispatch_command(state, &message.cmd, &message.args) {
        Ok(value) => format!("window.__gliderResolve({}, true, {});", message.id, value),
        Err(err) => {
            let err_json = serde_json::to_string(&err)
                .unwrap_or_else(|_| "\"Internal command error\"".to_string());
            format!("window.__gliderResolve({}, false, {});", message.id, err_json)
        }
    };

    if let Err(err) = proxy.send_event(UserEvent::EvalScript(response_script)) {
        tracing::warn!("Failed to queue IPC response event: {err}");
    }
}

fn dispatch_command(state: &SharedAppService, cmd: &str, args: &Value) -> Result<Value, String> {
    match cmd {
        "get_state_snapshot" => to_json(backend_api::get_state_snapshot(state)?),
        "bot_start" => to_json(backend_api::bot_start(state)?),
        "bot_pause" => to_json(backend_api::bot_pause(state)?),
        "bot_stop" => to_json(backend_api::bot_stop(state)?),
        "profile_load" => {
            let path = get_string_arg(args, &["path"])?;
            to_json(backend_api::profile_load(state, &path)?)
        }
        "profile_clear" => to_json(backend_api::profile_clear(state)?),
        "memory_attach_pid" => {
            let pid_u64 = get_u64_arg(args, &["pid"])?;
            let pid = u32::try_from(pid_u64).map_err(|_| "pid out of range for u32".to_string())?;
            to_json(backend_api::memory_attach_pid(state, pid)?)
        }
        "memory_attach_wow" => to_json(backend_api::memory_attach_wow(state)?),
        "memory_detach" => to_json(backend_api::memory_detach(state)?),
        "settings_set_poll_interval" => {
            let interval_ms = get_u64_arg(args, &["intervalMs", "interval_ms"])?;
            to_json(backend_api::settings_set_poll_interval(state, interval_ms)?)
        }
        "settings_set_telemetry" => {
            let enabled = get_bool_arg(args, &["telemetryEnabled", "telemetry_enabled"])?;
            to_json(backend_api::settings_set_telemetry(state, enabled)?)
        }
        "run_cycle_now" => to_json(backend_api::run_cycle_now(state)?),
        "run_scheduled_cycle" => to_json(backend_api::run_scheduled_cycle(state)?),
        _ => Err(format!("Unknown command: {cmd}")),
    }
}

fn to_json<T: serde::Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|err| format!("Failed to serialize command result: {err}"))
}

fn get_arg<'a>(args: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| args.get(*key))
}

fn get_string_arg(args: &Value, keys: &[&str]) -> Result<String, String> {
    match get_arg(args, keys) {
        Some(value) => value
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("Expected string for any of keys: {keys:?}")),
        None => Err(format!("Missing required argument. Expected any of keys: {keys:?}")),
    }
}

fn get_u64_arg(args: &Value, keys: &[&str]) -> Result<u64, String> {
    match get_arg(args, keys) {
        Some(value) => value
            .as_u64()
            .ok_or_else(|| format!("Expected unsigned integer for any of keys: {keys:?}")),
        None => Err(format!("Missing required argument. Expected any of keys: {keys:?}")),
    }
}

fn get_bool_arg(args: &Value, keys: &[&str]) -> Result<bool, String> {
    match get_arg(args, keys) {
        Some(value) => value
            .as_bool()
            .ok_or_else(|| format!("Expected boolean for any of keys: {keys:?}")),
        None => Err(format!("Missing required argument. Expected any of keys: {keys:?}")),
    }
}
