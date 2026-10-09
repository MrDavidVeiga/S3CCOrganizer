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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPathConfig {
    pub path: String,
    pub enabled: bool,
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
    #[serde(default)]
    pub catalog_sources: Vec<CatalogPathConfig>,
    #[serde(default)]
    pub catalog_ignored: Vec<CatalogPathConfig>,
    #[serde(default)]
    pub catalog_master_file: String,
    #[serde(default = "default_catalog_backup_limit")]
    pub catalog_backup_limit: u32,
    #[serde(default)]
    pub catalog_autosave: bool,
}

fn default_catalog_backup_limit() -> u32 { 5 }

impl Default for OrganizationProfile {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            destination_prefix: String::new(),
            collapse_to_category: false,
            rules: Vec::new(),
            protected_folders: Vec::new(),
            catalog_sources: Vec::new(),
            catalog_ignored: Vec::new(),
            catalog_master_file: String::new(),
            catalog_backup_limit: default_catalog_backup_limit(),
            catalog_autosave: false,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManualClassification {
    pub sha256: String,
    pub last_path: String,
    pub destination: String,
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
    #[serde(default)]
    pub manual_classifications: BTreeMap<String, ManualClassification>,
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
            manual_classifications: BTreeMap::new(),
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

fn is_package(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("package"))
        .unwrap_or(false)
}


fn valid_destination_component(value: &str) -> bool {
    if value.is_empty() || value == "." || value == ".." || value.ends_with(' ') || value.ends_with('.') {
        return false;
    }
    if value.chars().any(|ch| ch < '\u{20}' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')) {
        return false;
    }
    let stem = value.split('.').next().unwrap_or(value).trim().to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") {
        return false;
    }
    if stem.len() == 4
        && (stem.starts_with("COM") || stem.starts_with("LPT"))
        && stem.as_bytes()[3].is_ascii_digit()
        && stem.as_bytes()[3] != b'0'
    {
        return false;
    }
    true
}

fn validate_manual_destination(value: &str) -> Result<String, String> {
    let normalized = value.replace('/', "\\").trim_matches('\\').trim().to_string();
    if normalized.is_empty() {
        return Ok(normalized);
    }
    let parts = normalized.split('\\').collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|part| !valid_destination_component(part.trim())) {
        return Err("Manual destination contains an unsafe or invalid Windows folder component.".to_string());
    }
    Ok(parts.into_iter().map(str::trim).collect::<Vec<_>>().join("\\"))
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

// ReplaceFileW swaps an existing Windows file without removing the original
// first. On failure, the existing workspace JSON stays available.
#[cfg(windows)]
fn replace_workspace_windows(destination: &Path, replacement: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn ReplaceFileW(
            replaced: *const u16,
            replacement: *const u16,
            backup: *const u16,
            flags: u32,
            exclude: *mut std::ffi::c_void,
            reserved: *mut std::ffi::c_void,
        ) -> i32;
    }
    let wide = |path: &Path| path.as_os_str().encode_wide()
        .chain(std::iter::once(0)).collect::<Vec<u16>>();
    let original = wide(destination);
    let updated = wide(replacement);
    // REPLACEFILE_IGNORE_MERGE_ERRORS: metadata/ACL merge failure must not
    // silently prevent saving user edits. The replacement is on the same volume.
    let ok = unsafe {
        ReplaceFileW(original.as_ptr(), updated.as_ptr(), std::ptr::null(),
            0x2, std::ptr::null_mut(), std::ptr::null_mut())
    };
    if ok == 0 {
        return Err(format!("Could not atomically replace workspace store: {}",
            std::io::Error::last_os_error()));
    }
    Ok(())
}

static WORKSPACE_TEMP_SEQUENCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

fn save_to_root(root: &Path, store: &WorkspaceStore) -> Result<(), String> {
    use std::io::Write;
    use std::sync::atomic::Ordering;
    let path = store_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| "Workspace store has no parent.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create workspace directory: {error}"))?;
    let sequence = WORKSPACE_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp = parent.join(format!(".workspace-v1.{}.{}.tmp", std::process::id(), sequence));
    let data = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("Could not serialize workspace: {error}"))?;
    let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&temp)
        .map_err(|error| format!("Could not create workspace temp file: {error}"))?;
    if let Err(error) = file.write_all(&data).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temp);
        return Err(format!("Could not write workspace temp file: {error}"));
    }
    drop(file);
    #[cfg(windows)]
    let commit = if path.exists() {
        replace_workspace_windows(&path, &temp)
    } else {
        fs::rename(&temp, &path).map_err(|error| error.to_string())
    };
    #[cfg(not(windows))]
    let commit = fs::rename(&temp, &path).map_err(|error| error.to_string());
    if let Err(error) = commit {
        let _ = fs::remove_file(&temp);
        return Err(format!("Could not commit workspace store: {error}"));
    }
    Ok(())
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
        .replace('/', "\\")
        .split('\\')
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
    ensure_writable(&root)?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() || !is_package(&path) {
        return Err("Package is outside the selected root or is not a .package file.".to_string());
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
                .replace('/', "\\"),
            tags: normalized_tags,
            test_status: test_status.trim().to_string(),
            favorite,
            updated_at: Local::now().to_rfc3339(),
        },
    );
    save_to_root(&root, &store)?;
    Ok(store)
}

