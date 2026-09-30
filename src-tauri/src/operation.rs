use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

pub const CANCELLED_ERROR: &str = "__S3CC_OPERATION_CANCELLED__";

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OperationStatus {
    pub kind: String,
    pub running: bool,
    pub cancel_requested: bool,
    pub processed: usize,
    pub total: usize,
    pub current: Option<String>,
    pub phase: String,
    pub message: Option<String>,
}

fn registry() -> &'static Mutex<HashMap<String, OperationStatus>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, OperationStatus>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn with_status<F>(kind: &str, mut f: F)
where
    F: FnMut(&mut OperationStatus),
{
    let mut map = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let status = map.entry(kind.to_string()).or_insert_with(|| OperationStatus {
        kind: kind.to_string(),
        ..OperationStatus::default()
    });
    f(status);
}

pub fn begin(kind: &str, phase: &str) {
    with_status(kind, |status| {
        *status = OperationStatus {
            kind: kind.to_string(),
            running: true,
            cancel_requested: false,
            processed: 0,
            total: 0,
            current: None,
            phase: phase.to_string(),
            message: None,
        };
    });
}

pub fn set_total(kind: &str, total: usize) {
    with_status(kind, |status| status.total = total);
}

pub fn update(kind: &str, processed: usize, current: Option<String>, phase: &str) {
    with_status(kind, |status| {
        status.processed = processed;
        status.current = current.clone();
        status.phase = phase.to_string();
    });
}

pub fn set_message(kind: &str, message: Option<String>) {
    with_status(kind, |status| status.message = message.clone());
}

pub fn is_cancelled(kind: &str) -> bool {
    let map = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    map.get(kind)
        .map(|status| status.cancel_requested)
        .unwrap_or(false)
}

pub fn finish(kind: &str, phase: &str, message: Option<String>) {
    with_status(kind, |status| {
        status.running = false;
        status.current = None;
        status.phase = phase.to_string();
        status.message = message.clone();
    });
}

pub fn mark_cancelled(kind: &str) {
    with_status(kind, |status| {
        status.running = false;
        status.cancel_requested = true;
        status.current = None;
        status.phase = "cancelled".to_string();
        status.message = Some("Cancelled by user.".to_string());
    });
}

#[tauri::command]
pub fn get_operation_status(kind: String) -> OperationStatus {
    let map = registry().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    map.get(&kind).cloned().unwrap_or_else(|| OperationStatus {
        kind,
        phase: "idle".to_string(),
        ..OperationStatus::default()
    })
}

#[tauri::command]
pub fn cancel_operation(kind: String) -> bool {
    let mut changed = false;
    with_status(&kind, |status| {
        if status.running {
            status.cancel_requested = true;
            status.phase = "cancelling".to_string();
            status.message = Some("Cancellation requested.".to_string());
            changed = true;
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_scoped_by_operation_kind() {
        begin("scan-test", "starting");
        begin("duplicates-test", "starting");
        assert!(cancel_operation("scan-test".to_string()));
        assert!(is_cancelled("scan-test"));
        assert!(!is_cancelled("duplicates-test"));
        finish("duplicates-test", "complete", None);
    }
}
