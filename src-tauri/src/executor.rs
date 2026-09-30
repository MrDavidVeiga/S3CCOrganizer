use crate::{
    i18n::AppLanguage,
    manifest::{
        replace_manifest_atomic, sha256_file, write_manifest_atomic, RestoreEntry, RestoreManifest,
    },
    planner::{build_organization_plan, PlanItem},
};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub status: String,
    pub manifest_path: Option<String>,
    pub moved: usize,
    pub already_organized: usize,
    pub rolled_back: usize,
    pub errors: Vec<String>,
}

fn make_manifest_path(root: &Path) -> Result<PathBuf, String> {
    let base = root
        .parent()
        .unwrap_or(root)
        .join("S3CC Organizer")
        .join("Restore Manifests");

    fs::create_dir_all(&base)
        .map_err(|error| format!("Could not create restore manifest directory {}: {error}", base.display()))?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    for suffix in 0..10_000usize {
        let file_name = if suffix == 0 {
            format!("S3CC-Organizer-Restore-{stamp}.txt")
        } else {
            format!("S3CC-Organizer-Restore-{stamp}-{suffix}.txt")
        };
        let candidate = base.join(file_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Could not allocate a unique restore manifest filename.".to_string())
}

fn ready_items(plan_items: &[PlanItem]) -> Vec<&PlanItem> {
    plan_items
        .iter()
        .filter(|item| item.plan_status == "ready")
        .collect()
}

fn package_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("package"))
        .unwrap_or(false)
}

fn snapshot_entries(root: &Path, ready: &[&PlanItem]) -> Result<Vec<RestoreEntry>, String> {
    let mut planned = HashMap::<PathBuf, (&PlanItem, PathBuf)>::new();

    for &item in ready {
        let source = PathBuf::from(&item.source_path)
            .canonicalize()
            .map_err(|error| format!("Could not resolve planned source {}: {error}", item.source_path))?;
        let destination = item
            .destination_relative_path
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| format!("Planner item {} is missing destination.", item.name))?;
        planned.insert(source, (item, destination));
    }

    let mut entries = Vec::new();

    for entry in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = entry.map_err(|error| format!("Could not snapshot package tree: {error}"))?;
        if !entry.file_type().is_file() || !package_extension(entry.path()) {
            continue;
        }

        let absolute = entry
            .path()
            .canonicalize()
            .map_err(|error| format!("Could not resolve {}: {error}", entry.path().display()))?;
        let original_relative = absolute
            .strip_prefix(root)
            .map_err(|_| format!("Snapshot file escaped root: {}", absolute.display()))?
            .to_path_buf();

        let (sha256, size) = sha256_file(&absolute)
            .map_err(|error| format!("Could not hash snapshot file {}: {error}", absolute.display()))?;

        let organized_relative = if let Some((item, destination)) = planned.get(&absolute) {
            let expected_hash = item
                .sha256
                .as_ref()
                .ok_or_else(|| format!("Planner item {} is missing SHA-256.", item.name))?;
            if item.size != size || !expected_hash.eq_ignore_ascii_case(&sha256) {
                return Err(format!(
                    "File changed between Planner and manifest snapshot: {}",
                    absolute.display()
                ));
            }
            destination.clone()
        } else {
            original_relative.clone()
        };

        entries.push(RestoreEntry {
            sha256,
            size,
            original_relative_path: original_relative,
            organized_relative_path: organized_relative,
        });
    }

    entries.sort_by_key(|entry| {
        entry
            .original_relative_path
            .to_string_lossy()
            .to_ascii_lowercase()
    });

    Ok(entries)
}

fn manifest_from_snapshot(
    root: &Path,
    language: AppLanguage,
    entries: Vec<RestoreEntry>,
    created_directories: &[String],
) -> RestoreManifest {
    RestoreManifest {
        version: 1,
        created_at: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        organization_language: language,
        root: root.to_path_buf(),
        status: "PENDING".to_string(),
        created_directories: created_directories.iter().map(PathBuf::from).collect(),
        entries,
    }
}

fn verify_identity(path: &Path, expected_hash: &str, expected_size: u64) -> Result<(), String> {
    let (hash, size) = sha256_file(path)
        .map_err(|error| format!("Could not hash {}: {error}", path.display()))?;
    if size != expected_size || !hash.eq_ignore_ascii_case(expected_hash) {
        return Err(format!(
            "Identity changed for {} (expected size/hash {} / {}, got {} / {}).",
            path.display(),
            expected_size,
            expected_hash,
            size,
            hash
        ));
    }
    Ok(())
}

fn remove_empty_created_directories(root: &Path, directories: &[String]) {
    let mut ordered = directories
        .iter()
        .map(|relative| root.join(relative))
        .collect::<Vec<_>>();
    ordered.sort_by_key(|path| std::cmp::Reverse(path.components().count()));

    for path in ordered {
        if path.is_dir() {
            let _ = fs::remove_dir(&path);
        }
    }
}

