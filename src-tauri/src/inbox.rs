use crate::{i18n::AppLanguage,manifest::sha256_file,scanner::{scan_packages_core,ScanResult},workspace::ensure_writable};
use chrono::Local;
use serde::Serialize;
use std::{fs,path::{Path,PathBuf}};

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct InboxPlanItem{
    pub source_path:String,
    pub relative_path:String,
    pub destination_path:String,
    pub sha256:String,
    pub size:u64,
    pub status:String,
}
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct InboxImportPlan{
    pub source_root:String,
    pub mods_root:String,
    pub destination_root:String,
    pub items:Vec<InboxPlanItem>,
    pub ready:usize,
    pub blocked:usize,
    pub can_execute:bool,
}
#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct InboxImportResult{
    pub imported:usize,
    pub manifest_path:String,
    pub destination_root:String,
}
fn canonical_dir(raw:&str)->Result<PathBuf,String>{
    let p=PathBuf::from(raw.trim()).canonicalize().map_err(|e|format!("Could not resolve folder: {e}"))?;
    if !p.is_dir(){return Err("Selected path is not a directory.".into());}Ok(p)
}
fn is_package(path:&Path)->bool{path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("package")).unwrap_or(false)}
#[tauri::command]
pub fn scan_inbox(source_folder:String,language:AppLanguage)->Result<ScanResult,String>{
    let source=canonical_dir(&source_folder)?;
    scan_packages_core(source.to_string_lossy().to_string(),language,None)
}
#[tauri::command]
pub fn build_inbox_import_plan(folder:String,source_folder:String,selected_paths:Vec<String>)->Result<InboxImportPlan,String>{
    let root=canonical_dir(&folder)?;let source_root=canonical_dir(&source_folder)?;
    let destination_root=root.join("New CC");
    let mut items=Vec::new();let mut ready=0;let mut blocked=0;
    for raw in selected_paths{
        let source=PathBuf::from(&raw).canonicalize().map_err(|e|format!("Could not resolve inbox file: {e}"))?;
        if !source.starts_with(&source_root)||!source.is_file()||!is_package(&source){blocked+=1;continue;}
        let rel=source.strip_prefix(&source_root).unwrap_or(&source);
        let destination=destination_root.join(rel);
        let (sha,size)=sha256_file(&source).map_err(|e|format!("Could not hash {}: {e}",source.display()))?;
        let status=if destination.exists(){"collision"}else{"ready"}.to_string();
        if status=="ready"{ready+=1}else{blocked+=1}
        items.push(InboxPlanItem{
            source_path:source.to_string_lossy().to_string(),
            relative_path:rel.to_string_lossy().replace('/',"\\"),
            destination_path:destination.to_string_lossy().to_string(),sha256:sha,size,status
        });
    }
    Ok(InboxImportPlan{
        source_root:source_root.to_string_lossy().to_string(),mods_root:root.to_string_lossy().to_string(),
        destination_root:destination_root.to_string_lossy().to_string(),can_execute:ready>0&&blocked==0,items,ready,blocked
    })
}
#[tauri::command]
pub fn execute_inbox_import(folder:String,source_folder:String,selected_paths:Vec<String>)->Result<InboxImportResult,String>{
    let root=canonical_dir(&folder)?;ensure_writable(&root)?;
    let plan=build_inbox_import_plan(folder.clone(),source_folder,selected_paths)?;
    if !plan.can_execute{return Err("Inbox import is blocked by the preflight plan.".into());}
    let destination_root=PathBuf::from(&plan.destination_root);
    let mut copied=Vec::<PathBuf>::new();
    for item in &plan.items{
        let src=PathBuf::from(&item.source_path);let dst=PathBuf::from(&item.destination_path);
        if dst.exists(){for p in copied.iter().rev(){let _=fs::remove_file(p);}return Err(format!("Destination appeared during import: {}",dst.display()));}
        let parent=dst.parent().ok_or_else(||"Import destination has no parent.".to_string())?;
        fs::create_dir_all(parent).map_err(|e|format!("Could not create import folder: {e}"))?;
        fs::copy(&src,&dst).map_err(|e|format!("Could not copy {}: {e}",src.display()))?;
        let (hash,size)=sha256_file(&dst).map_err(|e|format!("Could not verify imported file: {e}"))?;
        if hash!=item.sha256||size!=item.size{
            let _=fs::remove_file(&dst);for p in copied.iter().rev(){let _=fs::remove_file(p);}
            return Err(format!("Imported copy failed verification: {}",dst.display()));
        }
        copied.push(dst);
    }
    let reports=root.parent().unwrap_or(&root).join("S3CC Organizer").join("Inbox Manifests");
    fs::create_dir_all(&reports).map_err(|e|e.to_string())?;
    let path=reports.join(format!("Inbox-Import-{}.json",Local::now().format("%Y%m%d-%H%M%S")));
    let data=serde_json::to_vec_pretty(&plan).map_err(|e|e.to_string())?;
    fs::write(&path,data).map_err(|e|e.to_string())?;
    Ok(InboxImportResult{imported:copied.len(),manifest_path:path.to_string_lossy().to_string(),destination_root:destination_root.to_string_lossy().to_string()})
}
