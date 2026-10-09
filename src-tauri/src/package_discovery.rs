//! Fast, complete enumeration for Duplicate and Conflict analysis.
//! Cache fingerprints, not the file list: CC may be added or removed between
//! the initial Organizer scan and a later read-only analysis.
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct PackageDiscovery {
    pub active_paths: Vec<PathBuf>,
    pub total_files: usize,
    pub disabled_packages: usize,
    pub other_files: usize,
}

/// Every ordinary file beneath the selected root is counted. Only active
/// *.package files are sent to fingerprint/conflict analyzers; *.package.disabled
/// files remain visible to the Organizer without being considered loaded mods.
pub fn discover_active_packages(root: &Path) -> Result<PackageDiscovery, String> {
    let mut result = PackageDiscovery {
        active_paths: Vec::new(),
        total_files: 0,
        disabled_packages: 0,
        other_files: 0,
    };
    for item in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = item.map_err(|error|
            format!("Could not enumerate all files under {}: {error}", root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        result.total_files += 1;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name.ends_with(".package") {
            result.active_paths.push(entry.into_path());
        } else if name.ends_with(".package.disabled") {
            result.disabled_packages += 1;
        } else {
            result.other_files += 1;
        }
    }
    result.active_paths.sort_by_key(|path|
        path.to_string_lossy().to_ascii_lowercase());
    debug_assert_eq!(
        result.total_files,
        result.active_paths.len() + result.disabled_packages + result.other_files
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn counts_every_file_without_capping_at_two_thousand() {
        let base = std::env::temp_dir().join(format!(
            "s3cc-discovery-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_nanos()
        ));
        fs::create_dir_all(base.join("Packages/Nested")).unwrap();
        for i in 0..2429 {
            fs::write(base.join("Packages/Nested").join(format!("{i:05}.package")), b"").unwrap();
        }
        fs::write(base.join("Packages/Nested/disabled.package.disabled"), b"").unwrap();
        fs::write(base.join("Packages/Nested/readme.txt"), b"").unwrap();
        let discovery = discover_active_packages(&base).unwrap();
        assert_eq!(discovery.total_files, 2431);
        assert_eq!(discovery.active_paths.len(), 2429);
        assert_eq!(discovery.disabled_packages, 1);
        assert_eq!(discovery.other_files, 1);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn no_name_assumption_or_cache_dependency() {
        let base = std::env::temp_dir().join(format!(
            "s3cc-discovery-other-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_nanos()
        ));
        fs::create_dir_all(base.join("Packages")).unwrap();
        fs::write(base.join("Packages/one.package"), b"").unwrap();
        let first = discover_active_packages(&base).unwrap();
        assert_eq!(first.active_paths.len(), 1);
        fs::write(base.join("Packages/two.package"), b"").unwrap();
        let second = discover_active_packages(&base).unwrap();
        assert_eq!(second.active_paths.len(), 2);
        fs::remove_file(base.join("Packages/one.package")).unwrap();
        let third = discover_active_packages(&base).unwrap();
        assert_eq!(third.active_paths.len(), 1);
        fs::remove_dir_all(base).unwrap();
    }
}
