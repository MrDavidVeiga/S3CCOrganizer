use crate::{catalog::TYPE_CASP, dbpf::Package, i18n::AppLanguage};
use quick_xml::{events::Event, Reader};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TYPE_MANIFEST: u32 = 0x73E9_3EEB;
const TYPE_KEY: u32 = 0x0166_038C;
const MAX_XML_BYTES: usize = 16 * 1024 * 1024;
const MAX_PAYLOAD_BYTES: usize = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
struct ManifestNames {
    display_name: Option<String>,
    package_title: Option<String>,
    package_id: Option<String>,
    localized_names: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct PackagedFile {
    name: String,
    guid: String,
    length: u32,
    offset: u64,
    content_type: String,
}

#[derive(Debug, Clone)]
struct Sims3Pack {
    xml_offset: u64,
    manifest: ManifestNames,
    packaged_files: Vec<PackagedFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sims3PackConversionItem {
    pub index: usize,
    pub packaged_name: String,
    pub proposed_file_name: String,
    pub display_name: String,
    pub name_source: String,
    pub content_type: String,
    pub size: u64,
    pub convertible: bool,
    pub warning: Option<String>,
    pub casp_name: Option<String>,
    pub manifest_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sims3PackInspection {
    pub path: String,
    pub file_name: String,
    pub display_name: String,
    pub package_count: usize,
    pub set: bool,
    pub items: Vec<Sims3PackConversionItem>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sims3PackConvertedItem {
    pub source_path: String,
    pub packaged_name: String,
    pub output_path: String,
    pub output_file_name: String,
    pub name_source: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sims3PackConversionResult {
    pub converted: usize,
    pub skipped: usize,
    pub items: Vec<Sims3PackConvertedItem>,
    pub errors: Vec<String>,
}

fn locale_preferences(language: AppLanguage) -> &'static [&'static str] {
    match language {
        AppLanguage::En => &["en-US", "en-GB", "en"],
        AppLanguage::Pt => &["pt-BR", "pt-PT", "pt"],
        AppLanguage::Es => &["es-ES", "es-MX", "es"],
    }
}

fn localized_name(manifest: &ManifestNames, language: AppLanguage) -> Option<String> {
    for wanted in locale_preferences(language) {
        if let Some((_, value)) = manifest
            .localized_names
            .iter()
            .find(|(locale, _)| locale.eq_ignore_ascii_case(wanted))
        {
            if !value.trim().is_empty() {
                return Some(value.trim().to_string());
            }
        }
    }

    let prefix = match language {
        AppLanguage::En => "en",
        AppLanguage::Pt => "pt",
        AppLanguage::Es => "es",
    };
    if let Some((_, value)) = manifest
        .localized_names
        .iter()
        .find(|(locale, _)| locale.to_ascii_lowercase().starts_with(prefix))
    {
        if !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }

    None
}

fn best_manifest_name(manifest: &ManifestNames, language: AppLanguage) -> Option<String> {
    localized_name(manifest, language)
        .or_else(|| manifest.package_title.as_ref().filter(|v| !v.trim().is_empty()).cloned())
        .or_else(|| manifest.display_name.as_ref().filter(|v| !v.trim().is_empty()).cloned())
}

fn sanitize_file_stem(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => out.push('_'),
            c if c.is_control() => out.push('_'),
            c => out.push(c),
        }
    }
    let trimmed = out.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        "Converted Package".to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_guid_like_name(value: &str) -> bool {
    let stem = Path::new(value)
        .file_stem()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| value.to_string());
    let compact = stem.trim().trim_start_matches("0x").replace('-', "");
    compact.len() >= 16 && compact.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn meaningful_packaged_name(value: &str) -> Option<String> {
    let stem = Path::new(value).file_stem()?.to_string_lossy().trim().to_string();
    if stem.is_empty() || is_guid_like_name(&stem) {
        None
    } else {
        Some(stem)
    }
}

fn attr_language(
    event: &quick_xml::events::BytesStart<'_>,
    reader: &Reader<&[u8]>,
) -> Option<String> {
    for attr in event.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case(b"language") {
            return attr
                .decode_and_unescape_value(reader)
                .ok()
                .map(|value| value.to_string());
        }
    }
    None
}

fn parse_manifest_and_files(xml: &str) -> Result<(ManifestNames, Vec<PackagedFile>), String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut manifest = ManifestNames::default();
    let mut files = Vec::new();
    let mut in_localized_names = false;
    let mut in_packaged_file = false;
    let mut cur_name = String::new();
    let mut cur_guid = String::new();
    let mut cur_length = 0u32;
    let mut cur_offset = 0u64;
    let mut cur_type = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                let tag = String::from_utf8_lossy(event.name().as_ref()).to_ascii_lowercase();
                match tag.as_str() {
                    "localizednames" => in_localized_names = true,
                    "localizedname" if in_localized_names => {
                        let locale = attr_language(&event, &reader).unwrap_or_default();
                        let value = reader.read_text(event.name()).unwrap_or_default().into_owned();
                        if !locale.trim().is_empty() && !value.trim().is_empty() {
                            manifest.localized_names.insert(locale, value.trim().to_string());
                        }
                    }
                    "displayname" if !in_packaged_file => {
                        let value = reader.read_text(event.name()).unwrap_or_default().into_owned();
                        if !value.trim().is_empty() {
                            manifest.display_name = Some(value.trim().to_string());
                        }
                    }
                    "packagetitle" if !in_packaged_file => {
                        let value = reader.read_text(event.name()).unwrap_or_default().into_owned();
                        if !value.trim().is_empty() {
                            manifest.package_title = Some(value.trim().to_string());
                        }
                    }
                    "packageid" if !in_packaged_file => {
                        let value = reader.read_text(event.name()).unwrap_or_default().into_owned();
                        if !value.trim().is_empty() {
                            manifest.package_id = Some(value.trim().to_string());
                        }
                    }
                    "packagedfile" => {
                        in_packaged_file = true;
                        cur_name.clear();
                        cur_guid.clear();
                        cur_length = 0;
                        cur_offset = 0;
                        cur_type.clear();
                    }
                    "name" if in_packaged_file => {
                        cur_name = reader.read_text(event.name()).unwrap_or_default().into_owned();
                    }
                    "guid" if in_packaged_file => {
                        cur_guid = reader.read_text(event.name()).unwrap_or_default().into_owned();
                    }
                    "length" if in_packaged_file => {
                        cur_length = reader
                            .read_text(event.name())
                            .unwrap_or_default()
                            .trim()
                            .parse::<u32>()
                            .unwrap_or(0);
                    }
                    "offset" if in_packaged_file => {
                        cur_offset = reader
                            .read_text(event.name())
                            .unwrap_or_default()
                            .trim()
                            .parse::<u64>()
                            .unwrap_or(0);
                    }
                    "contenttype" if in_packaged_file => {
                        cur_type = reader
                            .read_text(event.name())
                            .unwrap_or_default()
                            .to_ascii_lowercase();
                    }
                    _ => {}
                }
            }
            Ok(Event::End(event)) => {
                let tag = String::from_utf8_lossy(event.name().as_ref()).to_ascii_lowercase();
                match tag.as_str() {
                    "localizednames" => in_localized_names = false,
                    "packagedfile" if in_packaged_file => {
                        in_packaged_file = false;
                        files.push(PackagedFile {
                            name: cur_name.trim().to_string(),
                            guid: cur_guid.trim().to_string(),
                            length: cur_length,
                            offset: cur_offset,
                            content_type: cur_type.trim().to_string(),
                        });
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(format!("Sims3Pack XML parse error: {error}")),
            _ => {}
        }
        buf.clear();
    }

    Ok((manifest, files))
}

fn read_sims3pack(path: &Path) -> Result<Sims3Pack, String> {
    let mut file = File::open(path).map_err(|e| format!("Could not open {}: {e}", path.display()))?;
    let mut header = [0u8; 17];
    file.read_exact(&mut header)
        .map_err(|e| format!("Could not read Sims3Pack header: {e}"))?;

    if &header[4..11] != b"TS3Pack" {
        return Err("File is not a valid Sims3Pack.".to_string());
    }

    let xml_length = u32::from_le_bytes([header[13], header[14], header[15], header[16]]) as usize;
    if xml_length == 0 || xml_length > MAX_XML_BYTES {
        return Err(format!("Invalid Sims3Pack XML length: {xml_length}."));
    }

    let mut xml_bytes = vec![0u8; xml_length];
    file.read_exact(&mut xml_bytes)
        .map_err(|e| format!("Could not read Sims3Pack XML: {e}"))?;
    let xml = String::from_utf8(xml_bytes)
        .map_err(|e| format!("Sims3Pack XML is not valid UTF-8: {e}"))?;
    let (manifest, packaged_files) = parse_manifest_and_files(&xml)?;

    Ok(Sims3Pack {
        xml_offset: 17 + xml_length as u64,
        manifest,
        packaged_files,
    })
}

fn read_packaged_payload(path: &Path, pack: &Sims3Pack, item: &PackagedFile) -> Result<Vec<u8>, String> {
    let length = item.length as usize;
    if length == 0 || length > MAX_PAYLOAD_BYTES {
        return Err(format!("Invalid embedded payload size: {length}."));
    }
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let start = pack
        .xml_offset
        .checked_add(item.offset)
        .ok_or_else(|| "Embedded payload offset overflow.".to_string())?;
    let end = start
        .checked_add(item.length as u64)
        .ok_or_else(|| "Embedded payload range overflow.".to_string())?;
    if end > meta.len() {
        return Err("Embedded payload points outside the Sims3Pack.".to_string());
    }

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut bytes = vec![0u8; length];
    file.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(offset..offset + 4)?.try_into().ok()?))
}

