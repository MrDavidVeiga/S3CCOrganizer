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
    sync::{Mutex, OnceLock},
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
    pub instances: Vec<String>,
    pub scripted: bool,
    pub content_source: String,
    pub source_confidence: Option<String>,
    pub status: String,
    pub classification_confidence: String,
    pub creator: Option<String>,
    pub mod_name: Option<String>,
    pub gameplay_category: Option<String>,
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

#[derive(Debug, Clone)]
struct LatestScan {
    root: PathBuf,
    language: AppLanguage,
    result: ScanResult,
}

static LATEST_SCAN: OnceLock<Mutex<Option<LatestScan>>> = OnceLock::new();

fn latest_scan_slot() -> &'static Mutex<Option<LatestScan>> {
    LATEST_SCAN.get_or_init(|| Mutex::new(None))
}

fn remember_latest_scan(result: &ScanResult, language: AppLanguage) {
    let root = PathBuf::from(&result.root)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&result.root));
    if let Ok(mut slot) = latest_scan_slot().lock() {
        *slot = Some(LatestScan {
            root,
            language,
            result: result.clone(),
        });
    }
}

pub fn cached_scan_for(root: &Path, language: AppLanguage) -> Option<ScanResult> {
    let canonical = root.canonicalize().ok()?;
    let slot = latest_scan_slot().lock().ok()?;
    let cached = slot.as_ref()?;
    (cached.language == language && cached.root == canonical).then(|| cached.result.clone())
}

pub fn cached_scan_paths(root: &Path) -> Option<Vec<PathBuf>> {
    let canonical = root.canonicalize().ok()?;
    let slot = latest_scan_slot().lock().ok()?;
    let cached = slot.as_ref()?;
    if cached.root != canonical {
        return None;
    }

    Some(
        cached
            .result
            .items
            .iter()
            .map(|item| PathBuf::from(&item.path))
            .collect(),
    )
}

fn package_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("package"))
        .unwrap_or(false)
}

fn contains_ascii_case_insensitive(data: &[u8], needle: &str) -> bool {
    let needle = needle.as_bytes();
    if needle.is_empty() || data.len() < needle.len() {
        return false;
    }

    data.windows(needle.len()).any(|window| {
        window
            .iter()
            .zip(needle.iter())
            .all(|(left, right)| left.to_ascii_lowercase() == right.to_ascii_lowercase())
    })
}

fn store_name_hint(value: &str) -> bool {
    let normalized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { ' ' })
        .collect::<String>();
    normalized.split_whitespace().any(|token| token == "store")
}

fn detect_store_source(package: &Package, name: &str, relative: &str, has_s3sa: bool) -> (String, Option<String>) {
    let mut strong_internal_evidence = false;

    for entry in &package.entries {
        if !matches!(entry.type_id, TYPE_S3SA | TYPE_STBL_LOCAL | TYPE_MANIFEST_LOCAL | TYPE_NMAP_LOCAL) {
            continue;
        }
        let Ok(data) = package.data(entry) else {
            continue;
        };

        if contains_ascii_case_insensitive(&data, "sims3.store")
            || contains_ascii_case_insensitive(&data, "the sims 3 store")
            || contains_ascii_case_insensitive(&data, "sims3store")
        {
            strong_internal_evidence = true;
            break;
        }
    }

    if strong_internal_evidence {
        return ("the_sims_3_store".to_string(), Some("strong".to_string()));
    }

    if has_s3sa && (store_name_hint(name) || store_name_hint(relative)) {
        return ("the_sims_3_store".to_string(), Some("probable".to_string()));
    }

    ("custom_content".to_string(), None)
}


const TYPE_NMAP_LOCAL: u32 = 0x0166_038C;
const TYPE_STBL_LOCAL: u32 = 0x2205_57DA;
const TYPE_MANIFEST_LOCAL: u32 = 0x73E9_3EEB;
const TYPE_XML_LOCAL: u32 = 0x0333_406C;
const TYPE_ITUN_LOCAL: u32 = 0x03B3_3DDF;
const TYPE_CLIP_LOCAL: u32 = 0x6B20_C4F3;

fn localized_special_folder(language: AppLanguage, key: &str) -> &'static str {
    match (language, key) {
        (_, "nraas") => "NRaas Mods",
        (AppLanguage::En, "poses") => "Poses and Animations",
        (AppLanguage::Pt, "poses") => "Poses e Animações",
        (AppLanguage::Es, "poses") => "Poses y Animaciones",
        (AppLanguage::En, "gameplay") => "Gameplay",
        (AppLanguage::Pt, "gameplay") => "Jogabilidade",
        (AppLanguage::Es, "gameplay") => "Jugabilidad",
        (AppLanguage::En, "scripts") => "Scripts",
        (AppLanguage::Pt, "scripts") => "Scripts",
        (AppLanguage::Es, "scripts") => "Scripts",
        (AppLanguage::En, "cooking_food") => "Cooking & Food",
        (AppLanguage::Pt, "cooking_food") => "Culinária e Comida",
        (AppLanguage::Es, "cooking_food") => "Cocina y Comida",
        (AppLanguage::En, "services") => "Services",
        (AppLanguage::Pt, "services") => "Serviços",
        (AppLanguage::Es, "services") => "Servicios",
        (AppLanguage::En, "careers") => "Careers",
        (AppLanguage::Pt, "careers") => "Carreiras",
        (AppLanguage::Es, "careers") => "Carreras",
        (AppLanguage::En, "relationships") => "Relationships",
        (AppLanguage::Pt, "relationships") => "Relacionamentos",
        (AppLanguage::Es, "relationships") => "Relaciones",
        (AppLanguage::En, "story_progression") => "Story Progression",
        (AppLanguage::Pt, "story_progression") => "Progressão da História",
        (AppLanguage::Es, "story_progression") => "Progresión de la Historia",
        (AppLanguage::En, "ui") => "UI",
        (AppLanguage::Pt, "ui") => "Interface",
        (AppLanguage::Es, "ui") => "Interfaz",
        (AppLanguage::En, "utilities") => "Utilities",
        (AppLanguage::Pt, "utilities") => "Utilitários",
        (AppLanguage::Es, "utilities") => "Utilidades",
        _ => "Unknown",
    }
}

