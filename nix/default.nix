{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "voxide";
  # TODO: since all of the buildInputs are actually optional depending on
  # your config, I should probably make a way to install voxide
  # without all of them.
  buildInputs = with pkgs; [
    # Used in most example scripts/cmds, so practically required:
    fzf
    bash

    # Required for several scripts:
    ruby
    git

    # Editors/programs; can be customized to your tastes in config
    neovim
    yazi

    # Highly recommended for users of zellij:
    zellij
    # Highly recommended for users of terminals in neovim:
    neovim-remote
    # Recommended for users of ag -- another option is ripgrep, just replace ag with rg in voxide config:
    silver-searcher 
  ];

  version = "0.1.0"; # should match what's in Cargo.toml
  cargoLock.lockFile = ../Cargo.lock;
  src = pkgs.lib.cleanSource ../.;
}