fn read_i32(data: &[u8], offset: usize) -> Option<i32> {
    Some(i32::from_le_bytes(data.get(offset..offset + 4)?.try_into().ok()?))
}

fn read_7bit_encoded_int(data: &[u8], offset: &mut usize) -> Option<usize> {
    let mut result = 0usize;
    let mut shift = 0usize;
    loop {
        if shift >= usize::BITS as usize {
            return None;
        }
        let value = *data.get(*offset)?;
        *offset += 1;
        result |= ((value & 0x7F) as usize) << shift;
        if value & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
    }
}

fn read_be_7bit_string(data: &[u8], offset: &mut usize) -> Option<String> {
    let byte_len = read_7bit_encoded_int(data, offset)?;
    if byte_len == 0 {
        return Some(String::new());
    }
    let end = offset.checked_add(byte_len)?;
    let raw = data.get(*offset..end)?;
    if raw.len() % 2 != 0 {
        return None;
    }
    let words = raw
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    *offset = end;
    Some(String::from_utf16_lossy(&words))
}

fn casp_internal_name(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }
    let mut offset = 8usize;
    let preset_count = read_u32(data, offset)? as usize;
    offset += 4;
    if preset_count > 4096 {
        return None;
    }

    for _ in 0..preset_count {
        let xml_len = read_i32(data, offset)?;
        if xml_len < 0 {
            return None;
        }
        offset += 4;
        let xml_bytes = (xml_len as usize).checked_mul(2)?;
        offset = offset.checked_add(xml_bytes)?;
        offset = offset.checked_add(4)?;
        if offset > data.len() {
            return None;
        }
    }

    let name = read_be_7bit_string(data, &mut offset)?;
    let trimmed = name.trim_matches('\0').trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn key_names(package: &Package) -> BTreeMap<u64, String> {
    let mut names = BTreeMap::new();
    for entry in package.entries.iter().filter(|entry| entry.type_id == TYPE_KEY) {
        let Ok(data) = package.data(entry) else { continue };
        if data.len() < 8 || read_u32(&data, 0) != Some(1) {
            continue;
        }
        let Some(count) = read_u32(&data, 4).map(|v| v as usize) else { continue };
        if count > 100_000 {
            continue;
        }
        let mut offset = 8usize;
        for _ in 0..count {
            if offset + 12 > data.len() {
                break;
            }
            let instance = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            let length = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as usize;
            offset += 12;
            let Some(end) = offset.checked_add(length) else { break };
            if length > 4096 || end > data.len() {
                break;
            }
            let name = String::from_utf8_lossy(&data[offset..end])
                .trim_matches('\0')
                .trim()
                .to_string();
            if !name.is_empty() {
                names.entry(instance).or_insert(name);
            }
            offset = end;
        }
    }
    names
}