fn internal_signature(
    package: &Package,
    resource_types: &[u32],
    needles: &[&str],
) -> Option<String> {
    for entry in &package.entries {
        if !resource_types.contains(&entry.type_id) {
            continue;
        }
        let Ok(data) = package.data(entry) else {
            continue;
        };
        for needle in needles {
            if contains_ascii_case_insensitive(&data, needle) {
                return Some(format!("{} contains '{}'", resource_type_label(entry.type_id), needle));
            }
        }
    }
    None
}

fn strip_leading_status_tags(value: &str) -> String {
    let mut text = value.trim().to_string();
    loop {
        let trimmed = text.trim_start();
        if !trimmed.starts_with('[') {
            return trimmed.to_string();
        }
        let Some(end) = trimmed.find(']') else {
            return trimmed.to_string();
        };
        text = trimmed[end + 1..].trim_start().to_string();
    }
}

fn creator_candidate_from_filename(name: &str) -> Option<String> {
    let stem = Path::new(name)
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let stem = strip_leading_status_tags(&stem);
    // A standalone mod name (e.g. AnimatedWoohoo.package) is not its author.
    // Only a token delimited from a separate title can be an author prefix.
    // Natural-language filenames ("Al Fresco Street Market", "Retro Workout")
    // do not contain an author prefix just because their title has spaces.
    if !stem.chars().any(|ch| ch == '_' || ch == '-' || ch == '.') {
        return None;
    }
    let candidate = stem
        .split(|ch: char| ch == '_' || ch == '-' || ch == '.')
        .next()?
        .trim();
    if !(2..=40).contains(&candidate.len())
        || !candidate.chars().any(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    let lower = candidate.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "al" | "mod" | "mods" | "script" | "scripts" | "package" | "update" | "updated"
            | "new" | "fix" | "override" | "ts3" | "sims3" | "the"
    ) {
        return None;
    }
    Some(candidate.to_string())
}

fn pretty_creator_label(candidate: &str) -> String {
    match candidate.to_ascii_lowercase().as_str() {
        "twinsimming" => "TwinSimming".to_string(),
        "nraas" => "NRaas".to_string(),
        _ => {
            let mut chars = candidate.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase().collect::<String>(), chars.as_str()),
                None => candidate.to_string(),
            }
        }
    }
}

fn verified_script_creator(package: &Package, name: &str) -> Option<String> {
    let candidate = creator_candidate_from_filename(name)?;
    internal_signature(
        package,
        &[
            TYPE_S3SA,
            TYPE_NMAP_LOCAL,
            TYPE_XML_LOCAL,
            TYPE_ITUN_LOCAL,
            TYPE_STBL_LOCAL,
            TYPE_MANIFEST_LOCAL,
        ],
        &[candidate.as_str()],
    )?;
    Some(pretty_creator_label(&candidate))
}

// A standalone assembly name is not a creator. Match the complete package
// stem against embedded identifiers to recover primary gameplay assemblies
// that also include unrelated CASP/OBJD resources.
fn verified_standalone_script_identity(package: &Package, name: &str) -> bool {
    if creator_candidate_from_filename(name).is_some() {
        return false;
    }
    let Some(stem) = Path::new(name).file_stem() else { return false };
    let stem = strip_leading_status_tags(&stem.to_string_lossy());
    let stem = stem.trim();
    if !(8..=80).contains(&stem.len()) || !stem.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    internal_signature(
        package,
        &[TYPE_S3SA, TYPE_NMAP_LOCAL, TYPE_XML_LOCAL, TYPE_ITUN_LOCAL, TYPE_STBL_LOCAL, TYPE_MANIFEST_LOCAL],
        &[stem],
    ).is_some()
}

fn generic_container_folder(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "#+18" | "18+" | "18" | "mods" | "packages" | "downloads" | "download" | "gameplay" | "jogabilidade"
            | "jugabilidad" | "scripts" | "objects" | "objetos" | "buy" | "compra"
            | "build" | "construção" | "construccion" | "cakes" | "cookies" | "breads"
            | "pies" | "pastries" | "cupcakes" | "ingredients" | "savory"
            | "gourmet desserts" | "other baked goods" | "wedding cakes"
            | "birthday cakes"
    )
}

fn mod_name_from_relative(relative: &str) -> Option<String> {
    let path = Path::new(relative);
    let mut components = path
        .parent()?
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().trim().to_string()),
            _ => None,
        });

    let first = components.next()?;
    if first.is_empty() || generic_container_folder(&first) {
        return None;
    }
    Some(first)
}

fn mod_name_from_filename(name: &str) -> Option<String> {
    let stem = Path::new(name)
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let stem = strip_leading_status_tags(&stem);
    let without_creator = stem
        .split_once('_')
        .map(|(_, rest)| rest.trim())
        .unwrap_or(stem.trim());
    let lower = without_creator.to_ascii_lowercase();
    let mod_end = lower.find(" mod").map(|index| index + 4).or_else(|| {
        lower.starts_with("mod ").then_some(3)
    })?;
    let value = without_creator[..mod_end].trim();
    if value.len() < 3 || generic_container_folder(value) {
        None
    } else {
        Some(value.to_string())
    }
}

fn inferred_script_mod_name(name: &str, relative: &str) -> Option<String> {
    mod_name_from_relative(relative).or_else(|| mod_name_from_filename(name))
}

// Match the declared subject of a mod, not arbitrary text inside an S3SA
// assembly or its tuning data. Generic game frameworks reference cooking,
// careers, dialogs, etc. without primarily changing those features.
fn script_category(name: &str, language: AppLanguage) -> &'static str {
    let filename = name.to_ascii_lowercase();
    let named = |words: &[&str]| words.iter().any(|word| filename.contains(word));

    if named(&["relationship", "romance", "romantic", "woohoo", "dating", "conversation", "bettergreet"]) {
        return localized_special_folder(language, "relationships");
    }
    if named(&["storyprogression", "story progression"]) {
        return localized_special_folder(language, "story_progression");
    }
    if named(&["baking", "recipe", "food", "cooking", "cake", "pastry", "restaurant", "bistro", "kitchen", "pasteurize", "milk mod"]) {
        return localized_special_folder(language, "cooking_food");
    }
    if named(&["career", "profession"]) {
        return localized_special_folder(language, "careers");
    }
    if named(&["housekeeper", "gardener_service", "maid_service", "cleaningservice", "service_npc"]) {
        return localized_special_folder(language, "services");
    }
    if named(&["userinterface", "user interface", "ui mod", "hud", "dialog", "loading screen"]) {
        return localized_special_folder(language, "ui");
    }
    if named(&["utility", "utilities", "framework", "loader", "coremod", "core_mod", "smoothpatch", "monopatcher"]) {
        return localized_special_folder(language, "utilities");
    }
    localized_special_folder(language, "scripts")
}

