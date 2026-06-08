use crate::workspace::GitState;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitIndexEntry {
    modified: Option<(u64, u32)>,
    path: String,
    size: u32,
}

pub fn detect_git(root: &Path) -> GitState {
    let git_dir = root.join(".git");
    let branch = fs::read_to_string(git_dir.join("HEAD"))
        .ok()
        .and_then(|head| {
            head.strip_prefix("ref: refs/heads/")
                .map(|branch| branch.trim().to_string())
        });

    GitState {
        is_repo: git_dir.exists(),
        branch,
        uncommitted_changes: dirty_count(root, &git_dir),
    }
}

pub(crate) fn dirty_count(root: &Path, git_dir: &Path) -> Option<usize> {
    if !git_dir.is_dir() {
        return None;
    }
    let entries = read_index(&git_dir.join("index")).ok()?;
    let tracked = entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<HashSet<_>>();
    let tracked_changes = entries
        .iter()
        .filter(|entry| entry_is_dirty(root, entry))
        .count();
    let untracked = count_untracked(root, root, &tracked).ok()?;
    Some(tracked_changes + untracked)
}

fn entry_is_dirty(root: &Path, entry: &GitIndexEntry) -> bool {
    let path = root.join(path_from_git(&entry.path));
    let Ok(metadata) = fs::metadata(path) else {
        return true;
    };
    if !metadata.is_file() || metadata.len() != u64::from(entry.size) {
        return true;
    }
    entry
        .modified
        .is_some_and(|stamp| modified_stamp(&metadata) != Some(stamp))
}

fn count_untracked(
    root: &Path,
    current: &Path,
    tracked: &HashSet<String>,
) -> std::io::Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = entry.file_type()?;
        if skip_git_scan(&name) || file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            count += count_untracked(root, &path, tracked)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !tracked.contains(&relative) {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn read_index(path: &Path) -> std::io::Result<Vec<GitIndexEntry>> {
    let data = fs::read(path)?;
    if data.len() < 12 || &data[0..4] != b"DIRC" {
        return Err(invalid_index());
    }
    let version = read_u32(&data, 4)?;
    if ![2, 3].contains(&version) {
        return Err(invalid_index());
    }
    let count = read_u32(&data, 8)? as usize;
    let mut offset = 12;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let entry_start = offset;
        if offset + 62 > data.len() {
            return Err(invalid_index());
        }
        let modified = Some((
            u64::from(read_u32(&data, offset + 8)?),
            read_u32(&data, offset + 12)?,
        ));
        let size = read_u32(&data, offset + 36)?;
        let flags = read_u16(&data, offset + 60)?;
        offset += 62;
        if flags & 0x4000 != 0 {
            offset += 2;
        }
        let path_start = offset;
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        if offset >= data.len() {
            return Err(invalid_index());
        }
        let path = String::from_utf8_lossy(&data[path_start..offset]).to_string();
        offset += 1;
        while (offset - entry_start) % 8 != 0 {
            offset += 1;
        }
        entries.push(GitIndexEntry {
            modified,
            path,
            size,
        });
    }
    Ok(entries)
}

fn modified_stamp(metadata: &fs::Metadata) -> Option<(u64, u32)> {
    let duration = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some((duration.as_secs(), duration.subsec_nanos()))
}

fn path_from_git(path: &str) -> PathBuf {
    path.split('/').collect()
}

fn skip_git_scan(name: &str) -> bool {
    matches!(name, ".git" | ".tools" | "node_modules" | "target" | "dist")
}

fn read_u32(data: &[u8], offset: usize) -> std::io::Result<u32> {
    Ok(u32::from_be_bytes(read_fixed(data, offset)?))
}

fn read_u16(data: &[u8], offset: usize) -> std::io::Result<u16> {
    Ok(u16::from_be_bytes(read_fixed(data, offset)?))
}

fn read_fixed<const N: usize>(data: &[u8], offset: usize) -> std::io::Result<[u8; N]> {
    let end = offset.checked_add(N).ok_or_else(invalid_index)?;
    data.get(offset..end)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(invalid_index)
}

fn invalid_index() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, "unsupported Git index")
}
