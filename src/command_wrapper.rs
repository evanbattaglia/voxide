use crate::{config::Script, path_util};
use color_eyre::eyre::{eyre, OptionExt, Result, WrapErr};
use log::debug;
use std::{
    fs::File,
    path::Path,
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

/// Wrapper around std::process::Command to make it easier to use (supporting running scripts from
/// a string, through a tempfile) and to run commands as specified in config.
/// Many of the functions pass through to std::process::Command functions.
#[derive(Debug)]
pub struct CommandWrapper {
    command: std::process::Command,
    // Used to keep the file open until command is started. Only used when the script is embedded
    // in the config file or using a builtin scrpit (built-in to voxide binary). When filters are
    // used, multipl tempfiles may be needed.
    _tempfiles: Vec<tempfile::TempPath>,
}

impl CommandWrapper {
    pub fn new(script: &Script) -> Result<CommandWrapper> {
        match script {
            Script::Command(cmd) => Self::new_from_script_path(cmd),
            Script::CommandWithArgs(cmd_and_args) => {
                let mut res = Self::new_from_script_path(cmd_and_args[0].as_ref())?;
                for arg in &cmd_and_args[1..] {
                    res.command.arg(arg);
                }
                Ok(res)
            }
        }
    }

    pub fn new_from_script_path(cmd: &str) -> Result<CommandWrapper> {
        let mut command = Self::std_process_command_new(cmd.as_ref());
        command.env("PATH", path_util::path_with_prepended_script_paths());
        Ok(Self {
            command,
            _tempfiles: vec![],
        })
    }

    /// Calls std::process:Command.new(), but fixes issue on Termux where running scripts with shebangs
    /// don't work (because /usr/bin/env doesn't exist) by wrapping script in: sh -c '"$0" "$@"'
    fn std_process_command_new(command: &Path) -> Command {
        use std::env::consts::OS;
        if OS == "android" {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg("\"$0\" \"$@\"").arg(command);
            cmd
        } else {
            Command::new(command)
        }
    }

    /// Pass through to Command
    pub fn arg<T>(&mut self, arg: T) -> &mut Self
    where
        T: AsRef<std::ffi::OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<T>(&mut self, args: impl IntoIterator<Item = T>) -> &mut Self
    where
        T: AsRef<std::ffi::OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn stdout_to_tempfile(&mut self, tempfile: &NamedTempFile) -> Result<&mut Self> {
        self.command
            .stdout(Stdio::from(File::create(tempfile.path())?));
        Ok(self)
    }

    // Pass this command's stdout to another command's stdin, and return wrapper for the second
    pub fn thru_filter(mut self, filter: &Option<Script>) -> Result<Self> {
        if let Some(ref filter) = filter {
            debug!("Running command as filter: {:?}", self.command);
            let script_output = self.command.stdout(Stdio::piped()).spawn()?;
            let mut filter_wrapper = Self::new(filter)?;
            filter_wrapper.command.stdin(
                script_output
                    .stdout
                    .ok_or_eyre("failed to open stdout in pipe")?,
            );

            // need to make all temp files last until the last command is done,
            // so need to keep track of all _tempfiles
            self._tempfiles.append(&mut filter_wrapper._tempfiles);
            filter_wrapper._tempfiles = self._tempfiles;
            return Ok(filter_wrapper);
        }

        Ok(self)
    }

    pub fn run_get_output(&mut self) -> Result<String> {
        debug!("Running command and getting output: {:?}", self.command);
        let spawned = self
            .command
            .stdout(Stdio::piped())
            .spawn()
            .wrap_err("Failed to spawn script")?;

        let res = spawned.wait_with_output()?;
        if !res.status.success() {
            Err(eyre!("Script exited with status {}", res.status))
        } else {
            Ok(String::from_utf8(res.stdout)?)
        }
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Running command: {:?}", self.command);
        let mut spawned = self.command.spawn().wrap_err("Failed to spawn script")?;
        let res = spawned
            .wait()?
            .code()
            .ok_or(eyre!("Process killed by signal"))?;

        if res != 0 {
            Err(eyre!("Script exited with status {res}"))
        } else {
            Ok(())
        }
    }
}
