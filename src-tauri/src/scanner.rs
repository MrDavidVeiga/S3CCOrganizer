use crate::{
    catalog::{classify_resource, CatalogClassification, TYPE_CASP, TYPE_OBJD},
    dbpf::Package,
    i18n::AppLanguage,
    manifest::sha256_file,
    operation::{self, CANCELLED_ERROR},
    workspace::{load_workspace_for_root, split_destination},
    package_family::{
        classify_package_family, PackageFamilyClassification, PackageFamilyResult, TYPE_BBLN,
        TYPE_BGEO, TYPE_BONE_DELTA, TYPE_FACE, TYPE_FBLN, TYPE_HAIR_TONE, TYPE_S3SA,
        TYPE_SKIN_TONE,
    },
};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
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
    pub classification_reason: Option<String>,
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
    pub total_ms: u128,
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


const TYPE_NMAP_LOCAL: u32 = 0x0166_038C;
const TYPE_STBL_LOCAL: u32 = 0x2205_57DA;
const TYPE_MANIFEST_LOCAL: u32 = 0x73E9_3EEB;

fn nmap_names(data: &[u8]) -> Vec<String> {
    if data.len() < 8 {
        return Vec::new();
    }

    let version = u32::from_le_bytes(data[0..4].try_into().unwrap());
    if version != 1 {
        return Vec::new();
    }
    let count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
    let mut offset = 8usize;
    let mut names = Vec::with_capacity(count.min(64));

    for _ in 0..count {
        if offset.checked_add(12).map(|end| end <= data.len()) != Some(true) {
            return Vec::new();
        }
        offset += 8; // name hash
        let byte_count =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let Some(next) = offset.checked_add(byte_count) else {
            return Vec::new();
        };
        if next > data.len() {
            return Vec::new();
        }
        let name = String::from_utf8_lossy(&data[offset..next]).trim().to_string();
        if !name.is_empty() {
            names.push(name);
        }
        offset = next;
    }

    names
}

fn stbl_entries(data: &[u8]) -> Vec<(u64, String)> {
    if data.len() < 17 || &data[..4] != b"STBL" {
        return Vec::new();
    }

    let count = u32::from_le_bytes(data[7..11].try_into().unwrap()) as usize;
    let mut offset = 17usize;
    let mut entries = Vec::with_capacity(count.min(4096));

    for _ in 0..count {
        if offset.checked_add(12).map(|end| end <= data.len()) != Some(true) {
            return Vec::new();
        }
        let key = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let char_count =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let Some(byte_count) = char_count.checked_mul(2) else {
            return Vec::new();
        };
        let Some(next) = offset.checked_add(byte_count) else {
            return Vec::new();
        };
        if next > data.len() {
            return Vec::new();
        }

        let units = data[offset..next]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
        let text = char::decode_utf16(units)
            .map(|value| value.unwrap_or('\u{FFFD}'))
            .collect::<String>();
        entries.push((key, text));
        offset = next;
    }

    entries
}

