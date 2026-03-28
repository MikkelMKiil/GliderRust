use thiserror::Error;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

use crate::memory::types::MemorySnapshot;
use crate::offsets::WOTLK_3_3_5A;

#[derive(Debug, Error)]
pub enum MemoryReaderError {
    #[error("process not attached")]
    NotAttached,
    #[error("invalid pid")]
    InvalidPid,
    #[error("could not find a running WoW process")]
    WowProcessNotFound,
    #[error("failed to open process {0}")]
    OpenProcessFailed(u32),
    #[error("read process memory failed at 0x{0:016X}")]
    ReadMemoryFailed(usize),
}

#[derive(Debug)]
pub struct MemoryReader {
    attached_pid: Option<u32>,
    process_handle: Option<HANDLE>,
}

impl Default for MemoryReader {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryReader {
    pub fn new() -> Self {
        Self {
            attached_pid: None,
            process_handle: None,
        }
    }

    pub fn find_wow_pid() -> Result<u32, MemoryReaderError> {
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|_| MemoryReaderError::WowProcessNotFound)?
        };

        if snapshot == INVALID_HANDLE_VALUE {
            return Err(MemoryReaderError::WowProcessNotFound);
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut found: Option<u32> = None;

        if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
            loop {
                let exe_name = utf16_to_string(&entry.szExeFile);
                if is_wow_process_name(&exe_name) {
                    found = Some(entry.th32ProcessID);
                    break;
                }

                if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                    break;
                }
            }
        }

        let _ = unsafe { CloseHandle(snapshot) };

        found.ok_or(MemoryReaderError::WowProcessNotFound)
    }

    pub fn attach(&mut self, pid: u32) -> Result<(), MemoryReaderError> {
        if pid == 0 {
            return Err(MemoryReaderError::InvalidPid);
        }

        self.detach();

        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) }
            .map_err(|_| MemoryReaderError::OpenProcessFailed(pid))?;

        self.process_handle = Some(handle);
        self.attached_pid = Some(pid);
        Ok(())
    }

    pub fn attach_wow(&mut self) -> Result<u32, MemoryReaderError> {
        let pid = Self::find_wow_pid()?;
        self.attach(pid)?;
        Ok(pid)
    }

    pub fn detach(&mut self) {
        if let Some(handle) = self.process_handle.take() {
            let _ = unsafe { CloseHandle(handle) };
        }
        self.attached_pid = None;
    }

    pub fn is_attached(&self) -> bool {
        self.attached_pid.is_some() && self.process_handle.is_some()
    }

    pub fn read_snapshot(&self) -> Result<MemorySnapshot, MemoryReaderError> {
        let handle = self
            .process_handle
            .ok_or(MemoryReaderError::NotAttached)?;

        if self.attached_pid.is_none() {
            return Err(MemoryReaderError::NotAttached);
        }

        let mut diagnostics = Vec::new();

        let client_connection = read_u32(handle, WOTLK_3_3_5A.client_connection_addr)
            .map(|addr| addr as usize)
            .map_err(|_| MemoryReaderError::ReadMemoryFailed(WOTLK_3_3_5A.client_connection_addr))?;
        diagnostics.push(format!("trace client_connection=0x{client_connection:08X}"));
        if client_connection == 0 {
            return Err(MemoryReaderError::ReadMemoryFailed(WOTLK_3_3_5A.client_connection_addr));
        }

        let object_manager = read_u32(handle, client_connection + WOTLK_3_3_5A.object_manager_offset)
            .map(|addr| addr as usize)
            .map_err(|_| {
                MemoryReaderError::ReadMemoryFailed(
                    client_connection + WOTLK_3_3_5A.object_manager_offset,
                )
            })?;
        diagnostics.push(format!("trace object_manager=0x{object_manager:08X}"));
        if object_manager == 0 {
            return Err(MemoryReaderError::ReadMemoryFailed(
                client_connection + WOTLK_3_3_5A.object_manager_offset,
            ));
        }

        let static_local_guid = read_u64(handle, WOTLK_3_3_5A.player_id_addr)
            .ok()
            .filter(|guid| *guid != 0);
        diagnostics.push(format!(
            "trace static_local_guid={}",
            static_local_guid
                .map(|guid| format!("0x{guid:016X}"))
                .unwrap_or_else(|| "none".to_string())
        ));

        let manager_local_guid = read_u64(handle, object_manager + WOTLK_3_3_5A.local_guid_offset)
            .ok()
            .filter(|guid| *guid != 0);
        diagnostics.push(format!(
            "trace manager_local_guid={}",
            manager_local_guid
                .map(|guid| format!("0x{guid:016X}"))
                .unwrap_or_else(|| "none".to_string())
        ));

        let local_guid = static_local_guid.or(manager_local_guid);
        diagnostics.push(format!(
            "trace chosen_local_guid={}",
            local_guid
                .map(|guid| format!("0x{guid:016X}"))
                .unwrap_or_else(|| "none".to_string())
        ));

        let active_player_ptr = read_u32(handle, object_manager + WOTLK_3_3_5A.active_player_offset)
            .map(|addr| addr as usize)
            .map_err(|_| {
                MemoryReaderError::ReadMemoryFailed(object_manager + WOTLK_3_3_5A.active_player_offset)
            })?;
        diagnostics.push(format!("trace active_player_ptr=0x{active_player_ptr:08X}"));

        let mut player_base = active_player_ptr;

        if let Some(expected_guid) = local_guid {
            let active_guid = read_u64(handle, active_player_ptr + WOTLK_3_3_5A.object_guid_offset).ok();
            diagnostics.push(format!(
                "trace active_player_guid={}",
                active_guid
                    .map(|guid| format!("0x{guid:016X}"))
                    .unwrap_or_else(|| "unreadable".to_string())
            ));

            if active_player_ptr == 0 || active_guid != Some(expected_guid) {
                if let Some(found) = find_object_by_guid(
                    handle,
                    object_manager,
                    expected_guid,
                    "local player",
                    &mut diagnostics,
                ) {
                    diagnostics.push(format!(
                        "trace local player fallback selected object=0x{found:08X}"
                    ));
                    player_base = found;
                }
            }
        }

        diagnostics.push(format!("trace player_base=0x{player_base:08X}"));
        if player_base == 0 {
            return Err(MemoryReaderError::ReadMemoryFailed(
                object_manager + WOTLK_3_3_5A.active_player_offset,
            ));
        }

        let x = read_f32(handle, player_base + WOTLK_3_3_5A.player_xyz_offset)
            .map_err(|_| {
                diagnostics.push("player x read failed".to_string());
            })
            .ok();
        let y = read_f32(handle, player_base + WOTLK_3_3_5A.player_xyz_offset + 0x4)
            .map_err(|_| {
                diagnostics.push("player y read failed".to_string());
            })
            .ok();
        let z = read_f32(handle, player_base + WOTLK_3_3_5A.player_xyz_offset + 0x8)
            .map_err(|_| {
                diagnostics.push("player z read failed".to_string());
            })
            .ok();
        let position = match (x, y, z) {
            (Some(px), Some(py), Some(pz)) => Some((px, py, pz)),
            _ => None,
        };

        let heading_rad = read_f32(handle, player_base + WOTLK_3_3_5A.player_heading_offset)
            .map_err(|_| {
                diagnostics.push("player heading read failed".to_string());
            })
            .ok();

        let player_storage = read_u32(handle, player_base + WOTLK_3_3_5A.object_storage_ptr_offset)
            .map(|storage| storage as usize)
            .ok()
            .filter(|storage| *storage != 0);
        diagnostics.push(format!(
            "trace player_storage={}",
            player_storage
                .map(|value| format!("0x{value:08X}"))
                .unwrap_or_else(|| "none".to_string())
        ));

        let player_guid = read_u64(handle, player_base + WOTLK_3_3_5A.object_guid_offset)
            .ok()
            .or(local_guid)
            .filter(|guid| *guid != 0);

        let player_name = player_guid
            .and_then(|guid| lookup_name_by_guid(handle, guid, &mut diagnostics))
            .unwrap_or_else(|| "Unknown".to_string());

        let Some(player_storage) = player_storage else {
            diagnostics.push("player storage pointer unavailable".to_string());
            return Ok(MemorySnapshot {
                player_name,
                player_guid,
                player_current_health: None,
                player_max_health: None,
                player_race: None,
                player_level: None,
                player_faction: None,
                position,
                heading_rad,
                target_guid: None,
                target_name: None,
                target_current_health: None,
                target_max_health: None,
                target_race: None,
                target_level: None,
                target_faction: None,
                target_hostility: None,
                target_distance: None,
                diagnostics,
            });
        };

        let (player_current_health, player_max_health) = read_health_pair_with_fallback(
            handle,
            player_storage,
            WOTLK_3_3_5A.player_current_health_offset,
            WOTLK_3_3_5A.player_max_health_offset,
            "player",
            &mut diagnostics,
        );

        let player_bytes0 = read_u32(handle, player_storage + WOTLK_3_3_5A.unit_field_bytes0_offset).ok();
        let player_race = player_bytes0.map(|bytes| (bytes & 0xFF) as u8);
        let player_level = read_u32(handle, player_storage + WOTLK_3_3_5A.unit_field_level_offset).ok();
        let player_faction =
            read_u32(handle, player_storage + WOTLK_3_3_5A.unit_field_faction_template_offset).ok();

        let target_guid = read_u64(handle, WOTLK_3_3_5A.target_guid_addr)
            .map_err(|_| {
                diagnostics.push("target guid read failed".to_string());
            })
            .ok()
            .filter(|guid| *guid != 0);

        let target_base = target_guid.and_then(|guid| {
            find_object_by_guid(
                handle,
                object_manager,
                guid,
                "target",
                &mut diagnostics,
            )
        });

        if target_guid.is_some() && target_base.is_none() {
            diagnostics.push("target selected but target object was not found in object list".to_string());
        }

        let target_storage = target_base.and_then(|base| {
            read_u32(handle, base + WOTLK_3_3_5A.object_storage_ptr_offset)
                .map(|storage| storage as usize)
                .ok()
                .filter(|storage| *storage != 0)
        });

        if target_base.is_some() && target_storage.is_none() {
            diagnostics.push("target storage pointer unavailable".to_string());
        }

        let (target_current_health, target_max_health) = if let Some(storage) = target_storage {
            read_health_pair_with_fallback(
                handle,
                storage,
                WOTLK_3_3_5A.target_current_health_offset,
                WOTLK_3_3_5A.target_max_health_offset,
                "target",
                &mut diagnostics,
            )
        } else {
            (None, None)
        };

        let target_bytes0 = target_storage
            .and_then(|storage| read_u32(handle, storage + WOTLK_3_3_5A.unit_field_bytes0_offset).ok());
        let target_race = target_bytes0.map(|bytes| (bytes & 0xFF) as u8);
        let target_level = target_storage
            .and_then(|storage| read_u32(handle, storage + WOTLK_3_3_5A.unit_field_level_offset).ok());
        let target_faction = target_storage.and_then(|storage| {
            read_u32(handle, storage + WOTLK_3_3_5A.unit_field_faction_template_offset).ok()
        });

        let target_name = if let Some(guid) = target_guid {
            lookup_name_by_guid(handle, guid, &mut diagnostics)
                .or_else(|| target_base.and_then(|base| lookup_monster_name(handle, base, &mut diagnostics)))
        } else {
            None
        };

        let target_hostility = if let Some(target_base_addr) = target_base {
            get_hostility_label(handle, player_base, target_base_addr, &mut diagnostics)
                .map(|value| value.to_string())
        } else {
            None
        };

        let target_distance = if let (Some((px, py, pz)), Some(base)) = (position, target_base) {
            let tx = read_f32(handle, base + WOTLK_3_3_5A.player_xyz_offset).ok();
            let ty = read_f32(handle, base + WOTLK_3_3_5A.player_xyz_offset + 0x4).ok();
            let tz = read_f32(handle, base + WOTLK_3_3_5A.player_xyz_offset + 0x8).ok();

            match (tx, ty, tz) {
                (Some(tx), Some(ty), Some(tz)) => {
                    let dx = tx - px;
                    let dy = ty - py;
                    let dz = tz - pz;
                    Some((dx * dx + dy * dy + dz * dz).sqrt())
                }
                _ => {
                    diagnostics.push("target position read failed".to_string());
                    None
                }
            }
        } else {
            None
        };

        Ok(MemorySnapshot {
            player_name,
            player_guid,
            player_current_health,
            player_max_health,
            player_race,
            player_level,
            player_faction,
            position,
            heading_rad,
            target_guid,
            target_name,
            target_current_health,
            target_max_health,
            target_race,
            target_level,
            target_faction,
            target_hostility,
            target_distance,
            diagnostics,
        })
    }
}