fn package_casp_name(package: &Package) -> Option<String> {
    let key_map = key_names(package);
    for entry in package.entries.iter().filter(|entry| entry.type_id == TYPE_CASP) {
        if let Ok(data) = package.data(entry) {
            if let Some(name) = casp_internal_name(&data) {
                return Some(name);
            }
        }
        if let Some(name) = key_map.get(&entry.instance) {
            if !name.trim().is_empty() {
                return Some(name.trim().to_string());
            }
        }
    }
    None
}

fn package_manifest_names(package: &Package) -> ManifestNames {
    for entry in package.entries.iter().filter(|entry| entry.type_id == TYPE_MANIFEST) {
        let Ok(data) = package.data(entry) else { continue };
        let Ok(text) = std::str::from_utf8(&data) else { continue };
        if let Ok((manifest, _)) = parse_manifest_and_files(text) {
            if manifest.package_title.is_some()
                || manifest.display_name.is_some()
                || !manifest.localized_names.is_empty()
            {
                return manifest;
            }
        }
    }
    ManifestNames::default()
}

fn temporary_package_path(index: usize) -> Result<PathBuf, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "veigas_s3cc_manager_convert_{}_{}_{}.package",
        std::process::id(),
        nonce,
        index
    )))
}

fn inspect_payload(
    payload: &[u8],
    index: usize,
    language: AppLanguage,
) -> Result<(Option<String>, Option<String>), String> {
    if payload.len() < 96 || &payload[..4] != b"DBPF" {
        return Err("Embedded file is not a supported Sims 3 DBPF package.".to_string());
    }
    let temp = temporary_package_path(index)?;
    let result = (|| {
        fs::write(&temp, payload).map_err(|e| format!("Could not stage embedded package: {e}"))?;
        let package = Package::load(&temp).map_err(|e| format!("Invalid embedded package: {e}"))?;
        let manifest = package_manifest_names(&package);
        let manifest_name = best_manifest_name(&manifest, language);
        let casp_name = package_casp_name(&package);
        Ok((manifest_name, casp_name))
    })();
    let _ = fs::remove_file(&temp);
    result
}

