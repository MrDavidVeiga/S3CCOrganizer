use crate::i18n::AppLanguage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreEntry {
    pub sha256: String,
    pub size: u64,
    pub original_relative_path: PathBuf,
    pub organized_relative_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreManifest {
    pub version: u32,
    pub created_at: String,
    /// Language used when the categorized layout was generated.
    /// This is audit metadata only; restore identity never depends on translated labels.
    pub organization_language: AppLanguage,
    pub root: PathBuf,
    pub status: String,
    #[serde(default)]
    pub created_directories: Vec<PathBuf>,
    pub entries: Vec<RestoreEntry>,
}

impl RestoreManifest {
    pub fn contains_identity(&self, sha256: &str, size: u64) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.sha256.eq_ignore_ascii_case(sha256) && entry.size == size)
    }

    /// New files discovered during restore are placed under a folder localized
    /// using the CURRENT interface language, not the language stored in the manifest.
    pub fn uncategorized_destination(
        &self,
        current_language: AppLanguage,
        current_relative_path: &Path,
    ) -> PathBuf {
        PathBuf::from(current_language.not_categorized_folder()).join(current_relative_path)
    }
}

pub fn language_code(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::En => "en",
        AppLanguage::Pt => "pt",
        AppLanguage::Es => "es",
    }
}

pub fn parse_language(value: &str) -> Option<AppLanguage> {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" => Some(AppLanguage::En),
        "pt" => Some(AppLanguage::Pt),
        "es" => Some(AppLanguage::Es),
        _ => None,
    }
}

pub fn serialize_manifest(manifest: &RestoreManifest) -> String {
    let mut out = String::new();
    out.push_str("S3CC MANAGER RESTORE MANIFEST\n");
    out.push_str(&format!("version={}\n", manifest.version));
    out.push_str(&format!("created_at={}\n", manifest.created_at));
    out.push_str(&format!(
        "organization_language={}\n",
        language_code(manifest.organization_language)
    ));
    out.push_str(&format!("root={}\n", manifest.root.display()));
    out.push_str(&format!("status={}\n", manifest.status));
    out.push_str(&format!("files={}\n", manifest.entries.len()));
    out.push_str(&format!("created_directories={}\n", manifest.created_directories.len()));
    for directory in &manifest.created_directories {
        out.push_str(&format!("created_dir={}\n", directory.display()));
    }
    out.push('\n');

    for entry in &manifest.entries {
        out.push_str("[file]\n");
        out.push_str(&format!("sha256={}\n", entry.sha256));
        out.push_str(&format!("size={}\n", entry.size));
        out.push_str(&format!(
            "original={}\n",
            entry.original_relative_path.display()
        ));
        out.push_str(&format!(
            "organized={}\n",
            entry.organized_relative_path.display()
        ));
        out.push_str("[/file]\n\n");
    }

    out
}

pub fn parse_manifest(text: &str) -> Result<RestoreManifest, String> {
    let mut lines = text.lines();

    let header = lines.next().map(str::trim);
    if header != Some("S3CC MANAGER RESTORE MANIFEST")
        && header != Some("S3CC ORGANIZER RESTORE MANIFEST")
    {
        return Err("Invalid S3CC Manager restore manifest header.".to_string());
    }

    let mut version = None;
    let mut created_at = None;
    let mut organization_language = None;
    let mut root = None;
    let mut status = None;
    let mut declared_files = None;
    let mut declared_directories = None;
    let mut created_directories = Vec::new();
    let mut entries = Vec::new();

    let all_lines = lines.collect::<Vec<_>>();
    let mut index = 0usize;

    while index < all_lines.len() {
        let line = all_lines[index].trim();
        index += 1;

        if line.is_empty() {
            continue;
        }

        if line == "[file]" {
            let mut sha256 = None;
            let mut size = None;
            let mut original = None;
            let mut organized = None;
            let mut closed = false;

            while index < all_lines.len() {
                let file_line = all_lines[index].trim();
                index += 1;

                if file_line == "[/file]" {
                    closed = true;
                    break;
                }

                let Some((key, value)) = file_line.split_once('=') else {
                    return Err(format!("Malformed manifest file field: {file_line}"));
                };

                match key.trim() {
                    "sha256" => sha256 = Some(value.trim().to_string()),
                    "size" => {
                        size = Some(
                            value
                                .trim()
                                .parse::<u64>()
                                .map_err(|_| format!("Invalid file size: {value}"))?,
                        )
                    }
                    "original" => original = Some(PathBuf::from(value.trim())),
                    "organized" => organized = Some(PathBuf::from(value.trim())),
                    _ => {}
                }
            }

            if !closed {
                return Err("Unclosed [file] block in restore manifest.".to_string());
            }

            let sha256 = sha256.ok_or_else(|| "Manifest entry is missing sha256.".to_string())?;
            if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(format!("Invalid SHA-256 in restore manifest: {sha256}"));
            }

            entries.push(RestoreEntry {
                sha256,
                size: size.ok_or_else(|| "Manifest entry is missing size.".to_string())?,
                original_relative_path: original
                    .ok_or_else(|| "Manifest entry is missing original path.".to_string())?,
                organized_relative_path: organized
                    .ok_or_else(|| "Manifest entry is missing organized path.".to_string())?,
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("Malformed manifest line: {line}"));
        };

        match key.trim() {
            "version" => {
                version = Some(
                    value
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid manifest version: {value}"))?,
                )
            }
            "created_at" => created_at = Some(value.trim().to_string()),
            "organization_language" => {
                organization_language = parse_language(value)
            }
            "root" => root = Some(PathBuf::from(value.trim())),
            "status" => status = Some(value.trim().to_string()),
            "files" => {
                declared_files = Some(
                    value
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid files count: {value}"))?,
                )
            }
            "created_directories" => {
                declared_directories = Some(
                    value
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid created_directories count: {value}"))?,
                )
            }
            "created_dir" => created_directories.push(PathBuf::from(value.trim())),
            // Backward compatibility with the early preview format.
            "language" if organization_language.is_none() => {
                organization_language = parse_language(value)
            }
            "mode" => {}
            _ => {}
        }
    }

    let manifest = RestoreManifest {
        version: version.ok_or_else(|| "Manifest is missing version.".to_string())?,
        created_at: created_at.unwrap_or_default(),
        organization_language: organization_language
            .ok_or_else(|| "Manifest has an invalid or missing organization_language.".to_string())?,
        root: root.ok_or_else(|| "Manifest is missing root.".to_string())?,
        status: status.unwrap_or_else(|| "LEGACY".to_string()),
        created_directories,
        entries,
    };

    if manifest.version != 1 {
        return Err(format!(
            "Unsupported restore manifest version {}.",
            manifest.version
        ));
    }

    if let Some(expected) = declared_files {
        if expected != manifest.entries.len() {
            return Err(format!(
                "Manifest declares {expected} files but contains {} entries.",
                manifest.entries.len()
            ));
        }
    }

    if let Some(expected) = declared_directories {
        if expected != manifest.created_directories.len() {
            return Err(format!(
                "Manifest declares {expected} created directories but contains {}.",
                manifest.created_directories.len()
            ));
        }
    }

    Ok(manifest)
}