fn find_object_by_guid(
    handle: HANDLE,
    object_manager: usize,
    target_guid: u64,
    purpose: &str,
    diagnostics: &mut Vec<String>,
) -> Option<usize> {
    let first = read_u32(handle, object_manager + WOTLK_3_3_5A.first_object_offset)
        .map(|value| value as usize)
        .ok()
        .filter(|value| *value != 0);

    let mut current = match first {
        Some(value) => value,
        None => {
            diagnostics.push("first object pointer unavailable".to_string());
            return None;
        }
    };

    let mut steps = 0usize;
    while current != 0 && steps < 4096 {
        if let Ok(guid) = read_u64(handle, current + WOTLK_3_3_5A.object_guid_offset) {
            if guid == target_guid {
                diagnostics.push(format!(
                    "trace {purpose} guid matched object=0x{current:08X} after {steps} steps"
                ));
                return Some(current);
            }
        }

        let next = read_u32(handle, current + WOTLK_3_3_5A.next_object_offset)
            .map(|value| value as usize)
            .unwrap_or(0);

        if next == 0 || next == current {
            break;
        }

        current = next;
        steps += 1;
    }

    diagnostics.push(format!(
        "{purpose} guid 0x{target_guid:016X} not found during object list traversal"
    ));
    None
}