fn proposed_name(
    pack: &Sims3Pack,
    packaged: &PackagedFile,
    valid_package_count: usize,
    manifest_name: Option<&str>,
    casp_name: Option<&str>,
    index: usize,
    language: AppLanguage,
) -> (String, String, String) {
    if let Some(value) = manifest_name.filter(|v| !v.trim().is_empty()) {
        let stem = sanitize_file_stem(value);
        return (format!("{stem}.package"), stem, "manifest".to_string());
    }

    if valid_package_count == 1 {
        if let Some(value) = best_manifest_name(&pack.manifest, language) {
            let stem = sanitize_file_stem(&value);
            return (format!("{stem}.package"), stem, "sims3pack_manifest".to_string());
        }
    }

    if let Some(value) = casp_name.filter(|v| !v.trim().is_empty()) {
        let stem = sanitize_file_stem(value);
        return (format!("{stem}.package"), stem, "casp".to_string());
    }

    if let Some(value) = meaningful_packaged_name(&packaged.name) {
        let stem = sanitize_file_stem(&value);
        return (format!("{stem}.package"), stem, "packaged_file".to_string());
    }

    let fallback = format!("Converted Package {}", index + 1);
    (format!("{fallback}.package"), fallback, "fallback".to_string())
}

fn inspect_one(path: &Path, language: AppLanguage) -> Result<Sims3PackInspection, String> {
    let pack = read_sims3pack(path)?;
    let mut staged = Vec::new();
    let mut valid_count = 0usize;

    for (index, packaged) in pack.packaged_files.iter().enumerate() {
        if packaged.length == 0 {
            continue;
        }
        match read_packaged_payload(path, &pack, packaged) {
            Ok(payload) if payload.len() >= 4 && &payload[..4] == b"DBPF" => {
                valid_count += 1;
                staged.push((index, packaged, Some(payload), None));
            }
            Ok(_) => staged.push((
                index,
                packaged,
                None,
                Some("Embedded file is not a DBPF package and will not be converted.".to_string()),
            )),
            Err(error) => staged.push((index, packaged, None, Some(error))),
        }
    }

    let mut items = Vec::new();
    let mut warnings = Vec::new();
    for (index, packaged, payload, prewarning) in staged {
        if let Some(payload) = payload {
            match inspect_payload(&payload, index, language) {
                Ok((manifest_name, casp_name)) => {
                    let (file_name, display_name, name_source) = proposed_name(
                        &pack,
                        packaged,
                        valid_count,
                        manifest_name.as_deref(),
                        casp_name.as_deref(),
                        index,
                        language,
                    );
                    items.push(Sims3PackConversionItem {
                        index,
                        packaged_name: packaged.name.clone(),
                        proposed_file_name: file_name,
                        display_name,
                        name_source,
                        content_type: packaged.content_type.clone(),
                        size: packaged.length as u64,
                        convertible: true,
                        warning: prewarning,
                        casp_name,
                        manifest_name,
                    });
                }
                Err(error) => {
                    warnings.push(format!("{}: {error}", packaged.name));
                    items.push(Sims3PackConversionItem {
                        index,
                        packaged_name: packaged.name.clone(),
                        proposed_file_name: String::new(),
                        display_name: packaged.name.clone(),
                        name_source: "invalid".to_string(),
                        content_type: packaged.content_type.clone(),
                        size: packaged.length as u64,
                        convertible: false,
                        warning: Some(error),
                        casp_name: None,
                        manifest_name: None,
                    });
                }
            }
        } else {
            let warning = prewarning.unwrap_or_else(|| "Not convertible.".to_string());
            warnings.push(format!("{}: {warning}", packaged.name));
            items.push(Sims3PackConversionItem {
                index,
                packaged_name: packaged.name.clone(),
                proposed_file_name: String::new(),
                display_name: packaged.name.clone(),
                name_source: "non_package".to_string(),
                content_type: packaged.content_type.clone(),
                size: packaged.length as u64,
                convertible: false,
                warning: Some(warning),
                casp_name: None,
                manifest_name: None,
            });
        }
    }

    let display_name = best_manifest_name(&pack.manifest, language)
        .or_else(|| path.file_stem().map(|v| v.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Sims3Pack".to_string());

    Ok(Sims3PackInspection {
        path: path.to_string_lossy().to_string(),
        file_name: path.file_name().map(|v| v.to_string_lossy().to_string()).unwrap_or_default(),
        display_name,
        package_count: valid_count,
        set: valid_count > 1,
        items,
        warnings,
    })
}

fn unique_output_path(folder: &Path, file_name: &str) -> PathBuf {
    let base = Path::new(file_name);
    let stem = base
        .file_stem()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "Converted Package".to_string());
    let ext = base.extension().and_then(|v| v.to_str()).unwrap_or("package");
    let mut attempt = 0usize;
    loop {
        let name = if attempt == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem} ({attempt}).{ext}")
        };
        let candidate = folder.join(name);
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}

