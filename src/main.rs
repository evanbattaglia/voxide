mod command_wrapper;
mod config;
mod init_from_builtin_files;
mod mode_script_runner;
mod path_util;
mod transforms_applicator;

use crate::command_wrapper::CommandWrapper;
use crate::mode_script_runner::ModeScriptRunner;
use crate::transforms_applicator::TransformsApplicator;
use color_eyre::eyre::{OptionExt, Result, WrapErr};
use config::get_config;
use log::debug;
use std::cmp::Ordering;
use std::path::Path;

static README: &str = include_str!("../README.md");

// TODO BEFORE MORE BROAD ANNOUNCEMENT:
// - [ ] try prefixing with git repo... maybe as a transform? ['^', '$GITROOT'] etc. or maybe multiple filters? can transforms then just be another filter (voxide-try-regexes)? define script as voxide-try-regexes with the arguments!!!
// - [ ] line numbers -- make configurable
// - [ ] errors when config files conflict
// - [ ] convert ruby scripts to shell script when can
// - [ ] most important TODOs
// - [ ] more documentation
// - [ ] use Script for command too
// TODO AFTER RELEASE:
// - [ ] line numbers -- what to do if multipl files? auto make into quickfix?
// - [ ] tabry configs inside voxide config
// - [ ] todos, tests
// - [ ] inline scripts?
// - [ ] "locations" see todo?
// - [ ] transforms in modes? "config" could be implemented via

/// Add arguments (corresponding to files to open) to the Command, applying the transforms (as
/// defined in the config) if necessary.
fn add_file_args<'a>(
    output: &mut CommandWrapper,
    iter: impl Iterator<Item = &'a Path>,
    ta: Option<TransformsApplicator>,
) -> Result<()> {
    if let Some(ta) = ta {
        for arg in iter {
            if let Some(string_arg) = arg.to_str() {
                let (lineno, arg) = ta.apply_transforms(string_arg)?;
                output.arg(arg.as_ref());
                // TODO: make this configurable
                // TODO: doesn't work in vim if multiple files
                if let Some(lineno) = lineno {
                    output.arg(format!("+{}", lineno.0));
                }
            } else {
                // stderr println warning:
                eprintln!(
                    "Invalid unicode in path, cannot process transforms: {:?}",
                    arg
                );
                output.arg(arg);
            }
        }
    } else {
        output.args(iter);
    }

    Ok(())
}

fn run_editor<'a>(
    cmd: &str,
    iter: impl Iterator<Item = &'a Path>,
    ta: Option<TransformsApplicator>,
) -> Result<()> {
    // TODO real shell split, or just allokw array like script
    let mut cmd_iter = cmd.split_whitespace();
    let actual_cmd = cmd_iter.next().ok_or_eyre("Empty cmd")?;
    let mut output = CommandWrapper::new_from_script_path(actual_cmd)?;
    output.args(cmd_iter);

    add_file_args(&mut output, iter, ta)?;

    debug!("Running editor: {:?}", output);
    output.run()
}

fn help() -> Result<()> {
    let config_res = get_config();
    println!("voxide -- flexible, customizable file finder/opener");
    println!("voxide finds files and line numbers and opens them (via arguments to your");
    println!("editor or a quickfix file) using scripts/commands/transforms defined in the");
    println!("configuration file.");
    println!();
    println!("All invocations begin with the default (base) mode defined in the config file under");
    println!("`default_mode`. Any further modes (letters) in the first argument merge (layer) on");
    println!("top of this to determine the final mode's cmd, script, etc.");

    println!();
    println!("Run 'voxide --init' to initialize with a default config and scripts.");
    println!("Run 'voxide --readme' to print out the README.md file compiled into voxide.");
    println!("Run RUST_LOG=debug voxide... to show debugging info.");
    println!();
    println!("Usage: voxide [<mode letter(s)>] [<arguments to mode scripts> ...]");
    println!();
    match config_res {
        Ok(config) => {
            println!("Available modes:");
            let mut modes = config.modes.iter().collect::<Vec<(&char, &config::Mode)>>();
            modes.sort_by(|kv1, kv2| kv1.0.partial_cmp(kv2.0).unwrap_or(Ordering::Equal));
            for (k, v) in &modes {
                println!("{}:  {}", k, v.name);
                println!("    {}", v.desc);
            }
        }
        Err(_) => {
            println!("No valid config available; run with no arguments to get full error or run with --init to initialize with default config and scripts, or --help for more info.");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let mut args = std::env::args().skip(1);
    let first_arg = args.next();
    match first_arg.as_deref() {
        Some("--help") => return help(),
        Some("--init") => return init_from_builtin_files::init(),
        Some("--readme") => {
            println!("{}", README);
            return Ok(());
        }
        _ => (),
    };

    let config = get_config().wrap_err(
        "Invalid/missing config! Run with --init to install default config and scripts",
    )?;
    let mode = config.get_merged_mode(first_arg.as_deref())?;
    let mode_script_runner = ModeScriptRunner::new(&mode, args)?;

    let mut files_iter = mode_script_runner.files_iter().peekable();
    let first_is_dir = files_iter.peek().map_or(false, |x| Path::new(x).is_dir());
    let is_quickfix = mode.quickfix.unwrap_or(false);
    let ta = if is_quickfix {
        None
    } else {
        Some(TransformsApplicator::new(&config.transforms))
    };

    let cmd = mode.cmd_for_isdir_and_qf(first_is_dir, is_quickfix)?;

    run_editor(cmd, files_iter, ta)?;

    Ok(())
}
