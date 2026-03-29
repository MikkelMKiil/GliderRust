import { LEGACY_CONTROLS } from "./legacy-controls.js";

const state = {
  snapshot: null,
  legacyControls: LEGACY_CONTROLS,
  legacyRadioSelection: {},
  legacyActionLog: [],
};

const LIVE_LEGACY_KEYS = new Set([
  "GliderForm.cs:GlideButton",
  "GliderForm.cs:StopButton",
  "GliderForm.cs:KillButton",
  "GliderForm.cs:LoadProfileButton",
  "GliderForm.cs:QuickLoadButton",
  "GliderForm.cs:NewProfileButton",
  "GliderForm.cs:ConfigButton",
  "GliderForm.cs:EditProfileButton",
]);

const hasTauri = () => Boolean(window.__TAURI__?.core?.invoke);

function pick(obj, ...keys) {
  if (!obj || typeof obj !== "object") {
    return undefined;
  }
  for (const key of keys) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      return obj[key];
    }
  }
  return undefined;
}

function normalizeState(appState) {
  const cycleStats = pick(appState, "cycleStats", "cycle_stats") || {};

  return {
    memoryAttached: !!pick(appState, "memoryAttached", "memory_attached"),
    botState: pick(appState, "botState", "bot_state") || "Unknown",
    botStatus: pick(appState, "botStatus", "bot_status") || "Unknown",
    activeProfileName: pick(appState, "activeProfileName", "active_profile_name"),
    waypointProgress: pick(appState, "waypointProgress", "waypoint_progress"),
    snapshot: pick(appState, "snapshot"),
    statusMessage: pick(appState, "statusMessage", "status_message") || "Ready",
    cycleStats: {
      intervalMs: pick(cycleStats, "intervalMs", "interval_ms") || 0,
      cyclesLastMinute: pick(cycleStats, "cyclesLastMinute", "cycles_last_minute") || 0,
      totalCycles: pick(cycleStats, "totalCycles", "total_cycles") || 0,
      lastCycleMs: pick(cycleStats, "lastCycleMs", "last_cycle_ms") || 0,
    },
  };
}

async function invoke(command, args = {}) {
  if (!hasTauri()) {
    return mockInvoke(command, args);
  }
  return window.__TAURI__.core.invoke(command, args);
}

function setStatus(text) {
  const el = document.getElementById("statusBar");
  if (el) {
    el.textContent = text;
  }
}

function setTab(tabName) {
  const tabs = document.querySelectorAll(".tab");
  const panels = document.querySelectorAll(".panel");

  tabs.forEach((tab) => {
    const isActive = tab.dataset.tab === tabName;
    tab.classList.toggle("active", isActive);
  });

  panels.forEach((panel) => {
    const isActive = panel.id === `panel-${tabName}`;
    panel.classList.toggle("active", isActive);
  });
}

function bindTabs() {
  const tabs = document.querySelectorAll(".tab");
  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      setTab(tab.dataset.tab);
    });
  });
}

function render(appState) {
  const normalized = normalizeState(appState);
  state.snapshot = normalized;

  const attachPill = document.getElementById("attachPill");
  const botPill = document.getElementById("botPill");
  const stateText = document.getElementById("stateText");
  const statusText = document.getElementById("statusText");
  const profileText = document.getElementById("profileText");
  const waypointText = document.getElementById("waypointText");
  const pollText = document.getElementById("pollText");
  const snapshotPreview = document.getElementById("snapshotPreview");
  const diagText = document.getElementById("diagText");

  attachPill.textContent = normalized.memoryAttached ? "Attached" : "Detached";
  botPill.textContent = normalized.botState;
  stateText.textContent = normalized.botState;
  statusText.textContent = normalized.botStatus;
  profileText.textContent = normalized.activeProfileName || "None";

  const wp = normalized.waypointProgress;
  waypointText.textContent = Array.isArray(wp) ? `${wp[0]} / ${wp[1]}` : "-";
  pollText.textContent = `${normalized.cycleStats.intervalMs} ms`;

  snapshotPreview.textContent = normalized.snapshot
    ? JSON.stringify(normalized.snapshot, null, 2)
    : "No snapshot yet";

  const diagnostics = normalized.snapshot?.diagnostics || [];
  diagText.textContent = diagnostics.length ? diagnostics.join("\n") : "No diagnostics yet";

  setStatus(normalized.statusMessage || "Ready");
}

async function renderFromCommand(command, args = {}) {
  const result = await invoke(command, args);
  if (result && typeof result === "object" && result.state) {
    render(result.state);
  } else {
    render(result);
  }
  return result;
}

