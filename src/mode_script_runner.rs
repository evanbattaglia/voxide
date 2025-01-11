use crate::command_wrapper::CommandWrapper;
use crate::config::Mode;
use color_eyre::eyre::{eyre, Result};
use log::debug;
use std::{iter, path::Path};
use tempfile::NamedTempFile;

/// Runs script defined in a mode and returns the files to be opened (or a quickfix file)
pub enum ModeScriptRunner {
    Quickfix {
        // Used to keep the file open until the end of the program
        _named_file: NamedTempFile,
    },
    FilesList {
        newline_separated_files: String,
    },
}

impl ModeScriptRunner {
    pub fn new(mode: &Mode, argv: impl Iterator<Item = String>) -> Result<ModeScriptRunner> {
        let script = mode
            .script
            .as_ref()
            .ok_or_else(|| eyre!("No script found for mode {:?}", mode))?;
        let uses_tempfile = mode.script_uses_tempfile.unwrap_or(false);
        let quickfix = mode.quickfix.unwrap_or(false);

        let mut command = CommandWrapper::new(script)?;
        debug!("uses_tempfile={uses_tempfile}, quickfix={quickfix}");
        let res = match (uses_tempfile, quickfix) {
            (false, true) => {
                let tempfile = NamedTempFile::new()?;
                command.args(argv);
                command
                    .thru_filter(&mode.filter)?
                    .stdout_to_tempfile(&tempfile)?
                    .run()?;
                ModeScriptRunner::Quickfix {
                    _named_file: tempfile,
                }
            }
            (false, false) => {
                // get output and split by newline and use that as filenames to pass to editor
                command.args(argv);
                let newline_separated_files =
                    command.thru_filter(&mode.filter)?.run_get_output()?;
                ModeScriptRunner::FilesList {
                    newline_separated_files,
                }
            }
            (true, false) => {
                // pass tempfile name into script, then read tempfile to get filenames to  pass to
                // editor
                let tempfile = NamedTempFile::new()?;
                command.arg(tempfile.path());
                command.args(argv);
                command.run()?;
                let newline_separated_files = std::fs::read_to_string(tempfile.path())?;
                ModeScriptRunner::FilesList {
                    newline_separated_files,
                }
            }
            (true, true) => {
                // pass tempfile name into script, and pass tempfile name to editor as quickfix
                // file
                let tempfile = NamedTempFile::new()?;
                command.arg(tempfile.path());
                command.args(argv);
                command.run()?;
                ModeScriptRunner::Quickfix {
                    _named_file: tempfile,
                }
            }
        };

        Ok(res)
    }

    pub fn files_iter(&self) -> Box<dyn Iterator<Item = &Path> + '_> {
        match self {
            ModeScriptRunner::Quickfix { _named_file } => Box::new(iter::once(_named_file.path())),
            ModeScriptRunner::FilesList {
                newline_separated_files,
            } => Box::new(
                newline_separated_files
                    .split("\n")
                    .filter(|x| !x.is_empty())
                    .map(Path::new),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Mode, Script};

    fn mkmode(cmd_with_args: &[&str]) -> Mode {
        let cmd_vec: Vec<_> = cmd_with_args.iter().map(|x| x.to_string()).collect();
        let script = Some(Script::CommandWithArgs(cmd_vec));
        Mode {
            name: "foo".to_string(),
            desc: "foo".to_string(),
            script,
            cmd: Some("nvim".to_string()),
            quickfix_cmd: Some("nvim -q".to_string()),
            dir_cmd: Some("yazi".to_string()),
            script_uses_tempfile: Some(false),
            quickfix: Some(false),
            filter: None,
        }
    }

    #[test]
    fn test_script_stdout_to_files() -> Result<()> {
        let mode = mkmode(&["bash", "-c", "echo $0"]);
        let msr = ModeScriptRunner::new(&mode, vec!["ok".to_owned()].into_iter())?;
        let files = msr
            .files_iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files, vec!["ok"]);
        Ok(())
    }

    #[test]
    fn test_script_tempfile_to_files() -> Result<()> {
        let mode = mkmode(&["sh", "-c", "echo $1 > $0"]);
        let mode = Mode {
            script_uses_tempfile: Some(true),
            ..mode
        };
        let msr = ModeScriptRunner::new(&mode, vec!["hello".to_owned()].into_iter())?;
        let files = msr
            .files_iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files, vec!["hello"]);
        Ok(())
    }

    #[test]
    fn test_script_stdout_to_quickfix() -> Result<()> {
        let mode = mkmode(&["bash", "-c", "echo $0"]);
        let mode = Mode {
            quickfix: Some(true),
            ..mode
        };
        let msr = ModeScriptRunner::new(&mode, vec!["ok:123:foo".to_owned()].into_iter())?;
        let files = msr
            .files_iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        assert_eq!(
            std::fs::read_to_string(files.first().unwrap())?,
            "ok:123:foo\n"
        );
        Ok(())
    }

    #[test]
    fn test_script_tempfile_to_quickfix() -> Result<()> {
        let mode = mkmode(&["bash", "-c", "echo $1 > $0"]);
        let mode = Mode {
            quickfix: Some(true),
            script_uses_tempfile: Some(true),
            ..mode
        };
        let msr = ModeScriptRunner::new(&mode, vec!["ok:123:foo".to_owned()].into_iter())?;
        let files = msr
            .files_iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        assert_eq!(
            std::fs::read_to_string(files.first().unwrap())?,
            "ok:123:foo\n"
        );
        Ok(())
    }
}
