use crate::path_util;
use color_eyre::eyre::{Result, WrapErr};
use include_dir::{include_dir, Dir};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

static SCRIPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/scripts");
static EXAMPLE_CONFIG_FILE: &str = include_str!("../configs/000_EXAMPLE.toml");

const EXAMPLE_TOML_FILENAME: &str = "000_EXAMPLE.toml";

fn mkdir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).wrap_err("Failed to create directory")
}

fn install(src: &str, dest: &Path) -> Result<()> {
    fs::write(dest, src).wrap_err("Failed to write file")
}

fn init_config() -> Result<()> {
    let config_dir = path_util::config_dir()?;
    let example_toml_path = config_dir.join(EXAMPLE_TOML_FILENAME);

    eprintln!("### voxide config initialization ###");
    match (config_dir.exists(), example_toml_path.exists()) {
        (false, _) => {
            eprintln!(
                "No config directory found, creating and installing {}",
                example_toml_path.display()
            );
            mkdir(&config_dir)?;
            install(EXAMPLE_CONFIG_FILE, &example_toml_path)?;
        }
        (true, false) => {
            eprintln!(
                "WARNING: config directory found, but adding: {}",
                example_toml_path.display()
            );
            install(EXAMPLE_CONFIG_FILE, &example_toml_path)?;
        }
        (true, true) => {
            eprintln!(
                "WARNING: existing example config found, SKIPPING: {}",
                example_toml_path.display()
            );
            eprintln!("To install a new example config, rename or remove this file.");
        }
    }

    eprintln!("Please review the configs in: {}", config_dir.display());
    eprintln!("Note that configs are loaded in order by sorted filename.");
    eprintln!();
    Ok(())
}

fn init_scripts() -> Result<()> {
    let share_scripts_dir = path_util::share_scripts_dir()?;
    let config_scripts_dir = path_util::config_scripts_dir()?;

    eprintln!("### voxide scripts initialization ###");
    if share_scripts_dir.exists() {
        eprintln!(
            "Scripts directory exists, SKIPPING: {}",
            share_scripts_dir.display()
        );
        eprintln!("To install new scripts, move or remove this directory.");
        eprintln!("If there are any custom scripts there, consider moving to config directory (which takes precendence): {}", config_scripts_dir.display());
        return Ok(());
    } else {
        eprintln!(
            "Installing scripts in directory: {}",
            share_scripts_dir.display()
        );
        eprintln!("If adding/overwriting any custom scripts, it's recommended to add them to the alternative directory (which takes precedence): {}", config_scripts_dir.display());
        mkdir(&share_scripts_dir)?;
        for entry in SCRIPTS_DIR.files() {
            let dest = share_scripts_dir.join(entry.path());
            install(entry.contents_utf8().unwrap(), &dest)?;
            // chmod u+x dest:
            fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
        }
        eprintln!("Installed {} scripts", SCRIPTS_DIR.files().count());
    }

    Ok(())
}

pub fn init() -> Result<()> {
    init_config()?;
    init_scripts()?;

    Ok(())
}
