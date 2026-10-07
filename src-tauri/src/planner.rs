use crate::{
    i18n::AppLanguage,
    manifest::sha256_file,
    scanner::{scan_packages_core, ScanPackageItem},
    workspace::{
        active_profile, is_protected, load_workspace_for_root, matching_rule,
        split_destination,
    },
};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub source_relative_path: String,
    pub destination_path: Option<String>,
    pub destination_relative_path: Option<String>,
    pub classification_status: String,
    pub classification_reason: Option<String>,
    pub plan_status: String,
    pub sha256: Option<String>,
    pub size: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlanStats {
    pub selected: usize,
    pub ready: usize,
    pub already_organized: usize,
    pub collision_same_content: usize,
    pub collision_different_content: usize,
    pub blocked: usize,
    pub directories_to_create: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPlan {
    pub root: String,
    pub items: Vec<PlanItem>,
    pub stats: PlanStats,
    pub directories_to_create: Vec<String>,
    pub manifest_preview: String,
    pub can_execute: bool,
}

fn language_code(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::En => "en",
        AppLanguage::Pt => "pt",
        AppLanguage::Es => "es",
    }
}

fn invalid_windows_component(value: &str) -> bool {
    if value.is_empty() || value == "." || value == ".." {
        return true;
    }
    if value.ends_with(' ') || value.ends_with('.') {
        return true;
    }
    if value
        .chars()
        .any(|ch| ch < '\u{20}' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        return true;
    }

    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .trim()
        .to_ascii_uppercase();

    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

fn validate_destination_parts(parts: &[String]) -> Result<(), String> {
    if parts.is_empty() {
        return Err("destination has no category parts".to_string());
    }
    for part in parts {
        if invalid_windows_component(part) {
            return Err(format!("unsafe destination folder component: {part}"));
        }
    }
    Ok(())
}

fn path_is_within_root(root: &Path, candidate: &Path) -> bool {
    candidate
        .components()
        .zip(root.components())
        .all(|(candidate_component, root_component)| candidate_component == root_component)
        && candidate.components().count() >= root.components().count()
}

fn relative_key(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\\")
}

fn add_missing_directories(root: &Path, parts: &[String], output: &mut BTreeSet<String>) {
    let mut relative = PathBuf::new();
    for part in parts {
        relative.push(part);
        if !root.join(&relative).exists() {
            output.insert(relative_key(&relative));
        }
    }
}

fn mark_intra_plan_destination_collisions(items: &mut [PlanItem], stats: &mut PlanStats) {
    let mut destinations = HashMap::<String, Vec<usize>>::new();

    for (index, item) in items.iter().enumerate() {
        if item.plan_status != "ready" {
            continue;
        }
        let Some(destination) = item.destination_relative_path.as_ref() else {
            continue;
        };
        let key = destination.replace('/', "\\").to_ascii_lowercase();
        destinations.entry(key).or_default().push(index);
    }

    for indices in destinations.values().filter(|indices| indices.len() > 1) {
        let first = &items[indices[0]];
        let first_hash = first.sha256.as_deref().unwrap_or_default();
        let first_size = first.size;
        let same_content = indices.iter().all(|index| {
            let item = &items[*index];
            item.size == first_size
                && item
                    .sha256
                    .as_deref()
                    .map(|hash| hash.eq_ignore_ascii_case(first_hash))
                    .unwrap_or(false)
        });

        for index in indices {
            let item = &mut items[*index];
            if same_content {
                item.plan_status = "collision_same_content".to_string();
                stats.collision_same_content += 1;
                item.warnings.push(
                    "Multiple selected packages resolve to the same destination and are byte-identical. Resolve the duplicate before organizing."
                        .to_string(),
                );
            } else {
                item.plan_status = "collision_different_content".to_string();
                stats.collision_different_content += 1;
                item.warnings.push(
                    "Multiple selected packages resolve to the same destination but contain different data. Nothing will be overwritten."
                        .to_string(),
                );
            }
        }
    }
}

fn manifest_preview(
    root: &Path,
    language: AppLanguage,
    ready_items: &[PlanItem],
) -> String {
    let mut out = String::new();
    out.push_str("S3CC MANAGER RESTORE MANIFEST\n");
    out.push_str("version=1\n");
    out.push_str(&format!(
        "created_at={}\n",
        Local::now().to_rfc3339_opts(SecondsFormat::Secs, true)
    ));
    out.push_str(&format!("mode=PREVIEW\n"));
    out.push_str(&format!("organization_language={}\n", language_code(language)));
    out.push_str(&format!("root={}\n", root.display()));
    out.push_str(&format!("files={}\n\n", ready_items.len()));

    for item in ready_items {
        out.push_str("[file]\n");
        if let Some(hash) = &item.sha256 {
            out.push_str(&format!("sha256={hash}\n"));
        }
        out.push_str(&format!("size={}\n", item.size));
        out.push_str(&format!("original={}\n", item.source_relative_path));
        if let Some(destination) = &item.destination_relative_path {
            out.push_str(&format!("organized={destination}\n"));
        }
        out.push_str("[/file]\n\n");
    }

    out
}

fn make_blocked(item: &ScanPackageItem, reason: String) -> PlanItem {
    PlanItem {
        id: item.id.clone(),
        name: item.name.clone(),
        source_path: item.path.clone(),
        source_relative_path: item.relative_path.clone(),
        destination_path: None,
        destination_relative_path: None,
        classification_status: item.status.clone(),
        classification_reason: item.classification_reason.clone(),
        plan_status: "blocked".to_string(),
        sha256: None,
        size: item.file_size,
        warnings: vec![reason],
    }
}

#[tauri::command]
pub fn build_organization_plan(
    folder: String,
    language: AppLanguage,
    selected_paths: Vec<String>,
) -> Result<OrganizationPlan, String> {
    if selected_paths.is_empty() {
        return Err("No packages were selected.".to_string());
    }

    let root_input = PathBuf::from(folder.trim());
    let root = root_input
        .canonicalize()
        .map_err(|error| format!("Could not resolve root folder: {error}"))?;

    if !root.is_dir() {
        return Err(format!("Root is not a directory: {}", root.display()));
    }

    let workspace = load_workspace_for_root(&root);
    let profile = active_profile(&workspace);
    let scan = scan_packages_core(root.to_string_lossy().to_string(), language, None)?;

    let mut scan_by_canonical = HashMap::<PathBuf, &ScanPackageItem>::new();
    for item in &scan.items {
        if let Ok(path) = PathBuf::from(&item.path).canonicalize() {
            scan_by_canonical.insert(path, item);
        }
    }

    let mut selected = HashSet::<PathBuf>::new();
    for raw in selected_paths {
        let canonical = PathBuf::from(&raw)
            .canonicalize()
            .map_err(|error| format!("Could not resolve selected file {raw}: {error}"))?;

        if !path_is_within_root(&root, &canonical) {
            return Err(format!("Selected file escaped the selected root: {raw}"));
        }

        selected.insert(canonical);
    }

    let unknown_selection = selected
        .iter()
        .filter(|path| !scan_by_canonical.contains_key(*path))
        .take(3)
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if !unknown_selection.is_empty() {
        return Err(format!(
            "Selection contains files outside the current scan: {}",
            unknown_selection.join(", ")
        ));
    }

    let mut selected_hashes = HashMap::<PathBuf, String>::new();
    let mut selected_hash_set = HashSet::<String>::new();
    for path in &selected {
        let (hash, _) = sha256_file(path)
            .map_err(|error| format!("Could not hash selected package {}: {error}", path.display()))?;
        selected_hashes.insert(path.clone(), hash.clone());
        selected_hash_set.insert(hash);
    }

    let mut partial_group_hashes = HashSet::<String>::new();

    for group in workspace.groups.iter().filter(|group| group.keep_together) {
        let intersects = group
            .member_sha256
            .iter()
            .any(|hash| selected_hash_set.contains(hash));
        let complete = group
            .member_sha256
            .iter()
            .all(|hash| selected_hash_set.contains(hash));
        if intersects && !complete {
            for hash in &group.member_sha256 {
                if selected_hash_set.contains(hash) {
                    partial_group_hashes.insert(hash.clone());
                }
            }
        }
    }

    let mut stats = PlanStats {
        selected: selected.len(),
        ..PlanStats::default()
    };
    let mut items = Vec::with_capacity(selected.len());
    let mut directories = BTreeSet::new();

    let mut ordered = selected
        .iter()
        .filter_map(|path| scan_by_canonical.get(path).copied())
        .collect::<Vec<_>>();
    ordered.sort_by_key(|item| item.relative_path.to_ascii_lowercase());

    for item in ordered {
        if is_protected(&item.relative_path, &profile) {
            stats.blocked += 1;
            items.push(make_blocked(
                item,
                "The active profile protects this folder from automatic organization.".to_string(),
            ));
            continue;
        }

        let source = PathBuf::from(&item.path);
        let source_canonical = source
            .canonicalize()
            .map_err(|error| format!("Could not resolve source {}: {error}", source.display()))?;
        let source_hash = selected_hashes
            .get(&source_canonical)
            .cloned()
            .ok_or_else(|| format!("Missing selected package hash: {}", source.display()))?;

        if partial_group_hashes.contains(&source_hash) {
            stats.blocked += 1;
            items.push(make_blocked(
                item,
                "This package belongs to a Keep Together group. Select every member of the group before organizing it.".to_string(),
            ));
            continue;
        }

        if item.status != "classified" {
            stats.blocked += 1;
            items.push(make_blocked(
                item,
                format!("Classification status '{}' is not eligible for automatic organization.", item.status),
            ));
            continue;
        }

        let mut destination_parts = item.destination_parts.clone();
        let mut classification_reason = item.classification_reason.clone();

        if let Some(rule) = matching_rule(
            &profile,
            &item.name,
            &item.relative_path,
            item.category.as_deref(),
            item.sub_category.as_deref(),
            &item.detected_from,
        ) {
            destination_parts = split_destination(&rule.destination);
            classification_reason = Some(format!(
                "{} | Custom rule '{}' => {}",
                classification_reason.unwrap_or_default(),
                rule.name,
                destination_parts.join("\\")
            ));
        } else if profile.collapse_to_category {
            if let Some(category) = &item.category {
                destination_parts = vec![category.clone()];
                classification_reason = Some(format!(
                    "{} | Profile '{}' collapsed destination to category '{}'.",
                    classification_reason.unwrap_or_default(),
                    profile.name,
                    category
                ));
            }
        }

        let prefix = split_destination(&profile.destination_prefix);
        if !prefix.is_empty() {
            let mut prefixed = prefix;
            prefixed.extend(destination_parts);
            destination_parts = prefixed;
        }

        if let Err(reason) = validate_destination_parts(&destination_parts) {
            stats.blocked += 1;
            items.push(make_blocked(item, reason));
            continue;
        }

        if !path_is_within_root(&root, &source_canonical) {
            return Err(format!(
                "Source escaped the selected root: {}",
                source.display()
            ));
        }

        let file_name = source
            .file_name()
            .ok_or_else(|| format!("Source has no filename: {}", source.display()))?;
        let mut destination_relative = PathBuf::new();
        for part in &destination_parts {
            destination_relative.push(part);
        }
        destination_relative.push(file_name);

        let destination = root.join(&destination_relative);
        let destination_relative_text = relative_key(&destination_relative);

        if relative_key(Path::new(&item.relative_path)).eq_ignore_ascii_case(&destination_relative_text) {
            let (hash, size) = sha256_file(&source)
                .map_err(|error| format!("Could not hash {}: {error}", source.display()))?;
            stats.already_organized += 1;
            items.push(PlanItem {
                id: item.id.clone(),
                name: item.name.clone(),
                source_path: source.to_string_lossy().to_string(),
                source_relative_path: item.relative_path.clone(),
                destination_path: Some(destination.to_string_lossy().to_string()),
                destination_relative_path: Some(destination_relative_text),
                classification_status: item.status.clone(),
                classification_reason: classification_reason.clone(),
                plan_status: "already_organized".to_string(),
                sha256: Some(hash),
                size,
                warnings: Vec::new(),
            });
            continue;
        }

        let source_size = fs::metadata(&source)
            .map_err(|error| format!("Could not stat {}: {error}", source.display()))?
            .len();

        if destination.exists() {
            let (target_hash, target_size) = sha256_file(&destination)
                .map_err(|error| format!("Could not hash collision target {}: {error}", destination.display()))?;

            let same = source_size == target_size && source_hash.eq_ignore_ascii_case(&target_hash);
            if same {
                stats.collision_same_content += 1;
            } else {
                stats.collision_different_content += 1;
            }

            items.push(PlanItem {
                id: item.id.clone(),
                name: item.name.clone(),
                source_path: source.to_string_lossy().to_string(),
                source_relative_path: item.relative_path.clone(),
                destination_path: Some(destination.to_string_lossy().to_string()),
                destination_relative_path: Some(destination_relative_text),
                classification_status: item.status.clone(),
                classification_reason: classification_reason.clone(),
                plan_status: if same {
                    "collision_same_content".to_string()
                } else {
                    "collision_different_content".to_string()
                },
                sha256: Some(source_hash),
                size: source_size,
                warnings: vec![if same {
                    "Destination already contains a byte-identical file. No overwrite is allowed.".to_string()
                } else {
                    "Destination already contains a different file with the same name. No overwrite is allowed.".to_string()
                }],
            });
            continue;
        }

        add_missing_directories(&root, &destination_parts, &mut directories);
        stats.ready += 1;
        items.push(PlanItem {
            id: item.id.clone(),
            name: item.name.clone(),
            source_path: source.to_string_lossy().to_string(),
            source_relative_path: item.relative_path.clone(),
            destination_path: Some(destination.to_string_lossy().to_string()),
            destination_relative_path: Some(destination_relative_text),
            classification_status: item.status.clone(),
            classification_reason: classification_reason.clone(),
            plan_status: "ready".to_string(),
            sha256: Some(source_hash),
            size: source_size,
            warnings: Vec::new(),
        });
    }

    // Detect collisions created by the plan itself before any filesystem write occurs.
    // This catches multiple selected packages that resolve to the same final path.
    mark_intra_plan_destination_collisions(&mut items, &mut stats);

    for group in workspace.groups.iter().filter(|group| group.keep_together) {
        if !group
            .member_sha256
            .iter()
            .all(|hash| selected_hash_set.contains(hash))
        {
            continue;
        }

        let destinations = items
            .iter()
            .filter(|item| {
                item.sha256
                    .as_ref()
                    .map(|hash| group.member_sha256.contains(hash))
                    .unwrap_or(false)
            })
            .filter_map(|item| item.destination_relative_path.as_ref())
            .filter_map(|path| Path::new(path).parent().map(relative_key))
            .collect::<HashSet<_>>();

        if destinations.len() > 1 {
            for item in items.iter_mut().filter(|item| {
                item.sha256
                    .as_ref()
                    .map(|hash| group.member_sha256.contains(hash))
                    .unwrap_or(false)
            }) {
                if item.plan_status == "ready" {
                    item.plan_status = "blocked".to_string();
                    item.warnings.push(format!(
                        "Keep Together group '{}' would be split across multiple destination folders.",
                        group.name
                    ));
                }
            }
        }
    }

    stats.ready = items.iter().filter(|item| item.plan_status == "ready").count();
    stats.blocked = items.iter().filter(|item| item.plan_status == "blocked").count();

    let ready_items = items
        .iter()
        .filter(|item| item.plan_status == "ready")
        .cloned()
        .collect::<Vec<_>>();

    stats.directories_to_create = directories.len();

    let can_execute = !workspace.read_only
        && stats.ready > 0
        && stats.blocked == 0
        && stats.collision_same_content == 0
        && stats.collision_different_content == 0;

    Ok(OrganizationPlan {
        root: root.to_string_lossy().to_string(),
        manifest_preview: manifest_preview(&root, language, &ready_items),
        directories_to_create: directories.into_iter().collect(),
        stats,
        items,
        can_execute,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_components_reject_path_escape_and_windows_invalid_names() {
        assert!(validate_destination_parts(&["CAS".into(), "Roupas".into()]).is_ok());
        assert!(validate_destination_parts(&["..".into()]).is_err());
        assert!(validate_destination_parts(&["Bad/Folder".into()]).is_err());
        assert!(validate_destination_parts(&["CON".into()]).is_err());
        assert!(validate_destination_parts(&["Name.".into()]).is_err());
    }

    #[test]
    fn planned_destination_collision_is_detected_before_execution() {
        let mut items = vec![
            PlanItem {
                id: "1".into(),
                name: "same.package".into(),
                source_path: r"C:\\Mods\\A\\same.package".into(),
                source_relative_path: r"A\\same.package".into(),
                destination_path: Some(r"C:\\Mods\\CAS\\Sliders\\same.package".into()),
                destination_relative_path: Some(r"CAS\\Sliders\\same.package".into()),
                classification_status: "classified".into(),
                classification_reason: Some("test".into()),
                plan_status: "ready".into(),
                sha256: Some("A".repeat(64)),
                size: 10,
                warnings: vec![],
            },
            PlanItem {
                id: "2".into(),
                name: "same.package".into(),
                source_path: r"C:\\Mods\\B\\same.package".into(),
                source_relative_path: r"B\\same.package".into(),
                destination_path: Some(r"C:\\Mods\\CAS\\Sliders\\same.package".into()),
                destination_relative_path: Some(r"CAS\\Sliders\\same.package".into()),
                classification_status: "classified".into(),
                classification_reason: Some("test".into()),
                plan_status: "ready".into(),
                sha256: Some("B".repeat(64)),
                size: 10,
                warnings: vec![],
            },
        ];
        let mut stats = PlanStats::default();

        mark_intra_plan_destination_collisions(&mut items, &mut stats);

        assert_eq!(stats.collision_different_content, 2);
        assert!(items
            .iter()
            .all(|item| item.plan_status == "collision_different_content"));
    }

    #[test]
    fn manifest_preview_contains_original_and_organized_paths() {
        let item = PlanItem {
            id: "1".into(),
            name: "x.package".into(),
            source_path: "C:\\Mods\\Old\\x.package".into(),
            source_relative_path: "Old\\x.package".into(),
            destination_path: Some("C:\\Mods\\CAS\\Hair\\x.package".into()),
            destination_relative_path: Some("CAS\\Hair\\x.package".into()),
            classification_status: "classified".into(),
            classification_reason: Some("CASP clothingType=0x00000005 => CAS\\Hair".into()),
            plan_status: "ready".into(),
            sha256: Some("ABC".into()),
            size: 123,
            warnings: vec![],
        };

        let text = manifest_preview(Path::new("C:\\Mods"), AppLanguage::Pt, &[item]);
        assert!(text.contains("mode=PREVIEW"));
        assert!(text.contains("organization_language=pt"));
        assert!(text.contains("original=Old\\x.package"));
        assert!(text.contains("organized=CAS\\Hair\\x.package"));
    }
}