fn read_health_pair_with_fallback(
    handle: HANDLE,
    storage: usize,
    configured_current_offset: Option<usize>,
    configured_max_offset: Option<usize>,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> (Option<u32>, Option<u32>) {
    let mut candidates: Vec<(usize, usize, &str)> = Vec::new();
    if let (Some(cur), Some(max)) = (configured_current_offset, configured_max_offset) {
        candidates.push((cur, max, "configured"));
        candidates.push((cur * 4, max * 4, "configured_x4"));
    }
    candidates.push((0x60, 0x80, "legacy_descriptor_fields"));
    candidates.push((0x58, 0x70, "wowdev_unit_fields"));
    candidates.push((0x58 * 4, 0x70 * 4, "wowdev_unit_fields_x4"));
    candidates.push((0x6C, 0x74, "legacy_constants"));
    candidates.push((0x6C * 4, 0x74 * 4, "legacy_constants_x4"));

    candidates.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

    let mut best: Option<(i32, u32, u32, &str)> = None;

    for (cur_off, max_off, source) in candidates {
        let cur = read_u32(handle, storage + cur_off).ok();
        let max = read_u32(handle, storage + max_off).ok();

        diagnostics.push(format!(
            "trace {label}_hp_probe source={source} cur_off=0x{cur_off:X} max_off=0x{max_off:X} cur={:?} max={:?}",
            cur,
            max
        ));

        if let (Some(cur), Some(max)) = (cur, max) {
            if max > 0 && cur <= max && max <= 1_000_000 {
                let mut score = 0;
                if cur > 0 {
                    score += 15;
                } else {
                    score -= 10;
                }
                if max >= 50 {
                    score += 3;
                }
                if cur <= max {
                    score += 2;
                }
                if max > 0 && max <= 500_000 {
                    score += 2;
                }
                if source.contains("x4") {
                    score -= 2;
                } else {
                    score += 3;
                }
                if source == "legacy_descriptor_fields" {
                    score += 4;
                }

                let replace = best
                    .as_ref()
                    .map(|(best_score, _, _, _)| score > *best_score)
                    .unwrap_or(true);

                if replace {
                    best = Some((score, cur, max, source));
                }
            }
        }
    }

    if let Some((score, cur, max, source)) = best {
        diagnostics.push(format!(
            "trace {label}_hp_probe selected source={source} score={score} value={cur}/{max}"
        ));
        return (Some(cur), Some(max));
    }

    diagnostics.push(format!("{label} health read failed all candidate offsets"));
    (None, None)
}

fn lookup_name_by_guid(handle: HANDLE, guid: u64, diagnostics: &mut Vec<String>) -> Option<String> {
    let names_store_base = WOTLK_3_3_5A.player_names_store_addr + 0x8;
    let first_node_addr = names_store_base + WOTLK_3_3_5A.player_names_first_node_offset;

    let mut node = read_u32(handle, first_node_addr)
        .map(|value| value as usize)
        .ok()
        .filter(|value| *value != 0)?;

    let mut steps = 0usize;
    while node != 0 && (node & 1) == 0 && node != 28 && steps < 16384 {
        let guid_primary = read_u64(handle, node + WOTLK_3_3_5A.player_names_guid_offset_primary)
            .ok()
            .unwrap_or(0);
        let guid_fallback = read_u64(handle, node + WOTLK_3_3_5A.player_names_guid_offset_fallback)
            .ok()
            .unwrap_or(0);
        let node_guid = if guid_primary != 0 {
            guid_primary
        } else {
            guid_fallback
        };

        if node_guid == guid {
            if let Ok(name) = read_c_string(
                handle,
                node + WOTLK_3_3_5A.player_names_name_offset,
                32,
            ) {
                if !name.is_empty() {
                    diagnostics.push(format!(
                        "trace name_lookup matched guid=0x{guid:016X} name={name}"
                    ));
                    return Some(name);
                }
            }
        }

        let next = read_u32(handle, node + WOTLK_3_3_5A.player_names_next_offset)
            .map(|value| value as usize)
            .unwrap_or(0);
        if next == 0 || next == node {
            break;
        }

        node = next;
        steps += 1;
    }

    None
}

fn lookup_monster_name(handle: HANDLE, base: usize, diagnostics: &mut Vec<String>) -> Option<String> {
    let monster_def = read_u32(handle, base + WOTLK_3_3_5A.monster_definition_offset)
        .map(|value| value as usize)
        .ok()
        .filter(|value| *value != 0)?;

    let name_ptr = read_u32(handle, monster_def + WOTLK_3_3_5A.unit_name_second_offset)
        .map(|value| value as usize)
        .ok()
        .filter(|value| *value != 0)?;

    match read_c_string(handle, name_ptr, 64) {
        Ok(name) if !name.is_empty() => {
            diagnostics.push(format!(
                "trace monster_name_lookup matched base=0x{base:08X} name={name}"
            ));
            Some(name)
        }
        _ => None,
    }
}

fn get_hostility_label(
    handle: HANDLE,
    my_base: usize,
    other_base: usize,
    diagnostics: &mut Vec<String>,
) -> Option<&'static str> {
    let my_row = get_faction_group_row(handle, my_base)?;
    let other_row = get_faction_group_row(handle, other_base)?;

    diagnostics.push(format!(
        "trace faction_rows mine=0x{my_row:08X} other=0x{other_row:08X}"
    ));

    let my_flags = read_u32(handle, my_row + 12).unwrap_or(0);
    let my_group = read_u32(handle, my_row + 4).unwrap_or(0);
    let other_hostile_mask = read_u32(handle, other_row + 20).unwrap_or(0);
    let other_friendly_mask = read_u32(handle, other_row + 16).unwrap_or(0);

    if (my_flags & other_hostile_mask) > 0 {
        return Some("Hostile");
    }

    for i in 0..4 {
        let val = read_u32(handle, other_row + 24 + i * 4).unwrap_or(0);
        if val == 0 {
            break;
        }
        if val == my_group {
            return Some("Hostile");
        }
    }

    if (my_flags & other_friendly_mask) > 0 {
        return Some("Friendly");
    }

    for i in 0..4 {
        let val = read_u32(handle, other_row + 40 + i * 4).unwrap_or(0);
        if val == 0 {
            break;
        }
        if val == my_group {
            return Some("Friendly");
        }
    }

    let my_hostile_mask = read_u32(handle, my_row + 20).unwrap_or(0);
    let other_flags = read_u32(handle, other_row + 12).unwrap_or(0);
    if (my_hostile_mask & other_flags) > 0 {
        return Some("Friendly");
    }

    let other_group = read_u32(handle, other_row + 4).unwrap_or(0);
    for i in 0..4 {
        let val = read_u32(handle, my_row + 40 + i * 4).unwrap_or(0);
        if val == 0 {
            break;
        }
        if val == other_group {
            return Some("Friendly");
        }
    }

    Some("Neutral")
}

