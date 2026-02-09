use std::fs;
use std::path::{Path, PathBuf};

pub fn parse_time(s: &str) -> i64 {
    let mut total_seconds = 0;
    let parts = s.split_whitespace();
    for part in parts {
        if let Some(val) = part.strip_suffix("day") {
            if let Ok(n) = val.parse::<i64>() {
                total_seconds += n * 24 * 3600;
            }
        } else if let Some(val) = part.strip_suffix("days") {
            if let Ok(n) = val.parse::<i64>() {
                total_seconds += n * 24 * 3600;
            }
        } else if let Some(val) = part.strip_suffix("h") {
            if let Ok(n) = val.parse::<i64>() {
                total_seconds += n * 3600;
            }
        } else if let Some(val) = part.strip_suffix("m") {
            if let Ok(n) = val.parse::<i64>() {
                total_seconds += n * 60;
            }
        } else if let Some(val) = part.strip_suffix("s") {
            if let Ok(n) = val.parse::<i64>() {
                total_seconds += n;
            }
        }
    }
    total_seconds
}

pub fn get_zellij_config_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        let mut path = PathBuf::from(h);
        path.push(".config");
        path.push("zellij");
        path
    })
}

pub fn list_kdl_files(dir: &Path) -> Vec<String> {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut files: Vec<String> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "kdl") {
                    Some(path.file_name()?.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();
        files.sort();
        files
    } else {
        Vec::new()
    }
}
