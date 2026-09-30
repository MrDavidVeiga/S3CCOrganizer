use crate::{
    catalog::{classify_resource, CatalogClassification, TYPE_CASP, TYPE_OBJD},
    dbpf::Package,
    i18n::AppLanguage,
    operation::{self, CANCELLED_ERROR},
    package_family::{
        classify_package_family, PackageFamilyClassification, PackageFamilyResult, TYPE_BBLN,
        TYPE_BGEO, TYPE_BONE_DELTA, TYPE_FACE, TYPE_FBLN, TYPE_HAIR_TONE, TYPE_S3SA,
        TYPE_SKIN_TONE,
    },
};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanPackageItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub file_size: u64,
    pub resource_count: usize,
    pub catalog_resource_count: usize,
    pub resource_types: Vec<String>,
    pub status: String,
    pub detected_from: Vec<String>,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub gender: Option<String>,
    pub age: Option<String>,
    pub species: Option<String>,
    pub usage_categories: Vec<String>,
    pub destination_parts: Vec<String>,
    pub destination_path: Option<String>,
    pub candidate_destinations: Vec<String>,
    pub classifications: Vec<CatalogClassification>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanStats {
    pub packages: usize,
    pub classified: usize,
    pub mixed: usize,
    pub unknown: usize,
    pub needs_review: usize,
    pub invalid: usize,
    pub casp_resources: usize,
    pub objd_resources: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub root: String,
    pub items: Vec<ScanPackageItem>,
    pub stats: ScanStats,
}

fn package_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("package"))
        .unwrap_or(false)
}

fn resource_type_label(type_id: u32) -> String {
    match type_id {
        TYPE_CASP => "CASP".to_string(),
        TYPE_OBJD => "OBJD".to_string(),
        0x0333_406C => "XML".to_string(),
        0x03B3_3DDF => "ITUN".to_string(),
        TYPE_S3SA => "S3SA".to_string(),
        TYPE_SKIN_TONE => "SkinTone".to_string(),
        TYPE_HAIR_TONE => "HairTone".to_string(),
        TYPE_BONE_DELTA => "BoneDelta".to_string(),
        TYPE_FACE => "FACE".to_string(),
        TYPE_BBLN => "BBLN".to_string(),
        TYPE_BGEO => "BGEO".to_string(),
        TYPE_FBLN => "FBLN".to_string(),
        0x73E9_3EEB => "Manifest".to_string(),
        0x2205_57DA => "STBL".to_string(),
        0x00B2_D882 => "IMG".to_string(),
        0x015A_1849 => "GEOM".to_string(),
        0x7368_84F1 => "VPXY".to_string(),
        0xD4D9_FBE5 => "Pattern".to_string(),
        other => format!("0x{other:08X}"),
    }
}

fn localized_unknown(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::En => "Unknown",
        AppLanguage::Pt => "Desconhecido",
        AppLanguage::Es => "Desconocido",
    }
}

fn localized_mixed(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::En => "Mixed",
        AppLanguage::Pt => "Misto",
        AppLanguage::Es => "Mixto",
    }
}

fn localized_warning(language: AppLanguage, key: &str) -> &'static str {
    match (language, key) {
        (AppLanguage::En, "parse") => "A CASP/OBJD resource could not be fully interpreted.",
        (AppLanguage::Pt, "parse") => "Um resource CASP/OBJD não pôde ser interpretado completamente.",
        (AppLanguage::Es, "parse") => "Un resource CASP/OBJD no se pudo interpretar completamente.",
        (AppLanguage::En, "mixed") => "The package contains catalog resources with different destinations.",
        (AppLanguage::Pt, "mixed") => "O package contém resources de catálogo com destinos diferentes.",
        (AppLanguage::Es, "mixed") => "El package contiene resources de catálogo con destinos diferentes.",
        (AppLanguage::En, "none") => "No supported CASP/OBJD classification was found.",
        (AppLanguage::Pt, "none") => "Nenhuma classificação CASP/OBJD compatível foi encontrada.",
        (AppLanguage::Es, "none") => "No se encontró una clasificación CASP/OBJD compatible.",
        (AppLanguage::En, "ambiguous") => "The catalog flags indicate more than one possible destination.",
        (AppLanguage::Pt, "ambiguous") => "Os flags de catálogo indicam mais de um destino possível.",
        (AppLanguage::Es, "ambiguous") => "Los flags del catálogo indican más de un destino posible.",
        (AppLanguage::En, "family_ambiguous") => "The package contains multiple authoritative resource families and needs review.",
        (AppLanguage::Pt, "family_ambiguous") => "O package contém várias famílias de resources autoritativas e precisa de revisão.",
        (AppLanguage::Es, "family_ambiguous") => "El package contiene varias familias de resources autoritativas y necesita revisión.",
        _ => "Unknown",
    }
}