fn write_no_replace(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Could not create {}: {e}", path.display()))?;

    if let Err(error) = file.write_all(data) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(format!("Could not write {}: {error}", path.display()));
    }
    if let Err(error) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(format!("Could not finalize {}: {error}", path.display()));
    }
    Ok(())
}

#[tauri::command]
pub fn inspect_sims3packs(paths: Vec<String>, language: AppLanguage) -> Result<Vec<Sims3PackInspection>, String> {
    let mut results = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw.trim());
        if !path.is_file() {
            return Err(format!("Sims3Pack not found: {}", path.display()));
        }
        results.push(inspect_one(&path, language)?);
    }
    Ok(results)
}

#[tauri::command]
pub fn convert_sims3packs(
    paths: Vec<String>,
    destination_folder: String,
    language: AppLanguage,
) -> Result<Sims3PackConversionResult, String> {
    let destination = PathBuf::from(destination_folder.trim());
    if destination.as_os_str().is_empty() {
        return Err("No destination folder selected.".to_string());
    }
    fs::create_dir_all(&destination)
        .map_err(|e| format!("Could not create destination folder: {e}"))?;

    let mut result = Sims3PackConversionResult::default();

    for raw in paths {
        let source = PathBuf::from(raw.trim());
        let inspection = match inspect_one(&source, language) {
            Ok(value) => value,
            Err(error) => {
                result.errors.push(format!("{}: {error}", source.display()));
                continue;
            }
        };
        let pack = match read_sims3pack(&source) {
            Ok(value) => value,
            Err(error) => {
                result.errors.push(format!("{}: {error}", source.display()));
                continue;
            }
        };

        let output_root = if inspection.set {
            let set_name = source
                .file_stem()
                .map(|v| sanitize_file_stem(&v.to_string_lossy()))
                .unwrap_or_else(|| "Sims3Pack Set".to_string());
            let folder = destination.join(set_name);
            if let Err(error) = fs::create_dir_all(&folder) {
                result.errors.push(format!("{}: {error}", source.display()));
                continue;
            }
            folder
        } else {
            destination.clone()
        };

        for item in inspection.items.iter().filter(|item| item.convertible) {
            let Some(packaged) = pack.packaged_files.get(item.index) else {
                result.skipped += 1;
                result.errors.push(format!(
                    "{}: packaged item {} disappeared during conversion.",
                    source.display(),
                    item.index
                ));
                continue;
            };
            let payload = match read_packaged_payload(&source, &pack, packaged) {
                Ok(value) if value.len() >= 4 && &value[..4] == b"DBPF" => value,
                Ok(_) => {
                    result.skipped += 1;
                    continue;
                }
                Err(error) => {
                    result.skipped += 1;
                    result.errors.push(format!("{} / {}: {error}", source.display(), packaged.name));
                    continue;
                }
            };

            let mut target = unique_output_path(&output_root, &item.proposed_file_name);
            loop {
                match write_no_replace(&target, &payload) {
                    Ok(()) => break,
                    Err(error) if target.exists() => {
                        target = unique_output_path(&output_root, &item.proposed_file_name);
                    }
                    Err(error) => {
                        result.skipped += 1;
                        result.errors.push(error);
                        target = PathBuf::new();
                        break;
                    }
                }
            }
            if target.as_os_str().is_empty() {
                continue;
            }

            result.converted += 1;
            result.items.push(Sims3PackConvertedItem {
                source_path: source.to_string_lossy().to_string(),
                packaged_name: packaged.name.clone(),
                output_file_name: target.file_name().map(|v| v.to_string_lossy().to_string()).unwrap_or_default(),
                output_path: target.to_string_lossy().to_string(),
                name_source: item.name_source.clone(),
            });
        }

        result.skipped += inspection.items.iter().filter(|item| !item.convertible).count();
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_prefers_matching_localized_name() {
        let mut manifest = ManifestNames::default();
        manifest.display_name = Some("English fallback".into());
        manifest.localized_names.insert("pt-BR".into(), "Nome Brasileiro".into());
        manifest.localized_names.insert("es-ES".into(), "Nombre Español".into());
        assert_eq!(best_manifest_name(&manifest, AppLanguage::Pt).as_deref(), Some("Nome Brasileiro"));
        assert_eq!(best_manifest_name(&manifest, AppLanguage::Es).as_deref(), Some("Nombre Español"));
    }

    #[test]
    fn guid_packaged_names_are_not_used_as_friendly_names() {
        assert!(meaningful_packaged_name("0xc97ea9c600104fcbafdf2d64ffceae2b.package").is_none());
        assert_eq!(meaningful_packaged_name("CoolHair.package").as_deref(), Some("CoolHair"));
    }

    #[test]
    fn file_name_sanitizer_keeps_readable_names() {
        assert_eq!(sanitize_file_stem("Cool: Hair / Set"), "Cool_ Hair _ Set");
    }
}
