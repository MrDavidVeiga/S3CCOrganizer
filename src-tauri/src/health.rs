use crate::{
    dbpf::Package,
    workspace::ensure_writable,
    resource_cfg::{find_resource_cfg, package_priority, parse_resource_cfg, ResourceCfgInfo},
};
use serde::Serialize;
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct CoverageEntry {
    pub path:String,
    pub relative_path:String,
    pub covered:bool,
    pub priority:Option<i32>,
    pub rule:Option<String>,
    pub depth:usize,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all="camelCase")]
pub struct HealthStats {
    pub packages:usize,
    pub readable:usize,
    pub unreadable:usize,
    pub empty_folders:usize,
    pub resource_cfg_uncovered:usize,
    pub packages_outside_root:usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct ModsHealthReport {
    pub root:String,
    pub stats:HealthStats,
    pub empty_folders:Vec<String>,
    pub unreadable_packages:Vec<String>,
    pub outside_packages:Vec<String>,
    pub coverage:Vec<CoverageEntry>,
    pub resource_cfg:Option<ResourceCfgInfo>,
}

fn canonical_root(folder:&str)->Result<PathBuf,String>{
    let root=PathBuf::from(folder.trim()).canonicalize().map_err(|e|format!("Could not resolve root: {e}"))?;
    if !root.is_dir(){return Err("Selected root is not a directory.".into());}
    Ok(root)
}
fn is_package(path:&Path)->bool{
    path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("package")).unwrap_or(false)
}
fn rel(root:&Path,path:&Path)->String{
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('/',"\\")
}
fn empty_dirs(root:&Path)->Vec<String>{
    let mut out=Vec::new();
    for e in walkdir::WalkDir::new(root).follow_links(false).min_depth(1).into_iter().filter_map(Result::ok){
        if !e.file_type().is_dir(){continue;}
        let is_empty=fs::read_dir(e.path()).map(|mut it|it.next().is_none()).unwrap_or(false);
        if is_empty{out.push(rel(root,e.path()));}
    }
    out.sort_by_key(|v|v.to_ascii_lowercase());out
}
fn outside_packages(root:&Path)->Vec<String>{
    let Some(parent)=root.parent() else{return Vec::new();};
    let mut out=Vec::new();
    for e in walkdir::WalkDir::new(parent).follow_links(false).max_depth(6).into_iter().filter_map(Result::ok){
        if !e.file_type().is_file() || !is_package(e.path()) || e.path().starts_with(root){continue;}
        let text=e.path().to_string_lossy();
        if text.contains("S3CC Organizer"){continue;}
        out.push(e.path().to_string_lossy().to_string());
    }
    out.sort_by_key(|v|v.to_ascii_lowercase());out
}
#[tauri::command]
pub fn remove_empty_folder(folder:String,relative_path:String)->Result<bool,String>{
    let root=canonical_root(&folder)?;
    ensure_writable(&root)?;
    let normalized=relative_path.replace('\\',"/");
    let relative=Path::new(&normalized);
    if relative.is_absolute() || relative.components().any(|c|matches!(c,std::path::Component::ParentDir|std::path::Component::RootDir|std::path::Component::Prefix(_))){
        return Err("Unsafe empty-folder path.".into());
    }
    let candidate=root.join(relative);
    let meta=fs::symlink_metadata(&candidate).map_err(|e|format!("Could not inspect folder: {e}"))?;
    if meta.file_type().is_symlink() || !meta.is_dir(){return Err("Target is not a regular directory.".into());}
    let canonical=candidate.canonicalize().map_err(|e|e.to_string())?;
    if !canonical.starts_with(&root)||canonical==root{return Err("Target is outside the selected root.".into());}
    if fs::read_dir(&canonical).map_err(|e|e.to_string())?.next().is_some(){return Err("Folder is not empty.".into());}
    fs::remove_dir(&canonical).map_err(|e|format!("Could not remove empty folder: {e}"))?;
    Ok(true)
}

#[tauri::command]
pub fn analyze_mods_health(folder:String)->Result<ModsHealthReport,String>{
    let root=canonical_root(&folder)?;
    let cfg_path=find_resource_cfg(&root);
    let resource_cfg=cfg_path.as_ref().map(|p|parse_resource_cfg(p)).transpose()?;
    let cfg_dir=cfg_path.as_ref().and_then(|p|p.parent()).map(PathBuf::from);

    let mut coverage=Vec::new();
    let mut unreadable=Vec::new();
    let mut stats=HealthStats::default();

    for e in walkdir::WalkDir::new(&root).follow_links(false).into_iter().filter_map(Result::ok){
        if !e.file_type().is_file() || !is_package(e.path()){continue;}
        stats.packages+=1;
        match Package::load(e.path()){Ok(_)=>stats.readable+=1,Err(_)=>{stats.unreadable+=1;unreadable.push(rel(&root,e.path()));}}
        let priority=match (&resource_cfg,&cfg_dir){
            (Some(cfg),Some(dir))=>package_priority(cfg,dir,e.path()),
            _=>None
        };
        let covered=resource_cfg.is_none() || priority.is_some();
        if !covered{stats.resource_cfg_uncovered+=1;}
        let relative=rel(&root,e.path());
        coverage.push(CoverageEntry{
            path:e.path().to_string_lossy().to_string(),
            depth:Path::new(&relative).components().count().saturating_sub(1),
            relative_path:relative,
            covered,
            priority:priority.as_ref().map(|p|p.priority),
            rule:priority.map(|p|p.rule),
        });
    }
    coverage.sort_by_key(|e|e.relative_path.to_ascii_lowercase());
    let empty=empty_dirs(&root);stats.empty_folders=empty.len();
    let outside=outside_packages(&root);stats.packages_outside_root=outside.len();
    Ok(ModsHealthReport{
        root:root.to_string_lossy().to_string(),stats,empty_folders:empty,
        unreadable_packages:unreadable,outside_packages:outside,coverage,resource_cfg
    })
}
