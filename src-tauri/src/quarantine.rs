use crate::manifest::sha256_file;
use chrono::Local;
use serde::Serialize;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantinePlanItem {
    pub source_path: String,
    pub source_relative_path: String,
    pub destination_path: String,
    pub destination_relative_path: String,
    pub sha256: String,
    pub size: u64,
    pub status: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineStats {
    pub selected: usize,
    pub ready: usize,
    pub blocked: usize,
    pub collisions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantinePlan {
    pub root: String,
    pub quarantine_root: String,
    pub items: Vec<QuarantinePlanItem>,
    pub stats: QuarantineStats,
    pub manifest_preview: String,
    pub can_execute: bool,
}

fn within_root(root: &Path, candidate: &Path) -> bool {
    candidate.starts_with(root)
}

fn relative_text(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\")
}

#[tauri::command]
pub fn build_quarantine_plan(
    folder: String,
    selected_paths: Vec<String>,
) -> Result<QuarantinePlan, String> {
    if selected_paths.is_empty() {
        return Err("No duplicate files were selected for quarantine preview.".to_string());
    }

    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve Mods root: {error}"))?;
    if !root.is_dir() {
        return Err(format!("Selected root is not a directory: {}", root.display()));
    }

    let quarantine_base = root
        .parent()
        .unwrap_or(&root)
        .join("S3CC Organizer")
        .join("Quarantine");
    let session = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let quarantine_root = quarantine_base.join(session);

    let mut unique = HashSet::<PathBuf>::new();
    let mut items = Vec::new();
    let mut stats = QuarantineStats::default();

    for raw in selected_paths {
        let source = PathBuf::from(&raw)
            .canonicalize()
            .map_err(|error| format!("Could not resolve selected package {raw}: {error}"))?;

        if !within_root(&root, &source) || !source.is_file() {
            stats.blocked += 1;
            continue;
        }
        if !unique.insert(source.clone()) {
            continue;
        }

        stats.selected += 1;
        let relative = source
            .strip_prefix(&root)
            .map_err(|_| format!("Could not make relative path for {}", source.display()))?;
        let destination = quarantine_root.join(relative);
        let (sha256, size) = sha256_file(&source)
            .map_err(|error| format!("Could not hash {}: {error}", source.display()))?;

        let collision = destination.exists();
        if collision {
            stats.collisions += 1;
            stats.blocked += 1;
        } else {
            stats.ready += 1;
        }

        items.push(QuarantinePlanItem {
            source_path: source.to_string_lossy().to_string(),
            source_relative_path: relative_text(relative),
            destination_path: destination.to_string_lossy().to_string(),
            destination_relative_path: relative_text(relative),
            sha256,
            size,
            status: if collision { "collision" } else { "ready" }.to_string(),
            warnings: if collision {
                vec!["Quarantine destination already exists. Execution would be blocked.".to_string()]
            } else {
                Vec::new()
            },
        });
    }

    items.sort_by_key(|item| item.source_relative_path.to_ascii_lowercase());

    let mut manifest_preview = String::new();
    manifest_preview.push_str("S3CC ORGANIZER QUARANTINE PREVIEW\n");
    manifest_preview.push_str("version=1\n");
    manifest_preview.push_str("mode=PREVIEW_ONLY\n");
    manifest_preview.push_str(&format!("root={}\n", root.display()));
    manifest_preview.push_str(&format!("quarantine_root={}\n", quarantine_root.display()));
    manifest_preview.push_str(&format!("files={}\n\n", items.len()));
    for item in &items {
        manifest_preview.push_str("[file]\n");
        manifest_preview.push_str(&format!("sha256={}\n", item.sha256));
        manifest_preview.push_str(&format!("size={}\n", item.size));
        manifest_preview.push_str(&format!("original={}\n", item.source_relative_path));
        manifest_preview.push_str(&format!("quarantine={}\n", item.destination_relative_path));
        manifest_preview.push_str("[/file]\n\n");
    }

    Ok(QuarantinePlan {
        root: root.to_string_lossy().to_string(),
        quarantine_root: quarantine_root.to_string_lossy().to_string(),
        items,
        stats,
        manifest_preview,
        // Deliberately preview-only until real-package/runtime validation is complete.
        can_execute: false,
    })
}
