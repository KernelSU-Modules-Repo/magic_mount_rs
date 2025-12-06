use std::{fs, path::Path};

use serde::Serialize;

use crate::defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME};

#[derive(Debug, Serialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub disabled: bool,
    pub skip: bool,
}

fn read_prop<S>(vaule: S, key: &str) -> Option<String>
where
    S: AsRef<str> + Into<String>,
{
    for line in vaule.as_ref().lines() {
        if line.starts_with(key)
            && let Some((_, value)) = line.split_once('=')
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// Scans for modules that will be actually mounted by `magic_mount`.
/// Filters out modules that:
/// 1. Do not have a `system` directory.
/// 2. Are disabled or removed.
/// 3. Have the `skip_mount` flag.
pub fn scan_modules<P>(module_dir: P) -> Vec<ModuleInfo>
where
    P: AsRef<Path>,
{
    let mut modules = Vec::new();

    if let Ok(entries) = module_dir.as_ref().read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if !path.join("module.prop").exists() {
                continue;
            }

            if !path.join("system").is_dir() {
                continue;
            }

            let disabled =
                path.join(DISABLE_FILE_NAME).exists() || path.join(REMOVE_FILE_NAME).exists();
            let skip = path.join(SKIP_MOUNT_FILE_NAME).exists();
            if disabled || skip {
                continue;
            }

            let id = entry.file_name().to_string_lossy().to_string();
            let prop_path = path.join("module.prop");

            let prop = fs::read_to_string(prop_path).unwrap_or(String::new());
            let name = read_prop(&prop, "name").unwrap_or_else(|| id.clone());
            let version = read_prop(&prop, "version").unwrap_or_else(|| "unknown".to_string());
            let description =
                read_prop(&prop, "description").unwrap_or_else(|| "unknown".to_string());

            modules.push(ModuleInfo {
                id,
                name,
                version,
                description,
                disabled,
                skip,
            });
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}
