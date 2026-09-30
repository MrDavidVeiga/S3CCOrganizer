use crate::manifest::sha256_file;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub detected_from: Option<String>,
    pub name_contains: Option<String>,
    pub path_contains: Option<String>,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationProfile {
    pub id: String,
    pub name: String,
    pub destination_prefix: String,
    pub collapse_to_category: bool,
    pub rules: Vec<CustomRule>,
    pub protected_folders: Vec<String>,
}

impl Default for OrganizationProfile {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            destination_prefix: String::new(),
            collapse_to_category: false,
            rules: Vec::new(),
            protected_folders: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageMetadata {
    pub sha256: String,
    pub last_path: String,
    pub tags: Vec<String>,
    pub test_status: String,
    pub favorite: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageGroup {
    pub id: String,
    pub name: String,
    pub keep_together: bool,
    pub member_sha256: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStore {
    pub version: u32,
    pub read_only: bool,
    pub active_profile_id: String,
    pub profiles: Vec<OrganizationProfile>,
    pub package_metadata: BTreeMap<String, PackageMetadata>,
    pub groups: Vec<PackageGroup>,
}

impl Default for WorkspaceStore {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            read_only: false,
            active_profile_id: "default".to_string(),
            profiles: vec![OrganizationProfile::default()],
            package_metadata: BTreeMap::new(),
            groups: Vec::new(),
        }
    }
}

fn canonical_root(folder: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve selected root: {error}"))?;
    if !root.is_dir() {
        return Err(format!("Selected root is not a directory: {}", root.display()));
    }
    Ok(root)
}

fn store_path(root: &Path) -> PathBuf {
    root.parent()
        .unwrap_or(root)
        .join("S3CC Organizer")
        .join("Workspace")
        .join("workspace-v1.json")
}

fn load_from_root(root: &Path) -> WorkspaceStore {
    let path = store_path(root);
    let Ok(text) = fs::read_to_string(path) else {
        return WorkspaceStore::default();
    };
    let Ok(mut store) = serde_json::from_str::<WorkspaceStore>(&text) else {
        return WorkspaceStore::default();
    };
    if store.version != STORE_VERSION {
        return WorkspaceStore::default();
    }
    if !store.profiles.iter().any(|profile| profile.id == "default") {
        store.profiles.insert(0, OrganizationProfile::default());
    }
    if !store
        .profiles
        .iter()
        .any(|profile| profile.id == store.active_profile_id)
    {
        store.active_profile_id = "default".to_string();
    }
    store
}

fn save_to_root(root: &Path, store: &WorkspaceStore) -> Result<(), String> {
    let path = store_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| "Workspace store has no parent.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create workspace directory: {error}"))?;
    let temp = parent.join(".workspace-v1.json.tmp");
    let data = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("Could not serialize workspace: {error}"))?;
    fs::write(&temp, data)
        .map_err(|error| format!("Could not write workspace temp file: {error}"))?;
    #[cfg(windows)]
    {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| format!("Could not replace workspace store: {error}"))?;
        }
    }
    fs::rename(&temp, &path)
        .map_err(|error| format!("Could not commit workspace store: {error}"))
}

pub fn load_workspace_for_root(root: &Path) -> WorkspaceStore {
    load_from_root(root)
}

pub fn active_profile(store: &WorkspaceStore) -> OrganizationProfile {
    store
        .profiles
        .iter()
        .find(|profile| profile.id == store.active_profile_id)
        .cloned()
        .unwrap_or_default()
}

pub fn ensure_writable(root: &Path) -> Result<(), String> {
    if load_from_root(root).read_only {
        Err("The Organizer is in read-only mode for this Mods root.".to_string())
    } else {
        Ok(())
    }
}

pub fn normalize_relative_text(value: &str) -> String {
    value.replace('/', "\\").trim_matches('\\').to_ascii_lowercase()
}

pub fn is_protected(relative_path: &str, profile: &OrganizationProfile) -> bool {
    let relative = normalize_relative_text(relative_path);
    profile.protected_folders.iter().any(|folder| {
        let protected = normalize_relative_text(folder);
        !protected.is_empty()
            && (relative == protected || relative.starts_with(&(protected + "\\")))
    })
}

pub fn matching_rule<'a>(
    profile: &'a OrganizationProfile,
    name: &str,
    relative_path: &str,
    category: Option<&str>,
    sub_category: Option<&str>,
    detected_from: &[String],
) -> Option<&'a CustomRule> {
    let name_lower = name.to_ascii_lowercase();
    let path_lower = relative_path.to_ascii_lowercase();
    profile.rules.iter().find(|rule| {
        if !rule.enabled || rule.destination.trim().is_empty() {
            return false;
        }
        if let Some(expected) = &rule.category {
            if category.map(|v| !v.eq_ignore_ascii_case(expected)).unwrap_or(true) {
                return false;
            }
        }
        if let Some(expected) = &rule.sub_category {
            if sub_category.map(|v| !v.eq_ignore_ascii_case(expected)).unwrap_or(true) {
                return false;
            }
        }
        if let Some(expected) = &rule.detected_from {
            if !detected_from.iter().any(|v| v.eq_ignore_ascii_case(expected)) {
                return false;
            }
        }
        if let Some(expected) = &rule.name_contains {
            if !name_lower.contains(&expected.to_ascii_lowercase()) {
                return false;
            }
        }
        if let Some(expected) = &rule.path_contains {
            if !path_lower.contains(&expected.to_ascii_lowercase()) {
                return false;
            }
        }
        true
    })
}