fn normalize_slider_internal_name(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    let mut previous_lower = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && previous_lower && !out.ends_with(' ') {
                out.push(' ');
            }
            out.push(ch.to_ascii_lowercase());
            previous_lower = ch.is_ascii_lowercase();
        } else {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            previous_lower = false;
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn token_starts_with_any(normalized: &str, roots: &[&str]) -> bool {
    normalized
        .split_whitespace()
        .any(|token| roots.iter().any(|root| token.starts_with(root)))
}

fn token_contains(normalized: &str, value: &str) -> bool {
    normalized
        .split_whitespace()
        .any(|token| token.contains(value))
}

fn slider_region_keys(internal_name: &str) -> Option<(&'static str, Option<&'static str>)> {
    let name = normalize_slider_internal_name(internal_name);

    // Specific anatomy always wins over generic words such as "height".
    // This is intentionally based on internal NMAP/STBL names, never on the
    // CAS panel/category in which the creator placed the slider.
    if token_starts_with_any(&name, &["eyebrow", "brow"]) {
        return Some(("face", Some("eyebrows")));
    }
    if token_starts_with_any(&name, &["eyelid", "eyeball", "eye", "pupil", "iris"]) {
        return Some(("face", Some("eyes")));
    }
    if token_starts_with_any(&name, &["nostril", "septum", "nose", "bridge"]) {
        return Some(("face", Some("nose")));
    }
    if token_starts_with_any(
        &name,
        &["lip", "mouth", "philtrum", "cupid", "overlip", "frown", "smile"],
    ) {
        return Some(("face", Some("mouth_lips")));
    }
    if token_starts_with_any(&name, &["jaw", "chin"]) {
        return Some(("face", Some("jaw_chin")));
    }
    if token_starts_with_any(&name, &["cheek"]) || token_contains(&name, "cheek") {
        return Some(("face", Some("cheeks")));
    }
    if token_starts_with_any(&name, &["ear", "midear"]) {
        return Some(("face", Some("ears")));
    }
    if token_starts_with_any(&name, &["forehead"]) {
        return Some(("face", Some("forehead")));
    }
    if token_starts_with_any(&name, &["face"]) {
        return Some(("face", None));
    }

    if token_starts_with_any(&name, &["hat"]) {
        return Some(("head", Some("hats")));
    }
    if token_starts_with_any(&name, &["glasses"]) {
        return Some(("head", Some("glasses")));
    }
    if token_starts_with_any(&name, &["head", "skull"]) {
        return Some(("head", None));
    }

    if token_starts_with_any(&name, &["shoulder"]) {
        return Some(("body", Some("shoulders")));
    }
    if token_starts_with_any(&name, &["forearm", "arm", "bicep", "tricep", "elbow"]) {
        return Some(("body", Some("arms")));
    }
    if token_starts_with_any(&name, &["hand", "finger", "thumb", "nail"]) {
        return Some(("body", Some("hands")));
    }
    if token_starts_with_any(&name, &["breast", "chest", "nipple", "ribcage"]) {
        return Some(("body", Some("chest_breasts")));
    }
    if token_starts_with_any(&name, &["waist"]) {
        return Some(("body", Some("waist")));
    }
    if token_starts_with_any(&name, &["hip", "butt", "pelvis"]) {
        return Some(("body", Some("hips_butt")));
    }
    if token_starts_with_any(&name, &["leg", "thigh", "calf", "knee"]) {
        return Some(("body", Some("legs")));
    }
    if token_starts_with_any(&name, &["foot", "feet", "ankle", "toe"]) {
        return Some(("body", Some("feet")));
    }
    if token_starts_with_any(&name, &["neck"]) {
        return Some(("body", Some("neck")));
    }
    if token_starts_with_any(&name, &["torso", "body", "belly", "abdomen", "stomach", "back"]) {
        return Some(("body", Some("torso")));
    }
    if token_starts_with_any(&name, &["simheight", "height", "posture", "simscaler"]) {
        return Some(("body", Some("height_posture")));
    }

    None
}

fn slider_folder_label(language: AppLanguage, key: &str) -> &'static str {
    match (language, key) {
        (AppLanguage::En, "face") => "Face",
        (AppLanguage::Pt, "face") => "Rosto",
        (AppLanguage::Es, "face") => "Rostro",
        (AppLanguage::En, "body") => "Body",
        (AppLanguage::Pt, "body") => "Corpo",
        (AppLanguage::Es, "body") => "Cuerpo",
        (AppLanguage::En, "head") => "Head",
        (AppLanguage::Pt, "head") => "Cabeça",
        (AppLanguage::Es, "head") => "Cabeza",
        (AppLanguage::En, "eyebrows") => "Eyebrows",
        (AppLanguage::Pt, "eyebrows") => "Sobrancelhas",
        (AppLanguage::Es, "eyebrows") => "Cejas",
        (AppLanguage::En, "eyes") => "Eyes",
        (AppLanguage::Pt, "eyes") => "Olhos",
        (AppLanguage::Es, "eyes") => "Ojos",
        (AppLanguage::En, "nose") => "Nose",
        (AppLanguage::Pt, "nose") => "Nariz",
        (AppLanguage::Es, "nose") => "Nariz",
        (AppLanguage::En, "mouth_lips") => "Mouth & Lips",
        (AppLanguage::Pt, "mouth_lips") => "Boca e Lábios",
        (AppLanguage::Es, "mouth_lips") => "Boca y Labios",
        (AppLanguage::En, "jaw_chin") => "Jaw & Chin",
        (AppLanguage::Pt, "jaw_chin") => "Mandíbula e Queixo",
        (AppLanguage::Es, "jaw_chin") => "Mandíbula y Mentón",
        (AppLanguage::En, "cheeks") => "Cheeks",
        (AppLanguage::Pt, "cheeks") => "Bochechas",
        (AppLanguage::Es, "cheeks") => "Mejillas",
        (AppLanguage::En, "ears") => "Ears",
        (AppLanguage::Pt, "ears") => "Orelhas",
        (AppLanguage::Es, "ears") => "Orejas",
        (AppLanguage::En, "forehead") => "Forehead",
        (AppLanguage::Pt, "forehead") => "Testa",
        (AppLanguage::Es, "forehead") => "Frente",
        (AppLanguage::En, "hats") => "Hats",
        (AppLanguage::Pt, "hats") => "Chapéus",
        (AppLanguage::Es, "hats") => "Sombreros",
        (AppLanguage::En, "glasses") => "Glasses",
        (AppLanguage::Pt, "glasses") => "Óculos",
        (AppLanguage::Es, "glasses") => "Gafas",
        (AppLanguage::En, "shoulders") => "Shoulders",
        (AppLanguage::Pt, "shoulders") => "Ombros",
        (AppLanguage::Es, "shoulders") => "Hombros",
        (AppLanguage::En, "arms") => "Arms",
        (AppLanguage::Pt, "arms") => "Braços",
        (AppLanguage::Es, "arms") => "Brazos",
        (AppLanguage::En, "hands") => "Hands",
        (AppLanguage::Pt, "hands") => "Mãos",
        (AppLanguage::Es, "hands") => "Manos",
        (AppLanguage::En, "chest_breasts") => "Chest & Breasts",
        (AppLanguage::Pt, "chest_breasts") => "Peito e Seios",
        (AppLanguage::Es, "chest_breasts") => "Pecho y Senos",
        (AppLanguage::En, "waist") => "Waist",
        (AppLanguage::Pt, "waist") => "Cintura",
        (AppLanguage::Es, "waist") => "Cintura",
        (AppLanguage::En, "hips_butt") => "Hips & Butt",
        (AppLanguage::Pt, "hips_butt") => "Quadris e Bumbum",
        (AppLanguage::Es, "hips_butt") => "Caderas y Glúteos",
        (AppLanguage::En, "legs") => "Legs",
        (AppLanguage::Pt, "legs") => "Pernas",
        (AppLanguage::Es, "legs") => "Piernas",
        (AppLanguage::En, "feet") => "Feet",
        (AppLanguage::Pt, "feet") => "Pés",
        (AppLanguage::Es, "feet") => "Pies",
        (AppLanguage::En, "neck") => "Neck",
        (AppLanguage::Pt, "neck") => "Pescoço",
        (AppLanguage::Es, "neck") => "Cuello",
        (AppLanguage::En, "torso") => "Torso",
        (AppLanguage::Pt, "torso") => "Tronco",
        (AppLanguage::Es, "torso") => "Torso",
        (AppLanguage::En, "height_posture") => "Height & Posture",
        (AppLanguage::Pt, "height_posture") => "Altura e Postura",
        (AppLanguage::Es, "height_posture") => "Altura y Postura",
        _ => "Unknown",
    }
}