// A script assembly in an object does not by itself make a standalone gameplay
// mod. Conversely, actual gameplay mods may bundle CASP/OBJD resources. A
// verified internal author or an explicitly scripted source tree is independent
// evidence that the assembly is primary rather than embedded object behavior.
fn script_source_roles(relative: &str) -> (bool, bool, bool) {
    let mut gameplay = false;
    let mut scripts = false;
    let mut catalog_root = false;
    let mut first = true;
    for part in relative.split(['\\', '/']) {
        let normalized = part.trim().to_ascii_lowercase();
        if normalized.ends_with(".package") {
            break;
        }
        if first {
            catalog_root = matches!(normalized.as_str(), "buy" | "build" | "cas" | "objects");
            first = false;
        }
        if normalized == "gameplay" {
            gameplay = true;
        }
        if matches!(normalized.as_str(), "scripts" | "#8 scripts") {
            scripts = true;
        }
    }
    (gameplay, scripts, catalog_root)
}

// Choose catalog classification for a functional object with an embedded S3SA,
// but preserve independent gameplay mods even when they package supporting
// CASP/OBJD records. Store sets are bundles, not individual gameplay mods.
fn catalog_precedes_embedded_script(
    special: Option<&PackageFamilyClassification>,
    catalog_resource_count: usize,
    relative: &str,
    catalog_destinations: usize,
    catalog_ambiguous: bool,
) -> bool {
    if catalog_resource_count == 0 {
        return false;
    }
    let Some(classification) = special else { return false };
    if !classification.detected_from.iter().any(|source| source == "S3SA")
        || classification.detected_from.iter().any(|source| source == "NRaasInternal")
    {
        return false;
    }
    if store_name_hint(relative) {
        return true;
    }

    let (gameplay_folder, scripts_folder, catalog_root) = script_source_roles(relative);
    if gameplay_folder && !catalog_root {
        return false;
    }
    if catalog_root {
        return true;
    }
    // A single unambiguous OBJD/CASP destination is authoritative even when
    // an author name appears inside an object's script metadata.
    if catalog_destinations == 1 && !catalog_ambiguous {
        return true;
    }
    if scripts_folder {
        return false;
    }
    if classification.detected_from.iter().any(|source| {
        source == "InternalCreator" || source == "InternalScriptIdentity"
    }) && (catalog_destinations != 1 || catalog_ambiguous)
    {
        return false;
    }
    true
}

fn special_package_classification(
    package: &Package,
    type_ids: &BTreeSet<u32>,
    name: &str,
    relative: &str,
    language: AppLanguage,
    catalog_resource_count: usize,
) -> Option<PackageFamilyClassification> {
    // NRaas is detected from package internals rather than filenames. NMAP is
    // present in the supplied NRaas corpus, including tuning-only modules.
    if let Some(evidence) = internal_signature(
        package,
        &[TYPE_NMAP_LOCAL, TYPE_XML_LOCAL, TYPE_ITUN_LOCAL, TYPE_STBL_LOCAL, TYPE_MANIFEST_LOCAL],
        &["nraas"],
    ) {
        let folder = localized_special_folder(language, "nraas").to_string();
        return Some(PackageFamilyClassification {
            main_category: folder.clone(),
            sub_category: None,
            folder_parts: vec![folder.clone()],
            detected_from: vec!["NRaasInternal".to_string()],
            technical_reason: format!("Internal NRaas signature ({evidence}) => {folder}"),
        });
    }

    // Script packages are gameplay content even when the same package also
    // carries OBJD/CASP resources. A creator is used only when the filename
    // prefix is corroborated by the package's own internal resources.
    if type_ids.contains(&TYPE_S3SA) {
        let gameplay = localized_special_folder(language, "gameplay").to_string();
        let category = script_category(name, language).to_string();
        let creator = verified_script_creator(package, name);
        let mod_name = inferred_script_mod_name(name, relative);
        let mut folder_parts = vec![gameplay.clone()];
        let mut detected_from = vec!["S3SA".to_string()];
        if catalog_resource_count > 0
            && type_ids.contains(&TYPE_CASP)
            && type_ids.contains(&TYPE_OBJD)
            && !store_name_hint(relative)
            && verified_standalone_script_identity(package, name)
        {
            detected_from.push("InternalScriptIdentity".to_string());
        }

        if let Some(creator) = creator {
            folder_parts.push(creator);
            detected_from.push("InternalCreator".to_string());
        }
        if let Some(mod_name) = mod_name {
            folder_parts.push(mod_name);
            detected_from.push("ModName".to_string());
        } else {
            // If the mod name cannot be established safely, category remains a
            // useful fallback. It is always retained as metadata either way.
            folder_parts.push(category.clone());
        }

        return Some(PackageFamilyClassification {
            main_category: gameplay,
            sub_category: Some(category.clone()),
            folder_parts: folder_parts.clone(),
            detected_from,
            technical_reason: format!(
                "S3SA gameplay package; category metadata '{}'; physical destination => {}",
                category,
                folder_parts.join("\\")
            ),
        });
    }

    // A CLIP-only/content package is an animation/pose asset. Do not let CLIP
    // override authored gameplay scripts or CAS/OBJD catalog content.
    let clip_asset = type_ids.contains(&TYPE_CLIP_LOCAL)
        && !type_ids.contains(&TYPE_S3SA)
        && catalog_resource_count == 0;
    let pose_list = catalog_resource_count == 0
        && !type_ids.contains(&TYPE_S3SA)
        && internal_signature(
            package,
            &[TYPE_XML_LOCAL, TYPE_NMAP_LOCAL, TYPE_STBL_LOCAL, TYPE_MANIFEST_LOCAL],
            &["poselist", "pose list", "poseplayer", "pose player"],
        )
        .is_some();

    if clip_asset || pose_list {
        let folder = localized_special_folder(language, "poses").to_string();
        let mut detected = Vec::new();
        if clip_asset {
            detected.push("CLIP".to_string());
        }
        if pose_list {
            detected.push("PoseList".to_string());
        }
        return Some(PackageFamilyClassification {
            main_category: folder.clone(),
            sub_category: None,
            folder_parts: vec![folder.clone()],
            detected_from: detected.clone(),
            technical_reason: format!(
                "Pose/animation evidence [{}] => {}",
                detected.join(", "),
                folder
            ),
        });
    }

    None
}

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

