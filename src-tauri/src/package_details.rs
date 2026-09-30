use crate::{
    cache::{get_or_build, load_cache, save_cache},
    catalog::{TYPE_CASP, TYPE_OBJD},
};
use serde::Serialize;
use std::path::PathBuf;

const TYPE_IMG: u32 = 0x00B2_D882;
const TYPE_GEOM: u32 = 0x015A_1849;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageResourceDetails {
    pub index: usize,
    pub type_id: u32,
    pub type_hex: String,
    pub type_label: String,
    pub resource_class: String,
    pub group: u32,
    pub group_hex: String,
    pub instance: u64,
    pub instance_hex: String,
    pub tgi: String,
    pub disk_size: u32,
    pub memory_size: u32,
    pub compression: String,
    pub payload_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageTechnicalDetails {
    pub path: String,
    pub size: u64,
    pub file_sha256: String,
    pub dbpf_major: Option<u32>,
    pub dbpf_minor: Option<u32>,
    pub resource_count: usize,
    pub cache_hit: bool,
    pub parse_error: Option<String>,
    pub resources: Vec<PackageResourceDetails>,
}

fn type_label(type_id: u32) -> String {
    match type_id {
        TYPE_IMG => "_IMG".to_string(),
        TYPE_GEOM => "GEOM".to_string(),
        TYPE_MODL => "MODL".to_string(),
        TYPE_MATD => "MATD".to_string(),
        TYPE_MLOD => "MLOD".to_string(),
        TYPE_OBJK => "OBJK".to_string(),
        TYPE_XML => "XML".to_string(),
        TYPE_TXTC => "TXTC".to_string(),
        TYPE_TXTF => "TXTF".to_string(),
        TYPE_ITUN => "ITUN".to_string(),
        TYPE_CASP => "CASP".to_string(),
        TYPE_OBJD => "OBJD".to_string(),
        TYPE_STBL => "STBL".to_string(),
        TYPE_VPXY => "VPXY".to_string(),
        TYPE_S3SA => "S3SA".to_string(),
        other => format!("0x{other:08X}"),
    }
}

fn resource_class(type_id: u32) -> &'static str {
    match type_id {
        TYPE_CASP | TYPE_OBJD => "catalog",
        TYPE_S3SA => "script",
        TYPE_XML | TYPE_ITUN => "gameplay",
        TYPE_STBL => "text",
        TYPE_IMG | TYPE_GEOM | TYPE_MODL | TYPE_MLOD | TYPE_VPXY | TYPE_MATD | TYPE_TXTC
        | TYPE_TXTF | TYPE_OBJK => "visual",
        _ => "other",
    }
}

fn compression_label(value: u16) -> String {
    match value {
        0x0000 => "None".to_string(),
        0xFFFF => "RefPack".to_string(),
        other => format!("0x{other:04X}"),
    }
}

#[tauri::command]
pub fn get_package_technical_details(
    folder: String,
    package_path: String,
) -> Result<PackageTechnicalDetails, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve root folder: {error}"))?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package path: {error}"))?;

    if !path.starts_with(&root) {
        return Err("Package is outside the selected root.".to_string());
    }

    let mut cache = load_cache(&root);
    let (cached, cache_hit) = get_or_build(&path, &mut cache)?;
    let _ = save_cache(&root, &cache);

    let resources = cached
        .resources
        .iter()
        .enumerate()
        .map(|(index, resource)| PackageResourceDetails {
            index,
            type_id: resource.type_id,
            type_hex: format!("0x{:08X}", resource.type_id),
            type_label: type_label(resource.type_id),
            resource_class: resource_class(resource.type_id).to_string(),
            group: resource.group,
            group_hex: format!("0x{:08X}", resource.group),
            instance: resource.instance,
            instance_hex: format!("0x{:016X}", resource.instance),
            tgi: format!(
                "0x{:08X}-0x{:08X}-0x{:016X}",
                resource.type_id, resource.group, resource.instance
            ),
            disk_size: resource.file_size,
            memory_size: resource.mem_size,
            compression: compression_label(resource.compressed),
            payload_sha256: resource.payload_sha256.clone(),
        })
        .collect::<Vec<_>>();

    Ok(PackageTechnicalDetails {
        path: cached.path,
        size: cached.size,
        file_sha256: cached.file_sha256,
        dbpf_major: cached.dbpf_major,
        dbpf_minor: cached.dbpf_minor,
        resource_count: resources.len(),
        cache_hit,
        parse_error: cached.parse_error,
        resources,
    })
}
