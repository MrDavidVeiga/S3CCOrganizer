use crate::{
    workspace::ensure_writable,
    i18n::AppLanguage,
    manifest::{
        replace_manifest_atomic, sha256_file, write_manifest_atomic, RestoreEntry, RestoreManifest,
    },
    planner::{build_organization_plan_with_cfg, PlanItem},
    resource_cfg::{parse_resource_cfg, package_priority},
    resource_cfg_update::{apply_resource_cfg_update, rollback_resource_cfg_update},
};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashMap},
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
    pub old_folders_removed: usize,
    pub old_folders_retained: usize,
    pub errors: Vec<String>,
}

fn make_manifest_path(root: &Path) -> Result<PathBuf, String> {
    let base = root
        .parent()
        .unwrap_or(root)
        .join("S3CC Manager")
        .join("Restore Manifests");

    fs::create_dir_all(&base)
        .map_err(|error| format!("Could not create restore manifest directory {}: {error}", base.display()))?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    for suffix in 0..10_000usize {
        let file_name = if suffix == 0 {
            format!("S3CC-Manager-Restore-{stamp}.txt")
        } else {
            format!("S3CC-Manager-Restore-{stamp}-{suffix}.txt")
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
        .filter(|item| item.plan_status.starts_with("ready"))
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
        resource_cfg_restore: None,
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

// Remove only directories vacated by files actually moved in this
// transaction. Never sweep unrelated, user-created, or nonempty folders.
// Packages itself must always survive; retained files such as duplicates or
// merged packages must never be removed to make a folder disappear.
fn cleanup_vacated_source_directories(
    root: &Path,
    moved_pairs: &[(PathBuf, PathBuf, String, u64)],
) -> (usize, usize) {
    let packages = root.join("Packages");
    let overrides = root.join("Overrides");
    let mods_root = root.file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("Mods"));

    let mut old_directories = BTreeSet::<PathBuf>::new();
    for (source, _, _, _) in moved_pairs {
        // Files originally placed by old Manager versions under Mods/CAS or
        // Mods/Sliders also need their vacated category directories removed.
        // These roots are allowed only after the planner's strict legacy
        // category check, so a successful transaction may clean them safely.
        let boundary = if mods_root && source.starts_with(&packages) {
            &packages
        } else if mods_root && source.starts_with(&overrides) {
            &overrides
        } else {
            root
        };
        let mut current = source.parent();
        while let Some(dir) = current {
            if dir == boundary || !dir.starts_with(boundary) {
                break;
            }
            old_directories.insert(dir.to_path_buf());
            current = dir.parent();
        }
    }

    let mut directories = old_directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    let mut removed = 0;
    let mut retained = 0;
    for folder in directories {
        // remove_dir succeeds ONLY for genuinely empty folders.
        if fs::remove_dir(&folder).is_ok() {
            removed += 1;
        } else if folder.is_dir() {
            // This folder may still contain an exact duplicate, a merged mod,
            // an unclassified package or an unrelated companion file.
            retained += 1;
        }
    }
    (removed, retained)
}

#[tauri::command]
pub fn execute_organization(
    folder: String,
    language: AppLanguage,
    selected_paths: Vec<String>,
) -> Result<ExecutionResult, String> {
    execute_organization_with_cfg(folder, language, selected_paths, false, None, Vec::new())
}

#[tauri::command]
pub fn execute_organization_with_cfg(
    folder: String,
    language: AppLanguage,
    selected_paths: Vec<String>,
    update_resource_cfg: bool,
    expected_cfg_hash: Option<String>,
    expected_cfg_rules: Vec<String>,
) -> Result<ExecutionResult, String> {
    let plan = build_organization_plan_with_cfg(folder, language, selected_paths, update_resource_cfg)?;

    // Only explicitly ready items will be moved. Keep every blocked item,
    // duplicate and conflicting destination in place for later manual review.
    // The plan and file SHA-256 identities are rebuilt immediately before
    // execution, and the rollback manifest records only ready movements.
    let root = PathBuf::from(&plan.root);
    ensure_writable(&root)?;
    let ready = ready_items(&plan.items);

    if ready.is_empty() {
        return Ok(ExecutionResult {
            status: "NO_CHANGES".to_string(),
            manifest_path: None,
            moved: 0,
            already_organized: plan.stats.already_organized,
            rolled_back: 0,
            old_folders_removed: 0,
            old_folders_retained: 0,
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
    // Persist recovery information before touching Resource.cfg or CC files.
    write_manifest_atomic(&manifest_path, &manifest)?;

    let mut applied_cfg = None;
    if update_resource_cfg {
        let change = plan.resource_cfg_update.as_ref()
            .ok_or("Resource.cfg opt-in requires a valid Mods/Packages or Mods/Overrides root.")?;
        if expected_cfg_hash.as_deref() != Some(change.original_hash.as_str())
            || expected_cfg_rules != change.added_rules {
            manifest.status = "ROLLED_BACK".to_string();
            let _ = replace_manifest_atomic(&manifest_path, &manifest);
            return Err("Resource.cfg preview is stale; review the new plan before organizing.".into());
        }
        applied_cfg = apply_resource_cfg_update(change)?;

        // Every updated target must be loadable before the first package move.
        // A parsing / IO failure here must also roll the newly installed cfg back.
        let cfg_validation = (|| -> Result<(), String> {
            let cfg_path = PathBuf::from(&change.path);
            let parsed = parse_resource_cfg(&cfg_path)?;
            let cfg_dir = cfg_path.parent().ok_or("Resource.cfg parent missing")?;
            let uncovered = ready.iter().filter(|item| {
                item.destination_path.as_ref()
                    .is_none_or(|path| package_priority(&parsed, cfg_dir, Path::new(path)).is_none())
            }).count();
            if uncovered != 0 {
                return Err(format!("Updated Resource.cfg misses {uncovered} planned packages."));
            }
            Ok(())
        })();
        if let Err(error) = cfg_validation {
            if let Some(applied) = applied_cfg.take() {
                rollback_resource_cfg_update(applied)?;
            }
            manifest.status = "ROLLED_BACK".to_string();
            let _ = replace_manifest_atomic(&manifest_path, &manifest);
            return Err(format!("{error} No packages were moved; Resource.cfg was restored."));
        }

        if let Some(applied) = &applied_cfg {
            manifest.resource_cfg_restore = Some(applied.restore_snapshot(change));
            if let Err(error) = replace_manifest_atomic(&manifest_path, &manifest) {
                if let Some(applied) = applied_cfg.take() {
                    rollback_resource_cfg_update(applied)?;
                }
                return Err(format!("Could not persist Resource.cfg recovery metadata: {error}"));
            }
        }
    }

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

        if rollback_errors.is_empty() {
            if let Some(applied) = applied_cfg.take() {
                match rollback_resource_cfg_update(applied) {
                    Ok(()) => {
                        manifest.resource_cfg_restore = None;
                        if let Err(error) = replace_manifest_atomic(&manifest_path, &manifest) {
                            rollback_errors.push(error);
                            manifest.status = "ROLLBACK_INCOMPLETE".into();
                        }
                    }
                    Err(error) => {
                        rollback_errors.push(error);
                        manifest.status = "ROLLBACK_INCOMPLETE".to_string();
                        let _ = replace_manifest_atomic(&manifest_path, &manifest);
                    }
                }
            }
        }
        errors.extend(rollback_errors);

        return Ok(ExecutionResult {
            status: manifest.status.clone(),
            manifest_path: Some(manifest_path.to_string_lossy().to_string()),
            moved: moved_pairs.len(),
            already_organized: plan.stats.already_organized,
            rolled_back,
            old_folders_removed: 0,
            old_folders_retained: 0,
            errors,
        });
    }

    let (old_folders_removed, old_folders_retained) =
        cleanup_vacated_source_directories(&root, &moved_pairs);

    manifest.status = "COMPLETE".to_string();
    replace_manifest_atomic(&manifest_path, &manifest)?;

    Ok(ExecutionResult {
        status: "COMPLETE".to_string(),
        manifest_path: Some(manifest_path.to_string_lossy().to_string()),
        moved: moved_pairs.len(),
        already_organized: plan.stats.already_organized,
        rolled_back: 0,
        old_folders_removed,
        old_folders_retained,
        errors: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_empty_old_folders_without_touching_held_duplicates() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let mods = std::env::temp_dir().join(format!("s3cc-old-folder-{}-{nonce}", std::process::id())).join("Mods");
        let packages = mods.join("Packages");
        let old = packages.join("CAS").join("Sliders");
        let unrelated = packages.join("Custom Empty Folder");
        let retained = packages.join("Legacy").join("Jonha");
        let legacy = mods.join("CAS").join("Sliders");
        let override_old = mods.join("Overrides").join("Old Tuning");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&unrelated).unwrap();
        std::fs::create_dir_all(&retained).unwrap();
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::create_dir_all(&override_old).unwrap();
        std::fs::write(retained.join("Jonha_BASE.package"), b"pending Duplicates review").unwrap();

        let moved = vec![
            (
                old.join("moved.package"),
                packages.join("Sliders").join("moved.package"),
                String::new(),
                0,
            ),
            (
                retained.join("moved.package"),
                packages.join("Sliders").join("other.package"),
                String::new(),
                0,
            ),
            (
                legacy.join("moved.package"),
                packages.join("Sliders").join("migrated.package"),
                String::new(),
                0,
            ),
            (
                override_old.join("patch.package"),
                mods.join("Overrides").join("Tuning").join("patch.package"),
                String::new(),
                0,
            ),
        ];
        let (removed, preserved) = cleanup_vacated_source_directories(&mods, &moved);
        assert_eq!(removed, 5); // Legacy CAS branches and Old Tuning within Overrides
        assert_eq!(preserved, 2); // Legacy/Jonha and its parent
        assert!(packages.is_dir());
        assert!(mods.join("Overrides").is_dir());
        assert!(retained.join("Jonha_BASE.package").is_file());
        assert!(unrelated.is_dir());
        std::fs::remove_dir_all(mods.parent().unwrap()).unwrap();
    }

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

        let mut fallback = items[1].clone();
        fallback.plan_status = "ready_uncategorized".into();
        assert_eq!(ready_items(&[items[0].clone(), fallback]).len(), 2);
    }
}