fn slider_destination_from_internal_name(
    internal_name: &str,
    language: AppLanguage,
) -> Option<Vec<String>> {
    let (region, part) = slider_region_keys(internal_name)?;
    let mut result = vec![
        "CAS".to_string(),
        "Sliders".to_string(),
        slider_folder_label(language, region).to_string(),
    ];
    if let Some(part) = part {
        result.push(slider_folder_label(language, part).to_string());
    }
    Some(result)
}

fn slider_internal_candidates(package: &Package) -> Vec<(String, &'static str)> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::<String>::new();

    for entry in &package.entries {
        if entry.type_id != TYPE_NMAP_LOCAL {
            continue;
        }
        let Ok(data) = package.data(entry) else {
            continue;
        };
        if let Some(name) = nmap_names(&data).into_iter().next() {
            let key = name.to_ascii_lowercase();
            if seen.insert(key) {
                candidates.push((name, "NMAP"));
            }
        }
    }

    let mut stbl_resources = package
        .entries
        .iter()
        .filter(|entry| entry.type_id == TYPE_STBL_LOCAL)
        .collect::<Vec<_>>();
    stbl_resources.sort_by_key(|entry| ((entry.instance >> 56) != 0, entry.instance));

    for entry in stbl_resources {
        let Ok(data) = package.data(entry) else {
            continue;
        };
        if let Some((_, text)) = stbl_entries(&data)
            .into_iter()
            .find(|(_, text)| !text.trim().is_empty())
        {
            let text = text.trim().to_string();
            let key = text.to_ascii_lowercase();
            if seen.insert(key) {
                candidates.push((text, "STBL"));
            }
        }
    }

    candidates
}

