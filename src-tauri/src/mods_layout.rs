//! Detect The Sims 3 Mods layout from its real loading branches, including
//! backup copies (e.g. "Mods - Copia"), rather than relying on the folder name.
use std::path::Path;

/// The loading layout is identified by the actual Packages directory.
/// Its parent can be called Mods, Mods - Copia, Backup, Library, or anything
/// else. A folder named Mods with no Packages remains recognizable so the
/// planner can report the missing loading branch explicitly.
pub fn is_mods_root(path: &Path) -> bool {
    path.join("Packages").is_dir()
        || path.file_name().is_some_and(|name|
            name.to_string_lossy().eq_ignore_ascii_case("Mods"))
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
    // Source-path invariant: an existing Packages/Overrides ancestor is
    // authoritative, even if the parent has an arbitrary name, the user
    // selected Packages itself, or the selected root is farther above Mods.
    // Never infer the loading branch from the user's chosen scan-root name.
    let source_branch = source.ancestors().find(|ancestor| {
        ancestor.file_name().is_some_and(|name|
            name.to_string_lossy().eq_ignore_ascii_case("Packages")
                || name.to_string_lossy().eq_ignore_ascii_case("Overrides"))
    });
    if let Some(branch) = source_branch {
        if !destination.starts_with(branch) {
            return Err(format!(
                "Unsafe organization destination '{}': source is inside '{}', and may not leave that loading branch.",
                destination.display(), branch.display()
            ));
        }
    } else if let Some(mods) = mods_ancestor(selected_root) {
        // Recognized legacy categories beside Packages can be migrated into
        // Packages, never into a new sibling category or Overrides.
        let packages = mods.join("Packages");
        if !destination.starts_with(&packages) {
            return Err(format!(
                "Unsafe legacy migration destination '{}': expected '{}'.",
                destination.display(), packages.display()
            ));
        }
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
        // Copies remain recognized even when their optional cfg and
        // Overrides contents were not copied yet.
        fs::remove_file(mods.join("Resource.cfg")).unwrap();
        fs::remove_dir(mods.join("Overrides")).unwrap();
        assert!(is_mods_root(&mods));
        fs::create_dir_all(mods.join("Overrides")).unwrap();
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
    fn arbitrary_parent_name_does_not_change_packages_containment() {
        let base = std::env::temp_dir().join(format!(
            "s3cc-folder-independent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let library = base.join("My Custom CC Library");
        fs::create_dir_all(library.join("Packages/Old/More")).unwrap();
        fs::create_dir_all(library.join("Overrides")).unwrap();
        // Neither Resource.cfg nor a parent directory named Mods is needed.
        assert!(is_mods_root(&library));
        let source = library.join("Packages/Old/More/test.package");
        assert!(validate_organization_destination(&library, &source,
            &library.join("Packages/Sliders/Corpo/test.package")).is_ok());
        assert!(validate_organization_destination(&library, &source,
            &library.join("Sliders/Corpo/test.package")).is_err());
        assert!(validate_organization_destination(&library, &source,
            &library.join("Overrides/test.package")).is_err());

        // The same invariant applies when Packages itself is the scan root.
        let packages = library.join("Packages");
        assert!(validate_organization_destination(&packages, &source,
            &packages.join("Cabelos/test.package")).is_ok());
        assert!(validate_organization_destination(&packages, &source,
            &library.join("Cabelos/test.package")).is_err());

        // Selecting a higher parent still cannot change the branch identity.
        assert!(validate_organization_destination(&base, &source,
            &library.join("Cabelos/test.package")).is_err());
        assert!(validate_organization_destination(&base, &source,
            &library.join("Packages/Cabelos/test.package")).is_ok());

        // Overrides also retains its original branch.
        let overridden = library.join("Overrides/old.package");
        assert!(validate_organization_destination(&library, &overridden,
            &library.join("Overrides/Patches/old.package")).is_ok());
        assert!(validate_organization_destination(&library, &overridden,
            &library.join("Packages/old.package")).is_err());
        fs::remove_dir_all(base).unwrap();
    }
}
