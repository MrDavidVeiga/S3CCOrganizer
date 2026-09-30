use chrono::Local;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReportResult {
    pub directory: String,
    pub markdown_path: String,
    pub json_path: String,
}

fn reports_dir(root: &Path) -> PathBuf {
    root.parent()
        .unwrap_or(root)
        .join("S3CC Organizer")
        .join("Reports")
}

fn unique_paths(directory: &Path) -> (PathBuf, PathBuf) {
    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    for suffix in 0..1000usize {
        let base = if suffix == 0 {
            format!("S3CC-Organizer-Audit-{stamp}")
        } else {
            format!("S3CC-Organizer-Audit-{stamp}-{suffix}")
        };
        let markdown = directory.join(format!("{base}.md"));
        let json = directory.join(format!("{base}.json"));
        if !markdown.exists() && !json.exists() {
            return (markdown, json);
        }
    }

    (
        directory.join(format!("S3CC-Organizer-Audit-{stamp}-overflow.md")),
        directory.join(format!("S3CC-Organizer-Audit-{stamp}-overflow.json")),
    )
}

fn write_new(path: &Path, content: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!("Refusing to overwrite report: {}", path.display()));
    }
    fs::write(path, content)
        .map_err(|error| format!("Could not write report {}: {error}", path.display()))
}

#[tauri::command]
pub fn save_audit_report(
    folder: String,
    markdown: String,
    json_content: String,
) -> Result<AuditReportResult, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve Mods root: {error}"))?;
    if !root.is_dir() {
        return Err(format!("Selected root is not a directory: {}", root.display()));
    }

    let directory = reports_dir(&root);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create reports directory {}: {error}", directory.display()))?;

    let (markdown_path, json_path) = unique_paths(&directory);
    write_new(&markdown_path, markdown.as_bytes())?;

    if let Err(error) = write_new(&json_path, json_content.as_bytes()) {
        let _ = fs::remove_file(&markdown_path);
        return Err(error);
    }

    Ok(AuditReportResult {
        directory: directory.to_string_lossy().to_string(),
        markdown_path: markdown_path.to_string_lossy().to_string(),
        json_path: json_path.to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_live_outside_packages_tree() {
        let root = Path::new(r"C:\Mods\Packages");
        assert!(reports_dir(root)
            .to_string_lossy()
            .contains("S3CC Organizer"));
    }
}