fn slider_internal_evidence(
    package: &Package,
    language: AppLanguage,
) -> Option<(String, &'static str, Option<Vec<String>>)> {
    let candidates = slider_internal_candidates(package);
    for (name, source) in &candidates {
        if let Some(destination) = slider_destination_from_internal_name(name, language) {
            return Some((name.clone(), *source, Some(destination)));
        }
    }
    candidates
        .into_iter()
        .next()
        .map(|(name, source)| (name, source, None))
}


fn is_slider_morph_type(type_id: u32) -> bool {
    matches!(
        type_id,
        TYPE_BONE_DELTA | TYPE_FACE | TYPE_BBLN | TYPE_BGEO | TYPE_FBLN
    )
}

fn stbl_keys(data: &[u8]) -> Vec<u64> {
    stbl_entries(data).into_iter().map(|(key, _)| key).collect()
}

fn apply_slider_companion_classification(
    package_paths: &[PathBuf],
    items: &mut [ScanPackageItem],
) {
    let mut slider_instances = HashSet::<u64>::new();

    for path in package_paths {
        let Ok(package) = Package::load(path) else {
            continue;
        };
        for entry in &package.entries {
            if is_slider_morph_type(entry.type_id) {
                slider_instances.insert(entry.instance);
            }
        }
    }

    if slider_instances.is_empty() {
        return;
    }

    for (path, item) in package_paths.iter().zip(items.iter_mut()) {
        let Ok(package) = Package::load(path) else {
            continue;
        };

        let type_ids = package
            .entries
            .iter()
            .map(|entry| entry.type_id)
            .collect::<BTreeSet<_>>();

        let localization_only = type_ids.contains(&TYPE_STBL_LOCAL)
            && type_ids.iter().all(|type_id| {
                matches!(
                    *type_id,
                    TYPE_STBL_LOCAL | TYPE_NMAP_LOCAL | TYPE_MANIFEST_LOCAL
                )
            });
        if !localization_only {
            continue;
        }

        let mut matched_key = None;
        'resources: for entry in &package.entries {
            if entry.type_id != TYPE_STBL_LOCAL {
                continue;
            }
            let Ok(data) = package.data(entry) else {
                continue;
            };
            for key in stbl_keys(&data) {
                if slider_instances.contains(&key) {
                    matched_key = Some(key);
                    break 'resources;
                }
            }
        }

        let Some(key) = matched_key else {
            continue;
        };

        item.status = "classified".to_string();
        item.category = Some("CAS".to_string());
        item.sub_category = Some("Sliders".to_string());
        item.destination_parts = vec!["CAS".to_string(), "Sliders".to_string()];
        item.destination_path = Some("CAS\\Sliders".to_string());
        if !item
            .detected_from
            .iter()
            .any(|source| source == "STBL→Slider")
        {
            item.detected_from.push("STBL→Slider".to_string());
            item.detected_from.sort();
        }
        item.classification_reason = Some(format!(
            "STBL entry key 0x{key:016X} matches a morph resource instance in another package from the selected set => CAS\\Sliders"
        ));
    }
}

