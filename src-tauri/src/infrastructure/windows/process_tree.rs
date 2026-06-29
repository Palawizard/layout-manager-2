use std::collections::HashMap;

use windows::{
    Win32::Foundation::CloseHandle,
    Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    },
};

const MAX_ANCESTOR_DEPTH: usize = 64;

/// Returns whether `process_id` is the same process as, or a descendant of, `ancestor_id`.
#[must_use]
pub fn is_process_in_tree(process_id: u32, ancestor_id: u32) -> bool {
    if process_id == 0 || ancestor_id == 0 {
        return false;
    }
    if process_id == ancestor_id {
        return true;
    }
    let Ok(parents) = parent_process_map() else {
        return false;
    };
    let mut current = process_id;
    for _ in 0..MAX_ANCESTOR_DEPTH {
        let Some(parent) = parents.get(&current).copied() else {
            return false;
        };
        if parent == ancestor_id {
            return true;
        }
        if parent == current || parent == 0 {
            return false;
        }
        current = parent;
    }
    false
}

fn parent_process_map() -> Result<HashMap<u32, u32>, ()> {
    // SAFETY: TH32CS_SNAPPROCESS only requires a valid snapshot handle to be closed.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.map_err(|_| ())?;
    let mut parents = HashMap::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: `entry` is initialized with dwSize and the snapshot handle is valid.
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while has_entry {
        parents.insert(entry.th32ProcessID, entry.th32ParentProcessID);
        // SAFETY: `entry` remains valid for repeated enumeration on the same snapshot.
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    // SAFETY: The snapshot handle is closed exactly once here.
    let _ = unsafe { CloseHandle(snapshot) };
    Ok(parents)
}

use std::mem::size_of;

#[cfg(test)]
mod tests {
    use super::is_process_in_tree;

    #[test]
    fn treats_a_process_as_its_own_tree_root() {
        let pid = std::process::id();
        assert!(is_process_in_tree(pid, pid));
    }
}