// GEOM/VPXY clothing morph replacements often lack CASP. When their NMAP
// gives a stable `afTop...` or `afBottom...` mesh identifier, the internal
// resource itself confirms the category, gender and adult mesh family.
// Names of .package files are not used for this inference.
fn clothing_mesh_subtype(names: &[String]) -> Option<&'static str> {
    let mut kinds = BTreeSet::new();
    for name in names {
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("aftop") {
            kinds.insert("top");
        } else if lower.starts_with("afbottom") {
            kinds.insert("bottom");
        }
    }
    (kinds.len() == 1).then(|| *kinds.iter().next().unwrap())
}

fn geometry_clothing_from_nmap(
    package: &Package,
    types: &BTreeSet<u32>,
    language: AppLanguage,
) -> Option<PackageFamilyClassification> {
    if !types.contains(&0x015A_1849) || !types.contains(&0x7368_84F1)
        || types.contains(&TYPE_S3SA) || types.contains(&TYPE_CASP)
    {
        return None;
    }
    let names = package.entries.iter()
        .filter(|entry| entry.type_id == TYPE_NMAP_LOCAL)
        .filter_map(|entry| package.data(entry).ok())
        .flat_map(|data| nmap_names(&data))
        .collect::<Vec<_>>();
    let subtype = clothing_mesh_subtype(&names)?;
    let label = if subtype == "top" { language.top() } else { language.bottom() };
    let parts = vec![
        language.clothing().to_string(),
        language.female().to_string(),
        language.young_adult_adult().to_string(),
        label.to_string(),
    ];
    Some(PackageFamilyClassification {
        main_category: language.clothing().to_string(),
        sub_category: Some(label.to_string()),
        folder_parts: parts.clone(),
        detected_from: vec!["GEOM".to_string(), "VPXY".to_string(), "NMAP".to_string()],
        technical_reason: format!(
            "GEOM+VPXY female adult mesh with internal NMAP af{} => {}. No CASP; review before organizing.",
            if subtype == "top" { "Top" } else { "Bottom" },
            parts.join("\\"),
        ),
    })
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

    if token_starts_with_any(
        &name,
        &["penis", "testicle", "scrotum", "foreskin", "genital", "phallus", "erection"],
    ) {
        return Some(("body", Some("genitals")));
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
        (AppLanguage::En, "genitals") => "Genitals",
        (AppLanguage::Pt, "genitals") => "Genitais",
        (AppLanguage::Es, "genitals") => "Genitales",
        (AppLanguage::En, "other") => "Other",
        (AppLanguage::Pt, "other") => "Outros",
        (AppLanguage::Es, "other") => "Otros",
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

fn verified_slider_name_alias(filename: &str) -> Option<&'static str> {
    // Confirmed against the supplied slider corpus: OneEuroMutt's original
    // package filename omits the anatomical prefix. Only use this alias once
    // resources have independently established that the package is a slider.
    filename.eq_ignore_ascii_case("OneEuroMuttTip Width.package")
        .then_some("Nose Tip Width")
}

fn slider_internal_evidence(
    package: &Package,
    filename: &str,
    language: AppLanguage,
) -> Option<(String, &'static str, Option<Vec<String>>)> {
    let candidates = slider_internal_candidates(package);
    for (name, source) in &candidates {
        if let Some(destination) = slider_destination_from_internal_name(name, language) {
            return Some((name.clone(), *source, Some(destination)));
        }
    }
    // This specific alias has been verified by the user: the internal generic
    // label "Tip Width" is actually "Nose Tip Width". The enclosing scanner
    // has already confirmed morph resources, so the alias cannot classify
    // an unrelated package as a slider.
    if let Some(verified) = verified_slider_name_alias(filename) {
        return Some((
            verified.to_string(),
            "User-verified slider alias",
            slider_destination_from_internal_name(verified, language),
        ));
    }
    // Some sliders store only generic NMAP labels ("Tip Width", "Outer
    // Curve", "Middle Width"). Combine internal evidence with a clear
    // anatomical term in the package filename.
    if let Some((name, source)) = candidates.first() {
        if let Some(destination) = slider_destination_from_internal_name(filename, language) {
            return Some((format!("{name} | filename: {filename}"), *source, Some(destination)));
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
    slider_instances: &HashSet<u64>,
) {
    if slider_instances.is_empty() {
        return;
    }

    for (path, item) in package_paths.iter().zip(items.iter_mut()) {
        // Only STBL carriers can be companions; do not reopen hundreds of
        // direct morph packages during the second scanner pass.
        if !item.resource_types.iter().any(|label| label == "STBL") {
            continue;
        }
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
        item.destination_parts = vec!["Sliders".to_string()];
        item.destination_path = Some("Sliders".to_string());
        if !item
            .detected_from
            .iter()
            .any(|source| source == "STBL→Slider")
        {
            item.detected_from.push("STBL→Slider".to_string());
            item.detected_from.sort();
        }
        item.classification_reason = Some(format!(
            "STBL entry key 0x{key:016X} matches a morph resource instance in another package from the selected set => Sliders"
        ));
    }
}

fn top_level_relative_folder(relative: &str) -> Option<String> {
    let mut components = Path::new(relative).components().filter_map(|component| match component {
        std::path::Component::Normal(value) => Some(value.to_string_lossy().to_string()),
        _ => None,
    });
    let first = components.next()?;
    // A direct file at the selected root has no containing mod folder.
    components.next().map(|_| first)
}

// Only real, named mod folders may propagate a script's destination to
// companions. Standard resource roots and broad organizer categories group
// unrelated packages and must never become implicit mod identities.
fn named_mod_companion_folder(relative: &str) -> Option<String> {
    let folder = top_level_relative_folder(relative)?;
    let name = folder.trim().to_ascii_lowercase();
    let generic = matches!(
        name.as_str(),
        "packages" | "overrides" | "test" | "probation" | "dccache"
            | "mods" | "cc" | "custom content" | "downloads"
            | "build" | "buy" | "cas" | "gameplay" | "scripts"
            | "sliders" | "objects" | "poses" | "animations"
            | "hair" | "clothing" | "accessories" | "skins" | "textures"
            | "male" | "female" | "misc" | "miscellaneous"
            | "nraas" | "tuning" | "store" | "ui"
    );
    if generic || name.starts_with('#') || name.starts_with('!') || name.starts_with('+') {
        return None;
    }
    Some(folder)
}

fn may_reclassify_as_companion(item: &ScanPackageItem) -> bool {
    // Explicit CASP/OBJD classifications and independently identified
    // scripts must not be overwritten by a neighboring mod's destination.
    item.status != "invalid"
        && item.status != "mixed"
        && item.status != "needs_review"
        && !item.scripted
        && item.catalog_resource_count == 0
        && item.classification_confidence != "high"
        && !item.detected_from.iter().any(|source| value_is_authoritative_mod_source(source))
}

fn value_is_authoritative_mod_source(source: &str) -> bool {
    matches!(source, "NRaasInternal" | "ModName" | "CASP" | "OBJD")
}

fn apply_named_mod_companions(items: &mut [ScanPackageItem]) {
    let mut destinations = HashMap::<String, BTreeSet<String>>::new();

    for item in items.iter() {
        if !item.scripted || item.status != "classified" {
            continue;
        }
        if !item.detected_from.iter().any(|value| value == "ModName") {
            continue;
        }
        let Some(folder) = named_mod_companion_folder(&item.relative_path) else {
            continue;
        };
        destinations
            .entry(folder.to_ascii_lowercase())
            .or_default()
            .insert(item.destination_parts.join("\\"));
    }

    let unique = destinations
        .into_iter()
        .filter_map(|(folder, values)| {
            (values.len() == 1).then(|| (folder, values.into_iter().next().unwrap()))
        })
        .collect::<HashMap<_, _>>();

    for item in items.iter_mut() {
        if !may_reclassify_as_companion(item) {
            continue;
        }
        let Some(folder) = named_mod_companion_folder(&item.relative_path) else {
            continue;
        };
        let Some(destination) = unique.get(&folder.to_ascii_lowercase()) else {
            continue;
        };
        let parts = destination.split('\\').map(str::to_string).collect::<Vec<_>>();
        if parts.is_empty() || item.destination_parts == parts {
            continue;
        }

        item.status = "classified".to_string();
        item.classification_confidence = "medium".to_string();
        item.destination_parts = parts.clone();
        item.destination_path = Some(destination.clone());
        item.detected_from.push("ModFolderCompanion".to_string());
        item.detected_from.sort();
        item.detected_from.dedup();
        item.classification_reason = Some(format!(
            "{} | Companion package kept with the single named script mod detected in source folder '{}' => {}",
            item.classification_reason.clone().unwrap_or_default(),
            folder,
            destination
        ));
    }
}

fn apply_manual_classifications(root: &Path, items: &mut [ScanPackageItem]) {
    let workspace = load_workspace_for_root(root);
    if workspace.manual_classifications.is_empty() {
        return;
    }

    for item in items.iter_mut() {
        // Authoritative automatic classifications win. Manual review exists to
        // resolve Unknown/Mixed/Needs Review cases, so only those need hashing.
        if item.status == "invalid"
            || (item.status == "classified" && item.classification_confidence == "high")
        {
            continue;
        }

        let path = PathBuf::from(&item.path);
        let Ok((hash, _)) = sha256_file(&path) else {
            continue;
        };
        let Some(manual) = workspace.manual_classifications.get(&hash) else {
            continue;
        };
        let mut parts = split_destination(&manual.destination);
        if parts.first().is_some_and(|part| part.eq_ignore_ascii_case("CAS")) {
            parts.remove(0);
        }
        if parts.is_empty() {
            continue;
        }

        item.status = "classified".to_string();
        item.classification_confidence = "manual".to_string();
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
        TYPE_CLIP_LOCAL => "CLIP".to_string(),
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

fn classification_confidence(status: &str) -> &'static str {
    match status {
        "classified" => "high",
        "needs_review" | "mixed" => "medium",
        "unknown" | "invalid" => "low",
        _ => "low",
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

fn scan_one(
    root: &Path,
    path: &Path,
    language: AppLanguage,
    slider_instances: &mut HashSet<u64>,
) -> ScanPackageItem {
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
                instances: Vec::new(),
                scripted: false,
                content_source: "unknown".to_string(),
                source_confidence: None,
                status: "invalid".to_string(),
                classification_confidence: "low".to_string(),
                creator: None,
                mod_name: None,
                gameplay_category: None,
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

    let special_primary =
        special_package_classification(&package, &type_ids, &name, &relative, language, catalog_resource_count);
    if let Some(special) = &special_primary {
        for source in &special.detected_from {
            detected_from.insert(source.clone());
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

    let catalog_first = catalog_precedes_embedded_script(
        special_primary.as_ref(), catalog_resource_count, &relative, destinations.len(), has_ambiguous
    );
    let active_special = if catalog_first { None } else { special_primary.as_ref() };
    let (mut status, primary, mut destination_parts, mut destination_path) = if let Some(special) = active_special {
        (
            "classified".to_string(),
            None,
            special.folder_parts.clone(),
            Some(special.folder_parts.join("\\")),
        )
    } else if has_ambiguous {
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

    let mut classification_reason = active_special
        .map(|classification| classification.technical_reason.clone())
        .or_else(|| {
            primary
                .as_ref()
                .map(|classification| classification.technical_reason.clone())
        });
    let mut family_primary: Option<PackageFamilyClassification> = if catalog_first { None } else { special_primary };

    if status == "unknown" && catalog_resource_count == 0 {
        let family_result = geometry_clothing_from_nmap(&package, &type_ids, language)
            .map(PackageFamilyResult::Classified)
            .unwrap_or_else(|| classify_package_family(&type_ids, language));
        match family_result {
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
                    if slider_internal_candidates(&package).is_empty() {
                        // A morph-only package without an anatomical name is
                        // still a slider, but its body region is unknown.
                        destination_parts = vec![
                            "Sliders".to_string(),
                            slider_folder_label(language, "other").to_string(),
                        ];
                        destination_path = Some(destination_parts.join("\\"));
                    }
                    if let Some((internal_name, source, refined_destination)) =
                        slider_internal_evidence(&package, &name, language)
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
                            destination_parts = vec![
                                "Sliders".to_string(),
                                slider_folder_label(language, "other").to_string(),
                            ];
                            destination_path = Some(destination_parts.join("\\"));
                            classification_reason = Some(format!(
                                "{} | Internal slider name from {source} '{}' did not safely identify an anatomical region; grouped under Sliders/Other without inventing an anatomy.",
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

    // Collect morph IDs only when this package was positively classified
    // as a slider, never from clothing that happens to contain morphs.
    if status == "classified" && sub_category.as_deref() == Some("Sliders") {
        slider_instances.extend(
            package.entries.iter()
                .filter(|entry| is_slider_morph_type(entry.type_id))
                .map(|entry| entry.instance),
        );
    }

    let scripted = type_ids.contains(&TYPE_S3SA);
    let (content_source, source_confidence) =
        detect_store_source(&package, &name, &relative, scripted);

    let is_nraas = detected_from.contains("NRaasInternal");
    let creator = if is_nraas {
        Some("NRaas".to_string())
    } else if scripted {
        verified_script_creator(&package, &name)
    } else {
        None
    };
    // Embedded gameplay assemblies are still tracked by the 'scripted' flag,
    // but must not supply an unrelated mod/category label to a catalog object.
    let mod_name = if scripted && !is_nraas && !catalog_first {
        inferred_script_mod_name(&name, &relative)
    } else {
        None
    };
    let gameplay_category = if scripted && !is_nraas && !catalog_first {
        Some(script_category(&name, language).to_string())
    } else {
        None
    };
    let classification_confidence = classification_confidence(&status).to_string();

    ScanPackageItem {
        id,
        name,
        path: path.to_string_lossy().to_string(),
        relative_path: relative,
        file_size,
        resource_count: package.entries.len(),
        catalog_resource_count,
        resource_types: type_set.into_iter().collect(),
        instances: package.entries.iter()
            .map(|entry| format!("0x{:016X}", entry.instance))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        scripted,
        content_source,
        source_confidence,
        status,
        classification_confidence,
        creator,
        mod_name,
        gameplay_category,
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

// Previews must preserve the loading branch of the source: Packages stays in
// Packages, Overrides stays in Overrides. The semantic category is independent
// of which branch actually contains the package.
fn physical_destination_parts(
    root: &Path,
    source_relative: &str,
    parts: &[String],
) -> Vec<String> {
    let root_name = root.file_name().map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let source_top = source_relative.replace('\\', "/")
        .split('/').next().unwrap_or("").to_lowercase();
    let override_source = root_name == "overrides"
        || (root_name == "mods" && source_top == "overrides");

    let mut result = parts.to_vec();
    if result.first().is_some_and(|part| part.eq_ignore_ascii_case("Packages")
        || part.eq_ignore_ascii_case("Overrides"))
    {
        result.remove(0);
    }
    if result.first().is_some_and(|part| part.eq_ignore_ascii_case("CAS")) {
        result.remove(0);
    }
    if root_name == "mods" && !result.is_empty() {
        result.insert(0, if override_source {
            "Overrides".to_string()
        } else {
            "Packages".to_string()
        });
    }
    result
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
    let mut slider_instances = HashSet::<u64>::new();

    for (index, path) in package_paths.iter().enumerate() {
        if let Some(kind) = operation_kind {
            if operation::is_cancelled(kind) {
                return Err(CANCELLED_ERROR.to_string());
            }
        }

        items.push(scan_one(&root, path, language, &mut slider_instances));

        if let Some(kind) = operation_kind {
            let processed = index + 1;
            if processed == package_paths.len() || processed % 8 == 0 {
                operation::update(kind, processed, None, "scanning");
            }
        }
    }

    apply_slider_companion_classification(&package_paths, &mut items, &slider_instances);
    apply_named_mod_companions(&mut items);
    apply_manual_classifications(&root, &mut items);

    for item in &mut items {
        item.destination_parts = physical_destination_parts(&root, &item.relative_path, &item.destination_parts);
        if item.destination_path.is_some() {
            item.destination_path = Some(item.destination_parts.join("\\"));
        }
        item.candidate_destinations = item.candidate_destinations
            .iter()
            .map(|value| {
                physical_destination_parts(&root, &item.relative_path, &split_destination(value)).join("\\")
            })
            .collect();
    }

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
        Ok(scan) => {
            remember_latest_scan(scan, language);
            operation::finish(KIND, "complete", None);
        }
        Err(error) if error == CANCELLED_ERROR => operation::mark_cancelled(KIND),
        Err(error) => operation::finish(KIND, "error", Some(error.clone())),
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_categories_require_subject_evidence_not_random_assembly_strings() {
        let cases = [
            ("AnimatedWoohoo.package", "Relacionamentos"),
            ("TSS_MoreRomanticInteractions.package", "Relacionamentos"),
            ("Gamefreak130_KarmaPowers.package", "Scripts"),
            ("douglasveiga_HousekeeperService_v1.2.package", "Serviços"),
            ("douglasveiga_Gardener_service_NPC_v2.3.package", "Serviços"),
            ("Look! A living Sheep!.package", "Scripts"),
            ("zoeoe_knitting_resources.package", "Scripts"),
            ("simler90GameplayCoreMod-UPDATE199.package", "Utilitários"),
            ("twinsimming_Pasteurize Milk Mod.package", "Culinária e Comida"),
            ("TSS_KitchenTweaks_WithMikeyEdit.package", "Culinária e Comida"),
            ("NeoH4x0rGlobalOnlineBankingMod.package", "Scripts"),
            ("Gamefreak130_SmartphoneDating.package", "Relacionamentos"),
        ];
        for (name, expected) in cases {
            assert_eq!(script_category(name, AppLanguage::Pt), expected, "{name}");
        }
        assert_eq!(script_category("douglasveiga_HousekeeperService_v1.2.package", AppLanguage::En), "Services");
        assert_eq!(script_category("douglasveiga_HousekeeperService_v1.2.package", AppLanguage::Es), "Servicios");
        assert_eq!(creator_candidate_from_filename("AnimatedWoohoo.package"), None);
        assert_eq!(creator_candidate_from_filename("[Items] Al Fresco Street Market.package"), None);
        assert_eq!(creator_candidate_from_filename("Let's Take a Selfie by David Veiga.package"), None);
        assert_eq!(creator_candidate_from_filename("Retro Workout.package"), None);
        assert_eq!(creator_candidate_from_filename("ld_MonoPatcher.package"), Some("ld".to_string()));
        assert_eq!(creator_candidate_from_filename("icarusallsorts.EatOutsideRestaurant.package"), Some("icarusallsorts".to_string()));
        assert_eq!(creator_candidate_from_filename("twinsimming_Pasteurize Milk Mod.package"), Some("twinsimming".to_string()));
        assert_eq!(mod_name_from_relative(r"#+18\\AnimatedWoohoo.package"), None);
    }

    #[test]
    fn real_gameplay_mods_and_functional_objects_keep_distinct_destinations() {
        let script = PackageFamilyClassification {
            main_category: "Gameplay".into(),
            sub_category: Some("Scripts".into()),
            folder_parts: vec!["Gameplay".into(), "Scripts".into()],
            detected_from: vec!["S3SA".into()],
            technical_reason: "S3SA evidence".into(),
        };
        let catalog = |relative: &str, destinations, ambiguous| {
            catalog_precedes_embedded_script(Some(&script), 2, relative, destinations, ambiguous)
        };
        // Store bundles and premium content should never become Gameplay
        // just because an XML/assembly contains keyword matches.
        assert!(catalog("Packages/#9 Store/Store Content/[Items] Al Fresco.package", 4, false));
        assert!(catalog("Packages/#9 Store/Store Content/[PC] Basketball Hoop.package", 1, false));
        assert!(catalog("Buy/Entertainment/Sports/[PC] Basketball Hoop.package", 1, false));
        assert!(catalog("Buy/Debug/Buzz_ShellSoundEmitter.package", 1, false));
        assert!(catalog("Packages/ani_BistroStove.package", 1, false));
        assert!(catalog("Packages/Scripts/cmomoney_TimeShifter.package", 1, false));
        assert!(catalog("Packages/#8 Scripts/Objects/Fantuanss12_ProfessionalOven.package", 1, false));
        // Gameplay mods may bundle multiple object types or unreadable OBJD.
        assert!(!catalog("Packages/#8 Scripts/Gameplay/Services/HousekeeperService.package", 2, false));
        assert!(!catalog("Packages/#8 Scripts/Gameplay/Global Online Banking Mod/Bank.package", 1, false));
        assert!(!catalog("Packages/Scripts/Gamefreak130_KarmaPowers.package", 0, true));
        assert!(!catalog_precedes_embedded_script(None, 2, "Buy/Sports/Hoop.package", 1, false));
        assert!(!catalog_precedes_embedded_script(Some(&script), 0, "Buy/Sports/Hoop.package", 1, false));

        let verified = PackageFamilyClassification {
            detected_from: vec!["S3SA".into(), "InternalCreator".into()],
            ..script.clone()
        };
        assert!(!catalog_precedes_embedded_script(
            Some(&verified), 2, "#+18/AnimatedWoohoo.package", 2, false
        ));
        let standalone_identity = PackageFamilyClassification {
            detected_from: vec!["S3SA".into(), "InternalScriptIdentity".into()],
            ..script.clone()
        };
        assert!(!catalog_precedes_embedded_script(
            Some(&standalone_identity), 2, "#+18/AnimatedWoohoo.package", 2, false
        ));
        assert!(catalog_precedes_embedded_script(
            Some(&standalone_identity), 2, "Packages/#9 Store/Store Content/[Items] Al Fresco.package", 2, false
        ));
        assert!(catalog_precedes_embedded_script(
            Some(&script), 2, "#+18/UnknownResourceBundle.package", 2, false
        ));
        assert!(catalog_precedes_embedded_script(
            Some(&verified), 2, "Packages/#9 Store/Store Content/[Items] Le Cinema.package", 2, false
        ));
        assert!(catalog_precedes_embedded_script(
            Some(&verified), 2, "Packages/ani_BistroStove.package", 1, false
        ));
        assert!(catalog_precedes_embedded_script(
            Some(&verified), 2, "Buy/Decor/Fantuanss12_GroceryDeliveryService.package", 2, true
        ));
        let nraas = PackageFamilyClassification {
            detected_from: vec!["NRaasInternal".into()],
            ..script
        };
        assert!(!catalog_precedes_embedded_script(
            Some(&nraas), 2, "Packages/Scripts/NRaas.package", 1, false
        ));
    }

    #[test]
    fn generic_source_folders_cannot_reassign_unrelated_packages() {
        for source in [
            "Packages/Scripts/SomeGameplay.package",
            "Packages/Male Hair/SomeHair.package",
            "CAS/Sliders/Body/Morph.package",
            "Build/Windows/SomeWindow.package",
            "Buy/Decor/SomeObject.package",
            "#+18/Pns (TS3)/Rigged/Body.package",
            "NRaas/MasterController/Extension.package",
            "Overrides/UI/Tuning.package",
        ] {
            assert!(named_mod_companion_folder(source).is_none(), "{source}");
        }
    }

    #[test]
    fn specifically_named_mod_folder_can_group_companions() {
        assert_eq!(
            named_mod_companion_folder("Baking Mod/Cakes/Chocolate.package").as_deref(),
            Some("Baking Mod")
        );
        assert_eq!(
            named_mod_companion_folder("PrismHome/Assets/Strings.package").as_deref(),
            Some("PrismHome")
        );
    }

    #[test]
    fn independent_catalog_and_script_classifications_are_not_companions() {
        let mut item = ScanPackageItem {
            id: String::new(), name: String::new(), path: String::new(),
            relative_path: "Baking Mod/Cakes/Example.package".into(),
            file_size: 0, resource_count: 0, catalog_resource_count: 1,
            resource_types: Vec::new(), instances: Vec::new(),
            scripted: false, content_source: String::new(), source_confidence: None,
            status: "classified".into(), classification_confidence: "high".into(),
            creator: None, mod_name: None, gameplay_category: None,
            detected_from: Vec::new(), category: Some("Objects".into()),
            sub_category: None, gender: None, age: None, species: None,
            usage_categories: Vec::new(), destination_parts: vec!["Objects".into()],
            destination_path: Some("Objects".into()),
            candidate_destinations: Vec::new(), classifications: Vec::new(),
            classification_reason: None, warnings: Vec::new(),
        };
        assert!(!may_reclassify_as_companion(&item));
        item.catalog_resource_count = 0;
        item.scripted = true;
        assert!(!may_reclassify_as_companion(&item));
        item.scripted = false;
        item.classification_confidence = "low".into();
        item.status = "unknown".into();
        assert!(may_reclassify_as_companion(&item));
    }

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
    fn scan_previews_use_packages_without_cas_level() {
        let mods = Path::new("The Sims 3").join("Mods");
        let packages = mods.join("Packages");
        let legacy = vec!["CAS".to_string(), "Clothing".to_string(),
            "Male".to_string(), "YA-A".to_string(), "Top".to_string()];
        assert_eq!(
            physical_destination_parts(&mods, "Packages/legacy.package", &legacy),
            vec!["Packages", "Clothing", "Male", "YA-A", "Top"]
        );
        assert_eq!(
            physical_destination_parts(&packages, "legacy.package", &legacy),
            vec!["Clothing", "Male", "YA-A", "Top"]
        );
        assert_eq!(
            physical_destination_parts(&mods, "Packages/legacy.package", &["Packages".into(), "CAS".into(), "Sliders".into()]),
            vec!["Packages", "Sliders"]
        );
        assert_eq!(
            physical_destination_parts(&packages, "legacy.package", &["Packages".into(), "CAS".into(), "Sliders".into()]),
            vec!["Sliders"]
        );
    }

    #[test]
    fn overrides_remain_in_overrides_in_both_scan_modes() {
        let mods = Path::new("The Sims 3").join("Mods");
        let overrides = mods.join("Overrides");
        let parts = vec!["Gameplay".into(), "Tuning".into()];
        assert_eq!(
            physical_destination_parts(&mods, "Overrides/UI/foo.package", &parts),
            vec!["Overrides", "Gameplay", "Tuning"]
        );
        assert_eq!(
            physical_destination_parts(&mods, r"Overrides\\UI\\foo.package", &parts),
            vec!["Overrides", "Gameplay", "Tuning"]
        );
        assert_eq!(
            physical_destination_parts(&overrides, "UI/foo.package", &parts),
            parts
        );
        assert_eq!(
            physical_destination_parts(&mods, "Packages/foo.package", &parts),
            vec!["Packages", "Gameplay", "Tuning"]
        );
        assert_eq!(
            physical_destination_parts(
                &mods, "Overrides/foo.package",
                &["Packages".into(), "CAS".into(), "Sliders".into()]
            ),
            vec!["Overrides", "Sliders"]
        );
    }

    #[test]
    fn slider_region_uses_internal_anatomy_not_cas_panel() {
        assert_eq!(
            slider_destination_from_internal_name("Bloom_ArmTwist_slider", AppLanguage::Pt),
            Some(vec![
                "Sliders".to_string(),
                "Corpo".to_string(),
                "Braços".to_string(),
            ])
        );
        assert_eq!(
            slider_destination_from_internal_name("Bloom_LegLenght_slider", AppLanguage::Pt),
            Some(vec![
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
                "Sliders".to_string(),
                "Face".to_string(),
                "Nose".to_string(),
            ])
        );
        assert_eq!(
            slider_destination_from_internal_name("Shoulder Height", AppLanguage::En),
            Some(vec![
                "Sliders".to_string(),
                "Body".to_string(),
                "Shoulders".to_string(),
            ])
        );
    }

    #[test]
    fn pregnancy_clothing_nmap_has_internal_gender_and_subtype_evidence() {
        let names = vec![
            "afBottomNude_special_lod3".to_string(),
            "afBottomNude_special_lod2".to_string(),
        ];
        assert_eq!(clothing_mesh_subtype(&names), Some("bottom"));
        assert_eq!(
            clothing_mesh_subtype(&["afTopNude_special".to_string()]), Some("top")
        );
        assert_eq!(
            clothing_mesh_subtype(&["Tip Width".to_string()]), None
        );
        assert_eq!(
            clothing_mesh_subtype(&["afTopNude".to_string(), "afBottomJeans".to_string()]),
            None
        );
    }

    #[test]
    fn genital_sliders_use_anatomical_category_in_three_languages() {
        assert_eq!(
            slider_destination_from_internal_name("Penis Length", AppLanguage::En),
            Some(vec!["Sliders".into(), "Body".into(), "Genitals".into()])
        );
        assert_eq!(
            slider_destination_from_internal_name("Testicle Size", AppLanguage::Pt),
            Some(vec!["Sliders".into(), "Corpo".into(), "Genitais".into()])
        );
    }

    #[test]
    fn filename_can_refine_a_generic_internal_slider_name() {
        assert_eq!(
            slider_destination_from_internal_name("aWT_Mouth-UpperLip-TipWidth.package", AppLanguage::En),
            Some(vec!["Sliders".into(), "Face".into(), "Mouth & Lips".into()])
        );
        assert_eq!(
            slider_destination_from_internal_name("Lavender_MiddleFaceWidth.package", AppLanguage::En),
            Some(vec!["Sliders".into(), "Face".into()])
        );
        assert!(slider_destination_from_internal_name("OneEuroMuttTip Width.package", AppLanguage::En).is_none());
        assert_eq!(verified_slider_name_alias("OneEuroMuttTip Width.package"), Some("Nose Tip Width"));
        assert_eq!(
            slider_destination_from_internal_name(
                verified_slider_name_alias("OneEuroMuttTip Width.package").unwrap(),
                AppLanguage::En,
            ),
            Some(vec!["Sliders".into(), "Face".into(), "Nose".into()])
        );
        assert_eq!(verified_slider_name_alias("SomeoneElseTip Width.package"), None);
    }

    #[test]
    fn ambiguous_internal_name_stays_at_slider_root() {
        assert!(slider_destination_from_internal_name("Tip Width", AppLanguage::En).is_none());
        assert!(slider_destination_from_internal_name("Outer Curve", AppLanguage::En).is_none());
    }

    #[test]
    fn script_mod_name_prefers_named_source_folder() {
        assert_eq!(
            inferred_script_mod_name(
                "twinsimming_Chocolate Cake.package",
                "Baking Mod/Cakes/twinsimming_Chocolate Cake.package"
            )
            .as_deref(),
            Some("Baking Mod")
        );
    }

    #[test]
    fn script_mod_name_can_be_read_from_main_package_filename() {
        assert_eq!(
            mod_name_from_filename("twinsimming_Baking Mod V2.1.package").as_deref(),
            Some("Baking Mod")
        );
        assert_eq!(
            mod_name_from_filename("twinsimming_Baking Mod [Home Baker Career].package").as_deref(),
            Some("Baking Mod")
        );
    }

    #[test]
    fn pupil_heart_is_eye_not_ear() {
        assert_eq!(
            slider_region_keys("Pupil Heart"),
            Some(("face", Some("eyes")))
        );
    }
}
