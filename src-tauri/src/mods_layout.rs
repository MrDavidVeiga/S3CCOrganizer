//! Detect The Sims 3 Mods layout from its real loading branches, including
//! backup copies (e.g. "Mods - Copia"), rather than relying on the folder name.
use std::path::Path;

/// The exact name "Mods" also supports unit tests and incomplete installations.
/// A renamed copy is a Mods root when Packages exists along with Resource.cfg
/// or Overrides. A random directory with a child named Packages is not enough.
pub fn is_mods_root(path: &Path) -> bool {
    if path.file_name().is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("Mods")) {
        return true;
    }
    path.join("Packages").is_dir()
        && (path.join("Resource.cfg").is_file() || path.join("Overrides").is_dir())
}

/// Used by scanning, planning and Resource.cfg updates. Never match an
/// unrelated parent above the selected loading tree.
pub fn mods_ancestor(path: &Path) -> Option<&Path> {
    path.ancestors().find(|ancestor| is_mods_root(ancestor))
}

/// A selected copy is still subject to its own loading-branch containment.
/// Content originating inside Packages cannot be moved beside Packages.
pub fn validate_organization_destination(
    selected_root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), String> {
    if !source.starts_with(selected_root) || !destination.starts_with(selected_root) {
        return Err("Organization source/destination escaped the selected root.".into());
    }
    let Some(mods) = mods_ancestor(selected_root) else {
        return Ok(()); // Standalone CC staging folder, not a Mods layout.
    };
    let packages = mods.join("Packages");
    let overrides = mods.join("Overrides");
    let branch = if source.starts_with(&overrides) {
        &overrides
    } else {
        // The planner explicitly admits recognized legacy categories beside
        // Packages and routes them back into Packages. Never move a package
        // into or out of Overrides as part of ordinary organization.
        &packages
    };
    if !destination.starts_with(branch) {
        return Err(format!(
            "Unsafe organization destination '{}': expected a path inside '{}'.",
            destination.display(), branch.display()
        ));
    }
    if source.file_name().is_some_and(|name| name.to_string_lossy().to_ascii_lowercase().ends_with(".package.disabled"))
        && !destination.file_name().is_some_and(|name| name.to_string_lossy().to_ascii_lowercase().ends_with(".package.disabled"))
    {
        return Err("An inactive .package.disabled file must never be reactivated by organization.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn recognizes_renamed_mods_copy_and_preserves_packages() {
        let base = std::env::temp_dir().join(format!(
            "s3cc-mods-copy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mods = base.join("Mods - Copia");
        fs::create_dir_all(mods.join("Packages")).unwrap();
        fs::create_dir_all(mods.join("Overrides")).unwrap();
        fs::write(mods.join("Resource.cfg"), b"Priority 500\nPackedFile Packages/*/*.package\n").unwrap();

        assert!(is_mods_root(&mods));
        assert_eq!(mods_ancestor(&mods.join("Packages/Old")), Some(mods.as_path()));
        assert!(validate_organization_destination(
            &mods,
            &mods.join("Packages/Old/file.package"),
            &mods.join("Packages/Sliders/file.package")
        ).is_ok());
        assert!(validate_organization_destination(
            &mods,
            &mods.join("Genética/Old/skin.package"),
            &mods.join("Packages/Genética/skin.package")
        ).is_ok());
        assert!(validate_organization_destination(
            &mods,
            &mods.join("Packages/Old/file.package"),
            &mods.join("Cabelos/file.package")
        ).is_err());
        assert!(validate_organization_destination(
            &mods,
            &mods.join("Overrides/test.package"),
            &mods.join("Packages/test.package")
        ).is_err());
        assert!(validate_organization_destination(
            &mods,
            &mods.join("Packages/off.package.disabled"),
            &mods.join("Packages/Sliders/off.package")
        ).is_err());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn refuses_guessing_mods_from_unrelated_directory() {
        let base = std::env::temp_dir().join(format!(
            "s3cc-plain-folder-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(base.join("Packages")).unwrap();
        assert!(!is_mods_root(&base));
        fs::remove_dir_all(base).unwrap();
    }
}