async function refresh() {
  try {
    const current = await invoke("get_state_snapshot");
    render(current);
  } catch (err) {
    setStatus(`Failed to refresh state: ${String(err)}`);
  }
}

function bindActions() {
  document.getElementById("startBtn").addEventListener("click", async () => {
    await renderFromCommand("bot_start");
  });

  document.getElementById("pauseBtn").addEventListener("click", async () => {
    await renderFromCommand("bot_pause");
  });

  document.getElementById("stopBtn").addEventListener("click", async () => {
    await renderFromCommand("bot_stop");
  });

  document.getElementById("loadProfileBtn").addEventListener("click", async () => {
    const path = document.getElementById("profilePath").value.trim();
    await renderFromCommand("profile_load", { path });
  });

  document.getElementById("clearProfileBtn").addEventListener("click", async () => {
    await renderFromCommand("profile_clear");
  });

  document.getElementById("attachPidBtn").addEventListener("click", async () => {
    const pid = Number(document.getElementById("pidInput").value);
    await renderFromCommand("memory_attach_pid", { pid });
  });

  document.getElementById("autoAttachBtn").addEventListener("click", async () => {
    const result = await invoke("memory_attach_wow");
    if (result.state) {
      render(result.state);
      document.getElementById("pidInput").value = String(result.pid);
    } else {
      render(result);
    }
  });

  document.getElementById("detachBtn").addEventListener("click", async () => {
    await renderFromCommand("memory_detach");
  });

  document.getElementById("setPollBtn").addEventListener("click", async () => {
    const intervalMs = Number(document.getElementById("pollInput").value);
    await renderFromCommand("settings_set_poll_interval", { intervalMs });
  });

  document.getElementById("setTelemetryBtn").addEventListener("click", async () => {
    const telemetryEnabled = document.getElementById("telemetryInput").checked;
    await renderFromCommand("settings_set_telemetry", { telemetryEnabled });
  });
}

function normalizeLabel(label, fallback) {
  const text = (label || fallback || "Unnamed").replace(/&/g, "").trim();
  return text.length ? text : fallback || "Unnamed";
}

function legacyKey(control) {
  return `${control.form}:${control.control}`;
}

function legacyActionKind(control) {
  if (LIVE_LEGACY_KEYS.has(legacyKey(control))) {
    return "live";
  }
  if (control.type === "RadioButton") {
    return "local";
  }
  return "placeholder";
}

function appendLegacyLog(message, level = "placeholder") {
  const stamp = new Date().toLocaleTimeString();
  state.legacyActionLog.unshift(`[${stamp}] [${level}] ${message}`);
  state.legacyActionLog = state.legacyActionLog.slice(0, 40);

  const logEl = document.getElementById("legacyActionLog");
  if (logEl) {
    logEl.textContent = state.legacyActionLog.join("\n");
  }
}

function controlMatchesFilter(control, query) {
  if (!query) {
    return true;
  }
  const text = [
    control.form,
    control.control,
    control.label,
    control.clickHandler || "",
  ]
    .join(" ")
    .toLowerCase();
  return text.includes(query);
}

function updateLegacyCounts(allControls, visibleControls) {
  const totalEl = document.getElementById("legacyTotalCount");
  const visibleEl = document.getElementById("legacyVisibleCount");
  const buttonEl = document.getElementById("legacyButtonCount");
  const radioEl = document.getElementById("legacyRadioCount");

  const totalButtons = allControls.filter((c) => c.type === "Button").length;
  const totalRadios = allControls.filter((c) => c.type === "RadioButton").length;

  if (totalEl) {
    totalEl.textContent = String(allControls.length);
  }
  if (visibleEl) {
    visibleEl.textContent = String(visibleControls.length);
  }
  if (buttonEl) {
    buttonEl.textContent = String(totalButtons);
  }
  if (radioEl) {
    radioEl.textContent = String(totalRadios);
  }
}

