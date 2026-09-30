use crate::manifest::read_manifest;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreHistoryItem {
    pub path: String,
    pub file_name: String,
    pub status: String,
    pub created_at: String,
    pub modified_unix_ms: u128,
    pub files: usize,
    pub root: Option<String>,
    pub matches_selected_root: bool,
    pub valid: bool,
    pub error: Option<String>,
}

fn manifests_dir(root: &Path) -> PathBuf {
    root.parent()
        .unwrap_or(root)
        .join("S3CC Organizer")
        .join("Restore Manifests")
}

#[tauri::command]
pub fn list_restore_history(folder: String) -> Result<Vec<RestoreHistoryItem>, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve restore root: {error}"))?;
    let directory = manifests_dir(&root);

    if !directory.is_dir() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();

    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("Could not list {}: {error}", directory.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file()
            || !path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case("txt"))
                .unwrap_or(false)
        {
            continue;
        }

        let metadata = fs::metadata(&path).ok();
        let modified_unix_ms = metadata
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis())
            .unwrap_or(0);

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("manifest.txt")
            .to_string();

        match read_manifest(&path) {
            Ok(manifest) => {
                let manifest_root = manifest.root.canonicalize().ok();
                items.push(RestoreHistoryItem {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    status: manifest.status,
                    created_at: manifest.created_at,
                    modified_unix_ms,
                    files: manifest.entries.len(),
                    root: Some(manifest.root.to_string_lossy().to_string()),
                    matches_selected_root: manifest_root
                        .as_ref()
                        .map(|manifest_root| manifest_root == &root)
                        .unwrap_or(false),
                    valid: true,
                    error: None,
                });
            }
            Err(error) => items.push(RestoreHistoryItem {
                path: path.to_string_lossy().to_string(),
                file_name,
                status: "INVALID".to_string(),
                created_at: String::new(),
                modified_unix_ms,
                files: 0,
                root: None,
                matches_selected_root: false,
                valid: false,
                error: Some(error),
            }),
        }
    }

    items.sort_by_key(|item| std::cmp::Reverse(item.modified_unix_ms));
    Ok(items)
}
