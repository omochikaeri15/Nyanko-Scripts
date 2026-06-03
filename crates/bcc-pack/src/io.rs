use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

pub fn get_local_dir() -> PathBuf {
    std::env::current_exe()
        .map(|path| path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf())
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn save_local<T: Serialize>(filename: &str, data: &T) {
    let mut path = get_local_dir();
    path.push(filename);

    if let Ok(json) = serde_json::to_string_pretty(data) {
        let tmp_path = path.with_extension("tmp");
        if fs::write(&tmp_path, json).is_ok() {
            let _ = fs::rename(&tmp_path, &path);
        }
    }
}

pub fn load_local<T: DeserializeOwned>(filename: &str) -> Option<T> {
    let mut path = get_local_dir();
    path.push(filename);

    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<T>(&data) {
                return Some(parsed);
            }
        }
    }
    None
}