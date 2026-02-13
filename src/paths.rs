use std::path::{Path, PathBuf};

const BASE_DIR_ENV: &str = "COMMAND_GENERATOR_DIR";

pub fn base_dir() -> PathBuf {
    if let Ok(value) = std::env::var(BASE_DIR_ENV)
        && let Some(path) = normalize_dir(&value)
    {
        return path;
    }
    home_join(".command-generator").unwrap_or_else(|| PathBuf::from(".command-generator"))
}

pub fn cache_dir() -> PathBuf {
    base_dir().join(".cache")
}

pub fn sessions_dir() -> PathBuf {
    base_dir().join("sessions")
}

pub fn ensure_dirs() -> anyhow::Result<()> {
    std::fs::create_dir_all(cache_dir())?;
    std::fs::create_dir_all(sessions_dir())?;
    Ok(())
}

fn home_join(suffix: &str) -> Option<PathBuf> {
    std::env::var("HOME").ok().and_then(|home| {
        let home = home.trim();
        if home.is_empty() {
            None
        } else {
            Some(Path::new(home).join(suffix))
        }
    })
}

fn normalize_dir(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let expanded = expand_tilde(trimmed);
    Some(normalize_path(PathBuf::from(expanded)))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        normalized.push(component.as_os_str());
    }
    normalized
}

fn expand_tilde(value: &str) -> String {
    if (value == "~" || value.starts_with("~/"))
        && let Ok(home) = std::env::var("HOME")
    {
        let home = home.trim();
        if !home.is_empty() {
            if value == "~" {
                return home.to_string();
            }
            return format!("{}{}", home, &value[1..]);
        }
    }
    value.to_string()
}
