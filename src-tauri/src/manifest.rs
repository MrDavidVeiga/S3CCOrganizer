use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManifestLanguage {
    En,
    Pt,
    Es,
}

impl ManifestLanguage {
    pub fn not_categorized_folder(&self) -> &'static str {
        match self {
            Self::En => "Not Categorized",
            Self::Pt => "Não Categorizado",
            Self::Es => "Sin categorizar",
        }
    }
}

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
    pub language: ManifestLanguage,
    pub root: PathBuf,
    pub entries: Vec<RestoreEntry>,
}

impl RestoreManifest {
    pub fn contains_identity(&self, sha256: &str, size: u64) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.sha256.eq_ignore_ascii_case(sha256) && entry.size == size)
    }

    pub fn uncategorized_destination(&self, current_relative_path: &Path) -> PathBuf {
        PathBuf::from(self.language.not_categorized_folder()).join(current_relative_path)
    }
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
    fn localization_is_stable() {
        assert_eq!(ManifestLanguage::En.not_categorized_folder(), "Not Categorized");
        assert_eq!(ManifestLanguage::Pt.not_categorized_folder(), "Não Categorizado");
        assert_eq!(ManifestLanguage::Es.not_categorized_folder(), "Sin categorizar");
    }

    #[test]
    fn new_files_keep_current_relative_structure() {
        let manifest = RestoreManifest {
            version: 1,
            language: ManifestLanguage::Pt,
            root: PathBuf::from("Packages"),
            entries: vec![],
        };
        assert_eq!(
            manifest.uncategorized_destination(Path::new("CAS/Hair/Female/new.package")),
            PathBuf::from("Não Categorizado/CAS/Hair/Female/new.package")
        );
    }
}