fn get_faction_group_row(handle: HANDLE, base: usize) -> Option<usize> {
    let faction_sub = read_u32(handle, WOTLK_3_3_5A.faction_sub_addr).ok()?;
    let faction_off1 = read_u32(handle, base + WOTLK_3_3_5A.faction_off1_offset).ok()?;
    if faction_sub == 0 || faction_off1 == 0 {
        return None;
    }

    let faction_off2 = read_u32(
        handle,
        faction_off1 as usize + WOTLK_3_3_5A.faction_off2_offset,
    )
    .ok()?;
    let faction_base = read_u32(handle, WOTLK_3_3_5A.faction_base_addr).ok()?;
    if faction_off2 == 0 || faction_base == 0 {
        return None;
    }

    if faction_off2 < faction_sub {
        return None;
    }

    let row_delta = faction_off2 - faction_sub;
    if row_delta > 131_072 {
        return None;
    }

    let row = read_u32(handle, faction_base as usize + row_delta as usize * 4).ok()?;
    if row == 0 {
        return None;
    }

    Some(row as usize)
}

impl Drop for MemoryReader {
    fn drop(&mut self) {
        self.detach();
    }
}

fn read_u64(handle: HANDLE, address: usize) -> Result<u64, MemoryReaderError> {
    read_value::<u64>(handle, address)
}