function renderLegacyControls() {
  const host = document.getElementById("legacyForms");
  if (!host) {
    return;
  }

  const query = (document.getElementById("legacySearch")?.value || "")
    .trim()
    .toLowerCase();
  const showRadios = document.getElementById("showRadioControls")?.checked ?? true;

  const visibleControls = state.legacyControls
    .filter((control) => (showRadios ? true : control.type !== "RadioButton"))
    .filter((control) => controlMatchesFilter(control, query))
    .sort((a, b) => {
      if (a.form !== b.form) {
        return a.form.localeCompare(b.form);
      }
      return a.control.localeCompare(b.control);
    });

  updateLegacyCounts(state.legacyControls, visibleControls);

  host.replaceChildren();

  if (!visibleControls.length) {
    const empty = document.createElement("p");
    empty.className = "legacyCode";
    empty.textContent = "No controls match the current filter.";
    host.appendChild(empty);
    return;
  }

  const byForm = new Map();
  visibleControls.forEach((control) => {
    if (!byForm.has(control.form)) {
      byForm.set(control.form, []);
    }
    byForm.get(control.form).push(control);
  });

  for (const [form, controls] of byForm.entries()) {
    const formCard = document.createElement("article");
    formCard.className = "legacyFormCard";

    const header = document.createElement("div");
    header.className = "legacyFormHeader";

    const title = document.createElement("h3");
    title.textContent = form;

    const count = document.createElement("span");
    count.className = "legacyControlCount";
    count.textContent = `${controls.length} controls`;

    header.append(title, count);

    const grid = document.createElement("div");
    grid.className = "legacyControlsGrid";

    controls.forEach((control) => {
      const kind = legacyActionKind(control);

      const item = document.createElement("div");
      item.className = "legacyControl";

      const button = document.createElement("button");
      button.className = `btn legacyBtn ${control.type === "RadioButton" ? "radio" : ""}`;
      if (
        control.type === "RadioButton" &&
        state.legacyRadioSelection[control.form] === control.control
      ) {
        button.classList.add("selected");
      }
      button.textContent = normalizeLabel(control.label, control.control);
      button.title = control.clickHandler
        ? `${control.control} -> ${control.clickHandler}`
        : control.control;
      button.addEventListener("click", async () => {
        await handleLegacyControl(control);
      });

      const meta = document.createElement("div");
      meta.className = "legacyMetaRow";

      const code = document.createElement("span");
      code.className = "legacyCode";
      code.textContent = control.control;

      const typeChip = document.createElement("span");
      typeChip.className = "chip";
      typeChip.textContent = control.type;

      const modeChip = document.createElement("span");
      modeChip.className = `chip ${kind}`;
      modeChip.textContent = kind === "live" ? "Live" : kind === "local" ? "Local" : "Placeholder";

      meta.append(code, typeChip, modeChip);
      item.append(button, meta);
      grid.appendChild(item);
    });

    formCard.append(header, grid);
    host.appendChild(formCard);
  }
}

function selectedProfilePath() {
  return document.getElementById("profilePath")?.value.trim() || "";
}

async function runLegacyCommand(control, command, args = {}, successText = null) {
  try {
    await renderFromCommand(command, args);
    const message = successText || `${control.control} executed`;
    setStatus(message);
    appendLegacyLog(`${legacyKey(control)} -> ${command}`, "live");
  } catch (err) {
    const message = `Legacy action failed for ${control.control}: ${String(err)}`;
    setStatus(message);
    appendLegacyLog(message, "error");
  }
}

async function handleLegacyControl(control) {
  const key = legacyKey(control);

  if (control.type === "RadioButton") {
    state.legacyRadioSelection[control.form] = control.control;
    renderLegacyControls();
    const msg = `${control.form}: selected ${normalizeLabel(control.label, control.control)}`;
    setStatus(msg);
    appendLegacyLog(msg, "local");
    return;
  }

  if (key === "GliderForm.cs:GlideButton") {
    await runLegacyCommand(control, "bot_start", {}, "Glider button mapped to bot start");
    return;
  }

  if (key === "GliderForm.cs:StopButton") {
    await runLegacyCommand(control, "bot_stop", {}, "Stop button mapped to bot stop");
    return;
  }

  if (key === "GliderForm.cs:KillButton") {
    await runLegacyCommand(control, "bot_pause", {}, "1-Kill mapped to bot pause placeholder");
    return;
  }

  if (key === "GliderForm.cs:LoadProfileButton" || key === "GliderForm.cs:QuickLoadButton") {
    const path = selectedProfilePath();
    if (!path) {
      const msg = "Set a profile path first to use Load/QuickLoad";
      setStatus(msg);
      appendLegacyLog(msg, "placeholder");
      setTab("profiles");
      return;
    }
    await runLegacyCommand(control, "profile_load", { path }, `Loaded profile from ${path}`);
    return;
  }

  if (key === "GliderForm.cs:NewProfileButton") {
    await runLegacyCommand(control, "profile_clear", {}, "New Profile mapped to clear profile");
    return;
  }

  if (key === "GliderForm.cs:ConfigButton") {
    setTab("settings");
    const msg = "Configure opened Settings tab";
    setStatus(msg);
    appendLegacyLog(msg, "live");
    return;
  }

  if (key === "GliderForm.cs:EditProfileButton") {
    setTab("profiles");
    const msg = "Edit Profile opened Profiles tab";
    setStatus(msg);
    appendLegacyLog(msg, "live");
    return;
  }

  const placeholderText = `Placeholder mapped: ${control.form} / ${control.control}`;
  setStatus(placeholderText);
  appendLegacyLog(
    `${placeholderText}${control.clickHandler ? ` -> ${control.clickHandler}` : ""}`,
    "placeholder",
  );
}

