use crate::{restore_history::list_restore_history,structure_manager::list_manual_operations,snapshots::list_snapshots};
use chrono::{DateTime, Local};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct OperationHistoryItem{
    pub timestamp:String,
    pub kind:String,
    pub title:String,
    pub source:Option<String>,
    pub destination:Option<String>,
    pub status:String,
}
#[tauri::command]
pub fn get_operation_history(folder:String)->Result<Vec<OperationHistoryItem>,String>{
    let mut out=Vec::new();
    for record in list_manual_operations(folder.clone())?{
        out.push(OperationHistoryItem{
            timestamp:record.created_at.clone(),kind:"manual".into(),
            title:record.operation.clone(),source:record.source_relative_path.clone(),
            destination:record.destination_relative_path.clone(),status:"recorded".into()
        });
    }
    for manifest in list_restore_history(folder.clone())?{
        out.push(OperationHistoryItem{
            timestamp:manifest.created_at.clone(),kind:"restore_manifest".into(),
            title:manifest.file_name.clone(),source:manifest.root.clone(),
            destination:Some(manifest.path.clone()),status:manifest.status.clone()
        });
    }
    for snapshot in list_snapshots(folder.clone())?{
        out.push(OperationHistoryItem{
            timestamp:snapshot.created_at.clone(),kind:"snapshot".into(),
            title:format!("Snapshot {}",snapshot.id),source:Some(snapshot.root.clone()),
            destination:None,status:format!("{} packages",snapshot.entries.len())
        });
    }
    let root=PathBuf::from(folder.trim()).canonicalize().map_err(|e|format!("Could not resolve root: {e}"))?;
    let quarantine_dir=root.parent().unwrap_or(&root).join("S3CC Organizer").join("Quarantine Manifests");
    if quarantine_dir.is_dir(){
        for entry in std::fs::read_dir(&quarantine_dir).map_err(|e|e.to_string())?.filter_map(Result::ok){
            if !entry.path().is_file(){continue;}
            let timestamp=entry.metadata().ok().and_then(|m|m.modified().ok())
                .map(|time|DateTime::<Local>::from(time).to_rfc3339())
                .unwrap_or_default();
            let status=std::fs::read_to_string(entry.path()).ok()
                .and_then(|text|serde_json::from_str::<serde_json::Value>(&text).ok())
                .and_then(|value|value.get("status").and_then(|v|v.as_str()).map(str::to_string))
                .unwrap_or_else(||"recorded".into());
            out.push(OperationHistoryItem{
                timestamp,kind:"quarantine".into(),title:entry.file_name().to_string_lossy().to_string(),
                source:Some(root.to_string_lossy().to_string()),
                destination:Some(entry.path().to_string_lossy().to_string()),status
            });
        }
    }
    let inbox_dir=root.parent().unwrap_or(&root).join("S3CC Organizer").join("Inbox Manifests");
    if inbox_dir.is_dir(){
        for entry in std::fs::read_dir(inbox_dir).map_err(|e|e.to_string())?.filter_map(Result::ok){
            if !entry.path().is_file(){continue;}
            let timestamp=entry.metadata().ok().and_then(|m|m.modified().ok())
                .map(|time|DateTime::<Local>::from(time).to_rfc3339())
                .unwrap_or_default();
            out.push(OperationHistoryItem{
                timestamp,kind:"inbox_import".into(),title:entry.file_name().to_string_lossy().to_string(),
                source:None,destination:Some(entry.path().to_string_lossy().to_string()),status:"imported".into()
            });
        }
    }
    out.sort_by(|a,b|b.timestamp.cmp(&a.timestamp));
    Ok(out)
}
