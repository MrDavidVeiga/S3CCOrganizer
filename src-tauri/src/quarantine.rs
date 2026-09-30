use crate::{
    manifest::sha256_file,
    workspace::{ensure_writable, load_workspace_for_root},
};
use chrono::Local;
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineResult {
    pub moved: usize,
    pub quarantine_root: String,
    pub manifest_path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
struct QuarantineManifest<'a> {
    version: u32,
    created_at: String,
    status: &'a str,
    root: &'a str,
    quarantine_root: &'a str,
    items: &'a [QuarantinePlanItem],
}

fn within_root(root: &Path, candidate: &Path) -> bool {
    candidate.starts_with(root)
}

fn relative_text(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

fn write_manifest_atomic(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Quarantine manifest has no parent directory.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create quarantine manifest directory: {error}"))?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, data)
        .map_err(|error| format!("Could not write quarantine manifest temp file: {error}"))?;
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("Could not replace quarantine manifest: {error}"))?;
    }
    fs::rename(&temp, path)
        .map_err(|error| format!("Could not commit quarantine manifest: {error}"))
}

fn persist_manifest(path: &Path, plan: &QuarantinePlan, status: &str) -> Result<(), String> {
    let manifest = QuarantineManifest {
        version: 1,
        created_at: Local::now().to_rfc3339(),
        status,
        root: &plan.root,
        quarantine_root: &plan.quarantine_root,
        items: &plan.items,
    };
    let data = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("Could not serialize quarantine manifest: {error}"))?;
    write_manifest_atomic(path, &data)
}

fn rollback_moves(moved: &[(PathBuf, PathBuf, String, u64)]) -> bool {
    let mut ok = true;
    for (source, destination, hash, size) in moved.iter().rev() {
        if !destination.exists() || source.exists() {
            ok = false;
            continue;
        }
        if fs::rename(destination, source).is_err() {
            ok = false;
            continue;
        }
        match sha256_file(source) {
            Ok((actual_hash, actual_size))
                if actual_hash.eq_ignore_ascii_case(hash) && actual_size == *size => {}
            _ => ok = false,
        }
    }
    ok
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
    let session = Local::now().format("%Y%m%d-%H%M%S-%3f").to_string();
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
        if !source
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("package"))
            .unwrap_or(false)
        {
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
                vec!["Quarantine destination already exists. Execution is blocked.".to_string()]
            } else {
                Vec::new()
            },
        });
    }

    items.sort_by_key(|item| item.source_relative_path.to_ascii_lowercase());

    let mut manifest_preview = String::new();
    manifest_preview.push_str("S3CC ORGANIZER QUARANTINE PREVIEW\n");
    manifest_preview.push_str("version=1\n");
    manifest_preview.push_str("mode=PREVIEW\n");
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

    let read_only = load_workspace_for_root(&root).read_only;
    Ok(QuarantinePlan {
        root: root.to_string_lossy().to_string(),
        quarantine_root: quarantine_root.to_string_lossy().to_string(),
        items,
        can_execute: !read_only
            && stats.ready > 0
            && stats.blocked == 0
            && stats.collisions == 0,
        stats,
        manifest_preview,
    })
}

#[tauri::command]
pub fn execute_quarantine(
    folder: String,
    selected_paths: Vec<String>,
) -> Result<QuarantineResult, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve Mods root: {error}"))?;
    ensure_writable(&root)?;

    let plan = build_quarantine_plan(folder, selected_paths)?;
    if !plan.can_execute {
        return Err("Quarantine is blocked by the preflight plan.".to_string());
    }

    let manifest_dir = root
        .parent()
        .unwrap_or(&root)
        .join("S3CC Organizer")
        .join("Quarantine Manifests");
    let manifest_path = manifest_dir.join(format!(
        "Quarantine-{}.json",
        Local::now().format("%Y%m%d-%H%M%S-%3f")
    ));
    persist_manifest(&manifest_path, &plan, "PENDING")?;

    let mut moved = Vec::<(PathBuf, PathBuf, String, u64)>::new();

    for item in &plan.items {
        let source = PathBuf::from(&item.source_path);
        let destination = PathBuf::from(&item.destination_path);

        if !source.exists() || destination.exists() {
            let rolled_back = rollback_moves(&moved);
            let _ = persist_manifest(
                &manifest_path,
                &plan,
                if rolled_back { "ROLLED_BACK" } else { "ROLLBACK_INCOMPLETE" },
            );
            return Err(format!(
                "Quarantine preflight changed before execution: {}",
                source.display()
            ));
        }

        let (source_hash, source_size) = sha256_file(&source)
            .map_err(|error| format!("Could not verify source before quarantine: {error}"))?;
        if !source_hash.eq_ignore_ascii_case(&item.sha256) || source_size != item.size {
            let rolled_back = rollback_moves(&moved);
            let _ = persist_manifest(
                &manifest_path,
                &plan,
                if rolled_back { "ROLLED_BACK" } else { "ROLLBACK_INCOMPLETE" },
            );
            return Err(format!("Source changed after preview: {}", source.display()));
        }

        let parent = destination
            .parent()
            .ok_or_else(|| "Quarantine destination has no parent.".to_string())?;
        if let Err(error) = fs::create_dir_all(parent) {
            let rolled_back = rollback_moves(&moved);
            let _ = persist_manifest(
                &manifest_path,
                &plan,
                if rolled_back { "ROLLED_BACK" } else { "ROLLBACK_INCOMPLETE" },
            );
            return Err(format!("Could not create quarantine folder: {error}"));
        }

        if let Err(error) = fs::rename(&source, &destination) {
            let rolled_back = rollback_moves(&moved);
            let _ = persist_manifest(
                &manifest_path,
                &plan,
                if rolled_back { "ROLLED_BACK" } else { "ROLLBACK_INCOMPLETE" },
            );
            return Err(format!("Could not quarantine {}: {error}", source.display()));
        }

        let verified = sha256_file(&destination)
            .map(|(hash, size)| hash.eq_ignore_ascii_case(&item.sha256) && size == item.size)
            .unwrap_or(false);
        if !verified {
            moved.push((
                source.clone(),
                destination.clone(),
                item.sha256.clone(),
                item.size,
            ));
            let rolled_back = rollback_moves(&moved);
            let _ = persist_manifest(
                &manifest_path,
                &plan,
                if rolled_back { "ROLLED_BACK" } else { "ROLLBACK_INCOMPLETE" },
            );
            return Err(format!(
                "Quarantined file failed SHA-256/size verification: {}",
                destination.display()
            ));
        }

        moved.push((
            source,
            destination,
            item.sha256.clone(),
            item.size,
        ));
    }

    persist_manifest(&manifest_path, &plan, "COMPLETE")?;

    Ok(QuarantineResult {
        moved: moved.len(),
        quarantine_root: plan.quarantine_root,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        status: "COMPLETE".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_use_windows_style_for_manifests() {
        assert_eq!(
            relative_text(Path::new("CAS/Hair/test.package")),
            "CAS\\Hair\\test.package"
        );
    }
}
