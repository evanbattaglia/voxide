# [voxide](https://github.com/evanbattaglia/voxide)

voxide is a command-line utility which finds files in a variety of customizable ways and opens them in your editor or another program.

The name "voxide" is an homage to the spirit of minimizing keystrokes in the console, exemplified by tools such as [zoxide](https://github.com/ajeetdsouza/zoxide)/[z](https://github.com/rupa/z). I alias voxide to just `v` (mental shortening for 'vim').

## Modes
The central idea in the config is to define "modes", which are layerable one-letter commands, additive/merged in a similar way as the first argument of the `tar` command. The two pieces of a mode are:
* a `script` to get the files to open:
  * Examples: clipboard, git commit files, script in PATH, config file in standard location, vim history.
* a `cmd` to run to open the files. (for directories, `dir_cmd` is instead used if present, in quickfix mode, `quickfix_cmd` is used).
  * Examples: straight up 'vim', editor reading a quickfix file (many files with line numbers), open editor in new zellij window, file manager for directories.

There is a default (base) mode defined in your config, and any modes given as letters get merged onto that successively.

For instance, you could set up a `h` mode to choose files from your vim history and a `z` mode to edit a file in a new zellij pane. Then:
* `v hz` will choose a file from your history and open it in a new zellij pane
* `v h` will choose a file from your history and open it in vim (or the editor defined in the default mode in the config)
* `v z` will use the default file-finding mode (in the sample config, the middle-click clipboard if no args given, or files if given) and open it in a new zellij pane
* `v z foo.txt` will use the default mode to open foo.txt in a new zellij pane
* `v '' foo.txt` will use the default mode to open foo.txt
* `v` will use the default mode to open file file in the middle-click clipboard

See the [example config](config/000_EXAMPLE.toml) and [scripts](scripts/) directory for modes and scripts included with voxide.

## Transforms
Before opening the file, voxide applies transforms (regular expression replacements) until it finds an existing file.

Common use cases:
* strip `a/` and `b/` from git diff output (so `a/foo.bar.txt` opens `foo.txt`
* strip `/usr/src/app/....` in backtraces running in Docker to look for the file starting from the current directory

## Prerequisites
* Neovim (`nvim`), or change the config to use your favorite editor
* `fzf` is used in many/most modes
* ruby is needed for several scripts
* (optional) `yazi` or another file manager for opening directories
* (optional) `nvr` if using any of the remote vim modes (useful if using neovim terminals)
* (optional) `zellij` users can make use of several modes

If using Nix, these are included as dependencies in the nix derivation so you don't need to install them separately.

## Installation
* Build from source: checkout this repo, then run `cargo build --release`
* Nix: it can be tried without installing with `nix run github:evanbattaglia/voxide`

You will need a configuration and scripts to get started. Run `voxide --init` to populate the configuration directory (`~/.config/voxide`) and scripts directory (`~/.local/share/voxide/scripts/`) with defaults / examples compiled into the binary. From there, you can customize the configurations and/or scripts.

* Configs are read from `~/.config/voxide/*.toml` and merged, in sorted order. 
* Scripts are made available by prepending `~/.local/share/voxide/scripts` and `~/.config/voxide/scripts/` (with the latter having priority) to the PATH. I suggest leaving the shipped scripts in `~/.local/voxide/scripts/` and augmenting/overriding them with your own scripts in `~/.config/voxide/scripts/`.

## Usage examples
These assume the default example config and scripts, and assume you have aliased/symlinked `v` to `voxide`

### Most generally useful -- modes to find a file (`script`s)
```
# Open file from selected text -- middle-click, "primary" clipboard in Linux:
v

# Same, but with debugging output
RUST_LOG=debug v

# Open file referenced by (Ctrl-C/Ctrl-V, `XA_CLIPBOARD`) clipboard:
v k

# Choose a file from among changed files in git HEAD, index, and local changes
v g

# Choose a file from among changed files in git HEAD^^
v g @^^

# Choose a file from among changed files in git HEAD/index/local (alternative, 'numbered' script)
v j

# Open ~/.gitconfig in editor
v c git

# Open ~/.config/voxide in file manager (e.g. yazi, nnn):
v c voxide

# Open `~/.config/fish/**/*variables*` (choose with fzf)
v c fish variables

# Open ~/.config/fish/functions/foo.fish:
v w foo

# Open /usr/bin/bar:
v w bar

# Choose from lines that contain 'foo' (search using ag) and open in quickfix
v a foo

# Find file by acronym: open `**/bulk_column_updater.*`:
v m bcu

# Find file by acronym: open `app/controllers/developer_keys_controller.rb`:
v m APdkcR

# Choose from vim history _filtered to files under current directory_
v h
```

### Zellij
All but the first are modes changing the `cmd` (command to run) so can be combined with the above `script` modes.

```
# Choose from files printed in the current zellij terminal screen (pane)
v s

# Open from clipboard in a new zellij pane
v z

# Open foo.txt in a floatring zellij pane
v Z too.txt

# Open ~/bin/myscript in a floating zellij pane
v Zw myscript

# Open ~/.aws/config in a new zellij pane
v cz aws
```

### Neovim windows
```
# (When run in a neovim terminal) open file (from clipboard) in current neovim window (requires nvr)
v rk

# Open ~/.gitconfing in last (previous) neovim window (requires nvr)
v cl git

# Open foo.txt in a neovim split window (requires nvr)
v S foo.txt

# Open foo.txt in a neovim vertical split window (requires nvr)
v V foo.txt
```

### Utility / piping
```
# Echo primary clipboard contents
v -

# (fish) change directory to fish config directory:
cd (v -c fish)

# change directory to ~/dev/abc ("-" = echo, "d" = ~/dev directory)
cd $(v -d abc)

# Take backtrace from clipboard and transform each path (e.g. remove /usr/src/app) -- 'q' is quickfix output mode
v kq

# Open src/foo.rs (run transforms on stdin):
echo /usr/src/app/src/foo.rs | v i

# Run transforms and echo result
echo /usr/src/app/src/foo.rs:32 | v -i
```

### Rails
```
# Search by rails route:
v R

# Search by rails model:
v r m

# Open by rails model -- acronym, e.g. open `app/models/api_key.rb`:
v r m ak
```

### TODO
```
# Load quickfix from gerrit comments (using internal tool `grr` -- TODO open-source)
v G
```