fn read_u32(handle: HANDLE, address: usize) -> Result<u32, MemoryReaderError> {
    read_value::<u32>(handle, address)
}

fn read_f32(handle: HANDLE, address: usize) -> Result<f32, MemoryReaderError> {
    read_value::<f32>(handle, address)
}

fn read_c_string(handle: HANDLE, address: usize, max_len: usize) -> Result<String, MemoryReaderError> {
    let bytes = read_bytes(handle, address, max_len)?;
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    Ok(String::from_utf8_lossy(&bytes[..end]).trim().to_string())
}

fn read_bytes(handle: HANDLE, address: usize, len: usize) -> Result<Vec<u8>, MemoryReaderError> {
    let mut buffer = vec![0u8; len];
    let mut bytes_read: usize = 0;

    let success = unsafe {
        ReadProcessMemory(
            handle,
            address as *const core::ffi::c_void,
            buffer.as_mut_ptr().cast::<core::ffi::c_void>(),
            len,
            Some(&mut bytes_read),
        )
        .is_ok()
    };

    if !success || bytes_read == 0 {
        return Err(MemoryReaderError::ReadMemoryFailed(address));
    }

    buffer.truncate(bytes_read);
    Ok(buffer)
}

fn read_value<T: Copy + Default>(handle: HANDLE, address: usize) -> Result<T, MemoryReaderError> {
    let mut value: T = T::default();
    let mut bytes_read: usize = 0;

    let success = unsafe {
        ReadProcessMemory(
            handle,
            address as *const core::ffi::c_void,
            (&mut value as *mut T).cast::<core::ffi::c_void>(),
            std::mem::size_of::<T>(),
            Some(&mut bytes_read),
        )
        .is_ok()
    };

    if !success || bytes_read != std::mem::size_of::<T>() {
        return Err(MemoryReaderError::ReadMemoryFailed(address));
    }

    Ok(value)
}

fn utf16_to_string(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

fn is_wow_process_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "wow.exe" || lower == "wow-64.exe"
}

#[cfg(test)]
mod tests {
    use super::is_wow_process_name;

    #[test]
    fn detects_wow_process_names_case_insensitive() {
        assert!(is_wow_process_name("WoW.exe"));
        assert!(is_wow_process_name("wow.exe"));
        assert!(is_wow_process_name("wow-64.exe"));
        assert!(!is_wow_process_name("notepad.exe"));
    }
}
