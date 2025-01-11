use color_eyre::eyre::{eyre, Result};
use directories::UserDirs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/voxide";
const CONFIG_SCRIPTS_DIR: &str = ".config/voxide/scripts";
const SHARE_SCRIPTS_DIR: &str = ".local/share/voxide/scripts";

pub fn home_dir() -> Result<PathBuf> {
    Ok(UserDirs::new()
        .ok_or_else(|| eyre!("Could not find home directory"))?
        .home_dir()
        .to_path_buf())
}

pub fn config_dir() -> Result<PathBuf> {
    home_dir().map(|path| path.join(CONFIG_DIR))
}

pub fn share_scripts_dir() -> Result<PathBuf> {
    home_dir().map(|path| path.join(SHARE_SCRIPTS_DIR))
}

pub fn config_scripts_dir() -> Result<PathBuf> {
    home_dir().map(|path| path.join(CONFIG_SCRIPTS_DIR))
}

fn path_from_home_if_exists(rel_path: &str) -> Option<String> {
    home_dir()
        .ok()
        .map(|path| path.join(rel_path))
        .filter(|p| p.exists())
        .and_then(|p| p.to_str().map(str::to_owned))
}

/// Gets PATH but prepends ~/.config/voxide/scripts and ~/.local/share/voxide/scripts
pub fn path_with_prepended_script_paths() -> String {
    let path = std::env::var("PATH").ok();
    let path = match path.as_deref() {
        Some("") => None,
        _ => path,
    };
    if path.is_none() {
        eprintln!("Warning: PATH not set or invalid");
    }

    let conf_scripts = path_from_home_if_exists(CONFIG_SCRIPTS_DIR);
    let share_scripts = path_from_home_if_exists(SHARE_SCRIPTS_DIR);

    if conf_scripts.is_none() && share_scripts.is_none() {
        eprintln!("Warning: No scripts directory found in config or share directories. To install default scripts/config run with --init; run with --help for more info.");
    }

    let res = [conf_scripts, share_scripts, path]
        .iter()
        .filter_map(|x| x.as_deref())
        .collect::<Vec<&str>>()
        .join(":");
    res
}
