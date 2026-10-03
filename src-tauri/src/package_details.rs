use crate::{
    cache::{get_or_build, load_cache, save_cache},
    catalog::{TYPE_CASP, TYPE_OBJD},
    mesh_info::{analyze_package_meshes, PackageMeshInfo},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::path::PathBuf;

use crate::dbpf::Package;

const TYPE_IMG: u32 = 0x00B2_D882;
const TYPE_THUM_SMALL: u32 = 0x626F_60CC;
const TYPE_THUM_MED: u32 = 0x626F_60CD;
const TYPE_THUM_LARGE: u32 = 0x626F_60CE;
const TYPE_THUM_SMALL_ALT: u32 = 0x0580_A2B4;
const TYPE_THUM_MED_ALT: u32 = 0x0580_A2B5;
const TYPE_THUM_LARGE_ALT: u32 = 0x0580_A2B6;
const TYPE_ICON: u32 = 0x2E75_C765;
const TYPE_IMAG_JPG: u32 = 0x2F7D_0002;
const TYPE_IMAG_PNG: u32 = 0x2F7D_0004;
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
    pub mesh: PackageMeshInfo,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackagePreview {
    pub thumbnail_base64: Option<String>,
    pub mime_type: Option<String>,
}

fn preview_mime(data: &[u8]) -> Option<&'static str> {
    if data.len() >= 8 && &data[..8] == b"\x89PNG\r\n\x1a\n" {
        Some("image/png")
    } else if data.len() >= 3 && &data[..3] == b"\xFF\xD8\xFF" {
        Some("image/jpeg")
    } else {
        None
    }
}

fn package_preview_data(path: &std::path::Path) -> Result<PackagePreview, String> {
    let package = Package::load(path).map_err(|error| error.to_string())?;
    let catalog_instances = package.entries.iter()
        .filter(|entry| matches!(entry.type_id, TYPE_CASP | TYPE_OBJD))
        .map(|entry| entry.instance)
        .collect::<std::collections::HashSet<_>>();

    let primary_types = [
        TYPE_THUM_LARGE, TYPE_THUM_MED, TYPE_THUM_SMALL,
        TYPE_THUM_LARGE_ALT, TYPE_THUM_MED_ALT, TYPE_THUM_SMALL_ALT,
        TYPE_ICON,
    ];

    let mut candidates = Vec::<(u8, usize, Vec<u8>, &'static str)>::new();
    for entry in &package.entries {
        let priority = if primary_types.contains(&entry.type_id) {
            if catalog_instances.contains(&entry.instance) { 3 } else { 2 }
        } else if matches!(entry.type_id, TYPE_IMAG_PNG | TYPE_IMAG_JPG)
            && catalog_instances.contains(&entry.instance)
        {
            1
        } else {
            0
        };
        if priority == 0 || entry.mem_size > 16 * 1024 * 1024 {
            continue;
        }
        let Ok(data) = package.data(entry) else { continue };
        let Some(mime) = preview_mime(&data) else { continue };
        candidates.push((priority, data.len(), data, mime));
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let Some((_, _, data, mime)) = candidates.into_iter().next() else {
        return Ok(PackagePreview { thumbnail_base64: None, mime_type: None });
    };

    Ok(PackagePreview {
        thumbnail_base64: Some(STANDARD.encode(data)),
        mime_type: Some(mime.to_string()),
    })
}

#[tauri::command]
pub fn get_package_preview(folder: String, package_path: String) -> Result<PackagePreview, String> {
    let root = PathBuf::from(folder.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve root folder: {error}"))?;
    let path = PathBuf::from(package_path.trim())
        .canonicalize()
        .map_err(|error| format!("Could not resolve package path: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err("Package is outside the selected root.".to_string());
    }
    package_preview_data(&path)
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

    let mesh = analyze_package_meshes(&path);

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
        mesh,
    })
}
