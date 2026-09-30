use crate::{
    cache::{get_or_build_with_metrics, load_cache, retain_existing, save_cache},
    catalog::{TYPE_CASP, TYPE_OBJD},
    operation::{self, CANCELLED_ERROR},
    resource_cfg::{find_resource_cfg, package_priority, parse_resource_cfg, ResourceCfgInfo},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::WalkDir;

const TYPE_IMG: u32 = 0x00B2_D882;
const TYPE_GEOM: u32 = 0x015A_1849;
const TYPE_NMAP: u32 = 0x0166_038C;
const TYPE_MODL: u32 = 0x0166_1233;
const TYPE_MATD: u32 = 0x01D0_E75D;
const TYPE_MLOD: u32 = 0x01D1_0F34;
const TYPE_OBJK: u32 = 0x02DC_343F;
const TYPE_XML: u32 = 0x0333_406C;
const TYPE_TXTC: u32 = 0x033A_1435;
const TYPE_TXTF: u32 = 0x0341_ACC9;
const TYPE_ITUN: u32 = 0x03B3_3DDF;
const TYPE_STBL: u32 = 0x2205_57DA;
const TYPE_VPXY: u32 = 0x7368_84F1;
const TYPE_S3SA: u32 = 0x073F_AA07;
const TYPE_MANIFEST: u32 = 0x73E9_3EEB;

const TYPE_CATALOG_FENCE: u32 = 0x0418_FE2A;
const TYPE_CATALOG_STAIRS: u32 = 0x049C_A4CD;
const TYPE_CATALOG_PROXY: u32 = 0x04AC_5D93;
const TYPE_CATALOG_TERRAIN_GEOMETRY: u32 = 0x04B3_0669;
const TYPE_CATALOG_RAILING: u32 = 0x04C5_8103;
const TYPE_CATALOG_TERRAIN_PAINT: u32 = 0x04ED_4BB2;
const TYPE_CATALOG_FIREPLACE: u32 = 0x04F3_CC01;
const TYPE_CATALOG_WATER: u32 = 0x060B_390C;
const TYPE_CATALOG_POOL: u32 = 0x0A36_F07A;
const TYPE_CATALOG_FOUNDATION: u32 = 0x316C_78F2;

const MAX_FINDINGS: usize = 20_000;
const MAX_EVIDENCE_PER_FINDING: usize = 64;

#[derive(Debug, Clone)]
struct ResourceOccurrence {
    package_index: usize,
    type_id: u32,
    group: u32,
    instance: u64,
    payload_hash: String,
    payload_size: usize,
}

#[derive(Debug, Clone)]
struct PackageInfo {
    name: String,
    path: PathBuf,
    relative_path: PathBuf,
    readable: bool,
    load_priority: Option<i32>,
    load_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictPackage {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub load_priority: Option<i32>,
    pub load_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictEvidence {
    pub resource_type: u32,
    pub resource_type_hex: String,
    pub resource_label: String,
    pub resource_class: String,
    pub group: u32,
    pub group_hex: String,
    pub instance: u64,
    pub instance_hex: String,
    pub same_payload: bool,
    pub left_payload_sha256: String,
    pub right_payload_sha256: String,
    pub left_payload_size: usize,
    pub right_payload_size: usize,
    pub impact_kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFinding {
    pub id: String,
    pub kind: String,
    pub impact_kinds: Vec<String>,
    pub severity: String,
    pub left: ConflictPackage,
    pub right: ConflictPackage,
    pub shared_resource_count: usize,
    pub identical_payload_count: usize,
    pub different_payload_count: usize,
    pub evidence: Vec<ConflictEvidence>,
    pub evidence_truncated: bool,
    pub explanation_key: String,
    pub load_order_status: String,
    pub higher_priority_path: Option<String>,
    pub load_order_explanation_key: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConflictStats {
    pub packages_scanned: usize,
    pub readable_packages: usize,
    pub unreadable_packages: usize,
    pub package_pairs: usize,
    pub shared_identical: usize,
    pub visual_overrides: usize,
    pub catalog_overrides: usize,
    pub gameplay_overrides: usize,
    pub script_conflicts: usize,
    pub text_overrides: usize,
    pub potential_conflicts: usize,
    pub mixed_overrides: usize,
    pub analysis_truncated: bool,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub hashing_ms: u128,
    pub dbpf_load_ms: u128,
    pub resource_decode_ms: u128,
    pub comparison_ms: u128,
    pub total_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictAnalysis {
    pub root: String,
    pub resource_cfg: Option<ResourceCfgInfo>,
    pub findings: Vec<ConflictFinding>,
    pub stats: ConflictStats,
    pub errors: Vec<String>,
}

#[derive(Debug, Default)]
struct PairAccumulator {
    shared_resource_count: usize,
    identical_payload_count: usize,
    different_payload_count: usize,
    impact_kinds: BTreeSet<String>,
    evidence: Vec<ConflictEvidence>,
    evidence_truncated: bool,
}

fn is_package(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("package"))
        .unwrap_or(false)
}

fn resource_label(type_id: u32) -> String {
    match type_id {
        TYPE_IMG => "_IMG".into(),
        TYPE_GEOM => "GEOM".into(),
        TYPE_NMAP => "NMAP".into(),
        TYPE_MODL => "MODL".into(),
        TYPE_MATD => "MATD".into(),
        TYPE_MLOD => "MLOD".into(),
        TYPE_OBJK => "OBJK".into(),
        TYPE_XML => "XML".into(),
        TYPE_TXTC => "TXTC".into(),
        TYPE_TXTF => "TXTF".into(),
        TYPE_ITUN => "ITUN".into(),
        TYPE_CASP => "CASP".into(),
        TYPE_OBJD => "OBJD".into(),
        TYPE_STBL => "STBL".into(),
        TYPE_VPXY => "VPXY".into(),
        TYPE_S3SA => "S3SA".into(),
        TYPE_MANIFEST => "Manifest".into(),
        other => format!("0x{other:08X}"),
    }
}

fn is_visual(type_id: u32) -> bool {
    matches!(
        type_id,
        TYPE_IMG | TYPE_GEOM | TYPE_MODL | TYPE_MLOD | TYPE_VPXY | TYPE_MATD | TYPE_TXTC | TYPE_TXTF | TYPE_OBJK
    )
}

fn is_catalog(type_id: u32) -> bool {
    matches!(
        type_id,
        TYPE_CASP
            | TYPE_OBJD
            | TYPE_CATALOG_FENCE
            | TYPE_CATALOG_STAIRS
            | TYPE_CATALOG_PROXY
            | TYPE_CATALOG_TERRAIN_GEOMETRY
            | TYPE_CATALOG_RAILING
            | TYPE_CATALOG_TERRAIN_PAINT
            | TYPE_CATALOG_FIREPLACE
            | TYPE_CATALOG_WATER
            | TYPE_CATALOG_POOL
            | TYPE_CATALOG_FOUNDATION
    )
}

fn is_gameplay(type_id: u32) -> bool {
    matches!(type_id, TYPE_XML | TYPE_ITUN)
}

fn resource_class(type_id: u32) -> &'static str {
    if type_id == TYPE_S3SA {
        "script"
    } else if is_gameplay(type_id) {
        "gameplay"
    } else if is_catalog(type_id) {
        "catalog"
    } else if is_visual(type_id) {
        "visual"
    } else if type_id == TYPE_STBL {
        "text"
    } else if matches!(type_id, TYPE_NMAP | TYPE_MANIFEST) {
        "metadata"
    } else {
        "unknown"
    }
}

fn impact_for(type_id: u32, same_payload: bool) -> &'static str {
    if same_payload {
        return "shared_identical";
    }

    match resource_class(type_id) {
        "script" => "script_conflict",
        "gameplay" => "gameplay_override",
        "catalog" => "catalog_override",
        "visual" => "visual_override",
        "text" => "text_override",
        // Metadata and unknown payload differences are not promoted to a
        // specific conflict class without deeper semantics.
        _ => "potential_conflict",
    }
}

fn impact_priority(kind: &str) -> u8 {
    match kind {
        "script_conflict" => 70,
        "gameplay_override" => 60,
        "catalog_override" => 50,
        "visual_override" => 40,
        "text_override" => 30,
        "potential_conflict" => 20,
        "shared_identical" => 0,
        _ => 10,
    }
}

fn primary_kind(kinds: &BTreeSet<String>, different_payload_count: usize) -> String {
    if different_payload_count == 0 {
        return "shared_identical".to_string();
    }

    let non_shared = kinds
        .iter()
        .filter(|kind| kind.as_str() != "shared_identical")
        .collect::<Vec<_>>();

    if non_shared.len() > 1 {
        return "mixed_override".to_string();
    }

    non_shared
        .into_iter()
        .max_by_key(|kind| impact_priority(kind))
        .map(|kind| kind.to_string())
        .unwrap_or_else(|| "potential_conflict".to_string())
}

fn severity_for(kind: &str) -> &'static str {
    match kind {
        "script_conflict" => "high",
        "gameplay_override" => "high",
        "mixed_override" => "high",
        "catalog_override" => "warning",
        "visual_override" => "warning",
        "text_override" => "review",
        "potential_conflict" => "review",
        "shared_identical" => "info",
        _ => "review",
    }
}

fn explanation_key(kind: &str) -> &'static str {
    match kind {
        "shared_identical" => "same_tgi_same_payload",
        "visual_override" => "same_tgi_different_visual_payload",
        "catalog_override" => "same_tgi_different_catalog_payload",
        "gameplay_override" => "same_tgi_different_gameplay_payload",
        "script_conflict" => "same_tgi_different_script_payload",
        "text_override" => "same_tgi_different_text_payload",
        "mixed_override" => "multiple_override_classes",
        _ => "same_tgi_different_unknown_payload",
    }
}

fn package_ref(package: &PackageInfo) -> ConflictPackage {
    ConflictPackage {
        name: package.name.clone(),
        path: package.path.to_string_lossy().to_string(),
        relative_path: package.relative_path.to_string_lossy().to_string(),
        load_priority: package.load_priority,
        load_rule: package.load_rule.clone(),
    }
}

fn load_order_info(
    left: &PackageInfo,
    right: &PackageInfo,
    resource_cfg_present: bool,
    precedence_reliable: bool,
) -> (String, Option<String>, String) {
    if resource_cfg_present && !precedence_reliable {
        return (
            "advanced_cfg_unresolved".to_string(),
            None,
            "resource_cfg_advanced_unresolved".to_string(),
        );
    }

    match (left.load_priority, right.load_priority) {
        (Some(left_priority), Some(right_priority)) if left_priority > right_priority => (
            "resolved_by_priority".to_string(),
            Some(left.relative_path.to_string_lossy().to_string()),
            "higher_resource_cfg_priority".to_string(),
        ),
        (Some(left_priority), Some(right_priority)) if right_priority > left_priority => (
            "resolved_by_priority".to_string(),
            Some(right.relative_path.to_string_lossy().to_string()),
            "higher_resource_cfg_priority".to_string(),
        ),
        (Some(_), Some(_)) => (
            "same_priority".to_string(),
            None,
            "same_resource_cfg_priority".to_string(),
        ),
        (Some(_), None) | (None, Some(_)) => (
            "partially_matched".to_string(),
            None,
            "resource_cfg_only_one_match".to_string(),
        ),
        (None, None) if resource_cfg_present => (
            "unmatched".to_string(),
            None,
            "resource_cfg_no_matching_rule".to_string(),
        ),
        (None, None) => (
            "resource_cfg_missing".to_string(),
            None,
            "resource_cfg_missing".to_string(),
        ),
    }
}

fn pair_id(left: &PackageInfo, right: &PackageInfo) -> String {
    let mut paths = [
        left.relative_path.to_string_lossy().to_string(),
        right.relative_path.to_string_lossy().to_string(),
    ];
    paths.sort();
    let digest = format!("{:X}", Sha256::digest(format!("{}|{}", paths[0], paths[1]).as_bytes()));
    format!("conflict:{}", &digest[..16])
}

pub fn analyze_conflicts_core(
    folder: String,
    operation_kind: Option<&str>,
) -> Result<ConflictAnalysis, String> {
    let total_started = Instant::now();
    if folder.trim().is_empty() {
        return Err("No folder was selected.".to_string());
    }

    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve root folder: {error}"))?;

    if !root.is_dir() {
        return Err(format!("Folder does not exist: {}", root.display()));
    }

    let resource_cfg = find_resource_cfg(&root)
        .map(|path| parse_resource_cfg(&path))
        .transpose()?;
    let resource_cfg_directory = resource_cfg
        .as_ref()
        .and_then(|cfg| PathBuf::from(&cfg.path).parent().map(Path::to_path_buf));

    let mut paths = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_package(entry.path()))
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());

    if let Some(kind) = operation_kind {
        operation::set_total(kind, paths.len());
        operation::update(kind, 0, None, "indexing");
    }

    let mut cache = load_cache(&root);
    retain_existing(&mut cache, &paths);

    let mut packages = Vec::<PackageInfo>::with_capacity(paths.len());
    let mut resource_index =
        BTreeMap::<(u32, u32, u64), Vec<ResourceOccurrence>>::new();
    let mut errors = Vec::new();
    let mut cache_hits = 0usize;
    let mut cache_misses = 0usize;
    let mut hashing_ms = 0u128;
    let mut dbpf_load_ms = 0u128;
    let mut resource_decode_ms = 0u128;

    for (index, path) in paths.iter().enumerate() {
        if let Some(kind) = operation_kind {
            if operation::is_cancelled(kind) {
                let _ = save_cache(&root, &cache);
                return Err(CANCELLED_ERROR.to_string());
            }
            operation::update(
                kind,
                index,
                path.file_name().map(|value| value.to_string_lossy().to_string()),
                "indexing",
            );
        }
        let relative_path = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_path_buf();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("package")
            .to_string();

        let package_index = packages.len();
        let priority = match (&resource_cfg, &resource_cfg_directory) {
            (Some(cfg), Some(directory)) => package_priority(cfg, directory, &path),
            _ => None,
        };
        let load_priority = priority.as_ref().map(|value| value.priority);
        let load_rule = priority.map(|value| value.rule);

        let (cached, metrics) = match get_or_build_with_metrics(path, &mut cache) {
            Ok(value) => value,
            Err(error) => {
                packages.push(PackageInfo {
                    name,
                    path: path.clone(),
                    relative_path,
                    readable: false,
                    load_priority,
                    load_rule,
                });
                errors.push(error);
                continue;
            }
        };

        if metrics.cache_hit {
            cache_hits += 1;
        } else {
            cache_misses += 1;
        }
        hashing_ms += metrics.hash_ms;
        dbpf_load_ms += metrics.dbpf_load_ms;
        resource_decode_ms += metrics.resource_decode_ms;

        let readable = cached.parse_error.is_none();
        if let Some(error) = &cached.parse_error {
            errors.push(format!("{}: {error}", path.display()));
        }

        packages.push(PackageInfo {
            name,
            path: path.clone(),
            relative_path,
            readable,
            load_priority,
            load_rule,
        });

        if readable {
            for resource in &cached.resources {
                resource_index
                    .entry((resource.type_id, resource.group, resource.instance))
                    .or_default()
                    .push(ResourceOccurrence {
                        package_index,
                        type_id: resource.type_id,
                        group: resource.group,
                        instance: resource.instance,
                        payload_hash: resource.payload_sha256.clone(),
                        payload_size: resource.payload_size,
                    });
            }
        }

        if let Some(kind) = operation_kind {
            operation::update(
                kind,
                index + 1,
                path.file_name().map(|value| value.to_string_lossy().to_string()),
                "indexing",
            );
        }
    }

    if let Err(error) = save_cache(&root, &cache) {
        errors.push(error);
    }

    if let Some(kind) = operation_kind {
        if operation::is_cancelled(kind) {
            return Err(CANCELLED_ERROR.to_string());
        }
        operation::update(kind, paths.len(), None, "comparing");
    }

    let mut pairs = HashMap::<(usize, usize), PairAccumulator>::new();
    let mut truncated = false;
    let mut comparisons = 0usize;

    'resources: for occurrences in resource_index.values() {
        if let Some(kind) = operation_kind {
            if operation::is_cancelled(kind) {
                return Err(CANCELLED_ERROR.to_string());
            }
        }
        let mut per_package = BTreeMap::<usize, &ResourceOccurrence>::new();
        for occurrence in occurrences {
            per_package.entry(occurrence.package_index).or_insert(occurrence);
        }

        let unique = per_package.values().copied().collect::<Vec<_>>();
        if unique.len() < 2 {
            continue;
        }

        for left_pos in 0..unique.len() {
            for right_pos in (left_pos + 1)..unique.len() {
                comparisons += 1;
                if comparisons % 512 == 0 {
                    if let Some(kind) = operation_kind {
                        if operation::is_cancelled(kind) {
                            return Err(CANCELLED_ERROR.to_string());
                        }
                        operation::set_message(
                            kind,
                            Some(format!("Compared {comparisons} shared-resource pair(s)")),
                        );
                    }
                }

                let left = unique[left_pos];
                let right = unique[right_pos];
                let key = (
                    left.package_index.min(right.package_index),
                    left.package_index.max(right.package_index),
                );

                if !pairs.contains_key(&key) && pairs.len() >= MAX_FINDINGS {
                    truncated = true;
                    break 'resources;
                }

                let same_payload = left.payload_hash == right.payload_hash
                    && left.payload_size == right.payload_size;
                let impact = impact_for(left.type_id, same_payload).to_string();

                let accumulator = pairs.entry(key).or_default();
                accumulator.shared_resource_count += 1;
                if same_payload {
                    accumulator.identical_payload_count += 1;
                } else {
                    accumulator.different_payload_count += 1;
                }
                accumulator.impact_kinds.insert(impact.clone());

                if accumulator.evidence.len() < MAX_EVIDENCE_PER_FINDING {
                    accumulator.evidence.push(ConflictEvidence {
                        resource_type: left.type_id,
                        resource_type_hex: format!("0x{:08X}", left.type_id),
                        resource_label: resource_label(left.type_id),
                        resource_class: resource_class(left.type_id).to_string(),
                        group: left.group,
                        group_hex: format!("0x{:08X}", left.group),
                        instance: left.instance,
                        instance_hex: format!("0x{:016X}", left.instance),
                        same_payload,
                        left_payload_sha256: left.payload_hash.clone(),
                        right_payload_sha256: right.payload_hash.clone(),
                        left_payload_size: left.payload_size,
                        right_payload_size: right.payload_size,
                        impact_kind: impact,
                    });
                } else {
                    accumulator.evidence_truncated = true;
                }
            }
        }
    }

    let mut findings = Vec::with_capacity(pairs.len());
    let mut stats = ConflictStats {
        packages_scanned: packages.len(),
        readable_packages: packages.iter().filter(|package| package.readable).count(),
        unreadable_packages: packages.iter().filter(|package| !package.readable).count(),
        analysis_truncated: truncated,
        cache_hits,
        cache_misses,
        ..ConflictStats::default()
    };

    let mut ordered_pairs = pairs.into_iter().collect::<Vec<_>>();
    ordered_pairs.sort_by_key(|((left, right), _)| {
        (
            packages[*left].relative_path.to_string_lossy().to_ascii_lowercase(),
            packages[*right].relative_path.to_string_lossy().to_ascii_lowercase(),
        )
    });

    for ((left_index, right_index), accumulator) in ordered_pairs {
        let kind = primary_kind(&accumulator.impact_kinds, accumulator.different_payload_count);

        match kind.as_str() {
            "shared_identical" => stats.shared_identical += 1,
            "visual_override" => stats.visual_overrides += 1,
            "catalog_override" => stats.catalog_overrides += 1,
            "gameplay_override" => stats.gameplay_overrides += 1,
            "script_conflict" => stats.script_conflicts += 1,
            "text_override" => stats.text_overrides += 1,
            "potential_conflict" => stats.potential_conflicts += 1,
            "mixed_override" => stats.mixed_overrides += 1,
            _ => stats.potential_conflicts += 1,
        }

        let left = &packages[left_index];
        let right = &packages[right_index];
        let impact_kinds = accumulator.impact_kinds.iter().cloned().collect::<Vec<_>>();

        let (load_order_status, higher_priority_path, load_order_explanation_key) =
            load_order_info(
                left,
                right,
                resource_cfg.is_some(),
                resource_cfg
                    .as_ref()
                    .map(|cfg| cfg.precedence_reliable)
                    .unwrap_or(false),
            );

        findings.push(ConflictFinding {
            id: pair_id(left, right),
            severity: severity_for(&kind).to_string(),
            explanation_key: explanation_key(&kind).to_string(),
            kind,
            impact_kinds,
            left: package_ref(left),
            right: package_ref(right),
            shared_resource_count: accumulator.shared_resource_count,
            identical_payload_count: accumulator.identical_payload_count,
            different_payload_count: accumulator.different_payload_count,
            evidence: accumulator.evidence,
            evidence_truncated: accumulator.evidence_truncated,
            load_order_status,
            higher_priority_path,
            load_order_explanation_key,
        });
    }

    stats.package_pairs = findings.len();

    Ok(ConflictAnalysis {
        root: root.to_string_lossy().to_string(),
        resource_cfg,
        findings,
        stats,
        errors,
    })
}

