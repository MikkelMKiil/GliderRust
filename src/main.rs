use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::WindowBuilder;
use wry::http::{header::CONTENT_TYPE, Request, Response, StatusCode};
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

const FRONTEND_PROTOCOL: &str = "glider";

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
    let frontend_root = frontend_root_dir();

    let proxy = event_loop.create_proxy();
    let ipc_state = shared_state.clone();

    let webview = WebViewBuilder::new()
        .with_custom_protocol(FRONTEND_PROTOCOL.into(), move |_webview_id, request| {
            serve_frontend_asset(&frontend_root, request)
        })
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
    format!("{FRONTEND_PROTOCOL}://localhost/index.html")
}

fn frontend_root_dir() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend");
    root.canonicalize().unwrap_or(root)
}

fn serve_frontend_asset(
    frontend_root: &Path,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let response = match read_frontend_asset(frontend_root, request.uri().path()) {
        Ok((mime, bytes)) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, mime)
            .body(bytes)
            .unwrap_or_else(|err| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(format!("Failed to build asset response: {err}").into_bytes())
                    .unwrap()
            }),
        Err((status, message)) => Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(message.into_bytes())
            .unwrap(),
    };

    response.map(Into::into)
}

fn read_frontend_asset(
    frontend_root: &Path,
    request_path: &str,
) -> Result<(&'static str, Vec<u8>), (StatusCode, String)> {
    let relative = match request_path {
        "/" | "" => "index.html",
        _ => request_path.trim_start_matches('/'),
    };

    let candidate = frontend_root.join(relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|_| (StatusCode::NOT_FOUND, format!("Asset not found: {relative}")))?;

    if !canonical.starts_with(frontend_root) {
        return Err((
            StatusCode::FORBIDDEN,
            "Asset path escapes frontend root".to_string(),
        ));
    }

    let bytes = std::fs::read(&canonical).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read asset {relative}: {err}"),
        )
    })?;

    Ok((mime_for_asset(&canonical), bytes))
}

fn mime_for_asset(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn handle_ipc_message(
    raw_message: &str,
    state: &SharedAppService,
    proxy: &EventLoopProxy<UserEvent>,
) {
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
            format!(
                "window.__gliderResolve({}, false, {});",
                message.id, err_json
            )
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
        "settings_set_keybind" => {
            let action = get_string_arg(args, &["action"])?;
            let key = get_string_arg(args, &["key"])?;
            to_json(backend_api::settings_set_keybind(state, &action, &key)?)
        }
        "settings_set_rotation_slot" => {
            let slot_u64 = get_u64_arg(args, &["slot"])?;
            let slot =
                u8::try_from(slot_u64).map_err(|_| "slot out of range for u8".to_string())?;
            let key = get_string_arg(args, &["key"])?;
            to_json(backend_api::settings_set_rotation_slot(state, slot, &key)?)
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
        None => Err(format!(
            "Missing required argument. Expected any of keys: {keys:?}"
        )),
    }
}

fn get_u64_arg(args: &Value, keys: &[&str]) -> Result<u64, String> {
    match get_arg(args, keys) {
        Some(value) => value
            .as_u64()
            .ok_or_else(|| format!("Expected unsigned integer for any of keys: {keys:?}")),
        None => Err(format!(
            "Missing required argument. Expected any of keys: {keys:?}"
        )),
    }
}

fn get_bool_arg(args: &Value, keys: &[&str]) -> Result<bool, String> {
    match get_arg(args, keys) {
        Some(value) => value
            .as_bool()
            .ok_or_else(|| format!("Expected boolean for any of keys: {keys:?}")),
        None => Err(format!(
            "Missing required argument. Expected any of keys: {keys:?}"
        )),
    }
}