// A favorite toggle must preserve tags and test status from the current
// on-disk store, even when the frontend has no workspace metadata loaded.
fn apply_favorite_toggle(store: &mut WorkspaceStore, sha256: &str, relative: &str) {
    let entry = store.package_metadata.entry(sha256.to_string()).or_default();
    entry.sha256 = sha256.to_string();
    entry.last_path = relative.to_string();
    entry.favorite = !entry.favorite;
    entry.updated_at = Local::now().to_rfc3339();
}

#[tauri::command]
pub fn toggle_package_favorite(
    folder: String,
    package_path: String,
) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    ensure_writable(&root)?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() || !is_package(&path) {
        return Err("Package is outside the selected root or is not a .package file.".to_string());
    }
    let (sha256, _) = sha256_file(&path)
        .map_err(|error| format!("Could not hash package: {error}"))?;
    let relative = path.strip_prefix(&root).unwrap_or(&path)
        .to_string_lossy().replace('/', "\\");
    let mut store = load_from_root(&root);
    apply_favorite_toggle(&mut store, &sha256, &relative);
    save_to_root(&root, &store)?;
    Ok(store)
}

#[cfg(test)]
mod favorite_tests {
    use super::*;
    #[test]
    fn toggling_favorite_preserves_existing_tags_and_test_status() {
        let mut store = WorkspaceStore::default();
        store.package_metadata.insert("SHA".into(), PackageMetadata {
            sha256: "SHA".into(),
            last_path: "Packages\\Original.package".into(),
            tags: vec!["trusted".into(), "alpha".into()],
            test_status: "working".into(),
            favorite: false,
            updated_at: "old".into(),
        });
        apply_favorite_toggle(&mut store, "SHA", "Packages\\Moved.package");
        let meta = &store.package_metadata["SHA"];
        assert!(meta.favorite);
        assert_eq!(meta.tags, vec!["trusted", "alpha"]);
        assert_eq!(meta.test_status, "working");
        assert_eq!(meta.last_path, "Packages\\Moved.package");
        apply_favorite_toggle(&mut store, "SHA", "Packages\\Moved.package");
        assert!(!store.package_metadata["SHA"].favorite);
    }
}

#[tauri::command]
pub fn set_manual_classification(
    folder: String,
    package_path: String,
    destination: String,
) -> Result<WorkspaceStore, String> {
    let root = canonical_root(&folder)?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() || !is_package(&path) {
        return Err("Package is outside the selected root or is not a .package file.".to_string());
    }

    let (sha256, _) = sha256_file(&path)
        .map_err(|error| format!("Could not hash package: {error}"))?;
    let mut store = load_from_root(&root);
    let destination = validate_manual_destination(&destination)?;

    if destination.is_empty() {
        store.manual_classifications.remove(&sha256);
    } else {
        store.manual_classifications.insert(
            sha256.clone(),
            ManualClassification {
                sha256,
                last_path: path
                    .strip_prefix(&root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('/', "\\"),
                destination,
                updated_at: Local::now().to_rfc3339(),
            },
        );
    }

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
        if !path.starts_with(&root) || !path.is_file() || !is_package(&path) {
            return Err(format!("Group member is outside the selected root or is not a .package file: {raw}"));
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
        assert!(is_protected(r"NRaas\MasterController.package", &profile));
        assert!(!is_protected(r"CAS\Hair.package", &profile));
    }

    #[test]
    fn manual_destination_rejects_traversal_and_windows_reserved_names() {
        assert!(validate_manual_destination(r"CAS\Sliders\Body").is_ok());
        assert!(validate_manual_destination(r"CAS\..\Elsewhere").is_err());
        assert!(validate_manual_destination(r"CAS\CON").is_err());
        assert!(validate_manual_destination(r"C:\Mods").is_err());
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
                destination: r"Creators\Hair".into(),
                ..CustomRule::default()
            }],
            ..OrganizationProfile::default()
        };
        assert!(matching_rule(&profile, "a.package", "x", Some("CAS"), Some("Hair"), &[]).is_some());
    }
}