#[tauri::command]
pub fn execute_organization(
    folder: String,
    language: AppLanguage,
    selected_paths: Vec<String>,
) -> Result<ExecutionResult, String> {
    let plan = build_organization_plan(folder, language, selected_paths)?;

    if plan.stats.blocked > 0
        || plan.stats.collision_same_content > 0
        || plan.stats.collision_different_content > 0
    {
        return Err(format!(
            "Execution blocked by preflight: {} blocked, {} identical collision(s), {} different collision(s).",
            plan.stats.blocked,
            plan.stats.collision_same_content,
            plan.stats.collision_different_content
        ));
    }

    let root = PathBuf::from(&plan.root);
    let ready = ready_items(&plan.items);

    if ready.is_empty() {
        return Ok(ExecutionResult {
            status: "NO_CHANGES".to_string(),
            manifest_path: None,
            moved: 0,
            already_organized: plan.stats.already_organized,
            rolled_back: 0,
            errors: Vec::new(),
        });
    }

    let manifest_path = make_manifest_path(&root)?;
    let snapshot = snapshot_entries(&root, &ready)?;
    let mut manifest = manifest_from_snapshot(
        &root,
        language,
        snapshot,
        &plan.directories_to_create,
    );
    write_manifest_atomic(&manifest_path, &manifest)?;

    let mut moved_pairs: Vec<(PathBuf, PathBuf, String, u64)> = Vec::new();
    let mut errors = Vec::new();

    for item in ready {
        let source = PathBuf::from(&item.source_path);
        let destination = item
            .destination_path
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| format!("Planner item {} has no destination.", item.name))?;
        let expected_hash = item
            .sha256
            .as_ref()
            .ok_or_else(|| format!("Planner item {} has no SHA-256.", item.name))?;

        let step_result = (|| -> Result<(), String> {
            if !source.is_file() {
                return Err(format!("Source disappeared before move: {}", source.display()));
            }
            if destination.exists() {
                return Err(format!(
                    "Destination appeared after preflight; refusing overwrite: {}",
                    destination.display()
                ));
            }

            verify_identity(&source, expected_hash, item.size)?;

            let parent = destination
                .parent()
                .ok_or_else(|| format!("Destination has no parent: {}", destination.display()))?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;

            fs::rename(&source, &destination).map_err(|error| {
                format!(
                    "Could not move {} -> {}: {error}",
                    source.display(),
                    destination.display()
                )
            })?;

            verify_identity(&destination, expected_hash, item.size)?;
            Ok(())
        })();

        match step_result {
            Ok(()) => moved_pairs.push((
                source,
                destination,
                expected_hash.clone(),
                item.size,
            )),
            Err(error) => {
                errors.push(error);
                break;
            }
        }
    }

    if !errors.is_empty() {
        let mut rollback_errors = Vec::new();
        let mut rolled_back = 0usize;

        for (source, destination, expected_hash, expected_size) in moved_pairs.iter().rev() {
            let rollback = (|| -> Result<(), String> {
                if source.exists() {
                    return Err(format!(
                        "Rollback destination already exists: {}",
                        source.display()
                    ));
                }
                if !destination.is_file() {
                    return Err(format!(
                        "Moved file disappeared before rollback: {}",
                        destination.display()
                    ));
                }

                verify_identity(destination, expected_hash, *expected_size)?;

                if let Some(parent) = source.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        format!("Could not recreate original folder {}: {error}", parent.display())
                    })?;
                }

                fs::rename(destination, source).map_err(|error| {
                    format!(
                        "Could not rollback {} -> {}: {error}",
                        destination.display(),
                        source.display()
                    )
                })?;

                verify_identity(source, expected_hash, *expected_size)?;
                Ok(())
            })();

            match rollback {
                Ok(()) => rolled_back += 1,
                Err(error) => rollback_errors.push(error),
            }
        }

        remove_empty_created_directories(&root, &plan.directories_to_create);

        manifest.status = if rollback_errors.is_empty() {
            "ROLLED_BACK".to_string()
        } else {
            "ROLLBACK_INCOMPLETE".to_string()
        };

        if let Err(error) = replace_manifest_atomic(&manifest_path, &manifest) {
            rollback_errors.push(error);
        }

        errors.extend(rollback_errors);

        return Ok(ExecutionResult {
            status: manifest.status.clone(),
            manifest_path: Some(manifest_path.to_string_lossy().to_string()),
            moved: moved_pairs.len(),
            already_organized: plan.stats.already_organized,
            rolled_back,
            errors,
        });
    }

    manifest.status = "COMPLETE".to_string();
    replace_manifest_atomic(&manifest_path, &manifest)?;

    Ok(ExecutionResult {
        status: "COMPLETE".to_string(),
        manifest_path: Some(manifest_path.to_string_lossy().to_string()),
        moved: moved_pairs.len(),
        already_organized: plan.stats.already_organized,
        rolled_back: 0,
        errors: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_ready_items_are_written_to_manifest() {
        let items = vec![
            PlanItem {
                id: "1".into(),
                name: "a.package".into(),
                source_path: r"C:\Mods\a.package".into(),
                source_relative_path: "a.package".into(),
                destination_path: Some(r"C:\Mods\CAS\a.package".into()),
                destination_relative_path: Some(r"CAS\a.package".into()),
                classification_status: "classified".into(),
                classification_reason: Some("test classification".into()),
                plan_status: "ready".into(),
                sha256: Some("A".repeat(64)),
                size: 10,
                warnings: vec![],
            },
            PlanItem {
                id: "2".into(),
                name: "b.package".into(),
                source_path: r"C:\Mods\b.package".into(),
                source_relative_path: "b.package".into(),
                destination_path: Some(r"C:\Mods\CAS\b.package".into()),
                destination_relative_path: Some(r"CAS\b.package".into()),
                classification_status: "classified".into(),
                classification_reason: Some("test classification".into()),
                plan_status: "already_organized".into(),
                sha256: Some("B".repeat(64)),
                size: 20,
                warnings: vec![],
            },
        ];

        assert_eq!(ready_items(&items).len(), 1);
    }
}
