use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const STORE_VERSION: u32 = 1;
const INTENTIONAL_MARK: &str = "intentional_override";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDecisionRecord {
    pub decision_key: String,
    pub mark: String,
    pub left_sha256: String,
    pub right_sha256: String,
    pub left_relative_path: String,
    pub right_relative_path: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConflictDecisionStore {
    version: u32,
    decisions: BTreeMap<String, ConflictDecisionRecord>,
}

impl Default for ConflictDecisionStore {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            decisions: BTreeMap::new(),
        }
    }
}

fn store_path(root: &Path) -> PathBuf {
    root.parent()
        .unwrap_or(root)
        .join("S3CC Organizer")
        .join("Review")
        .join("conflict-decisions-v1.json")
}

fn load_store(root: &Path) -> ConflictDecisionStore {
    let path = store_path(root);
    let Ok(text) = fs::read_to_string(path) else {
        return ConflictDecisionStore::default();
    };
    let Ok(store) = serde_json::from_str::<ConflictDecisionStore>(&text) else {
        return ConflictDecisionStore::default();
    };
    if store.version != STORE_VERSION {
        return ConflictDecisionStore::default();
    }
    store
}

fn save_store(root: &Path, store: &ConflictDecisionStore) -> Result<(), String> {
    let path = store_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| format!("Review-store path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create review directory {}: {error}", parent.display()))?;

    let temp = parent.join(".conflict-decisions-v1.json.tmp");
    let data = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("Could not serialize conflict decisions: {error}"))?;
    fs::write(&temp, data)
        .map_err(|error| format!("Could not write review-store temp {}: {error}", temp.display()))?;

    #[cfg(windows)]
    {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| format!("Could not replace review store {}: {error}", path.display()))?;
        }
    }

    fs::rename(&temp, &path)
        .map_err(|error| format!("Could not commit review store {}: {error}", path.display()))
}

fn canonical_root(folder: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve Mods root: {error}"))?;
    if !root.is_dir() {
        return Err(format!("Selected root is not a directory: {}", root.display()));
    }
    Ok(root)
}

#[tauri::command]
pub fn load_conflict_decisions(folder: String) -> Result<Vec<ConflictDecisionRecord>, String> {
    let root = canonical_root(&folder)?;
    Ok(load_store(&root).decisions.into_values().collect())
}

#[tauri::command]
pub fn set_conflict_decision(
    folder: String,
    decision_key: String,
    mark: Option<String>,
    left_sha256: String,
    right_sha256: String,
    left_relative_path: String,
    right_relative_path: String,
) -> Result<Vec<ConflictDecisionRecord>, String> {
    let root = canonical_root(&folder)?;
    if decision_key.trim().is_empty() {
        return Err("Decision key is empty.".to_string());
    }

    let mut store = load_store(&root);

    match mark.as_deref() {
        None => {
            store.decisions.remove(&decision_key);
        }
        Some(INTENTIONAL_MARK) => {
            let mut hashes = [left_sha256, right_sha256];
            hashes.sort();
            let mut paths = [left_relative_path, right_relative_path];
            paths.sort();

            store.decisions.insert(
                decision_key.clone(),
                ConflictDecisionRecord {
                    decision_key,
                    mark: INTENTIONAL_MARK.to_string(),
                    left_sha256: hashes[0].clone(),
                    right_sha256: hashes[1].clone(),
                    left_relative_path: paths[0].clone(),
                    right_relative_path: paths[1].clone(),
                    updated_at: Local::now().to_rfc3339(),
                },
            );
        }
        Some(other) => {
            return Err(format!("Unsupported persistent conflict mark: {other}"));
        }
    }

    save_store(&root, &store)?;
    Ok(store.decisions.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_path_is_outside_packages_tree() {
        let root = Path::new(r"C:\Mods\Packages");
        let path = store_path(root).to_string_lossy().to_string();
        assert!(path.contains("S3CC Organizer"));
        assert!(path.contains("Review"));
    }
}
