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

        // Validate basic read access using a known static address.
        // This is a guard rail to ensure the process handle is usable.
        let test_addr = WOTLK_3_3_5A.player_guid;
        let _ = read_u64(handle, test_addr)?;

        // Placeholder values until concrete offset mapping is implemented.
        Ok(MemorySnapshot {
            player_name: "Unknown".to_string(),
            player_health: 100,
            position: (0.0, 0.0, 0.0),
            target_guid: None,
        })
    }
}

impl Drop for MemoryReader {
    fn drop(&mut self) {
        self.detach();
    }
}

fn read_u64(handle: HANDLE, address: usize) -> Result<u64, MemoryReaderError> {
    let mut value: u64 = 0;
    let mut bytes_read: usize = 0;

    let success = unsafe {
        ReadProcessMemory(
            handle,
            address as *const core::ffi::c_void,
            (&mut value as *mut u64).cast::<core::ffi::c_void>(),
            std::mem::size_of::<u64>(),
            Some(&mut bytes_read),
        )
        .is_ok()
    };

    if !success || bytes_read != std::mem::size_of::<u64>() {
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
