use crate::dbpf::Package;
use serde::Serialize;
use std::{collections::{HashMap,HashSet},path::{Path,PathBuf}};

const TYPE_VPXY:u32=0x7368_84F1;
const TYPE_OBJK:u32=0x02DC_343F;
const TYPE_MODL:u32=0x0166_1233;
const TYPE_MLOD:u32=0x01D1_0F34;
const TYPE_XML:u32=0x0333_406C;

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct DependencyEvidence{
    pub source_package:String,
    pub source_relative_path:String,
    pub source_resource:String,
    pub target_package:String,
    pub target_relative_path:String,
    pub target_resource:String,
    pub evidence:String,
}
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct DependencyAnalysis{
    pub root:String,
    pub findings:Vec<DependencyEvidence>,
    pub scanned_packages:usize,
    pub scanned_reference_resources:usize,
    pub truncated:bool,
}
fn is_reference_type(t:u32)->bool{matches!(t,TYPE_VPXY|TYPE_OBJK|TYPE_MODL|TYPE_MLOD|TYPE_XML)}
fn canonical_root(folder:&str)->Result<PathBuf,String>{
    let root=PathBuf::from(folder.trim()).canonicalize().map_err(|e|format!("Could not resolve root: {e}"))?;
    if !root.is_dir(){return Err("Selected root is not a directory.".into());} Ok(root)
}
fn is_package(path:&Path)->bool{
    path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("package")).unwrap_or(false)
}
fn tgi_bytes(t:u32,g:u32,i:u64)->[u8;16]{
    let mut b=[0u8;16];
    b[0..4].copy_from_slice(&t.to_le_bytes());
    b[4..8].copy_from_slice(&g.to_le_bytes());
    b[8..16].copy_from_slice(&i.to_le_bytes());
    b
}
#[tauri::command]
pub fn analyze_dependencies(folder:String)->Result<DependencyAnalysis,String>{
    let root=canonical_root(&folder)?;
    let mut packages=Vec::new();
    let mut targets=HashMap::<[u8;16],Vec<(usize,String)>>::new();
    for e in walkdir::WalkDir::new(&root).follow_links(false).into_iter().filter_map(Result::ok){
        if !e.file_type().is_file()||!is_package(e.path()){continue;}
        if let Ok(pkg)=Package::load(e.path()){
            let idx=packages.len();
            let rel=e.path().strip_prefix(&root).unwrap_or(e.path()).to_string_lossy().replace('/',"\");
            for r in &pkg.entries{
                targets.entry(tgi_bytes(r.type_id,r.group,r.instance)).or_default().push((idx,r.key_string()));
            }
            packages.push((e.path().to_path_buf(),rel,pkg));
        }
    }
    let mut findings=Vec::new();
    let mut seen=HashSet::new();
    let mut reference_resources=0usize;
    let mut truncated=false;
    'packages:for (source_idx,(source_path,source_rel,pkg)) in packages.iter().enumerate(){
        for r in &pkg.entries{
            if !is_reference_type(r.type_id){continue;}
            let Ok(data)=pkg.data(r) else{continue;};
            reference_resources+=1;
            if data.len()<16{continue;}
            for offset in 0..=data.len()-16{
                let mut key=[0u8;16];key.copy_from_slice(&data[offset..offset+16]);
                let Some(matches)=targets.get(&key) else{continue;};
                for (target_idx,target_resource) in matches{
                    if *target_idx==source_idx{continue;}
                    let pair=(source_idx,*target_idx,r.key_string(),target_resource.clone());
                    if !seen.insert(pair.clone()){continue;}
                    findings.push(DependencyEvidence{
                        source_package:source_path.to_string_lossy().to_string(),
                        source_relative_path:source_rel.clone(),
                        source_resource:r.key_string(),
                        target_package:packages[*target_idx].0.to_string_lossy().to_string(),
                        target_relative_path:packages[*target_idx].1.clone(),
                        target_resource:target_resource.clone(),
                        evidence:format!("Exact 16-byte little-endian TGI reference found at payload offset 0x{offset:X}."),
                    });
                    if findings.len()>=5000{truncated=true;break 'packages;}
                }
            }
        }
    }
    findings.sort_by_key(|f|(f.source_relative_path.to_ascii_lowercase(),f.target_relative_path.to_ascii_lowercase()));
    Ok(DependencyAnalysis{root:root.to_string_lossy().to_string(),findings,scanned_packages:packages.len(),scanned_reference_resources:reference_resources,truncated})
}