function bindLegacyPanel() {
  document.getElementById("legacySearch")?.addEventListener("input", () => {
    renderLegacyControls();
  });

  document.getElementById("showRadioControls")?.addEventListener("change", () => {
    renderLegacyControls();
  });

  renderLegacyControls();
  appendLegacyLog(
    `Loaded ${state.legacyControls.length} controls from legacy WinForms inventory`,
    "live",
  );
}

function mockState() {
  return {
    config: {
      telemetryEnabled: true,
      memoryPollMs: 2500,
    },
    statusMessage: "Frontend shell running in mock mode",
    memoryAttached: false,
    botState: "Stopped",
    botStatus: "Idle",
    activeProfileName: null,
    waypointProgress: null,
    navOutput: {
      desiredHeadingRad: null,
      headingErrorRad: null,
      distanceToWaypoint: null,
      waypointReached: false,
    },
    suggestedInputs: [],
    snapshot: null,
    cycleStats: {
      intervalMs: 2500,
      cyclesLastMinute: 0,
      totalCycles: 0,
      lastCycleMs: 0,
    },
  };
}

let mock = mockState();

async function mockInvoke(command, args) {
  switch (command) {
    case "get_state_snapshot":
      return mock;
    case "bot_start":
      mock = {
        ...mock,
        botState: "Running",
        botStatus: "PollingMemory",
        statusMessage: "Bot started",
      };
      return mock;
    case "bot_pause":
      mock = {
        ...mock,
        botState: "Paused",
        botStatus: "Paused",
        statusMessage: "Bot paused",
      };
      return mock;
    case "bot_stop":
      mock = {
        ...mock,
        botState: "Stopped",
        botStatus: "Idle",
        statusMessage: "Bot stopped",
      };
      return mock;
    case "profile_load":
      mock = {
        ...mock,
        activeProfileName: args.path || "Unknown",
        statusMessage: "Profile loaded (mock)",
      };
      return mock;
    case "profile_clear":
      mock = {
        ...mock,
        activeProfileName: null,
        statusMessage: "Profile cleared",
      };
      return mock;
    case "memory_attach_pid":
      mock = {
        ...mock,
        memoryAttached: true,
        statusMessage: `Attached to PID ${args.pid}`,
      };
      return mock;
    case "memory_attach_wow":
      mock = {
        ...mock,
        memoryAttached: true,
        statusMessage: "Auto-attached to WoW PID 1234",
      };
      return {
        pid: 1234,
        state: mock,
      };
    case "memory_detach":
      mock = {
        ...mock,
        memoryAttached: false,
        statusMessage: "Detached from process",
      };
      return mock;
    case "settings_set_poll_interval":
      mock = {
        ...mock,
        config: {
          ...mock.config,
          memoryPollMs: args.intervalMs,
        },
        cycleStats: {
          ...mock.cycleStats,
          intervalMs: args.intervalMs,
        },
        statusMessage: `Poll interval set to ${args.intervalMs} ms`,
      };
      return mock;
    case "settings_set_telemetry":
      mock = {
        ...mock,
        config: {
          ...mock.config,
          telemetryEnabled: !!args.telemetryEnabled,
        },
        statusMessage: `Telemetry ${args.telemetryEnabled ? "enabled" : "disabled"}`,
      };
      return mock;
    default:
      return mock;
  }
}

function initRuntimeHint() {
  const hint = document.getElementById("runtimeHint");
  hint.textContent = hasTauri()
    ? "Connected to Tauri runtime"
    : "Mock mode (open in Tauri to enable Rust backend)";
}

function startPolling() {
  setInterval(async () => {
    await refresh();
  }, 1500);
}

async function init() {
  bindTabs();
  bindActions();
  bindLegacyPanel();
  initRuntimeHint();
  setTab("home");
  await refresh();
  startPolling();
}

init();