pub fn read_manifest(path: &Path) -> Result<RestoreManifest, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read manifest {}: {error}", path.display()))?;
    parse_manifest(&text)
}

pub fn write_manifest_atomic(path: &Path, manifest: &RestoreManifest) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Manifest path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create manifest directory {}: {error}", parent.display()))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("Invalid manifest filename: {}", path.display()))?;
    let temp = parent.join(format!(".{file_name}.tmp"));

    if temp.exists() {
        fs::remove_file(&temp)
            .map_err(|error| format!("Could not remove stale manifest temp file {}: {error}", temp.display()))?;
    }

    let content = serialize_manifest(manifest);
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| format!("Could not create manifest temp file {}: {error}", temp.display()))?;

        file.write_all(content.as_bytes())
            .map_err(|error| format!("Could not write manifest temp file {}: {error}", temp.display()))?;
        file.sync_all()
            .map_err(|error| format!("Could not sync manifest temp file {}: {error}", temp.display()))?;
    }

    if path.exists() {
        fs::remove_file(&temp).ok();
        return Err(format!(
            "Refusing to overwrite an existing restore manifest: {}",
            path.display()
        ));
    }

    fs::rename(&temp, path)
        .map_err(|error| format!("Could not commit restore manifest {}: {error}", path.display()))
}

pub fn replace_manifest_atomic(path: &Path, manifest: &RestoreManifest) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Manifest path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("Invalid manifest filename: {}", path.display()))?;
    let temp = parent.join(format!(".{file_name}.replace.tmp"));

    if temp.exists() {
        fs::remove_file(&temp).ok();
    }

    let content = serialize_manifest(manifest);
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| format!("Could not create manifest replacement {}: {error}", temp.display()))?;

        file.write_all(content.as_bytes())
            .map_err(|error| format!("Could not write manifest replacement {}: {error}", temp.display()))?;
        file.sync_all()
            .map_err(|error| format!("Could not sync manifest replacement {}: {error}", temp.display()))?;
    }

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("Could not replace manifest {}: {error}", path.display()))?;
    }

    fs::rename(&temp, path)
        .map_err(|error| format!("Could not finalize manifest replacement {}: {error}", path.display()))
}

pub fn sha256_file(path: &Path) -> io::Result<(String, u64)> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 128];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok((format!("{:X}", hasher.finalize()), size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_files_follow_current_interface_language() {
        let manifest = RestoreManifest {
            version: 1,
            created_at: "2026-09-29T17:00:00-03:00".to_string(),
            organization_language: AppLanguage::En,
            root: PathBuf::from("Packages"),
            status: "COMPLETE".to_string(),
            created_directories: vec![],
            entries: vec![],
        };

        assert_eq!(
            manifest.uncategorized_destination(
                AppLanguage::Pt,
                Path::new("CAS/Cabelos/Feminino/new.package")
            ),
            PathBuf::from("Não Categorizado/CAS/Cabelos/Feminino/new.package")
        );
    }

    #[test]
    fn manifest_round_trip_preserves_paths_and_identity() {
        let manifest = RestoreManifest {
            version: 1,
            created_at: "2026-09-29T17:00:00-03:00".to_string(),
            organization_language: AppLanguage::Pt,
            root: PathBuf::from(r"C:\Mods\Packages"),
            status: "COMPLETE".to_string(),
            created_directories: vec![],
            entries: vec![RestoreEntry {
                sha256: "A".repeat(64),
                size: 123,
                original_relative_path: PathBuf::from(r"Creator\x.package"),
                organized_relative_path: PathBuf::from(r"CAS\Roupas\x.package"),
            }],
        };

        let text = serialize_manifest(&manifest);
        let parsed = parse_manifest(&text).unwrap();

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.organization_language, AppLanguage::Pt);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].sha256, "A".repeat(64));
        assert_eq!(
            parsed.entries[0].original_relative_path,
            PathBuf::from(r"Creator\x.package")
        );
    }
}
