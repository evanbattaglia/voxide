use crate::path_util::config_dir;
use color_eyre::eyre::{eyre, OptionExt, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::DirEntry;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub modes: HashMap<char, Mode>,
    pub transforms: Vec<(String, String)>,
    pub default_mode: Mode,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mode {
    pub name: String,
    pub desc: String,
    pub script: Option<Script>,
    pub filter: Option<Script>,
    pub script_uses_tempfile: Option<bool>,
    // TODO: make cmd, etc. like script probably where it can be an array
    pub cmd: Option<String>,
    pub quickfix_cmd: Option<String>,
    pub dir_cmd: Option<String>,
    pub quickfix: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Script {
    Command(String),
    CommandWithArgs(Vec<String>),
}

impl Mode {
    pub fn cmd_for_isdir_and_qf(&self, is_dir: bool, is_quickfix: bool) -> Result<&String> {
        match (is_dir, is_quickfix) {
            (_isdir, true) => self
                .quickfix_cmd
                .as_ref()
                .ok_or_eyre("No quickfix_cmd found in combined modes"),
            (true, _is_qf) => self
                .dir_cmd
                .as_ref()
                .or(self.cmd.as_ref())
                .ok_or_eyre("No dir_cmd or cmd found in combined modes"),
            _ => self
                .cmd
                .as_ref()
                .ok_or_eyre("No cmd found in combined modes"),
        }
    }

    // Merge other into self, preferring self's values
    fn reverse_merge(&mut self, other: &Mode) {
        if let (None, Some(other_script)) = (&self.script, &other.script) {
            // there can be a max of script/script_with_tempfile
            self.script = Some(other_script.clone());
            self.quickfix = other.quickfix;
            self.script_uses_tempfile = other.script_uses_tempfile;
        }

        if let (None, Some(cmd)) = (&self.cmd, &other.cmd) {
            self.cmd = Some(cmd.clone());
        }

        if let (None, Some(dir_cmd)) = (&self.dir_cmd, &other.dir_cmd) {
            self.dir_cmd = Some(dir_cmd.clone());
        }

        if let (None, Some(quickfix_cmd)) = (&self.quickfix_cmd, &other.quickfix_cmd) {
            self.quickfix_cmd = Some(quickfix_cmd.clone());
        }

        if let (None, Some(filter)) = (&self.filter, &other.filter) {
            self.filter = Some(filter.clone());
        }
    }
}

impl AppConfig {
    pub fn get_merged_mode(&self, letters: Option<&str>) -> Result<Mode> {
        let default = std::iter::once(&self.default_mode);
        let lets = letters
            .unwrap_or("")
            .chars()
            .map(|c| self.get_mode(c))
            .collect::<Result<Vec<&Mode>>>()?;
        let mut combined = default.chain(lets).rev();
        let mut aggregate_mode = combined
            .next()
            .ok_or_eyre("Internal error, no iterator for default mode")?
            .to_owned();
        for mode in combined {
            aggregate_mode.reverse_merge(mode);
        }

        debug!("Combined merged mode is: {:?}", &aggregate_mode);
        Ok(aggregate_mode)
    }

    pub fn get_mode(&self, mode_letter: char) -> Result<&Mode> {
        self.modes
            .get(&mode_letter)
            .ok_or_else(|| eyre!("No mode found for {mode_letter}"))
    }
}

fn is_entry_of_toml_file(entry: &DirEntry) -> bool {
    match entry.path().extension() {
        Some(x) => x == "toml",
        _ => false,
    }
}

pub fn get_config() -> Result<AppConfig> {
    let mut figment = Figment::new();
    let path = config_dir()?;

    let mut entries = std::fs::read_dir(path.clone())?
        .filter(|entry| match entry {
            Err(_) => true,
            Ok(entry) => is_entry_of_toml_file(entry),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if entries.is_empty() {
        return Err(eyre!("No config files found in {}, run with --init to install default config and scripts or --help for more info", path.display()));
    }
    entries.sort_by_key(|a| a.path());

    for entry in entries {
        figment = figment.merge(Toml::file(entry.path()));
    }
    Ok(figment.extract()?)
    // .filter_map(|entry| entry?.path()).filter(|path| path.extension()? == "toml");
    // let config: AppConfig = Figment::new().merge(Toml::file(path.clone())).extract()?;
    // let mut tmp = Figment::new();
    // tmp = tmp.merge(Toml::file(path.clone()));
    // tmp = tmp.merge(Toml::file(path.clone()));
    // let cfg = tmp.extract::<AppConfig>()?;
}