#[tauri::command]
pub async fn analyze_conflicts(folder: String) -> Result<ConflictAnalysis, String> {
    const KIND: &str = "conflicts";
    operation::begin(KIND, "starting");

    let joined = tauri::async_runtime::spawn_blocking(move || {
        analyze_conflicts_core(folder, Some(KIND))
    })
    .await;

    let result = match joined {
        Ok(result) => result,
        Err(error) => {
            let message = format!("Conflicts worker failed: {error}");
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
    fn advanced_resource_cfg_does_not_claim_a_winner() {
        let package = |name: &str, priority: i32| PackageInfo {
            name: name.to_string(),
            path: PathBuf::from(name),
            relative_path: PathBuf::from(name),
            readable: true,
            load_priority: Some(priority),
            load_rule: Some("Packages/*.package".to_string()),
        };

        let (status, winner, _) =
            load_order_info(&package("a.package", 1000), &package("b.package", 500), true, false);

        assert_eq!(status, "advanced_cfg_unresolved");
        assert!(winner.is_none());
    }

    #[test]
    fn same_payload_is_shared_not_conflict() {
        assert_eq!(impact_for(TYPE_ITUN, true), "shared_identical");
    }

    #[test]
    fn gameplay_payload_difference_is_gameplay_override() {
        assert_eq!(impact_for(TYPE_ITUN, false), "gameplay_override");
        assert_eq!(impact_for(TYPE_XML, false), "gameplay_override");
    }

    #[test]
    fn script_payload_difference_is_script_conflict() {
        assert_eq!(impact_for(TYPE_S3SA, false), "script_conflict");
    }

    #[test]
    fn visual_and_catalog_payload_differences_are_not_generic_conflicts() {
        assert_eq!(impact_for(TYPE_IMG, false), "visual_override");
        assert_eq!(impact_for(TYPE_CASP, false), "catalog_override");
        assert_eq!(impact_for(TYPE_OBJD, false), "catalog_override");
    }

    #[test]
    fn multiple_different_override_classes_are_mixed() {
        let kinds = ["visual_override".to_string(), "gameplay_override".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(primary_kind(&kinds, 2), "mixed_override");
    }

    #[test]
    fn identical_only_pair_is_informational() {
        let kinds = ["shared_identical".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(primary_kind(&kinds, 0), "shared_identical");
        assert_eq!(severity_for("shared_identical"), "info");
    }
}
