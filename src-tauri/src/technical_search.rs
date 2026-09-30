use crate::{cache::{get_or_build,load_cache,save_cache},dbpf::Package};
use serde::Serialize;
use std::path::{Path,PathBuf};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TechnicalSearchHit{
    pub package_path:String,
    pub relative_path:String,
    pub file_sha256:String,
    pub type_hex:Option<String>,
    pub group_hex:Option<String>,
    pub instance_hex:Option<String>,
    pub tgi:Option<String>,
    pub payload_sha256:Option<String>,
}
fn canonical_root(folder:&str)->Result<PathBuf,String>{
    let root=PathBuf::from(folder.trim()).canonicalize().map_err(|e|format!("Could not resolve root: {e}"))?;
    if !root.is_dir(){return Err("Selected root is not a directory.".into());}Ok(root)
}
fn is_package(path:&Path)->bool{path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("package")).unwrap_or(false)}
fn parse_hex_u32(s:&str)->Option<u32>{u32::from_str_radix(s.trim_start_matches("0x"),16).ok()}
fn parse_hex_u64(s:&str)->Option<u64>{u64::from_str_radix(s.trim_start_matches("0x"),16).ok()}
#[tauri::command]
pub fn technical_search(folder:String,query:String)->Result<Vec<TechnicalSearchHit>,String>{
    let root=canonical_root(&folder)?;
    let q=query.trim();
    if q.is_empty(){return Ok(Vec::new());}
    let lower=q.to_ascii_lowercase();
    let mut type_filter=None;let mut group_filter=None;let mut instance_filter=None;let mut sha_filter=None;
    if let Some(v)=lower.strip_prefix("type:"){type_filter=parse_hex_u32(v);}
    else if let Some(v)=lower.strip_prefix("group:"){group_filter=parse_hex_u32(v);}
    else if let Some(v)=lower.strip_prefix("instance:"){instance_filter=parse_hex_u64(v);}
    else if let Some(v)=lower.strip_prefix("sha:"){sha_filter=Some(v.to_ascii_uppercase());}

    let mut cache=load_cache(&root);let mut hits=Vec::new();
    for e in walkdir::WalkDir::new(&root).follow_links(false).into_iter().filter_map(Result::ok){
        if !e.file_type().is_file()||!is_package(e.path()){continue;}
        let rel=e.path().strip_prefix(&root).unwrap_or(e.path()).to_string_lossy().replace('/',"\\");
        let (cached,_)=get_or_build(e.path(),&mut cache)?;
        let generic=type_filter.is_none()&&group_filter.is_none()&&instance_filter.is_none()&&sha_filter.is_none();
        if generic && (rel.to_ascii_lowercase().contains(&lower)||cached.file_sha256.to_ascii_lowercase().contains(&lower)){
            hits.push(TechnicalSearchHit{package_path:e.path().to_string_lossy().to_string(),relative_path:rel.clone(),file_sha256:cached.file_sha256.clone(),type_hex:None,group_hex:None,instance_hex:None,tgi:None,payload_sha256:None});
        }
        if let Some(sha)=&sha_filter{
            if cached.file_sha256.starts_with(sha){
                hits.push(TechnicalSearchHit{package_path:e.path().to_string_lossy().to_string(),relative_path:rel.clone(),file_sha256:cached.file_sha256.clone(),type_hex:None,group_hex:None,instance_hex:None,tgi:None,payload_sha256:None});
            }
        }
        for r in &cached.resources{
            if type_filter.map(|v|v!=r.type_id).unwrap_or(false){continue;}
            if group_filter.map(|v|v!=r.group).unwrap_or(false){continue;}
            if instance_filter.map(|v|v!=r.instance).unwrap_or(false){continue;}
            if type_filter.is_none()&&group_filter.is_none()&&instance_filter.is_none(){continue;}
            hits.push(TechnicalSearchHit{
                package_path:e.path().to_string_lossy().to_string(),relative_path:rel.clone(),file_sha256:cached.file_sha256.clone(),
                type_hex:Some(format!("0x{:08X}",r.type_id)),group_hex:Some(format!("0x{:08X}",r.group)),
                instance_hex:Some(format!("0x{:016X}",r.instance)),
                tgi:Some(format!("0x{:08X}-0x{:08X}-0x{:016X}",r.type_id,r.group,r.instance)),
                payload_sha256:Some(r.payload_sha256.clone()),
            });
            if hits.len()>=5000{break;}
        }
        if hits.len()>=5000{break;}
    }
    let _=save_cache(&root,&cache);
    hits.sort_by_key(|h|h.relative_path.to_ascii_lowercase());
    Ok(hits)
}
