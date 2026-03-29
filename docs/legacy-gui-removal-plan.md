# Legacy GUI Removal Plan (WinForms -> HTML)

## Scope Baseline

The legacy inventory was extracted from ReversedMMOGlider `UI/Forms` and mapped into the frontend data source:

- Total interactive controls: 75
- Buttons: 60
- Radio buttons: 15
- Forms: 13
- Source of truth: `frontend/legacy-controls.js`

### Per-form control counts

- ConfigForm.cs: 23
- DebuffList.cs: 2
- DebuffPick.cs: 5
- EvoConfigWindow.cs: 3
- FactionReminder.cs: 3
- GliderForm.cs: 18
- GliderWarning.cs: 3
- KeyEditor.cs: 4
- LaunchpadReminder.cs: 1
- LaunchpadReminder_1.cs: 1
- ProfileProps.cs: 8
- ProfileWizard.cs: 2
- RemindBar.cs: 2

## Current Implementation Status

The HTML shell now includes a Legacy Remaster panel that renders every extracted control (button and radio) from the inventory.

### Live-mapped controls (functional)

- GliderForm.cs:GlideButton -> bot_start
- GliderForm.cs:StopButton -> bot_stop
- GliderForm.cs:KillButton -> bot_pause (temporary semantic mapping)
- GliderForm.cs:LoadProfileButton -> profile_load
- GliderForm.cs:QuickLoadButton -> profile_load
- GliderForm.cs:NewProfileButton -> profile_clear
- GliderForm.cs:ConfigButton -> switch to Settings tab
- GliderForm.cs:EditProfileButton -> switch to Profiles tab

### Placeholder/local behavior

- Every remaining button is present and clickable.
- Unwired actions are logged as explicit placeholders (with original form/control metadata).
- Radio controls are represented as local segmented toggles, preserving selection intent and visibility.

## Full Legacy GUI Removal Plan

### Phase 1: Freeze command contract

- Finalize backend command names and payloads in Rust.
- Add explicit response models for dialogs and wizard-like workflows.
- Keep all command failures user-visible (no silent fallbacks).

### Phase 2: Complete action parity by form

- Replace placeholders with live commands for ConfigForm, KeyEditor, ProfileProps, ProfileWizard, and dialog forms.
- Implement missing operations behind command handlers (save profile, waypoint management, debuff/keymap tools, help routes).
- Add one frontend action test per legacy control key to guarantee it is wired.

### Phase 3: Dialog/workflow migration

- Convert modal WinForms interactions into HTML modal components.
- Preserve legacy branching logic for Yes/No/Cancel and wizard navigation.
- Add deterministic state machine tests for wizard and reminder flows.

### Phase 4: Cutover toggle and soak

- Run HTML frontend as default with egui still compiled behind a feature flag.
- Add telemetry for unimplemented control presses and command errors.
- Require a clean soak window where all critical flows pass without egui fallback.

### Phase 5: Remove egui legacy UI

- Delete egui panel modules and presentation glue after parity checklist is green.
- Retain backend service and command API as the single UI integration path.
- Keep one internal fallback build profile for emergency diagnostics (non-default).

## Egui Removal Gate Checklist

Before deleting egui UI code:

- 100% of legacy control keys in `frontend/legacy-controls.js` are either live or intentionally retired with documented rationale.
- All live-mapped controls have tests.
- Startup, attach, glide, profile load/edit/save, and settings workflows are validated in HTML-only mode.
- No critical user path depends on egui widgets.
- Build and test pass in HTML-first desktop runtime.
