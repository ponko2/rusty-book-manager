{
  description = "vscode-remote-try-rust";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    inputs@{ fenix, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, system, ... }:
        let
          inherit
            (import (fetchTarball {
              name = "sqlx-cli-0.7.3";
              url = "https://github.com/NixOS/nixpkgs/archive/336eda0d07dc5e2be1f923990ad9fdb6bc8e28e3.tar.gz";
              sha256 = "0v8vnmgw7cifsp5irib1wkc0bpxzqcarlv8mdybk6dck5m7p10lr";
            }) { inherit system; })
            sqlx-cli
            ;
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              fenix.overlays.default
            ];
          };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              (pkgs.fenix.fromToolchainFile {
                file = ./rust-toolchain.toml;
                sha256 = "sqSWJDUxc+zaz1nBWMAJKTAGBuGWP25GCftIOlCEAtA=";
              })
              cargo-machete
              cargo-make
              cargo-nextest
              editorconfig-checker
              nodejs
              postgresql
              redis
              rust-analyzer
              sqlx-cli
              yamllint
            ];
          };
          formatter = pkgs.nixfmt-tree;
        };
    };
}
