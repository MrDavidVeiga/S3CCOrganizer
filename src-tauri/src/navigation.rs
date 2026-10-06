use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn existing_path(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw.trim());
    if raw.trim().is_empty() {
        return Err("No path was provided.".to_string());
    }
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    Ok(path)
}

#[cfg(target_os = "windows")]
fn open_directory_impl(path: &Path) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn reveal_file_impl(path: &Path) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg("/select,")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not reveal {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_directory_impl(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn reveal_file_impl(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not reveal {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn open_directory_impl(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn reveal_file_impl(path: &Path) -> Result<(), String> {
    let parent = path.parent().unwrap_or(path);
    open_directory_impl(parent)
}

#[tauri::command]
pub fn open_directory(path: String) -> Result<(), String> {
    let path = existing_path(&path)?;
    let directory = if path.is_dir() {
        path
    } else {
        path.parent()
            .ok_or_else(|| "Path has no parent directory.".to_string())?
            .to_path_buf()
    };
    open_directory_impl(&directory)
}

#[tauri::command]
pub fn reveal_path(path: String) -> Result<(), String> {
    let path = existing_path(&path)?;
    if path.is_dir() {
        open_directory_impl(&path)
    } else {
        reveal_file_impl(&path)
    }
}

fn valid_external_url(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err("Only complete HTTP/HTTPS URLs can be opened.".to_string());
    }
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(value.to_string())
    } else if lower.starts_with("www.") {
        Ok(format!("https://{value}"))
    } else {
        Err("Only complete HTTP/HTTPS URLs can be opened.".to_string())
    }
}

#[cfg(target_os = "windows")]
fn open_url_impl(url: &str) -> Result<(), String> {
    Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map_err(|error| format!("Could not open URL: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_url_impl(url: &str) -> Result<(), String> {
    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|error| format!("Could not open URL: {error}"))?;
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn open_url_impl(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|error| format!("Could not open URL: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let url = valid_external_url(&url)?;
    open_url_impl(&url)
}