fn scan_one(root: &Path, path: &Path, language: AppLanguage) -> ScanPackageItem {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("package")
        .to_string();
    let id = path.to_string_lossy().to_string();
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let package = match Package::load(path) {
        Ok(package) => package,
        Err(error) => {
            return ScanPackageItem {
                id,
                name,
                path: path.to_string_lossy().to_string(),
                relative_path: relative,
                file_size,
                resource_count: 0,
                catalog_resource_count: 0,
                resource_types: Vec::new(),
                status: "invalid".to_string(),
                detected_from: Vec::new(),
                category: None,
                sub_category: None,
                gender: None,
                age: None,
                species: None,
                usage_categories: Vec::new(),
                destination_parts: Vec::new(),
                destination_path: None,
                candidate_destinations: Vec::new(),
                classifications: Vec::new(),
                warnings: vec![error.to_string()],
            };
        }
    };

    let mut type_set = BTreeSet::new();
    let mut type_ids = BTreeSet::new();
    let mut detected_from = BTreeSet::new();
    let mut classifications = Vec::new();
    let mut warnings = Vec::new();
    let mut catalog_resource_count = 0usize;

    for entry in &package.entries {
        type_ids.insert(entry.type_id);
        type_set.insert(resource_type_label(entry.type_id));
        if !matches!(entry.type_id, TYPE_CASP | TYPE_OBJD) {
            continue;
        }

        catalog_resource_count += 1;
        detected_from.insert(resource_type_label(entry.type_id));

        match package.data(entry) {
            Ok(data) => match classify_resource(entry.type_id, &data, language) {
                Some(classification) => classifications.push(classification),
                None => warnings.push(format!(
                    "{} {}",
                    localized_warning(language, "parse"),
                    entry.key_string()
                )),
            },
            Err(error) => warnings.push(format!("{}: {error}", entry.key_string())),
        }
    }

    let mut destinations: HashMap<String, CatalogClassification> = HashMap::new();
    let mut candidate_destinations = BTreeSet::new();
    let mut has_ambiguous = false;

    for classification in &classifications {
        for parts in &classification.candidate_folder_parts {
            if !parts.is_empty() {
                candidate_destinations.insert(parts.join("\\"));
            }
        }

        if classification.ambiguous {
            has_ambiguous = true;
            continue;
        }

        if !classification.folder_parts.is_empty() {
            let key = classification.folder_parts.join("\\");
            destinations
                .entry(key)
                .or_insert_with(|| classification.clone());
        }
    }

    let (mut status, primary, mut destination_parts, mut destination_path) = if has_ambiguous {
        warnings.push(localized_warning(language, "ambiguous").to_string());
        (
            "needs_review".to_string(),
            None,
            Vec::new(),
            None,
        )
    } else if destinations.len() == 1 {
        let (path, classification) = destinations.into_iter().next().unwrap();
        (
            "classified".to_string(),
            Some(classification.clone()),
            classification.folder_parts.clone(),
            Some(path),
        )
    } else if destinations.len() > 1 {
        warnings.push(localized_warning(language, "mixed").to_string());
        (
            "mixed".to_string(),
            None,
            Vec::new(),
            None,
        )
    } else if catalog_resource_count > 0 {
        (
            "needs_review".to_string(),
            None,
            Vec::new(),
            None,
        )
    } else {
        (
            "unknown".to_string(),
            None,
            Vec::new(),
            None,
        )
    };

    let mut family_primary: Option<PackageFamilyClassification> = None;

    if status == "unknown" && catalog_resource_count == 0 {
        match classify_package_family(&type_ids, language) {
            PackageFamilyResult::Classified(classification) => {
                for source in &classification.detected_from {
                    detected_from.insert(source.clone());
                }
                destination_parts = classification.folder_parts.clone();
                destination_path = Some(destination_parts.join("\\"));
                status = "classified".to_string();
                family_primary = Some(classification);
            }
            PackageFamilyResult::Ambiguous(candidates) => {
                for classification in candidates {
                    for source in &classification.detected_from {
                        detected_from.insert(source.clone());
                    }
                    if !classification.folder_parts.is_empty() {
                        candidate_destinations.insert(classification.folder_parts.join("\\"));
                    }
                }
                warnings.push(localized_warning(language, "family_ambiguous").to_string());
                status = "needs_review".to_string();
            }
            PackageFamilyResult::None => {
                warnings.push(localized_warning(language, "none").to_string());
            }
        }
    }

    let mut usage_categories = BTreeSet::new();
    for classification in &classifications {
        for usage in &classification.usage_categories {
            usage_categories.insert(usage.clone());
        }
    }

    let (category, sub_category, gender, age, species) = if let Some(primary) = primary {
        (
            Some(primary.main_category),
            primary.sub_category,
            primary.gender,
            primary.age,
            primary.species,
        )
    } else if let Some(family) = family_primary {
        (
            Some(family.main_category),
            family.sub_category,
            None,
            None,
            None,
        )
    } else if status == "mixed" {
        (
            Some(localized_mixed(language).to_string()),
            None,
            None,
            None,
            None,
        )
    } else {
        (
            Some(localized_unknown(language).to_string()),
            None,
            None,
            None,
            None,
        )
    };

    ScanPackageItem {
        id,
        name,
        path: path.to_string_lossy().to_string(),
        relative_path: relative,
        file_size,
        resource_count: package.entries.len(),
        catalog_resource_count,
        resource_types: type_set.into_iter().collect(),
        status,
        detected_from: detected_from.into_iter().collect(),
        category,
        sub_category,
        gender,
        age,
        species,
        usage_categories: usage_categories.into_iter().collect(),
        destination_parts,
        destination_path,
        candidate_destinations: candidate_destinations.into_iter().collect(),
        classifications,
        warnings,
    }
}