fn apply_manual_classifications(root: &Path, items: &mut [ScanPackageItem]) {
    let workspace = load_workspace_for_root(root);
    if workspace.manual_classifications.is_empty() {
        return;
    }

    for item in items.iter_mut() {
        if item.status == "invalid" {
            continue;
        }

        let path = PathBuf::from(&item.path);
        let Ok((hash, _)) = sha256_file(&path) else {
            continue;
        };
        let Some(manual) = workspace.manual_classifications.get(&hash) else {
            continue;
        };
        let parts = split_destination(&manual.destination);
        if parts.is_empty() {
            continue;
        }

        item.status = "classified".to_string();
        item.destination_parts = parts.clone();
        item.destination_path = Some(parts.join("\\"));
        item.detected_from.push("ManualReview".to_string());
        item.detected_from.sort();
        item.detected_from.dedup();
        item.classification_reason = Some(format!(
            "Manual review stored by SHA-256 {} => {}",
            hash,
            parts.join("\\")
        ));
        if item.category.as_deref() == Some("Desconhecido")
            || item.category.as_deref() == Some("Desconocido")
            || item.category.as_deref() == Some("Unknown")
            || item.category.is_none()
        {
            item.category = parts.first().cloned();
            item.sub_category = parts.get(1).cloned();
        }
    }
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
                classification_reason: None,
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
    let mut classification_reason = primary
        .as_ref()
        .map(|classification| classification.technical_reason.clone());

    if status == "unknown" && catalog_resource_count == 0 {
        match classify_package_family(&type_ids, language) {
            PackageFamilyResult::Classified(classification) => {
                for source in &classification.detected_from {
                    detected_from.insert(source.clone());
                }
                let is_slider_family = classification.sub_category.as_deref() == Some("Sliders");
                destination_parts = classification.folder_parts.clone();
                destination_path = Some(destination_parts.join("\\"));
                status = "classified".to_string();
                classification_reason = Some(classification.technical_reason.clone());
                family_primary = Some(classification);

                if is_slider_family {
                    if let Some((internal_name, source, refined_destination)) =
                        slider_internal_evidence(&package, language)
                    {
                        detected_from.insert(source.to_string());
                        if let Some(parts) = refined_destination {
                            destination_parts = parts;
                            destination_path = Some(destination_parts.join("\\"));
                            classification_reason = Some(format!(
                                "{} | Internal slider name from {source} '{}' => {}",
                                classification_reason.unwrap_or_default(),
                                internal_name,
                                destination_parts.join("\\")
                            ));
                        } else {
                            classification_reason = Some(format!(
                                "{} | Internal slider name from {source} '{}' did not safely identify an anatomical region; kept at CAS\\Sliders.",
                                classification_reason.unwrap_or_default(),
                                internal_name
                            ));
                        }
                    }
                }
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

    if classification_reason.is_none() && !classifications.is_empty() {
        classification_reason = Some(
            classifications
                .iter()
                .map(|classification| classification.technical_reason.clone())
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }

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
        classification_reason,
        warnings,
    }
}

pub fn scan_packages_core(
    folder: String,
    language: AppLanguage,
    operation_kind: Option<&str>,
) -> Result<ScanResult, String> {
    let total_started = Instant::now();
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

        items.push(scan_one(&root, path, language));

        if let Some(kind) = operation_kind {
            operation::update(
                kind,
                index + 1,
                path.file_name().map(|value| value.to_string_lossy().to_string()),
                "scanning",
            );
        }
    }

    apply_slider_companion_classification(&package_paths, &mut items);
    apply_manual_classifications(&root, &mut items);

    let mut stats = ScanStats::default();
    stats.packages = items.len();
    for item in &items {
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
    }

    stats.total_ms = total_started.elapsed().as_millis();

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

    let joined = tauri::async_runtime::spawn_blocking(move || {
        scan_packages_core(folder, language, Some(KIND))
    })
    .await;

    let result = match joined {
        Ok(result) => result,
        Err(error) => {
            let message = format!("Scanner worker failed: {error}");
            operation::finish(KIND, "error", Some(message.clone()));
            return Err(message);
        }
    };

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


    #[test]
    fn slider_morph_types_are_authoritative_for_companion_matching() {
        assert!(is_slider_morph_type(TYPE_BGEO));
        assert!(is_slider_morph_type(TYPE_FACE));
        assert!(is_slider_morph_type(TYPE_FBLN));
        assert!(!is_slider_morph_type(TYPE_STBL_LOCAL));
    }

    #[test]
    fn stbl_key_parser_reads_slider_label_hashes() {
        let key = 0x745F_376D_11D5_E6C3u64;
        let text = "Butt".encode_utf16().collect::<Vec<_>>();
        let mut data = Vec::new();
        data.extend_from_slice(b"STBL");
        data.extend_from_slice(&[2, 0, 0]);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&[0; 6]);
        data.extend_from_slice(&key.to_le_bytes());
        data.extend_from_slice(&(text.len() as u32).to_le_bytes());
        for unit in text {
            data.extend_from_slice(&unit.to_le_bytes());
        }

        assert_eq!(stbl_keys(&data), vec![key]);
    }

    #[test]
    fn nmap_parser_reads_internal_slider_names() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&0x8D243219BAF14119u64.to_le_bytes());
        let name = b"Bloom_LegLenght_slider";
        data.extend_from_slice(&(name.len() as u32).to_le_bytes());
        data.extend_from_slice(name);

        assert_eq!(nmap_names(&data), vec!["Bloom_LegLenght_slider"]);
    }

    #[test]
    fn slider_region_uses_internal_anatomy_not_cas_panel() {
        assert_eq!(
            slider_destination_from_internal_name("Bloom_ArmTwist_slider", AppLanguage::Pt),
            Some(vec![
                "CAS".to_string(),
                "Sliders".to_string(),
                "Corpo".to_string(),
                "Braços".to_string(),
            ])
        );
        assert_eq!(
            slider_destination_from_internal_name("Bloom_LegLenght_slider", AppLanguage::Pt),
            Some(vec![
                "CAS".to_string(),
                "Sliders".to_string(),
                "Corpo".to_string(),
                "Pernas".to_string(),
            ])
        );
    }

    #[test]
    fn specific_anatomy_wins_over_generic_height_words() {
        assert_eq!(
            slider_destination_from_internal_name("Nose Tip Height", AppLanguage::En),
            Some(vec![
                "CAS".to_string(),
                "Sliders".to_string(),
                "Face".to_string(),
                "Nose".to_string(),
            ])
        );
        assert_eq!(
            slider_destination_from_internal_name("Shoulder Height", AppLanguage::En),
            Some(vec![
                "CAS".to_string(),
                "Sliders".to_string(),
                "Body".to_string(),
                "Shoulders".to_string(),
            ])
        );
    }

    #[test]
    fn ambiguous_internal_name_stays_at_slider_root() {
        assert!(slider_destination_from_internal_name("Tip Width", AppLanguage::En).is_none());
        assert!(slider_destination_from_internal_name("Outer Curve", AppLanguage::En).is_none());
    }

    #[test]
    fn pupil_heart_is_eye_not_ear() {
        assert_eq!(
            slider_region_keys("Pupil Heart"),
            Some(("face", Some("eyes")))
        );
    }
}
