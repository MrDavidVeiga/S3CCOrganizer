use crate::i18n::AppLanguage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
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
    /// Language used when the categorized layout was generated.
    /// This is audit metadata only; restore identity never depends on translated labels.
    pub organization_language: AppLanguage,
    pub root: PathBuf,
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
            organization_language: AppLanguage::En,
            root: PathBuf::from("Packages"),
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
}