pub fn split_destination(value: &str) -> Vec<String> {
    value
        .replace('/', "\")
        .split('\')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[tauri::command]
pub fn load_workspace(folder: String) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    Ok(load_from_root(&root))
}

#[tauri::command]
pub fn save_workspace(folder: String, store: WorkspaceStore) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let mut normalized = store;
    normalized.version = STORE_VERSION;
    if !normalized.profiles.iter().any(|profile| profile.id == "default") {
        normalized.profiles.insert(0, OrganizationProfile::default());
    }
    if !normalized
        .profiles
        .iter()
        .any(|profile| profile.id == normalized.active_profile_id)
    {
        normalized.active_profile_id = "default".to_string();
    }
    save_to_root(&root, &normalized)?;
    Ok(normalized)
}

#[tauri::command]
pub fn set_read_only(folder: String, read_only: bool) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let mut store = load_from_root(&root);
    store.read_only = read_only;
    save_to_root(&root, &store)?;
    Ok(store)
}

#[tauri::command]
pub fn set_package_metadata(
    folder: String,
    package_path: String,
    tags: Vec<String>,
    test_status: String,
    favorite: bool,
) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err("Package is outside the selected root.".to_string());
    }
    let (sha256, _) = sha256_file(&path)
        .map_err(|error| format!("Could not hash package: {error}"))?;
    let mut store = load_from_root(&root);
    let mut normalized_tags = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    normalized_tags.sort_by_key(|tag| tag.to_ascii_lowercase());
    normalized_tags.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    store.package_metadata.insert(
        sha256.clone(),
        PackageMetadata {
            sha256,
            last_path: path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('/', "\"),
            tags: normalized_tags,
            test_status: test_status.trim().to_string(),
            favorite,
            updated_at: Local::now().to_rfc3339(),
        },
    );
    save_to_root(&root, &store)?;
    Ok(store)
}

#[tauri::command]
pub fn save_package_group(
    folder: String,
    id: String,
    name: String,
    keep_together: bool,
    package_paths: Vec<String>,
) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let mut members = Vec::new();
    for raw in package_paths {
        let path = PathBuf::from(&raw)
            .canonicalize()
            .map_err(|error| format!("Could not resolve group package: {error}"))?;
        if !path.starts_with(&root) || !path.is_file() {
            return Err(format!("Group package is outside the selected root: {raw}"));
        }
        let (hash, _) = sha256_file(&path)
            .map_err(|error| format!("Could not hash group package: {error}"))?;
        members.push(hash);
    }
    members.sort();
    members.dedup();
    if members.is_empty() {
        return Err("A package group must contain at least one package.".to_string());
    }

    let mut store = load_from_root(&root);
    let group_id = if id.trim().is_empty() {
        format!("group-{}", Local::now().timestamp_millis())
    } else {
        id.trim().to_string()
    };
    let group = PackageGroup {
        id: group_id.clone(),
        name: name.trim().to_string(),
        keep_together,
        member_sha256: members,
        updated_at: Local::now().to_rfc3339(),
    };
    if let Some(existing) = store.groups.iter_mut().find(|item| item.id == group_id) {
        *existing = group;
    } else {
        store.groups.push(group);
    }
    save_to_root(&root, &store)?;
    Ok(store)
}

#[tauri::command]
pub fn delete_package_group(folder: String, id: String) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let mut store = load_from_root(&root);
    store.groups.retain(|group| group.id != id);
    save_to_root(&root, &store)?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_folder_matches_descendants() {
        let profile = OrganizationProfile {
            protected_folders: vec!["NRaas".to_string()],
            ..OrganizationProfile::default()
        };
        assert!(is_protected("NRaas\MasterController.package", &profile));
        assert!(!is_protected("CAS\Hair.package", &profile));
    }

    #[test]
    fn custom_rule_can_match_catalog_fields() {
        let profile = OrganizationProfile {
            rules: vec![CustomRule {
                id: "r1".into(),
                name: "Hair".into(),
                enabled: true,
                category: Some("CAS".into()),
                sub_category: Some("Hair".into()),
                destination: "Creators\Hair".into(),
                ..CustomRule::default()
            }],
            ..OrganizationProfile::default()
        };
        assert!(matching_rule(&profile, "a.package", "x", Some("CAS"), Some("Hair"), &[]).is_some());
    }
}
