use std::{
    path::Path,
    process::Command
};
/// Calls std::process:Command.new(), but fixes issue on Termux where running scripts with shebangs
/// don't work (because /usr/bin/env doesn't exist) by wrapping script in: sh -c '"$0" "$@"'
pub fn std_process_command_new(command: &Path) -> Command {
    use std::env::consts::OS;
    if OS == "linux" && ! Path::exists(Path::new("/usr/bin/env")) {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("\"$0\" \"$@\"").arg(command);
        cmd
    } else {
        Command::new(command)
    }
}