pub fn scan_packages_core(
    folder: String,
    language: AppLanguage,
    operation_kind: Option<&str>,
) -> Result<ScanResult, String> {
    let root = PathBuf::from(folder.trim());
    if folder.trim().is_empty() {
        return Err("No folder was selected.".to_string());
    }
    if !root.is_dir() {
        return Err(format!("Folder does not exist: {}", root.display()));
    }

    let mut package_paths = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && package_extension(entry.path()))
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    package_paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());

    if let Some(kind) = operation_kind {
        operation::set_total(kind, package_paths.len());
        operation::update(kind, 0, None, "scanning");
    }

    let mut stats = ScanStats::default();
    let mut items = Vec::with_capacity(package_paths.len());

    for (index, path) in package_paths.iter().enumerate() {
        if let Some(kind) = operation_kind {
            if operation::is_cancelled(kind) {
                return Err(CANCELLED_ERROR.to_string());
            }
            operation::update(
                kind,
                index,
                path.file_name().map(|value| value.to_string_lossy().to_string()),
                "scanning",
            );
        }

        let item = scan_one(&root, path, language);
        stats.packages += 1;
        stats.casp_resources += item
            .classifications
            .iter()
            .filter(|classification| classification.source == "CASP")
            .count();
        stats.objd_resources += item
            .classifications
            .iter()
            .filter(|classification| classification.source == "OBJD")
            .count();

        match item.status.as_str() {
            "classified" => stats.classified += 1,
            "mixed" => stats.mixed += 1,
            "needs_review" => stats.needs_review += 1,
            "invalid" => stats.invalid += 1,
            _ => stats.unknown += 1,
        }

        items.push(item);

        if let Some(kind) = operation_kind {
            operation::update(
                kind,
                index + 1,
                path.file_name().map(|value| value.to_string_lossy().to_string()),
                "scanning",
            );
        }
    }

    Ok(ScanResult {
        root: root.to_string_lossy().to_string(),
        items,
        stats,
    })
}

#[tauri::command]
pub async fn scan_packages(folder: String, language: AppLanguage) -> Result<ScanResult, String> {
    const KIND: &str = "scan";
    operation::begin(KIND, "starting");

    let result = tauri::async_runtime::spawn_blocking(move || {
        scan_packages_core(folder, language, Some(KIND))
    })
    .await
    .map_err(|error| format!("Scanner worker failed: {error}"))?;

    match &result {
        Ok(_) => operation::finish(KIND, "complete", None),
        Err(error) if error == CANCELLED_ERROR => operation::mark_cancelled(KIND),
        Err(error) => operation::finish(KIND, "error", Some(error.clone())),
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_extension_is_case_insensitive() {
        assert!(package_extension(Path::new("Hair.package")));
        assert!(package_extension(Path::new("Hair.PACKAGE")));
        assert!(!package_extension(Path::new("Hair.sims3pack")));
    }
}
